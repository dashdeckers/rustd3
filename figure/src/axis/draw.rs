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
use crate::scale::{Kind, Position};
use crate::text;

/// An axis with its scale type erased: what layout and drawing ask of it.
pub(crate) trait Draw<'a>: 'a {
    /// The side of the panel it is on.
    fn orient(&self) -> Orient;

    /// Its handle's position in the figure's scale table.
    fn handle(&self) -> usize;

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

    /// Each of about `count` ticks' position along a panel `length` px
    /// long, aligned with [`texts`](Draw::texts): across [0, length] for a
    /// horizontal axis, up [length, 0] for a vertical one; `None` for a key
    /// outside the domain or a position that is not finite.
    fn positions(&self, resolved: &Resolved, length: f64, count: usize) -> Vec<Option<f64>>;

    /// With `.minor(true)`, the positions of core's minor ticks for about
    /// `count` ticks ([`Ticks::minor`]) along a panel `length` px long, as
    /// [`positions`](Draw::positions) places ticks, those not finite left
    /// out; none otherwise.
    fn minors(&self, resolved: &Resolved, length: f64, count: usize) -> Vec<f64>;

    /// Each of about `count` ticks' absolute index, aligned with
    /// [`texts`](Draw::texts), which the stride of the collision pass counts
    /// from: the value over d3's tick step when the ticks are evenly spaced
    /// on that grid; for a time scale, the units of its tick interval from
    /// the Unix epoch floored in its zone; the key's position in the domain
    /// for band and point scales; the position in the set otherwise (log
    /// ticks).
    fn indices(&self, resolved: &Resolved, count: usize) -> Vec<i64>;

    /// For a time scale, whether each of about `count` ticks follows the
    /// one before it by fewer units than its tick interval's step (the
    /// first stepped day of a month after a month's last, the first stepped
    /// hour after a clock change), aligned with [`texts`](Draw::texts); empty
    /// for other scales.
    fn restarts(&self, resolved: &Resolved, count: usize) -> Vec<bool>;

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
        self.scale.index()
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
        Axis::texts(self, resolved.scale(self.scale), count, resolved.locale())
    }

    fn formats(&self) -> bool {
        self.format.is_some()
    }

    fn positions(&self, resolved: &Resolved, length: f64, count: usize) -> Vec<Option<f64>> {
        let ranged = self.ranged(resolved, length);
        self.set(&ranged, count)
            .values()
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
        let scale = resolved.scale(self.scale);
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
        let values = set.values();
        let positions = (0..values.len() as i64).collect();
        if numbers.is_some() || self.values.is_none() {
            return positions;
        }
        // Keys chosen by `.values`: their places among the scale's own
        // ticks, which for band and point scales are the domain's keys.
        let place = |key| Position::position(scale, key).map(f64::to_bits);
        let full = scale.tick_set(count);
        let mut at = HashMap::new();
        for (i, key) in full.values().iter().enumerate() {
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

    fn restarts(&self, resolved: &Resolved, count: usize) -> Vec<bool> {
        let Some((time, instants)) = self.instants(resolved, count) else {
            return Vec::new();
        };
        let interval = time.tick_interval(count);
        let zone = time.get_zone();
        let step = i64::from(step(interval));
        let follows = |pair: &[Timestamp]| {
            interval
                .count(pair[0], pair[1], zone)
                .is_some_and(|units| units < step)
        };
        let after = instants.windows(2).map(follows);
        std::iter::once(false)
            .chain(after)
            .take(instants.len())
            .collect()
    }

    fn notation(&self, resolved: &Resolved, count: usize) -> Option<String> {
        let locale = resolved.locale().number();
        Axis::notation(self, resolved.scale(self.scale), count).map(|n| n.label(locale))
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
        let scale = resolved.scale(self.scale);
        let time = scale.as_any().downcast_ref::<Time>()?;
        let set = self.set(scale, count);
        let instants = set.values().iter().map(|v| (v as &dyn Any).downcast_ref());
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
            .scale(self.scale)
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

/// Each of `instants`' count of units of `time`'s tick interval for about
/// `count` ticks from the Unix epoch floored to that interval in the
/// scale's zone (Phase 27's `Interval::count`, the step ignored), so the
/// kept labels stay on the same instants while a time axis pans; `None`
/// when a boundary or count leaves jiff's range or `i64`.
fn intervals(time: &Time, instants: &[Timestamp], count: usize) -> Option<Vec<i64>> {
    let interval = time.tick_interval(count);
    let zone = time.get_zone();
    let epoch = interval.floor(Timestamp::UNIX_EPOCH, zone)?;
    instants
        .iter()
        .map(|&t| interval.count(epoch, t, zone))
        .collect()
}
