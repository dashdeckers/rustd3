//! [`Invert`]: position scales with a typed inverse.

use crate::scale::{Kind, Position};

/// A position scale with a typed inverse, checked at compile time:
/// brushes accept only these. Continuous scales, never band, point or
/// ordinal ones.
pub trait Invert: Position {
    /// The domain value at output position `px`; `None` when it is not
    /// finite.
    fn invert(&self, px: f64) -> Option<<Self as Kind>::Domain>;
}
