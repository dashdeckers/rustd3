//! matplotlib-style one-call conveniences built from the figure API, behind
//! the `helpers` feature.
//!
//! - [`subplots`]: a grid of panels, x shared per column and y per row.
//! - [`stats`]: statistical charts (box plots, violins, histograms, ECDF,
//!   error bars) and the statistics behind them as pure functions.
//! - [`labels`]: value labels on bars and pies.
//! - [`legend`]: size and color legends of a scatter's channels.
//! - [`chart`]: one-call chart presets that end in a figure or a file.
//!
//! Each helper is ordinary code over the figure's handles, marks and
//! drawing list, written to be copied and changed. For a chart no helper
//! offers, copy the nearest one. Helpers draw with the theme's sizes, its
//! pixel snapping and the guard band, as the figure's own marks do. Value
//! labels shape their text with the same kit custom marks use
//! ([`Fits::label`](crate::Fits::label),
//! [`Fits::get_locale`](crate::Fits::get_locale),
//! [`Fits::get_theme`](crate::Fits::get_theme),
//! [`Ctx::get_label`](crate::mark::Ctx::get_label),
//! [`Ctx::get_locale`](crate::mark::Ctx::get_locale),
//! [`Ctx::line`](crate::mark::Ctx::line)). Only two things here use what
//! code outside this crate cannot reach: value labels set their text in the
//! theme's label style, and scatter legends use the figure's legend layout.
//!
//! ```
//! use rustd3_figure::helpers::{stats, subplots};
//!
//! let values = vec![1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 4.7, 5.1, 14.0];
//! let mut grid = subplots(1, 2);
//! let (Some(x0), Some(x1), Some(y)) = (grid.get_x(0), grid.get_x(1), grid.get_y(0)) else {
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

pub mod chart;
pub mod labels;
pub mod legend;
pub mod stats;
mod subplots;

pub use subplots::{Subplots, subplots};
