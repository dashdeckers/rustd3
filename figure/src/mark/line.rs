//! [`Line`]: a polyline or curve through points.

use kurbo::Point;
use rustd3_core::color::Rgb;
use rustd3_core::series::{Data, Points, m4};
use rustd3_core::shape::{self, Curve};

use crate::check::Issue;
use crate::figure::Fits;
use crate::legend::Swatch;
use crate::list::{self, Command, List};
use crate::mark::color::{Alpha, Color};
use crate::mark::{Ctx, Mark, across, at, dashes, guard, positive, stroke, within};
use crate::scale::{Kind, Paint, Position, ScaleRef};
use crate::thin::{self, Thin};
use crate::{hit, path};

/// A line through points in their order: one `Stroke` command, broken
/// where a point cannot be placed. A recording channel on an x scale with
/// an index bridge is thinned to M4's points per column first.
pub struct Line<'a, X: Position, Y: Position> {
    points: Points<'a, <X as Kind>::Domain, <Y as Kind>::Domain>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    color: Color,
    width: Option<f64>,
    dash: Vec<f64>,
    curve: Curve,
    thin: Thin,
    dodge: bool,
}

impl<'a, X: Position, Y: Position> std::fmt::Debug for Line<'a, X, Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Line").finish_non_exhaustive()
    }
}

/// A line through `data` (pairs, a recording channel or two slices) on `x`
/// and `y`, straight from point to point, solid, in the theme palette's
/// first color and the theme's line width, thinned when `data` is a
/// recording channel ([`Thin::Auto`]).
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_core::shape::Curve;
/// use rustd3_figure::Figure;
/// use rustd3_figure::mark::line;
///
/// let mut fig = Figure::new();
/// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
/// fig.panel(0, 0).mark(
///     line([(0.0, 1.0), (1.0, 3.0), (2.0, 2.0)], x, y)
///         .curve(Curve::MonotoneX)
///         .dash([6.0, 3.0]),
/// );
/// ```
pub fn line<'a, X: Position, Y: Position>(
    data: impl Data<'a, <X as Kind>::Domain, <Y as Kind>::Domain>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> Line<'a, X, Y> {
    Line {
        points: data.into_points(),
        x,
        y,
        color: Color::Theme,
        width: None,
        dash: Vec::new(),
        curve: Curve::Linear,
        thin: Thin::Auto,
        dodge: false,
    }
}

impl<X: Position, Y: Position> Line<'_, X, Y> {
    /// Strokes the line with `palette`'s color for `key`, looked up when
    /// the figure draws; a key the palette lacks draws the theme's gray.
    #[must_use]
    pub fn stroke<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self {
        Self {
            color: Color::keyed(palette, key),
            ..self
        }
    }

    /// Strokes the line with `c`.
    #[must_use]
    pub fn color(self, c: Rgb) -> Self {
        Self {
            color: Color::Fixed(c),
            ..self
        }
    }

    /// The line's width in px; by default the theme's, 1.5 pt (2 px). A
    /// width that is not finite and positive leaves it as it was.
    #[must_use]
    pub fn width(self, px: f64) -> Self {
        Self {
            width: positive(px).or(self.width),
            ..self
        }
    }

    /// Dashes: on and off lengths in px, from offset 0 at the first point
    /// drawn. A thinned line on a column grid a widget holds
    /// ([`Held::grids`](crate::Held::grids)) instead starts the pattern at
    /// the first sample of the column containing the grid's origin. A
    /// pattern with a negative or non-finite length, or none longer than 0,
    /// leaves the line as it was.
    #[must_use]
    pub fn dash(self, pattern: impl IntoIterator<Item = f64>) -> Self {
        let dash = dashes(pattern).unwrap_or(self.dash);
        Self { dash, ..self }
    }

    /// How the line joins its points: d3's curves, straight by default.
    /// Thinned lines join their points straight.
    #[must_use]
    pub fn curve(self, c: Curve) -> Self {
        Self { curve: c, ..self }
    }

    /// Whether the line thins a recording channel: [`Thin::Auto`] by
    /// default, [`Thin::Off`] to draw every sample.
    #[must_use]
    pub fn thin(self, t: Thin) -> Self {
        Self { thin: t, ..self }
    }

    /// Vertices at the center of the stroke key's sub-band (band x only).
    /// The band is split into one sub-band per palette key, in the
    /// palette's order, so that a line runs through the dodged dots of its
    /// key.
    #[must_use]
    pub fn dodge(self) -> Self {
        Self {
            dodge: true,
            ..self
        }
    }
}

