//! [`Dots`]: a scatter of points.

use std::collections::HashMap;

use kurbo::{Point, Shape};
use peniko::Brush;
use rustd3_core::color::Rgb;
use rustd3_core::series::{Data, Points};

use crate::check::Issue;
use crate::figure::Fits;
use crate::legend::Swatch;
use crate::list::{self, Command, List, Paints};
use crate::mark::channel::Channel;
use crate::mark::color::{Alpha, Color};
use crate::mark::{Ctx, Error, Mark, across, at, positive, within};
use crate::scale::{Kind, Paint, Position, ScaleRef};
use crate::{hit, text};

/// How closely a dot's circle follows the true circle, in px.
const TOLERANCE: f64 = 1e-3;

/// A scatter: one circle per point, in one color or colored and sized per
/// point through handles, drawn as one `Markers` command.
pub struct Dots<'a, X: Position, Y: Position> {
    points: Points<'a, <X as Kind>::Domain, <Y as Kind>::Domain>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    color: Color,
    r: Option<f64>,
    opacity: f64,
    dodge: bool,
    jitter: f64,
    fill_by: Option<Result<Channel<Rgb>, Error>>,
    size_by: Option<Result<Channel<f64>, Error>>,
}

impl<'a, X: Position, Y: Position> std::fmt::Debug for Dots<'a, X, Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dots").finish_non_exhaustive()
    }
}

/// One dot as drawn: its datum's index, where it is centered in the
/// panel's coordinates, its color from `fill_by` and its radius in px.
struct Placed {
    index: usize,
    at: Point,
    fill: Option<Rgb>,
    r: f64,
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
        fill_by: None,
        size_by: None,
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
    #[must_use]
    pub fn fill<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self {
        Self {
            color: Color::keyed(palette, key),
            ..self
        }
    }

    /// Fills the dots with `c`.
    #[must_use]
    pub fn color(self, c: Rgb) -> Self {
        Self {
            color: Color::Fixed(c),
            ..self
        }
    }

    /// Each dot's radius in px; by default half the theme's marker size.
    /// A radius that is not finite and positive leaves it as it was. Radii
    /// from this and from `size_by` are drawn at most three times the plot
    /// area's larger side, the guard band's reach.
    #[must_use]
    pub fn r(self, px: f64) -> Self {
        Self {
            r: positive(px).or(self.r),
            ..self
        }
    }

    /// Places the dots in their fill key's slot of the band. The band is
    /// split into one sub-band per palette key, in the palette's order, so
    /// that dots, bars and lines of one key line up. It acts along x when
    /// x has bands, else along y when y has them. Scales without bands
    /// place dots at their value.
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
    #[must_use]
    pub fn dodge(self) -> Self {
        Self {
            dodge: true,
            ..self
        }
    }

    /// Spreads the dots across `share` of their band (or dodged slot), as
    /// seaborn's `stripplot` does. Each dot sits `0.5 + share·(j − 0.5)` of
    /// the way across, where `j` in [0, 1) is a hash of the point's index,
    /// so the same data jitters the same way in every render and on every
    /// platform. `share` is clamped to [0, 1] and is 0 by default (the
    /// band's middle). NaN leaves it as it was.
    #[must_use]
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
    #[must_use]
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

    /// Colors each dot through `palette`, a continuous or binned color
    /// handle, from its value in `values` (one per point, in point order),
    /// in place of [`fill`](Dots::fill)'s color, which still picks the
    /// dodge slot. The colors resolve when the figure draws, into one
    /// `Markers` command with a paint table, and a fitted handle fits to the
    /// values. A dot whose value has no color (NaN, a value the scale does
    /// not take) is skipped. A channel of another length than the points
    /// is never truncated: the mark draws nothing and
    /// [`Figure::check`](crate::Figure::check) reports
    /// [`Error::ChannelLength`].
    ///
    /// ```
    /// use rustd3_core::{scale, scheme};
    /// use rustd3_figure::Figure;
    /// use rustd3_figure::mark::dots;
    ///
    /// let mut fig = Figure::new();
    /// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    /// let heat = fig.fit(scale::sequential([0.0, 1.0], scheme::viridis));
    /// let radius = fig.scale(scale::sqrt().domain([0.0, 100.0]).range([0.0, 12.0]));
    /// fig.panel(0, 0).mark(
    ///     dots([(0.0, 1.0), (1.0, 3.0), (2.0, 2.0)], x, y)
    ///         .fill_by(heat, [20.0, 45.0, 80.0])
    ///         .size_by(radius, [10.0, 50.0, 100.0]),
    /// );
    /// ```
    #[must_use]
    pub fn fill_by<C>(self, palette: ScaleRef<C>, values: impl IntoIterator<Item = f64>) -> Self
    where
        C: Paint<Domain = f64>,
    {
        let channel = Channel::fill(palette, values.into_iter().collect());
        Self {
            fill_by: Some(channel.of(self.points.len())),
            ..self
        }
    }

