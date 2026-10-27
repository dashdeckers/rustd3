//! d3-array's ticks: about `count` nicely rounded values between two
//! bounds, their step, and bounds extended to them.
//!
//! The algorithm is d3's `tickSpec`, with bail-outs where d3 would return
//! `[NaN]`, throw, or allocate without bound: a non-finite increment or
//! tick index, or more ticks than `min(10_000, ⌈10 · max(count, 1)⌉)`,
//! give no ticks.
//!
//! ```
//! use rustd3_core::ticks;
//!
//! assert_eq!(ticks::ticks(-10.0, 10.0, 5.0), [-10.0, -5.0, 0.0, 5.0, 10.0]);
//! assert_eq!(ticks::nice(0.13, 9.71, 5.0), [0.0, 10.0]);
//! ```
//!
//! The tick rules that scale transforms state as their own are building
//! blocks here too: [`linear`], [`log`] and [`subdivide`].

mod rules;

pub(crate) use rules::pow10;
pub use rules::{linear, log, subdivide};

use crate::js;

/// √50: above it, the step rounds to 10.
const E10: f64 = 7.0710678118654755;
/// √10: above it, the step rounds to 5.
const E5: f64 = 3.1622776601683795;
/// √2: above it, the step rounds to 2.
const E2: f64 = std::f64::consts::SQRT_2;
/// The absolute ceiling on the number of ticks.
const MAX_TICKS: f64 = 10_000.0;
/// `nice` gives up extending after this many steps.
const NICE_ITERATIONS: usize = 10;

/// d3's tickSpec: the first and last tick index and the increment.
#[derive(Debug, Clone, Copy)]
struct Spec {
    i1: f64,
    i2: f64,
    /// The step, or for steps below 1 its negated inverse, so that every
    /// tick is an exact integer divided by an exact integer.
    increment: f64,
}

impl Spec {
    /// d3's tickSpec for start ≤ stop.
    fn new(start: f64, stop: f64, count: f64) -> Spec {
        // JavaScript's Math.max(0, count) propagates NaN; f64::max drops it.
        let divisor = if count.is_nan() || count > 0.0 {
            count
        } else {
            0.0
        };
        let step = (stop - start) / divisor;
        let power = step.log10().floor();
        let error = step / 10f64.powf(power);
        let factor = if error >= E10 {
            10.0
        } else if error >= E5 {
            5.0
        } else if error >= E2 {
            2.0
        } else {
            1.0
        };
        let (mut i1, mut i2, increment);
        if power < 0.0 {
            let inverse = 10f64.powf(-power) / factor;
            i1 = js::round(start * inverse);
            i2 = js::round(stop * inverse);
            if i1 / inverse < start {
                i1 += 1.0;
            }
            if i2 / inverse > stop {
                i2 -= 1.0;
            }
            increment = -inverse;
        } else {
            increment = 10f64.powf(power) * factor;
            i1 = js::round(start / increment);
            i2 = js::round(stop / increment);
            if i1 * increment < start {
                i1 += 1.0;
            }
            if i2 * increment > stop {
                i2 -= 1.0;
            }
        }
        // No tick fits: retry with twice the count, as d3 does (at most twice).
        if i2 < i1 && (0.5..2.0).contains(&count) {
            return Spec::new(start, stop, count * 2.0);
        }
        Spec { i1, i2, increment }
    }
}

/// d3.ticks: about `count` nicely rounded values in [start, stop], in the
/// order of start → stop. `count` is f64 as in d3 (NaN, 0.5 and negative
/// counts are meaningful there and tested).
pub fn ticks(start: f64, stop: f64, count: f64) -> Vec<f64> {
    if count.is_nan() || count <= 0.0 {
        return Vec::new();
    }
    if start == stop {
        return vec![start];
    }
    let reverse = stop < start;
    let Spec { i1, i2, increment } = if reverse {
        Spec::new(stop, start, count)
    } else {
        Spec::new(start, stop, count)
    };
    // d3 returns [NaN] when the increment overflows and throws a RangeError
    // when its inverse does (subnormal spans).
    if !increment.is_finite() || !i1.is_finite() || !i2.is_finite() || i2 < i1 {
        return Vec::new();
    }
    let n = i2 - i1 + 1.0;
    if n > (10.0 * count.max(1.0)).ceil().min(MAX_TICKS) {
        return Vec::new();
    }
    let Some(n) = js::to_int(n).and_then(|n| usize::try_from(n).ok()) else {
        return Vec::new();
    };
    (0..n)
        .map(|i| {
            let k = if reverse {
                i2 - i as f64
            } else {
                i1 + i as f64
            };
            if increment < 0.0 {
                k / -increment
            } else {
                k * increment
            }
        })
        .collect()
}

/// d3.tickIncrement: the tick step for [start, stop] (start ≤ stop), or for
/// steps below 1 its negated inverse.
#[doc(alias = "tickIncrement")]
pub fn tick_increment(start: f64, stop: f64, count: f64) -> f64 {
    Spec::new(start, stop, count).increment
}

/// d3.tickStep: the signed difference between adjacent ticks of [start, stop].
#[doc(alias = "tickStep")]
pub fn tick_step(start: f64, stop: f64, count: f64) -> f64 {
    let reverse = stop < start;
    let increment = if reverse {
        tick_increment(stop, start, count)
    } else {
        tick_increment(start, stop, count)
    };
    let step = if increment < 0.0 {
        1.0 / -increment
    } else {
        increment
    };
    if reverse { -step } else { step }
}

/// d3.nice: [start, stop] extended to tick boundaries.
pub fn nice(mut start: f64, mut stop: f64, count: f64) -> [f64; 2] {
    let mut previous = None;
    for _ in 0..NICE_ITERATIONS {
        let step = tick_increment(start, stop, count);
        if previous == Some(step) || step == 0.0 || !step.is_finite() {
            break;
        }
        if step > 0.0 {
            start = (start / step).floor() * step;
            stop = (stop / step).ceil() * step;
        } else {
            start = (start * step).ceil() / step;
            stop = (stop * step).floor() / step;
        }
        previous = Some(step);
    }
    [start, stop]
}
