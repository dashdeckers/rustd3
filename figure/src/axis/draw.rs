//! [`Draw`]: an axis with its scale type erased, as a panel holds it, and
//! drawing a resolved axis.

use std::f64::consts::FRAC_PI_2;

use kurbo::{BezPath, Cap, Point, Rect};

use rustd3_core::axis::Orient;
use rustd3_core::color::Rgb;
use rustd3_core::scale::Ticks;

use crate::axis::labels::{anchor, get};
use crate::axis::{Axis, Labels, across};
use crate::figure::Resolved;
use crate::list::{self, Command, List};
use crate::scale::{Kind, Position};
use crate::text::{self, Anchor};
use crate::theme::Theme;

/// An axis with its scale type erased: what layout and drawing ask of it.
pub(crate) trait Draw<'a>: 'a {
    /// The side of the panel it is on.
    fn orient(&self) -> Orient;

    /// Its palette color, when tinted and the palette has one for its key.
    fn tint(&self, resolved: &Resolved) -> Option<Rgb>;

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
    fn title(&self) -> Option<&text::Text>;

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

    fn tint(&self, resolved: &Resolved) -> Option<Rgb> {
        self.tint.as_ref().and_then(|tint| tint(resolved))
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

    fn title(&self) -> Option<&text::Text> {
        self.label.as_ref()
    }

    fn outward(&self) -> f64 {
        self.outward.unwrap_or(0.0)
    }
}

impl Labels {
    /// Appends the axis beside plot area `area` to `list`: its line in the
    /// theme's edge color (its own when tinted) with square caps, so two
    /// axes meet at a closed corner as matplotlib's projecting spines do,
    /// its ticks at `positions`
    /// (along the plot area, aligned with the ticks), its tick labels a tick
    /// pad past the ticks, and its title a label pad past the labels'
    /// extent, turned a quarter counter-clockwise on the left and clockwise
    /// on the right.
    pub(crate) fn draw(
        &self,
        positions: &[Option<f64>],
        area: Rect,
        text: &text::Cache,
        theme: &Theme,
        list: &mut List,
    ) {
        let orient = self.orient;
        let outward = if matches!(orient, Orient::Top | Orient::Left) {
            -1.0
        } else {
            1.0
        };
        // The axis line, and a point `across` px outward of it `along` px
        // from the plot area's start.
        let line = match orient {
            Orient::Top => area.y0 - self.outward,
            Orient::Bottom => area.y1 + self.outward,
            Orient::Left => area.x0 - self.outward,
            Orient::Right => area.x1 + self.outward,
        };
        let at = |along: f64, across_: f64| {
            if across(orient) {
                Point::new(area.x0 + along, line + outward * across_)
            } else {
                Point::new(line + outward * across_, area.y0 + along)
            }
        };
        let length = if across(orient) {
            area.width()
        } else {
            area.height()
        };
        let stroke = list::stroke(theme.axis_width());
        let edge = if self.tinted { self.ink } else { theme.edge() };
        let mut path = BezPath::new();
        path.move_to(at(0.0, 0.0));
        path.line_to(at(length, 0.0));
        list.commands.push(Command::Stroke {
            path,
            style: stroke.clone().with_caps(Cap::Square),
            paint: list::brush(edge),
        });
        let mut ticks = BezPath::new();
        for p in positions.iter().flatten() {
            ticks.move_to(at(*p, 0.0));
            ticks.line_to(at(*p, theme.tick()));
        }
        list.commands.push(Command::Stroke {
            path: ticks,
            style: stroke,
            paint: list::brush(self.ink),
        });
        let gap = theme.tick() + theme.tick_pad();
        for (key, p) in self.ticks.iter().zip(positions) {
            if let (Some(shaped), Some(p)) = (get(text, key), p) {
                shaped.place(list, at(*p, gap), anchor(orient), 0.0);
            }
        }
        if let Some(title) = self.title.as_ref().and_then(|key| get(text, key)) {
            let beyond = gap + self.extent(text) + theme.label_pad();
            let angle = match orient {
                Orient::Top | Orient::Bottom => 0.0,
                Orient::Left => -FRAC_PI_2,
                Orient::Right => FRAC_PI_2,
            };
            let v = if orient == Orient::Bottom {
                text::anchor::Vertical::Top
            } else {
                text::anchor::Vertical::Bottom
            };
            let anchor = Anchor {
                h: text::anchor::Horizontal::Middle,
                v,
            };
            title.place(list, at(length / 2.0, beyond), anchor, angle);
        }
    }
}
