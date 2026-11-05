//! [`Draw`]: an axis with its scale type erased, as a panel holds it.

use std::any::Any;
use std::collections::HashMap;

use rustd3_core::axis::Orient;
use rustd3_core::color::Rgb;
use rustd3_core::jiff::Timestamp;
use rustd3_core::scale::Ticks;
use rustd3_core::time::{Interval, Time};

use crate::axis::{Axis, across};
use crate::figure::Resolved;
use crate::layout::collide;
use crate::scale::{Kind, Position, ScaleRef};
use crate::text;

/// An axis with its scale type erased: what layout and drawing ask of it.
pub(crate) trait Draw<'a>: 'a {
    /// The side of the panel it is on.
    fn orient(&self) -> Orient;

    /// Its handle's position in the figure's scale table.
    fn handle(&self) -> usize;
    /// Its handle, its scale type erased.
    fn scale(&self) -> ScaleRef<()>;
    /// The palette its tint reads, if tinted.
    fn tint_handle(&self) -> Option<ScaleRef<()>>;

    /// Its palette color, when tinted and the palette has one for its key.
    fn tint(&self, resolved: &Resolved) -> Option<Rgb>;

    /// Whether `.labels(b)` set its tick labels and title on or off.
    fn labels(&self) -> Option<bool>;

    /// The tick count `.ticks(n)` set.
    fn count(&self) -> Option<usize>;

    /// The offset `.outward(px)` set, px from the plot area.
    fn outward(&self) -> Option<f64>;

    /// The tick labels' rotation `.rotate(deg)` set, degrees
    /// counter-clockwise; 0 by default.
    fn rotation(&self) -> f64;

    /// The tick labels of the resolved scale's about `count` ticks, in the
    /// figure's locale.
    fn texts(&self, resolved: &Resolved, count: usize) -> Vec<String>;

    /// Whether `.format` sets its labels, so an empty one is the user's,
    /// not the scale's blanking.
    fn formats(&self) -> bool;

    /// The position of each of about `count` ticks along a panel `length`
    /// px long, aligned with [`texts`](Draw::texts). Positions run across
    /// [0, length] for a horizontal axis and up [length, 0] for a vertical
    /// one. `None` for a key outside the domain or a position that is not
    /// finite.
    fn positions(&self, resolved: &Resolved, length: f64, count: usize) -> Vec<Option<f64>>;

    /// The positions of core's minor ticks for about `count` ticks
    /// ([`Ticks::minor`]) along a panel `length` px long, placed as
    /// [`positions`](Draw::positions) places ticks. Positions that are not
    /// finite are left out. Empty unless the axis has `.minor(true)`.
    fn minors(&self, resolved: &Resolved, length: f64, count: usize) -> Vec<f64>;

    /// The absolute index of each of about `count` ticks, aligned with
    /// [`texts`](Draw::texts). The collision pass counts its stride from
    /// these indices. The index is:
    ///
    /// - for a time scale, the units of its tick interval from the Unix
    ///   epoch floored in its zone;
    /// - the value over d3's tick step, when the ticks are evenly spaced on
    ///   that grid;
    /// - for band and point scales, the key's position in the domain;
    /// - otherwise (log ticks), the position in the set.
    fn indices(&self, resolved: &Resolved, count: usize) -> Vec<i64>;

    /// For a time scale, how each of about `count` ticks follows the one
    /// before it, aligned with [`texts`](Draw::texts). A tick follows by
    /// its tick interval's step, or by fewer units (the first stepped day of
    /// a month after a month's last, the first stepped hour after a clock
    /// change). A tick that follows by fewer units also records whether it
    /// starts its parent unit (the month of stepped days, the day of
    /// stepped hours, as `Interval::floor` one unit up finds it), since its
    /// label is then the coarser one of the two. Empty for other scales.
    fn restarts(&self, resolved: &Resolved, count: usize) -> Vec<collide::Restart>;

    /// The offset or multiplier label of about `count` ticks
    /// (`Axis::offset`), when the axis has one.
    fn notation(&self, resolved: &Resolved, count: usize) -> Option<String>;

