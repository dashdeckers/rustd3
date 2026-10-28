//! [`Rect`]: one rectangle per row in data units, and the rectangle code
//! bars and histograms draw through.

use kurbo::{Join, Shape};
use peniko::Brush;

use crate::check::Issue;
use crate::figure::Fits;
use crate::list::{self, Command, List};
use crate::mark::color::Color;
use crate::mark::{Ctx, Mark, guard, reach};
use crate::scale::{Paint, Position, ScaleRef};

/// How closely a rectangle's path follows it, in px: exact, as it has no
/// curves.
const TOLERANCE: f64 = 0.1;

/// Observable Plot's `rect`: one rectangle per row, `x0..x1 × y0..y1` in
/// data units, filled (in the theme palette's first color unless given a
/// fill or only a stroke) and optionally outlined.
pub struct Rect<X: Position<Domain = f64>, Y: Position<Domain = f64>> {
    rows: Vec<([f64; 2], [f64; 2])>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    fill: Option<Color>,
    stroke: Option<Color>,
}

/// One filled rectangle per row `([x0, x1], [y0, y1])` on `x` and `y`:
/// all four edges enter fitting, none is held. Rows over shared bin edges
/// stack histograms.
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
pub fn rect<X, Y>(
    rows: impl IntoIterator<Item = ([f64; 2], [f64; 2])>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> Rect<X, Y>
where
    X: Position<Domain = f64>,
    Y: Position<Domain = f64>,
{
    Rect {
        rows: rows.into_iter().collect(),
        x,
        y,
        fill: None,
        stroke: None,
    }
}

impl<X: Position<Domain = f64>, Y: Position<Domain = f64>> Rect<X, Y> {
    /// Fills the rectangles with `palette`'s color for `key`, looked up
    /// when the figure draws; a key the palette lacks draws the theme's
    /// gray.
    pub fn fill<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self {
        Self {
            fill: Some(Color::keyed(palette, key)),
            ..self
        }
    }

    /// Outlines the rectangles in `palette`'s color for `key`, 1 pt wide
    /// (seaborn's patch edges). As in Observable Plot, a stroked rectangle
    /// without [`fill`](Rect::fill) is not filled.
    pub fn stroke<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self {
        Self {
            stroke: Some(Color::keyed(palette, key)),
            ..self
        }
    }
}

impl<'a, X: Position<Domain = f64>, Y: Position<Domain = f64>> Mark<'a> for Rect<X, Y> {
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
    }

    /// Per row whose edges are placed, a `Fill` command (and a `Stroke`
    /// when outlined) of the rectangle clipped to the guard band, its edges
    /// on device pixels on raster targets; nothing for a row with no area.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let (sx, sy) = (ctx.x(self.x), ctx.y(self.y));
        let band = guard::band(ctx.rect().width(), ctx.rect().height());
        let fill = match (&self.fill, &self.stroke) {
            (Some(color), _) => Some(color),
            (None, Some(_)) => None,
            (None, None) => Some(&Color::Theme),
        };
        let fill = fill.map(|color| list::brush(color.resolve(ctx)));
        let stroke = self.stroke.as_ref().map(|c| list::brush(c.resolve(ctx)));
        for ([x0, x1], [y0, y1]) in &self.rows {
            let placed = (|| {
                let [x0, x1] = [x0, x1].map(|&v| reach(&sx, v, [band.x0, band.x1]));
                let [y0, y1] = [y0, y1].map(|&v| reach(&sy, v, [band.y0, band.y1]));
                Some(kurbo::Rect::new(x0?, y0?, x1?, y1?))
            })();
            let Some(r) = placed else {
                continue;
            };
            if let Some(paint) = &fill {
                self::fill(list, ctx, r, paint);
            }
            if let Some(paint) = &stroke {
                outline(list, ctx, r, paint);
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

/// Pushes `r`, a rectangle in the panel's coordinates, filled with `paint`:
/// clipped to the guard band of `ctx`'s plot area, its edges on device
/// pixel boundaries on raster targets; nothing when that leaves no area.
/// The rectangle filled, if any.
pub(super) fn fill(
    list: &mut List,
    ctx: &Ctx<'_>,
    r: kurbo::Rect,
    paint: &Brush,
) -> Option<kurbo::Rect> {
    let band = guard::band(ctx.rect().width(), ctx.rect().height());
    let r = ctx.snap().rect(r.abs().intersect(band));
    (r.width() > 0.0 && r.height() > 0.0).then(|| {
        list.commands.push(Command::Fill {
            path: r.to_path(TOLERANCE),
            paint: paint.clone(),
        });
        r
    })
}

/// Pushes the outline of `r` stroked with `paint`, the theme's patch edge
/// wide with mitred corners, on whole device pixels on raster targets.
fn outline(list: &mut List, ctx: &Ctx<'_>, r: kurbo::Rect, paint: &Brush) {
    let band = guard::band(ctx.rect().width(), ctx.rect().height());
    let r = r.abs().intersect(band);
    let (snap, width) = (ctx.snap(), ctx.theme().patch_width());
    let at = |v: f64| snap.line(v, width);
    let r = kurbo::Rect::new(at(r.x0), at(r.y0), at(r.x1), at(r.y1));
    list.commands.push(Command::Stroke {
        path: r.to_path(TOLERANCE),
        style: kurbo::Stroke::new(snap.width(width)).with_join(Join::Miter),
        paint: paint.clone(),
    });
}
