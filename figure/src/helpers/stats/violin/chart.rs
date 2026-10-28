//! [`Violin`]: kernel density estimates of groups mirrored about their
//! key slots.

use kurbo::{BezPath, Cap, Point};

use crate::check::Issue;
use crate::figure::Fits;
use crate::helpers::stats::boxplot::{Summary, Whis, summary};
use crate::helpers::stats::fill::Fill;
use crate::helpers::stats::slots::Slots;
use crate::helpers::stats::violin::Inner;
use crate::list::{self, Command, List};
use crate::mark::{Ctx, Mark, guard};
use crate::scale::{Kind, Paint, Position, ScaleRef};
use crate::stats::Kde;

/// Points of each body's curve by default: matplotlib's `violinplot`.
const POINTS: usize = 100;

/// The body's fill opacity, as densities draw it.
const FILL: f64 = 0.35;

/// seaborn's violin line width, in patch edge widths (1.25 ×
/// `patch.linewidth`): the quartile lines' width.
const LINE: f64 = 1.25;

/// The inner box's width, in violin line widths (seaborn's `box_width`).
const BOX: f64 = 4.5;

/// The inner whisker's width, in inner box widths (seaborn's
/// `whis_width`).
const WHISKER: f64 = 1.0 / 3.0;

/// The median dash's length and thickness, in inner box widths (seaborn's
/// marker size and edge width).
const DASH: (f64, f64) = (1.0 / 1.2, 1.0 / 5.0);

/// The quartile lines' dashes, in violin line widths: q1, median, q3.
const QUARTILES: [[f64; 2]; 3] = [[1.25, 0.75], [2.5, 1.0], [1.25, 0.75]];

/// Violins: per group, its kernel density estimate mirrored about the
/// middle of its key slot (see [`violin`]).
pub struct Violin<X: Position, Y: Position<Domain = f64>> {
    /// The keys and their finite values.
    groups: Vec<(<X as Kind>::Domain, Vec<f64>)>,
    /// Per group, its body for `points` and `cut`.
    bodies: Vec<Body>,
    /// Per group, its box plot summary, for the inner box and quartiles.
    summaries: Vec<Option<Summary>>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    points: usize,
    cut: f64,
    inner: Inner,
    fill: Fill,
    dodge: bool,
}

/// What one group draws.
enum Body {
    /// No finite values: nothing.
    Empty,
    /// One value, or values that are all equal: a line across the slot.
    Flat(f64),
    /// The estimate: positions along the value axis, evenly spaced, and
    /// the density at each.
    Curve(Vec<(f64, f64)>),
}

