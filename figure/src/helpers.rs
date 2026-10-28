//! matplotlib's one-call conveniences, composed from the figure API
//! behind the `helpers` feature: [`subplots`], a grid of panels whose x is
//! shared per column and y per row; in [`stats`] the statistical charts
//! (box plots, violins, histogram options, ECDF, error bars) with the
//! statistics behind them as pure functions; value labels on bars and
//! pies ([`labels`]); and size and color legends of a scatter's channels
//! ([`legend`]).
//!
//! Each helper is ordinary code over the figure's handles, marks and
//! drawing list, written to be copied and changed: a chart a helper does
//! not offer is a copy of the nearest one. They draw with the theme's
//! sizes, its pixel snapping and the guard band, as the figure's own marks
//! do. Value labels and scatter legends are the exception: they shape
//! their text through the figure and lay out through its legend layout,
//! which code outside this crate cannot reach.
//!
//! ```
//! use rustd3_figure::helpers::{stats, subplots};
//!
//! let values = vec![1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 4.7, 5.1, 14.0];
//! let mut grid = subplots(1, 2);
//! let (x0, x1, y) = (grid.x(0), grid.x(1), grid.y(0));
//! grid.panel(0, 0).mark(stats::boxplot([(0.0, values.clone())], x0, y));
//! grid.panel(0, 1).mark(stats::boxplot([(0.0, values)], x1, y).notch(true));
//! let fig = grid.into_figure();
//! ```

pub mod axes;
pub mod labels;
pub mod legend;
pub mod stats;
mod subplots;

pub use subplots::{Subplots, subplots};
