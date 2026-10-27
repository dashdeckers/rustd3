//! d3's interaction as values and state machines: renderer-agnostic
//! [`Pointer`] input with its [`Modifiers`]; d3-zoom's [`Transform`], a
//! value, and [`Zoom`], the behavior that changes it; [`Rescale`], what a
//! transform or a pixel-space gesture does to a scale; and d3-brush's
//! [`BrushX`] and [`BrushY`], which report [`BrushEvent`]s. The host
//! translates its events into `Pointer`; the behaviors keep only their
//! gesture state and return what changed.
//!
//! Two `Transform`s exist on purpose: `interact::Transform` (d3's
//! `ZoomTransform`, here) and the trait `scale::Transform`. Code that needs
//! both writes the module path at every use.

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
