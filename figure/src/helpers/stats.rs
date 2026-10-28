//! Statistical charts as marks, and the statistics behind them as pure
//! functions returning plain data: [`boxplot()`] with
//! [`boxplot::summary`].
//!
//! Every chart is added with [`Panel::mark`](crate::Panel::mark) like any
//! mark. Its statistic is graded against matplotlib, numpy and scipy
//! (`dev/fixtures/stats.py`).

pub mod boxplot;
mod fill;
mod slots;

pub use boxplot::chart::{BoxPlot, boxplot};
