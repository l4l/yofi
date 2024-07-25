use sctk::{
    reexports::client::{protocol::wl_seat::WlSeat, *},
    seat::{Capability, SeatHandler, SeatState},
};

use super::Window;

impl SeatHandler for Window {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: WlSeat,
        capability: Capability,
    ) {
        match capability {
            Capability::Keyboard if self.input.keyboard.is_none() => {
                let wl_keyboard = match self.seat_state.get_keyboard_with_repeat(
                    qh,
                    &seat,
                    None,
                    self.loop_handle.clone(),
                    Box::new(|_state, _wl_kbd, _event| {}),
                ) {
                    Ok(k) => k,
                    Err(err) => {
                        self.error = Some(err.into());
                        return;
                    }
                };
                self.input.keyboard = Some(wl_keyboard);
            }
            Capability::Pointer if self.input.pointer.is_none() => {
                if let Ok(p) = self.seat_state.get_pointer(qh, &seat) {
                    self.input.pointer = Some(p);
                }
            }
            _ => {}
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: WlSeat,
        capability: Capability,
    ) {
        if let Capability::Keyboard = capability {
            if let Some(k) = self.input.keyboard.take() {
                k.release();
            }
        }
    }

    fn new_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: WlSeat) {}

    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: WlSeat) {}
}
