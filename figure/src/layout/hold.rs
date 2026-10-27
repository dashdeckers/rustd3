//! [`Areas`]: the plot areas a gesture holds.

use kurbo::{Rect, Size};

/// Plot areas a gesture holds, with the size and panel count they were
/// made for: [`Resolved::areas`](crate::Resolved::areas) takes them at the
/// gesture's start, and
/// [`Figure::resolve_held`](crate::Figure::resolve_held) keeps them while
/// labels change, so the plot does not shift under the pointer.
#[derive(Debug, Clone, PartialEq)]
pub struct Areas {
    rects: Vec<Rect>,
    size: Size,
}

impl Areas {
    /// The plot areas `rects`, one per panel, of a figure laid out for
    /// `size`.
    pub(crate) fn new(rects: Vec<Rect>, size: Size) -> Areas {
        Areas { rects, size }
    }

    /// The plot areas, when they were made for `size` and `panels` panels.
    pub(crate) fn fit(&self, size: Size, panels: usize) -> Option<&[Rect]> {
        (self.size == size && self.rects.len() == panels).then_some(&self.rects)
    }
}
