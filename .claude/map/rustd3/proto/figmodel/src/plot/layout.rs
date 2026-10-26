//! A weighted grid with per-boundary gaps. A stand-in for
//! problem #3, which will measure axis protrusions instead
//! of taking fixed margins.

/// The default theme's em, for code that has no theme.
pub const EM: f64 = 11.0 * 4.0 / 3.0;

/// A gap length: pixels, or multiples of the text size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    Px(f64),
    Em(f64),
}

pub fn px(v: f64) -> Length {
    Length::Px(v)
}

pub fn em(v: f64) -> Length {
    Length::Em(v)
}

impl Length {
    pub fn to_px(self, em: f64) -> f64 {
        match self {
            Length::Px(v) => v,
            Length::Em(v) => v * em,
        }
    }
}

impl From<f64> for Length {
    fn from(v: f64) -> Self {
        Length::Px(v)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// Column and row sizes are relative weights; gaps and
/// margins are pixels.
#[derive(Debug, Clone)]
pub struct Grid {
    cols: Vec<f64>,
    rows: Vec<f64>,
    col_gaps: Vec<Length>,
    row_gaps: Vec<Length>,
    /// Top, right, bottom, left; `None` measures them.
    margin: Option<[f64; 4]>,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            cols: vec![],
            rows: vec![],
            col_gaps: vec![],
            row_gaps: vec![],
            margin: None,
        }
    }
}

impl Grid {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cols(self, w: impl IntoIterator<Item = f64>) -> Self {
        Self { cols: w.into_iter().collect(), ..self }
    }

    pub fn rows(self, h: impl IntoIterator<Item = f64>) -> Self {
        Self { rows: h.into_iter().collect(), ..self }
    }

    /// Gap `i` is the whitespace between column `i` and
    /// `i + 1`, measured between their decorated boxes.
    pub fn col_gaps<L: Into<Length>>(
        self,
        g: impl IntoIterator<Item = L>,
    ) -> Self {
        let col_gaps = g.into_iter().map(Into::into);
        Self { col_gaps: col_gaps.collect(), ..self }
    }

    pub fn row_gaps<L: Into<Length>>(
        self,
        g: impl IntoIterator<Item = L>,
    ) -> Self {
        let row_gaps = g.into_iter().map(Into::into);
        Self { row_gaps: row_gaps.collect(), ..self }
    }

    /// Fixed margins instead of measured protrusions.
    pub fn margin(self, margin: [f64; 4]) -> Self {
        Self { margin: Some(margin), ..self }
    }

    pub(crate) fn fixed(&self) -> Option<[f64; 4]> {
        self.margin
    }

    pub(crate) fn weights(&self) -> [&[f64]; 2] {
        [&self.rows, &self.cols]
    }

    pub(crate) fn gaps(&self, em: f64) -> [Vec<f64>; 2] {
        let px = |g: &[Length]| g.iter().map(|l| l.to_px(em)).collect();
        [px(&self.row_gaps), px(&self.col_gaps)]
    }

    pub(crate) fn rects(
        &self,
        [w, h]: [f64; 2],
        cells: &[(usize, usize)],
        [extra_top, extra_bottom]: [f64; 2],
        [top, right, bottom, left]: [f64; 4],
    ) -> Vec<Rect> {
        let n_rows = cells.iter().map(|c| c.0 + 1).max();
        let n_cols = cells.iter().map(|c| c.1 + 1).max();
        let (top, bottom) = (top + extra_top, bottom + extra_bottom);
        let [row_gaps, col_gaps] = self.gaps(EM);
        let xs = spans(
            &self.cols,
            &col_gaps,
            n_cols.unwrap_or(1),
            left,
            w - left - right,
        );
        let ys = spans(
            &self.rows,
            &row_gaps,
            n_rows.unwrap_or(1),
            top,
            h - top - bottom,
        );
        cells
            .iter()
            .map(|&(r, c)| Rect {
                x: xs[c].0,
                y: ys[r].0,
                w: xs[c].1,
                h: ys[r].1,
            })
            .collect()
    }
}

/// Start and length of each track.
fn spans(
    weights: &[f64],
    gaps: &[f64],
    n: usize,
    start: f64,
    total: f64,
) -> Vec<(f64, f64)> {
    let weight = |i: usize| weights.get(i).copied().unwrap_or(1.0);
    let gap = |i: usize| gaps.get(i).copied().unwrap_or(16.0);
    let free = total - (0..n.saturating_sub(1)).map(gap).sum::<f64>();
    let sum: f64 = (0..n).map(weight).sum();
    let mut at = start;
    (0..n)
        .map(|i| {
            let len = free * weight(i) / sum;
            let span = (at, len);
            at += len + if i + 1 < n { gap(i) } else { 0.0 };
            span
        })
        .collect()
}
