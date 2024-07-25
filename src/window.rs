use anyhow::Context;
use sctk::{
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_shm, delegate_xdg_shell, delegate_xdg_window,
    output::OutputState,
    reexports::client::{
        protocol::{wl_keyboard::WlKeyboard, wl_pointer::WlPointer, wl_surface::WlSurface},
        *,
    },
    reexports::{
        calloop::{EventLoop, LoopHandle},
        calloop_wayland_source::WaylandSource,
    },
    registry::RegistryState,
    seat::SeatState,
    shell::{
        wlr_layer,
        xdg::{self, window as xdg_win},
        WaylandSurface,
    },
    shm::{
        slot::{Buffer, SlotPool},
        Shm,
    },
};

use crate::state::State;
pub use pointer::Params as PointerParams;

mod compositor;
mod keyboard;
mod layer_shell;
mod output;
mod pointer;
mod registry;
mod seat;
mod shm;
mod xdg_window;

pub struct Params {
    pub width: u32,
    pub height: u32,
    pub force_window: bool,
    pub window_offsets: Option<(i32, i32)>,
    pub scale: Option<u16>,
}

pub struct Window {
    config: crate::config::Config,
    state: State,

    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,

    buffer: Option<Buffer>,
    pool: SlotPool,
    shm: Shm,
    surface: RenderSurface,
    configured_surface: bool,
    width: u32,
    height: u32,
    scale: u16,

    input: InputSource,
    key_modifiers: sctk::seat::keyboard::Modifiers,
    wheel_scroll_pending: f64,

    loop_handle: LoopHandle<'static, Window>,
    exit: bool,

    error: Option<anyhow::Error>,
}

struct InputSource {
    keyboard: Option<WlKeyboard>,
    pointer: Option<WlPointer>,
}

enum RenderSurface {
    Xdg(xdg_win::Window),
    LayerShell(wlr_layer::LayerSurface),
}

impl std::ops::Deref for RenderSurface {
    type Target = WlSurface;

    fn deref(&self) -> &Self::Target {
        match &self {
            RenderSurface::LayerShell(s) => s.wl_surface(),
            RenderSurface::Xdg(s) => s.wl_surface(),
        }
    }
}

impl Window {
    pub fn new(
        config: crate::config::Config,
        state: State,
    ) -> anyhow::Result<(Self, EventLoop<'static, Self>)> {
        let conn = Connection::connect_to_env()?;

        let (globals, event_queue) = globals::registry_queue_init(&conn)?;
        let qh = event_queue.handle();
        let event_loop: EventLoop<Window> =
            EventLoop::try_new().context("failed to initialize the event loop")?;
        let loop_handle = event_loop.handle();
        WaylandSource::new(conn.clone(), event_queue).insert(loop_handle)?;

        let params: Params = config.param();
        let scale = params.scale.unwrap_or(1);
        let (width, height) = (params.width, params.height);
        let shm = Shm::bind(&globals, &qh).context("wl_shm is not available")?;
        let pool = SlotPool::new(
            (4 * width * u32::from(scale) * height * u32::from(scale)) as usize,
            &shm,
        )
        .context("Failed to create a memory pool!")?;

        let compositor = sctk::compositor::CompositorState::bind(&globals, &qh)
            .context("wl_compositor is not available")?;
        let surface = compositor.create_surface(&qh);
        let surface = if let Some(layer_shell) = wlr_layer::LayerShell::bind(&globals, &qh)
            .ok()
            .filter(|_| !params.force_window)
        {
            let layer = layer_shell.create_layer_surface(
                &qh,
                surface,
                wlr_layer::Layer::Top,
                Some(crate::prog_name!()),
                None,
            );

            if let Some((top_offset, left_offset)) = params.window_offsets {
                layer.set_anchor(wlr_layer::Anchor::LEFT | wlr_layer::Anchor::TOP);
                layer.set_margin(top_offset, 0, 0, left_offset);
            }
            layer.set_size(width, height);
            layer.set_keyboard_interactivity(wlr_layer::KeyboardInteractivity::Exclusive);

            layer.commit();

            RenderSurface::LayerShell(layer)
        } else {
            let xdg_shell =
                xdg::XdgShell::bind(&globals, &qh).context("xdg shell is not available")?;
            let window = xdg_shell.create_window(surface, xdg_win::WindowDecorations::None, &qh);
            window.set_title(crate::prog_name!());
            window.set_min_size(Some((width, height)));
            window.unset_fullscreen();
            RenderSurface::Xdg(window)
        };

        Ok((
            Self {
                config,
                state,
                registry_state: RegistryState::new(&globals),
                seat_state: SeatState::new(&globals, &qh),
                output_state: OutputState::new(&globals, &qh),
                buffer: None,
                pool,
                shm,
                surface,
                configured_surface: false,
                width,
                height,
                scale,
                input: InputSource {
                    keyboard: None,
                    pointer: None,
                },
                key_modifiers: Default::default(),
                wheel_scroll_pending: 0.0,
                loop_handle: event_loop.handle(),
                exit: false,
                error: None,
            },
            event_loop,
        ))
    }

