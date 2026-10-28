//! [`Subplots`]: a figure with one panel per cell of a grid, x shared per
//! column and y per row.

use rustd3_core::scale::{self, Linear};

use crate::axis;
use crate::scale::ScaleRef;
use crate::{Figure, Panel};

/// A figure with one panel per cell of a grid and its handles: one fitted
/// linear x per column and one fitted linear y per row. Other scale kinds
/// come from copying [`subplots`].
pub struct Subplots<'a> {
    fig: Figure<'a>,
    x: Vec<ScaleRef<Linear>>,
    y: Vec<ScaleRef<Linear>>,
}

/// One panel per cell of a `rows × cols` grid; x fitted and shared per
/// column, y per row, with bottom and left axes on every panel. Layout
/// labels only the outer ones (the bottom row's x, the left column's y),
/// as matplotlib's `subplots(sharex="col", sharey="row")` does.
///
/// ```
/// use rustd3_figure::helpers::subplots;
/// use rustd3_figure::mark::dots;
///
/// let mut grid = subplots(2, 3);
/// for row in 0..2 {
///     for col in 0..3 {
///         let (x, y) = (grid.x(col), grid.y(row));
///         let k = (row * 3 + col) as f64;
///         grid.panel(row, col).mark(dots([(k, 1.0), (k + 1.0, 2.0)], x, y));
///     }
/// }
/// grid.figure().title("Six panels, three x and two y scales");
/// let fig = grid.into_figure();
/// ```
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
    /// Column `col`'s x handle.
    ///
    /// # Panics
    ///
    /// When `col` is not below the column count.
    pub fn x(&self, col: usize) -> ScaleRef<Linear> {
        self.x[col]
    }

    /// Row `row`'s y handle.
    ///
    /// # Panics
    ///
    /// When `row` is not below the row count.
    pub fn y(&self, row: usize) -> ScaleRef<Linear> {
        self.y[row]
    }

    /// The panel in cell `(row, col)`. Debug builds panic for a cell
    /// outside the grid, where a release build makes a panel without axes.
    pub fn panel(&mut self, row: usize, col: usize) -> &mut Panel<'a> {
        debug_assert!(
            row < self.y.len() && col < self.x.len(),
            "a cell outside the grid"
        );
        self.fig.panel(row, col)
    }

    /// The figure, for its title, theme, legends or more scales.
    pub fn figure(&mut self) -> &mut Figure<'a> {
        &mut self.fig
    }

    /// The figure, done.
    pub fn into_figure(self) -> Figure<'a> {
        self.fig
    }
}
