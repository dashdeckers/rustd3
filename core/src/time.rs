//! Time with explicit zones, on jiff: [`Interval`], d3-time's calendar
//! intervals with microseconds and nanoseconds beside them, the lazy
//! [`interval::Range`] of an interval's boundaries, d3's tick functions,
//! [`ticks`], [`ticks_with`], [`tick_interval`] and their `utc_` twins,
//! [`Time`], d3's time scale, made by [`time()`] and [`utc()`], and
//! d3-time-format's formats: a [`Format`] compiled under a [`Locale`]
//! (d3's timeFormatLocale plus the [`Ladder`] tick labels climb, built
//! from a [`Definition`]), or under the neutral locale by [`format()`], and
//! d3's [`iso_format`] and [`iso_parse`].
//!
//! Every zone is a parameter: nothing here reads the system's clock or
//! zone. Instants are jiff [`Timestamp`]s and arithmetic on them is in
//! `i128` nanoseconds, never through an `f64` epoch value (in 2026 one of
//! those resolves 256 ns). The zone is consulted only where the calendar
//! needs it, from minutes up.
//!
//! ```
//! use rustd3_core::jiff::{Timestamp, tz::TimeZone};
//! use rustd3_core::time::Interval;
//!
//! let t: Timestamp = "2026-10-25T12:00:00.123456789Z".parse()?;
//! let floor = Interval::Microsecond(5).floor(t, &TimeZone::UTC);
//! assert_eq!(floor, Some("2026-10-25T12:00:00.123455Z".parse()?));
//! let ticks: Vec<Timestamp> = Interval::Millisecond(50)
//!     .range(Timestamp::from_millisecond(1_000)?, Timestamp::from_millisecond(1_200)?, &TimeZone::UTC)
//!     .collect();
//! assert_eq!(ticks.len(), 4);
//! # Ok::<(), rustd3_core::jiff::Error>(())
//! ```

pub mod interval;

mod definition;
mod format;
mod locale;
// Generated: dev/harness/gen/time_locales.mjs writes the layout.
#[rustfmt::skip]
mod locales;
mod scale;
mod ticks;

use jiff::Timestamp;
use jiff::civil;
use jiff::tz::TimeZone;

pub use definition::Definition;
pub use format::{Format, format, iso_format, iso_parse};
pub use locale::{Ladder, Locale};
pub use scale::Time;
pub use ticks::{tick_interval, ticks, ticks_with, utc_tick_interval, utc_ticks};

/// A unit of time and a step: d3-time's intervals (`timeSecond` …
/// `timeYear`, `utcDay`, `unixDay`), with microseconds and nanoseconds
/// beside milliseconds. A closed set: d3's custom intervals are not ported.
///
/// The step is d3's `every`; a step of 0 behaves as 1. Fixed units take it
/// as a multiple of the unit since the epoch; the others keep the
/// boundaries whose field is a multiple of the step, as d3's filtered
/// intervals do.
///
/// Days follow the local calendar: in Los Angeles the Sunday daylight
/// saving time starts on is 23 hours long, and still one day.
///
/// ```
/// use rustd3_core::jiff::{Timestamp, tz::TimeZone};
/// use rustd3_core::time::Interval;
///
/// let la = TimeZone::posix("PST8PDT,M3.2.0,M11.1.0")?;
/// let at = |s: &str| s.parse::<Timestamp>();
/// let saturday = at("2011-03-12T08:00:00Z")?; // local midnight
/// let day = Interval::Day(1);
/// assert_eq!(day.offset(saturday, 1, &la), Some(at("2011-03-13T08:00:00Z")?));
/// assert_eq!(day.offset(saturday, 2, &la), Some(at("2011-03-14T07:00:00Z")?));
/// assert_eq!(day.floor(at("2011-03-13T20:00:00Z")?, &la), Some(at("2011-03-13T08:00:00Z")?));
/// assert_eq!(day.count(saturday, at("2011-03-14T07:00:00Z")?, &la), Some(2));
/// // every(k) keeps boundaries whose field is a multiple of k.
/// let every_other = day.every(2).expect("a step of 2");
/// let odd_days: Vec<Timestamp> =
///     every_other.range(saturday, at("2011-03-18T07:00:00Z")?, &la).collect();
/// let expected = ["2011-03-13T08:00:00Z", "2011-03-15T07:00:00Z", "2011-03-17T07:00:00Z"];
/// assert_eq!(odd_days, expected.map(|s| s.parse::<Timestamp>().expect("an instant")));
/// assert_eq!(day.every(0), None);
/// # Ok::<(), rustd3_core::jiff::Error>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Interval {
    /// Multiples of `k` nanoseconds since the epoch.
    Nanosecond(u32),
    /// Multiples of `k` microseconds since the epoch.
    Microsecond(u32),
    /// Multiples of `k` milliseconds since the epoch (d3's
    /// `timeMillisecond.every(k)`).
    Millisecond(u32),
    /// Whole seconds; `every(k)` keeps those whose second of the UTC minute
    /// is a multiple of `k` (d3's `timeSecond`, the same in every zone).
    Second(u32),
    /// Whole minutes of the local clock; `every(k)` keeps the local minutes
    /// that are multiples of `k`.
    Minute(u32),
    /// Whole hours of the local clock; `every(k)` keeps the local hours
    /// that are multiples of `k`.
    Hour(u32),
    /// Local days, starting at the day's first instant; `every(k)` keeps
    /// the days of the month ≡ 1 (mod k) (d3's `timeDay`, `utcDay` in UTC).
    Day(u32),
    /// Local days; `every(k)` keeps those whose count from 1970-01-01 in
    /// the zone's calendar is a multiple of `k` (d3's `unixDay` in UTC,
    /// which `utc` ticks use).
    UnixDay(u32),
    /// Weeks starting on the weekday (d3's `timeSunday` … `timeSaturday`;
    /// `timeWeek` is Sunday); `every(k)` keeps those a multiple of `k`
    /// weeks from the week that holds the epoch's local date.
    Week(jiff::civil::Weekday, u32),
    /// Local months; `every(k)` keeps the months whose index from January
    /// (0) is a multiple of `k`.
    Month(u32),
    /// Local years, starting on January 1; with a step `k`, the years that
    /// are multiples of `k` (d3's `timeYear.every`).
    Year(u32),
}

