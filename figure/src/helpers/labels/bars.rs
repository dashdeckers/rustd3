//! [`Bars`]: matplotlib's `bar_label`.

use kurbo::Point;
use rustd3_core::format::{Format, Specifier};
use rustd3_core::locale::Locale;

use crate::check::Issue;
use crate::figure::Fits;
use crate::helpers::labels::{Edge, PX_PER_PT, key, place};
use crate::list::List;
use crate::mark::{self, Ctx, Mark, guard};
use crate::scale::{Kind, Position, ScaleRef};

/// The value labels of bars (see [`bars`]).
pub struct Bars<X: Position, Y: Position<Domain = f64>> {
    rows: Vec<(<X as Kind>::Domain, f64)>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    format: Specifier,
    /// pt between a bar's end and its label.
    padding: f64,
    center: bool,
}

/// matplotlib's `bar_label` for bars of `data` drawn on `x` and `y` from 0
/// (as [`bar_y`](crate::mark::bar_y) draws them): each finite value,
/// written in the figure's locale, at its bar's end, centred across the
/// bar (the middle of its band on band scales), offset outward by the
/// padding (3 pt by default): above a bar that rises from 0, its line
/// box's bottom 3 pt over the end, and below one that falls, its line
/// box's top 3 pt under it; [`center`](Bars::center) puts it inside. The
/// labels enter no fitting and the y scale keeps the bars' margins, so a
/// label over the tallest bar may need more room than they give (as in
/// matplotlib).
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::Figure;
/// use rustd3_figure::helpers::labels;
/// use rustd3_figure::mark::bar_y;
///
/// let rows = [("a", 3.0), ("b", -1.0), ("c", 2.5)];
/// let mut fig = Figure::new();
/// let x = fig.scale(scale::band(["a", "b", "c"]));
/// let y = fig.fit(scale::linear());
/// fig.panel(0, 0)
///     .mark(bar_y(rows, x, y))
///     .mark(labels::bars(&rows, x, y).format(".1f".parse()?).padding(2.0));
/// # Ok::<(), rustd3_core::format::InvalidFormat>(())
/// ```
pub fn bars<X, Y>(data: &[(<X as Kind>::Domain, f64)], x: ScaleRef<X>, y: ScaleRef<Y>) -> Bars<X, Y>
where
    X: Position,
    Y: Position<Domain = f64>,
{
    Bars {
        rows: data.to_vec(),
        x,
        y,
        format: Specifier::default(),
        padding: 3.0,
        center: false,
    }
}

impl<X: Position, Y: Position<Domain = f64>> Bars<X, Y> {
    /// Each value written by `spec`, a core format specifier, in the
    /// figure's locale: `".1f".parse()?` writes 3 as "3.0". By default d3's
    /// `""` (up to 12 significant digits, trailing zeros trimmed), which
    /// writes what matplotlib's default `%g` writes for values of six
    /// significant digits or fewer.
    pub fn format(self, spec: Specifier) -> Self {
        Self {
            format: spec,
            ..self
        }
    }

    /// The distance between a bar's end and its label, pt (3 by default,
    /// not scaled by the theme's context, as matplotlib's `padding`). A
    /// distance that is not finite leaves it as it was.
    pub fn padding(self, pt: f64) -> Self {
        Self {
            padding: if pt.is_finite() { pt } else { self.padding },
            ..self
        }
    }

    /// Each label inside its bar, its line box centred halfway between the
    /// bar's base and its end (matplotlib's `label_type="center"`).
    pub fn center(self) -> Self {
        Self {
            center: true,
            ..self
        }
    }

    /// The formatter of the labels under `locale`.
    fn formatter(&self, locale: &Locale) -> Format {
        Format::new(locale.number(), self.format.clone())
    }
}

impl<'a, X: Position, Y: Position<Domain = f64>> Mark<'a> for Bars<X, Y> {
    /// Reports the labels to be shaped; no values.
    fn fit(&self, fits: &mut Fits) {
        fits.across(self.x);
        let format = self.formatter(fits.locale());
        for (_, v) in self.rows.iter().filter(|(_, v)| v.is_finite()) {
            let label = key(format.format(*v), fits.theme());
            fits.label(label);
        }
    }

    /// Each finite value's label at its bar's end (or middle), when its key
    /// is placed and the point lies within the guard band around the plot
    /// area.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let (sx, sy) = (ctx.x(self.x), ctx.y(self.y));
        let band = guard::band(ctx.rect().width(), ctx.rect().height());
        let format = self.formatter(ctx.locale());
        let pad = self.padding * PX_PER_PT;
        for (k, v) in self.rows.iter().filter(|(_, v)| v.is_finite()) {
            let x = sx.position(k).map(|p| p + sx.bandwidth() / 2.0);
            let end = sy.position(v);
            let base = mark::reach(&sy, 0.0, [band.y0, band.y1]);
            let (Some(x), Some(end), Some(base)) = (x, end, base) else {
                continue;
            };
            let (y, edge) = if self.center {
                ((base + end) / 2.0, Edge::Middle)
            } else if end <= base {
                (end - pad, Edge::Bottom)
            } else {
                (end + pad, Edge::Top)
            };
            let at = Point::new(x, y);
            if at.is_finite() && mark::within(band, at) {
                place(&key(format.format(*v), ctx.theme()), at, edge, ctx, list);
            }
        }
    }

    /// Keys outside a band or point domain, which draw nothing.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        let sx = ctx.x(self.x);
        let count = self
            .rows
            .iter()
            .filter(|(k, _)| sx.position(k).is_none())
            .count();
        if count > 0 {
            issues.push(Issue::OutsideDomain {
                handle: self.x.index(),
                count,
            });
        }
    }
}
