//! Where a recording's x values come from, and the x types it can hold.

mod timestamp;

use std::fmt;

use crate::js;

/// An x type a recording can hold: `f64`, or jiff's `Timestamp`.
pub trait Coordinate: Copy + PartialOrd + fmt::Debug + 'static + crate::sealed::Coordinate {
    /// A regular clock's step: `f64` for `f64`, `SignedDuration` for `Timestamp`.
    type Period: Copy + PartialEq + fmt::Debug;

    /// `start + index · period`, in the type's own precision.
    fn at(start: Self, period: Self::Period, index: u64) -> Self;

    /// An estimate of the first index whose `at` is ≥ `value`, saturating
    /// at 0 for `value < start`; the recording adjusts it against `at`.
    fn index(start: Self, period: Self::Period, value: Self) -> u64;

    /// Whether a value may be stored (finite for `f64`; any instant but
    /// jiff's last, where `at` saturates, for `Timestamp`).
    fn is_valid(self) -> bool;

    /// Whether a period is usable (finite and positive for `f64`, positive
    /// for `SignedDuration`).
    fn is_valid_period(period: Self::Period) -> bool;
}

/// Where a recording's x values come from.
#[derive(Debug, Clone, PartialEq)]
pub enum Clock<X: Coordinate> {
    /// Sample i (counted from the first sample ever appended) is at
    /// `start + i · period`; `drop_front` keeps this numbering.
    Regular {
        /// The x of the first sample ever appended.
        start: X,
        /// The step between consecutive samples.
        period: X::Period,
    },
    /// One stored, non-decreasing x per sample.
    Explicit(Vec<X>),
}

impl Coordinate for f64 {
    type Period = f64;

    fn at(start: f64, period: f64, index: u64) -> f64 {
        start + index as f64 * period
    }

    /// `⌈(value − start) / period⌉`, 0 below `start` and for NaN, and
    /// `u64::MAX` past 2⁵³.
    fn index(start: f64, period: f64, value: f64) -> u64 {
        let estimate = ((value - start) / period).ceil().max(0.0);
        js::to_int(estimate).map_or(u64::MAX, i64::unsigned_abs)
    }

    fn is_valid(self) -> bool {
        self.is_finite()
    }

    fn is_valid_period(period: f64) -> bool {
        period.is_finite() && period > 0.0
    }
}

impl crate::sealed::Coordinate for f64 {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_index_estimate_saturates_below_the_start_and_past_two_to_the_fifty_third() {
        assert_eq!(f64::index(0.0, 1.0, -5.0), 0);
        assert_eq!(f64::index(0.0, 1.0, f64::NAN), 0);
        assert_eq!(f64::index(0.0, 1.0, 2.5), 3);
        assert_eq!(f64::index(0.0, 1.0, 1e300), u64::MAX);
        assert_eq!(f64::index(0.0, 1.0, f64::INFINITY), u64::MAX);
    }

    #[test]
    fn periods_must_be_finite_and_positive() {
        assert!(f64::is_valid_period(1e-300));
        for period in [0.0, -0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(!f64::is_valid_period(period), "{period}");
        }
    }
}
