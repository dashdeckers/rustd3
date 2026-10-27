//! [`Dots`]: a scatter of points.

use kurbo::{Point, Shape};
use rustd3_core::color::Rgb;
use rustd3_core::series::{Data, Points};

use crate::figure::Fits;
use crate::list::{self, Command, List};
use crate::mark::color::Color;
use crate::mark::{Ctx, Mark, at, positive};
use crate::scale::{Kind, Paint, Position, ScaleRef};

/// How closely a dot's circle follows the true circle, in px.
const TOLERANCE: f64 = 1e-3;

/// A scatter: one circle per point, all in one color, drawn as one
/// `Markers` command.
pub struct Dots<'a, X: Position, Y: Position> {
    points: Points<'a, <X as Kind>::Domain, <Y as Kind>::Domain>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    color: Color,
    r: Option<f64>,
    opacity: f64,
}

/// A scatter of `data` (pairs, a recording channel or two slices) through
/// `x` and `y`, in the theme palette's first color, each dot the theme's
/// marker size across.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_core::scheme::COLORBLIND;
/// use rustd3_figure::Figure;
/// use rustd3_figure::mark::dots;
///
/// let mut fig = Figure::new();
/// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
/// let color = fig.scale(scale::ordinal(["a", "b"], COLORBLIND));
/// fig.panel(0, 0)
///     .mark(dots([(0.0, 1.0), (1.0, 3.0)], x, y).fill(color, "a"))
///     .mark(dots([(0.5, 2.0)], x, y).fill(color, "b").r(6.0).opacity(0.5));
/// ```
pub fn dots<'a, X: Position, Y: Position>(
    data: impl Data<'a, <X as Kind>::Domain, <Y as Kind>::Domain>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> Dots<'a, X, Y> {
    Dots {
        points: data.into_points(),
        x,
        y,
        color: Color::Theme,
        r: None,
        opacity: 1.0,
    }
}

impl<X: Position, Y: Position> Dots<'_, X, Y> {
    /// Fills the dots with `palette`'s color for `key`, looked up when the
    /// figure draws; a key the palette lacks draws the theme's gray.
    pub fn fill<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self {
        Self {
            color: Color::keyed(palette, key),
            ..self
        }
    }

    /// Fills the dots with `c`.
    pub fn color(self, c: Rgb) -> Self {
        Self {
            color: Color::Fixed(c),
            ..self
        }
    }

    /// Each dot's radius in px; by default half the theme's marker size.
    /// A radius that is not finite and positive leaves it as it was.
    pub fn r(self, px: f64) -> Self {
        Self {
            r: positive(px).or(self.r),
            ..self
        }
    }

    /// The dots' opacity, clamped to [0, 1]; 1 by default. NaN leaves it as
    /// it was.
    pub fn opacity(self, a: f64) -> Self {
        Self {
            opacity: if a.is_nan() {
                self.opacity
            } else {
                a.clamp(0.0, 1.0)
            },
            ..self
        }
    }
}

impl<'a, X: Position, Y: Position> Mark<'a> for Dots<'a, X, Y> {
    fn fit(&self, fits: &mut Fits) {
        super::fit(&self.points, self.x, self.y, fits);
    }

    /// One `Markers` command: a circle at every point placed on both
    /// scales, finite, and at most a radius outside the plot area; nothing
    /// when no point is.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let (sx, sy) = (ctx.x(self.x), ctx.y(self.y));
        let r = self.r.unwrap_or(ctx.theme().marker() / 2.0);
        let bounds = ctx.rect().with_origin(Point::ORIGIN).inflate(r, r);
        let at: Vec<Point> = self
            .points
            .iter()
            .filter_map(|(a, b)| Some(Point::new(at(&sx, &a)?, at(&sy, &b)?)))
            .filter(|p| within(bounds, *p))
            .collect();
        if at.is_empty() {
            return;
        }
        let color = self.color.resolve(ctx);
        list.commands.push(Command::Markers {
            shape: kurbo::Circle::new(Point::ORIGIN, r).to_path(TOLERANCE),
            at,
            paint: list::brush(color.with_opacity(color.opacity * self.opacity)),
            sizes: None,
            paints: None,
        });
    }
}

/// Whether `p` lies in `bounds`, edges included.
fn within(bounds: kurbo::Rect, p: Point) -> bool {
    (bounds.x0..=bounds.x1).contains(&p.x) && (bounds.y0..=bounds.y1).contains(&p.y)
}
