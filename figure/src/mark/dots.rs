//! [`Dots`]: a scatter of points.

use kurbo::{Point, Shape};
use rustd3_core::color::Rgb;
use rustd3_core::series::{Data, Points};

use crate::check::Issue;
use crate::figure::Fits;
use crate::hit;
use crate::list::{self, Command, List};
use crate::mark::color::Color;
use crate::mark::{Ctx, Mark, across, at, positive, within};
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
    dodge: bool,
    jitter: f64,
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
///
/// The data's types are the handles' domains: timestamps on a linear
/// handle do not compile (E0271, `<Linear as Kind>::Domain ==
/// Timestamp`), on a time handle they do.
///
/// ```compile_fail,E0271
/// # use rustd3_figure::{Figure, mark::dots};
/// # use rustd3_core::scale;
/// let mut fig = Figure::new();
/// let x = fig.fit(scale::linear());
/// let y = fig.fit(scale::linear());
/// let t = rustd3_core::jiff::Timestamp::UNIX_EPOCH;
/// fig.panel(0, 0).mark(dots([(t, 1.0)], x, y)); // E0271
/// ```
/// ```
/// # use rustd3_figure::{Figure, mark::dots};
/// # use rustd3_core::{scale, time};
/// let mut fig = Figure::new();
/// let x = fig.fit(time::utc());
/// let y = fig.fit(scale::linear());
/// let t = rustd3_core::jiff::Timestamp::UNIX_EPOCH;
/// fig.panel(0, 0).mark(dots([(t, 1.0)], x, y));
/// ```
///
/// A palette is not a position: an ordinal color handle as y does not
/// compile (E0277, `Ordinal<Source, Rgb>: Position` is not satisfied);
/// it colors dots through [`fill`](Dots::fill).
///
/// ```compile_fail,E0277
/// # use rustd3_figure::{Figure, mark::dots};
/// # use rustd3_core::{scale, scheme};
/// #[derive(Clone, PartialEq)]
/// enum Source { Hub, Bearing }
///
/// let mut fig = Figure::new();
/// let speed = fig.fit(scale::linear());
/// let color = fig.scale(scale::ordinal([Source::Hub, Source::Bearing], scheme::COLORBLIND));
/// fig.panel(0, 0).mark(dots([(0.0, Source::Hub)], speed, color)); // E0277
/// ```
/// ```
/// # use rustd3_figure::{Figure, mark::dots};
/// # use rustd3_core::{scale, scheme};
/// #[derive(Clone, PartialEq)]
/// enum Source { Hub, Bearing }
///
/// let mut fig = Figure::new();
/// let (speed, value) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
/// let color = fig.scale(scale::ordinal([Source::Hub, Source::Bearing], scheme::COLORBLIND));
/// fig.panel(0, 0).mark(dots([(0.0, 1.0)], speed, value).fill(color, Source::Hub));
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
        dodge: false,
        jitter: 0.0,
    }
}

