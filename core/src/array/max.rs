//! d3.max.

/// d3.max: the greatest of `values` that is not NaN; `None` when there is
/// none, where d3 returns undefined.
pub fn max(values: impl IntoIterator<Item = f64>) -> Option<f64> {
    let mut max = None;
    for v in values {
        // d3's `max < value || (max === undefined && value >= value)`.
        if max.map_or(!v.is_nan(), |m| m < v) {
            max = Some(v);
        }
    }
    max
}
