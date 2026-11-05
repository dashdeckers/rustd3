//! The wall-clock units, minutes and hours. They step by fixed durations
//! and floor on the local clock, using the zone's offset at the instant,
//! with one zone lookup per floor (d3's `minute.js` and `hour.js:4–6`).

use jiff::Timestamp;
use jiff::tz::TimeZone;

use super::fine::{self, SECOND};
use crate::time::at;

/// One minute, in nanoseconds.
pub(super) const MINUTE: i128 = 60 * SECOND;
/// One hour, in nanoseconds.
pub(super) const HOUR: i128 = 60 * MINUTE;

/// The latest instant at or before `t` whose local time in `tz` is a
/// multiple of `size` (a minute or an hour): `t` less the local remainder,
/// with the offset at `t`.
pub(super) fn floor(t: Timestamp, size: i128, tz: &TimeZone) -> Option<Timestamp> {
    let ns = t.as_nanosecond();
    let local = ns + i128::from(tz.to_offset(t).seconds()) * SECOND;
    at(ns - local.rem_euclid(size))
}

/// Units of `size` between the local floors of `start` and `end`.
pub(super) fn count(start: Timestamp, end: Timestamp, size: i128, tz: &TimeZone) -> Option<i64> {
    fine::units(floor(start, size, tz)?, floor(end, size, tz)?, size)
}

/// The minute of the local hour (d3's `getMinutes`).
pub(super) fn minute(t: Timestamp, tz: &TimeZone) -> i64 {
    i64::from(tz.to_datetime(t).minute())
}

/// The hour of the local day (d3's `getHours`).
pub(super) fn hour(t: Timestamp, tz: &TimeZone) -> i64 {
    i64::from(tz.to_datetime(t).hour())
}
