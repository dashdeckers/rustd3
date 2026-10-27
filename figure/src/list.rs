//! The drawing list every backend renders: seven commands on kurbo and
//! peniko types, the font table their glyph runs index, and the parts a
//! figure divides them into.

mod bounds;
mod check;
mod clip;
mod command;
pub mod dump;
mod error;
mod outline;
mod paints;
pub mod part;
mod run;

use std::ops::Range;

use kurbo::{Cap, Join, Rect, Size};
use peniko::{Brush, Color, FontData};
use rustd3_core::color::Rgb;

use crate::text;

pub use bounds::Bounds;
pub use check::check;
pub use clip::Clip;
pub use command::Command;
pub use error::Error;
pub use outline::Outlines;
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
    let _ = assert::<Bounds>;
};

/// The list as text, for snapshots: a header, one line per command, then
/// one per part (hashes are not dumped).
///
/// ```text
/// size 320x240 fonts 1 parts 2
/// 0 fill M0,0L320,0L320,240L0,240Z #ffffff
/// 1 push matrix(1 0 0 1 40 20) rect(0,0,260,180)
/// 2 stroke M0,90L260,90 w1.5 butt miter 4 dash[6,3]@0 #4c72b0
/// 3 markers n3 bbox(-1,-1,1,1) #dd8452 sizes[1,2,3] paints[2:0,1,0]
/// 4 glyphs font0 11 "Wheel: RL" ids[58,75,72,72,79,29,3,53,47] matrix(1 0 0 1 10 200) #262626
/// 5 image rect(0,0,16,16) 16x16 nearest
/// 6 pop
/// part 0 1..7 bounds(40,20,300,200)
/// ```
pub fn dump(list: &List, options: dump::Options) -> String {
    dump::write(list, options)
}

/// A stroke of `width` with butt caps and miter joins limited at 4, the
/// SVG, tiny-skia and PDF defaults; kurbo's own default is round, which
/// backends would draw differently at every corner.
pub const fn stroke(width: f64) -> kurbo::Stroke {
    kurbo::Stroke::new(width)
        .with_caps(Cap::Butt)
        .with_join(Join::Miter)
        .with_miter_limit(4.0)
}

/// A solid brush of `color`, its channels rounded to 8 bits.
pub(crate) fn brush(color: Rgb) -> Brush {
    let [r, g, b, a] = text::channels(color);
    Brush::Solid(Color::from_rgba8(r, g, b, a))
}
