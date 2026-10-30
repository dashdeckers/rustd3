//! [`Area`]: the region between a line and a base; [`Ribbon`]: the region
//! between two lines.

use kurbo::{BezPath, Point, Shape};
use rustd3_core::color::Rgb;
use rustd3_core::series::{self, Data, Points, Series, m4};

use crate::check::Issue;
use crate::figure::Fits;
use crate::hatch::Hatch;
use crate::legend::Swatch;
use crate::list::{self, Clip, Command, List};
use crate::mark::color::Color;
use crate::mark::{Ctx, Error, Mark, admits, at, polygon, reach, runs};
use crate::scale::{Kind, Paint, Position, ScaleRef};
use crate::thin::Thin;

/// An opaque ribbon color's opacity unless set: a band that lines drawn
/// over it show through (seaborn's error bands use 0.2, matplotlib's
/// examples 0.2–0.3).
const BAND: f64 = 0.25;

/// The region between a line through points and a horizontal base: one
/// `Fill` command, broken where a point cannot be placed. A recording
/// channel on an x scale with an index bridge is thinned to M4's points
/// per column first, which keeps every column's extremes.
pub struct Area<'a, X: Position, Y: Position<Domain = f64>> {
    points: Points<'a, <X as Kind>::Domain, f64>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    base: f64,
    color: Color,
    opacity: f64,
    hatch: Option<Hatch>,
}

/// The region between two lines with one x per row, a ribbon's lower and
/// upper edge: one `Fill` command, broken where a row cannot be placed.
pub struct Ribbon<'a, X: Position, Y: Position<Domain = f64>> {
    lower: Points<'a, <X as Kind>::Domain, f64>,
    upper: Points<'a, <X as Kind>::Domain, f64>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    color: Color,
    /// `None`: a band ([`BAND`]).
    opacity: Option<f64>,
    hatch: Option<Hatch>,
    thin: Thin,
    error: Option<Error>,
}

/// The region between `data` (pairs, a recording channel or two slices)
/// and the base y = 0 on `x` and `y`, in the theme palette's first color,
/// opaque. Its base is held: the y margin stops there.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::Figure;
/// use rustd3_figure::mark::area;
///
/// let mut fig = Figure::new();
/// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
/// fig.panel(0, 0).mark(area([(0.0, 1.0), (1.0, 3.0), (2.0, 2.0)], x, y));
/// ```
pub fn area<'a, X, Y>(
    data: impl Data<'a, <X as Kind>::Domain, f64>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> Area<'a, X, Y>
where
    X: Position,
    Y: Position<Domain = f64>,
{
    Area {
        points: data.into_points(),
        x,
        y,
        base: 0.0,
        color: Color::Theme,
        opacity: 1.0,
        hatch: None,
    }
}

/// The region between `y0` and `y1` of rows `(x, y0, y1)` on `x` and
/// `y`, a band in the theme palette's first color: an opaque color at 0.25
/// opacity, so a line drawn over it shows, a translucent one at its own
/// ([`opacity`](Ribbon::opacity) sets a factor instead):
/// `shape::stack`'s `[y0, y1]` plugs in directly. Both edges enter
/// fitting; 0 is held when an edge reaches exactly 0 (matplotlib's sticky
/// edge), so a stacked area stands on the axis, and nothing else is.
/// Never thinned.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::Figure;
/// use rustd3_figure::mark::ribbon;
///
/// let mut fig = Figure::new();
/// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
/// fig.panel(0, 0).mark(ribbon([(0.0, 1.0, 2.0), (1.0, 0.5, 3.0)], x, y));
/// ```
pub fn ribbon<'a, X, Y>(
    rows: impl IntoIterator<Item = (<X as Kind>::Domain, f64, f64)>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> Ribbon<'a, X, Y>
where
    X: Position,
    Y: Position<Domain = f64>,
{
    let (lower, upper): (Vec<_>, Vec<_>) = rows
        .into_iter()
        .map(|(a, y0, y1)| ((a.clone(), y0), (a, y1)))
        .unzip();
    Ribbon::new(lower.into_points(), upper.into_points(), x, y, None)
}