    /// Sizes each dot through `radius`, a numeric handle whose output is a
    /// radius in px, from its value in `values` (one per point, in point
    /// order), in place of [`r`](Dots::r). A fitted handle fits to the
    /// values with the theme's margin, as any numeric handle; give its
    /// domain to map values exactly. A dot whose radius is not finite is
    /// skipped. A channel of another length than the points is never
    /// truncated: the mark draws nothing and
    /// [`Figure::check`](crate::Figure::check) reports
    /// [`Error::ChannelLength`].
    #[must_use]
    pub fn size_by<S>(self, radius: ScaleRef<S>, values: impl IntoIterator<Item = f64>) -> Self
    where
        S: Position<Domain = f64>,
    {
        let channel = Channel::size(radius, values.into_iter().collect());
        Self {
            size_by: Some(channel.of(self.points.len())),
            ..self
        }
    }
}

impl<'a, X: Position, Y: Position> Mark<'a> for Dots<'a, X, Y> {
    /// The points, and the values of `fill_by` and `size_by` to their
    /// handles; nothing when a channel's length differs from the points.
    fn fit(&self, fits: &mut Fits) {
        if self.error().is_some() {
            return;
        }
        super::fit(&self.points, self.x, self.y, fits);
        self.color.report(Swatch::Dot, None, Alpha::OPAQUE, fits);
        if let Some(Ok(channel)) = &self.fill_by {
            channel.fit(fits);
        }
        if let Some(Ok(channel)) = &self.size_by {
            channel.fit(fits);
        }
    }

    /// One `Markers` command: a circle at every point placed on both
    /// scales, finite, with a color and a finite radius from its channels,
    /// and at most its radius outside the plot area (grown by a widget's
    /// overscan). Nothing is drawn when no point qualifies. Sized dots
    /// stamp a unit circle at per-point radii, and colored dots index a
    /// table of their distinct colors (8-bit, in first-seen order).
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        // One pass over the points into the command's own vectors: the
        // centers, and the radii and colors only when a channel sets them.
        let mut at: Vec<Point> = Vec::with_capacity(self.points.len());
        let mut sizes: Option<Vec<f64>> = self.size_by.is_some().then(Vec::new);
        let mut fills: Option<Vec<Rgb>> = self.fill_by.is_some().then(Vec::new);
        for dot in self.placed(ctx, ctx.view()) {
            at.push(dot.at);
            if let Some(sizes) = &mut sizes {
                sizes.push(dot.r);
            }
            if let Some(fills) = &mut fills {
                fills.push(
                    dot.fill
                        .expect("invariant: placed dots of fill_by have a color"),
                );
            }
        }
        if at.is_empty() {
            return;
        }
        let color = self.color.resolve(ctx);
        let shown = |c: Rgb| c.with_opacity(c.opacity * self.opacity);
        // Sized dots stamp a unit circle, followed as closely at their
        // largest radius as a plain dot's circle is.
        let (r, tolerance) = match &sizes {
            Some(sizes) => (
                1.0,
                TOLERANCE / sizes.iter().fold(1.0, |m: f64, r| m.max(r.abs())),
            ),
            None => (self.radius(ctx), TOLERANCE),
        };
        let paints = fills.map(|fills| paints(fills.into_iter().map(shown)));
        list.commands.push(Command::Markers {
            shape: kurbo::Circle::new(Point::ORIGIN, r).to_path(tolerance),
            at,
            paint: list::brush(shown(color)),
            sizes,
            paints,
        });
    }

    /// Every point drawn in the plot area (not past it into an overscan),
    /// with its index in the data.
    fn hits(&self, ctx: &Ctx<'_>, sink: &mut hit::Sink) {
        for dot in self.placed(ctx, ctx.get_rect().with_origin(Point::ORIGIN)) {
            sink.push(dot.index, dot.at);
        }
    }

    /// A channel whose length differs from the points; collected data over
    /// 100,000 points; a fill key the palette lacks; keys outside a band or
    /// point domain.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        if let Some(error) = self.error() {
            issues.push(Issue::Mark {
                panel: ctx.get_panel(),
                mark: ctx.get_mark(),
                error: error.clone(),
            });
        }
        super::issues(&self.points, &self.color, ctx, issues);
        super::outside(&self.points, (self.x, self.y), ctx, issues);
    }
}

