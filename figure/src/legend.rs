//! Legends: what an axis is to a position scale, a legend is to a palette.
//! A [`Legend`] is a value built once from an ordinal color handle and
//! placed in a figure [`Slot`] outside the grid
//! ([`Figure::legend`](crate::Figure::legend)) or in a corner of a panel
//! ([`Panel::legend`](crate::Panel::legend)); one value may go in both.
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

mod draw;
mod inset;
mod slot;
mod swatch;

use std::fmt::Display;

use rustd3_core::color::Rgb;
use rustd3_core::scale::Ordinal;

use crate::figure::Resolved;
use crate::scale::ScaleRef;
use crate::text::Text;

pub(crate) use draw::{Block, Content};
pub use inset::Inset;
pub use slot::Slot;
pub use swatch::Swatch;

/// A legend of an ordinal palette: one entry per key of its domain, in
/// domain order, a swatch in the key's color (a square unless
/// [`glyph`](Legend::glyph) says otherwise) beside the key's `Display`
/// text, under an optional title.
pub struct Legend<K> {
    palette: ScaleRef<Ordinal<K, Rgb>>,
    title: Option<Text>,
    glyph: Swatch,
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
pub fn legend<K>(palette: ScaleRef<Ordinal<K, Rgb>>) -> Legend<K>
where
    K: PartialEq + Clone + Display + 'static,
{
    Legend {
        palette,
        title: None,
        glyph: Swatch::Square,
    }
}

impl<K> Legend<K> {
    /// A title before the entries: left of a row, above a column.
    pub fn title(self, t: impl Into<Text>) -> Self {
        Legend {
            title: Some(t.into()),
            ..self
        }
    }

    /// What each entry shows in its key's color: a square (the default),
    /// a dot or a line.
    pub fn glyph(self, g: Swatch) -> Self {
        Legend { glyph: g, ..self }
    }
}

/// A legend with its key type erased: what layout and drawing ask of it.
pub(crate) trait Draw {
    /// Each key's color (`None` when the palette has none for it) and
    /// text, in domain order.
    fn entries(&self, resolved: &Resolved) -> Vec<(Option<Rgb>, String)>;

    /// The title.
    fn title(&self) -> Option<&Text>;

    /// What each entry shows.
    fn glyph(&self) -> Swatch;
}

impl<K> Draw for Legend<K>
where
    K: PartialEq + Clone + Display + 'static,
{
    fn entries(&self, resolved: &Resolved) -> Vec<(Option<Rgb>, String)> {
        let palette = resolved.scale(self.palette);
        palette
            .get_domain()
            .iter()
            .map(|key| (palette.apply(key).copied(), key.to_string()))
            .collect()
    }

    fn title(&self) -> Option<&Text> {
        self.title.as_ref()
    }

    fn glyph(&self) -> Swatch {
        self.glyph
    }
}
