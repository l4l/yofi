use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};

use sctk::{
    environment::Environment,
    reexports::{calloop, client::protocol::wl_keyboard},
    seat::keyboard::{map_keyboard_repeat, Event as KbEvent, KeyState, RepeatKind, RepeatSource},
    seat::{with_seat_data, SeatData, SeatListener},
};

pub struct ModifierState {
    pub ctrl: bool,
    pub shift: bool,
}

pub struct KeyPress {
    pub keysym: u32,
    pub sym: Option<char>,
    pub ctrl: bool,
    pub shift: bool,
}

#[derive(Default)]
struct SeatInfo {
    keyboard: Option<(wl_keyboard::WlKeyboard, calloop::Source<RepeatSource>)>,
}

fn send_event(state: &mut ModifierState, tx: &Sender<KeyPress>, event: KbEvent) {
    match event {
        KbEvent::Key {
            keysym,
            state: KeyState::Pressed,
            utf8,
            ..
        } => {
            log::trace!(
                "key {:?}: {:x} (text: {:?})",
                KeyState::Pressed,
                keysym,
                utf8
            );
            tx.send(KeyPress {
                keysym,
                sym: utf8.and_then(|s| s.chars().next()),
                ctrl: state.ctrl,
                shift: state.shift,
            })
            .expect("key handling failed");
        }
        KbEvent::Key {
            keysym,
            state,
            utf8,
            ..
        } => log::trace!("key {:?}: {:x} (text: {:?})", state, keysym, utf8),
        KbEvent::Modifiers { modifiers } => {
            log::trace!("modifiers changed to {:?}", modifiers);
            state.ctrl = modifiers.ctrl;
            state.shift = modifiers.shift;
        }
        KbEvent::Repeat { keysym, utf8, .. } => {
            log::trace!("key repeat {:x} (text: {:?})", keysym, utf8);
            tx.send(KeyPress {
                keysym,
                sym: utf8.and_then(|s| s.chars().next()),
                ctrl: state.ctrl,
                shift: state.shift,
            })
            .expect("key handling failed");
        }
        KbEvent::Enter { .. } | KbEvent::Leave { .. } => {}
    }
}

pub struct InputHandler {
    _seat_listener: SeatListener,
}

impl InputHandler {
    pub fn new(
        env: &Environment<super::Env>,
        event_loop: &calloop::EventLoop<()>,
    ) -> (Self, Receiver<KeyPress>) {
        let (tx, rx) = mpsc::channel();

        let mut seats = HashMap::<String, SeatInfo>::new();

        let loop_handle = event_loop.handle();
        let mut seat_handler = move |seat, seat_data: &SeatData| {
            let mut state = ModifierState {
                ctrl: false,
                shift: false,
            };
            let tx = tx.clone();
            let data = seats.entry(seat_data.name.clone()).or_default();
            if seat_data.has_keyboard && !seat_data.defunct {
                if data.keyboard.is_none() {
                    let seat_name = seat_data.name.as_str();
                    let h = loop_handle.clone();
                    data.keyboard = map_keyboard_repeat(
                        h,
                        &seat,
                        None,
                        RepeatKind::System,
                        move |event, _, _| {
                            send_event(&mut state, &tx, event);
                        },
                    )
                    .map_err(|e| {
                        log::error!("failed to map keyboard on seat {}: {:?}", seat_name, e)
                    })
                    .ok();
                }
            } else if let Some((kbd, source)) = data.keyboard.take() {
                kbd.release();
                loop_handle.remove(source);
            }
        };

        for seat in env.get_all_seats() {
            if let Some(seat_data) = with_seat_data(&seat, Clone::clone) {
                seat_handler(seat, &seat_data);
            }
        }

        let _seat_listener =
            env.listen_for_seats(move |seat, seat_data, _| seat_handler(seat, &seat_data));

        (Self { _seat_listener }, rx)
    }
}
