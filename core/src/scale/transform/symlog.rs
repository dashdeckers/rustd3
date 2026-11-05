//! The symmetric log transform, under d3's scaleSymlog (symlog.js), with
//! ticks by decades past the constant.

use crate::format::{Format, InvalidFormat, Locale, Specifier};
use crate::scale::{self, TickFormat, Transform, transform};
use crate::ticks;

/// The most minor ticks the decade rule returns.
const MAX_MINOR: usize = 10_000;
/// Below this many constants from 0 the domain is nearly linear.
const LINEAR_SPAN: f64 = 100.0;

/// d3's symlog transform, `sign(x)·ln(1 + |x/c|)`: linear near 0 and
/// logarithmic beyond the constant c. Ticks are d3's linear ticks while the
/// domain stays within two decades past the constant, and thinned decades
/// (`ticks::decades`) beyond. Minor ticks are 2–9 per decade between decade
/// majors. Labels follow the ticks, with d3's linear tick format for linear
/// ticks and `",.12~g"` for decades.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Symlog {
    constant: f64,
    /// Whether ten or more decades ±c·10ᵏ lie inside the domain, which
    /// leaves out the decade branch's minor ticks.
    crowded: bool,
}

impl Symlog {
    /// The transform with constant `constant`, for a scale's
    /// [`transform`](crate::scale::Sequential::transform).
    ///
    /// ```
    /// use rustd3_core::scale::{self, transform};
    ///
    /// let s = scale::sequential([0.0, 100.0], |t| t).transform(transform::Symlog::new(1.0));
    /// assert_eq!(s.apply(10.0), 0.5195737064824407);
    /// ```
    #[must_use]
    pub fn new(constant: f64) -> Symlog {
        Symlog {
            constant,
            crowded: false,
        }
    }

    /// The decades branch of the tick rule: `ticks::decades` once the
    /// larger end is 100 constants or more from 0 and at least two decades
    /// fall inside the domain. `None` where the rule ticks linearly.
    fn decades(&self, domain: [f64; 2], count: usize) -> Option<Vec<f64>> {
        let largest = domain[0].abs().max(domain[1].abs());
        if largest / self.constant < LINEAR_SPAN {
            return None;
        }
        Some(ticks::decades(domain, self.constant, count)).filter(|decades| decades.len() >= 2)
    }

    /// Whether |v| is exactly c·10ᵏ for some k.
    fn decade(&self, v: f64) -> bool {
        let k = (v.abs() / self.constant).log10().round();
        // In range of i32 once finite and within ±1000.
        k.is_finite() && k.abs() <= 1000.0 && ticks::decade(self.constant, k as i32) == v.abs()
    }

    /// m·c·10ᵏ for m = 2 … 9 strictly between `p` and `q`, nonzero and of
    /// one sign, in the order p → q.
    fn between(&self, p: f64, q: f64) -> Vec<f64> {
        let (low, high) = (p.abs().min(q.abs()), p.abs().max(q.abs()));
        let k = |v: f64| (v / self.constant).log10().floor();
        let (i, j) = (k(low), k(high));
        if !i.is_finite() || !j.is_finite() || i < -1000.0 || j > 1000.0 {
            return Vec::new();
        }
        let sign = p.signum();
        // In range of i32 by the check above.
        let mut values: Vec<f64> = ((i as i32)..=(j as i32))
            .flat_map(|k| (2..10).map(move |m| f64::from(m) * ticks::decade(self.constant, k)))
            .filter(|&t| low < t && t < high)
            .map(|t| sign * t)
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
        self.decades(domain, count)
            .unwrap_or_else(|| ticks::linear(domain, count))
    }

    /// Decade minors when every nonzero major is ±c·10ᵏ, or when every one
    /// but the first and last is and at least two of those share a sign (an
    /// axis's majors extended by one step past each end). These are
    /// m·c·10ᵏ for m = 2 … 9 strictly between neighboring majors of one
    /// sign at most a decade apart, as matplotlib's log minors, so there are
    /// none next to 0 and none across thinned decades. They are capped at
    /// 10,000, and there are none at all once ten or more decades ±c·10ᵏ
    /// lie inside the domain (matplotlib's `n_avail >= 10`). Otherwise the
    /// minors are five equal steps in data space between majors, since
    /// evenly spaced majors never hold two decades of one sign.
    fn minor(&self, major: &[f64]) -> Vec<f64> {
        let tied = |v: &f64| *v == 0.0 || self.decade(*v);
        let inner = major
            .get(1..major.len().saturating_sub(1))
            .unwrap_or_default();
        let paired = |positive: bool| {
            inner
                .iter()
                .filter(|v| (**v > 0.0) == positive && self.decade(**v))
                .count()
                >= 2
        };
        let decades =
            major.iter().all(tied) || (inner.iter().all(tied) && (paired(true) || paired(false)));
        if !decades {
            return ticks::subdivide(&transform::Identity, major, 5);
        }
        if self.crowded {
            return Vec::new();
        }
        let mut minor = Vec::new();
        for pair in major.windows(2) {
            let (p, q) = (pair[0], pair[1]);
            let one_sign = p != 0.0 && q != 0.0 && p.signum() == q.signum();
            if one_sign && !ticks::thinned((q / p).log10()) {
                minor.extend(self.between(p, q));
            }
            if minor.len() >= MAX_MINOR {
                minor.truncate(MAX_MINOR);
                break;
            }
        }
        minor
    }

    /// The transform told how many decades ±c·10ᵏ (k ≥ 0, the values
    /// `ticks::decades` can draw) lie inside the domain.
    fn for_domain(&self, domain: [f64; 2]) -> Symlog {
        let in_view = ticks::decades(domain, self.constant, usize::MAX)
            .into_iter()
            .filter(|v| *v != 0.0)
            .count();
        Symlog {
            crowded: ticks::crowded(in_view as f64),
            ..*self
        }
    }

    /// The format of the branch `ticks` takes. Decades label every tick,
    /// the specifier defaulting to `",.12~g"` as asinh's labels do. Linear
    /// ticks take d3's linear tick format.
    fn tick_format(
        &self,
        locale: &Locale,
        domain: [f64; 2],
        count: usize,
        specifier: Option<&str>,
    ) -> Result<TickFormat, InvalidFormat> {
        if self.decades(domain, count).is_none() {
            return scale::tick_format(locale, domain, count, specifier).map(TickFormat::from);
        }
        let specifier: Specifier = specifier.unwrap_or(",.12~g").parse()?;
        Ok(TickFormat::from(Format::new(locale, specifier)))
    }
}
