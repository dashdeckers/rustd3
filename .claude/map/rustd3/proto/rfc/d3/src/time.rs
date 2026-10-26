//! Time scales on jiff, with the time zone explicit instead of the process-local `Date`.

use crate::interpolate::Interpolate;
use crate::scale::TickSource;
use jiff::{Timestamp, ToSpan, Zoned, tz::TimeZone};

/// A calendar interval: d3's `timeSecond`, `timeHour`, `timeMonth` and friends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interval {
    Second(u32),
    Minute(u32),
    Hour(u32),
    Day(u32),
    Week,
    Month(u32),
    Year(u32),
}

impl Interval {
    /// The latest boundary at or before `t`.
    pub fn floor(self, t: &Zoned) -> Zoned {
        let t = t.clone();
        let trunc = |t: Zoned, sec: bool, min: bool, hour: bool| {
            let mut b = t.with().subsec_nanosecond(0);
            if sec { b = b.second(0); }
            if min { b = b.minute(0); }
            if hour { b = b.hour(0); }
            b.build().expect("truncating within a day stays valid")
        };
        match self {
            Interval::Second(_) => trunc(t, false, false, false),
            Interval::Minute(_) => trunc(t, true, false, false),
            Interval::Hour(_) => trunc(t, true, true, false),
            Interval::Day(_) => t.start_of_day().expect("day start exists"),
            Interval::Week => {
                let back = t.weekday().to_sunday_zero_offset() as i64;
                t.start_of_day().expect("day start exists").checked_sub(back.days()).expect("in range")
            }
            Interval::Month(_) => t.first_of_month().expect("valid").start_of_day().expect("day start exists"),
            Interval::Year(_) => t.first_of_year().expect("valid").start_of_day().expect("day start exists"),
        }
    }

    fn step(self, t: &Zoned) -> Zoned {
        let span = match self {
            Interval::Second(_) => 1.second(),
            Interval::Minute(_) => 1.minute(),
            Interval::Hour(_) => 1.hour(),
            Interval::Day(_) => 1.day(),
            Interval::Week => 1.week(),
            Interval::Month(_) => 1.month(),
            Interval::Year(_) => 1.year(),
        };
        t.checked_add(span).expect("in range")
    }

    /// d3's `interval.every(k)`: keep boundaries whose field is a multiple of `k`.
    fn keep(self, t: &Zoned) -> bool {
        match self {
            Interval::Second(k) => t.second() as u32 % k == 0,
            Interval::Minute(k) => t.minute() as u32 % k == 0,
            Interval::Hour(k) => t.hour() as u32 % k == 0,
            Interval::Day(k) => (t.day() as u32 - 1) % k == 0,
            Interval::Week => true,
            Interval::Month(k) => (t.month() as u32 - 1) % k == 0,
            Interval::Year(k) => t.year() as u32 % k == 0,
        }
    }

    /// Every boundary in `[start, stop)`.
    pub fn range(self, start: &Zoned, stop: &Zoned) -> Vec<Zoned> {
        let mut t = self.floor(start);
        if t < *start {
            t = self.step(&t);
        }
        let mut out = Vec::new();
        while t < *stop {
            if self.keep(&t) {
                out.push(t.clone());
            }
            t = self.step(&t);
        }
        out
    }
}

/// `d3.scaleTime()` with an explicit zone.
#[derive(Debug, Clone, PartialEq)]
pub struct Time {
    domain: [Timestamp; 2],
    range: [f64; 2],
    tz: TimeZone,
}

pub fn time(tz: TimeZone) -> Time {
    Time { domain: [Timestamp::UNIX_EPOCH, Timestamp::UNIX_EPOCH + 24.hours()], range: [0.0, 1.0], tz }
}

impl Time {
    pub fn domain(self, domain: [Timestamp; 2]) -> Self {
        Self { domain, ..self }
    }

    pub fn range(self, range: [f64; 2]) -> Self {
        Self { range, ..self }
    }

