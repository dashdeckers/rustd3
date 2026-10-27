//! The log transform, under d3's scaleLog (log.js).

use std::f64::consts::E;

use crate::format::{Format, InvalidFormat, Locale, Specifier};
use crate::js;
use crate::scale::{TickFormat, Transform};
use crate::ticks;

/// The most minor ticks the log rule returns.
const MAX_MINOR: usize = 10_000;

/// d3's log transform: `ln x`, or `−ln(−x)` over a domain that starts
/// below zero. The base sets ticks, nice, minor ticks and labels, not the
/// map; a value of the other sign than the domain maps to NaN, as in d3.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Log {
    base: f64,
    /// Whether the domain starts below zero, which reflects every value.
    negative: bool,
    /// Whether ten or more integer powers of the base lie inside the
    /// domain, which leaves out minor ticks.
    crowded: bool,
}

impl Log {
    /// The transform of base `base`, for a scale's
    /// [`transform`](crate::scale::Sequential::transform); the scale sets it
    /// for its domain, reflected below zero. `scale::log()` and the `base`
    /// builders make it too.
    ///
    /// ```
    /// use rustd3_core::scale::{self, transform};
    /// use rustd3_core::scheme;
    ///
    /// let s = scale::sequential([1.0, 1000.0], scheme::viridis).transform(transform::Log::new(10.0));
    /// assert_eq!(s.apply(10.0).format_hex(), "#31688e");
    /// ```
    pub fn new(base: f64) -> Log {
        Log {
            base,
            negative: false,
            crowded: false,
        }
    }

    /// The base.
    pub(crate) fn base(&self) -> f64 {
        self.base
    }

    /// Whether the base is an integer, which d3's ticks and labels ask.
    pub(crate) fn integer(&self) -> bool {
        self.base % 1.0 == 0.0
    }

    /// d3's `logs`: the logarithm in this base, reflected for a negative
    /// domain.
    pub(crate) fn logs(&self, x: f64) -> f64 {
        if self.negative {
            -logarithm(self.base, -x)
        } else {
            logarithm(self.base, x)
        }
    }

    /// d3's `pows`: this base raised to `x`, reflected for a negative
    /// domain.
    pub(crate) fn pows(&self, x: f64) -> f64 {
        if self.negative {
            -power(self.base, -x)
        } else {
            power(self.base, x)
        }
    }

    /// m · base^e for m = 2 … base − 1 strictly between the magnitudes
    /// `a` < `b`, ascending, appended to `minor` until it holds 10,000;
    /// nothing when the log of either is not finite.
    fn multiples(&self, a: f64, b: f64, minor: &mut Vec<f64>) {
        let (from, to) = (self.logs(a).floor(), self.logs(b).ceil());
        if !from.is_finite() || !to.is_finite() {
            return;
        }
        for e in (from as i32)..=(to as i32) {
            let e = f64::from(e);
            let decade = |m: f64| {
                if e < 0.0 {
                    m / self.pows(-e)
                } else {
                    m * self.pows(e)
                }
            };
            // Start just below `a`, so that a large base skips its empty multiples.
            let below = if e < 0.0 {
                a * self.pows(-e)
            } else {
                a / self.pows(e)
            };
            let start = (below.floor() - 1.0).max(2.0);
            for m in (0_u32..)
                .map(|i| start + f64::from(i))
                .take_while(|&m| m < self.base)
            {
                let t = decade(m);
                if t <= a {
                    continue;
                }
                if t >= b {
                    break;
                }
                minor.push(t);
                if minor.len() == MAX_MINOR {
                    return;
                }
            }
        }
    }
}

impl Transform for Log {
    fn forward(&self, x: f64) -> f64 {
        if self.negative { -(-x).ln() } else { x.ln() }
    }

    fn inverse(&self, y: f64) -> f64 {
        if self.negative { -(-y).exp() } else { y.exp() }
    }

    fn ticks(&self, domain: [f64; 2], count: usize) -> Vec<f64> {
        ticks::log(self.base, domain, count)
    }

