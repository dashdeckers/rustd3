//! [`Interval`]'s operations, and [`Range`], the lazy sequence of its
//! boundaries.
//!
//! Each unit has a floor and an offset by whole units. A step either
//! scales the unit (the fixed units) or filters its boundaries by a field,
//! as d3's `interval.filter` does (`interval.js:37–68`): the floor steps
//! back to the latest boundary that passes, and an offset of `n` moves
//! unit by unit until it has passed `|n|` of them. Results that leave
//! jiff's range (years −9999 to 9999), where d3 has an Invalid Date, are
//! `None`.

mod clock;
mod fine;
mod range;

use jiff::Timestamp;
use jiff::tz::TimeZone;

use crate::time::Interval;

pub use range::Range;

impl Interval {
    /// The latest boundary at or before `t` (d3's `interval.floor`).
    pub fn floor(self, t: Timestamp, tz: &TimeZone) -> Option<Timestamp> {
        let mut floor = self.unit_floor(t, tz)?;
        while !self.keeps(floor, tz) {
            floor = self.unit_floor(before(floor)?, tz)?;
        }
        Some(floor)
    }

    /// The earliest boundary at or after `t` (d3's `interval.ceil`):
    /// `floor(offset(floor(t − 1 ns), 1))`.
    pub fn ceil(self, t: Timestamp, tz: &TimeZone) -> Option<Timestamp> {
        let floor = self.floor(before(t)?, tz)?;
        self.floor(self.offset(floor, 1, tz)?, tz)
    }

    /// The nearer of the floor and the ceil of `t`, the ceil on a tie (d3's
    /// `interval.round`).
    pub fn round(self, t: Timestamp, tz: &TimeZone) -> Option<Timestamp> {
        let (floor, ceil) = (self.floor(t, tz)?, self.ceil(t, tz)?);
        let ns = t.as_nanosecond();
        Some(if ns - floor.as_nanosecond() < ceil.as_nanosecond() - ns {
            floor
        } else {
            ceil
        })
    }

    /// `t` moved by `steps` intervals and not floored (d3's
    /// `interval.offset`): a fixed unit moves by `steps` multiples of its
    /// step; a filtered one moves unit by unit until it has passed `|steps|`
    /// kept boundaries.
    pub fn offset(self, t: Timestamp, steps: i64, tz: &TimeZone) -> Option<Timestamp> {
        if !self.filters() {
            return self.unit_offset(t, steps);
        }
        let direction = steps.signum();
        let mut t = t;
        for _ in 0..steps.unsigned_abs() {
            t = self.unit_offset(t, direction)?;
            while !self.keeps(t, tz) {
                t = self.unit_offset(t, direction)?;
            }
        }
        Some(t)
    }

    /// Units between `floor(start)` and `floor(end)`, the unit's own floors
    /// with the step ignored (d3's `interval.count`; an extension for
    /// stepped intervals, which have no count in d3). `None` when the count
    /// leaves `i64` (nanoseconds over more than 292 years).
    pub fn count(self, start: Timestamp, end: Timestamp, tz: &TimeZone) -> Option<i64> {
        match self {
            Interval::Nanosecond(_) => fine::count(start, end, fine::NANOSECOND),
            Interval::Microsecond(_) => fine::count(start, end, fine::MICROSECOND),
            Interval::Millisecond(_) => fine::count(start, end, fine::MILLISECOND),
            Interval::Second(_) => fine::count(start, end, fine::SECOND),
            Interval::Minute(_) => clock::count(start, end, clock::MINUTE, tz),
            Interval::Hour(_) => clock::count(start, end, clock::HOUR, tz),
        }
    }

    /// The same unit with step `k`, or `None` for 0 (d3's `every`, which
    /// returns null there).
    pub fn every(self, k: u32) -> Option<Interval> {
        (k > 0).then_some(match self {
            Interval::Nanosecond(_) => Interval::Nanosecond(k),
            Interval::Microsecond(_) => Interval::Microsecond(k),
            Interval::Millisecond(_) => Interval::Millisecond(k),
            Interval::Second(_) => Interval::Second(k),
            Interval::Minute(_) => Interval::Minute(k),
            Interval::Hour(_) => Interval::Hour(k),
        })
    }

