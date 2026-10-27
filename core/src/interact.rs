//! d3's interaction as values and state machines: renderer-agnostic
//! [`Pointer`] input with its [`Modifiers`]; d3-zoom's [`Transform`], a
//! value, and [`Zoom`], the behavior that changes it; [`Rescale`], what a
//! transform or a pixel-space gesture does to a scale; d3-brush's
//! [`BrushX`] and [`BrushY`], which report [`BrushEvent`]s; and d3-drag's
//! [`Drag`], which reports [`drag::Event`]s. The host
//! translates its events into `Pointer`; the behaviors keep only their
//! gesture state and return what changed.
//!
//! Two `Transform`s exist on purpose: `interact::Transform` (d3's
//! `ZoomTransform`, here) and the trait `scale::Transform`. Code that needs
//! both writes the module path at every use.

pub mod drag;

mod brush;
mod modifiers;
mod pointer;
mod rescale;
mod transform;
mod zoom;

pub use brush::{BrushEvent, BrushX, BrushY};
pub use modifiers::Modifiers;
pub use pointer::{Button, Delta, Pointer};
pub use rescale::Rescale;
pub use transform::Transform;
pub use zoom::Zoom;

/// d3.drag(): a press picks a subject, and the pointer drags it until the
/// left button is released (d3-drag 3.0.0, `drag.js`). The caller asks
/// what is under the pointer only on a press; reported positions keep the
/// subject's offset from the press, so a dragged circle does not jump to
/// the pointer. Its methods and [`drag::Event`] are in [`mod@drag`].
///
/// d3's filter applies: presses of other buttons or with ctrl held do
/// nothing. Modifier changes, wheel steps and double clicks do nothing,
/// and a drag continues after [`Pointer::Left`].
///
/// A circle at (10, 10) pressed at (12, 13) and dragged (d3's trace G1):
///
/// ```
/// use rustd3_core::interact::drag::Event;
/// use rustd3_core::interact::{Button, Drag, Modifiers, Pointer};
///
/// let mut drag = Drag::default();
/// let press = Pointer::Pressed { x: 12.0, y: 13.0, button: Button::Left, modifiers: Modifiers::NONE };
/// let circle = |_x, _y| Some((10.0, 10.0));
/// assert_eq!(drag.handle(press, circle), Some(Event::Start { x: 10.0, y: 10.0 }));
/// let to = Pointer::Moved { x: 20.0, y: 25.0 };
/// let moved = Event::Drag { x: 18.0, y: 22.0, dx: 8.0, dy: 12.0 };
/// assert_eq!(drag.handle(to, |_, _| None), Some(moved));
/// let release = Pointer::Released { button: Button::Left };
/// let end = Event::End { x: 18.0, y: 22.0, moved: true };
/// assert_eq!(drag.handle(release, |_, _| None), Some(end));
/// assert!(!drag.is_active());
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Drag {
    click_distance: f64,
    gesture: Option<drag::Gesture>,
}
