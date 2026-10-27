//! Time with explicit zones, on jiff: [`Interval`], d3-time's calendar
//! intervals with microseconds and nanoseconds beside them, and the lazy
//! [`interval::Range`] of an interval's boundaries.
//!
//! Every zone is a parameter: nothing here reads the system's clock or
//! zone. Instants are jiff [`Timestamp`](jiff::Timestamp)s and arithmetic
//! on them is in `i128` nanoseconds, never through an `f64` epoch value
//! (in 2026 one of those resolves 256 ns). The zone is consulted only where
//! the calendar needs it, from minutes up.
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

/// A unit of time and a step: d3-time's intervals (`timeSecond` …
/// `timeYear`, `utcDay`, `unixDay`), with microseconds and nanoseconds
/// beside milliseconds. A closed set: d3's custom intervals are not ported.
///
/// The step is d3's `every`; a step of 0 behaves as 1. Fixed units take it
/// as a multiple of the unit since the epoch; the others keep the
/// boundaries whose field is a multiple of the step, as d3's filtered
/// intervals do.
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
