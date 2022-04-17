use sctk::{
    delegate_seat,
    reexports::client::{protocol::wl_seat, ConnectionHandle, QueueHandle},
    seat::{Capability, SeatHandler, SeatState},
};

use super::WindowState;

delegate_seat!(WindowState);

impl SeatHandler for WindowState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.protocols.seat_state
    }

    fn new_seat(&mut self, _: &mut ConnectionHandle, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if matches!(capability, Capability::Keyboard) && self.keyboard.is_none() {
            let keyboard = self
                .protocols
                .seat_state
                .get_keyboard(conn, qh, &seat, None)
                .expect("Failed to create keyboard");
            self.keyboard = Some(keyboard);
        }
    }

    fn remove_capability(
        &mut self,
        conn: &mut ConnectionHandle,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if matches!(capability, Capability::Keyboard) && self.keyboard.is_some() {
            self.keyboard.take().unwrap().release(conn);
        }
    }

    fn remove_seat(&mut self, _: &mut ConnectionHandle, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {
    }
}
