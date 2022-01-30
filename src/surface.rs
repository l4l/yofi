use std::cell::Cell;
use std::convert::TryInto;
use std::rc::Rc;

use sctk::{
    environment::Environment,
    reexports::{
        client::protocol::{wl_shm, wl_surface::WlSurface},
        client::Main,
        protocols::wlr::unstable::layer_shell::v1::client::{
            zwlr_layer_shell_v1,
            zwlr_layer_surface_v1::{
                Anchor, Event as ZEvent, KeyboardInteractivity, ZwlrLayerSurfaceV1,
            },
        },
    },
    shm::DoubleMemPool,
    window::{Event as WEvent, FallbackFrame, Window},
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
    pub force_window: bool,
    pub window_offsets: Option<(i32, i32)>,
    pub scale: Option<u16>,
}

enum RenderSurface {
    LayerShell {
        surface: WlSurface,
        layer_surface: Main<ZwlrLayerSurfaceV1>,
    },
    Window(Window<FallbackFrame>),
}

impl Drop for RenderSurface {
    fn drop(&mut self) {
        match self {
            RenderSurface::LayerShell {
                layer_surface,
                surface,
            } => {
                layer_surface.destroy();
                surface.destroy();
            }
            RenderSurface::Window(window) => window.surface().destroy(),
        }
    }
}

impl std::ops::Deref for RenderSurface {
    type Target = WlSurface;

    fn deref(&self) -> &Self::Target {
        match &self {
            RenderSurface::LayerShell { surface, .. } => surface,
            RenderSurface::Window(s) => s.surface(),
        }
    }
}

pub struct Surface {
    surface: RenderSurface,
    next_render_event: Rc<Cell<Option<RenderEvent>>>,
    pools: DoubleMemPool,
    scale: Rc<Cell<u16>>,
    dimensions: (u32, u32),
}

impl Surface {
    pub fn new(env: &Environment<super::Env>, params: Params) -> Self {
        let pools = env
            .create_double_pool(|_| {})
            .expect("Failed to create a memory pool!");

        let next_render_event = Rc::new(Cell::new(None::<RenderEvent>));

        let scale = Rc::new(Cell::new(params.scale.unwrap_or(1)));

        let scale1 = Rc::clone(&scale);
        let next_render_event_handle = Rc::clone(&next_render_event);
        let surface = env
            .create_surface_with_scale_callback(move |scale, _, _| {
                scale1.set(scale.try_into().expect("invalid surface scale factor"));
                next_render_event_handle.set(Some(RenderEvent::ScaleUpdate));
            })
            .detach();

        let width = params.width;
        let height = params.height;

        let next_render_event_handle = Rc::clone(&next_render_event);
        let surface = if let Some(layer_shell) = env
            .get_global::<zwlr_layer_shell_v1::ZwlrLayerShellV1>()
            .filter(|_| !params.force_window)
        {
            let layer_surface = layer_shell.get_layer_surface(
                &surface,
                None,
                zwlr_layer_shell_v1::Layer::Top,
                crate::prog_name!().to_owned(),
            );

            if let Some((top_offset, left_offset)) = params.window_offsets {
                let mut anchor = Anchor::Left;
                anchor.insert(Anchor::Top);
                layer_surface.set_anchor(anchor);
                layer_surface.set_margin(top_offset, 0, 0, left_offset);
            }
            layer_surface.set_size(width, height);
            layer_surface.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);

            layer_surface.quick_assign(move |layer_surface, event, _| match event {
                ZEvent::Closed => next_render_event_handle.set(Some(RenderEvent::Closed)),
                _ if matches!(next_render_event_handle.get(), Some(RenderEvent::Closed)) => {}
                ZEvent::Configure {
                    serial,
                    width,
                    height,
                } => {
                    next_render_event_handle.set(Some(RenderEvent::Resized { width, height }));
                    layer_surface.ack_configure(serial);
                }
                _ => {}
            });

            // Commit so that the server will send a configure event
            surface.commit();

            RenderSurface::LayerShell {
                surface,
                layer_surface,
            }
        } else {
            let mut window = env
                .create_window(
                    surface,
                    None,
                    (width, height),
                    move |event, _| match event {
                        WEvent::Close => {
                            next_render_event_handle.set(Some(RenderEvent::Closed));
                        }
                        _ if next_render_event_handle.get().is_some() => {}
                        WEvent::Configure {
                            new_size: Some((width, height)),
                            ..
                        } => {
                            next_render_event_handle
                                .set(Some(RenderEvent::Resized { width, height }));
                        }
                        WEvent::Configure { .. } | WEvent::Refresh => {
                            next_render_event_handle.set(Some(RenderEvent::Refresh));
                        }
                    },
                )
                .expect("failed to create a window");

            window.set_title(crate::prog_name!().to_owned());
            window.unset_fullscreen();
            window.resize(width, height);
            RenderSurface::Window(window)
        };

        Self {
            surface,
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
            Some(RenderEvent::Resized { width, height }) => {
                self.dimensions = (width, height);
                EventStatus::ShouldRedraw
            }
            Some(RenderEvent::Refresh) | Some(RenderEvent::ScaleUpdate) => {
                EventStatus::ShouldRedraw
            }
            None => EventStatus::Idle,
        }
    }

    pub fn redraw<D>(&mut self, drawables: impl Iterator<Item = D>)
    where
        D: Drawable,
    {
        let scale = self.scale.get();
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
            let buf: &mut [u8] = pool.mmap();
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

        if let RenderSurface::Window(ref mut window) = self.surface {
            window.refresh();
        }

        // Finally, commit the surface
        self.surface.commit();
    }
}

#[derive(PartialEq, Copy, Clone)]
enum RenderEvent {
    Resized { width: u32, height: u32 },
    ScaleUpdate,
    Refresh,
    Closed,
}
