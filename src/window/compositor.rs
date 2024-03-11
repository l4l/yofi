use anyhow::Context;
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
        let old_scale = self.scale;
        self.scale = new_factor.try_into().expect("invalid surface scale factor");
        if old_scale != self.scale {
            let size = (4 * self.width() * self.height())
                .try_into()
                .expect("pixel buffer overflow");
            if let Err(err) = self
                .pool
                .resize(size)
                .with_context(|| format!("on pool resize to {size}"))
            {
                self.error = Some(err);
            }
        }
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
