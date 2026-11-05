//! d3.median.

use crate::array::quantile;

/// d3.median: [`quantile`] at 0.5.
pub fn median(values: impl IntoIterator<Item = f64>) -> Option<f64> {
    quantile(values, 0.5)
}
