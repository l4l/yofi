use sctk::{
    delegate_keyboard,
    reexports::client::{
        protocol::{wl_keyboard, wl_surface},
        ConnectionHandle, QueueHandle,
    },
    seat::keyboard::{KeyEvent, KeyboardHandler, Modifiers},
    shell::xdg::{
        window::{Window, WindowConfigure, WindowHandler, XdgWindowState},
        XdgShellHandler, XdgShellState,
    },
};

use super::WindowState;
use crate::input::KeyPress;

delegate_keyboard!(WindowState);

impl KeyboardHandler for WindowState {
    fn enter(
        &mut self,
        _: &mut ConnectionHandle,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        keysyms: &[u32],
    ) {
    }

    fn leave(
        &mut self,
        _: &mut ConnectionHandle,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
    }

    fn press_key(
        &mut self,
        _conn: &mut ConnectionHandle,
        _qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        self.process_event(KeyPress {
            keysym: event.keysym,
            sym: event.utf8.and_then(|s| s.chars().next()),
            ctrl: self.modifiers.ctrl,
            shift: self.modifiers.shift,
        });
        self.need_redraw = true;
    }

    fn release_key(
        &mut self,
        _: &mut ConnectionHandle,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
    }

    fn update_modifiers(
        &mut self,
        _: &mut ConnectionHandle,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _serial: u32,
        modifiers: Modifiers,
    ) {
        self.modifiers = modifiers;
    }
}
