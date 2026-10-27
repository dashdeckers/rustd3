//! [`Inset`]: where a legend sits inside a panel.

use kurbo::{Point, Rect, Size};

/// A corner of a panel's plot area. An inset legend is a boxed column
/// 0.5 em from that corner, drawn over the panel's marks.
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
    /// corner of plot area `area`.
    pub(crate) fn place(self, area: Rect, size: Size, gap: f64) -> Point {
        let left = area.x0 + gap;
        let right = area.x1 - gap - size.width;
        let top = area.y0 + gap;
        let bottom = area.y1 - gap - size.height;
        match self {
            Inset::TopLeft => Point::new(left, top),
            Inset::TopRight => Point::new(right, top),
            Inset::BottomLeft => Point::new(left, bottom),
            Inset::BottomRight => Point::new(right, bottom),
        }
    }
}
