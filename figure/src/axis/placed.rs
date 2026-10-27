//! [`Placed`]: an axis as layout placed it, the boxes of its tick labels,
//! and drawing it.

use std::f64::consts::FRAC_PI_2;

use kurbo::{BezPath, Cap, Point, Rect};
use rustd3_core::axis::Orient;

use crate::axis::labels::{anchor, get, notation};
use crate::axis::{Labels, across};
use crate::layout::Plan;
use crate::list::{self, Command, List};
use crate::text::{self, Anchor};
use crate::theme::Theme;

/// An axis as layout placed it: its labels, its plan, its ticks' positions
/// along the plot area (aligned with the labels), its tick labels' boxes
/// and which of them the collision passes kept.
pub(crate) struct Placed {
    /// Its labels.
    pub(crate) labels: Labels,
    /// Its plan.
    pub(crate) plan: Plan,
    /// Its ticks' positions along the plot area.
    pub(crate) positions: Vec<Option<f64>>,
    /// Its tick labels' boxes in list px; `None` for one not placed.
    pub(crate) boxes: Vec<Option<Rect>>,
    /// Which tick labels are drawn.
    pub(crate) keep: Vec<bool>,
    /// How far above the plot area a vertical axis's offset or multiplier
    /// label sits, a tick pad below its bottom: the panel's top reach
    /// without those labels.
    pub(crate) above: f64,
}

impl Placed {
    /// The axis with `labels` and `plan` beside plot area `area`, its ticks
    /// at `positions`, every tick label kept: each label's box is its box
    /// (advance width by ink height) anchored a tick and a tick pad outside
    /// the axis line, `None` for a label that is not placed or that typst
    /// rejected.
    pub(crate) fn new(
        labels: Labels,
        plan: Plan,
        positions: Vec<Option<f64>>,
        area: Rect,
        text: &text::Cache,
        theme: &Theme,
    ) -> Placed {
        let mut placed = Placed {
            labels,
            plan,
            positions,
            boxes: Vec::new(),
            keep: Vec::new(),
            above: 0.0,
        };
        placed.boxes = placed.measure(area, text, theme);
        placed.keep = vec![true; placed.boxes.len()];
        placed
    }

    /// Each tick label's box beside plot area `area`.
    fn measure(&self, area: Rect, text: &text::Cache, theme: &Theme) -> Vec<Option<Rect>> {
        let spine = self.spine(area);
        let gap = theme.tick() + theme.tick_pad();
        let anchor = anchor(self.labels.orient);
        self.labels
            .ticks
            .iter()
            .zip(&self.positions)
            .map(|(key, p)| {
                let shaped = get(text, key)?;
                let at = spine.at((*p)?, gap);
                Some(shaped.bounds(anchor) + at.to_vec2())
            })
            .collect()
    }

    /// Appends the axis beside plot area `area` to `list`: its line in the
    /// theme's edge color (its own when tinted) with square caps, so two
    /// axes meet at a closed corner as matplotlib's projecting spines do,
    /// its ticks, its kept tick labels a tick pad past the ticks, its
    /// offset or multiplier label (at the right end of a horizontal axis, a
    /// tick pad beyond the labels; above the top end of a vertical one), and
    /// its title a label pad past all of them, turned a quarter
    /// counter-clockwise on the left and clockwise on the right.
    pub(crate) fn draw(&self, area: Rect, text: &text::Cache, theme: &Theme, list: &mut List) {
        let labels = &self.labels;
        let orient = labels.orient;
        let spine = self.spine(area);
        let length = spine.length();
        let stroke = list::stroke(theme.axis_width());
        let edge = if labels.tinted {
            labels.ink
        } else {
            theme.edge()
        };
        let mut path = BezPath::new();
        path.move_to(spine.at(0.0, 0.0));
        path.line_to(spine.at(length, 0.0));
        list.commands.push(Command::Stroke {
            path,
            style: stroke.clone().with_caps(Cap::Square),
            paint: list::brush(edge),
        });
        let mut ticks = BezPath::new();
        for p in self.positions.iter().flatten() {
            ticks.move_to(spine.at(*p, 0.0));
            ticks.line_to(spine.at(*p, theme.tick()));
        }
        list.commands.push(Command::Stroke {
            path: ticks,
            style: stroke,
            paint: list::brush(labels.ink),
        });
        let gap = theme.tick() + theme.tick_pad();
        let kept = labels.ticks.iter().zip(&self.positions).zip(&self.keep);
        for ((key, p), _) in kept.filter(|(_, keep)| **keep) {
            if let (Some(shaped), Some(p)) = (get(text, key), p) {
                shaped.place(list, spine.at(*p, gap), anchor(orient), 0.0);
            }
        }
        let extent = labels.extent(text);
        if let Some(shaped) = labels.notation.as_ref().and_then(|key| get(text, key)) {
            let at = if across(orient) {
                spine.at(length, gap + extent + theme.tick_pad())
            } else {
                spine.at(0.0, 0.0) - kurbo::Vec2::new(0.0, self.above + theme.tick_pad())
            };
            shaped.place(list, at, notation(orient), 0.0);
        }
        if let Some(title) = labels.title.as_ref().and_then(|key| get(text, key)) {
            let raise = if across(orient) {
                labels.raise(text, theme)
            } else {
                0.0
            };
            let beyond = gap + extent + raise + theme.label_pad();
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
            title.place(list, spine.at(length / 2.0, beyond), anchor, angle);
        }
    }

    /// Where the axis sits beside plot area `area`.
    fn spine(&self, area: Rect) -> Spine {
        Spine {
            orient: self.labels.orient,
            area,
            offset: self.plan.offset,
        }
    }
}

/// An axis line: its side of a plot area and its offset outside it.
struct Spine {
    orient: Orient,
    area: Rect,
    offset: f64,
}

impl Spine {
    /// The plot area's length along the axis.
    fn length(&self) -> f64 {
        if across(self.orient) {
            self.area.width()
        } else {
            self.area.height()
        }
    }

    /// The point `beyond` px outward of the axis line, `along` px from the
    /// plot area's start (its left or top edge).
    fn at(&self, along: f64, beyond: f64) -> Point {
        let area = self.area;
        let (line, outward) = match self.orient {
            Orient::Top => (area.y0 - self.offset, -1.0),
            Orient::Bottom => (area.y1 + self.offset, 1.0),
            Orient::Left => (area.x0 - self.offset, -1.0),
            Orient::Right => (area.x1 + self.offset, 1.0),
        };
        if across(self.orient) {
            Point::new(area.x0 + along, line + outward * beyond)
        } else {
            Point::new(line + outward * beyond, area.y0 + along)
        }
    }
}
