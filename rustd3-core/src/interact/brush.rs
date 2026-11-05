//! d3-brush's one-dimensional brushes, [`BrushX`] and [`BrushY`]. They
//! share a crate-private engine, d3's `brush(dim)`, and report
//! [`BrushEvent`]s.

mod engine;
mod event;
mod x;
mod y;

pub use event::BrushEvent;
pub use x::BrushX;
pub use y::BrushY;