/// d3.scaleTime with an explicit zone: domain from local midnight on
/// 2000-01-01 to 2000-01-02 in `zone`, range [0, 1], ticks laid out in
/// `zone` (days of the month).
///
/// A day in Berlin across the end of daylight saving time: d3's
/// 3-hour interval, at local midnight, 03:00 … 21:00 and midnight again.
///
/// ```
/// use rustd3_core::jiff::{Timestamp, tz::TimeZone};
/// use rustd3_core::time::{self, Interval};
///
/// // Berlin's rules since 1996, without a time zone database.
/// let berlin = TimeZone::posix("CET-1CEST,M3.5.0,M10.5.0/3")?;
/// let day: [Timestamp; 2] = ["2026-10-24T22:00:00Z".parse()?, "2026-10-25T23:00:00Z".parse()?];
/// let x = time::time(berlin).domain(day).range([0.0, 800.0]);
/// assert_eq!(x.tick_interval(8), Interval::Hour(3));
/// let ticks = x.ticks(8);
/// let hours: Vec<i8> = ticks.iter().map(|&t| x.zone().to_datetime(t).hour()).collect();
/// assert_eq!(hours, [0, 3, 6, 9, 12, 15, 18, 21, 0]);
/// assert_eq!(ticks[1], "2026-10-25T02:00:00Z".parse()?);
/// # Ok::<(), rustd3_core::jiff::Error>(())
/// ```
#[doc(alias = "scaleTime")]
#[must_use]
pub fn time(zone: TimeZone) -> Time {
    Time::new(default_domain(&zone), zone, false)
}

/// d3.scaleUtc: domain 2000-01-01T00:00Z to 2000-01-02T00:00Z, range
/// [0, 1], ticks laid out in UTC with d3's utc intervals (days counted from
/// 1970-01-01, `UnixDay`).
///
/// It differs from `time(TimeZone::UTC)` only in its ticks' days:
///
/// ```
/// use rustd3_core::jiff::{Timestamp, tz::TimeZone};
/// use rustd3_core::time::{self, Interval};
///
/// let week: [Timestamp; 2] = ["2011-01-01T00:00:00Z".parse()?, "2011-01-09T00:00:00Z".parse()?];
/// let day_of_month = |x: &time::Time| -> Vec<i8> {
///     x.ticks(4).iter().map(|&t| x.zone().to_datetime(t).day()).collect()
/// };
/// let unix = time::utc().domain(week);
/// assert_eq!(unix.tick_interval(4), Interval::UnixDay(2));
/// assert_eq!(day_of_month(&unix), [2, 4, 6, 8]);
/// let calendar = time::time(TimeZone::UTC).domain(week);
/// assert_eq!(calendar.tick_interval(4), Interval::Day(2));
/// assert_eq!(day_of_month(&calendar), [1, 3, 5, 7, 9]);
/// # Ok::<(), rustd3_core::jiff::Error>(())
/// ```
#[doc(alias = "scaleUtc")]
#[must_use]
pub fn utc() -> Time {
    Time::new(default_domain(&TimeZone::UTC), TimeZone::UTC, true)
}

/// d3's default time domain: local midnight on 2000-01-01 and 2000-01-02
/// in `zone`.
fn default_domain(zone: &TimeZone) -> [Timestamp; 2] {
    [1, 2].map(|day| {
        zone.to_timestamp(civil::date(2000, 1, day).at(0, 0, 0, 0))
            .expect("invariant: January 2000 is within jiff's range in every zone")
    })
}

/// The instant `ns` nanoseconds from the epoch, if jiff can hold it.
pub(crate) fn at(ns: i128) -> Option<Timestamp> {
    // jiff-core 0.1.1's from_nanosecond checks only that the seconds fit
    // i64 (timestamp.rs:171–185): out of jiff's range it debug-asserts.
    let range = Timestamp::MIN.as_nanosecond()..=Timestamp::MAX.as_nanosecond();
    if range.contains(&ns) {
        Timestamp::from_nanosecond(ns).ok()
    } else {
        None
    }
}

/// The instant `ns` nanoseconds from the epoch, or jiff's first or last
/// instant past them.
pub(crate) fn saturating(ns: i128) -> Timestamp {
    let ns = ns.clamp(
        Timestamp::MIN.as_nanosecond(),
        Timestamp::MAX.as_nanosecond(),
    );
    Timestamp::from_nanosecond(ns).expect("invariant: clamped to jiff's range")
}
