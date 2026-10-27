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
}

impl Log {
    /// The transform of base `base` over a positive domain.
    pub(crate) fn new(base: f64) -> Log {
        Log {
            base,
            negative: false,
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
    /// first and last major, the majors excluded; otherwise the default.
    fn minor(&self, major: &[f64]) -> Vec<f64> {
        let (Some(&first), Some(&last)) = (major.first(), major.last()) else {
            return Vec::new();
        };
        if !self.integer() {
            return ticks::subdivide(self, major, 5);
        }
        // Magnitudes are computed as d3 computes ticks over a positive
        // domain, so that a minor tick equal to a major is equal bit for bit.
        let unsigned = Log::new(self.base);
        let (a, b) = (first.abs().min(last.abs()), first.abs().max(last.abs()));
        let (from, to) = (unsigned.logs(a).floor(), unsigned.logs(b).ceil());
        if !from.is_finite() || !to.is_finite() {
            return Vec::new();
        }
        let sign = if first < 0.0 || last < 0.0 { -1.0 } else { 1.0 };
        let mut minor = Vec::new();
        'decades: for e in (from as i32)..=(to as i32) {
            let e = f64::from(e);
            let decade = |m: f64| {
                if e < 0.0 {
                    m / unsigned.pows(-e)
                } else {
                    m * unsigned.pows(e)
                }
            };
            // Start just below `a`, so that a large base skips its empty multiples.
            let below = if e < 0.0 {
                a * unsigned.pows(-e)
            } else {
                a / unsigned.pows(e)
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
                if !major.contains(&(sign * t)) {
                    minor.push(sign * t);
                    if minor.len() == MAX_MINOR {
                        break 'decades;
                    }
                }
            }
        }
        // Magnitudes ascend; the majors' order decides the output's.
        if (sign > 0.0) != (first < last) {
            minor.reverse();
        }
        minor
    }

    fn for_domain(&self, domain: [f64; 2]) -> Log {
        Log {
            negative: domain[0] < 0.0,
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
