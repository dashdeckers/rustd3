//! What [`Bin`](crate::array::Bin) lays values out by and into: the
//! [`Thresholds`] rule, d3's three threshold-count rules, and the
//! [`Bucket`]s.

mod bucket;
mod thresholds;

pub use bucket::Bucket;
pub use thresholds::{Thresholds, freedman_diaconis, scott, sturges};
