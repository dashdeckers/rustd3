//! [`Tracker`]: iced events in, core's [`Pointer`] input out, for the
//! zoom, brush and drag behaviors of `rustd3_core::interact`.

use iced_core::time::Instant;
use iced_core::{Event, Point, Rectangle, keyboard, mouse, window};
use rustd3_core::interact::{Button, Delta, Modifiers, Pointer};

/// The time of iced's double click: the same button again within this many
/// milliseconds (`core/src/mouse/click.rs:73–84`).
const DOUBLE_CLICK_MS: u128 = 300;
/// The distance of iced's double click: the second press less than this
/// many logical pixels away.
const DOUBLE_CLICK_PX: f64 = 6.0;

/// Turns iced events into [`Pointer`] input in an area's own coordinates.
///
/// Apps keep one in their `canvas::Program::State` and pass every event
/// with the program's bounds and cursor to [`translate`](Tracker::translate):
///
/// - A move inside the area, or anywhere while a button is held, is
///   `Moved` relative to the area. Drags continue outside, as d3's window
///   listeners do. Leaving the area with no button held, or leaving the
///   window, is `Left`, once.
/// - A press inside is `Pressed`. The release of the held button,
///   anywhere, is `Released`.
/// - A wheel step inside is `Wheel`. Its delta is negated to the DOM's
///   sign (iced's positive y scrolls up), and pixel deltas, which iced
///   passes in physical pixels, are divided by the scale factor.
/// - A modifier change, anywhere, is `Modifiers`, and later presses and
///   wheel steps carry it.
/// - A `RedrawRequested` after a press stamps the press with the redraw's
///   time. It is `DoubleClicked` when iced's rule holds: the same button
///   as the last stamped press, at most 300 ms later and less than 6 px
///   away.
///
/// Time comes only from `RedrawRequested`, so while a press waits for one,
/// [`wants_redraw`](Tracker::wants_redraw) is true. A program that
/// publishes no message for the event then returns
/// `Action::request_redraw()` (iced redraws after a published message
/// anyway). One press waits at a time, and a second press before the
/// redraw takes its place.
///
/// ```
/// # mod rustd3 { pub use rustd3_core::interact; }
/// use iced_core::{mouse, Event, Point, Rectangle, Size};
/// use rustd3::interact::{Delta, Pointer};
/// use rustd3_iced::pointer::Tracker;
///
/// let mut tracker = Tracker::new();
/// let bounds = Rectangle::new(Point::new(100.0, 50.0), Size::new(600.0, 300.0));
/// let cursor = mouse::Cursor::Available(Point::new(550.0, 200.0));
/// let wheel = Event::Mouse(mouse::Event::WheelScrolled {
///     delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
/// });
/// match tracker.translate(&wheel, bounds, cursor) {
///     Some(Pointer::Wheel { x, y, delta, .. }) => {
///         assert_eq!((x, y), (450.0, 150.0));
///         assert_eq!(delta, Delta::Lines { x: -0.0, y: -1.0 });
///     }
///     other => panic!("expected a wheel step, got {other:?}"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Tracker {
    /// The modifiers of the last `ModifiersChanged`.
    modifiers: Modifiers,
    /// The button pressed inside the area and not yet released.
    held: Option<Button>,
    /// A press waiting for the next `RedrawRequested` to be timed.
    waiting: Option<Press>,
    /// The last timed press that did not complete a double click.
    last: Option<(Press, Instant)>,
    /// Device pixels per logical pixel, for pixel wheel deltas.
    scale_factor: f32,
    /// Whether the area has seen the pointer since it last reported `Left`.
    inside: bool,
}

/// A press in the area's coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Press {
    x: f64,
    y: f64,
    button: Button,
}

impl Default for Tracker {
    fn default() -> Self {
        Tracker {
            modifiers: Modifiers::NONE,
            held: None,
            waiting: None,
            last: None,
            scale_factor: 1.0,
            inside: false,
        }
    }
}

impl Tracker {
    /// A tracker with no button held, no modifiers and a scale factor of 1.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Divides `ScrollDelta::Pixels`, which iced passes in physical
    /// pixels, by `factor`. A factor that is not finite and positive is
    /// ignored.
    #[must_use]
    pub fn scale_factor(self, factor: f32) -> Self {
        if factor.is_finite() && factor > 0.0 {
            Tracker {
                scale_factor: factor,
                ..self
            }
        } else {
            self
        }
    }

