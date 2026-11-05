//! [`Clip`]: what a group clips its commands to.

use kurbo::{BezPath, Rect};

/// The clip of a group. Rectangles are cheap in every backend, while only
/// path clips can be missing from one, so the two are distinct.
#[derive(Debug, Clone, PartialEq)]
pub enum Clip {
    /// An axis-aligned rectangle in the group's coordinates.
    Rect(Rect),
    /// Any path in the group's coordinates, nonzero fill.
    Path(BezPath),
}