/// The region between two channels of one recording, `lower` and `upper`,
/// a band as [`ribbon`]'s, thinned together ([`Thin::Auto`]): per column,
/// M4 over both channels
/// at once (core's `Series::m4_joint`), so both edges keep their extremes
/// and the lower edge stays below the upper one wherever the data does.
/// 0 is held when a channel's least or greatest value is exactly 0, as
/// [`ribbon`] holds it. Channels of two recordings draw nothing, and
/// [`Figure::check`](crate::Figure::check) reports
/// [`Error::Recordings`].
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_core::series::{Clock, Recording};
/// use rustd3_figure::Figure;
/// use rustd3_figure::mark::ribbon_series;
///
/// let low: Vec<f64> = (0..1000).map(|i| (f64::from(i) * 0.01).sin() - 1.0).collect();
/// let high: Vec<f64> = low.iter().map(|v| v + 2.0).collect();
/// let rec = Recording::new(Clock::Regular { start: 0.0, period: 0.1 }, [low, high])?;
/// let mut fig = Figure::new();
/// let (t, v) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
/// let (lower, upper) = (rec.series(0).expect("low"), rec.series(1).expect("high"));
/// fig.panel(0, 0).mark(ribbon_series(lower, upper, t, v));
/// # Ok::<(), rustd3_core::series::Error>(())
/// ```
pub fn ribbon_series<'a, C, X, Y>(
    lower: Series<'a, C>,
    upper: Series<'a, C>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> Ribbon<'a, X, Y>
where
    C: series::Coordinate,
    X: Position<Domain = C>,
    Y: Position<Domain = f64>,
{
    let (left, right) = (lower.recording().id(), upper.recording().id());
    let error = (left != right).then_some(Error::Recordings { left, right });
    Ribbon::new(lower.into_points(), upper.into_points(), x, y, error)
}

impl<X: Position, Y: Position<Domain = f64>> Area<'_, X, Y> {
    /// The base the region reaches down (or up) to; 0 by default. It is
    /// held: the y margin stops there. A base that is not finite leaves it
    /// as it was.
    pub fn base(self, y0: f64) -> Self {
        Self {
            base: if y0.is_finite() { y0 } else { self.base },
            ..self
        }
    }

    /// Fills the region with `palette`'s color for `key`, looked up when
    /// the figure draws; a key the palette lacks draws the theme's gray.
    pub fn fill<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self {
        Self {
            color: Color::keyed(palette, key),
            ..self
        }
    }

    /// Fills the region with `c`.
    pub fn color(self, c: Rgb) -> Self {
        Self {
            color: Color::Fixed(c),
            ..self
        }
    }

    /// The fill's opacity, clamped to [0, 1], times its color's own; 1 by
    /// default. NaN leaves it as it was.
    pub fn opacity(self, a: f64) -> Self {
        Self {
            opacity: clamped(a, self.opacity),
            ..self
        }
    }

    /// Hatches the region with `h`, after its fill; legend swatches of the
    /// fill key repeat it.
    pub fn hatch(self, h: Hatch) -> Self {
        Self {
            hatch: Some(h),
            ..self
        }
    }
}

impl<'a, X: Position, Y: Position<Domain = f64>> Mark<'a> for Area<'a, X, Y> {
    fn fit(&self, fits: &mut Fits) {
        super::fit(&self.points, self.x, self.y, fits);
        fits.add(self.y, &self.base);
        fits.stick(self.y, self.base);
        self.color.report(Swatch::Square, self.hatch.as_ref(), fits);
    }

    /// One `Fill` command: per run of points placed on both scales (or of
    /// M4's samples of a thinned channel), the polygon down to the base,
    /// clipped to the guard band, then its hatch when set; nothing when no
    /// run has two points or the base cannot be placed.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let (Some(sx), Some(sy)) = (ctx.x(self.x), ctx.y(self.y)) else {
            return;
        };
        let band = ctx.band();
        let Some(base) = reach(&sy, self.base, [band.y0, band.y1]) else {
            return;
        };
        let place = |(a, b)| Some(Point::new(at(&sx, &a)?, at(&sy, &b)?));
        let placed: Vec<Option<Point>> =
            match super::thinned(&self.points, (Thin::Auto, self.x), ctx, &sx, &sy) {
                Some(lines) => sequence(&lines)
                    .map(|i| i.and_then(|i| self.points.get(i).and_then(place)))
                    .collect(),
                None => self.points.iter().map(place).collect(),
            };
        let mut path = BezPath::new();
        for run in runs(placed).iter().filter(|run| run.len() > 1) {
            let (first, last) = (run[0], run[run.len() - 1]);
            let mut outline = Vec::with_capacity(run.len() + 2);
            outline.push(Point::new(first.x, base));
            outline.extend(run);
            outline.push(Point::new(last.x, base));
            polygon(&mut path, &outline, band);
        }
        let paint = (&self.color, Some(self.opacity), self.hatch.as_ref());
        fill(list, path, paint, ctx);
    }

    /// Collected data over 100,000 points; a fill key the palette lacks;
    /// keys outside a band or point domain.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        super::issues(&self.points, &self.color, ctx, issues);
        super::outside(&self.points, (self.x, self.y), ctx, issues);
    }
}

