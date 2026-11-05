//! [`Subplots`]: a figure with one panel per cell of a grid, x shared per
//! column and y per row.

use rustd3_core::scale::{self, Linear};

use crate::scale::ScaleRef;
use crate::{Figure, Panel, axis};

/// A figure with one panel per cell of a grid and its handles: one fitted
/// linear x per column and one fitted linear y per row. For other scale
/// kinds, copy [`subplots`].
pub struct Subplots<'a> {
    fig: Figure<'a>,
    x: Vec<ScaleRef<Linear>>,
    y: Vec<ScaleRef<Linear>>,
}

impl<'a> std::fmt::Debug for Subplots<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subplots").finish_non_exhaustive()
    }
}

/// One panel per cell of a `rows × cols` grid, with x fitted and shared
/// per column, y per row, and bottom and left axes on every panel. The
/// layout labels only the outer axes (the bottom row's x, the left
/// column's y), as matplotlib's `subplots(sharex="col", sharey="row")`
/// does.
///
/// ```
/// use rustd3_figure::helpers::subplots;
/// use rustd3_figure::mark::dots;
///
/// let mut grid = subplots(2, 3);
/// for row in 0..2 {
///     for col in 0..3 {
///         let (Some(x), Some(y)) = (grid.get_x(col), grid.get_y(row)) else {
///             continue;
///         };
///         let k = (row * 3 + col) as f64;
///         if let Some(panel) = grid.panel(row, col) {
///             panel.mark(dots([(k, 1.0), (k + 1.0, 2.0)], x, y));
///         }
///     }
/// }
/// grid.figure().title("Six panels, three x and two y scales");
/// let fig = grid.into_figure();
/// ```
#[must_use]
pub fn subplots<'a>(rows: usize, cols: usize) -> Subplots<'a> {
    let mut fig = Figure::new();
    let x: Vec<_> = (0..cols).map(|_| fig.fit(scale::linear())).collect();
    let y: Vec<_> = (0..rows).map(|_| fig.fit(scale::linear())).collect();
    for (row, &y) in y.iter().enumerate() {
        for (col, &x) in x.iter().enumerate() {
            fig.panel(row, col)
                .axis(axis::bottom(x))
                .axis(axis::left(y));
        }
    }
    Subplots { fig, x, y }
}

impl<'a> Subplots<'a> {
    /// Column `col`'s x handle; `None` past the last column.
    #[must_use]
    pub fn get_x(&self, col: usize) -> Option<ScaleRef<Linear>> {
        self.x.get(col).copied()
    }

    /// Row `row`'s y handle; `None` past the last row.
    #[must_use]
    pub fn get_y(&self, row: usize) -> Option<ScaleRef<Linear>> {
        self.y.get(row).copied()
    }

    /// The panel in cell `(row, col)`; `None` for a cell outside the grid.
    pub fn panel(&mut self, row: usize, col: usize) -> Option<&mut Panel<'a>> {
        (row < self.y.len() && col < self.x.len()).then(|| self.fig.panel(row, col))
    }

    /// The figure, for its title, theme, legends or more scales.
    pub fn figure(&mut self) -> &mut Figure<'a> {
        &mut self.fig
    }

    /// The figure, done.
    #[must_use]
    pub fn into_figure(self) -> Figure<'a> {
        self.fig
    }
}
