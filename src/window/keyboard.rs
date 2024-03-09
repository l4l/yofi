use sctk::{
    reexports::client::{
        protocol::{wl_keyboard::WlKeyboard, wl_surface::WlSurface},
        *,
    },
    seat::keyboard::{KeyboardHandler, Modifiers},
};

use super::Window;

impl KeyboardHandler for Window {
    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &WlKeyboard,
        _serial: u32,
        event: sctk::seat::keyboard::KeyEvent,
    ) {
        use sctk::seat::keyboard::Keysym;
        type M = Modifiers;
        match (event.keysym, self.key_modifiers) {
            (Keysym::Escape, _) | (Keysym::c, M { ctrl: true, .. }) => {
                self.exit = true;
            }
            (Keysym::Down, _)
            | (Keysym::j, M { ctrl: true, .. })
            | (Keysym::Tab, M { shift: false, .. })
            | (Keysym::ISO_Left_Tab, M { shift: false, .. }) => self.state.next_item(),
            (Keysym::Up, _)
            | (Keysym::k, M { ctrl: true, .. })
            | (Keysym::Tab, M { shift: true, .. })
            | (Keysym::ISO_Left_Tab, M { shift: true, .. }) => self.state.prev_item(),
            (Keysym::Left, _) => self.state.prev_subitem(),
            (Keysym::Right, _) => self.state.next_subitem(),
            (Keysym::Return, M { ctrl, .. }) | (Keysym::ISO_Enter, M { ctrl, .. }) => {
                if let Err(err) = self.state.eval_input(ctrl) {
                    self.error = Some(err);
                }
            }
            (Keysym::BackSpace, M { ctrl: false, .. }) => self.state.remove_input_char(),
            (Keysym::BackSpace, M { ctrl: true, .. }) | (Keysym::w, M { ctrl: true, .. }) => {
                self.state.remove_input_word()
            }
            (Keysym::bracketright, M { ctrl: true, .. }) => self.state.clear_input(),
            // XXX: use if-let guards once stabilized
            (_, M { ctrl: false, .. }) if event.utf8.is_some() => {
                self.state.append_to_input(event.utf8.as_ref().unwrap())
            }
            (k, m) => log::debug!(
                "unhandled sym: {:?} (ctrl: {}, shift: {})",
                k,
                m.ctrl,
                m.shift
            ),
        }
    }

    fn update_modifiers(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &WlKeyboard,
        _serial: u32,
        modifiers: sctk::seat::keyboard::Modifiers,
    ) {
        self.key_modifiers = modifiers;
    }

    fn release_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &WlKeyboard,
        _serial: u32,
        _event: sctk::seat::keyboard::KeyEvent,
    ) {
    }

    fn enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &WlKeyboard,
        _surface: &WlSurface,
        _serial: u32,
        _raw: &[u32],
        _keysyms: &[sctk::seat::keyboard::Keysym],
    ) {
    }

    fn leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &WlKeyboard,
        _surface: &WlSurface,
        _serial: u32,
    ) {
    }
}
