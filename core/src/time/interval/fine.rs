//! The fixed units, nanoseconds to seconds: multiples of a size since the
//! epoch in `i128` nanoseconds, the same in every zone (d3's
//! `millisecond.js` and `second.js`).

use jiff::Timestamp;

use super::at;

/// One nanosecond.
pub(super) const NANOSECOND: i128 = 1;
/// One microsecond, in nanoseconds.
pub(super) const MICROSECOND: i128 = 1_000;
/// One millisecond, in nanoseconds.
pub(super) const MILLISECOND: i128 = 1_000_000;
/// One second, in nanoseconds.
pub(super) const SECOND: i128 = 1_000_000_000;

/// The latest multiple of `size` nanoseconds since the epoch at or before
/// `t`.
pub(super) fn floor(t: Timestamp, size: i128) -> Option<Timestamp> {
    at(t.as_nanosecond().div_euclid(size) * size)
}

/// `t` plus `n` times `size` nanoseconds.
pub(super) fn offset(t: Timestamp, n: i64, size: i128) -> Option<Timestamp> {
    at(t.as_nanosecond()
        .checked_add(i128::from(n).checked_mul(size)?)?)
}

/// Multiples of `size` between the floors of `start` and `end`.
pub(super) fn count(start: Timestamp, end: Timestamp, size: i128) -> Option<i64> {
    units(floor(start, size)?, floor(end, size)?, size)
}

/// Whole `size`s from `start` to `end`, rounded down as d3 rounds a count.
pub(super) fn units(start: Timestamp, end: Timestamp, size: i128) -> Option<i64> {
    i64::try_from((end.as_nanosecond() - start.as_nanosecond()).div_euclid(size)).ok()
}

/// The second of the UTC minute, 0 to 59 (d3's `getUTCSeconds`; Unix time
/// has no leap seconds).
pub(super) fn second_of_minute(t: Timestamp) -> i64 {
    t.as_nanosecond().div_euclid(SECOND).rem_euclid(60) as i64
}
