//! [`Rect`]: one rectangle per row in data units, and the rectangle code
//! bars and histograms draw through.

use std::any::Any;

use kurbo::{Join, Shape};
use peniko::Brush;
use rustd3_core::color::Rgb;

use crate::check::Issue;
use crate::figure::Fits;
use crate::legend::Swatch;
use crate::list::{self, Command, List};
use crate::mark::color::{Alpha, Color};
use crate::mark::{Ctx, Mark, at, positive};
use crate::scale::{Kind, Paint, Position, ScaleRef};

/// How closely a rectangle's path follows it, in px: exact, as it has no
/// curves.
const TOLERANCE: f64 = 0.1;

/// Observable Plot's `rect`: one rectangle per row, `x0..x1 × y0..y1` in
/// data units, filled (in the theme palette's first color unless given a
/// fill or only a stroke) and optionally outlined.
pub struct Rect<X: Position, Y: Position> {
    rows: Vec<Row<X, Y>>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    fill: Option<Color>,
    stroke: Option<Color>,
    stroke_width: Option<f64>,
}

/// One rectangle: its two x edges and its two y edges, in data units.
type Row<X, Y> = ([<X as Kind>::Domain; 2], [<Y as Kind>::Domain; 2]);

impl<X: Position, Y: Position> std::fmt::Debug for Rect<X, Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Rect").finish_non_exhaustive()
    }
}

