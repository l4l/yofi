use sctk::{
    reexports::client::*,
    shell::xdg::window::{self, WindowConfigure, WindowHandler},
};

use super::Window;

impl WindowHandler for Window {
    fn request_close(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _window: &window::Window,
    ) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        window: &window::Window,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        let (w, h) = configure.new_size;
        let new_w = w.map(|w| w.get()).unwrap_or(self.width);
        let new_h = h.map(|h| h.get()).unwrap_or(self.height);
        self.update_size(new_w, new_h);

        if !self.configured_surface {
            window.set_title(crate::prog_name!().to_owned());
            window.unset_fullscreen();
            self.configured_surface = true;
            self.dirty = true;
            self.draw();
        }
    }
}
