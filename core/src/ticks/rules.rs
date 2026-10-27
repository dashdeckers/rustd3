//! Tick rules: what a [`scale::Transform`](crate::scale::Transform) calls
//! to state its ticks and minor ticks. Each returns values in the order of
//! its domain or majors, at most 10,000 of them.

use crate::scale::{Transform, transform};
use crate::ticks;

/// The most values a built-in rule returns.
const MAX_VALUES: usize = 10_000;

/// d3's linear ticks in data space: `ticks(d[0], d[1], count)`.
pub fn linear(domain: [f64; 2], count: usize) -> Vec<f64> {
    ticks::ticks(domain[0], domain[1], count as f64)
}

/// d3's log ticks (log.js `ticks`), reflected for a domain that starts
/// below zero: for an integer base and fewer than `count` decades, 1, 2, …,
/// base − 1 times each power of the base inside the domain, or d3's linear
/// ticks when that gives fewer than `count / 2`; otherwise powers of the
/// base at d3's ticks of the exponents. Empty when the log of an end is not
/// finite or the base is not positive or is 1; the decades branch also
/// serves an integer base whose per-decade values would number above
/// 10,000.
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

/// `k` equal steps in transformed space between neighbouring majors (the
/// majors themselves excluded), stopping after 10,000 values.
pub fn subdivide<T: Transform>(transform: &T, major: &[f64], k: usize) -> Vec<f64> {
    major
        .windows(2)
        .flat_map(|pair| {
            let (a, b) = (transform.forward(pair[0]), transform.forward(pair[1]));
            (1..k).map(move |i| a + (b - a) * i as f64 / k as f64)
        })
        .map(|u| transform.inverse(u))
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
    fn log_takes_the_decades_branch_past_ten_thousand_multiples() {
        let ticks = log(100_000.0, [1.0, 1e10], 10);
        assert_eq!(ticks, [1.0, 1e5, 1e10]);
    }
}