/// One filled rectangle per row `([x0, x1], [y0, y1])` on `x` and `y`.
/// All four edges enter fitting, and none is held. Rows over shared bin
/// edges stack histograms. The edges are values of the handles' domains,
/// so a span of time on a time handle is a pair of timestamps.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::Figure;
/// use rustd3_figure::mark::rect;
///
/// let mut fig = Figure::new();
/// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
/// fig.panel(0, 0).mark(rect([([0.0, 1.0], [0.0, 2.0]), ([1.0, 3.0], [1.0, 4.0])], x, y));
/// ```
///
/// A night shaded on a time axis:
///
/// ```
/// use rustd3_core::jiff::Timestamp;
/// use rustd3_core::{scale, time};
/// use rustd3_figure::Figure;
/// use rustd3_figure::mark::rect;
///
/// let night: [Timestamp; 2] = ["2026-10-24T18:00:00Z".parse()?, "2026-10-25T06:00:00Z".parse()?];
/// let mut fig = Figure::new();
/// let t = fig.fit(time::utc());
/// let kw = fig.scale(scale::linear().domain([0.0, 100.0]));
/// fig.panel(0, 0).mark(rect([(night, [0.0, 100.0])], t, kw));
/// # Ok::<(), rustd3_core::jiff::Error>(())
/// ```
pub fn rect<X, Y>(
    rows: impl IntoIterator<Item = Row<X, Y>>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> Rect<X, Y>
where
    X: Position,
    Y: Position,
{
    Rect {
        rows: rows.into_iter().collect(),
        x,
        y,
        fill: None,
        stroke: None,
        stroke_width: None,
    }
}

impl<X: Position, Y: Position> Rect<X, Y> {
    /// Fills the rectangles with `palette`'s color for `key`, looked up
    /// when the figure draws; a key the palette lacks draws the theme's
    /// gray.
    #[must_use]
    pub fn fill<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self {
        Self {
            fill: Some(Color::keyed(palette, key)),
            ..self
        }
    }

    /// Fills the rectangles with `c`, a tint no palette names.
    ///
    /// ```
    /// use rustd3_core::color::Rgb;
    /// use rustd3_core::scale;
    /// use rustd3_figure::Figure;
    /// use rustd3_figure::mark::rect;
    ///
    /// let mut fig = Figure::new();
    /// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    /// // A pale band behind the marks, framed in gray.
    /// let band = rect([([2.0, 4.0], [0.0, 10.0])], x, y)
    ///     .fill_color(Rgb::hex(0xeeeeee))
    ///     .stroke_color(Rgb::hex(0x949494));
    /// fig.panel(0, 0).mark(band);
    /// ```
    #[must_use]
    pub fn fill_color(self, c: Rgb) -> Self {
        Self {
            fill: Some(Color::Fixed(c)),
            ..self
        }
    }

    /// Outlines the rectangles in `palette`'s color for `key`, as wide as
    /// the theme's patch edges (1 pt, seaborn's) unless
    /// [`stroke_width`](Rect::stroke_width) says otherwise. As in
    /// Observable Plot, a stroked rectangle without [`fill`](Rect::fill)
    /// is not filled.
    #[must_use]
    pub fn stroke<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self {
        Self {
            stroke: Some(Color::keyed(palette, key)),
            ..self
        }
    }

    /// Outlines the rectangles in `c`, as [`stroke`](Rect::stroke) does
    /// in a palette's color: without [`fill`](Rect::fill) or
    /// [`fill_color`](Rect::fill_color) they are not filled.
    #[must_use]
    pub fn stroke_color(self, c: Rgb) -> Self {
        Self {
            stroke: Some(Color::Fixed(c)),
            ..self
        }
    }

    /// The outline's width in px; by default the theme's patch edge, 1 pt.
    /// The outline is centered on the rectangle's edges, so half of it lies
    /// outside. A width that is not finite and positive leaves it as it
    /// was.
    ///
    /// ```
    /// use rustd3_core::color::Rgb;
    /// use rustd3_core::scale;
    /// use rustd3_figure::Figure;
    /// use rustd3_figure::mark::rect;
    ///
    /// let mut fig = Figure::new();
    /// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    /// let ink = fig.scale(scale::ordinal(["picked"], [Rgb::hex(0xd62728)]));
    /// // A heatmap's picked cell, boxed 3 px wide.
    /// fig.panel(0, 0)
    ///     .mark(rect([([2.0, 3.0], [1.0, 2.0])], x, y).stroke(ink, "picked").stroke_width(3.0));
    /// ```
    #[must_use]
    pub fn stroke_width(self, px: f64) -> Self {
        Self {
            stroke_width: positive(px).or(self.stroke_width),
            ..self
        }
    }
}

impl<'a, X: Position, Y: Position> Mark<'a> for Rect<X, Y> {
    fn fit(&self, fits: &mut Fits) {
        fits.across(self.x);
        for ([x0, x1], [y0, y1]) in &self.rows {
            for v in [x0, x1] {
                fits.add(self.x, v);
            }
            for v in [y0, y1] {
                fits.add(self.y, v);
            }
        }
        for color in self.fill.iter().chain(&self.stroke) {
            color.report(Swatch::Square, None, Alpha::OPAQUE, fits);
        }
    }

    /// Per row whose edges are placed, a `Fill` command (and a `Stroke`
    /// when outlined) of the rectangle clipped to the guard band, its edges
    /// on device pixels on raster targets; nothing for a row with no area.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let (Some(sx), Some(sy)) = (ctx.x(self.x), ctx.y(self.y)) else {
            return;
        };
        let band = ctx.band();
        let fill = match (&self.fill, &self.stroke) {
            (Some(color), _) => Some(color),
            (None, Some(_)) => None,
            (None, None) => Some(&Color::Theme),
        };
        let fill = fill.map(|color| list::brush(color.resolve(ctx)));
        let stroke = self.stroke.as_ref().map(|c| list::brush(c.resolve(ctx)));
        let width = self
            .stroke_width
            .unwrap_or_else(|| ctx.get_theme().get_patch_width());
        for ([x0, x1], [y0, y1]) in &self.rows {
            let placed = (|| {
                let [x0, x1] = [x0, x1].map(|v| edge(&sx, v, [band.x0, band.x1]));
                let [y0, y1] = [y0, y1].map(|v| edge(&sy, v, [band.y0, band.y1]));
                Some(kurbo::Rect::new(x0?, y0?, x1?, y1?))
            })();
            let Some(r) = placed else {
                continue;
            };
            if let Some(paint) = &fill {
                self::fill(list, ctx, r, paint);
            }
            if let Some(paint) = &stroke {
                outline(list, ctx, r, (paint, width));
            }
        }
    }

    /// A fill or stroke key the palette lacks.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        for color in [&self.fill, &self.stroke].into_iter().flatten() {
            super::unknown(color, ctx, issues);
        }
    }
}

