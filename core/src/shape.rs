//! d3-shape's line and area generators over iterators of already-scaled
//! geometry (Part I, Accessors, option A), drawing into any
//! [`PathContext`](crate::path::PathContext) with one of d3's curves.
//!
//! ```
//! use rustd3_core::path;
//! use rustd3_core::shape::{self, Curve};
//!
//! let pts = [(0.0, 0.0), (1.0, 2.0), (2.0, 1.0), (3.0, 3.0)];
//! let d = path::svg(|p| shape::line().curve(Curve::StepAfter).draw(pts, p));
//! assert_eq!(d, "M0,0L1,0L1,2L2,2L2,1L3,1L3,3");
//!
//! let rows = [(0.0, 10.0, 4.0), (1.0, 10.0, 2.0), (2.0, 10.0, 5.0)];
//! let d = path::svg(|p| shape::area().draw(rows.map(Some), p));
//! assert_eq!(d, "M0,4L1,2L2,5L2,10L1,10L0,10Z");
//! ```

mod area;
mod curve;
mod line;

pub use area::{Area, Span};
pub use curve::Curve;
pub use line::{Line, Vertex};

/// d3.line(): straight segments, `None` items breaking the line.
pub fn line() -> Line {
    Line::default()
}

/// d3.area(): straight segments over a baseline at 0, `None` items
/// breaking the area.
pub fn area() -> Area {
    Area::default()
}