impl<X: Position, Y: Position> Dots<'_, X, Y> {
    /// Each dot's radius in px without `size_by`.
    fn radius(&self, ctx: &Ctx<'_>) -> f64 {
        cap(
            self.r.unwrap_or(ctx.get_theme().get_marker() / 2.0),
            largest(ctx),
        )
    }

    /// The error a channel keeps, `fill_by`'s first.
    fn error(&self) -> Option<&Error> {
        [
            self.fill_by.as_ref().and_then(|c| c.as_ref().err()),
            self.size_by.as_ref().and_then(|c| c.as_ref().err()),
        ]
        .into_iter()
        .flatten()
        .next()
    }

    /// Each dot drawn, in data order: placed on both scales, finite, with
    /// a color and a finite radius when its channels give them, and at
    /// most its radius outside `area` (in the panel's coordinates, the plot
    /// area or the view a widget grows it to). None when a channel keeps an
    /// error or a handle is of another figure.
    fn placed<'c>(&'c self, ctx: &Ctx<'_>, area: kurbo::Rect) -> impl Iterator<Item = Placed> + 'c {
        let scales = ctx.x(self.x).zip(ctx.y(self.y));
        let r = self.radius(ctx);
        let most = largest(ctx);
        let slot = self.color.slot(ctx).filter(|_| self.dodge);
        let along_y = scales
            .as_ref()
            .is_some_and(|(sx, sy)| sx.bandwidth() == 0.0 && sy.bandwidth() > 0.0);
        // Unjittered dots skip the hash: 0.5 + 0 × (u − 0.5) is 0.5.
        let share = move |i: usize| {
            if self.jitter == 0.0 {
                0.5
            } else {
                0.5 + self.jitter * (jitter(i) - 0.5)
            }
        };
        let count = if self.error().is_some() || scales.is_none() {
            0
        } else {
            self.points.len()
        };
        let fills = self
            .fill_by
            .as_ref()
            .and_then(|c| c.as_ref().ok())
            .map(|c| c.resolve(ctx));
        let sizes = self
            .size_by
            .as_ref()
            .and_then(|c| c.as_ref().ok())
            .map(|c| c.resolve(ctx));
        self.points
            .iter()
            .enumerate()
            .take(count)
            .filter_map(move |(i, (a, b))| {
                let (sx, sy) = scales.as_ref()?;
                let fill = match &fills {
                    Some(fills) => Some(fills.get(i).copied().flatten()?),
                    None => None,
                };
                let r = match &sizes {
                    Some(sizes) => cap(sizes.get(i).copied().flatten()?, most),
                    None => r,
                };
                let at = if along_y {
                    Point::new(at(sx, &a)?, across(sy, &b, slot, share(i))?)
                } else {
                    Point::new(across(sx, &a, slot, share(i))?, at(sy, &b)?)
                };
                within(area.inflate(r.abs(), r.abs()), at).then_some(Placed {
                    index: i,
                    at,
                    fill,
                    r,
                })
            })
    }
}

/// The guard band's reach, three times the plot area's larger side: a
/// dot that wide covers the band from anywhere it is drawn.
fn largest(ctx: &Ctx<'_>) -> f64 {
    let rect = ctx.get_rect();
    3.0 * rect.width().abs().max(rect.height().abs())
}

/// `r` at most `most` in magnitude, keeping its sign, so that its circle
/// is followed by a path of bounded length.
fn cap(r: f64, most: f64) -> f64 {
    if r.abs() > most { most.copysign(r) } else { r }
}

/// The paint table of `colors`, one per dot: their distinct 8-bit colors
/// in first-seen order, and each dot's index into it.
fn paints(colors: impl Iterator<Item = Rgb>) -> Paints {
    let mut seen: HashMap<[u8; 4], u32> = HashMap::new();
    let mut table: Vec<Brush> = Vec::new();
    let index = colors
        .map(|c| {
            *seen.entry(text::channels(c)).or_insert_with(|| {
                table.push(list::brush(c));
                u32::try_from(table.len() - 1).expect("invariant: fewer than 2^32 distinct colors")
            })
        })
        .collect();
    Paints { table, index }
}

/// Point `i`'s jitter in [0, 1): a splitmix64-style hash of its index,
/// the same on every run and platform.
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
