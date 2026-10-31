//! d3.bisectLeft and d3.bisectRight over ascending numbers.

/// d3.bisectLeft: where `x` would go in the ascending `sorted` to keep it
/// sorted, before any equal values. NaN elements, which compare as neither
/// less nor more, stay to the right; a NaN `x` gives `sorted.len()`.
#[doc(alias = "bisectLeft")]
#[must_use]
pub fn bisect_left(sorted: &[f64], x: f64) -> usize {
    bisect(sorted, x, |v| v < x)
}

/// d3.bisectRight (d3.bisect): as [`bisect_left`], after any equal values.
#[doc(alias = "bisectRight")]
#[must_use]
pub fn bisect_right(sorted: &[f64], x: f64) -> usize {
    bisect(sorted, x, |v| v <= x)
}

/// bisector.js's loop: the first index whose value is not `before` `x`.
fn bisect(sorted: &[f64], x: f64, before: impl Fn(f64) -> bool) -> usize {
    let (mut lo, mut hi) = (0, sorted.len());
    // d3 returns `hi` when `x` does not compare equal to itself.
    if x.is_nan() {
        return hi;
    }
    while lo < hi {
        let mid = (lo + hi) / 2;
        if before(sorted[mid]) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}
