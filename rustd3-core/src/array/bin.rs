//! The inputs and outputs of [`Bin`](crate::array::Bin): the
//! [`Thresholds`] rule, d3's three threshold-count rules, and the
//! [`Bucket`]s it fills.

mod bucket;
mod thresholds;

pub use bucket::Bucket;
pub use thresholds::{Thresholds, freedman_diaconis, scott, sturges};
