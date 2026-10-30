//! matplotlib's one-call conveniences, composed from the figure API
//! behind the `helpers` feature: [`subplots`], a grid of panels whose x is
//! shared per column and y per row; in [`stats`] the statistical charts
//! (box plots, violins, histogram options, ECDF, error bars) with the
//! statistics behind them as pure functions; value labels on bars and
//! pies ([`labels`]); size and color legends of a scatter's channels
//! ([`legend`]); axis defaults ([`axes`]); and one-call chart presets
//! ending in a figure or a file ([`chart`]).
//!
//! Each helper is ordinary code over the figure's handles, marks and
//! drawing list, written to be copied and changed: a chart a helper does
//! not offer is a copy of the nearest one. They draw with the theme's
//! sizes, its pixel snapping and the guard band, as the figure's own marks
//! do, and value labels shape their text through the text kit custom
//! marks use ([`Fits::label`](crate::Fits::label),
//! [`Fits::locale`](crate::Fits::locale),
//! [`Fits::theme`](crate::Fits::theme),
//! [`Ctx::label`](crate::mark::Ctx::label),
//! [`Ctx::locale`](crate::mark::Ctx::locale),
//! [`Ctx::line`](crate::mark::Ctx::line)). Two things here reach what code
//! outside this crate cannot: value labels set their text in the theme's
//! label style, and scatter legends lay out through the figure's legend
//! layout.
//!
//! ```
//! use rustd3_figure::helpers::{stats, subplots};
//!
//! let values = vec![1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 4.7, 5.1, 14.0];
//! let mut grid = subplots(1, 2);
//! let (Some(x0), Some(x1), Some(y)) = (grid.x(0), grid.x(1), grid.y(0)) else {
//!     return;
//! };
//! if let Some(left) = grid.panel(0, 0) {
//!     left.mark(stats::boxplot([(0.0, values.clone())], x0, y));
//! }
//! if let Some(right) = grid.panel(0, 1) {
//!     right.mark(stats::boxplot([(0.0, values)], x1, y).notch(true));
//! }
//! let fig = grid.into_figure();
//! ```

pub mod axes;
pub mod chart;
pub mod labels;
pub mod legend;
pub mod stats;
mod subplots;

pub use subplots::{Subplots, subplots};