impl<'a, X: Position, Y: Position> Mark<'a> for Line<'a, X, Y> {
    fn fit(&self, fits: &mut Fits) {
        super::fit(&self.points, self.x, self.y, fits);
        let dash = match self.dash[..] {
            [] => None,
            [on] => Some([on, on]),
            [on, off, ..] => Some([on, off]),
        };
        self.color
            .report(Swatch::Line { dash }, None, Alpha::OPAQUE, fits);
    }

    /// One `Stroke` command through every point placed on both scales, or
    /// through the samples M4 keeps per column of a thinned line, joined
    /// straight. Each run of placed points is clipped to the guard band and
    /// joined by the curve; nothing is drawn when no segment is left. A
    /// dashed thinned line on a held column grid sets its dash offset so
    /// that the pattern's phase is 0 at the first point drawn in the column
    /// holding the grid's origin, measured along its run.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let (Some(sx), Some(sy)) = (ctx.x(self.x), ctx.y(self.y)) else {
            return;
        };
        let slot = self.slot(ctx);
        let place = |(a, b)| Some(Point::new(across(&sx, &a, slot, 0.5)?, at(&sy, &b)?));
        let band = ctx.band();
        let thinned = self.thinned(ctx, &sx, &sy);
        let anchored = thinned.is_some();
        let (items, curve) = match thinned {
            Some(lines) => {
                let placed = lines.iter().flat_map(|line| {
                    let points = line.iter().map(|&i| self.points.get(i).and_then(place));
                    points.chain([None])
                });
                (guard::clip(placed, band), Curve::Linear)
            }
            None => (guard::clip(self.points.iter().map(place), band), self.curve),
        };
        let path = path::bez(|p| {
            shape::line()
                .curve(curve)
                .draw(items.iter().map(|q| q.map(|q| (q.x, q.y))), p);
        });
        if path.elements().is_empty() {
            return;
        }
        let width = self.width.unwrap_or(ctx.get_theme().get_line_width());
        let mut style = stroke(width, &self.dash);
        if let Some(grid) = ctx.grid(self.x.get_index()).filter(|_| anchored) {
            style.dash_offset = phase(&items, grid, &self.dash);
        }
        list.commands.push(Command::Stroke {
            path,
            style,
            paint: list::brush(self.color.resolve(ctx)),
        });
    }

    /// The points the line passes through inside the plot area, with their
    /// indices in the data. For a thinned line these are M4's kept samples,
    /// read through the draw's memo so there is no second query; otherwise
    /// they are every placed point.
    fn hits(&self, ctx: &Ctx<'_>, sink: &mut hit::Sink) {
        let (Some(sx), Some(sy)) = (ctx.x(self.x), ctx.y(self.y)) else {
            return;
        };
        let slot = self.slot(ctx);
        let place = |(a, b)| Some(Point::new(across(&sx, &a, slot, 0.5)?, at(&sy, &b)?));
        let area = ctx.get_rect().with_origin(Point::ORIGIN);
        let mut push = |i: usize, p: Option<Point>| {
            if let Some(p) = p.filter(|p| within(area, *p)) {
                sink.push(i, p);
            }
        };
        match self.thinned(ctx, &sx, &sy) {
            Some(lines) => {
                for &i in lines.iter().flatten() {
                    push(i, self.points.get(i).and_then(place));
                }
            }
            None => {
                for (i, p) in self.points.iter().enumerate() {
                    push(i, place(p));
                }
            }
        }
    }

    /// Collected data over 100,000 points; a stroke key the palette
    /// lacks; keys outside a band or point domain.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        super::issues(&self.points, &self.color, ctx, issues);
        super::outside(&self.points, (self.x, self.y), ctx, issues);
    }
}

impl<X: Position, Y: Position> Line<'_, X, Y> {
    /// M4's sample indices per column of the plot area, when the line
    /// thins: [`Thin::Auto`], an x scale with an index bridge, and a
    /// recording channel for data.
    fn thinned(&self, ctx: &Ctx<'_>, sx: &X, sy: &Y) -> Option<m4::Lines> {
        super::thinned(&self.points, (self.thin, self.x), ctx, sx, sy)
    }

    /// The stroke key's slot, when the line is dodged.
    fn slot(&self, ctx: &Ctx<'_>) -> Option<(usize, usize)> {
        self.color.slot(ctx).filter(|_| self.dodge)
    }
}

