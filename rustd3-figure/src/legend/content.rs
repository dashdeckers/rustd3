//! [`Content`]: a guide over one resolve, and how it is measured and laid
//! out in a figure slot or a panel corner.

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
pub(crate) enum Content<'g, 'a> {
    /// A legend of an ordinal palette's keys.
    Keys(keys::Content),
    /// A colorbar.
    Bar(Box<colorbar::Content<'g, 'a>>),
}

impl<'g, 'a> Content<'g, 'a> {
    /// `stored` over the resolved scales, in `theme`'s styles.
    pub(crate) fn new(stored: &'g Stored<'a>, resolved: &Resolved, theme: &Theme) -> Self {
        match stored {
            Stored::Keys(legend) => Content::Keys(keys::Content::new(&**legend, resolved, theme)),
            Stored::Bar(bar) => Content::Bar(Box::new(colorbar::Content::new(&**bar, resolved))),
        }
    }

    /// Every key, for the text cache's batch: a colorbar's as last
    /// measured.
    pub(crate) fn keys(&self) -> Box<dyn Iterator<Item = &Key> + '_> {
        match self {
            Content::Keys(content) => Box::new(content.keys()),
            Content::Bar(content) => Box::new(content.keys()),
        }
    }

    /// Measures a colorbar for a pass in figure slot `slot` beside plot
    /// areas whose union is `extent`, and returns its tick count
    /// ([`colorbar::Content::measure`]). `None` for a legend, which needs
    /// no measuring.
    pub(crate) fn measure_slot(
        &mut self,
        slot: Slot,
        extent: Rect,
        resolved: &Resolved,
        theme: &Theme,
        text: &text::Cache,
    ) -> Option<usize> {
        let Content::Bar(content) = self else {
            return None;
        };
        let (orient, length) = along(slot, extent);
        Some(content.measure(orient, length, resolved, theme, text))
    }

    /// Measures a colorbar for a pass in a corner of plot area `area`, as
    /// [`measure_slot`](Content::measure_slot) does.
    pub(crate) fn measure_inset(
        &mut self,
        area: Rect,
        resolved: &Resolved,
        theme: &Theme,
        text: &text::Cache,
    ) -> Option<usize> {
        let Content::Bar(content) = self else {
            return None;
        };
        let orient = corner(area);
        Some(content.measure(orient, inset(orient, area), resolved, theme, text))
    }

    /// The guide laid out for figure slot `slot` beside plot areas whose
    /// union is `extent`. A legend is a row on top and bottom, and a column
    /// at the sides. A colorbar runs along the side, as long as `extent`
    /// along it, with its ticks away from the grid.
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
                let (orient, length) = along(slot, extent);
                Block::Bar(Box::new(content.block(orient, length, text, theme)))
            }
        }
    }

    /// The guide laid out for a corner of plot area `area`, framed. A
    /// legend is a column. A colorbar runs along the area's longer side, at
    /// 40% of its length, in the orientation last measured. That is
    /// vertical with ticks on the right when the area is at least as tall
    /// as wide, else horizontal with ticks below.
    pub(crate) fn inset(&self, area: Rect, text: &text::Cache, theme: &Theme) -> Block {
        let em = theme.em();
        match self {
            Content::Keys(content) => Block::Keys(content.column(text, theme).framed(em)),
            Content::Bar(content) => {
                let orient = content.orient().unwrap_or_else(|| corner(area));
                let block = content.block(orient, inset(orient, area), text, theme);
                Block::Bar(Box::new(block.framed(em)))
            }
        }
    }
}

/// The side of a colorbar's bar its ticks take in figure slot `slot`, and
/// its length beside plot areas whose union is `extent`.
fn along(slot: Slot, extent: Rect) -> (Orient, f64) {
    match slot {
        Slot::Top => (Orient::Top, extent.width()),
        Slot::Bottom => (Orient::Bottom, extent.width()),
        Slot::Left => (Orient::Left, extent.height()),
        Slot::Right => (Orient::Right, extent.height()),
    }
}

/// The side an inset colorbar's ticks take in a corner of plot area
/// `area`: right of a vertical bar when the area is at least as tall as
/// wide, else below a horizontal one.
fn corner(area: Rect) -> Orient {
    if area.height() >= area.width() {
        Orient::Right
    } else {
        Orient::Bottom
    }
}

/// An inset colorbar's length in plot area `area`, its ticks on side
/// `orient`: 40% of the side it runs along.
fn inset(orient: Orient, area: Rect) -> f64 {
    match orient {
        Orient::Left | Orient::Right => INSET * area.height(),
        Orient::Top | Orient::Bottom => INSET * area.width(),
    }
}
