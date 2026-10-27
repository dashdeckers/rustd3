//! d3.variance.

/// d3.variance: the unbiased sample variance (over n − 1) of `values` that
/// are not NaN, by Welford's algorithm, which neither overflows on large
/// values nor loses precision to a large mean; `None` for fewer than two,
/// where d3 returns undefined.
pub fn variance(values: impl IntoIterator<Item = f64>) -> Option<f64> {
    let (mut count, mut mean, mut sum) = (0usize, 0.0, 0.0);
    for v in values.into_iter().filter(|v| !v.is_nan()) {
        count += 1;
        let delta = v - mean;
        mean += delta / count as f64;
        sum += delta * (v - mean);
    }
    (count > 1).then(|| sum / (count - 1) as f64)
}
