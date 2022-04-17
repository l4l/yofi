use std::cell::{Cell, RefCell};
use std::convert::TryInto;
use std::rc::Rc;

use sctk::{
    compositor::{CompositorHandler, CompositorState},
    delegate_registry,
    output::{OutputHandler, OutputState},
    reexports::{
        client::{
            protocol::{
                wl_buffer, wl_keyboard::WlKeyboard, wl_output, wl_shm, wl_surface::WlSurface,
            },
            Connection, ConnectionHandle, Dispatch, EventQueue, QueueHandle,
        },
        protocols::{
            wlr::unstable::layer_shell::v1::client::{
                zwlr_layer_shell_v1,
                zwlr_layer_surface_v1::{
                    self, Anchor, Event as ZEvent, KeyboardInteractivity, ZwlrLayerSurfaceV1,
                },
            },
            xdg_shell::client::xdg_surface,
        },
    },
    registry::{ProvidesRegistryState, RegistryState},
    seat::{keyboard::Modifiers, SeatState},
    shell::{
        layer::LayerState,
        xdg::{
            window::{Window, WindowConfigure, WindowHandler, XdgWindowState},
            XdgShellHandler, XdgShellState,
        },
    },
    shm::{pool::raw::RawPool, ShmHandler, ShmState},
};

use crate::{config::Config, draw, input::KeyPress, state};

mod compositor_handler;
mod keyboard_handler;
mod output_handler;
mod seat_handler;
mod shm_handler;
mod xdg_handler;

enum RenderKind {
    Layer {
        layer_state: LayerState,
    },
    Xdg {
        xdg_shell_state: XdgShellState<WindowState>,
        xdg_window_state: XdgWindowState,
        window: Option<Window>,
    },
}

struct ProtocolStates {
    registry: RegistryState,
    compositor_state: CompositorState,
    output_state: OutputState,
    shm_state: ShmState,
    seat_state: SeatState,
    render_kind: RenderKind,
}

pub struct Inner {
    surface: WlSurface,
    scale: u16,
    dimensions: (u32, u32),
    initial_configure: bool,
    need_redraw: bool,
    pool: RefCell<RawPool>,
}

pub struct WindowState {
    protocols: ProtocolStates,
    keyboard: Option<WlKeyboard>,
    window_state: Option<Inner>,
    config: Config,
    state: state::State,
    modifiers: Modifiers,
    running: bool,
}

impl WindowState {
    pub fn new(
        config: Config,
        state: state::State,
        registry: RegistryState,
        conn: &Connection,
        queue: &mut EventQueue<Self>,
    ) -> Self {
        let force_window = config.param::<crate::surface::Params>().force_window;
        let layer_state = LayerState::new();

        assert!(layer_state.is_available());

        let render_kind = if force_window || !layer_state.is_available() {
            RenderKind::Xdg {
                xdg_shell_state: XdgShellState::new(),
                xdg_window_state: XdgWindowState::new(),
                window: None,
            }
        } else {
            RenderKind::Layer { layer_state }
        };

        let mut window = WindowState {
            protocols: ProtocolStates {
                registry,
                compositor_state: CompositorState::new(),
                output_state: OutputState::new(),
                shm_state: ShmState::new(),
                seat_state: SeatState::new(),
                render_kind,
            },
            keyboard: None,
            window_state: None,
            config,
            state,
            modifiers: Default::default(),
            running: true,
        };

        conn.roundtrip();
        queue.blocking_dispatch(&mut window).unwrap();
        queue.blocking_dispatch(&mut window).unwrap();

        window.create_window(&mut conn.handle(), &queue.handle());

        window
    }

    fn create_window(&mut self, mut conn: &mut ConnectionHandle, qh: &QueueHandle<Self>) {
        let params: crate::surface::Params = self.config.param();

        let pool = self
            .protocols
            .shm_state
            .new_raw_pool(
                (params.width * params.height * 4) as usize,
                &mut conn,
                &qh,
                (),
            )
            .expect("Failed to create pool");

        let surface = self
            .protocols
            .compositor_state
            .create_surface(&mut conn, &qh)
            .expect("surface creation");

        let dimensions = (params.width, params.height);

        match &mut self.protocols.render_kind {
            RenderKind::Layer { layer_state } => {
                let mut layer_shell = layer_state.wlr_layer_shell();
                let layer_shell = layer_shell.as_mut().unwrap();
                let layer_surface = layer_shell
                    .get_layer_surface(
                        &mut conn,
                        &surface,
                        None,
                        zwlr_layer_shell_v1::Layer::Top,
                        crate::prog_name!().to_owned(),
                        &qh,
                        (),
                    )
                    .expect("get_layer_surface failed");

                if let Some((top_offset, left_offset)) = params.window_offsets {
                    let mut anchor = Anchor::Left;
                    anchor.insert(Anchor::Top);
                    layer_surface.set_anchor(&mut conn, anchor);
                    layer_surface.set_margin(&mut conn, top_offset, 0, 0, left_offset);
                }

                layer_surface.set_size(&mut conn, dimensions.0, dimensions.1);
                layer_surface
                    .set_keyboard_interactivity(&mut conn, KeyboardInteractivity::Exclusive);

                todo!()
            }
            RenderKind::Xdg {
                xdg_shell_state,
                xdg_window_state,
                window,
            } => {
                *window = Some(
                    Window::builder()
                        .title(crate::prog_name!().to_owned())
                        .min_size(dimensions)
                        .map(
                            &mut conn,
                            &qh,
                            xdg_shell_state,
                            xdg_window_state,
                            surface.clone(),
                        )
                        .expect("window creation"),
                );
            }
        };

        self.window_state = Some(Inner {
            surface,
            dimensions,
            scale: 1,
            initial_configure: true,
            need_redraw: true,
            pool: RefCell::new(pool),
        });
    }

