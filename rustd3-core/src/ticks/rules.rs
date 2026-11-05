//! Tick rules: what a [`scale::Transform`](crate::scale::Transform) calls
//! to state its ticks and minor ticks. Each returns values in the order of
//! its domain or majors, at most 10,000 of them.

use crate::scale::{Transform, transform};
use crate::ticks;

/// The most values a built-in rule returns.
const MAX_VALUES: usize = 10_000;
/// How far past an end, relative to it, [`transformed`] keeps a value.
const ROUND_TRIP: f64 = 1e-9;
/// The most decades [`decades`] walks from the constant up.
const MAX_DECADES: i32 = 400;

/// d3's linear ticks in data space: `ticks(d[0], d[1], count)`, and none
/// for a non-finite end (where d3 gives `[∞]` for equal infinite ends).
#[must_use]
pub fn linear(domain: [f64; 2], count: usize) -> Vec<f64> {
    if !domain[0].is_finite() || !domain[1].is_finite() {
        return Vec::new();
    }
    ticks::ticks(domain[0], domain[1], count as f64)
}

/// d3's linear ticks of `forward(domain)`, mapped back through `inverse`.
/// A value the round trip carries past an end by more than a relative
/// 1e-9 (a transform that underflows or overflows on the domain, such as
/// the squares of subnormals) is left out.
pub fn transformed<T: Transform>(transform: &T, domain: [f64; 2], count: usize) -> Vec<f64> {
    let [low, high] = if domain[1] < domain[0] {
        [domain[1], domain[0]]
    } else {
        domain
    };
    let (low, high) = (low - ROUND_TRIP * low.abs(), high + ROUND_TRIP * high.abs());
    linear(domain.map(|x| transform.forward(x)), count)
        .into_iter()
        .map(|u| transform.inverse(u))
        .filter(|v| v.is_finite() && (low..=high).contains(v))
        .collect()
}

/// 0 and ±c·10ᵏ inside the domain, in domain order, thinned to stay near
/// `count` values. The decades are k = 0, 1, … while c·10ᵏ is at most the
/// larger magnitude of the ends (at most 400 of them), on both sides of 0
/// when the domain straddles it. Every ⌈decades · sides / count⌉-th decade
/// is kept. Empty unless `constant` is positive and both ends are finite.
pub fn decades(domain: [f64; 2], constant: f64, count: usize) -> Vec<f64> {
    let reverse = domain[1] < domain[0];
    let [a, b] = if reverse {
        [domain[1], domain[0]]
    } else {
        domain
    };
    if constant.is_nan() || constant <= 0.0 || !a.is_finite() || !b.is_finite() {
        return Vec::new();
    }
    let largest = a.abs().max(b.abs());
    let powers: Vec<f64> = (0..MAX_DECADES)
        .map(|k| decade(constant, k))
        .take_while(|&p| p <= largest)
        .collect();
    let sides = if a < 0.0 && 0.0 < b { 2 } else { 1 };
    let step = (powers.len() * sides).div_ceil(count.max(1)).max(1);
    let mut z = Vec::new();
    if a <= 0.0 && 0.0 <= b {
        z.push(0.0);
    }
    for &p in powers.iter().step_by(step) {
        z.extend([-p, p].into_iter().filter(|v| (a..=b).contains(v)));
    }
    z.sort_by(f64::total_cmp);
    if reverse {
        z.reverse();
    }
    z
}

