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
        qh: &QueueHandle<Self>,
        window: &window::Window,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        let (w, h) = configure.new_size;
        self.width = w.map(|w| w.get()).unwrap_or(self.width);
        self.height = h.map(|h| h.get()).unwrap_or(self.height);

        window.set_title(crate::prog_name!().to_owned());
        window.unset_fullscreen();

        if !self.configured_surface {
            self.configured_surface = true;
            self.draw(qh);
        }
    }
}
