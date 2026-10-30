//! d3-shape over iterators of already-scaled geometry: lines and areas with d3's curves, their radial forms, pies
//! and arcs, stacks and symbols, each drawing into any
//! [`PathContext`](crate::path::PathContext).
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
//!
//! [`LineOf`] wraps the line in d3-style accessors over a slice of data.
//! The generators [`Line`], [`Area`], [`Pie`] and [`Stack`] are defined
//! here and built through the methods in their modules,
//! [`line`](mod@line), [`area`](mod@area), [`pie`](mod@pie) and
//! [`stack`](mod@stack), which also hold the radial forms and the stack's
//! orders and offsets.

mod arc;
pub mod area;
mod curve;
pub mod line;
mod line_of;
pub mod pie;
mod slice;
pub mod stack;
mod symbol;

pub use arc::Arc;
pub use curve::Curve;
pub use line_of::LineOf;
pub use slice::Slice;
pub use symbol::Symbol;

/// An item a line draws: a point, or `None` for a gap.
pub trait Vertex {
    /// The point, or `None` to break the line (d3's `defined`).
    fn vertex(self) -> Option<(f64, f64)>;
}

/// d3.line() over already-scaled points, built by consuming builders.
///
/// ```
/// use rustd3_core::path;
/// use rustd3_core::shape;
///
/// let readings = [Some(75.0), Some(25.0), None, Some(50.0), Some(0.0)];
/// let points = readings
///     .iter()
///     .enumerate()
///     .map(|(i, v)| v.map(|v| (i as f64 * 100.0, v)));
/// let d = path::svg(|p| shape::line().draw(points, p));
/// assert_eq!(d, "M0,75L100,25M300,50L400,0");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Line {
    curve: Curve,
}

/// An item an area draws: `(x, y0, y1)`, or `(x, y1)` over the area's
/// baseline, or `None` for a gap.
pub trait Span {
    /// The item as `(x, y0, y1)`, a 2-tuple taking `baseline` as `y0`; `None`
    /// breaks the area (d3's `defined`).
    fn span(self, baseline: f64) -> Option<(f64, f64, f64)>;
}

/// d3.area() over already-scaled items, built by consuming builders.
///
/// ```
/// use rustd3_core::path;
/// use rustd3_core::shape;
///
/// let tops = [(0.0, 4.0), (1.0, 2.0), (2.0, 5.0)];
/// let d = path::svg(|p| shape::area().y0(300.0).draw(tops, p));
/// assert_eq!(d, "M0,4L1,2L2,5L2,300L1,300L0,300Z");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Area {
    curve: Curve,
    /// The `y0` of 2-tuple items; 0 is d3's default.
    baseline: f64,
}

/// d3.pie(): values to the angles of [`Slice`]s, largest first by default;
/// built by consuming builders, laid out by [`Pie::layout`]. Its methods
/// and [`pie::Sort`] are in [`pie`](mod@pie).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pie {
    sort: pie::Sort,
    start_angle: f64,
    end_angle: f64,
    pad_angle: f64,
}

/// d3.stack() over columns: orders and offsets as builders, laid out by
/// [`Stack::layout`]. Its methods, [`stack::Order`] and [`stack::Offset`]
/// are in [`stack`](mod@stack).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Stack {
    order: stack::Order,
    offset: stack::Offset,
}

/// d3.line(): straight segments, `None` items breaking the line.
pub fn line() -> Line {
    Line::default()
}

/// d3.line() with accessors over a slice of data, a wrapper of
/// [`line()`] for data that is not geometry yet: x and y 0, every
/// datum defined, until set.
pub fn line_of<'a, D>() -> LineOf<'a, D> {
    LineOf::default()
}

/// d3.area(): straight segments over a baseline at 0, `None` items
/// breaking the area.
pub fn area() -> Area {
    Area::default()
}

/// d3.pie(): descending, from 0 to τ, unpadded.
pub fn pie() -> Pie {
    Pie::default()
}

/// d3.arc(): radii 0, no corners, padded as each slice says.
pub fn arc() -> Arc {
    Arc::default()
}

/// d3.stack() over columns: one layer per
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