/// Where edge `v` lies along `scale`, clamped to `ends` (a guard band's
/// extent along it); `None` for a value the scale places nowhere. A number
/// is placed as bars place their values ([`number`]).
fn edge<S: Position>(scale: &S, v: &<S as Kind>::Domain, ends: [f64; 2]) -> Option<f64> {
    match (v as &dyn Any).downcast_ref::<f64>() {
        Some(v) => number(scale, *v, ends),
        None => at(scale, v).map(|p| p.clamp(ends[0], ends[1])),
    }
}

/// Where the number `v` lies along `scale`, whose domain is `f64`,
/// clamped to `[lo, hi]`. A value the scale's transform does not take (0
/// or less on a log scale) lies at the end of the guard band on the side
/// of the domain it lies beyond, as `mark::reach` places it. `None` for
/// NaN, or for a scale whose domain is not `f64`.
fn number<S: Position>(scale: &S, v: f64, [lo, hi]: [f64; 2]) -> Option<f64> {
    if v.is_nan() {
        return None;
    }
    let position = |v: f64| {
        let v = (&v as &dyn Any).downcast_ref::<<S as Kind>::Domain>()?;
        scale.position(v)
    };
    let p = position(v)?;
    if !p.is_nan() {
        return Some(p.clamp(lo, hi));
    }
    let ends = Kind::ends(scale)?;
    let [d0, d1] = *(&ends as &dyn Any).downcast_ref::<[f64; 2]>()?;
    let (low, high) = (d0.min(d1), d0.max(d1));
    let (beyond, other) = if v < low {
        (low, high)
    } else if v > high {
        (high, low)
    } else {
        return None;
    };
    let toward = position(beyond)? > position(other)?;
    Some(if toward { hi } else { lo })
}

/// Pushes `r`, a rectangle in the panel's coordinates, filled with `paint`
/// and clipped to the guard band of `ctx`'s plot area. On raster targets
/// its edges lie on device pixel boundaries. Pushes nothing when clipping
/// leaves no area. Returns the rectangle filled, if any.
pub(super) fn fill(
    list: &mut List,
    ctx: &Ctx<'_>,
    r: kurbo::Rect,
    paint: &Brush,
) -> Option<kurbo::Rect> {
    let band = ctx.band();
    let r = ctx.snap().rect(r.abs().intersect(band));
    (r.width() > 0.0 && r.height() > 0.0).then(|| {
        list.commands.push(Command::Fill {
            path: r.to_path(TOLERANCE),
            paint: paint.clone(),
        });
        r
    })
}

/// Pushes the outline of `r` stroked with `paint`, `width` px wide with
/// mitered corners, on whole device pixels on raster targets.
fn outline(list: &mut List, ctx: &Ctx<'_>, r: kurbo::Rect, (paint, width): (&Brush, f64)) {
    let band = ctx.band();
    let r = r.abs().intersect(band);
    let snap = ctx.snap();
    let at = |v: f64| snap.line(v, width);
    let r = kurbo::Rect::new(at(r.x0), at(r.y0), at(r.x1), at(r.y1));
    list.commands.push(Command::Stroke {
        path: r.to_path(TOLERANCE),
        style: kurbo::Stroke::new(snap.width(width)).with_join(Join::Miter),
        paint: paint.clone(),
    });
}