    /// The title along the axis.
    fn title(&self) -> Option<&text::Text>;
}

impl<'a, S> Draw<'a> for Axis<'a, S>
where
    S: Position + Ticks<Domain = <S as Kind>::Domain>,
{
    fn orient(&self) -> Orient {
        self.orient
    }

    fn handle(&self) -> usize {
        self.scale.get_index()
    }

    fn scale(&self) -> ScaleRef<()> {
        self.scale.erased()
    }

    fn tint_handle(&self) -> Option<ScaleRef<()>> {
        self.palette
    }

    fn tint(&self, resolved: &Resolved) -> Option<Rgb> {
        self.tint.as_ref().and_then(|tint| tint(resolved))
    }

    fn labels(&self) -> Option<bool> {
        self.labels
    }

    fn count(&self) -> Option<usize> {
        self.count
    }

    fn outward(&self) -> Option<f64> {
        self.outward
    }

    fn rotation(&self) -> f64 {
        self.rotate
    }

    fn texts(&self, resolved: &Resolved, count: usize) -> Vec<String> {
        Axis::texts(self, resolved.own(self.scale), count, resolved.locale())
    }

    fn formats(&self) -> bool {
        self.format.is_some()
    }

    fn positions(&self, resolved: &Resolved, length: f64, count: usize) -> Vec<Option<f64>> {
        let ranged = self.ranged(resolved, length);
        self.set(&ranged, count)
            .get_values()
            .iter()
            .map(|v| Ticks::position(&ranged, v, 0.0).filter(|p| p.is_finite()))
            .collect()
    }

    fn minors(&self, resolved: &Resolved, length: f64, count: usize) -> Vec<f64> {
        if !self.minor {
            return Vec::new();
        }
        let ranged = self.ranged(resolved, length);
        let set = self.set(&ranged, count);
        Ticks::minor(&ranged, &set)
            .iter()
            .filter_map(|v| Ticks::position(&ranged, v, 0.0).filter(|p| p.is_finite()))
            .collect()
    }

    fn indices(&self, resolved: &Resolved, count: usize) -> Vec<i64> {
        let scale = resolved.own(self.scale);
        if let Some((time, instants)) = self.instants(resolved, count)
            && let Some(indices) = intervals(time, &instants, count)
        {
            return indices;
        }
        let numbers = self.numbers(scale, count);
        if let Some(grid) = numbers
            .as_ref()
            .and_then(|(values, domain)| collide::grid(values, *domain, count))
        {
            return grid;
        }
        let set = self.set(scale, count);
        let values = set.get_values();
        let positions = (0..values.len() as i64).collect();
        if numbers.is_some() || self.values.is_none() {
            return positions;
        }
        // Keys chosen by `.values`: their places among the scale's own
        // ticks, which for band and point scales are the domain's keys.
        let place = |key| Position::position(scale, key).map(f64::to_bits);
        let full = scale.tick_set(count);
        let mut at = HashMap::new();
        for (i, key) in full.get_values().iter().enumerate() {
            if let Some(bits) = place(key) {
                at.entry(bits).or_insert(i as i64);
            }
        }
        values
            .iter()
            .zip(positions)
            .map(|(key, i)| {
                place(key)
                    .and_then(|bits| at.get(&bits).copied())
                    .unwrap_or(i)
            })
            .collect()
    }

    fn restarts(&self, resolved: &Resolved, count: usize) -> Vec<collide::Restart> {
        let Some((time, instants)) = self.instants(resolved, count) else {
            return Vec::new();
        };
        let interval = time.tick_interval(count);
        let zone = time.get_zone();
        let step = i64::from(step(interval));
        let starts = |t: Timestamp| parent(interval).is_some_and(|p| p.floor(t, zone) == Some(t));
        let follows = |pair: &[Timestamp]| {
            let close = interval
                .count(pair[0], pair[1], zone)
                .is_some_and(|units| units < step);
            match (close, starts(pair[1])) {
                (false, _) => collide::Restart::Step,
                (true, true) => collide::Restart::Starts,
                (true, false) => collide::Restart::Follows,
            }
        };
        let after = instants.windows(2).map(follows);
        std::iter::once(collide::Restart::Step)
            .chain(after)
            .take(instants.len())
            .collect()
    }

    fn notation(&self, resolved: &Resolved, count: usize) -> Option<String> {
        let locale = resolved.locale().get_number();
        Axis::notation(self, resolved.own(self.scale), count).map(|n| n.label(locale))
    }

    fn title(&self) -> Option<&text::Text> {
        self.label.as_ref()
    }
}

