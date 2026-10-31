//! [`Ecdf`]: the empirical cumulative distribution of values, drawn
//! through [`line`](crate::mark::line).

use crate::check::Issue;
use crate::figure::Fits;
use crate::helpers::stats::keyed::{Keyed, keyed};
use crate::list::List;
use crate::mark::{self, Ctx, Mark};
use crate::scale::{Paint, Position, ScaleRef};

/// The empirical cumulative distribution of values (see [`ecdf`]).
pub struct Ecdf<X: Position<Domain = f64>, Y: Position<Domain = f64>> {
    /// The finite values, ascending.
    sorted: Vec<f64>,
    complementary: bool,
    color: Option<Box<dyn Keyed<X, Y>>>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
}

/// The empirical cumulative distribution of the finite `values` on `x`
/// and `y`, matplotlib's `Axes.ecdf`: a step up by 1/n at each sorted
/// value, from 0 at the least to 1 at the greatest (each value's step
/// ends at `(value, i/n)`), drawn as a [`line`](crate::mark::line) in
/// the theme palette's first color. y reports 0 and 1 and holds both, as
/// matplotlib's sticky edges do; x reports the least and greatest value.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::Figure;
/// use rustd3_figure::helpers::stats::ecdf;
///
/// let mut fig = Figure::new();
/// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
/// let values = [1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 4.7, 5.1, 14.0];
/// fig.panel(0, 0)
///     .mark(ecdf(values, x, y))
///     .mark(ecdf(values, x, y).complementary(true));
/// ```
pub fn ecdf<X, Y>(
    values: impl IntoIterator<Item = f64>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> Ecdf<X, Y>
where
    X: Position<Domain = f64>,
    Y: Position<Domain = f64>,
{
    let mut sorted: Vec<f64> = values.into_iter().filter(|v| v.is_finite()).collect();
    sorted.sort_by(f64::total_cmp);
    Ecdf {
        sorted,
        complementary: false,
        color: None,
        x,
        y,
    }
}

impl<X: Position<Domain = f64>, Y: Position<Domain = f64>> Ecdf<X, Y> {
    /// The complementary distribution, 1 − ECDF: a step down by 1/n at
    /// each sorted value, from 1 to 0.
    #[must_use]
    pub fn complementary(self, yes: bool) -> Self {
        Self {
            complementary: yes,
            ..self
        }
    }

    /// Strokes the steps with `palette`'s color for `key`, looked up when
    /// the figure draws; a key the palette lacks draws the theme's gray.
    #[must_use]
    pub fn stroke<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self {
        Self {
            color: Some(keyed(palette, key)),
            ..self
        }
    }

    /// The line the steps are drawn as: per sorted value `x_i`, its rise
    /// from `(x_i, i/n)` to `(x_i, (i + 1)/n)` (falls from `1 − i/n` when
    /// complementary), joined by the flat runs between values.
    fn steps(&self) -> mark::Line<'static, X, Y> {
        let n = self.sorted.len() as f64;
        let level = |k: usize| {
            let share = k as f64 / n;
            if self.complementary {
                1.0 - share
            } else {
                share
            }
        };
        let points: Vec<(f64, f64)> = self
            .sorted
            .iter()
            .enumerate()
            .flat_map(|(i, &v)| [(v, level(i)), (v, level(i + 1))])
            .collect();
        let line = mark::line(points, self.x, self.y);
        match &self.color {
            Some(color) => color.line(line),
            None => line,
        }
    }
}

impl<'a, X: Position<Domain = f64>, Y: Position<Domain = f64>> Mark<'a> for Ecdf<X, Y> {
    fn fit(&self, fits: &mut Fits) {
        if let (Some(first), Some(last)) = (self.sorted.first(), self.sorted.last()) {
            fits.add(self.x, first);
            fits.add(self.x, last);
            for v in [0.0, 1.0] {
                fits.add(self.y, &v);
                fits.stick(self.y, v);
            }
        }
    }

    /// The steps as `line` draws them; nothing without finite values.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        if !self.sorted.is_empty() {
            self.steps().draw(ctx, list);
        }
    }

    /// A stroke key the palette lacks.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        self.steps().issues(ctx, issues);
    }
}
