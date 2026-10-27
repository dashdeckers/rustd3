//! [`Draw`]: an axis with its scale type erased, as a panel holds it.

use std::collections::HashMap;

use rustd3_core::axis::Orient;
use rustd3_core::color::Rgb;
use rustd3_core::scale::Ticks;

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

    /// The tick labels of the resolved scale's about `count` ticks.
    fn texts(&self, resolved: &Resolved, count: usize) -> Vec<String>;

    /// Whether `.format` sets its labels, so an empty one is the user's,
    /// not the scale's blanking.
    fn formats(&self) -> bool;

    /// Each of about `count` ticks' position along a panel `length` px
    /// long, aligned with [`texts`](Draw::texts): across [0, length] for a
    /// horizontal axis, up [length, 0] for a vertical one; `None` for a key
    /// outside the domain or a position that is not finite.
    fn positions(&self, resolved: &Resolved, length: f64, count: usize) -> Vec<Option<f64>>;

    /// Each of about `count` ticks' absolute index, aligned with
    /// [`texts`](Draw::texts), which the stride of the collision pass counts
    /// from: the value over d3's tick step when the ticks are evenly spaced
    /// on that grid; the key's position in the domain for band and point
    /// scales; the position in the set otherwise (log ticks).
    fn indices(&self, resolved: &Resolved, count: usize) -> Vec<i64>;

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

    fn texts(&self, resolved: &Resolved, count: usize) -> Vec<String> {
        Axis::texts(self, resolved.scale(self.scale), count)
    }

    fn formats(&self) -> bool {
        self.format.is_some()
    }

    fn positions(&self, resolved: &Resolved, length: f64, count: usize) -> Vec<Option<f64>> {
        let range = if across(self.orient) {
            [0.0, length]
        } else {
            [length, 0.0]
        };
        let ranged = resolved.scale(self.scale).with_range(range);
        let ranged = ranged
            .as_any()
            .downcast_ref::<S>()
            .expect("invariant: a scale's with_range keeps its type");
        self.set(ranged, count)
            .values()
            .iter()
            .map(|v| Ticks::position(ranged, v, 0.0).filter(|p| p.is_finite()))
            .collect()
    }

    fn indices(&self, resolved: &Resolved, count: usize) -> Vec<i64> {
        let scale = resolved.scale(self.scale);
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

    fn notation(&self, resolved: &Resolved, count: usize) -> Option<String> {
        Axis::notation(self, resolved.scale(self.scale), count).map(|n| n.label())
    }

    fn title(&self) -> Option<&text::Text> {
        self.label.as_ref()
    }
}
