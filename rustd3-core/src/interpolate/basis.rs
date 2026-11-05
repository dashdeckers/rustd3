//! d3.interpolateBasis: a uniform cubic B-spline through values.

use std::sync::Arc;

/// d3.interpolateBasis: a uniform cubic B-spline through `values` at `t` in
/// [0, 1], with `t` outside held at the ends. The outer neighbors of the
/// ends are reflected, so the spline starts at the first value and ends at
/// the last. Fewer than two values give NaN, as d3's undefined arithmetic
/// does.
///
/// ```
/// use rustd3_core::interpolate;
///
/// let spline = interpolate::basis(&[0.0, 10.0, 0.0]);
/// assert_eq!(spline(0.5), 6.666666666666667);
/// assert_eq!((spline(0.0), spline(1.0)), (0.0, 0.0));
/// ```
#[doc(alias = "interpolateBasis")]
pub fn basis(values: &[f64]) -> impl Fn(f64) -> f64 + Clone + Send + Sync + use<> {
    let values: Arc<[f64]> = values.into();
    move |t| at(&values, t, |v| *v)
}

/// d3's basis.js at `t`, over the `channel` of each of `values`. With
/// `n = len − 1`, it evaluates segment 0 at 0 for `t ≤ 0`, segment `n − 1`
/// at 1 for `t ≥ 1`, and segment `floor(t·n)` otherwise. NaN for fewer
/// than two values.
pub(crate) fn at<V>(values: &[V], t: f64, channel: impl Fn(&V) -> f64) -> f64 {
    let Some(last) = values.len().checked_sub(2) else {
        return f64::NAN;
    };
    let n = last + 1;
    let (i, t) = if t <= 0.0 {
        (0, 0.0)
    } else if t >= 1.0 {
        (last, 1.0)
    } else {
        // In 0..n for t in (0, 1); NaN casts to 0 and stays NaN below.
        ((t * n as f64).floor() as usize, t)
    };
    // JavaScript reads undefined past the end, which arithmetic makes NaN.
    let value = |k: usize| values.get(k).map_or(f64::NAN, &channel);
    let (v1, v2) = (value(i), value(i + 1));
    let v0 = if i > 0 { value(i - 1) } else { 2.0 * v1 - v2 };
    let v3 = if i < last {
        value(i + 2)
    } else {
        2.0 * v2 - v1
    };
    spline((t - i as f64 / n as f64) * n as f64, [v0, v1, v2, v3])
}

/// d3's `basis(t1, v0, v1, v2, v3)`: the cubic B-spline weights at `t1`.
fn spline(t1: f64, [v0, v1, v2, v3]: [f64; 4]) -> f64 {
    let t2 = t1 * t1;
    let t3 = t2 * t1;
    ((1.0 - 3.0 * t1 + 3.0 * t2 - t3) * v0
        + (4.0 - 6.0 * t2 + 3.0 * t3) * v1
        + (1.0 + 3.0 * t1 + 3.0 * t2 - 3.0 * t3) * v2
        + t3 * v3)
        / 6.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fewer_than_two_values_give_nan() {
        assert!(basis(&[])(0.5).is_nan());
        assert!(basis(&[3.0])(0.0).is_nan());
    }

    #[test]
    fn two_values_reflect_both_neighbours_into_a_line() {
        let line = basis(&[0.0, 6.0]);
        assert_eq!(line(0.5), 3.0);
        assert_eq!(line(-1.0), 0.0);
        assert_eq!(line(2.0), 6.0);
        assert!(line(f64::NAN).is_nan());
    }
}
