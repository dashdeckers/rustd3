//! [`Slice`], one wedge of a pie.

/// One wedge: d3's arc datum, with positions in place of `data`. It is
/// plain data. [`Pie::layout`](crate::shape::Pie::layout) writes it and
/// [`Arc`](crate::shape::Arc) draws it, and a literal draws any sector or
/// annulus.
///
/// ```
/// use std::f64::consts::PI;
///
/// use rustd3_core::shape::{self, pie};
///
/// let slices = shape::pie().sort(pie::Sort::Input).layout([1.0, 3.0]);
/// assert_eq!((slices[1].datum, slices[1].index), (1, 1));
/// assert!((slices[0].end_angle - PI / 2.0).abs() < 1e-12);
/// ```
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
    /// The padding between neighboring wedges, in radians.
    pub pad_angle: f64,
}
