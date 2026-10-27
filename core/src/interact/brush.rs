//! d3-brush's one-dimensional brushes, [`BrushX`] and [`BrushY`], over
//! one crate-private engine, d3's `brush(dim)`; they report
//! [`BrushEvent`]s.

mod engine;
mod event;
mod x;
mod y;

pub use event::BrushEvent;
pub use x::BrushX;
pub use y::BrushY;