    /// For an integer base, m · base^k for m = 2 … base − 1 between the
    /// first and last major, the majors excluded, and none between
    /// neighbouring majors more than one decade of the base apart (d3's
    /// thinned decades, where matplotlib draws none); otherwise the default.
    /// None at all for a base below 3 or once ten or more integer powers of
    /// the base lie inside the domain, as matplotlib's `LogLocator` with
    /// `subs='auto'`.
    fn minor(&self, major: &[f64]) -> Vec<f64> {
        let (Some(&first), Some(&last)) = (major.first(), major.last()) else {
            return Vec::new();
        };
        if self.crowded || self.base < 3.0 {
            return Vec::new();
        }
        if !self.integer() {
            return ticks::subdivide(self, major, 5);
        }
        // Magnitudes are computed as d3 computes ticks over a positive
        // domain, so that a minor tick equal to a major is equal bit for bit.
        let unsigned = Log::new(self.base);
        let (a, b) = (first.abs().min(last.abs()), first.abs().max(last.abs()));
        let mut magnitudes: Vec<f64> = major
            .iter()
            .map(|v| v.abs())
            .filter(|m| (a..=b).contains(m))
            .collect();
        magnitudes.sort_by(f64::total_cmp);
        magnitudes.dedup();
        let mut minor = Vec::new();
        for pair in magnitudes.windows(2) {
            let (low, high) = (pair[0], pair[1]);
            if minor.len() == MAX_MINOR {
                break;
            }
            if !ticks::thinned(unsigned.logs(high) - unsigned.logs(low)) {
                unsigned.multiples(low, high, &mut minor);
            }
        }
        let sign = if first < 0.0 || last < 0.0 { -1.0 } else { 1.0 };
        let mut minor: Vec<f64> = minor.into_iter().map(|t| sign * t).collect();
        // Magnitudes ascend; the majors' order decides the output's.
        if (sign > 0.0) != (first < last) {
            minor.reverse();
        }
        minor
    }

    /// The transform reflected for a domain that starts below zero, and
    /// told how many integer powers of the base lie inside the domain
    /// (matplotlib's `n_avail`).
    fn for_domain(&self, domain: [f64; 2]) -> Log {
        let unsigned = Log::new(self.base);
        let [a, b] = domain.map(f64::abs);
        let in_view = unsigned.logs(a.max(b)).floor() - unsigned.logs(a.min(b)).ceil() + 1.0;
        Log {
            negative: domain[0] < 0.0,
            crowded: ticks::crowded(in_view),
            ..*self
        }
    }

    /// d3's log nice (nice.js): the ordered ends floored and ceiled to
    /// powers of the base; the domain unchanged when the log of an end is
    /// not finite. The count is ignored.
    fn nice(&self, domain: [f64; 2], _count: usize) -> [f64; 2] {
        let log = self.for_domain(domain);
        let reverse = domain[1] < domain[0];
        let [x0, x1] = if reverse {
            [domain[1], domain[0]]
        } else {
            domain
        };
        let (i, j) = (log.logs(x0), log.logs(x1));
        if !i.is_finite() || !j.is_finite() {
            return domain;
        }
        let (low, high) = (log.pows(i.floor()), log.pows(j.ceil()));
        if reverse { [high, low] } else { [low, high] }
    }

    /// d3's log tickFormat (log.js 105–118): the specifier defaults to `s`
    /// for base 10 and `,` otherwise, trimmed for an integer base when it
    /// sets no precision; a tick keeps its label when its leading digit is
    /// at most `max(1, base · count / ticks(10).len())`.
    fn tick_format(
        &self,
        locale: &Locale,
        domain: [f64; 2],
        count: usize,
        specifier: Option<&str>,
    ) -> Result<TickFormat, InvalidFormat> {
        let default = if self.base == 10.0 { "s" } else { "," };
        let specifier: Specifier = specifier.unwrap_or(default).parse()?;
        let specifier = if self.integer() && specifier.precision().is_none() {
            specifier.with_trim(true)
        } else {
            specifier
        };
        let ticks = self.ticks(domain, 10).len();
        let k = js::max(1.0, self.base * count as f64 / ticks as f64);
        let format = Format::new(locale, specifier);
        Ok(TickFormat::thinned(format, self.for_domain(domain), k))
    }
}

/// d3's logp: `ln`, `log10` and `log2` for their bases, else `ln x / ln b`.
fn logarithm(base: f64, x: f64) -> f64 {
    if base == E {
        x.ln()
    } else if base == 10.0 {
        x.log10()
    } else if base == 2.0 {
        x.log2()
    } else {
        x.ln() / base.ln()
    }
}

/// d3's powp: the exact `pow10` for base 10 and an integral exponent, `exp`
/// for e, else `powf`.
fn power(base: f64, x: f64) -> f64 {
    if base == 10.0 && x.fract() == 0.0 {
        ticks::pow10(x)
    } else if base == E {
        x.exp()
    } else {
        base.powf(x)
    }
}
