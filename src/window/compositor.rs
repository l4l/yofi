use sctk::{
    compositor::CompositorHandler,
    reexports::client::{
        protocol::{wl_output, wl_surface::WlSurface},
        *,
    },
};

use super::Window;

impl CompositorHandler for Window {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        new_factor: i32,
    ) {
        self.scale = new_factor.try_into().expect("invalid surface scale factor");
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _new_transform: wl_output::Transform,
    ) {
        log::warn!("unexpected transform_changed")
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _time: u32,
    ) {
        self.draw(qh);
    }
}
