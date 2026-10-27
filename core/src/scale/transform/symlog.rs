//! The symmetric log transform, under d3's scaleSymlog (symlog.js), with
//! ticks by decades past the constant.

use crate::scale::{Transform, transform};
use crate::ticks;

/// The most minor ticks the decade rule returns.
const MAX_MINOR: usize = 10_000;
/// Below this many constants from 0 the domain is nearly linear.
const LINEAR_SPAN: f64 = 100.0;

/// d3's symlog transform, `sign(x)·ln(1 + |x/c|)`: linear near 0 and
/// logarithmic beyond the constant c. Ticks are d3's linear ticks while the
/// domain stays within two decades past the constant and thinned decades
/// (`ticks::decades`) beyond; minor ticks are 2–9 per decade between decade
/// majors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Symlog {
    constant: f64,
}

impl Symlog {
    /// The transform with constant `constant`.
    pub(crate) fn new(constant: f64) -> Symlog {
        Symlog { constant }
    }

    /// The decade k of a major `v` when |v| is exactly c·10ᵏ.
    fn exponent(&self, v: f64) -> Option<i32> {
        let k = (v.abs() / self.constant).log10().round();
        if !k.is_finite() || k.abs() > 1000.0 {
            return None;
        }
        // In range by the check above.
        let k = k as i32;
        (ticks::decade(self.constant, k) == v.abs()).then_some(k)
    }

    /// m·c·10ᵏ for m = 2 … 9 and the decades from `from`'s to `to`'s,
    /// between two decade majors of one sign, in the order from → to.
    fn between(&self, from: (f64, i32), to: (f64, i32)) -> Vec<f64> {
        let ((p, i), (q, j)) = (from, to);
        let (low, high) = (i.min(j), i.max(j));
        let sign = p.signum();
        let mut values: Vec<f64> = (low..high)
            .flat_map(|k| (2..10).map(move |m| (m, k)))
            .map(|(m, k)| sign * f64::from(m) * ticks::decade(self.constant, k))
            .collect();
        if (q < p) != (sign < 0.0) {
            values.reverse();
        }
        values
    }
}

impl Transform for Symlog {
    fn forward(&self, x: f64) -> f64 {
        x.signum() * (x / self.constant).abs().ln_1p()
    }

    fn inverse(&self, y: f64) -> f64 {
        y.signum() * y.abs().exp_m1() * self.constant
    }

    /// d3's linear ticks while the larger end is under 100 constants from
    /// 0; otherwise `ticks::decades`, or the linear ticks again when fewer
    /// than two decades fall inside the domain.
    fn ticks(&self, domain: [f64; 2], count: usize) -> Vec<f64> {
        let largest = domain[0].abs().max(domain[1].abs());
        if largest / self.constant < LINEAR_SPAN {
            return ticks::linear(domain, count);
        }
        let decades = ticks::decades(domain, self.constant, count);
        if decades.len() < 2 {
            ticks::linear(domain, count)
        } else {
            decades
        }
    }

    /// When every nonzero major is ±c·10ᵏ, m·c·10ᵏ for m = 2 … 9 between
    /// neighbouring majors of one sign (none next to 0), at most 10,000;
    /// otherwise five equal steps in data space between majors.
    fn minor(&self, major: &[f64]) -> Vec<f64> {
        let exponents: Option<Vec<Option<i32>>> = major
            .iter()
            .map(|&v| {
                if v == 0.0 {
                    Some(None)
                } else {
                    self.exponent(v).map(Some)
                }
            })
            .collect();
        let Some(exponents) = exponents else {
            return ticks::subdivide(&transform::Identity, major, 5);
        };
        let mut minor = Vec::new();
        for (pair, k) in major.windows(2).zip(exponents.windows(2)) {
            if let [Some(i), Some(j)] = *k
                && pair[0].signum() == pair[1].signum()
            {
                minor.extend(self.between((pair[0], i), (pair[1], j)));
            }
            if minor.len() >= MAX_MINOR {
                minor.truncate(MAX_MINOR);
                break;
            }
        }
        minor
    }
}
