//! Statistical charts as marks, and the statistics behind them as pure
//! functions returning plain data.
//!
//! - [`boxplot()`], with its statistics in [`boxplot::summary`].
//! - [`violin()`].
//! - [`hist()`], with its bins in [`hist::table`], and histograms stacked
//!   over shared edges with [`hist::stacked`].
//! - [`ecdf()`].
//! - [`errorbar()`].
//!
//! Every chart is added with [`Panel::mark`](crate::Panel::mark) like any
//! mark (stacked histograms add their marks with `add_to`). The statistics
//! are tested against matplotlib, numpy and scipy, using the fixture
//! `tests/fixtures/stats.json`.

pub mod boxplot;
mod ecdf;
pub mod errorbar;
mod fill;
pub mod hist;
mod keyed;
mod slots;
pub mod violin;

pub use boxplot::chart::{BoxPlot, boxplot};
pub use ecdf::{Ecdf, ecdf};
pub use errorbar::chart::{ErrorBar, errorbar};
pub use hist::chart::{Hist, hist};
pub use violin::chart::{Violin, violin};
