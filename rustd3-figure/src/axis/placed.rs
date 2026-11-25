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
use crate::text::anchor::{Horizontal, Vertical};
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
    /// Its minor ticks' positions along the plot area (`Axis::minor`).
    pub(crate) minors: Vec<f64>,
    /// The height above the plot area that lies a tick pad below a vertical
    /// axis's offset or multiplier label: the panel's top reach without
    /// those labels.
    pub(crate) above: f64,
}

impl Placed {
    /// The axis with `labels` and `plan` beside plot area `area`, its ticks
    /// at `positions`, with every tick label kept. Each label's box is its
    /// box ([`Labels::oriented`]) at the end of its tick plus the tick pad,
    /// or `None` for a label that is not placed, empty, or rejected by
    /// typst.
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
            minors: Vec::new(),
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
    /// theme's edge color (its own when tinted). Square caps make two axes
    /// meet at a closed corner, as matplotlib's projecting spines do. On
    /// raster targets (`snap`) the line and its square-capped ends lie on
    /// whole device pixels. Styles without axis lines draw nothing.
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

    /// Appends the axis's tick marks beside plot area `area` to `list`.
    /// Ticks whose label the scale's format blanks (d3's log ticks between
    /// labeled ones) and minor ticks take the minor size and width. On
    /// raster targets (`snap`) the marks lie on whole device pixels.
    pub(crate) fn marks(&self, area: Rect, theme: &Theme, snap: Snap, list: &mut List) {
        self.ticks(&self.spine(area, theme), theme, snap, list);
    }

