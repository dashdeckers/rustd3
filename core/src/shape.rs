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

mod arc;
mod area;
mod curve;
mod line;
pub mod pie;
mod slice;
pub mod stack;
mod symbol;

pub use arc::Arc;
pub use area::{Area, Span};
pub use curve::Curve;
pub use line::{Line, Vertex};
pub use slice::Slice;
pub use symbol::Symbol;

/// d3.pie(): values to the angles of [`Slice`]s, largest first by default;
/// built by consuming builders, laid out by [`Pie::layout`]. Its methods
/// and [`pie::Sort`] are in [`pie`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pie {
    sort: pie::Sort,
    start_angle: f64,
    end_angle: f64,
    pad_angle: f64,
}

/// d3.line(): straight segments, `None` items breaking the line.
pub fn line() -> Line {
    Line::default()
}

/// d3.area(): straight segments over a baseline at 0, `None` items
/// breaking the area.
pub fn area() -> Area {
    Area::default()
}

/// d3.stack() over columns: orders and offsets as builders, laid out by
/// [`Stack::layout`]. Its methods, [`stack::Order`] and [`stack::Offset`]
/// are in [`stack`] (the module).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Stack {
    order: stack::Order,
    offset: stack::Offset,
}

/// d3.pie(): descending, from 0 to τ, unpadded.
pub fn pie() -> Pie {
    Pie::default()
}

/// d3.arc(): radii 0, no corners, padded as each slice says.
pub fn arc() -> Arc {
    Arc::default()
}

/// d3.stack() over columns (Part I, Shapes and `PathContext`): one layer per
/// column, in the columns' order on a zero baseline, each `[y0, y1]` per
/// row; rows are the shortest column's length.
///
/// ```
/// use rustd3_core::shape;
///
/// let apples = [3.0, 2.0, 1.0];
/// let bananas = [1.0, 1.0, 4.0];
/// let layers = shape::stack(&[&apples, &bananas]);
/// assert_eq!(layers[1], [[3.0, 4.0], [2.0, 3.0], [1.0, 5.0]]);
/// ```
pub fn stack(columns: &[&[f64]]) -> Vec<Vec<[f64; 2]>> {
    Stack::default().layout(columns)
}
