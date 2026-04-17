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
            self.state.mark_dirty();
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
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        time: u32,
    ) {
        self.state.update_marquee(time);
        // On throttle-skipped ticks keep the frame-callback chain alive with
        // a bare re-request + commit; Wayland ties callbacks to commits.
        if self.state.is_marquee_animating() && !self.state.is_dirty() {
            self.surface.frame(&self.qh, (*self.surface).clone());
            self.surface.commit();
        }
    }
}
