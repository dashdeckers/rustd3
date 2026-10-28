//! [`Dim`]: the axis a brush selects along.

/// The axis a brush selects along: d3's `brushX` or `brushY`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dim {
    /// Across the plot area; the selection spans its height.
    X,
    /// Up the plot area; the selection spans its width.
    Y,
}
