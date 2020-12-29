use std::cell::Cell;
use std::convert::TryInto;
use std::rc::Rc;

use sctk::{
    environment::Environment,
    reexports::{
        client::protocol::{wl_shm, wl_surface},
        client::Main,
        protocols::wlr::unstable::layer_shell::v1::client::{
            zwlr_layer_shell_v1, zwlr_layer_surface_v1,
        },
    },
    shm::DoubleMemPool,
};

use crate::draw::{DrawTarget, Drawable, Point, Space};

pub enum EventStatus {
    Finished,
    ShouldRedraw,
    Idle,
}

pub struct Params {
    pub width: u32,
    pub height: u32,
    pub window_offsets: Option<(i32, i32)>,
    pub scale: Option<u16>,
}

pub struct Surface {
    surface: wl_surface::WlSurface,
    layer_surface: Main<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    next_render_event: Rc<Cell<Option<RenderEvent>>>,
    pools: DoubleMemPool,
    scale: Option<u16>,
    dimensions: (u32, u32),
}

impl Surface {
    pub fn new(env: &Environment<super::Env>, params: Params) -> Self {
        let pools = env
            .create_double_pool(|_| {})
            .expect("Failed to create a memory pool!");

        let surface = env.create_surface().detach();
        let layer_shell = env.require_global::<zwlr_layer_shell_v1::ZwlrLayerShellV1>();

        let layer_surface = layer_shell.get_layer_surface(
            &surface,
            None,
            zwlr_layer_shell_v1::Layer::Top,
            crate::prog_name!().to_owned(),
        );

        let scale = params.scale;
        let width = params.width;
        let height = params.height;

        if let Some((top_offset, left_offset)) = params.window_offsets {
            let mut anchor = zwlr_layer_surface_v1::Anchor::Left;
            anchor.insert(zwlr_layer_surface_v1::Anchor::Top);
            layer_surface.set_anchor(anchor);
            layer_surface.set_margin(top_offset, 0, 0, left_offset);
        }
        layer_surface.set_size(width, height);
        layer_surface.set_keyboard_interactivity(1);

        let next_render_event = Rc::new(Cell::new(None::<RenderEvent>));
        let next_render_event_handle = Rc::clone(&next_render_event);

        layer_surface.quick_assign(move |layer_surface, event, _| {
            if matches!(event, zwlr_layer_surface_v1::Event::Closed) {
                next_render_event_handle.set(Some(RenderEvent::Closed));
                return;
            }

            if let zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } = event
            {
                if !matches!(next_render_event_handle.get(), Some(RenderEvent::Closed)) {
                    next_render_event_handle.set(Some(RenderEvent::Configure { width, height }));
                    layer_surface.ack_configure(serial);
                    return;
                }
            }
        });

        // Commit so that the server will send a configure event
        surface.commit();

        Self {
            surface,
            layer_surface,
            next_render_event,
            pools,
            scale,
            dimensions: (width, height),
        }
    }

    /// Handles any events that have occurred since the last call, redrawing if needed.
    /// Returns true if the surface should be dropped.
    pub fn handle_events(&mut self) -> EventStatus {
        match self.next_render_event.take() {
            Some(RenderEvent::Closed) => EventStatus::Finished,
            Some(RenderEvent::Configure { width, height }) => {
                self.dimensions = (width, height);
                EventStatus::ShouldRedraw
            }
            None => EventStatus::Idle,
        }
    }

    pub fn redraw<D>(&mut self, drawables: impl Iterator<Item = D>)
    where
        D: Drawable,
    {
        let scale = self.scale.unwrap_or_else(|| {
            sctk::get_surface_scale_factor(&self.surface)
                .try_into()
                .expect("invalid surface scale factor")
        });
        self.surface.set_buffer_scale(scale.into());

        let pool = if let Some(pool) = self.pools.pool() {
            pool
        } else {
            return;
        };

        let width = self.dimensions.0 * u32::from(scale);
        let height = self.dimensions.1 * u32::from(scale);

        // First make sure the pool is the right size
        pool.resize((4 * width * height) as usize).unwrap();

        {
            let buf: &mut [u8] = &mut pool.mmap();
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
        let buffer = pool.buffer(
            0,
            width as i32,
            height as i32,
            4 * width as i32,
            wl_shm::Format::Argb8888,
        );

        // Attach the buffer to the surface and mark the entire surface as damaged
        self.surface.attach(Some(&buffer), 0, 0);
        self.surface
            .damage_buffer(0, 0, width as i32, height as i32);

        // Finally, commit the surface
        self.surface.commit();
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        self.layer_surface.destroy();
        self.surface.destroy();
    }
}

#[derive(PartialEq, Copy, Clone)]
enum RenderEvent {
    Configure { width: u32, height: u32 },
    Closed,
}