    /// Translates `event` into pointer input for the area `bounds`, with
    /// the cursor where iced reports it. Returns `None` for events that
    /// make none.
    pub fn translate(
        &mut self,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Pointer> {
        match event {
            Event::Mouse(event) => self.mouse(*event, bounds, cursor),
            Event::Keyboard(keyboard::Event::ModifiersChanged(held)) => {
                self.modifiers = modifiers(*held);
                Some(Pointer::Modifiers(self.modifiers))
            }
            Event::Window(window::Event::RedrawRequested(now)) => self.stamp(*now),
            Event::Keyboard(_)
            | Event::Window(_)
            | Event::Touch(_)
            | Event::InputMethod(_)
            | Event::Clipboard(_)
            | Event::Waken => None,
        }
    }

    /// Whether a press waits for the next `RedrawRequested` to be timed.
    /// While this is true, a program that publishes nothing returns
    /// `Action::request_redraw()`.
    #[must_use]
    pub fn wants_redraw(&self) -> bool {
        self.waiting.is_some()
    }

    /// The input a mouse event makes.
    fn mouse(
        &mut self,
        event: mouse::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Pointer> {
        match event {
            mouse::Event::CursorMoved { position } => self.moved(position, bounds),
            mouse::Event::CursorLeft => self.leave(),
            mouse::Event::ButtonPressed(pressed) => {
                let (x, y) = local(cursor.position_in(bounds)?);
                let button = button(pressed);
                self.inside = true;
                self.held = self.held.or(Some(button));
                self.waiting = Some(Press { x, y, button });
                Some(Pointer::Pressed {
                    x,
                    y,
                    button,
                    modifiers: self.modifiers,
                })
            }
            mouse::Event::ButtonReleased(released) => {
                let button = button(released);
                (self.held == Some(button)).then(|| {
                    self.held = None;
                    Pointer::Released { button }
                })
            }
            mouse::Event::WheelScrolled { delta } => {
                let (x, y) = local(cursor.position_in(bounds)?);
                self.inside = true;
                Some(Pointer::Wheel {
                    x,
                    y,
                    delta: self.delta(delta),
                    modifiers: self.modifiers,
                })
            }
            mouse::Event::CursorEntered => None,
        }
    }

    /// `Moved` inside the area or during a drag, and `Left` on leaving it.
    fn moved(&mut self, position: Point, bounds: Rectangle) -> Option<Pointer> {
        let inside = bounds.contains(position);
        if inside || self.held.is_some() {
            self.inside |= inside;
            let (x, y) = local(Point::new(position.x - bounds.x, position.y - bounds.y));
            Some(Pointer::Moved { x, y })
        } else {
            self.leave()
        }
    }

    /// `Left`, once, unless a button is held.
    fn leave(&mut self) -> Option<Pointer> {
        if self.inside && self.held.is_none() {
            self.inside = false;
            Some(Pointer::Left)
        } else {
            None
        }
    }

    /// Times the waiting press, and returns `DoubleClicked` when it
    /// completes one.
    fn stamp(&mut self, now: Instant) -> Option<Pointer> {
        let press = self.waiting.take()?;
        let double = self.last.is_some_and(|(last, at)| {
            last.button == press.button
                && now > at
                && (now - at).as_millis() <= DOUBLE_CLICK_MS
                && (press.x - last.x).hypot(press.y - last.y) < DOUBLE_CLICK_PX
        });
        if double {
            // A third press starts afresh, as iced's triple click is no
            // double click.
            self.last = None;
            Some(Pointer::DoubleClicked {
                x: press.x,
                y: press.y,
            })
        } else {
            self.last = Some((press, now));
            None
        }
    }

    /// A wheel delta in the DOM's sign, with pixels in logical pixels.
    fn delta(&self, delta: mouse::ScrollDelta) -> Delta {
        match delta {
            mouse::ScrollDelta::Lines { x, y } => Delta::Lines {
                x: -f64::from(x),
                y: -f64::from(y),
            },
            mouse::ScrollDelta::Pixels { x, y } => {
                let k = f64::from(self.scale_factor);
                Delta::Pixels {
                    x: -f64::from(x) / k,
                    y: -f64::from(y) / k,
                }
            }
        }
    }
}

/// An iced point as `f64` coordinates.
fn local(at: Point) -> (f64, f64) {
    (f64::from(at.x), f64::from(at.y))
}

/// iced's mouse button as core's.
fn button(button: mouse::Button) -> Button {
    match button {
        mouse::Button::Left => Button::Left,
        mouse::Button::Right => Button::Right,
        mouse::Button::Middle => Button::Middle,
        mouse::Button::Back => Button::Back,
        mouse::Button::Forward => Button::Forward,
        mouse::Button::Other(n) => Button::Other(n),
    }
}

/// iced's keyboard modifiers as core's.
fn modifiers(held: keyboard::Modifiers) -> Modifiers {
    [
        (held.shift(), Modifiers::SHIFT),
        (held.control(), Modifiers::CTRL),
        (held.alt(), Modifiers::ALT),
        (held.logo(), Modifiers::LOGO),
    ]
    .into_iter()
    .filter(|&(on, _)| on)
    .fold(Modifiers::NONE, |all, (_, one)| all.union(one))
}
