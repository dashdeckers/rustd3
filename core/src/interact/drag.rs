//! d3-drag 3.0.0's behavior (`drag.js`) for one pointer: [`Drag`]'s
//! methods and the [`Event`]s it reports.

mod event;

pub use event::Event;

use crate::interact::{Button, Drag, Pointer};
use crate::js;

/// One press's gesture: d3's `beforestart` closure (121–166) and the
/// mouse locals of `mousedowned` (50–63).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Gesture {
    /// d3's `mousedownx` and `mousedowny`: where the press was.
    press: (f64, f64),
    /// The subject's offset from the press, d3's `dx` and `dy` (138–139).
    offset: (f64, f64),
    /// d3's `p`: the pointer's last position.
    point: (f64, f64),
    /// d3's `mousemoving`: the pointer travelled beyond the click
    /// distance.
    moved: bool,
}

impl Drag {
    /// d3's `drag.clickDistance(px)`: how far the pointer may travel from
    /// the press before the end reports `moved`; 0 by default.
    #[must_use]
    pub fn click_distance(self, px: f64) -> Self {
        Drag {
            click_distance: px,
            ..self
        }
    }

    /// Feeds one event; what d3 emits for it. `subject` is asked on a
    /// press: the dragged thing's position, or None to ignore the press.
    ///
    /// A left press without ctrl asks `subject` at the press (d3's filter,
    /// 9–11, drops the rest) and starts at the subject's position; each
    /// move reports the pointer plus the subject's offset from the press,
    /// and the movement since the previous event; the left button's
    /// release ends the drag at the last position. A press during a drag
    /// starts anew when `subject` gives a position, as d3's does.
    pub fn handle(
        &mut self,
        input: Pointer,
        subject: impl FnOnce(f64, f64) -> Option<(f64, f64)>,
    ) -> Option<Event> {
        match input {
            Pointer::Pressed {
                x,
                y,
                button,
                modifiers,
            } => {
                if button != Button::Left || modifiers.get_ctrl() {
                    return None;
                }
                let (sx, sy) = subject(x, y)?;
                // `s.x - p[0] || 0`: a NaN offset is none.
                let offset = (or_zero(sx - x), or_zero(sy - y));
                self.gesture = Some(Gesture {
                    press: (x, y),
                    offset,
                    point: (x, y),
                    moved: false,
                });
                Some(Event::Start {
                    x: x + offset.0,
                    y: y + offset.1,
                })
            }
            Pointer::Moved { x, y } => {
                let gesture = self.gesture.as_mut()?;
                if !gesture.moved {
                    let (dx, dy) = (x - gesture.press.0, y - gesture.press.1);
                    gesture.moved = dx * dx + dy * dy > self.click_distance * self.click_distance;
                }
                let (x0, y0) = gesture.point;
                gesture.point = (x, y);
                Some(Event::Drag {
                    x: x + gesture.offset.0,
                    y: y + gesture.offset.1,
                    dx: x - x0,
                    dy: y - y0,
                })
            }
            Pointer::Released { button } => {
                if button != Button::Left {
                    return None;
                }
                let gesture = self.gesture.take()?;
                Some(Event::End {
                    x: gesture.point.0 + gesture.offset.0,
                    y: gesture.point.1 + gesture.offset.1,
                    moved: gesture.moved,
                })
            }
            Pointer::Wheel { .. }
            | Pointer::DoubleClicked { .. }
            | Pointer::Modifiers(_)
            | Pointer::Left => None,
        }
    }

    /// A drag is in progress.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.gesture.is_some()
    }
}

/// JavaScript's `x || 0` for a number: 0 for ±0 and NaN.
fn or_zero(x: f64) -> f64 {
    if js::truthy(x) { x } else { 0.0 }
}