impl<'a, X: Position, Y: Position<Domain = f64>> Ribbon<'a, X, Y> {
    /// A ribbon between `lower` and `upper`, which keeps `error` if it
    /// cannot be drawn.
    fn new(
        lower: Points<'a, <X as Kind>::Domain, f64>,
        upper: Points<'a, <X as Kind>::Domain, f64>,
        x: ScaleRef<X>,
        y: ScaleRef<Y>,
        error: Option<Error>,
    ) -> Self {
        Ribbon {
            lower,
            upper,
            x,
            y,
            color: Color::Theme,
            opacity: None,
            hatch: None,
            thin: Thin::Auto,
            error,
        }
    }

    /// Fills the ribbon with `palette`'s color for `key`, looked up when
    /// the figure draws; a key the palette lacks draws the theme's gray.
    pub fn fill<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self {
        Self {
            color: Color::keyed(palette, key),
            ..self
        }
    }

    /// Fills the ribbon with `c`.
    pub fn color(self, c: Rgb) -> Self {
        Self {
            color: Color::Fixed(c),
            ..self
        }
    }

    /// The fill's opacity, clamped to [0, 1], times its color's own, in
    /// place of a band's (an opaque color at 0.25, a translucent one at its
    /// own); 1 for an opaque ribbon, as stacked areas are. NaN leaves it as
    /// it was.
    pub fn opacity(self, a: f64) -> Self {
        Self {
            opacity: if a.is_nan() {
                self.opacity
            } else {
                Some(a.clamp(0.0, 1.0))
            },
            ..self
        }
    }

    /// Hatches the ribbon with `h`, after its fill; legend swatches of the
    /// fill key repeat it.
    pub fn hatch(self, h: Hatch) -> Self {
        Self {
            hatch: Some(h),
            ..self
        }
    }

    /// Whether a ribbon over two channels of a recording thins them
    /// together: [`Thin::Auto`] by default, [`Thin::Off`] to draw every
    /// sample. Rows given as values are never thinned.
    pub fn thin(self, t: Thin) -> Self {
        Self { thin: t, ..self }
    }

    /// M4 over both edges per column of the plot area, when the ribbon
    /// thins: [`Thin::Auto`], an x scale with an index bridge, and two
    /// channels of one recording.
    fn thinned(&self, ctx: &Ctx<'_>, sx: &X, sy: &Y) -> Option<m4::Lines> {
        if self.thin == Thin::Off {
            return None;
        }
        let indexed = sx.indexed()?;
        let edges = ctx.edges(indexed, self.x.index())?;
        let valid = |v| admits(sy, v);
        indexed.thin_joint(&self.lower, &self.upper, &edges, &valid, &mut ctx.frame())
    }
}

