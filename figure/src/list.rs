//! The drawing list every backend renders: seven commands on kurbo and
//! peniko types, the font table their glyph runs index, and the parts a
//! figure divides them into.

mod check;
mod clip;
mod command;
mod error;
mod paints;
pub mod part;
mod run;

use std::ops::Range;

use kurbo::{Cap, Join, Rect, Size};
use peniko::FontData;

pub use check::check;
pub use clip::Clip;
pub use command::Command;
pub use error::Error;
pub use paints::Paints;
pub use run::{Glyph, GlyphRun};

/// A drawing list: what a figure draws, in logical px, for any backend.
///
/// Every backend validates a list with [`check`] before drawing it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct List {
    /// Logical size in px; rasters multiply by their scale.
    pub size: Size,
    /// The fonts glyph runs index.
    pub fonts: Vec<FontData>,
    /// The commands, drawn in order.
    pub commands: Vec<Command>,
    /// The figure's elements: ordered, non-overlapping command ranges.
    pub parts: Vec<Part>,
}

/// One figure element (a background, a mark, an axis, a title, a legend)
/// in a list: its commands, their bounds and a hash of what it drew.
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    /// The element's identity, stable across draws.
    pub id: part::Id,
    /// Commands `start..end`, Push/Pop balanced inside.
    pub commands: Range<usize>,
    /// Union of the commands' bounds, in list px.
    pub bounds: Rect,
    /// FNV-1a 64 of what the part drew. The figure's list builder fills
    /// it from Phase 17; until then, and in hand-built lists, it is 0.
    pub hash: u64,
}

// Phase 19 moves `Arc<List>` to a render worker.
const _: () = {
    fn assert<T: Send + Sync>() {}
    let _ = assert::<List>;
};

/// A stroke of `width` with butt caps and miter joins limited at 4, the
/// SVG, tiny-skia and PDF defaults; kurbo's own default is round, which
/// backends would draw differently at every corner.
pub const fn stroke(width: f64) -> kurbo::Stroke {
    kurbo::Stroke::new(width)
        .with_caps(Cap::Butt)
        .with_join(Join::Miter)
        .with_miter_limit(4.0)
}
