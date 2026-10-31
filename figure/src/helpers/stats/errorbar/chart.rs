//! [`ErrorBar`]: a vertical bar per key from its value's lower to its
//! upper error.

use kurbo::{BezPath, Point};

use crate::check::Issue;
use crate::figure::Fits;
use crate::list::{self, Command, List};
use crate::mark::{Ctx, Mark, guard};
use crate::scale::{Kind, Position, ScaleRef};

/// px per pt: caps are measured as matplotlib's `capsize`, in points not
/// scaled by the theme's context.
const PX_PER_PT: f64 = 4.0 / 3.0;

/// Error bars: per row, a vertical bar at its key from `value − below` to
/// `value + above` (see [`errorbar`]).
pub struct ErrorBar<X: Position, Y: Position<Domain = f64>> {
    rows: Vec<(<X as Kind>::Domain, f64, super::Err)>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    /// How far the caps reach to each side, pt.
    caps: Option<f64>,
}

/// One error bar per row `(key, value, error)` on `x` and `y`: a vertical
/// line at the key (the middle of its band) from `value − below` to
/// `value + above`, in ink and the theme's line width (matplotlib's
/// `bar(yerr=…)` draws them black, seaborn dark gray). No caps by default,
/// matplotlib's `capsize` 0 and seaborn's; [`caps`](ErrorBar::caps) adds
/// them. The bars' ends enter fitting; a row whose error is negative or
/// not finite draws nothing.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::Figure;
/// use rustd3_figure::helpers::stats::errorbar;
/// use rustd3_figure::mark::bar_y;
///
/// let mut fig = Figure::new();
/// let x = fig.scale(scale::band(["a", "b"]).padding(0.2));
/// let y = fig.fit(scale::linear());
/// fig.panel(0, 0)
///     .mark(bar_y([("a", 3.0), ("b", 4.5)], x, y))
///     .mark(
///         errorbar(
///             [("a", 3.0, errorbar::Err::Sym(0.4)), ("b", 4.5, errorbar::Err::Asym(0.2, 0.8))],
///             x,
///             y,
///         )
///         .caps(4.0),
///     );
/// ```
pub fn errorbar<X, Y>(
    rows: impl IntoIterator<Item = (<X as Kind>::Domain, f64, super::Err)>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> ErrorBar<X, Y>
where
    X: Position,
    Y: Position<Domain = f64>,
{
    ErrorBar {
        rows: rows.into_iter().collect(),
        x,
        y,
        caps: None,
    }
}

impl<X: Position, Y: Position<Domain = f64>> ErrorBar<X, Y> {
    /// Caps reaching `pt` points to each side of each bar, matplotlib's
    /// `capsize`. A size that is not finite and positive leaves them as
    /// they were.
    #[must_use]
    pub fn caps(self, pt: f64) -> Self {
        let caps = if pt.is_finite() && pt > 0.0 {
            Some(pt)
        } else {
            self.caps
        };
        Self { caps, ..self }
    }
}

impl<'a, X: Position, Y: Position<Domain = f64>> Mark<'a> for ErrorBar<X, Y> {
    fn fit(&self, fits: &mut Fits) {
        for (key, value, error) in &self.rows {
            fits.add(self.x, key);
            for v in error.ends(*value).into_iter().flatten() {
                fits.add(self.y, &v);
            }
        }
    }

    /// One `Stroke` command of every bar and cap, clamped to the guard band
    /// and, on raster targets, on whole device pixels.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let (Some(sx), Some(sy)) = (ctx.x(self.x), ctx.y(self.y)) else {
            return;
        };
        let band = guard::band(ctx.rect().width(), ctx.rect().height());
        let (snap, width) = (ctx.snap(), ctx.theme().get_line_width());
        let at = |v: f64| sy.position(&v).filter(|p| p.is_finite());
        let mut path = BezPath::new();
        for (key, value, error) in &self.rows {
            let centre = sx
                .position(key)
                .map(|p| p + sx.bandwidth() / 2.0)
                .filter(|p| p.is_finite());
            let Some(ends) = error.ends(*value) else {
                continue;
            };
            let (Some(centre), [Some(low), Some(high)]) = (centre, ends.map(at)) else {
                continue;
            };
            let x = snap.line(centre, width).clamp(band.x0, band.x1);
            let [low, high] = [low, high].map(|y| y.clamp(band.y0, band.y1));
            path.move_to((x, low));
            path.line_to((x, high));
            if let Some(pt) = self.caps {
                let reach = pt * PX_PER_PT;
                for y in [low, high] {
                    let y = snap.line(y, width);
                    path.move_to(Point::new((centre - reach).max(band.x0), y));
                    path.line_to(Point::new((centre + reach).min(band.x1), y));
                }
            }
        }
        if !path.elements().is_empty() {
            list.commands.push(Command::Stroke {
                path,
                style: list::stroke(snap.width(width)),
                paint: list::brush(ctx.theme().get_ink()),
            });
        }
    }

    /// Keys outside a band or point domain.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        let Some(sx) = ctx.raw(self.x) else {
            return;
        };
        let outside = self
            .rows
            .iter()
            .filter(|(k, _, _)| sx.position(k).is_none())
            .count();
        if outside > 0 {
            issues.push(Issue::OutsideDomain {
                handle: self.x.index(),
                count: outside,
            });
        }
    }
}
