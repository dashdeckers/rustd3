//! [`Paints`]: per-point paints of a markers command.

use peniko::Brush;

/// A paint table and one index into it per point.
#[derive(Debug, Clone, PartialEq)]
pub struct Paints {
    /// The distinct paints.
    pub table: Vec<Brush>,
    /// One index into `table` per point.
    pub index: Vec<u32>,
}
