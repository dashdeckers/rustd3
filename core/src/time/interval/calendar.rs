//! The calendar units, days to years: civil arithmetic on the local date
//! in the zone, resolved once per result as `Date` resolves a local time
//! (jiff's compatible disambiguation), so São Paulo's skipped midnight
//! floors to 01:00 and a day is not always 24 hours. Offsets keep the
//! local time of day and carry a day of the month past the month's end
//! into the next, as `setDate`, `setMonth` and `setFullYear` do (d3's
//! `day.js`, `week.js`, `month.js` and `year.js`).

use jiff::civil::{Date, DateTime, Time, Weekday};
use jiff::tz::TimeZone;
use jiff::{SignedDuration, Timestamp};

/// The civil date days are counted from (d3's `unixDay`).
const EPOCH: Date = Date::constant(1970, 1, 1);
/// One day, in seconds.
const DAY: i64 = 86_400;

/// The first instant of `t`'s local day.
pub(super) fn day_floor(t: Timestamp, tz: &TimeZone) -> Option<Timestamp> {
    midnight(tz.to_datetime(t).date(), tz)
}

/// The first instant of the local day of the latest weekday `w` at or
/// before `t`'s.
pub(super) fn week_floor(t: Timestamp, w: Weekday, tz: &TimeZone) -> Option<Timestamp> {
    midnight(date_at(week_day(t, w, tz))?, tz)
}

/// The first instant of `t`'s local month.
pub(super) fn month_floor(t: Timestamp, tz: &TimeZone) -> Option<Timestamp> {
    midnight(tz.to_datetime(t).date().first_of_month(), tz)
}

/// The first instant of January 1 of `t`'s local year floored to a
/// multiple of `k` (`year.js:15–26`).
pub(super) fn year_floor(t: Timestamp, k: i64, tz: &TimeZone) -> Option<Timestamp> {
    let year = i64::from(tz.to_datetime(t).year()).div_euclid(k) * k;
    midnight(Date::new(i16::try_from(year).ok()?, 1, 1).ok()?, tz)
}

/// `t` moved by `n` local days at the same local time.
pub(super) fn day_offset(t: Timestamp, n: i64, tz: &TimeZone) -> Option<Timestamp> {
    let local = tz.to_datetime(t);
    let date = date_at(day_number(local.date()).checked_add(n)?)?;
    resolve(date.to_datetime(local.time()), tz)
}

/// `t` moved by `n` local months at the same day of the month (carried
/// past the month's end) and local time.
pub(super) fn month_offset(t: Timestamp, n: i64, tz: &TimeZone) -> Option<Timestamp> {
    let local = tz.to_datetime(t);
    let month = i64::from(local.year()) * 12 + i64::from(local.month() - 1);
    shifted(local, month.checked_add(n)?, tz)
}

/// `t` moved by `n` local years at the same month, day (Feb 29 carried to
/// Mar 1) and local time.
pub(super) fn year_offset(t: Timestamp, n: i64, tz: &TimeZone) -> Option<Timestamp> {
    let local = tz.to_datetime(t);
    let month = i64::from(local.year())
        .checked_add(n)?
        .checked_mul(12)?
        .checked_add(i64::from(local.month() - 1))?;
    shifted(local, month, tz)
}

/// Local days from 1970-01-01 to `t`'s local date.
pub(super) fn day_of_epoch(t: Timestamp, tz: &TimeZone) -> i64 {
    day_number(tz.to_datetime(t).date())
}

/// Weeks from the week of weekday `w` that holds `start`'s local date to
/// the one that holds `end`'s.
pub(super) fn weeks(start: Timestamp, end: Timestamp, w: Weekday, tz: &TimeZone) -> i64 {
    (week_day(end, w, tz) - week_day(start, w, tz)) / 7
}

/// The local day of the month, from 1.
pub(super) fn day_of_month(t: Timestamp, tz: &TimeZone) -> i64 {
    i64::from(tz.to_datetime(t).day())
}

/// The local month as months since January of year 0.
pub(super) fn month_index(t: Timestamp, tz: &TimeZone) -> i64 {
    let local = tz.to_datetime(t);
    i64::from(local.year()) * 12 + i64::from(local.month() - 1)
}

/// The local year.
pub(super) fn year(t: Timestamp, tz: &TimeZone) -> i64 {
    i64::from(tz.to_datetime(t).year())
}

/// The day number of the latest weekday `w` at or before `t`'s local date.
fn week_day(t: Timestamp, w: Weekday, tz: &TimeZone) -> i64 {
    let date = tz.to_datetime(t).date();
    let back = (date.weekday().to_sunday_zero_offset() - w.to_sunday_zero_offset()).rem_euclid(7);
    day_number(date) - i64::from(back)
}

/// `local`'s day of the month and time in month `month` (months since year
/// 0), the day carried past the month's end.
fn shifted(local: DateTime, month: i64, tz: &TimeZone) -> Option<Timestamp> {
    let year = i16::try_from(month.div_euclid(12)).ok()?;
    let first = Date::new(year, i8::try_from(month.rem_euclid(12) + 1).ok()?, 1).ok()?;
    let date = date_at(day_number(first) + i64::from(local.day() - 1))?;
    resolve(date.to_datetime(local.time()), tz)
}

/// The first instant of `date` in `tz`.
fn midnight(date: Date, tz: &TimeZone) -> Option<Timestamp> {
    resolve(date.to_datetime(Time::midnight()), tz)
}

/// The instant of the local `dt`, as `Date` resolves it: in a gap the
/// offset before the transition, in a fold the earlier instant.
fn resolve(dt: DateTime, tz: &TimeZone) -> Option<Timestamp> {
    tz.to_timestamp(dt).ok()
}

/// Days from 1970-01-01 to `date`.
fn day_number(date: Date) -> i64 {
    date.duration_since(EPOCH).as_secs().div_euclid(DAY)
}

/// The date `day` days from 1970-01-01, if jiff holds it.
fn date_at(day: i64) -> Option<Date> {
    EPOCH
        .checked_add(SignedDuration::from_secs(day.checked_mul(DAY)?))
        .ok()
}
