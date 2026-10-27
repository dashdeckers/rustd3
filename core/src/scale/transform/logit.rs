//! The logit transform, with matplotlib's `LogitLocator` as its tick rule.

use crate::format::{Format, InvalidFormat, Locale, Specifier};
use crate::scale::{TickFormat, Transform};
use crate::ticks;

/// matplotlib's `standard_minpos`: where `nonsingular` moves an end at or
/// beyond 0 or 1.
const MIN_POSITIVE: f64 = 1e-7;

/// The logit transform, `ln(p / (1 − p))`, for probabilities in (0, 1).
/// Ticks are 0.5 and decades towards 0 and 1 (matplotlib's
/// `LogitLocator`); nice leaves the domain alone, and labels default to
/// `".12~g"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Logit;

impl Transform for Logit {
    fn forward(&self, x: f64) -> f64 {
        (x / (1.0 - x)).ln()
    }

    fn inverse(&self, y: f64) -> f64 {
        1.0 / (1.0 + (-y).exp())
    }

    /// matplotlib's `LogitLocator.tick_values` with `nbins = max(count,
    /// 1)`: the ideal ticks 10ᵇ (b < 0), 0.5 and 1 − 10⁻ᵇ (b > 0) between
    /// the ends' indices, every ⌈ideal / nbins⌉-th when there are more
    /// than `nbins`; d3's linear ticks when fewer than two ideal ticks lie
    /// between. Values outside the domain are dropped; domain order.
    fn ticks(&self, domain: [f64; 2], count: usize) -> Vec<f64> {
        let reverse = domain[1] < domain[0];
        let [a, b] = if reverse {
            [domain[1], domain[0]]
        } else {
            domain
        };
        let [low, high] = nonsingular(a, b);
        let first = if low < 0.5 {
            low.log10().floor()
        } else if low < 0.9 {
            0.0
        } else {
            -(1.0 - low).log10().ceil()
        };
        let last = if high <= 0.5 {
            high.log10().ceil()
        } else if high <= 0.9 {
            1.0
        } else {
            -(1.0 - high).log10().floor()
        };
        if !first.is_finite() || !last.is_finite() {
            return Vec::new();
        }
        // Both indices lie within ±400: the ends are positive doubles below 1.
        let (first, last) = (first as i32, last as i32);
        let ideal = usize::try_from(last - first - 1).unwrap_or(0);
        let mut z = if ideal >= 2 {
            // At most `ideal`, so within ±400 too.
            let factor = ideal.div_ceil(count.max(1)) as i32;
            (first..=last)
                .filter(|index| index.rem_euclid(factor) == 0)
                .map(ideal_tick)
                .collect()
        } else {
            ticks::linear([low, high], count)
        };
        z.retain(|v| (a..=b).contains(v));
        z.sort_by(f64::total_cmp);
        if reverse {
            z.reverse();
        }
        z
    }

    /// The domain unchanged: nicing towards 0 or 1 would reach the ends the
    /// transform sends to infinity.
    fn nice(&self, domain: [f64; 2], _count: usize) -> [f64; 2] {
        domain
    }

    /// Every tick labelled, the specifier defaulting to `".12~g"`.
    fn tick_format(
        &self,
        locale: &Locale,
        _domain: [f64; 2],
        _count: usize,
        specifier: Option<&str>,
    ) -> Result<TickFormat, InvalidFormat> {
        let specifier: Specifier = specifier.unwrap_or(".12~g").parse()?;
        Ok(TickFormat::from(Format::new(locale, specifier)))
    }
}

/// matplotlib's `LogitLocator.nonsingular` on ordered ends, with no axis:
/// non-finite ends, or none inside (0, 1), give (10⁻⁷, 1 − 10⁻⁷); an end
/// at or beyond 0 or 1 moves to 10⁻⁷ from it; equal ends v spread to
/// (0.1v, 1 − 0.1v).
fn nonsingular(a: f64, b: f64) -> [f64; 2] {
    if !a.is_finite() || !b.is_finite() || b <= 0.0 || a >= 1.0 {
        return [MIN_POSITIVE, 1.0 - MIN_POSITIVE];
    }
    let low = if a <= 0.0 { MIN_POSITIVE } else { a };
    let high = if b >= 1.0 { 1.0 - MIN_POSITIVE } else { b };
    if low == high {
        [0.1 * low, 1.0 - 0.1 * low]
    } else {
        [low, high]
    }
}

/// The ideal tick of index `b`: 10ᵇ below 0, 0.5 at 0, 1 − 10⁻ᵇ above.
fn ideal_tick(b: i32) -> f64 {
    match b.signum() {
        -1 => ticks::pow10(f64::from(b)),
        0 => 0.5,
        _ => 1.0 - ticks::pow10(f64::from(-b)),
    }
}
