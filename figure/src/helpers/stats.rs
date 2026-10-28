//! Statistical charts as marks, and the statistics behind them as pure
//! functions returning plain data: [`boxplot()`] with
//! [`boxplot::summary`]; [`violin()`]; [`hist()`] with [`hist::table`],
//! and histograms stacked over shared edges with [`hist::stacked`].
//!
//! Every chart is added with [`Panel::mark`](crate::Panel::mark) like any
//! mark (stacked histograms add their marks with `add_to`). The statistics
//! are graded against matplotlib, numpy and scipy
//! (`dev/fixtures/stats.py`).

pub mod boxplot;
mod fill;
pub mod hist;
mod keyed;
mod slots;
pub mod violin;

pub use boxplot::chart::{BoxPlot, boxplot};
pub use hist::chart::{Hist, hist};
pub use violin::chart::{Violin, violin};
