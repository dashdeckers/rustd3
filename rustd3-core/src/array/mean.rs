//! d3.mean.

/// d3.mean: the sum of `values` that are not NaN over their count; `None`
/// when there is none, where d3 returns undefined.
pub fn mean(values: impl IntoIterator<Item = f64>) -> Option<f64> {
    let (count, sum) = values
        .into_iter()
        .filter(|v| !v.is_nan())
        .fold((0usize, 0.0), |(count, sum), v| (count + 1, sum + v));
    (count > 0).then(|| sum / count as f64)
}
