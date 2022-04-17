use sctk::{
    delegate_xdg_shell, delegate_xdg_window,
    reexports::{
        client::{ConnectionHandle, QueueHandle},
        protocols::xdg_shell::client::xdg_surface,
    },
    shell::xdg::{
        window::{Window, WindowConfigure, WindowHandler, XdgWindowState},
        XdgShellHandler, XdgShellState,
    },
};

use super::{RenderKind, WindowState};

delegate_xdg_shell!(WindowState);

impl XdgShellHandler for WindowState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState<Self> {
        match &mut self.protocols.render_kind {
            RenderKind::Layer { .. } => panic!("unexpected RenderKind"),
            RenderKind::Xdg {
                xdg_shell_state, ..
            } => xdg_shell_state,
        }
    }
}

delegate_xdg_window!(WindowState);

impl WindowHandler for WindowState {
    fn xdg_window_state(&mut self) -> &mut XdgWindowState {
        match &mut self.protocols.render_kind {
            RenderKind::Layer { .. } => panic!("unexpected RenderKind"),
            RenderKind::Xdg {
                xdg_window_state, ..
            } => xdg_window_state,
        }
    }

    fn request_close(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        window: &Window,
    ) {
        self.running = false;
    }

    fn configure(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        window: &Window,
        configure: WindowConfigure,
        serial: u32,
    ) {
        if self.initial_configure {
            self.initial_configure = false;
            self.draw(conn, qh)
        }
    }
}
