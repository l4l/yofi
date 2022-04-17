use sctk::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor,
    reexports::client::{protocol::wl_surface, ConnectionHandle, QueueHandle},
};

use super::WindowState;

delegate_compositor!(WindowState);

impl CompositorHandler for WindowState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.protocols.compositor_state
    }

    fn scale_factor_changed(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        new_factor: i32,
    ) {
        use std::convert::TryInto;

        if let Ok(scale) = new_factor.try_into() {
            self.scale = scale;
            self.need_redraw = true;
        }
    }

    fn frame(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.draw(conn, qh)
    }
}
