//! [`Bucket`]: one bin of a layout.

/// One bin: its extent from `x0` (inclusive) to `x1` (exclusive, except
/// that the last bin also holds values equal to its `x1`), and the indices
/// of the laid-out values that fall in it, so results point back to their
/// data. Plain data. Where d3's edges are undefined because there is no
/// value to take a domain from, they are NaN.
#[derive(Debug, Clone, PartialEq)]
pub struct Bucket {
    /// The lower edge.
    pub x0: f64,
    /// The upper edge.
    pub x1: f64,
    /// Indices of the values in this bin, ascending.
    pub indices: Vec<usize>,
}
