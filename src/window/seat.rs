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
            Capability::Keyboard if self.keyboard.is_none() => {
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
                self.keyboard = Some(wl_keyboard);
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
            if let Some(k) = self.keyboard.take() {
                k.release();
            }
        }
    }

    fn new_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: WlSeat) {}

    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: WlSeat) {}
}