/// List px past the anchor at which [`phase`] puts the pattern's phase 0.
///
/// A dash starting exactly on the anchor, a vertex, would start an ulp to
/// either side of it depending on how the arc length rounds, and that
/// differs between two draws of a pan. When it starts an ulp before, the
/// dash holds a sliver of the previous segment and joins it, so the corner
/// gains a wedge (measured at up to 17/255 at the seam of two tiles). A
/// millionth of a px past the anchor is far above the rounding, so every
/// draw starts the dash on the following segment.
const PAST: f64 = 1e-6;

/// The dash offset that puts the phase of `pattern` at 0, [`PAST`] after
/// an anchor point. The anchor is the first of `items` (runs of points
/// broken by `None`, as the line draws them) that lies in the column of
/// `grid` holding its origin, `[origin, origin + 1 / density)` across.
/// Both column edges sit a millionth of a column to the left, as thinning
/// takes them ([`thin::LEAD`]), so a sample on the origin, which a pan can
/// place an ulp to either side, stays in the column. The offset is the arc
/// length from the start of the anchor's run plus `PAST`, negated, modulo
/// the pattern's length, since each run starts the pattern afresh at the
/// offset. 0 without a pattern or such a point.
fn phase(items: &[Option<Point>], grid: thin::Grid, pattern: &[f64]) -> f64 {
    let period: f64 = pattern.iter().sum();
    if period <= 0.0 || !period.is_finite() {
        return 0.0;
    }
    let lead = thin::LEAD / grid.density;
    let column = grid.origin - lead..grid.origin + 1.0 / grid.density - lead;
    let mut length = 0.0;
    let mut previous: Option<Point> = None;
    for item in items {
        let Some(p) = *item else {
            previous = None;
            continue;
        };
        length = match previous {
            Some(q) => length + q.distance(p),
            None => 0.0,
        };
        if column.contains(&p.x) {
            return (-length - PAST).rem_euclid(period);
        }
        previous = Some(p);
    }
    0.0
}

#[cfg(test)]
mod tests {
    use kurbo::Point;

    use super::phase;
    use crate::thin::Grid;

    #[test]
    fn the_phase_is_measured_from_the_anchor_points_run() {
        let grid = Grid {
            origin: 10.0,
            density: 1.0,
        };
        let p = |x: f64, y: f64| Some(Point::new(x, y));
        // A first run, a break, then a run reaching the column [10, 11)
        // 3 + 3.5 = 6.5 px after its start: the pattern of 4 px starts 1.5
        // px in, so that 6.5 px later it is at 0.
        let items = [
            p(0.0, 0.0),
            p(5.0, 0.0),
            None,
            p(4.0, 0.0),
            p(7.0, 0.0),
            p(10.5, 0.0),
        ];
        assert!((phase(&items, grid, &[3.0, 1.0]) - 1.5).abs() < 1e-5);
        assert_eq!(phase(&items, grid, &[]), 0.0);
        let elsewhere = Grid {
            origin: 50.0,
            density: 1.0,
        };
        assert_eq!(phase(&items, elsewhere, &[3.0, 1.0]), 0.0);
    }

    #[test]
    fn a_point_an_ulp_left_of_the_origin_anchors() {
        let grid = Grid {
            origin: 10.0,
            density: 1.0,
        };
        let p = |x: f64| Some(Point::new(x, 0.0));
        // A pan can put the sample on the origin either side of it; both
        // anchor, and the column's right edge leads too.
        let left = [p(0.0), p(10.0 - 1e-12), p(10.5)];
        let right = [p(0.0), p(10.0 + 1e-12), p(10.5)];
        assert!((phase(&left, grid, &[3.0, 1.0]) - 2.0).abs() < 1e-5);
        assert!((phase(&right, grid, &[3.0, 1.0]) - 2.0).abs() < 1e-5);
        let past = [p(0.0), p(11.0 - 1e-12)];
        assert_eq!(phase(&past, grid, &[3.0, 1.0]), 0.0);
    }
}
