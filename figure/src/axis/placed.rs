//! [`Placed`]: an axis as layout placed it, the boxes of its tick labels,
//! and drawing it.

use std::f64::consts::FRAC_PI_2;

use kurbo::{BezPath, Cap, Point, Rect};
use rustd3_core::axis::Orient;

use crate::axis::labels::{align, get, notation};
use crate::axis::{Labels, across};
use crate::layout::{Obb, Plan};
use crate::list::{self, Command, List};
use crate::mark::Snap;
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
    /// Its tick labels' boxes in list px, turned with rotated labels;
    /// `None` for one not placed.
    pub(crate) boxes: Vec<Option<Obb>>,
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
    /// ([`Labels::oriented`]) at the end of its tick plus the tick pad,
    /// `None` for a label that is not placed, empty, or that typst
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
    fn measure(&self, area: Rect, text: &text::Cache, theme: &Theme) -> Vec<Option<Obb>> {
        let spine = self.spine(area, theme);
        let gap = theme.tick() + theme.tick_pad();
        self.labels
            .ticks
            .iter()
            .zip(&self.positions)
            .map(|(key, p)| {
                let shaped = get(text, key.as_ref()?)?;
                let at = spine.at((*p)?, gap);
                Some(self.labels.oriented(shaped) + at.to_vec2())
            })
            .collect()
    }

    /// Appends the axis line beside plot area `area` to `list`, in the
    /// theme's edge color (its own when tinted) with square caps, so two
    /// axes meet at a closed corner as matplotlib's projecting spines do;
    /// on raster targets (`snap`) the line and its square-capped ends lie
    /// on whole device pixels. Nothing in the styles without axis lines.
    pub(crate) fn line(&self, area: Rect, theme: &Theme, snap: Snap, list: &mut List) {
        let labels = &self.labels;
        let Some(edge) = theme.edge() else {
            return;
        };
        let edge = if labels.tinted { labels.ink } else { edge };
        let spine = self.spine(area, theme);
        let length = spine.length();
        let width = theme.axis_width();
        let mut path = BezPath::new();
        path.move_to(spine.snapped(0.0, 0.0, width, snap));
        path.line_to(spine.snapped(length, 0.0, width, snap));
        list.commands.push(Command::Stroke {
            path,
            style: list::stroke(snap.width(width)).with_caps(Cap::Square),
            paint: list::brush(edge),
        });
    }

    /// Appends the rest of the axis beside plot area `area` to `list`: its
    /// tick marks in the styles that draw them ([`marks`](Placed::marks)),
    /// then its labels and title ([`texts`](Placed::texts)).
    pub(crate) fn draw(
        &self,
        area: Rect,
        text: &text::Cache,
        theme: &Theme,
        snap: Snap,
        list: &mut List,
    ) {
        if theme.ticks() {
            self.marks(area, theme, snap, list);
        }
        self.texts(area, text, theme, list);
    }

    /// Appends the axis's tick marks beside plot area `area` to `list`:
    /// those whose label the scale's format blanks at the minor size and
    /// width (d3's log ticks between labelled ones), on whole device pixels
    /// on raster targets (`snap`).
    pub(crate) fn marks(&self, area: Rect, theme: &Theme, snap: Snap, list: &mut List) {
        self.ticks(&self.spine(area, theme), theme, snap, list);
    }

    /// Appends the axis's kept tick labels beside plot area `area` to
    /// `list`, a tick pad past the ticks on the baselines their line box
    /// gives ([`Labels::baseline`]), its offset or multiplier label (at the
    /// right end of a horizontal axis, a tick pad beyond the labels; above
    /// the top end of a vertical one); rotated labels are turned about their
    /// first baseline's left end, placed by [`Labels::oriented`]; and its
    /// title a label pad past all of them, turned a quarter
    /// counter-clockwise on the left and clockwise on the right. Labels
    /// keep their positions on raster targets.
    pub(crate) fn texts(&self, area: Rect, text: &text::Cache, theme: &Theme, list: &mut List) {
        let labels = &self.labels;
        let orient = labels.orient;
        let spine = self.spine(area, theme);
        let length = spine.length();
        let gap = theme.tick() + theme.tick_pad();
        let baseline = Anchor {
            h: align(orient),
            v: text::anchor::Vertical::Baseline,
        };
        let kept = labels.ticks.iter().zip(&self.positions).zip(&self.keep);
        let turned = Anchor {
            h: text::anchor::Horizontal::Start,
            v: text::anchor::Vertical::Baseline,
        };
        for ((key, p), _) in kept.filter(|(_, keep)| **keep) {
            if let (Some(shaped), Some(p)) = (key.as_ref().and_then(|k| get(text, k)), p) {
                if labels.angle == 0.0 {
                    let at = spine.at(*p, gap) + kurbo::Vec2::new(0.0, labels.baseline(shaped));
                    shaped.place(list, at, baseline, 0.0);
                } else {
                    let at = spine.at(*p, gap) + labels.origin(shaped);
                    shaped.place(list, at, turned, -labels.angle);
                }
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

    /// Adds to `path` a grid line across plot area `area` at each of the
    /// axis's ticks whose label the scale's format does not blank (major
    /// ticks), as hairlines `width` px wide on whole device pixels on
    /// raster targets (`snap`): vertical lines for a horizontal axis,
    /// horizontal ones for a vertical axis.
    pub(crate) fn grid(&self, area: Rect, width: f64, snap: Snap, path: &mut BezPath) {
        let horizontal = across(self.labels.orient);
        for (i, p) in self.positions.iter().enumerate() {
            let Some(p) = p else { continue };
            if self.labels.minor.get(i).copied().unwrap_or(false) {
                continue;
            }
            if horizontal {
                let x = snap.line(area.x0 + p, width);
                path.move_to((x, area.y0));
                path.line_to((x, area.y1));
            } else {
                let y = snap.line(area.y0 + p, width);
                path.move_to((area.x0, y));
                path.line_to((area.x1, y));
            }
        }
    }

    /// Whether the axis runs along the panel's width (top or bottom).
    pub(crate) fn horizontal(&self) -> bool {
        across(self.labels.orient)
    }

    /// Appends the tick marks along `spine` to `list`: major ticks at the
    /// theme's tick size and width, those whose label the scale's format
    /// blanks at the minor size and width, on whole device pixels on raster
    /// targets (`snap`).
    fn ticks(&self, spine: &Spine, theme: &Theme, snap: Snap, list: &mut List) {
        let labels = &self.labels;
        let width = theme.tick_width();
        let mut ticks = BezPath::new();
        let mut minor = BezPath::new();
        for (i, p) in self.positions.iter().enumerate() {
            let Some(p) = p else { continue };
            let (path, length, w) = if labels.minor.get(i).copied().unwrap_or(false) {
                (&mut minor, theme.minor_tick(), theme.minor_width())
            } else {
                (&mut ticks, theme.tick(), width)
            };
            path.move_to(spine.snapped(*p, 0.0, w, snap));
            path.line_to(spine.snapped(*p, length, w, snap));
        }
        list.commands.push(Command::Stroke {
            path: ticks,
            style: list::stroke(snap.width(width)),
            paint: list::brush(labels.ink),
        });
        if !minor.elements().is_empty() {
            list.commands.push(Command::Stroke {
                path: minor,
                style: list::stroke(snap.width(theme.minor_width())),
                paint: list::brush(labels.ink),
            });
        }
    }

    /// Where the axis sits beside plot area `area`, as wide as `theme`'s
    /// axis lines.
    fn spine(&self, area: Rect, theme: &Theme) -> Spine {
        Spine {
            orient: self.labels.orient,
            area,
            offset: self.plan.offset,
            width: theme.axis_width(),
        }
    }
}

/// An axis line: its side of a plot area, its offset outside it and its
/// width.
struct Spine {
    orient: Orient,
    area: Rect,
    offset: f64,
    width: f64,
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

    /// [`at`](Spine::at) with the axis line where `snap` lays a line of its
    /// width and `along` where it lays a hairline `w` px wide across it, so
    /// the axis line, its ends and its ticks cover whole device pixels on
    /// raster targets.
    fn snapped(&self, along: f64, beyond: f64, w: f64, snap: Snap) -> Point {
        let base = self.at(along, 0.0);
        let snapped = if across(self.orient) {
            Point::new(snap.line(base.x, w), snap.line(base.y, self.width))
        } else {
            Point::new(snap.line(base.x, self.width), snap.line(base.y, w))
        };
        snapped + (self.at(along, beyond) - base)
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
