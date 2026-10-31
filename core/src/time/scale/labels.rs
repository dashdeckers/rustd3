//! [`Time`]'s tick labels: d3's multi-scale tick format (`time.js:29–36`)
//! through the locale's ladder, in the scale's zone, and [`Ticks`] for the
//! time scale.

use jiff::Timestamp;
use jiff::civil::Weekday;

use crate::locale::Locale;
use crate::scale::Ticks;
use crate::ticks;
use crate::time::locale::Rung;
use crate::time::{self, Interval, Time};

/// The time scale's ticks: d3's `ticks(count)` with the interval they
/// were chosen with, positioned by `apply` and labelled tick by tick
/// through the locale's time half.
///
/// A day in Berlin across the end of daylight saving time, with d3's
/// 3-hour interval, under the neutral locale's ISO ladder and d3's:
///
/// ```
/// use rustd3_core::jiff::{Timestamp, tz::TimeZone};
/// use rustd3_core::locale::Locale;
/// use rustd3_core::scale::Ticks;
/// use rustd3_core::time::{self, Interval};
///
/// let berlin = TimeZone::posix("CET-1CEST,M3.5.0,M10.5.0/3")?;
/// let day: [Timestamp; 2] = ["2026-10-24T22:00:00Z".parse()?, "2026-10-25T23:00:00Z".parse()?];
/// let st = time::time(berlin).domain(day).range([0.0, 800.0]);
/// assert_eq!(st.tick_interval(8), Interval::Hour(3));
/// let set = st.tick_set(8);
/// assert_eq!(
///     st.labels(&set, &Locale::default()),
///     ["10-25", "03:00", "06:00", "09:00", "12:00", "15:00", "18:00", "21:00", "10-26"]
/// );
/// assert_eq!(
///     st.labels(&set, &Locale::EN_US),
///     ["Oct 25", "03 AM", "06 AM", "09 AM", "12 PM", "03 PM", "06 PM", "09 PM", "Mon 26"]
/// );
/// let x = st.position(&set.values()[1], 0.0).expect("a position");
/// assert!((x - 800.0 * 4.0 / 25.0).abs() < 1e-9);
/// # Ok::<(), rustd3_core::jiff::Error>(())
/// ```
impl Ticks for Time {
    type Domain = Timestamp;
    type Step = Interval;

    /// `ticks(count)`, chosen with `tick_interval(count)`.
    fn tick_set(&self, count: usize) -> ticks::Set<Timestamp, Interval> {
        ticks::Set::new(self.ticks(count), self.tick_interval(count), count)
    }

    /// `apply(value)`; the offset is unused.
    fn position(&self, value: &Timestamp, _offset: f64) -> Option<f64> {
        Some(self.apply(*value))
    }

    /// [`tick_label`](Time::tick_label) of each tick, independently, under
    /// `locale`'s time half; the set's step is not consulted.
    fn labels(&self, ticks: &ticks::Set<Timestamp, Interval>, locale: &Locale) -> Vec<String> {
        ticks
            .values()
            .iter()
            .map(|&t| self.tick_label(t, locale.time()))
            .collect()
    }

    fn extent(&self) -> [f64; 2] {
        self.get_range()
    }
}

impl Time {
    /// d3's multi-scale tick format through `locale`'s ladder, in the
    /// scale's zone: `t` labelled at the finest unit it is not aligned to.
    ///
    /// Off the whole second the rung follows `t`'s own fraction:
    /// nanoseconds when it has a non-zero part below the microsecond, else
    /// microseconds when below the millisecond, else milliseconds. Then, on
    /// the second but not the minute, seconds; minutes; hours; on a day's
    /// start, days, or weeks on a Sunday; months on a month's first day;
    /// years on January 1. A boundary past jiff's range counts as aligned.
    ///
    /// ```
    /// use rustd3_core::jiff::{Timestamp, tz::TimeZone};
    /// use rustd3_core::time::{self, Locale};
    ///
    /// let la = TimeZone::posix("PST8PDT,M3.2.0,M11.1.0")?;
    /// let x = time::time(la);
    /// let at = |s: &str| s.parse::<Timestamp>();
    /// assert_eq!(x.tick_label(at("1990-01-01T08:00:00.012Z")?, &Locale::EN_US), ".012");
    /// assert_eq!(x.tick_label(at("1990-01-02T08:00:00Z")?, &Locale::EN_US), "Tue 02");
    /// assert_eq!(x.tick_label(at("1990-02-01T08:00:00Z")?, &Locale::EN_US), "February");
    /// assert_eq!(x.tick_label(at("1990-02-01T08:00:00Z")?, &Locale::NEUTRAL), "1990-02");
    /// assert_eq!(x.tick_label(at("1990-02-01T08:00:00.000001Z")?, &Locale::NEUTRAL), "00.000001");
    /// # Ok::<(), rustd3_core::jiff::Error>(())
    /// ```
    #[must_use]
    pub fn tick_label(&self, t: Timestamp, locale: &time::Locale) -> String {
        locale
            .format(locale.get_ladder().rung(self.rung(t)))
            .format(t, &self.zone)
    }

    /// The rung `t` is labelled at (d3's `tickFormat` choice, with the
    /// sub-second rungs by `t`'s fraction).
    fn rung(&self, t: Timestamp) -> Rung {
        let off = |interval: Interval| interval.floor(t, &self.zone).is_some_and(|floor| floor < t);
        let ns = t.as_nanosecond();
        if off(Interval::Second(1)) {
            if ns.rem_euclid(1_000) != 0 {
                Rung::Nanosecond
            } else if ns.rem_euclid(1_000_000) != 0 {
                Rung::Microsecond
            } else {
                Rung::Millisecond
            }
        } else if off(Interval::Minute(1)) {
            Rung::Second
        } else if off(Interval::Hour(1)) {
            Rung::Minute
        } else if off(Interval::Day(1)) {
            Rung::Hour
        } else if off(Interval::Month(1)) {
            if off(Interval::Week(Weekday::Sunday, 1)) {
                Rung::Day
            } else {
                Rung::Week
            }
        } else if off(Interval::Year(1)) {
            Rung::Month
        } else {
            Rung::Year
        }
    }
}
