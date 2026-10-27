//! Layout by protrusions: the [`Grid`] of weighted tracks and the gaps
//! between them, as [`Length`]s in px or [`em`].
//!
//! Each column takes the largest reach of its panels' decorations past
//! their plot areas on each side, each row likewise; reaches, gaps and
//! padding are fixed, the rest is split by weight, and positions are
//! running sums, so panels of one column share `x` and `w` exactly.
//!
//! ```
//! use rustd3_figure::layout::{Grid, em};
//!
//! // The wheel figure's columns: wide scatters, narrow side panels, small
//! // gaps inside a block and a gutter between blocks.
//! let grid = Grid::new()
//!     .cols([1.0, 0.25, 1.0, 0.25])
//!     .col_gaps([em(0.4), em(1.6), em(0.4)]);
//! # let _ = grid;
//! ```

mod cell;
pub(crate) mod collide;
mod hold;
mod passes;
mod plan;
mod protrusion;
mod slots;
mod snap;
mod tracks;

pub use cell::Cell;
pub use hold::Areas;
pub(crate) use passes::{Laid, Layout};
pub use plan::Plan;
pub(crate) use protrusion::Reach;
pub(crate) use snap::Snap;
pub(crate) use tracks::tracks;

/// The canvas's padding, em.
pub(crate) const PAD: f64 = 0.8;

/// From a panel's corner to an inset legend, em: matplotlib's
/// `legend.borderaxespad`.
pub(crate) const INSET: f64 = 0.5;

/// A length in px, or in multiples of the theme's tick-label size, so gaps
/// scale with the text. Plain data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    /// Exact pixels.
    Px(f64),
    /// Multiples of the tick-label size (14.667 px at notebook).
    Em(f64),
}

/// `x` times the theme's tick-label size.
pub fn em(x: f64) -> Length {
    Length::Em(x)
}

/// `x` px exactly.
pub fn px(x: f64) -> Length {
    Length::Px(x)
}

impl Length {
    /// The length in px, where an em is `em` px.
    pub(crate) fn to_px(self, em: f64) -> f64 {
        match self {
            Length::Px(v) => v,
            Length::Em(v) => v * em,
        }
    }
}

/// The figure's grid: relative weights of rows and columns, and the gaps
/// between neighbouring tracks, measured as the whitespace between their
/// decorated boxes. A track without a weight weighs 1; a missing gap is
/// `em(1.0)`. The grid has as many tracks as it has weights or as its
/// panels need, whichever is more.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Grid {
    rows: Vec<f64>,
    cols: Vec<f64>,
    row_gaps: Vec<Length>,
    col_gaps: Vec<Length>,
}

impl Grid {
    /// Equal tracks with `em(1.0)` gaps.
    pub fn new() -> Self {
        Grid::default()
    }

    /// The rows' relative heights, top to bottom.
    pub fn rows(self, weights: impl IntoIterator<Item = f64>) -> Self {
        Grid {
            rows: weights.into_iter().collect(),
            ..self
        }
    }

    /// The columns' relative widths, left to right.
    pub fn cols(self, weights: impl IntoIterator<Item = f64>) -> Self {
        Grid {
            cols: weights.into_iter().collect(),
            ..self
        }
    }

    /// Gap `i` is the whitespace between row `i` and row `i + 1`.
    pub fn row_gaps(self, gaps: impl IntoIterator<Item = Length>) -> Self {
        Grid {
            row_gaps: gaps.into_iter().collect(),
            ..self
        }
    }

    /// Gap `i` is the whitespace between column `i` and column `i + 1`.
    pub fn col_gaps(self, gaps: impl IntoIterator<Item = Length>) -> Self {
        Grid {
            col_gaps: gaps.into_iter().collect(),
            ..self
        }
    }

    /// The row weights.
    pub(crate) fn row_weights(&self) -> &[f64] {
        &self.rows
    }

    /// The column weights.
    pub(crate) fn col_weights(&self) -> &[f64] {
        &self.cols
    }

    /// The `tracks − 1` row gaps in px, missing ones `em(1.0)`.
    pub(crate) fn row_gaps_px(&self, tracks: usize, em: f64) -> Vec<f64> {
        gaps_px(&self.row_gaps, tracks, em)
    }

    /// The `tracks − 1` column gaps in px, missing ones `em(1.0)`.
    pub(crate) fn col_gaps_px(&self, tracks: usize, em: f64) -> Vec<f64> {
        gaps_px(&self.col_gaps, tracks, em)
    }
}

/// The `tracks − 1` gaps of `gaps` in px, missing ones `em(1.0)`.
fn gaps_px(gaps: &[Length], tracks: usize, em: f64) -> Vec<f64> {
    (0..tracks.saturating_sub(1))
        .map(|i| gaps.get(i).copied().unwrap_or(Length::Em(1.0)).to_px(em))
        .collect()
}
