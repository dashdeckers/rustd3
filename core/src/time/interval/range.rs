//! [`Range`]: an interval's boundaries between two instants, one at a time.

use std::iter::FusedIterator;

use jiff::Timestamp;
use jiff::tz::TimeZone;

use crate::time::Interval;

/// The boundaries of an interval in `[ceil(start), stop)`, every `step`-th
/// one, computed as they are taken (d3's `interval.range`, which builds an
/// array). Each next boundary is `floor(offset(previous, step))`, and the
/// range ends at the first that is not strictly between the previous one
/// and `stop`, or that leaves jiff's range. It owns a clone of the zone,
/// which shares jiff's data.
///
/// A day holds 86.4 trillion nanoseconds; taking the first three computes
/// three.
///
/// ```
/// use rustd3_core::jiff::{Timestamp, tz::TimeZone};
/// use rustd3_core::time::Interval;
///
/// let at = |s: &str| s.parse::<Timestamp>();
/// let (start, stop) = (at("2026-10-25T00:00:00Z")?, at("2026-10-26T00:00:00Z")?);
/// let mut nanos = Interval::Nanosecond(1).range(start, stop, &TimeZone::UTC);
/// assert_eq!(nanos.nth(2), Some(at("2026-10-25T00:00:00.000000002Z")?));
/// let hours = Interval::Hour(1).range_step(start, stop, 6, &TimeZone::UTC);
/// let six_hourly: Vec<i8> = hours.map(|t| t.to_zoned(TimeZone::UTC).hour()).collect();
/// assert_eq!(six_hourly, [0, 6, 12, 18]);
/// # Ok::<(), rustd3_core::jiff::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct Range {
    interval: Interval,
    zone: TimeZone,
    step: u32,
    stop: Timestamp,
    next: Option<Timestamp>,
}

impl Range {
    /// The range from `ceil(start)`, empty unless that is before `stop` and
    /// the step is at least 1.
    pub(super) fn new(
        interval: Interval,
        start: Timestamp,
        stop: Timestamp,
        step: u32,
        zone: &TimeZone,
    ) -> Range {
        let next = if step == 0 {
            None
        } else {
            interval.ceil(start, zone).filter(|&first| first < stop)
        };
        Range {
            interval,
            zone: zone.clone(),
            step,
            stop,
            next,
        }
    }
}

impl Iterator for Range {
    type Item = Timestamp;

    fn next(&mut self) -> Option<Timestamp> {
        let current = self.next?;
        self.next = self
            .interval
            .offset(current, i64::from(self.step), &self.zone)
            .and_then(|t| self.interval.floor(t, &self.zone))
            .filter(|&following| current < following && following < self.stop);
        Some(current)
    }
}

impl FusedIterator for Range {}
