//! [`Orient`]: which side of the axis line ticks and labels go.

use crate::axis::Anchor;

/// Which side of the axis line ticks and labels go: d3's `axisTop`,
/// `axisRight`, `axisBottom` and `axisLeft`.
///
/// ```
/// use rustd3_core::{axis, path, scale};
///
/// let x = scale::linear().domain([0.0, 1.0]).range([0.0, 100.0]);
/// let top = axis::new(axis::Orient::Top, &x).ticks(1);
/// assert_eq!(path::svg(|p| top.draw(p)), path::svg(|p| axis::top(&x).ticks(1).draw(p)));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orient {
    /// A horizontal axis with ticks and labels above it.
    Top,
    /// A vertical axis with ticks and labels to its right.
    Right,
    /// A horizontal axis with ticks and labels below it.
    Bottom,
    /// A vertical axis with ticks and labels to its left.
    Left,
}

impl Orient {
    /// d3's `k`: −1 where ticks point up or left, +1 otherwise.
    pub(super) fn sign(self) -> f64 {
        match self {
            Orient::Top | Orient::Left => -1.0,
            Orient::Right | Orient::Bottom => 1.0,
        }
    }

    /// The point `along` the axis line and `across` it, in axis
    /// coordinates.
    pub(super) fn point(self, along: f64, across: f64) -> (f64, f64) {
        match self {
            Orient::Top | Orient::Bottom => (along, across),
            Orient::Right | Orient::Left => (across, along),
        }
    }

    /// d3's `text-anchor` of the labels.
    pub(super) fn anchor(self) -> Anchor {
        match self {
            Orient::Top | Orient::Bottom => Anchor::Middle,
            Orient::Right => Anchor::Start,
            Orient::Left => Anchor::End,
        }
    }

    /// d3's `dy` of the labels, in em.
    pub(super) fn dy(self) -> f64 {
        match self {
            Orient::Top => 0.0,
            Orient::Bottom => 0.71,
            Orient::Right | Orient::Left => 0.32,
        }
    }
}