    fn ms(t: Timestamp) -> f64 {
        t.as_millisecond() as f64
    }

    pub fn apply(&self, t: Timestamp) -> f64 {
        let [d0, d1] = self.domain.map(Self::ms);
        self.range[0].lerp(&self.range[1], (Self::ms(t) - d0) / (d1 - d0))
    }

    pub fn invert(&self, y: f64) -> Timestamp {
        let [r0, r1] = self.range;
        let [d0, d1] = self.domain.map(Self::ms);
        Timestamp::from_millisecond(d0.lerp(&d1, (y - r0) / (r1 - r0)).round() as i64).expect("in range")
    }

    pub fn get_domain(&self) -> [Timestamp; 2] {
        self.domain
    }

    pub fn get_range(&self) -> [f64; 2] {
        self.range
    }

    /// The interval d3 would pick for about `count` ticks.
    pub fn tick_interval(&self, count: usize) -> Interval {
        const S: f64 = 1e3;
        const M: f64 = 60.0 * S;
        const H: f64 = 60.0 * M;
        const D: f64 = 24.0 * H;
        let table = [
            (Interval::Second(1), S), (Interval::Second(5), 5.0 * S), (Interval::Second(15), 15.0 * S),
            (Interval::Second(30), 30.0 * S), (Interval::Minute(1), M), (Interval::Minute(5), 5.0 * M),
            (Interval::Minute(15), 15.0 * M), (Interval::Minute(30), 30.0 * M), (Interval::Hour(1), H),
            (Interval::Hour(3), 3.0 * H), (Interval::Hour(6), 6.0 * H), (Interval::Hour(12), 12.0 * H),
            (Interval::Day(1), D), (Interval::Day(2), 2.0 * D), (Interval::Week, 7.0 * D),
            (Interval::Month(1), 30.0 * D), (Interval::Month(3), 90.0 * D), (Interval::Year(1), 365.0 * D),
        ];
        let [d0, d1] = self.domain.map(Self::ms);
        let target = (d1 - d0).abs() / count as f64;
        let i = table.partition_point(|&(_, ms)| ms <= target);
        if i == table.len() {
            let years = d3_core::ticks::tick_step(d0 / (365.0 * D), d1 / (365.0 * D), count as f64);
            return Interval::Year(years.max(1.0) as u32);
        }
        if i == 0 {
            return Interval::Second(1);
        }
        let (lo, hi) = (table[i - 1], table[i]);
        if target / lo.1 < hi.1 / target { lo.0 } else { hi.0 }
    }

    pub fn ticks(&self, count: usize) -> Vec<Zoned> {
        let [a, b] = self.domain.map(|t| t.to_zoned(self.tz.clone()));
        let end = b.checked_add(1.nanosecond()).expect("in range");
        self.tick_interval(count).range(&a, &end)
    }

    /// d3's default multi-scale time format: the coarsest field that changed.
    pub fn tick_label(t: &Zoned) -> String {
        let at = |i: Interval| Interval::floor(i, t) < *t;
        let fmt = if t.subsec_nanosecond() != 0 {
            ".%3f"
        } else if at(Interval::Minute(1)) {
            ":%S"
        } else if at(Interval::Hour(1)) {
            "%I:%M"
        } else if at(Interval::Day(1)) {
            "%I %p"
        } else if at(Interval::Month(1)) {
            if at(Interval::Week) { "%a %d" } else { "%b %d" }
        } else if at(Interval::Year(1)) {
            "%B"
        } else {
            "%Y"
        };
        t.strftime(fmt).to_string()
    }
}

impl TickSource for Time {
    type Value = Zoned;
    fn tick_values(&self, count: usize) -> Vec<Zoned> {
        self.ticks(count)
    }
    fn position(&self, v: &Zoned) -> f64 {
        self.apply(v.timestamp())
    }
    fn label(&self, v: &Zoned, _count: usize) -> String {
        Self::tick_label(v)
    }
    fn extent(&self) -> [f64; 2] {
        self.range
    }
}
