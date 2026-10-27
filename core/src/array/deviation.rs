//! d3.deviation.

use crate::array::variance;

/// d3.deviation: the square root of [`variance`]; `None` for fewer than two
/// values that are not NaN.
pub fn deviation(values: impl IntoIterator<Item = f64>) -> Option<f64> {
    variance(values).map(f64::sqrt)
}