impl<'a, X: Position, Y: Position<Domain = f64>> Mark<'a> for Ribbon<'a, X, Y> {
    /// Both edges, each as a line does (a recording channel through the x
    /// scale's index), holding 0 when an edge reaches exactly 0; nothing
    /// for a ribbon that cannot be drawn.
    fn fit(&self, fits: &mut Fits) {
        if self.error.is_some() {
            return;
        }
        super::fit(&self.lower, self.x, self.y, fits);
        super::fit(&self.upper, self.x, self.y, fits);
        let (Some(sx), Some(sy)) = (fits.scale(self.x), fits.scale(self.y)) else {
            return;
        };
        if reaches_zero(&self.lower, sx, sy) || reaches_zero(&self.upper, sx, sy) {
            fits.stick(self.y, 0.0);
        }
        self.color.report(Swatch::Square, self.hatch.as_ref(), fits);
    }

    /// One `Fill` command: per run of rows whose x and both edges are
    /// placed (M4's joint samples of thinned channels), the polygon along
    /// the upper edge and back along the lower one, clipped to the guard
    /// band, then its hatch when set; nothing when no run has two rows.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        if self.error.is_some() {
            return;
        }
        let (Some(sx), Some(sy)) = (ctx.x(self.x), ctx.y(self.y)) else {
            return;
        };
        let band = ctx.band();
        let row = |i: usize| {
            let (a, low) = self.lower.get(i)?;
            let (_, high) = self.upper.get(i)?;
            let px = at(&sx, &a)?;
            Some((
                Point::new(px, at(&sy, &high)?),
                Point::new(px, at(&sy, &low)?),
            ))
        };
        let placed: Vec<Option<(Point, Point)>> = match self.thinned(ctx, &sx, &sy) {
            Some(lines) => sequence(&lines).map(|i| i.and_then(row)).collect(),
            None => (0..self.lower.len().min(self.upper.len()))
                .map(row)
                .collect(),
        };
        let mut path = BezPath::new();
        for run in runs(placed).iter().filter(|run| run.len() > 1) {
            let upper = run.iter().map(|(high, _)| *high);
            let lower = run.iter().rev().map(|(_, low)| *low);
            let outline: Vec<Point> = upper.chain(lower).collect();
            polygon(&mut path, &outline, band);
        }
        let paint = (&self.color, self.opacity, self.hatch.as_ref());
        fill(list, path, paint, ctx);
    }

    /// A ribbon over two recordings; collected rows over 100,000; a fill
    /// key the palette lacks; keys outside a band or point domain.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        if let Some(error) = &self.error {
            issues.push(Issue::Mark {
                panel: ctx.panel(),
                mark: ctx.mark(),
                error: error.clone(),
            });
        }
        super::issues(&self.lower, &self.color, ctx, issues);
        super::outside(&self.lower, (self.x, self.y), ctx, issues);
    }
}

/// Whether some value of `points` that `y` admits is exactly 0: a recording
/// channel's least or greatest one, read from its index through `x`'s
/// bridge (a 0 strictly inside the channel's range stops no margin), other
/// data value by value.
fn reaches_zero<X: Position, Y: Position<Domain = f64>>(
    points: &Points<'_, <X as Kind>::Domain, f64>,
    x: &X,
    y: &Y,
) -> bool {
    let valid = |v| admits(y, v);
    let channel = points
        .indexed()
        .and_then(|channel| x.indexed()?.series_extent(&channel, None, &valid));
    match channel {
        Some(extent) => extent
            .y
            .is_some_and(|[low, high]| low == 0.0 || high == 0.0),
        None => points.iter().any(|(_, v)| v == 0.0 && valid(v)),
    }
}

/// M4's polylines as one sequence of sample indices, `None` after each.
fn sequence(lines: &m4::Lines) -> impl Iterator<Item = Option<usize>> + '_ {
    lines
        .iter()
        .flat_map(|line| line.iter().map(|&i| Some(i)).chain([None]))
}

/// `a` clamped to [0, 1]; `was` for NaN.
fn clamped(a: f64, was: f64) -> f64 {
    if a.is_nan() { was } else { a.clamp(0.0, 1.0) }
}

/// What a region is filled with: its color, the opacity it multiplies the
/// color's by (`None`: a band's), and its hatch.
type Filling<'p> = (&'p Color, Option<f64>, Option<&'p Hatch>);

/// Pushes `path` filled in `paint`'s color at its opacity (a band's: an
/// opaque color at 0.25, a translucent one at its own), then hatched
/// (clipped to the path) when set, unless it is empty.
fn fill(list: &mut List, path: BezPath, (color, opacity, hatch): Filling<'_>, ctx: &Ctx<'_>) {
    if path.elements().is_empty() {
        return;
    }
    let bounds = path.bounding_box();
    let clip = hatch.map(|_| Clip::Path(path.clone()));
    let c = color.resolve(ctx);
    let alpha = match opacity {
        Some(a) => c.opacity * a,
        None if c.opacity < 1.0 => c.opacity,
        None => BAND,
    };
    list.commands.push(Command::Fill {
        path,
        paint: list::brush(c.with_opacity(alpha)),
    });
    if let Some(clip) = clip {
        super::hatch(hatch, clip, bounds, ctx, list);
    }
}
