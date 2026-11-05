//! The inverse hyperbolic sine transform, with matplotlib's `AsinhLocator`
//! as its tick rule.

use std::f64::consts::LN_10;

use crate::format::{Format, InvalidFormat, Locale, Specifier};
use crate::scale::{TickFormat, Transform};
use crate::ticks;

/// The most values the rule returns: the tick generators' cap at its
/// largest.
const MAX_TICKS: usize = 10_000;
/// `AsinhLocator`'s `symthresh`: ends this close to symmetric about 0 tick
/// as if they were.
const SYMMETRIC: f64 = 0.2;

/// The inverse hyperbolic sine transform, `w·asinh(x/w)`: linear within
/// about the linear width w of 0 and logarithmic beyond, and defined for
/// negative values. Ticks are signed powers of ten (matplotlib's
/// `AsinhLocator`), and labels default to `",.12~g"`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Asinh {
    linear_width: f64,
}

impl Asinh {
    /// The transform of linear width `linear_width`.
    pub(crate) fn new(linear_width: f64) -> Asinh {
        Asinh { linear_width }
    }
}

impl Transform for Asinh {
    fn forward(&self, x: f64) -> f64 {
        self.linear_width * (x / self.linear_width).asinh()
    }

    fn inverse(&self, y: f64) -> f64 {
        self.linear_width * (y / self.linear_width).sinh()
    }

    /// matplotlib's `AsinhLocator` in base 10 with `numticks = min(count,
    /// 10,000) + 1`. Ends nearly symmetric about 0 are made symmetric. The
    /// transformed ends are split into `numticks` equal steps. When they
    /// straddle 0, the steps nearer 0 than half a step are dropped and 0 is
    /// added. Each step, mapped back, is rounded down to a signed power of
    /// ten, as matplotlib does through `⌊ln|x| / ln 10⌋`. Fewer than two
    /// distinct powers give equal steps of the ends instead, each value
    /// once (one tick for a zero-width domain). Values outside the domain
    /// are dropped, and the ticks are in domain order. Empty for a
    /// non-finite end.
    fn ticks(&self, domain: [f64; 2], count: usize) -> Vec<f64> {
        let reverse = domain[1] < domain[0];
        let [a, b] = if reverse {
            [domain[1], domain[0]]
        } else {
            domain
        };
        if !a.is_finite() || !b.is_finite() {
            return Vec::new();
        }
        let steps = count.min(MAX_TICKS) + 1;
        let [low, high] = if a * b < 0.0 && (1.0 + b / a).abs() < SYMMETRIC {
            let bound = a.abs().max(b.abs());
            [-bound, bound]
        } else {
            [a, b]
        };
        let [y0, y1] = [self.forward(low), self.forward(high)];
        let mut ys = linspace(y0, y1, steps);
        if y0 * y1 < 0.0 {
            let span = y1 - y0;
            ys.retain(|y| (y / span).abs() > 0.5 / steps as f64);
            ys.push(0.0);
        }
        let mut z: Vec<f64> = ys
            .into_iter()
            .map(|y| power_below(self.inverse(y)))
            .collect();
        z.sort_by(f64::total_cmp);
        z.dedup();
        if z.len() < 2 {
            // Equal steps of a zero-width or subnormal span repeat a value;
            // each tick is kept once.
            z = linspace(low, high, steps.min(MAX_TICKS));
            z.dedup();
        }
        z.retain(|v| (a..=b).contains(v));
        if reverse {
            z.reverse();
        }
        z
    }

    /// Every tick labeled, the specifier defaulting to `",.12~g"`.
    fn tick_format(
        &self,
        locale: &Locale,
        _domain: [f64; 2],
        _count: usize,
        specifier: Option<&str>,
    ) -> Result<TickFormat, InvalidFormat> {
        let specifier: Specifier = specifier.unwrap_or(",.12~g").parse()?;
        Ok(TickFormat::from(Format::new(locale, specifier)))
    }
}

/// `sign(x)·10^⌊log10|x|⌋`, or 0 for 0. The decade is read a relative
/// 1e-9 above the exact power, so a value one ulp under 10ᵏ, which the
/// platform's `sinh(asinh(10ᵏ))` can return, still ticks at 10ᵏ. This
/// keeps the ticks independent of the libm in use.
fn power_below(x: f64) -> f64 {
    if x == 0.0 {
        0.0
    } else {
        x.signum() * ticks::pow10((x.abs().ln() / LN_10 + 1e-9).floor())
    }
}

/// numpy's `linspace(start, stop, n)`: `start + i · step`, the last value
/// `stop` itself; `[start]` for one value.
fn linspace(start: f64, stop: f64, n: usize) -> Vec<f64> {
    if n < 2 {
        return vec![start; n];
    }
    let div = (n - 1) as f64;
    let step = (stop - start) / div;
    (0..n)
        .map(|i| {
            if i == n - 1 {
                stop
            } else if step == 0.0 {
                i as f64 / div * (stop - start) + start
            } else {
                i as f64 * step + start
            }
        })
        .collect()
}