    fn width(&self) -> u32 {
        self.width * u32::from(self.scale)
    }

    fn height(&self) -> u32 {
        self.height * u32::from(self.scale)
    }

    pub fn draw(&mut self, qh: &QueueHandle<Self>) {
        let width = self.width().try_into().expect("width overflow");
        let height = self.height().try_into().expect("height overflow");
        let stride = width * 4;
        self.surface.set_buffer_scale(self.scale.into());

        if self
            .buffer
            .as_ref()
            .filter(|b| b.height() != height || b.stride() != stride)
            .is_some()
        {
            self.buffer.take();
        }

        const FORMAT: protocol::wl_shm::Format = protocol::wl_shm::Format::Argb8888;
        let mut buffer = self.buffer.take().unwrap_or_else(|| {
            self.pool
                .create_buffer(width, height, stride, FORMAT)
                .expect("create buffer")
                .0
        });

        let canvas = match self.pool.canvas(&buffer) {
            Some(canvas) => canvas,
            None => {
                // This should be rare, but if the compositor has not released the previous
                // buffer, we need double-buffering.
                let (second_buffer, canvas) = self
                    .pool
                    .create_buffer(width, height, stride, FORMAT)
                    .expect("create buffer");
                buffer = second_buffer;
                canvas
            }
        };

        use crate::draw::*;
        let mut dt = {
            #[allow(clippy::needless_lifetimes)]
            fn transmute_slice<'a>(a: &'a mut [u8]) -> &'a mut [u32] {
                assert_eq!(a.as_ptr().align_offset(std::mem::align_of::<u32>()), 0);
                assert_eq!(a.len() % 4, 0);
                // Safety:
                // - (asserted) it's well-aligned for *u32
                // - canvas is a valid mut slice
                // - it does not alias with original reference as it's been shadowed
                // - len does not overflow as it reduced from the valid len
                // - lifetimes are same
                unsafe {
                    &mut *std::ptr::slice_from_raw_parts_mut(a.as_mut_ptr().cast(), a.len() / 4)
                }
            }
            let canvas = transmute_slice(canvas);
            DrawTarget::from_backing(width, height, canvas)
        };

        let mut space_left = Space {
            width: width as f32,
            height: height as f32,
        };
        let mut point = Point::new(0., 0.);

        let mut drawables = crate::draw::make_drawables(&self.config, &mut self.state);
        while let Some(d) = drawables.borrowed_next() {
            let occupied = d.draw(&mut dt, self.scale, space_left, point);
            debug_assert!(
                occupied.width <= space_left.width && occupied.height <= space_left.height
            );

            point.y += occupied.height;
            space_left.height -= occupied.height;
        }

        self.surface.damage_buffer(0, 0, width, height);
        self.surface.frame(qh, self.surface.clone());
        buffer.attach_to(&self.surface).expect("buffer attach");
        self.buffer = Some(buffer);
        self.surface.commit();
    }

    pub fn asked_exit(&self) -> bool {
        self.exit
    }

    pub fn take_error(&mut self) -> Option<anyhow::Error> {
        self.error.take()
    }
}

delegate_compositor!(Window);
delegate_output!(Window);
delegate_shm!(Window);
delegate_seat!(Window);
delegate_keyboard!(Window);
delegate_xdg_shell!(Window);
delegate_layer!(Window);
delegate_xdg_window!(Window);
delegate_registry!(Window);
delegate_pointer!(Window);