/// One violin per group `(key, values)` on `x` and `y`: the Gaussian
/// kernel density estimate of the finite values with Scott's bandwidth
/// ([`stats::Kde`](crate::stats::Kde)), evaluated at 100 points evenly
/// spaced over `[min − cut·h, max + cut·h]` (`cut` 0: the data's range,
/// matplotlib's `violinplot`), mirrored about the middle of the group's
/// key slot (as box plots sit), filled at 0.35 opacity and outlined 0.75
/// pt wide as densities are. Widths are shared within the mark: the
/// largest density among its groups spans the slot's width, so areas
/// compare between groups. Inside, a miniature box plot in ink
/// ([`Inner::Box`]). A group of one value, or of values that are all
/// equal, draws a line across the slot at the value; a group without
/// finite values draws nothing. The bodies' ends enter fitting.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::Figure;
/// use rustd3_figure::helpers::stats::{violin, violin::Inner};
///
/// let mut fig = Figure::new();
/// let day = fig.scale(scale::band(["mon", "tue"]).padding(0.1));
/// let y = fig.fit(scale::linear());
/// let groups = [
///     ("mon", vec![1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 4.7, 5.1]),
///     ("tue", vec![2.0, 2.5, 3.5, 4.5, 5.0, 5.5, 6.0, 7.0]),
/// ];
/// fig.panel(0, 0).mark(violin(groups, day, y).inner(Inner::Quartiles).cut(2.0));
/// ```
pub fn violin<X, Y>(
    groups: impl IntoIterator<Item = (<X as Kind>::Domain, Vec<f64>)>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> Violin<X, Y>
where
    X: Position,
    Y: Position<Domain = f64>,
{
    let groups: Vec<(_, Vec<f64>)> = groups
        .into_iter()
        .map(|(key, values)| (key, values.into_iter().filter(|v| v.is_finite()).collect()))
        .collect();
    let summaries = groups
        .iter()
        .map(|(_, values)| summary(values, Whis::default()))
        .collect();
    Violin {
        groups,
        bodies: Vec::new(),
        summaries,
        x,
        y,
        points: POINTS,
        cut: 0.0,
        inner: Inner::default(),
        fill: Fill::theme(),
        dodge: false,
    }
    .traced()
}

impl<X: Position, Y: Position<Domain = f64>> Violin<X, Y> {
    /// Points of each body's curve; 100 by default. A count below 2
    /// leaves it as it was.
    pub fn points(self, n: usize) -> Self {
        let points = if n >= 2 { n } else { self.points };
        Self { points, ..self }.traced()
    }

    /// How far past the data the bodies run, in bandwidths; 0 by default
    /// (matplotlib), seaborn's default is 2. A value that is not finite
    /// and not negative leaves it as it was.
    pub fn cut(self, k: f64) -> Self {
        let cut = if k.is_finite() && k >= 0.0 {
            k
        } else {
            self.cut
        };
        Self { cut, ..self }.traced()
    }

    /// What is drawn inside each body; [`Inner::Box`] by default.
    pub fn inner(self, inner: Inner) -> Self {
        Self { inner, ..self }
    }

    /// Fills the bodies with `palette`'s color for `key` (at 0.35 opacity,
    /// the outline opaque), looked up when the figure draws; a key the
    /// palette lacks draws the theme's gray.
    pub fn fill<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self {
        Self {
            fill: Fill::keyed(palette, key),
            ..self
        }
    }

    /// Places each group in its fill key's part of the slot, the slot split
    /// into one part per palette key in the palette's order, so that
    /// violins of several marks stand side by side.
    pub fn dodge(self) -> Self {
        Self {
            dodge: true,
            ..self
        }
    }

    /// The same with each group's body for its `points` and `cut`.
    fn traced(self) -> Self {
        let bodies = self
            .groups
            .iter()
            .map(
                |(_, values)| match (Kde::new(values.iter().copied()), values.first()) {
                    (Some(kde), _) => Body::Curve(kde.grid(self.points, self.cut)),
                    (None, Some(&v)) => Body::Flat(v),
                    (None, None) => Body::Empty,
                },
            )
            .collect();
        Self { bodies, ..self }
    }

    /// The largest density among the groups; 0 without a curve.
    fn peak(&self) -> f64 {
        self.bodies
            .iter()
            .flat_map(|body| match body {
                Body::Curve(curve) => curve.as_slice(),
                Body::Empty | Body::Flat(_) => &[],
            })
            .map(|(_, d)| *d)
            .fold(0.0, f64::max)
    }
}

impl<'a, X: Position, Y: Position<Domain = f64>> Mark<'a> for Violin<X, Y> {
    fn fit(&self, fits: &mut Fits) {
        for ((key, _), body) in self.groups.iter().zip(&self.bodies) {
            fits.add(self.x, key);
            match body {
                Body::Empty => {}
                Body::Flat(v) => fits.add(self.y, v),
                Body::Curve(curve) => {
                    for (v, _) in curve.first().into_iter().chain(curve.last()) {
                        fits.add(self.y, v);
                    }
                }
            }
        }
    }

    /// One `Fill` command of every body and a `Stroke` of their outlines
    /// (and of the lines of flat groups), clipped to the guard band; then
    /// the inner box plots' or quartiles' strokes. Groups whose key or
    /// values cannot be placed are skipped.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let (sx, sy) = (ctx.x(self.x), ctx.y(self.y));
        let w = ctx.rect().width();
        let band = guard::band(w, ctx.rect().height());
        let dodge = self.fill.slot(ctx).filter(|_| self.dodge);
        let slots = Slots::new(&sx, self.groups.iter().map(|(k, _)| k), w, dodge);
        let peak = self.peak();
        let at = |v: f64| sy.position(&v).filter(|p| p.is_finite());
        let mut bodies = BezPath::new();
        let mut outlines = BezPath::new();
        let mut inner = Inside::new(ctx, band);
        for (((key, _), body), s) in self.groups.iter().zip(&self.bodies).zip(&self.summaries) {
            let Some((start, width)) = slots.of(&sx, key) else {
                continue;
            };
            let (centre, half) = (start + width / 2.0, width / 2.0);
            match body {
                Body::Empty => {}
                Body::Flat(v) => {
                    if let Some(y) = at(*v) {
                        let y = y.clamp(band.y0, band.y1);
                        outlines.move_to((centre - half, y));
                        outlines.line_to((centre + half, y));
                    }
                }
                Body::Curve(curve) => {
                    let Some(placed) = curve
                        .iter()
                        .map(|&(v, d)| Some((at(v)?, half * d / peak)))
                        .collect::<Option<Vec<(f64, f64)>>>()
                    else {
                        continue;
                    };
                    let right = placed.iter().map(|&(y, r)| Point::new(centre + r, y));
                    let left = placed.iter().rev().map(|&(y, r)| Point::new(centre - r, y));
                    let outline: Vec<Point> = right.chain(left).collect();
                    let clipped = guard::polygon(&outline, band);
                    if let Some((&first, rest)) = clipped.split_first() {
                        bodies.move_to(first);
                        for &p in rest {
                            bodies.line_to(p);
                        }
                        bodies.close_path();
                    }
                    if let Some(s) = s {
                        let reach = |v: f64| half * density(curve, v) / peak;
                        match self.inner {
                            Inner::None => {}
                            Inner::Box => inner.boxed(centre, s, &at),
                            Inner::Quartiles => inner.quartiles(centre, s, &at, reach),
                        }
                    }
                }
            }
        }
        let color = self.fill.resolve(ctx);
        let edge = ctx.theme().density_width();
        if !bodies.elements().is_empty() {
            list.commands.push(Command::Fill {
                path: bodies.clone(),
                paint: list::brush(color.with_opacity(color.opacity * FILL)),
            });
            outlines.extend(bodies.iter());
        }
        if !outlines.elements().is_empty() {
            list.commands.push(Command::Stroke {
                path: outlines,
                style: list::stroke(edge).with_join(kurbo::Join::Round),
                paint: list::brush(color),
            });
        }
        inner.finish(list);
    }

    /// A fill key the palette lacks; keys outside a band or point domain.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        self.fill.issues(ctx, issues);
        let sx = ctx.raw(self.x);
        let outside = self
            .groups
            .iter()
            .filter(|(k, _)| sx.position(k).is_none())
            .count();
        if outside > 0 {
            issues.push(Issue::OutsideDomain {
                handle: self.x.index(),
                count: outside,
            });
        }
    }
}

