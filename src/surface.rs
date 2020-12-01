use std::cell::Cell;
use std::io::{Seek, SeekFrom, Write};
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

pub struct Surface {
    surface: wl_surface::WlSurface,
    layer_surface: Main<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    next_render_event: Rc<Cell<Option<RenderEvent>>>,
    pools: DoubleMemPool,
    dimensions: (u32, u32),
}

impl Surface {
    pub fn new(env: &Environment<super::Env>) -> Self {
        let pools = env
            .create_double_pool(|_| {})
            .expect("Failed to create a memory pool!");

        let surface = env.create_surface().detach();
        let layer_shell = env.require_global::<zwlr_layer_shell_v1::ZwlrLayerShellV1>();

        let layer_surface = layer_shell.get_layer_surface(
            &surface,
            None,
            zwlr_layer_shell_v1::Layer::Top,
            "yofi".to_owned(),
        );

        let width = 400;
        let height = 512;

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
        let pool = if let Some(pool) = self.pools.pool() {
            pool
        } else {
            return;
        };

        let width = self.dimensions.0;
        let height = self.dimensions.1;

        // First make sure the pool is the right size
        pool.resize((4 * width * height) as usize).unwrap();

        let mut dt = DrawTarget::new(width as i32, height as i32);

        dt.clear(raqote::SolidSource::from_unpremultiplied_argb(
            0x77, 0xff, 0xff, 0xff,
        ));

        let mut space_left = Space {
            width: width as f32,
            height: height as f32,
        };
        let mut point = Point::new(0., 0.);

        for d in drawables {
            let occupied = d.draw(&mut dt, space_left, point);
            debug_assert!(
                occupied.width <= space_left.width && occupied.height <= space_left.height
            );

            point.y += occupied.height;
            space_left.height -= occupied.height;
        }

        pool.seek(SeekFrom::Start(0)).unwrap();
        let buf = dt.get_data();
        let buf =
            unsafe { &*std::ptr::slice_from_raw_parts(buf.as_ptr() as *const u8, buf.len() * 4) };
        pool.write(&buf).unwrap();

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
