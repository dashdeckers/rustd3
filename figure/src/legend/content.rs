//! [`Content`]: a guide over one resolve, and how it is laid out in a
//! figure slot or a panel corner.

use kurbo::Rect;
use rustd3_core::axis::Orient;

use crate::colorbar;
use crate::figure::Resolved;
use crate::legend::{Block, Slot, Stored, keys};
use crate::text::{self, Key};
use crate::theme::Theme;

/// An inset colorbar's length, as a share of its panel's long side.
const INSET: f64 = 0.4;

/// A guide over one resolve: a legend's keys or a colorbar.
pub(crate) enum Content {
    /// A legend of an ordinal palette's keys.
    Keys(keys::Content),
    /// A colorbar.
    Bar(colorbar::Content),
}

impl Content {
    /// `stored` over the resolved scales, in `theme`'s styles.
    pub(crate) fn new(stored: &Stored<'_>, resolved: &Resolved, theme: &Theme) -> Content {
        match stored {
            Stored::Keys(legend) => Content::Keys(keys::Content::new(&**legend, resolved, theme)),
            Stored::Bar(bar) => Content::Bar(colorbar::Content::new(&**bar, resolved)),
        }
    }

    /// Every key, for the text cache's batch.
    pub(crate) fn keys(&self) -> Box<dyn Iterator<Item = &Key> + '_> {
        match self {
            Content::Keys(content) => Box::new(content.keys()),
            Content::Bar(_) => Box::new(std::iter::empty()),
        }
    }

    /// The guide laid out for figure slot `slot` beside plot areas whose
    /// union is `extent`: legends as a row on top and bottom, a column at
    /// the sides; colorbars along the side, as long as `extent` across it.
    pub(crate) fn slotted(
        &self,
        slot: Slot,
        extent: Rect,
        text: &text::Cache,
        theme: &Theme,
    ) -> Block {
        match self {
            Content::Keys(content) => Block::Keys(match slot {
                Slot::Top | Slot::Bottom => content.row(text, theme),
                Slot::Left | Slot::Right => content.column(text, theme),
            }),
            Content::Bar(content) => {
                let (orient, length) = match slot {
                    Slot::Top => (Orient::Top, extent.width()),
                    Slot::Bottom => (Orient::Bottom, extent.width()),
                    Slot::Left => (Orient::Left, extent.height()),
                    Slot::Right => (Orient::Right, extent.height()),
                };
                Block::Bar(content.block(orient, length, theme))
            }
        }
    }

    /// The guide laid out for a corner of plot area `area`, framed:
    /// legends as a column, colorbars along the area's longer side
    /// (vertical with ticks on the right when it is at least as tall as
    /// wide, else horizontal with ticks below), 40% of its length.
    pub(crate) fn inset(&self, area: Rect, text: &text::Cache, theme: &Theme) -> Block {
        let em = theme.em();
        match self {
            Content::Keys(content) => Block::Keys(content.column(text, theme).framed(em)),
            Content::Bar(content) => {
                let (orient, side) = if area.height() >= area.width() {
                    (Orient::Right, area.height())
                } else {
                    (Orient::Bottom, area.width())
                };
                Block::Bar(content.block(orient, INSET * side, theme).framed(em))
            }
        }
    }
}