    /// Appends the axis's kept tick labels, its group labels, its offset or
    /// multiplier label and its title beside plot area `area` to `list`.
    ///
    /// Tick labels sit a tick pad past the ticks, on the baselines their
    /// line box gives ([`Labels::baseline`]). Rotated labels are turned
    /// about their first baseline's left end and placed by
    /// [`Labels::oriented`]. The offset or multiplier label sits at the
    /// right end of a horizontal axis, a tick pad beyond the labels, or
    /// above the top end of a vertical one. The title sits a label pad past
    /// all of them. Beside a vertical axis it is turned a quarter
    /// counter-clockwise so that it reads bottom to top on either side, as
    /// matplotlib draws every vertical label (its descenders toward a left
    /// axis, its ascenders toward a right one). Labels keep their positions
    /// on raster targets.
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
        self.groups(&spine, gap + extent, text, theme, list);
        let extent = extent + labels.grouped(text, theme);
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
            let angle = if across(orient) { 0.0 } else { -FRAC_PI_2 };
            let v = match orient {
                Orient::Bottom | Orient::Right => text::anchor::Vertical::Top,
                Orient::Top | Orient::Left => text::anchor::Vertical::Bottom,
            };
            let anchor = Anchor {
                h: text::anchor::Horizontal::Middle,
                v,
            };
            title.place(list, spine.at(length / 2.0, beyond), anchor, angle);
        }
    }

    /// Appends the group labels and their lines to `list`, `beyond` px
    /// past the axis line along `spine` (the tick labels' outer edge).
    ///
    /// Each group's line lies half a tick pad further and spans its ticks,
    /// reaching half the closest tick spacing less 3 px past the first and
    /// the last, so neighboring groups' lines stay apart. Its label sits a
    /// tick pad past `beyond`, centered on the line: on the tick labels'
    /// shared baseline under or over a horizontal axis, and beside a
    /// vertical one aligned as its tick labels are. A label that would
    /// come within 2 px of the one drawn before it is left out, with its
    /// line kept.
    fn groups(
        &self,
        spine: &Spine,
        beyond: f64,
        text: &text::Cache,
        theme: &Theme,
        list: &mut List,
    ) {
        let labels = &self.labels;
        if labels.groups.is_empty() {
            return;
        }
        let orient = labels.orient;
        let placed: Vec<f64> = self.positions.iter().flatten().copied().collect();
        let spacing = placed
            .windows(2)
            .map(|pair| (pair[1] - pair[0]).abs())
            .filter(|d| *d > 0.0)
            .fold(f64::INFINITY, f64::min);
        let reach = if spacing.is_finite() {
            (spacing / 2.0 - 3.0).max(0.0)
        } else {
            0.0
        };
        let [ascender, descender] = labels.line;
        let row = beyond + theme.tick_pad();
        let (across_axis, anchor) = match orient {
            Orient::Bottom => (
                row + ascender,
                anchor_of(Horizontal::Middle, Vertical::Baseline),
            ),
            Orient::Top => (
                row + descender,
                anchor_of(Horizontal::Middle, Vertical::Baseline),
            ),
            Orient::Left => (row, anchor_of(Horizontal::End, Vertical::Middle)),
            Orient::Right => (row, anchor_of(Horizontal::Start, Vertical::Middle)),
        };
        let mut lines = BezPath::new();
        let mut taken = f64::NEG_INFINITY;
        for group in &labels.groups {
            let run = self.positions.get(group.first..=group.last);
            let within: Vec<f64> = run.into_iter().flatten().flatten().copied().collect();
            let low = within.iter().copied().fold(f64::INFINITY, f64::min);
            let high = within.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            if within.is_empty() {
                continue;
            }
            if high - low + 2.0 * reach > 0.0 {
                let at = beyond + theme.tick_pad() / 2.0;
                lines.move_to(spine.at(low - reach, at));
                lines.line_to(spine.at(high + reach, at));
            }
            let Some(shaped) = get(text, &group.key) else {
                continue;
            };
            let center = (low + high) / 2.0;
            let half = if across(orient) {
                shaped.get_width() / 2.0
            } else {
                (shaped.get_ascent() + shaped.get_descent()) / 2.0
            };
            if center - half < taken + 2.0 {
                continue;
            }
            taken = center + half;
            shaped.place(list, spine.at(center, across_axis), anchor, 0.0);
        }
        if !lines.elements().is_empty() {
            list.commands.push(Command::Stroke {
                path: lines,
                style: list::stroke(theme.minor_width()),
                paint: list::brush(labels.ink),
            });
        }
    }

    /// Adds to `path` a grid line at each of the axis's major ticks (those
    /// whose label the scale's format does not blank). Each line crosses
    /// plot area `area` and runs `overscan` px past its ends, vertical for a
    /// horizontal axis and horizontal for a vertical one. On raster targets
    /// (`snap`) the lines are hairlines `width` px wide on whole device
    /// pixels.
    pub(crate) fn grid(
        &self,
        area: Rect,
        overscan: f64,
        (width, snap): (f64, Snap),
        path: &mut BezPath,
    ) {
        let horizontal = across(self.labels.orient);
        let reach = area.inflate(overscan, overscan);
        for (i, p) in self.positions.iter().enumerate() {
            let Some(p) = p else { continue };
            if self.labels.minor.get(i).copied().unwrap_or(false) {
                continue;
            }
            if horizontal {
                let x = snap.line(area.x0 + p, width);
                path.move_to((x, reach.y0));
                path.line_to((x, reach.y1));
            } else {
                let y = snap.line(area.y0 + p, width);
                path.move_to((reach.x0, y));
                path.line_to((reach.x1, y));
            }
        }
    }

    /// Whether the axis runs along the panel's width (top or bottom).
    pub(crate) fn horizontal(&self) -> bool {
        across(self.labels.orient)
    }

    /// Appends the tick marks along `spine` to `list`. Major ticks take the
    /// theme's tick size and width; ticks whose label the scale's format
    /// blanks and minor ticks take the minor size and width. On raster
    /// targets (`snap`) they lie on whole device pixels.
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
        let (length, w) = (theme.minor_tick(), theme.minor_width());
        for p in &self.minors {
            minor.move_to(spine.snapped(*p, 0.0, w, snap));
            minor.line_to(spine.snapped(*p, length, w, snap));
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

/// The anchor of halves `h` and `v`.
fn anchor_of(h: Horizontal, v: Vertical) -> Anchor {
    Anchor { h, v }
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

    /// [`at`](Spine::at), snapped. The axis line goes where `snap` lays a
    /// line of its width, and `along` goes where `snap` lays a hairline `w`
    /// px wide across it. The axis line, its ends and its ticks then cover
    /// whole device pixels on raster targets.
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