impl<S> Axis<'_, S>
where
    S: Position + Ticks<Domain = <S as Kind>::Domain>,
{
    /// The resolved time scale and its about `count` ticks' instants, when
    /// the axis's scale is a time scale.
    fn instants<'r>(
        &self,
        resolved: &'r Resolved,
        count: usize,
    ) -> Option<(&'r Time, Vec<Timestamp>)> {
        let scale = resolved.own(self.scale);
        let time = scale.as_any().downcast_ref::<Time>()?;
        let set = self.set(scale, count);
        let instants = set
            .get_values()
            .iter()
            .map(|v| (v as &dyn Any).downcast_ref());
        let instants: Option<Vec<Timestamp>> = instants.map(Option::<&Timestamp>::copied).collect();
        Some((time, instants?))
    }

    /// The resolved scale over a panel `length` px long: across [0,
    /// length] for a horizontal axis, up [length, 0] for a vertical one.
    fn ranged(&self, resolved: &Resolved, length: f64) -> S {
        let range = if across(self.orient) {
            [0.0, length]
        } else {
            [length, 0.0]
        };
        resolved
            .own(self.scale)
            .with_range(range)
            .as_any()
            .downcast_ref::<S>()
            .expect("invariant: a scale's with_range keeps its type")
            .clone()
    }
}

/// The step of `interval`: how many of its units a stepped interval's
/// boundaries lie apart when no month, day or clock change restarts them.
fn step(interval: Interval) -> u32 {
    match interval {
        Interval::Nanosecond(k)
        | Interval::Microsecond(k)
        | Interval::Millisecond(k)
        | Interval::Second(k)
        | Interval::Minute(k)
        | Interval::Hour(k)
        | Interval::Day(k)
        | Interval::UnixDay(k)
        | Interval::Week(_, k)
        | Interval::Month(k)
        | Interval::Year(k) => k,
    }
}

/// The unit at which a stepped `interval`'s runs restart, whose start a
/// time format labels on the coarser rung. It is the minute of stepped
/// seconds, the hour of stepped minutes, the day of stepped hours, the
/// month of stepped days and the year of stepped months. `None` for the
/// intervals whose steps count from the epoch and never restart.
fn parent(interval: Interval) -> Option<Interval> {
    match interval {
        Interval::Second(_) => Some(Interval::Minute(1)),
        Interval::Minute(_) => Some(Interval::Hour(1)),
        Interval::Hour(_) => Some(Interval::Day(1)),
        Interval::Day(_) => Some(Interval::Month(1)),
        Interval::Month(_) => Some(Interval::Year(1)),
        Interval::Nanosecond(_)
        | Interval::Microsecond(_)
        | Interval::Millisecond(_)
        | Interval::UnixDay(_)
        | Interval::Week(..)
        | Interval::Year(_) => None,
    }
}

/// The number of units of `time`'s tick interval (for about `count`
/// ticks) from the Unix epoch to each of `instants`, with the epoch
/// floored to that interval in the scale's zone (`Interval::count`,
/// ignoring the step). The labels the collision pass keeps so stay on the
/// same instants while a time axis pans. `None` when a boundary or count
/// leaves jiff's range or `i64`.
fn intervals(time: &Time, instants: &[Timestamp], count: usize) -> Option<Vec<i64>> {
    let interval = time.tick_interval(count);
    let zone = time.get_zone();
    let epoch = interval.floor(Timestamp::UNIX_EPOCH, zone)?;
    instants
        .iter()
        .map(|&t| interval.count(epoch, t, zone))
        .collect()
}
