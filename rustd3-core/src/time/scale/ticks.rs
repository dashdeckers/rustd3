//! [`Time`]'s ticks and nice: d3's time tick functions over the domain,
//! laid out in the scale's zone, with d3's utc intervals for a utc scale.

use jiff::Timestamp;

use crate::time::{self, Interval, Time};

impl Time {
    /// d3's tick interval for `ticks(count)`: [`time::tick_interval`] over
    /// the domain, or [`time::utc_tick_interval`] for [`time::utc`].
    #[must_use]
    pub fn tick_interval(&self, count: usize) -> Interval {
        let [start, stop] = self.domain;
        if self.utc_ticks {
            time::utc_tick_interval(start, stop, count)
        } else {
            time::tick_interval(start, stop, count)
        }
    }

    /// d3's `ticks(count)`: about `count` boundaries of
    /// [`tick_interval`](Time::tick_interval) in the scale's zone, from the
    /// domain's start to its end, both included and in the domain's order;
    /// at most `min(10·count, 10_000)`.
    #[must_use]
    pub fn ticks(&self, count: usize) -> Vec<Timestamp> {
        let [start, stop] = self.domain;
        if self.utc_ticks {
            time::utc_ticks(start, stop, count)
        } else {
            time::ticks(start, stop, count, &self.zone)
        }
    }

    /// d3's `ticks(interval)`: the boundaries of `interval` in the scale's
    /// zone over the domain, both ends included and in the domain's order;
    /// at most 10,000.
    #[must_use]
    pub fn ticks_with(&self, interval: Interval) -> Vec<Timestamp> {
        let [start, stop] = self.domain;
        time::ticks_with(start, stop, interval, &self.zone)
    }

    /// d3's `nice(count)`: [`nice_with`](Time::nice_with) the
    /// [`tick_interval`](Time::tick_interval) of `count`. A count of 0 has
    /// no interval in d3 and leaves the domain as it is.
    ///
    /// A day in Los Angeles from 00:12 to 23:48 extends to local midnights,
    /// or with weeks to the Sundays around it:
    ///
    /// ```
    /// use rustd3_core::jiff::{Timestamp, civil::Weekday, tz::TimeZone};
    /// use rustd3_core::time::{self, Interval};
    ///
    /// let la = TimeZone::posix("PST8PDT,M3.2.0,M11.1.0")?;
    /// let at = |s: &str| s.parse::<Timestamp>();
    /// let day = [at("2009-01-01T08:12:00Z")?, at("2009-01-02T07:48:00Z")?];
    /// let x = time::time(la).domain(day);
    /// let midnights = [at("2009-01-01T08:00:00Z")?, at("2009-01-02T08:00:00Z")?];
    /// assert_eq!(x.clone().nice(10).get_domain(), midnights);
    /// let weeks = x.clone().nice_with(Interval::Week(Weekday::Sunday, 1)).get_domain();
    /// assert_eq!(weeks, [at("2008-12-28T08:00:00Z")?, at("2009-01-04T08:00:00Z")?]);
    /// assert_eq!(x.clone().nice(0), x);
    /// # Ok::<(), rustd3_core::jiff::Error>(())
    /// ```
    #[must_use]
    pub fn nice(self, count: usize) -> Self {
        let interval = self.tick_interval(count);
        self.nice_with(interval)
    }

    /// d3's `nice(interval)`: the earlier end floored and the later ceiled
    /// to boundaries of `interval` in the scale's zone, a reversed domain
    /// staying reversed. The domain stays as it is when either boundary
    /// would leave jiff's range.
    #[must_use]
    pub fn nice_with(self, interval: Interval) -> Self {
        let [start, stop] = self.domain;
        let reversed = stop < start;
        let (low, high) = if reversed {
            (stop, start)
        } else {
            (start, stop)
        };
        let ends = interval
            .floor(low, &self.zone)
            .zip(interval.ceil(high, &self.zone));
        match ends {
            Some((low, high)) if reversed => self.domain([high, low]),
            Some((low, high)) => self.domain([low, high]),
            None => self,
        }
    }
}
