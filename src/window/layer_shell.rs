use sctk::shell::wlr_layer::{LayerShellHandler, LayerSurface, LayerSurfaceConfigure};

use super::Window;

impl LayerShellHandler for Window {
    fn closed(
        &mut self,
        _conn: &sctk::reexports::client::Connection,
        _qh: &sctk::reexports::client::QueueHandle<Self>,
        _layer: &LayerSurface,
    ) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &sctk::reexports::client::Connection,
        _qh: &sctk::reexports::client::QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let (w, h) = configure.new_size;
        let (cw, ch) = self.content_size();
        let new_w = if w > 0 { w } else { cw };
        let new_h = if h > 0 { h } else { ch };
        self.update_size(new_w, new_h);

        if !self.configured_surface {
            self.configured_surface = true;
            self.dirty = true;
            self.draw();
        }
    }
}
