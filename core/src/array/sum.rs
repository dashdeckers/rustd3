//! d3.sum.

/// d3.sum: the sum of `values` that are not NaN, added in order; 0 when
/// there is none.
pub fn sum(values: impl IntoIterator<Item = f64>) -> f64 {
    values
        .into_iter()
        .filter(|v| !v.is_nan())
        .fold(0.0, |sum, v| sum + v)
}