/// d3's log ticks (log.js `ticks`), reflected for a domain that starts
/// below zero.
///
/// For an integer base and fewer than `count` decades, the ticks are 1, 2,
/// …, base − 1 times each power of the base inside the domain, or d3's
/// linear ticks when that gives fewer than `count / 2`. Otherwise they are
/// powers of the base at d3's ticks of the exponents. That second branch
/// also serves an integer base whose per-decade values would number above
/// 10,000, and it leaves out a power that overflows (`log2` of `f64::MAX`
/// rounds up to 1024).
///
/// Empty when the log of an end is not finite, or the base is not positive
/// or is 1.
#[must_use]
pub fn log(base: f64, domain: [f64; 2], count: usize) -> Vec<f64> {
    let log = transform::Log::new(base).for_domain(domain);
    let reverse = domain[1] < domain[0];
    let [u, v] = if reverse {
        [domain[1], domain[0]]
    } else {
        domain
    };
    let (i, j) = (log.logs(u), log.logs(v));
    if base.is_nan() || base <= 0.0 || base == 1.0 || !i.is_finite() || !j.is_finite() {
        return Vec::new();
    }
    let n = count as f64;
    let (first, last) = (i.floor(), j.ceil());
    let per_decade = (last - first + 1.0) * (base - 1.0);
    let mut z = if log.integer() && j - i < n && per_decade <= MAX_VALUES as f64 {
        let z = multiples(&log, [u, v], [first as i32, last as i32]);
        if z.len() * 2 < count {
            ticks::ticks(u, v, n)
        } else {
            z
        }
    } else {
        ticks::ticks(i, j, (j - i).min(n))
            .into_iter()
            .map(|e| log.pows(e))
            .filter(|t| t.is_finite())
            .collect()
    };
    if reverse {
        z.reverse();
    }
    z
}

/// d3's per-decade log ticks: k · base^e for k = 1 … base − 1 and the
/// exponents `decades`, inside [u, v], in ascending order and computed as
/// d3 computes them (division by the inverse power below the unit).
fn multiples(log: &transform::Log, [u, v]: [f64; 2], decades: [i32; 2]) -> Vec<f64> {
    let base = log.base();
    let ks = || (1_u32..).map(f64::from).take_while(move |&k| k < base);
    let mut z = Vec::new();
    for e in decades[0]..=decades[1] {
        let e = f64::from(e);
        // A positive domain counts k up; a reflected one counts it down, so
        // both ascend.
        let candidates: Vec<f64> = if u > 0.0 {
            ks().map(|k| {
                if e < 0.0 {
                    k / log.pows(-e)
                } else {
                    k * log.pows(e)
                }
            })
            .collect()
        } else {
            let mut down: Vec<f64> = ks().collect();
            down.reverse();
            down.into_iter()
                .map(|k| {
                    if e > 0.0 {
                        k / log.pows(-e)
                    } else {
                        k * log.pows(e)
                    }
                })
                .collect()
        };
        for t in candidates {
            if t < u {
                continue;
            }
            if t > v {
                break;
            }
            z.push(t);
        }
    }
    z
}

/// d3's exact `pow10` (log.js): `10^k` as the literal `1e{k}` parses, for
/// an integral `k` (other finite `k` through `powf`), 0 at −∞, and `k`
/// itself at +∞ and NaN.
pub(crate) fn pow10(k: f64) -> f64 {
    if k.is_finite() {
        format!("1e{k}").parse().unwrap_or_else(|_| 10f64.powf(k))
    } else if k < 0.0 {
        0.0
    } else {
        k
    }
}

/// Whether two majors `decades` apart, in their transform's base, skip a
/// decade. matplotlib's `LogLocator` (3.11.2, `subs='auto'`) draws no minor
/// ticks under a major stride above one decade, so the log and symlog
/// minor rules leave such gaps empty. The tolerance absorbs the float noise
/// of majors extended by one step.
pub(crate) fn thinned(decades: f64) -> bool {
    decades.abs() > 1.0 + 1e-9
}

/// Whether `in_view` decade ticks inside a domain crowd out its minor
/// ticks: matplotlib's `LogLocator` (3.11.2, `subs='auto'`) draws none once
/// ten or more integer powers of the base lie inside the domain
/// (`n_avail >= 10`).
pub(crate) fn crowded(in_view: f64) -> bool {
    in_view >= 10.0
}

