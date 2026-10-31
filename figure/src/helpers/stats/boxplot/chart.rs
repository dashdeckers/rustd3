//! [`BoxPlot`]: box plots of groups along a key axis.

use std::f64::consts::TAU;

use kurbo::{BezPath, CircleSegment, Point, Rect, Shape};

use crate::check::Issue;
use crate::figure::Fits;
use crate::helpers::stats::boxplot::{Summary, Whis, summary};
use crate::helpers::stats::fill::Fill;
use crate::helpers::stats::slots::Slots;
use crate::list::{self, Command, List};
use crate::mark::{Ctx, Mark, guard};
use crate::scale::{Kind, Paint, Position, ScaleRef};

/// How closely marker shapes follow their curves, in px.
const TOLERANCE: f64 = 0.01;

/// The box's share of its slot's width.
const BOX: f64 = 0.5;

/// The caps' and the notch's waist's share of the box's width.
const WAIST: f64 = 0.5;

/// Box plots: per group, a box from the first to the third quartile with a
/// median line, whiskers with caps, and the fliers beyond them, in the
/// group's key slot (see [`boxplot`]).
pub struct BoxPlot<X: Position, Y: Position<Domain = f64>> {
    /// The keys and their finite values.
    groups: Vec<(<X as Kind>::Domain, Vec<f64>)>,
    /// Per group, its summary under `whis`.
    summaries: Vec<Option<Summary>>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    whis: Whis,
    notch: bool,
    means: bool,
    fill: Fill,
    dodge: bool,
}

/// One box plot per group `(key, values)` on `x` and `y`, from
/// [`boxplot::summary`](super::summary) with 1.5 × IQR whiskers. Each
/// group sits in its key's band (as bars sit, on scales without bands): a
/// box half the slot's width from `q1` to `q3`, filled in the theme
/// palette's first color with a 1 pt edge in ink, a median line, whiskers
/// to the most extreme data within their reach with caps half the box's
/// width, and the fliers beyond them as hollow dots, all lines in ink and
/// the patch edge's width (matplotlib's `boxplot` under seaborn's colors).
/// Whisker ends and fliers enter fitting; the box does not hold zero. A
/// group without finite values draws nothing.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::Figure;
/// use rustd3_figure::helpers::stats::{boxplot, boxplot::Whis};
///
/// let mut fig = Figure::new();
/// let day = fig.scale(scale::band(["mon", "tue"]).padding(0.2));
/// let y = fig.fit(scale::linear());
/// let groups = [
///     ("mon", vec![1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 14.0]),
///     ("tue", vec![2.0, 2.5, 3.5, 4.5, 5.0, 5.5, 6.0, 7.0]),
/// ];
/// fig.panel(0, 0).mark(boxplot(groups, day, y).notch(true).whis(Whis::Percentiles(5.0, 95.0)));
/// ```
pub fn boxplot<X, Y>(
    groups: impl IntoIterator<Item = (<X as Kind>::Domain, Vec<f64>)>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> BoxPlot<X, Y>
where
    X: Position,
    Y: Position<Domain = f64>,
{
    let groups: Vec<_> = groups
        .into_iter()
        .map(|(key, values)| (key, values.into_iter().filter(|v| v.is_finite()).collect()))
        .collect();
    BoxPlot {
        summaries: Vec::new(),
        groups,
        x,
        y,
        whis: Whis::default(),
        notch: false,
        means: false,
        fill: Fill::theme(),
        dodge: false,
    }
    .summarized()
}

impl<X: Position, Y: Position<Domain = f64>> BoxPlot<X, Y> {
    /// Notched boxes: narrowing to half their width at the median, the
    /// notch from `notch_low` to `notch_high` (matplotlib's shape); the
    /// median line spans the waist.
    pub fn notch(self, yes: bool) -> Self {
        Self { notch: yes, ..self }
    }

    /// How far the whiskers reach; [`Whis::Iqr(1.5)`](Whis::Iqr) by
    /// default.
    pub fn whis(self, whis: Whis) -> Self {
        Self { whis, ..self }.summarized()
    }

    /// Fills the boxes with `palette`'s color for `key`, looked up when
    /// the figure draws; a key the palette lacks draws the theme's gray.
    pub fn fill<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self {
        Self {
            fill: Fill::keyed(palette, key),
            ..self
        }
    }

    /// Places each group in its fill key's part of the slot, the slot split
    /// into one part per palette key in the palette's order, so that box
    /// plots of several marks stand side by side.
    pub fn dodge(self) -> Self {
        Self {
            dodge: true,
            ..self
        }
    }

    /// Marks each group's mean with a triangle in ink (matplotlib's
    /// `showmeans`).
    pub fn means(self, yes: bool) -> Self {
        Self { means: yes, ..self }
    }

    /// The same with each group's summary under its `whis`.
    fn summarized(self) -> Self {
        let summaries = self
            .groups
            .iter()
            .map(|(_, values)| summary(values, self.whis))
            .collect();
        Self { summaries, ..self }
    }
}

impl<'a, X: Position, Y: Position<Domain = f64>> Mark<'a> for BoxPlot<X, Y> {
    fn fit(&self, fits: &mut Fits) {
        for ((key, _), s) in self.groups.iter().zip(&self.summaries) {
            fits.add(self.x, key);
            let Some(s) = s else { continue };
            for v in [s.whisker_low, s.whisker_high].iter().chain(&s.fliers) {
                fits.add(self.y, v);
            }
        }
    }

    /// One `Fill` command of every box, one `Stroke` of their edges,
    /// medians, whiskers and caps, and `Markers` for the fliers (and the
    /// means): boxes and lines clipped to the guard band and, on raster
    /// targets, on device pixels; groups whose quartiles or whisker ends
    /// cannot be placed are skipped.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let (Some(sx), Some(sy)) = (ctx.x(self.x), ctx.y(self.y)) else {
            return;
        };
        let w = ctx.rect().width();
        let dodge = self.fill.slot(ctx).filter(|_| self.dodge);
        let slots = Slots::new(&sx, self.groups.iter().map(|(k, _)| k), w, dodge);
        let mut shapes = Shapes::new(ctx, self.notch);
        for ((key, _), s) in self.groups.iter().zip(&self.summaries) {
            let (Some(s), Some((start, width))) = (s, slots.of(&sx, key)) else {
                continue;
            };
            let at = |v: f64| sy.position(&v).filter(|p| p.is_finite());
            let ends = [s.q1, s.median, s.q3, s.whisker_low, s.whisker_high].map(at);
            let [Some(q1), Some(median), Some(q3), Some(low), Some(high)] = ends else {
                continue;
            };
            let centre = start + width / 2.0;
            shapes.group(centre, BOX * width / 2.0, [q1, median, q3], [low, high]);
            if self.notch {
                let notch = [s.notch_low, s.notch_high].map(at);
                if let [Some(n0), Some(n1)] = notch {
                    shapes.notch(centre, BOX * width / 2.0, [q1, median, q3], [n0, n1]);
                }
            } else {
                shapes.plain(centre, BOX * width / 2.0, [q1, median, q3]);
            }
            shapes.fliers(centre, s.fliers.iter().filter_map(|v| at(*v)));
            if self.means {
                shapes.mean(centre, at(s.mean));
            }
        }
        shapes.finish(&self.fill, list);
    }

    /// A fill key the palette lacks; keys outside a band or point domain.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        self.fill.issues(ctx, issues);
        let Some(sx) = ctx.raw(self.x) else {
            return;
        };
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

