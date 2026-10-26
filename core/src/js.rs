//! JavaScript's number semantics where d3's output depends on them.

use std::fmt;

/// 2⁵³: every integer up to it is an `f64`.
const SAFE: f64 = 9_007_199_254_740_992.0;

/// A number as JavaScript's `String(x)` prints it, without its exponent
/// notation: Rust's shortest round-trip digits, `0` for −0, and `NaN`,
/// `Infinity`, `-Infinity`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Number(pub(crate) f64);

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Number(x) = *self;
        if x == 0.0 {
            f.write_str("0")
        } else if x == f64::INFINITY {
            f.write_str("Infinity")
        } else if x == f64::NEG_INFINITY {
            f.write_str("-Infinity")
        } else {
            write!(f, "{x}")
        }
    }
}

/// JavaScript's `Math.round`: ties toward +∞ (`f64::round` differs at −2.5).
pub(crate) fn round(x: f64) -> f64 {
    let r = x.floor();
    if x - r >= 0.5 { r + 1.0 } else { r }
}

/// `round(x)` as an integer: `Some` iff it is finite and |round(x)| ≤ 2⁵³.
pub(crate) fn to_int(x: f64) -> Option<i64> {
    let r = round(x);
    // An integer of at most 2⁵³ converts exactly; NaN and ±∞ fail the test.
    (r.abs() <= SAFE).then_some(r as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_takes_ties_toward_positive_infinity() {
        assert_eq!(round(2.5), 3.0);
        assert_eq!(round(-2.5), -2.0);
        assert_eq!(round(-2.6), -3.0);
        assert_eq!(round(0.499_999_999_999_999_94), 0.0);
        assert!(round(f64::NAN).is_nan());
        assert_eq!(round(f64::INFINITY), f64::INFINITY);
    }

    #[test]
    fn to_int_holds_exactly_the_safe_integers() {
        assert_eq!(to_int(-2.5), Some(-2));
        assert_eq!(to_int(SAFE), Some(9_007_199_254_740_992));
        assert_eq!(to_int(-SAFE), Some(-9_007_199_254_740_992));
        assert_eq!(to_int(SAFE * 2.0), None);
        assert_eq!(to_int(f64::NAN), None);
        assert_eq!(to_int(f64::NEG_INFINITY), None);
    }

    #[test]
    fn numbers_print_zero_and_infinities_as_javascript_does() {
        let print = |x| Number(x).to_string();
        assert_eq!(print(-0.0), "0");
        assert_eq!(print(1.5), "1.5");
        assert_eq!(print(f64::NEG_INFINITY), "-Infinity");
        assert_eq!(print(f64::NAN), "NaN");
        assert_eq!(print(1e21), "1000000000000000000000");
    }
}