    pub fn draw(&mut self, mut conn: &mut ConnectionHandle, qh: &QueueHandle<Self>) {
        use std::iter::once;

        self.state.process_entries();

        let (tx, rx) = oneshot::channel();

        let background = draw::Widget::background(self.config.param());
        let input_widget = draw::Widget::input_text(self.state.raw_input(), self.config.param());
        let list_view_widget = draw::Widget::list_view(
            self.state.processed_entries(),
            self.state.skip_offset(),
            self.state.selected_item(),
            tx,
            self.config.param(),
        );
        let drawables = once(background)
            .chain(once(input_widget))
            .chain(once(list_view_widget));

        {
            use crate::draw::{DrawTarget, Drawable, Point, Space};

            let scale = self.scale;
            // self.surface.set_buffer_scale(scale.into());

            // let pool = if let Some(pool) = self.pool.pool() {
            //     pool
            // } else {
            //     return;
            // };

            let width = self.dimensions.0 * u32::from(scale);
            let height = self.dimensions.1 * u32::from(scale);

            // First make sure the pool is the right size
            let mut pool = self.pool.borrow_mut();
            pool.resize((4 * width * height) as usize, conn).unwrap();

            {
                let buf = pool.mmap();
                let buf_ptr: *mut u32 = buf.as_mut_ptr() as *mut _;
                let buf: &mut [u32] =
                    unsafe { &mut *std::ptr::slice_from_raw_parts_mut(buf_ptr, buf.len() / 4) };
                let mut dt = DrawTarget::from_buf(width as i32, height as i32, buf);

                let mut space_left = Space {
                    width: width as f32,
                    height: height as f32,
                };
                let mut point = Point::new(0., 0.);

                for d in drawables {
                    let occupied = d.draw(&mut dt, scale, space_left, point);
                    debug_assert!(
                        occupied.width <= space_left.width && occupied.height <= space_left.height
                    );

                    point.y += occupied.height;
                    space_left.height -= occupied.height;
                }
            }

            // Create a new buffer from the pool
            let buffer = pool
                .create_buffer(
                    0,
                    width as i32,
                    height as i32,
                    4 * width as i32,
                    wl_shm::Format::Argb8888,
                    (),
                    &mut conn,
                    qh,
                )
                .expect("create_buffer failed");

            // Attach the buffer to the surface and mark the entire surface as damaged
            self.surface.attach(&mut conn, Some(&buffer), 0, 0);
            self.surface
                .damage_buffer(&mut conn, 0, 0, width as i32, height as i32);

            // if let RenderSurface::Window(ref mut window) = self.surface {
            //     window.refresh();
            // }

            self.surface.commit(conn);
        }

        self.state.update_skip_offset(rx.recv().unwrap());
    }

    pub fn request_redraw(&mut self, conn: &mut ConnectionHandle, qh: &QueueHandle<Self>) {
        self.surface.frame(conn, qh, self.surface.clone());
    }

    pub fn process_event(&mut self, key: KeyPress) -> bool {
        self.state.process_event(key)
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn need_redraw(&self) -> bool {
        self.need_redraw
    }
}

impl std::ops::Deref for WindowState {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        self.window_state.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for WindowState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.window_state.as_mut().unwrap()
    }
}

delegate_registry!(WindowState: [
    CompositorState,
    OutputState,
    ShmState,
    SeatState,
    XdgShellState<WindowState>,
    XdgWindowState,
]);

impl ProvidesRegistryState for WindowState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.protocols.registry
    }
}

// TODO: Pending changes regarding WlBuffer on pools
impl Dispatch<wl_buffer::WlBuffer> for WindowState {
    type UserData = ();

    fn event(
        &mut self,
        _proxy: &wl_buffer::WlBuffer,
        _event: wl_buffer::Event,
        _data: &Self::UserData,
        _conn: &mut ConnectionHandle,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrLayerSurfaceV1> for WindowState {
    type UserData = ();

    fn event(
        &mut self,
        _proxy: &ZwlrLayerSurfaceV1,
        _event: zwlr_layer_surface_v1::Event,
        _data: &Self::UserData,
        _conn: &mut ConnectionHandle,
        _qh: &QueueHandle<Self>,
    ) {
    }
}
