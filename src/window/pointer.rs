use sctk::reexports::client::protocol::wl_pointer::AxisSource;
use sctk::reexports::client::{protocol, Connection, QueueHandle};
use sctk::seat::keyboard::Modifiers;
use sctk::seat::pointer::{PointerEvent, PointerEventKind, PointerHandler, *};

use super::Window;

// According to https://wayland.freedesktop.org/libinput/doc/1.19.0/wheel-api.html
// wheel typically has this angle per step.
// This actually should be configured and auto-detected (from udev probably?) but
// for now it should work for most cases and could be tuned via config.
const SCROLL_PER_STEP: f64 = 15.0;

pub struct Params {
    pub launch_on_middle: bool,
    pub wheel_scroll_multiplier: f64,
}

impl PointerHandler for Window {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &protocol::wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        let mut changed = false;
        let config = self.config.param::<Params>();

        for event in events {
            // Ignore events for other surfaces
            if event.surface != *self.surface {
                continue;
            }

            match event.kind {
                PointerEventKind::Release {
                    button: BTN_LEFT, ..
                } if self.surface.is_overlay() => {
                    let (ox, oy) = self.content_offset();
                    let left = f64::from(ox);
                    let top = f64::from(oy);
                    let (cw, ch) = self.content_size();
                    let right = cw as f64 + left;
                    let bottom = ch as f64 + top;

                    let (px, py) = event.position;
                    if px < left || py < top || px > right || py > bottom {
                        self.exit = true;
                    } else {
                        // TODO: implement precise clicks on items
                        continue;
                    }
                }
                PointerEventKind::Release {
                    button: BTN_MIDDLE, ..
                } if config.launch_on_middle => {
                    let with_fork = matches!(self.key_modifiers, Modifiers { ctrl: true, .. });
                    if let Err(err) = self.state.eval_input(with_fork) {
                        self.error = Some(err);
                    }
                }
                PointerEventKind::Release {
                    button: BTN_RIGHT, ..
                } => self.exit = true,
                PointerEventKind::Release {
                    button: BTN_BACK, ..
                } => self.state.prev_subitem(),
                PointerEventKind::Release {
                    button: BTN_FORWARD,
                    ..
                } => self.state.next_subitem(),
                PointerEventKind::Axis {
                    vertical:
                        AxisScroll {
                            absolute,
                            discrete: _,
                            // XXX: handle this one?
                            stop: _,
                        },
                    source:
                        Some(AxisSource::Wheel)
                        | Some(AxisSource::Finger)
                        | Some(AxisSource::Continuous),
                    time: _,
                    horizontal: _,
                } => {
                    self.wheel_scroll_pending += absolute;
                }
                PointerEventKind::Enter { .. }
                | PointerEventKind::Leave { .. }
                | PointerEventKind::Motion { .. }
                | PointerEventKind::Press { .. }
                | PointerEventKind::Release { .. }
                | PointerEventKind::Axis { .. } => continue,
            }
            changed = true;
        }

        if changed {
            let scroll_per_step = SCROLL_PER_STEP
                * if config.wheel_scroll_multiplier > 0.0 {
                    config.wheel_scroll_multiplier
                } else {
                    1.0
                };
            let wheel_steps = (self.wheel_scroll_pending / scroll_per_step) as i32;
            if wheel_steps != 0 {
                self.wheel_scroll_pending -= f64::from(wheel_steps) * scroll_per_step;
            }
            let is_wheel_down = wheel_steps > 0;
            for _ in 0..wheel_steps.abs() {
                if is_wheel_down {
                    self.state.next_item();
                } else {
                    self.state.prev_item();
                }
            }
        }
    }
}
