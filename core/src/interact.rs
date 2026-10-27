//! d3's interaction as values and state machines: renderer-agnostic
//! [`Pointer`] input with its [`Modifiers`]; d3-zoom's
//! [`Transform`](crate::interact::Transform), a value, and [`Zoom`], the
//! behavior that changes it. The host translates its events into
//! `Pointer`; the behaviors keep only their gesture state and return what
//! changed.
//!
//! Two `Transform`s exist on purpose: `interact::Transform` (d3's
//! `ZoomTransform`, here) and the trait `scale::Transform`. Code that needs
//! both writes the module path at every use.

mod modifiers;
mod pointer;
mod transform;
mod zoom;

pub use modifiers::Modifiers;
pub use pointer::{Button, Delta, Pointer};
pub use transform::Transform;
pub use zoom::Zoom;
