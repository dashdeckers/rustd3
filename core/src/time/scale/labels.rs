//! [`Time`]'s tick labels: d3's multi-scale tick format (`time.js:29–36`)
//! through the locale's ladder, in the scale's zone.

use jiff::Timestamp;
use jiff::civil::Weekday;

use crate::time::locale::Rung;
use crate::time::{self, Interval, Time};

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
    pub fn tick_label(&self, t: Timestamp, locale: &time::Locale) -> String {
        locale
            .format(locale.ladder().rung(self.rung(t)))
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
