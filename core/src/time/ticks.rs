//! d3's time ticks, `timeTicks`, `timeTickInterval`, `utcTicks` and
//! `utcTickInterval` (`d3-time/src/ticks.js:13–56`), with an explicit zone
//! and below a second down to nanoseconds.

use jiff::civil::Weekday;
use jiff::tz::TimeZone;
use jiff::{SignedDuration, Timestamp};

use crate::ticks::tick_step;
use crate::time::Interval;

/// One second, in nanoseconds.
const SECOND: i128 = 1_000_000_000;
/// One minute, in nanoseconds.
const MINUTE: i128 = 60 * SECOND;
/// One hour, in nanoseconds.
const HOUR: i128 = 60 * MINUTE;
/// One day, in nanoseconds.
const DAY: i128 = 24 * HOUR;
/// d3's month, 30 days, in nanoseconds.
const MONTH: i128 = 30 * DAY;
/// d3's year, 365 days, in nanoseconds.
const YEAR: i128 = 365 * DAY;
/// The most ticks any call returns.
const MAX_TICKS: usize = 10_000;

/// d3's table of tick intervals and their nominal lengths; the `utc`
/// functions take `UnixDay` for `Day`.
const TABLE: [(Interval, i128); 18] = [
    (Interval::Second(1), SECOND),
    (Interval::Second(5), 5 * SECOND),
    (Interval::Second(15), 15 * SECOND),
    (Interval::Second(30), 30 * SECOND),
    (Interval::Minute(1), MINUTE),
    (Interval::Minute(5), 5 * MINUTE),
    (Interval::Minute(15), 15 * MINUTE),
    (Interval::Minute(30), 30 * MINUTE),
    (Interval::Hour(1), HOUR),
    (Interval::Hour(3), 3 * HOUR),
    (Interval::Hour(6), 6 * HOUR),
    (Interval::Hour(12), 12 * HOUR),
    (Interval::Day(1), DAY),
    (Interval::Day(2), 2 * DAY),
    (Interval::Week(Weekday::Sunday, 1), 7 * DAY),
    (Interval::Month(1), MONTH),
    (Interval::Month(3), 3 * MONTH),
    (Interval::Year(1), YEAR),
];

/// d3.timeTickInterval: the interval of about `count` ticks between
/// `start` and `stop`. The table's entry nearest to `|stop − start| /
/// count` by ratio; beyond a year, years stepped by d3-array's tick step;
/// below a second, milliseconds, microseconds or nanoseconds stepped by it
/// (d3 stops at whole milliseconds). The choice depends on the span alone;
/// `zone` is the one [`ticks`] lays the interval out in.
#[doc(alias = "timeTickInterval")]
pub fn tick_interval(
    start: Timestamp,
    stop: Timestamp,
    count: usize,
    _zone: &TimeZone,
) -> Interval {
    choose(start, stop, count)
}

/// d3.timeTicks: about `count` boundaries of [`tick_interval`] in `zone`
/// from `start` to `stop`, both included, in the order of `start → stop`;
/// at most `min(10·count, 10_000)`, so a count of 0 gives none. Equal
/// ends give one tick.
#[doc(alias = "timeTicks")]
pub fn ticks(start: Timestamp, stop: Timestamp, count: usize, zone: &TimeZone) -> Vec<Timestamp> {
    let interval = tick_interval(start, stop, count, zone);
    lay_out(start, stop, interval, zone, cap(count))
}

/// d3.timeTicks with an interval: its boundaries in `zone` from `start` to
/// `stop`, both included, in the order of `start → stop`; at most 10,000.
#[doc(alias = "timeTicks")]
pub fn ticks_with(
    start: Timestamp,
    stop: Timestamp,
    interval: Interval,
    zone: &TimeZone,
) -> Vec<Timestamp> {
    lay_out(start, stop, interval, zone, MAX_TICKS)
}

/// d3.utcTickInterval: [`tick_interval`] with days counted from
/// 1970-01-01 (`UnixDay`) where it would take days of the month.
#[doc(alias = "utcTickInterval")]
pub fn utc_tick_interval(start: Timestamp, stop: Timestamp, count: usize) -> Interval {
    let interval = choose(start, stop, count);
    if let Interval::Day(k) = interval {
        Interval::UnixDay(k)
    } else {
        interval
    }
}

/// d3.utcTicks: [`ticks`] in UTC over [`utc_tick_interval`].
#[doc(alias = "utcTicks")]
pub fn utc_ticks(start: Timestamp, stop: Timestamp, count: usize) -> Vec<Timestamp> {
    let interval = utc_tick_interval(start, stop, count);
    lay_out(start, stop, interval, &TimeZone::UTC, cap(count))
}

/// d3's `tickInterval` on the span in `i128` nanoseconds.
fn choose(start: Timestamp, stop: Timestamp, count: usize) -> Interval {
    let span = (stop.as_nanosecond() - start.as_nanosecond()).abs();
    let count = count as f64;
    let target = span as f64 / count;
    let i = TABLE.partition_point(|&(_, length)| length as f64 <= target);
    if i == TABLE.len() {
        let years = tick_step(0.0, span as f64 / YEAR as f64, count);
        // A float to integer cast saturates, and NaN casts to 0.
        return Interval::Year(years.max(1.0) as u32);
    }
    if i == 0 {
        let step = tick_step(0.0, span as f64, count).max(1.0);
        return if step >= 1e6 {
            Interval::Millisecond((step / 1e6) as u32)
        } else if step >= 1e3 {
            Interval::Microsecond((step / 1e3) as u32)
        } else {
            Interval::Nanosecond(step as u32)
        };
    }
    let ((below, shorter), (above, longer)) = (TABLE[i - 1], TABLE[i]);
    if target / (shorter as f64) < (longer as f64) / target {
        below
    } else {
        above
    }
}

/// The boundaries of `interval` from the earlier end to the later, both
/// included, at most `cap` of them, in the order of `start → stop`.
fn lay_out(
    start: Timestamp,
    stop: Timestamp,
    interval: Interval,
    zone: &TimeZone,
    cap: usize,
) -> Vec<Timestamp> {
    let reverse = stop < start;
    let (low, high) = if reverse {
        (stop, start)
    } else {
        (start, stop)
    };
    // d3's `+stop + 1` makes the stop inclusive; jiff's last instant is no
    // interval's boundary but the nanosecond's.
    let end = high
        .checked_add(SignedDuration::from_nanos(1))
        .unwrap_or(high);
    let mut ticks: Vec<Timestamp> = interval.range(low, end, zone).take(cap).collect();
    if reverse {
        ticks.reverse();
    }
    ticks
}

/// d3-array's cap on a tick count: `min(10·count, 10_000)`.
fn cap(count: usize) -> usize {
    count.saturating_mul(10).min(MAX_TICKS)
}
