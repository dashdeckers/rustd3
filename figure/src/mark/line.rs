//! [`Line`]: a polyline or curve through points.

use kurbo::Point;
use rustd3_core::color::Rgb;
use rustd3_core::series::{Data, Points, m4};
use rustd3_core::shape::{self, Curve};

use crate::figure::Fits;
use crate::list::{self, Command, List};
use crate::mark::color::Color;
use crate::mark::{Ctx, Mark, admits, at, dashes, guard, positive, stroke};
use crate::path;
use crate::scale::{Kind, Paint, Position, ScaleRef};
use crate::thin::{self, Thin};

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
    }
}

impl<X: Position, Y: Position> Line<'_, X, Y> {
    /// Strokes the line with `palette`'s color for `key`, looked up when
    /// the figure draws; a key the palette lacks draws the theme's gray.
    pub fn stroke<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self {
        Self {
            color: Color::keyed(palette, key),
            ..self
        }
    }

    /// Strokes the line with `c`.
    pub fn color(self, c: Rgb) -> Self {
        Self {
            color: Color::Fixed(c),
            ..self
        }
    }

    /// The line's width in px; by default the theme's, 1.5 pt (2 px). A
    /// width that is not finite and positive leaves it as it was.
    pub fn width(self, px: f64) -> Self {
        Self {
            width: positive(px).or(self.width),
            ..self
        }
    }

    /// Dashes: on and off lengths in px, from offset 0 at the first point
    /// drawn. A pattern with a negative or non-finite length, or none
    /// longer than 0, leaves the line as it was.
    pub fn dash(self, pattern: impl IntoIterator<Item = f64>) -> Self {
        let dash = dashes(pattern).unwrap_or(self.dash);
        Self { dash, ..self }
    }

    /// How the line joins its points: d3's curves, straight by default.
    /// Thinned lines join their points straight.
    pub fn curve(self, c: Curve) -> Self {
        Self { curve: c, ..self }
    }

    /// Whether the line thins a recording channel: [`Thin::Auto`] by
    /// default, [`Thin::Off`] to draw every sample.
    pub fn thin(self, t: Thin) -> Self {
        Self { thin: t, ..self }
    }
}

impl<'a, X: Position, Y: Position> Mark<'a> for Line<'a, X, Y> {
    fn fit(&self, fits: &mut Fits) {
        super::fit(&self.points, self.x, self.y, fits);
    }

    /// One `Stroke` command through every point placed on both scales, or
    /// through the samples M4 keeps per column of a thinned line, joined
    /// straight: each run of placed points is clipped to the guard band
    /// and joined by the curve; nothing when no segment is left.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let (sx, sy) = (ctx.x(self.x), ctx.y(self.y));
        let place = |(a, b)| Some(Point::new(at(&sx, &a)?, at(&sy, &b)?));
        let band = guard::band(ctx.rect().width(), ctx.rect().height());
        let (items, curve) = match self.thinned(ctx, &sx, &sy) {
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
        let width = self.width.unwrap_or(ctx.theme().line_width());
        list.commands.push(Command::Stroke {
            path,
            style: stroke(width, &self.dash),
            paint: list::brush(self.color.resolve(ctx)),
        });
    }
}

impl<X: Position, Y: Position> Line<'_, X, Y> {
    /// M4's sample indices per column of the plot area, when the line
    /// thins: [`Thin::Auto`], an x scale with an index bridge, and a
    /// recording channel for data.
    fn thinned(&self, ctx: &Ctx<'_>, sx: &X, sy: &Y) -> Option<m4::Lines> {
        if self.thin == Thin::Off {
            return None;
        }
        let channel = self.points.indexed()?;
        let indexed = sx.indexed()?;
        let edges = thin::edges(indexed, ctx.rect().width(), ctx.columns())?;
        indexed.thin(&channel, &edges, &|v| admits(sy, v), &mut ctx.frame())
    }
}
