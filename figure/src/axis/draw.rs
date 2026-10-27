//! [`Draw`]: an axis with its scale type erased, as a panel holds it.

use rustd3_core::axis::Orient;
use rustd3_core::color::Rgb;
use rustd3_core::scale::Ticks;

use crate::axis::{Axis, across};
use crate::figure::Resolved;
use crate::scale::{Kind, Position};
use crate::text::Text;

/// An axis with its scale type erased: what layout and drawing ask of it.
pub(crate) trait Draw<'a>: 'a {
    /// The side of the panel it is on.
    fn orient(&self) -> Orient;

    /// Its color: its palette color when tinted and the palette has one,
    /// else `ink`.
    fn ink(&self, resolved: &Resolved, ink: Rgb) -> Rgb;

    /// Whether its tick labels and title are drawn.
    fn labeled(&self) -> bool;

    /// The tick labels of the resolved scale.
    fn texts(&self, resolved: &Resolved) -> Vec<String>;

    /// Each tick's position along a panel `length` px long, aligned with
    /// [`texts`](Draw::texts): across [0, length] for a horizontal axis, up
    /// [length, 0] for a vertical one; `None` for a key outside the domain
    /// or a position that is not finite.
    fn positions(&self, resolved: &Resolved, length: f64) -> Vec<Option<f64>>;

    /// The title along the axis.
    fn title(&self) -> Option<&Text>;

    /// How far it sits from the plot area, px.
    fn outward(&self) -> f64;
}

impl<'a, S> Draw<'a> for Axis<'a, S>
where
    S: Position + Ticks<Domain = <S as Kind>::Domain>,
{
    fn orient(&self) -> Orient {
        self.orient
    }

    fn ink(&self, resolved: &Resolved, ink: Rgb) -> Rgb {
        self.tint
            .as_ref()
            .and_then(|tint| tint(resolved))
            .unwrap_or(ink)
    }

    fn labeled(&self) -> bool {
        self.labels.unwrap_or(true)
    }

    fn texts(&self, resolved: &Resolved) -> Vec<String> {
        Axis::texts(self, resolved.scale(self.scale))
    }

    fn positions(&self, resolved: &Resolved, length: f64) -> Vec<Option<f64>> {
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
        self.set(ranged)
            .values()
            .iter()
            .map(|v| Ticks::position(ranged, v, 0.0).filter(|p| p.is_finite()))
            .collect()
    }

    fn title(&self) -> Option<&Text> {
        self.label.as_ref()
    }

    fn outward(&self) -> f64 {
        self.outward.unwrap_or(0.0)
    }
}
