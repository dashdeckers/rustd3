//! [`Slice`], one wedge of a pie.

/// One wedge: d3's arc datum, with positions in place of `data` (Part I,
/// Shapes and `PathContext`). Plain data: [`Pie::layout`](crate::shape::Pie::layout)
/// writes it and [`Arc`](crate::shape::Arc) draws it; a literal draws any
/// sector or annulus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slice {
    /// Position of the datum in the input (d3's `data`, as an index).
    pub datum: usize,
    /// Position in drawing order (d3's `index`): the rank after sorting.
    pub index: usize,
    /// The datum's value; a value that is not positive spans no angle.
    pub value: f64,
    /// Radians, clockwise from 12 o'clock.
    pub start_angle: f64,
    /// Radians, clockwise from 12 o'clock; below `start_angle` the wedge
    /// runs anticlockwise.
    pub end_angle: f64,
    /// The padding between neighbouring wedges, in radians.
    pub pad_angle: f64,
}
