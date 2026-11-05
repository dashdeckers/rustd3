//! d3.count.

/// d3.count: how many of `values` are not NaN (d3 skips null, undefined
/// and NaN).
pub fn count(values: impl IntoIterator<Item = f64>) -> usize {
    values.into_iter().filter(|v| !v.is_nan()).count()
}
