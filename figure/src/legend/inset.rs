//! [`Inset`]: where a legend sits inside a panel.

use kurbo::{Point, Rect, Size};

/// A corner of a panel's plot area. An inset legend is a boxed column
/// 0.5 em from that corner, drawn over the panel's marks; further insets
/// in the same corner stack inward, each 0.5 em past the one before (down
/// from a top corner, up from a bottom one).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inset {
    /// The top left corner.
    TopLeft,
    /// The top right corner.
    TopRight,
    /// The bottom left corner.
    BottomLeft,
    /// The bottom right corner.
    BottomRight,
}

impl Inset {
    /// The top-left corner of a box of `size` placed `gap` px inside this
    /// corner of plot area `area` and `inward` px further from its top or
    /// bottom edge.
    pub(crate) fn place(self, area: Rect, size: Size, gap: f64, inward: f64) -> Point {
        let left = area.x0 + gap;
        let right = area.x1 - gap - size.width;
        let top = area.y0 + gap + inward;
        let bottom = area.y1 - gap - inward - size.height;
        match self {
            Inset::TopLeft => Point::new(left, top),
            Inset::TopRight => Point::new(right, top),
            Inset::BottomLeft => Point::new(left, bottom),
            Inset::BottomRight => Point::new(right, bottom),
        }
    }
}