/// c·10ᵏ through an exact power of ten. Where 10ᵏ alone would overflow, the
/// power is split so that a small enough `c` still gives a finite product.
pub(crate) fn decade(c: f64, k: i32) -> f64 {
    let k = f64::from(k);
    let power = pow10(k);
    if power.is_finite() {
        c * power
    } else {
        c * pow10(k - 300.0) * 1e300
    }
}

/// `k` equal steps in transformed space between neighboring majors (the
/// majors themselves excluded), stopping after 10,000 values. Steps that
/// map back to no finite value (majors outside the transform's domain, such
/// as a square root's negatives) are left out.
pub fn subdivide<T: Transform>(transform: &T, major: &[f64], k: usize) -> Vec<f64> {
    major
        .windows(2)
        .flat_map(|pair| {
            let (a, b) = (transform.forward(pair[0]), transform.forward(pair[1]));
            (1..k).map(move |i| a + (b - a) * i as f64 / k as f64)
        })
        .map(|u| transform.inverse(u))
        .filter(|v| v.is_finite())
        .take(MAX_VALUES)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pow10_parses_the_literal_as_d3_does() {
        assert_eq!(pow10(-1.0), 0.1);
        assert_eq!(pow10(-7.0), 1e-7);
        assert_eq!(pow10(22.0), 1e22);
        assert_eq!(pow10(400.0), f64::INFINITY);
        assert_eq!(pow10(-400.0), 0.0);
        assert_eq!(pow10(f64::NEG_INFINITY), 0.0);
        assert_eq!(pow10(f64::INFINITY), f64::INFINITY);
        assert!(pow10(f64::NAN).is_nan());
        assert_eq!(pow10(0.5), 10f64.powf(0.5));
    }

    #[test]
    fn subdivide_stops_after_ten_thousand_values() {
        let major = [0.0, 1.0, 2.0];
        assert_eq!(
            subdivide(&transform::Identity, &major, 20_000).len(),
            10_000
        );
        assert_eq!(subdivide(&transform::Identity, &major, 2), [0.5, 1.5]);
    }

    #[test]
    fn linear_gives_no_ticks_for_an_infinite_end() {
        assert!(linear([f64::INFINITY, f64::INFINITY], 10).is_empty());
        assert!(linear([0.0, f64::NEG_INFINITY], 10).is_empty());
        assert_eq!(linear([1.0, 1.0], 10), [1.0]);
    }

    #[test]
    fn transformed_leaves_out_values_the_round_trip_carries_away() {
        let squares = transform::Pow::new(2.0);
        assert!(transformed(&squares, [-1e-320, -5e-324], 1).is_empty());
        assert!(transformed(&squares, [1e300, f64::MAX], 2).is_empty());
        assert_eq!(
            transformed(&squares, [0.0, 2.0], 2),
            [0.0, 2.0f64.sqrt(), 2.0]
        );
    }

    #[test]
    fn subdivide_leaves_out_steps_outside_the_transforms_domain() {
        let log = transform::Log::new(10.0);
        assert!(subdivide(&log, &[-1.0, 0.0, 1.0], 2).is_empty());
        let middle = subdivide(&log, &[1.0, 100.0], 2);
        assert_eq!(middle.len(), 1);
        assert!((middle[0] - 10.0).abs() < 1e-12);
    }

    #[test]
    fn log_leaves_out_a_power_that_overflows() {
        let ticks = log(2.0, [-0.5, -f64::MAX], 1000);
        assert!(ticks.iter().all(|t| t.is_finite()));
        assert_eq!(ticks.first(), Some(&-0.5));
    }

    #[test]
    fn log_takes_the_decades_branch_past_ten_thousand_multiples() {
        let ticks = log(100_000.0, [1.0, 1e10], 10);
        assert_eq!(ticks, [1.0, 1e5, 1e10]);
    }
}
