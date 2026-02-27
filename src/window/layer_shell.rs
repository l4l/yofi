use sctk::shell::wlr_layer::{
    KeyboardInteractivity, LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
};

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
        qh: &sctk::reexports::client::QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let (w, h) = configure.new_size;
        let (cw, ch) = self.content_size();
        self.width = if w > 0 { w } else { cw };
        self.height = if h > 0 { h } else { ch };
        layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);

        if !self.configured_surface {
            self.configured_surface = true;
            self.draw(qh);
        }
    }
}