/// The box plots of one mark as they are drawn: the boxes' fill, the lines
/// in ink, and the markers.
struct Shapes<'c> {
    ctx: &'c Ctx<'c>,
    /// The guard band of the plot area.
    band: Rect,
    /// The width of every line: the theme's patch edge.
    edge: f64,
    notch: bool,
    boxes: BezPath,
    lines: BezPath,
    fliers: Vec<Point>,
    means: Vec<Point>,
}

impl<'c> Shapes<'c> {
    /// Empty shapes for the panel `ctx` draws.
    fn new(ctx: &'c Ctx<'c>, notch: bool) -> Shapes<'c> {
        Shapes {
            ctx,
            band: guard::band(ctx.rect().width(), ctx.rect().height()),
            edge: ctx.theme().get_patch_width(),
            notch,
            boxes: BezPath::new(),
            lines: BezPath::new(),
            fliers: Vec::new(),
            means: Vec::new(),
        }
    }

    /// `x` and `y` clamped to the guard band: exact for the axis-aligned
    /// lines and boxes it places.
    fn clamp(&self, x: f64, y: f64) -> Point {
        Point::new(
            x.clamp(self.band.x0, self.band.x1),
            y.clamp(self.band.y0, self.band.y1),
        )
    }

    /// The whiskers and caps of a group centred at `centre`, its box `half`
    /// px to each side, from its quartiles `[q1, _, q3]` to its whisker
    /// ends `[low, high]` (px).
    fn group(&mut self, centre: f64, half: f64, [q1, _, q3]: [f64; 3], [low, high]: [f64; 2]) {
        let snap = self.ctx.snap();
        let x = snap.line(centre, self.edge);
        let cap = WAIST * half;
        for (from, to) in [(q1, low), (q3, high)] {
            self.segment(self.clamp(x, from), self.clamp(x, to));
            let y = snap.line(to, self.edge);
            self.segment(self.clamp(centre - cap, y), self.clamp(centre + cap, y));
        }
    }

    /// A plain box `half` px to each side of `centre` from `q1` to `q3`,
    /// with its median line (px): the fill's edges on device pixel
    /// boundaries and the edge's lines on whole device pixels, as
    /// rectangles are drawn.
    fn plain(&mut self, centre: f64, half: f64, [q1, median, q3]: [f64; 3]) {
        let snap = self.ctx.snap();
        let r = Rect::new(centre - half, q3, centre + half, q1).abs();
        let filled = snap.rect(r.intersect(self.band));
        if filled.width() > 0.0 && filled.height() > 0.0 {
            self.boxes.extend(filled.path_elements(0.1));
        }
        let r = r.intersect(self.band);
        let at = |v: f64| snap.line(v, self.edge);
        let edged = Rect::new(at(r.x0), at(r.y0), at(r.x1), at(r.y1));
        self.lines.extend(edged.path_elements(0.1));
        let y = at(median);
        self.segment(self.clamp(edged.x0, y), self.clamp(edged.x1, y));
    }

    /// A notched box `half` px to each side of `centre` from `q1` to `q3`,
    /// narrowing to half its width at the median between the notch's ends
    /// `[n0, n1]` (px), clipped to the guard band; its median line spans
    /// the waist.
    fn notch(&mut self, centre: f64, half: f64, [q1, median, q3]: [f64; 3], [n0, n1]: [f64; 2]) {
        let (x0, x1) = (centre - half, centre + half);
        let waist = WAIST * half;
        let outline = [
            (x0, q1),
            (x1, q1),
            (x1, n0),
            (centre + waist, median),
            (x1, n1),
            (x1, q3),
            (x0, q3),
            (x0, n1),
            (centre - waist, median),
            (x0, n0),
        ]
        .map(Point::from);
        let clipped = guard::polygon(&outline, self.band);
        if let Some((&first, rest)) = clipped.split_first() {
            let mut path = BezPath::new();
            path.move_to(first);
            for &p in rest {
                path.line_to(p);
            }
            path.close_path();
            self.boxes.extend(path.iter());
            self.lines.extend(path.iter());
        }
        let y = self.ctx.snap().line(median, self.edge);
        self.segment(self.clamp(centre - waist, y), self.clamp(centre + waist, y));
    }

    /// Fliers at `ys` (px) on the line `x = centre`, those within a
    /// marker's radius of the plot area.
    fn fliers(&mut self, centre: f64, ys: impl Iterator<Item = f64>) {
        let r = self.ctx.theme().get_marker() / 2.0;
        let h = self.ctx.rect().height();
        let shown = |y: &f64| (-r..=h + r).contains(y);
        self.fliers
            .extend(ys.filter(shown).map(|y| Point::new(centre, y)));
    }

    /// The mean at `y` (px), when placed, on the line `x = centre`.
    fn mean(&mut self, centre: f64, y: Option<f64>) {
        let r = self.ctx.theme().get_marker() / 2.0;
        let h = self.ctx.rect().height();
        if let Some(y) = y.filter(|y| (-r..=h + r).contains(y)) {
            self.means.push(Point::new(centre, y));
        }
    }

    /// A straight line from `a` to `b`.
    fn segment(&mut self, a: Point, b: Point) {
        self.lines.move_to(a);
        self.lines.line_to(b);
    }

    /// Pushes the boxes filled with `fill`, then the lines, the fliers and
    /// the means in ink.
    fn finish(self, fill: &Fill, list: &mut List) {
        let theme = self.ctx.theme();
        let ink = list::brush(theme.get_ink());
        if !self.boxes.elements().is_empty() {
            list.commands.push(Command::Fill {
                path: self.boxes,
                paint: list::brush(fill.resolve(self.ctx)),
            });
        }
        if !self.lines.elements().is_empty() {
            let width = self.ctx.snap().width(self.edge);
            let style = if self.notch {
                list::stroke(width).with_join(kurbo::Join::Round)
            } else {
                list::stroke(width)
            };
            list.commands.push(Command::Stroke {
                path: self.lines,
                style,
                paint: ink.clone(),
            });
        }
        let r = theme.get_marker() / 2.0;
        if !self.fliers.is_empty() {
            let ring = CircleSegment::new(Point::ORIGIN, r, r - self.edge, 0.0, TAU);
            list.commands.push(Command::Markers {
                shape: ring.to_path(TOLERANCE),
                at: self.fliers,
                paint: ink.clone(),
                sizes: None,
                paints: None,
            });
        }
        if !self.means.is_empty() {
            let mut triangle = BezPath::new();
            triangle.move_to((0.0, -r));
            triangle.line_to((r * 0.866, r / 2.0));
            triangle.line_to((-r * 0.866, r / 2.0));
            triangle.close_path();
            list.commands.push(Command::Markers {
                shape: triangle,
                at: self.means,
                paint: ink,
                sizes: None,
                paints: None,
            });
        }
    }
}