/// The density of `curve`, evenly spaced, at `v`: linear between its
/// points, 0 outside them.
fn density(curve: &[(f64, f64)], v: f64) -> f64 {
    let i = curve.partition_point(|(at, _)| *at < v);
    match (i.checked_sub(1).and_then(|j| curve.get(j)), curve.get(i)) {
        (Some(&(v0, d0)), Some(&(v1, d1))) => d0 + (d1 - d0) * (v - v0) / (v1 - v0),
        (None, Some(&(v1, d1))) if v1 == v => d1,
        (None | Some(_), None) | (None, Some(_)) => 0.0,
    }
}

/// What violins draw inside their bodies, in ink: the inner box plots'
/// whiskers, boxes and medians, or the quartile lines.
struct Inside<'c> {
    ctx: &'c Ctx<'c>,
    band: kurbo::Rect,
    whiskers: BezPath,
    boxes: BezPath,
    medians: BezPath,
    quartiles: [BezPath; 3],
}

impl<'c> Inside<'c> {
    /// Nothing yet, for the panel `ctx` draws, clipped to `band`.
    fn new(ctx: &'c Ctx<'c>, band: kurbo::Rect) -> Inside<'c> {
        Inside {
            ctx,
            band,
            whiskers: BezPath::new(),
            boxes: BezPath::new(),
            medians: BezPath::new(),
            quartiles: [BezPath::new(), BezPath::new(), BezPath::new()],
        }
    }

    /// The inner box width, px.
    fn box_width(&self) -> f64 {
        BOX * LINE * self.ctx.theme().patch_width()
    }

    /// A vertical line at `x` from `a` to `b` (px), clamped to the band.
    fn vertical(path: &mut BezPath, band: kurbo::Rect, x: f64, [a, b]: [f64; 2]) {
        path.move_to((x, a.clamp(band.y0, band.y1)));
        path.line_to((x, b.clamp(band.y0, band.y1)));
    }

    /// A horizontal line at `y` from `x0` to `x1` (px), when `y` lies in
    /// the band.
    fn horizontal(path: &mut BezPath, band: kurbo::Rect, y: f64, [x0, x1]: [f64; 2]) {
        if (band.y0..=band.y1).contains(&y) {
            path.move_to((x0, y));
            path.line_to((x1, y));
        }
    }

    /// The miniature box plot of `s` on the line `x = centre`, placed by
    /// `at`.
    fn boxed(&mut self, centre: f64, s: &Summary, at: &impl Fn(f64) -> Option<f64>) {
        let [Some(low), Some(q1), Some(median), Some(q3), Some(high)] =
            [s.whisker_low, s.q1, s.median, s.q3, s.whisker_high].map(at)
        else {
            return;
        };
        Self::vertical(&mut self.whiskers, self.band, centre, [low, high]);
        Self::vertical(&mut self.boxes, self.band, centre, [q1, q3]);
        let half = DASH.0 * self.box_width() / 2.0;
        Self::horizontal(
            &mut self.medians,
            self.band,
            median,
            [centre - half, centre + half],
        );
    }

    /// Lines at the quartiles of `s` across the body, `reach(v)` px to each
    /// side of `x = centre`, placed by `at`.
    fn quartiles(
        &mut self,
        centre: f64,
        s: &Summary,
        at: &impl Fn(f64) -> Option<f64>,
        reach: impl Fn(f64) -> f64,
    ) {
        for (path, v) in self.quartiles.iter_mut().zip([s.q1, s.median, s.q3]) {
            if let Some(y) = at(v) {
                let r = reach(v);
                Self::horizontal(path, self.band, y, [centre - r, centre + r]);
            }
        }
    }

    /// Pushes the strokes: whiskers, boxes and medians, or the quartile
    /// lines with their dashes.
    fn finish(self, list: &mut List) {
        let theme = self.ctx.theme();
        let (ink, line) = (list::brush(theme.get_ink()), LINE * theme.patch_width());
        let box_width = self.box_width();
        let strokes = [
            (self.whiskers, WHISKER * box_width, ink.clone()),
            (self.boxes, box_width, ink.clone()),
            (
                self.medians,
                DASH.1 * box_width,
                list::brush(theme.get_background()),
            ),
        ];
        for (path, width, paint) in strokes {
            if !path.elements().is_empty() {
                list.commands.push(Command::Stroke {
                    path,
                    style: list::stroke(width),
                    paint,
                });
            }
        }
        for (path, dash) in self.quartiles.into_iter().zip(QUARTILES) {
            if !path.elements().is_empty() {
                let style = list::stroke(line)
                    .with_caps(Cap::Butt)
                    .with_dashes(0.0, dash.map(|d| d * line));
                list.commands.push(Command::Stroke {
                    path,
                    style,
                    paint: ink.clone(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn densities_between_grid_points_are_linear() {
        let curve = [(0.0, 1.0), (1.0, 3.0), (2.0, 2.0)];
        assert_eq!(density(&curve, 0.5), 2.0);
        assert_eq!(density(&curve, 1.0), 3.0);
        assert_eq!(density(&curve, 0.0), 1.0);
        assert_eq!(density(&curve, 2.0), 2.0);
        assert_eq!(density(&curve, -1.0), 0.0);
        assert_eq!(density(&curve, 3.0), 0.0);
    }
}