impl<X: Position, Y: Position> Dots<'_, X, Y> {
    /// Fills the dots with `palette`'s color for `key`, looked up when the
    /// figure draws; a key the palette lacks draws the theme's gray. The
    /// key has the palette's key type: a key of another type does not
    /// compile (E0308).
    ///
    /// ```compile_fail,E0308
    /// # use rustd3_figure::{Figure, mark::dots};
    /// # use rustd3_core::{scale, scheme};
    /// #[derive(Clone, PartialEq)]
    /// enum Source { Hub, Bearing }
    /// enum Wheel { FL, FR }
    ///
    /// let mut fig = Figure::new();
    /// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    /// let color = fig.scale(scale::ordinal([Source::Hub, Source::Bearing], scheme::COLORBLIND));
    /// fig.panel(0, 0).mark(dots([(0.0, 1.0)], x, y).fill(color, Wheel::FL)); // E0308
    /// ```
    /// ```
    /// # use rustd3_figure::{Figure, mark::dots};
    /// # use rustd3_core::{scale, scheme};
    /// #[derive(Clone, PartialEq)]
    /// enum Source { Hub, Bearing }
    ///
    /// let mut fig = Figure::new();
    /// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    /// let color = fig.scale(scale::ordinal([Source::Hub, Source::Bearing], scheme::COLORBLIND));
    /// fig.panel(0, 0).mark(dots([(0.0, 1.0)], x, y).fill(color, Source::Bearing));
    /// ```
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

    /// Places the dots in their fill key's slot of the band: the band
    /// split into one sub-band per palette key, in the palette's order, so
    /// that dots, bars and lines of one key line up. It acts along x when
    /// x has bands, else along y when y has; scales without bands place
    /// dots at their value.
    ///
    /// ```
    /// use rustd3_core::scale;
    /// use rustd3_core::scheme::COLORBLIND;
    /// use rustd3_figure::Figure;
    /// use rustd3_figure::mark::dots;
    ///
    /// let mut fig = Figure::new();
    /// let day = fig.scale(scale::band(["mon", "tue"]).padding(0.1));
    /// let y = fig.fit(scale::linear());
    /// let color = fig.scale(scale::ordinal(["a", "b"], COLORBLIND));
    /// fig.panel(0, 0)
    ///     .mark(dots([("mon", 1.0), ("mon", 2.0)], day, y).fill(color, "a").dodge().jitter(0.8))
    ///     .mark(dots([("tue", 1.5)], day, y).fill(color, "b").dodge());
    /// ```
    pub fn dodge(self) -> Self {
        Self {
            dodge: true,
            ..self
        }
    }

    /// Spreads the dots across `share` of their band (or dodged slot), as
    /// seaborn's `stripplot` does: each at `0.5 + share·(j − 0.5)` of the
    /// way across, with `j` in [0, 1) a hash of the point's index, so the
    /// same data jitters the same way in every render and on every
    /// platform. Clamped to [0, 1]; 0 by default (the band's middle). NaN
    /// leaves it as it was.
    pub fn jitter(self, share: f64) -> Self {
        Self {
            jitter: if share.is_nan() {
                self.jitter
            } else {
                share.clamp(0.0, 1.0)
            },
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
        let r = self.radius(ctx);
        let at: Vec<Point> = self.placed(ctx).map(|(_, p)| p).collect();
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

    /// Every point drawn, with its index in the data.
    fn hits(&self, ctx: &Ctx<'_>, sink: &mut hit::Sink) {
        for (i, p) in self.placed(ctx) {
            sink.push(i, p);
        }
    }

    /// Collected data over 100,000 points; a fill key the palette lacks;
    /// keys outside a band or point domain.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        super::issues(&self.points, &self.color, ctx, issues);
        super::outside(&self.points, (self.x, self.y), ctx, issues);
    }
}

impl<X: Position, Y: Position> Dots<'_, X, Y> {
    /// Each dot's radius in px.
    fn radius(&self, ctx: &Ctx<'_>) -> f64 {
        self.r.unwrap_or(ctx.theme().marker() / 2.0)
    }

    /// Each point placed on both scales, finite, and at most a radius
    /// outside the plot area, with its index in the data, in the panel's
    /// coordinates.
    fn placed<'c>(&'c self, ctx: &Ctx<'_>) -> impl Iterator<Item = (usize, Point)> + 'c {
        let (sx, sy) = (ctx.x(self.x), ctx.y(self.y));
        let r = self.radius(ctx);
        let bounds = ctx.rect().with_origin(Point::ORIGIN).inflate(r, r);
        let slot = self.color.slot(ctx).filter(|_| self.dodge);
        let along_y = sx.bandwidth() == 0.0 && sy.bandwidth() > 0.0;
        let share = move |i: usize| 0.5 + self.jitter * (jitter(i) - 0.5);
        self.points
            .iter()
            .enumerate()
            .filter_map(move |(i, (a, b))| {
                let p = if along_y {
                    Point::new(at(&sx, &a)?, across(&sy, &b, slot, share(i))?)
                } else {
                    Point::new(across(&sx, &a, slot, share(i))?, at(&sy, &b)?)
                };
                Some((i, p))
            })
            .filter(move |(_, p)| within(bounds, *p))
    }
}

/// Point `i`'s jitter in [0, 1): a splitmix64-style hash of its index
/// (`proto/figmodel`'s), the same on every run and platform.
fn jitter(i: usize) -> f64 {
    let mut z = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z ^= z >> 29;
    z = z.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^= z >> 32;
    (z >> 11) as f64 / (1_u64 << 53) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jitter_is_a_fixed_hash_of_the_index_in_the_unit_interval() {
        assert_eq!(jitter(0), 0.0);
        let first: Vec<f64> = (0..1000).map(jitter).collect();
        assert!(first.iter().all(|j| (0.0..1.0).contains(j)));
        let mean = first.iter().sum::<f64>() / 1000.0;
        assert!((mean - 0.5).abs() < 0.03, "{mean}");
        assert_eq!(first, (0..1000).map(jitter).collect::<Vec<_>>());
        assert_eq!(format!("{:.12}", jitter(1)), format!("{:.12}", first[1]));
    }
}
