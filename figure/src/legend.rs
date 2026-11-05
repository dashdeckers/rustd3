//! Legends: what an axis is to a position scale, a legend is to a palette.
//! A [`Legend`] is a value built once from an ordinal color handle and
//! placed in a figure [`Slot`] outside the grid
//! ([`Figure::legend`](crate::Figure::legend)) or in a corner of a panel
//! ([`Panel::legend`](crate::Panel::legend)); one value may go in both.
//! Both take any [`Guide`]: a legend, or a
//! [`Colorbar`](crate::colorbar::Colorbar) of a continuous or binned color
//! scale.
//!
//! ```
//! use rustd3_core::scale;
//! use rustd3_core::scheme::COLORBLIND;
//! use rustd3_figure::Figure;
//! use rustd3_figure::legend::{Inset, Slot, legend};
//!
//! let mut fig = Figure::new();
//! let color = fig.scale(scale::ordinal(["accel x", "accel z"], COLORBLIND));
//! let key = legend(color).title("source");
//! fig.panel(0, 0).legend(Inset::TopLeft, key.clone());
//! fig.legend(Slot::Bottom, key);
//! ```

mod block;
mod content;
mod entry;
mod guide;
mod inset;
pub(crate) mod keys;
mod slot;
mod swatch;

use std::any::Any;
use std::fmt::Display;

pub(crate) use block::Block;
pub(crate) use content::Content;
pub(crate) use entry::Entry;
pub use guide::Guide;
pub(crate) use guide::{Erased, Stored};
pub use inset::Inset;
use rustd3_core::color::Rgb;
use rustd3_core::scale::Ordinal;
pub use slot::Slot;
pub use swatch::Swatch;

use crate::figure::Resolved;
use crate::scale::ScaleRef;
use crate::text::Text;
use crate::theme::Theme;

/// A legend of an ordinal palette: one entry per key of its domain, in
/// domain order, a swatch in the key's color beside the key's `Display`
/// text, under an optional title. Each key's swatch stands for the
/// figure's marks drawn in its color: a square when one of them is an
/// area, ribbon, bar, histogram, density or rectangle (hatched as the
/// first hatched one is), else a line when one is a line (dashed by the
/// first on and off lengths of its dash), else a dot for dots; a key no
/// mark uses gets a square. [`glyph`](Legend::glyph) sets one swatch for
/// every key.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_core::scheme::COLORBLIND;
/// use rustd3_figure::Figure;
/// use rustd3_figure::legend::{Inset, legend};
/// use rustd3_figure::mark::{dots, line};
///
/// let mut fig = Figure::new();
/// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
/// let color = fig.scale(scale::ordinal(["model", "measured"], COLORBLIND));
/// fig.panel(0, 0)
///     .mark(line([(0.0, 0.0), (1.0, 1.0)], x, y).stroke(color, "model"))
///     .mark(dots([(0.5, 0.6)], x, y).fill(color, "measured"))
///     // A line beside "model", a dot beside "measured".
///     .legend(Inset::TopLeft, legend(color));
/// ```
pub struct Legend<K> {
    palette: ScaleRef<Ordinal<K, Rgb>>,
    title: Option<Text>,
    glyph: Option<Swatch>,
}

impl<K> std::fmt::Debug for Legend<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Legend").finish_non_exhaustive()
    }
}

impl<K> Clone for Legend<K> {
    fn clone(&self) -> Self {
        Legend {
            palette: self.palette,
            title: self.title.clone(),
            glyph: self.glyph,
        }
    }
}

/// A legend of `palette`'s keys.
#[must_use]
pub fn legend<K>(palette: ScaleRef<Ordinal<K, Rgb>>) -> Legend<K>
where
    K: PartialEq + Clone + Display + 'static,
{
    Legend {
        palette,
        title: None,
        glyph: None,
    }
}

impl<K> Legend<K> {
    /// A title before the entries: left of a row, above a column.
    #[must_use]
    pub fn title(self, t: impl Into<Text>) -> Self {
        Legend {
            title: Some(t.into()),
            ..self
        }
    }

    /// What every entry shows in its key's color, a square, a dot or a
    /// line, in place of the swatch of the key's marks.
    #[must_use]
    pub fn glyph(self, g: Swatch) -> Self {
        Legend {
            glyph: Some(g),
            ..self
        }
    }
}

impl<'a, K> Guide<'a> for Legend<K>
where
    K: PartialEq + Clone + Display + 'static,
{
    fn erase(self) -> Erased<'a> {
        Erased(Stored::Keys(Box::new(self)))
    }
}

/// A legend with its key type erased: what layout and drawing ask of it.
pub(crate) trait Draw {
    /// The handle it is drawn from.
    fn scale(&self) -> ScaleRef<()>;

    /// The entries over the resolved scales, in `theme`'s colors: for a
    /// legend of an ordinal palette, each key's color (a square's at the
    /// most opaque fill of its marks), text, swatch and the hatch of the marks
    /// filled with it, in domain order.
    fn entries(&self, resolved: &Resolved, theme: &Theme) -> Vec<Entry>;

    /// The title.
    fn title(&self) -> Option<&Text>;
}

impl<K> Draw for Legend<K>
where
    K: PartialEq + Clone + Display + 'static,
{
    fn scale(&self) -> ScaleRef<()> {
        self.palette.erased()
    }

    fn entries(&self, resolved: &Resolved, _theme: &Theme) -> Vec<Entry> {
        let Some(palette) = resolved.get_scale(self.palette) else {
            return Vec::new();
        };
        palette
            .get_domain()
            .iter()
            .map(|key| {
                let index = self.palette.get_index();
                let is = |k: &dyn Any| k.downcast_ref::<K>() == Some(key);
                let glyph = self
                    .glyph
                    .or_else(|| resolved.glyph(index, is))
                    .unwrap_or_default();
                // A square repeats the most opaque fill of its marks.
                let shown = |c: Rgb| match glyph {
                    Swatch::Square => c.with_opacity(resolved.opacity(index, is, c.opacity)),
                    Swatch::Dot | Swatch::Line { .. } => c,
                };
                Entry {
                    color: palette.apply(key).copied().map(shown),
                    label: key.to_string(),
                    glyph,
                    hatch: resolved.hatch(index, is).cloned(),
                    size: None,
                }
            })
            .collect()
    }

    fn title(&self) -> Option<&Text> {
        self.title.as_ref()
    }
}