    /// The boundaries in `[ceil(start), stop)`, lazily (d3's
    /// `interval.range`); empty unless `ceil(start) < stop`.
    pub fn range(self, start: Timestamp, stop: Timestamp, tz: &TimeZone) -> Range {
        Range::new(self, start, stop, 1, tz)
    }

    /// Every `step`-th boundary from `ceil(start)` before `stop` (d3's
    /// `interval.range(start, stop, step)`); empty for a step of 0.
    pub fn range_step(self, start: Timestamp, stop: Timestamp, step: u32, tz: &TimeZone) -> Range {
        Range::new(self, start, stop, step, tz)
    }

    /// Whether the step filters the unit's boundaries rather than scaling
    /// the unit.
    fn filters(self) -> bool {
        match self {
            Interval::Nanosecond(_) | Interval::Microsecond(_) | Interval::Millisecond(_) => false,
            Interval::Second(k) | Interval::Minute(k) | Interval::Hour(k) => k > 1,
        }
    }

    /// Whether the boundary `t` passes the step's filter: its field is a
    /// multiple of the step.
    fn keeps(self, t: Timestamp, tz: &TimeZone) -> bool {
        let multiple = |field: i64, k: u32| field % i64::from(k) == 0;
        match self {
            Interval::Nanosecond(_) | Interval::Microsecond(_) | Interval::Millisecond(_) => true,
            Interval::Second(k) => k <= 1 || multiple(fine::second_of_minute(t), k),
            Interval::Minute(k) => k <= 1 || multiple(clock::minute(t, tz), k),
            Interval::Hour(k) => k <= 1 || multiple(clock::hour(t, tz), k),
        }
    }

    /// The unit's floor, scaled by the step for the fixed units.
    fn unit_floor(self, t: Timestamp, tz: &TimeZone) -> Option<Timestamp> {
        match self {
            Interval::Nanosecond(k) => fine::floor(t, fine::NANOSECOND * scale(k)),
            Interval::Microsecond(k) => fine::floor(t, fine::MICROSECOND * scale(k)),
            Interval::Millisecond(k) => fine::floor(t, fine::MILLISECOND * scale(k)),
            Interval::Second(_) => fine::floor(t, fine::SECOND),
            Interval::Minute(_) => clock::floor(t, clock::MINUTE, tz),
            Interval::Hour(_) => clock::floor(t, clock::HOUR, tz),
        }
    }

    /// `t` moved by `n` units, scaled by the step for the fixed units.
    fn unit_offset(self, t: Timestamp, n: i64) -> Option<Timestamp> {
        match self {
            Interval::Nanosecond(k) => fine::offset(t, n, fine::NANOSECOND * scale(k)),
            Interval::Microsecond(k) => fine::offset(t, n, fine::MICROSECOND * scale(k)),
            Interval::Millisecond(k) => fine::offset(t, n, fine::MILLISECOND * scale(k)),
            Interval::Second(_) => fine::offset(t, n, fine::SECOND),
            Interval::Minute(_) => fine::offset(t, n, clock::MINUTE),
            Interval::Hour(_) => fine::offset(t, n, clock::HOUR),
        }
    }
}

/// The factor a step scales a fixed unit by: 0 reads as 1.
fn scale(k: u32) -> i128 {
    i128::from(k.max(1))
}

/// The instant `ns` nanoseconds from the epoch, if jiff can hold it.
fn at(ns: i128) -> Option<Timestamp> {
    // jiff-core 0.1.1's from_nanosecond checks only that the seconds fit
    // i64 (timestamp.rs:171–185): out of jiff's range it debug-asserts.
    let range = Timestamp::MIN.as_nanosecond()..=Timestamp::MAX.as_nanosecond();
    if range.contains(&ns) {
        Timestamp::from_nanosecond(ns).ok()
    } else {
        None
    }
}

/// The instant 1 ns before `t`.
fn before(t: Timestamp) -> Option<Timestamp> {
    at(t.as_nanosecond() - 1)
}
