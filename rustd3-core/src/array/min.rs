//! d3.min.

/// d3.min: the least of `values` that is not NaN; `None` when there is
/// none, where d3 returns undefined.
pub fn min(values: impl IntoIterator<Item = f64>) -> Option<f64> {
    let mut min = None;
    for v in values {
        // d3's `min > value || (min === undefined && value >= value)`.
        if min.map_or(!v.is_nan(), |m| m > v) {
            min = Some(v);
        }
    }
    min
}
