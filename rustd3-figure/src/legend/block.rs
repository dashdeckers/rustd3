//! [`Block`]: a guide laid out, and drawing it.

use kurbo::{Point, Rect, Size};

use crate::legend::keys;
use crate::list::List;
use crate::mark::Snap;
use crate::theme::Theme;
use crate::{colorbar, text};

/// A guide laid out, relative to its top-left corner: a legend's keys or a
/// colorbar.
pub(crate) enum Block {
    /// A legend of an ordinal palette's keys.
    Keys(keys::Block),
    /// A colorbar.
    Bar(Box<colorbar::Block>),
}

impl Block {
    /// Its size, px.
    pub(crate) fn size(&self) -> Size {
        match self {
            Block::Keys(block) => block.size(),
            Block::Bar(block) => block.size(),
        }
    }

    /// A colorbar's bar, relative to the top-left corner; `None` for a
    /// legend.
    pub(crate) fn bar(&self) -> Option<Rect> {
        match self {
            Block::Keys(_) => None,
            Block::Bar(block) => Some(block.bar()),
        }
    }

    /// Appends the block with its top-left corner at `at`, with hairlines
    /// and edges on device pixels on raster targets (`snap`).
    pub(crate) fn draw(
        &self,
        at: Point,
        text: &text::Cache,
        theme: &Theme,
        snap: Snap,
        list: &mut List,
    ) {
        match self {
            Block::Keys(block) => block.draw(at, text, theme, list),
            Block::Bar(block) => block.draw(at, text, theme, snap, list),
        }
    }
}
