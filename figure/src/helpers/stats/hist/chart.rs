//! [`Hist`]: a histogram with matplotlib's options, drawn through
//! [`rect`](crate::mark::rect) or [`line`](crate::mark::line).

use crate::check::Issue;
use crate::figure::Fits;
use crate::helpers::stats::hist::{Bins, Error, Table, default_bins, table};
use crate::helpers::stats::keyed::{Keyed, keyed};
use crate::list::List;
use crate::mark::{self, Ctx, Mark};
use crate::scale::{Paint, Position, ScaleRef};

/// A histogram of numbers with matplotlib's options (see [`hist`]).
pub struct Hist<X: Position<Domain = f64>, Y: Position<Domain = f64>> {
    values: Vec<f64>,
    weights: Option<Vec<f64>>,
    bins: Option<Bins>,
    density: bool,
    cumulative: bool,
    step: bool,
    /// The bins of the options above.
    table: Result<Table, Error>,
    color: Option<Box<dyn Keyed<X, Y>>>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
}

impl<X: Position<Domain = f64>, Y: Position<Domain = f64>> std::fmt::Debug for Hist<X, Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hist").finish_non_exhaustive()
    }
}

/// What a histogram draws through: its bars or its outline.
enum Drawn<X: Position<Domain = f64>, Y: Position<Domain = f64>> {
    Bars(mark::Rect<X, Y>),
    Outline(mark::Line<'static, X, Y>),
}

/// A histogram of `values` on `x` with its heights on `y`: the options
/// layer over [`mark::histogram`](crate::mark::histogram), matplotlib's
/// `hist`. Its bins are [`hist::table`](super::table)'s for the options
/// set (by default d3's nice bins with Sturges' count, as `histogram`'s),
/// drawn as one [`rect`](crate::mark::rect) per bin from 0, in the theme
/// palette's first color, or as their outline. The bars stand on 0, which
/// is held: the y margin stops there. A table that cannot be made (edges
/// that do not increase, weights of another length) draws nothing, and
/// [`Figure::check`](crate::Figure::check) reports it.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::Figure;
/// use rustd3_figure::helpers::stats::{hist, hist::Bins};
///
/// let mut fig = Figure::new();
/// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
/// let values = [1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 4.7, 5.1, 14.0];
/// fig.panel(0, 0)
///     .mark(hist(values, x, y).bins(Bins::Auto).density(true))
///     .mark(hist(values, x, y).bins(Bins::Auto).density(true).cumulative(true).step(true));
/// ```
pub fn hist<X, Y>(
    values: impl IntoIterator<Item = f64>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> Hist<X, Y>
where
    X: Position<Domain = f64>,
    Y: Position<Domain = f64>,
{
    Hist {
        values: values.into_iter().collect(),
        weights: None,
        bins: None,
        density: false,
        cumulative: false,
        step: false,
        table: Ok(Table {
            edges: Vec::new(),
            counts: Vec::new(),
        }),
        color: None,
        x,
        y,
    }
    .tabled()
}

impl<X: Position<Domain = f64>, Y: Position<Domain = f64>> Hist<X, Y> {
    /// How the values are binned.
    #[must_use]
    pub fn bins(self, b: Bins) -> Self {
        Self {
            bins: Some(b),
            ..self
        }
        .tabled()
    }

    /// Each value's weight, in the values' order: bins sum weights instead
    /// of counting. Their count must be the values' (non-finite values
    /// included), or the histogram draws nothing.
    #[must_use]
    pub fn weights(self, w: impl IntoIterator<Item = f64>) -> Self {
        Self {
            weights: Some(w.into_iter().collect()),
            ..self
        }
        .tabled()
    }

    /// Heights as densities: count / (total weight · width), so the bars'
    /// area sums to 1.
    #[must_use]
    pub fn density(self, yes: bool) -> Self {
        Self {
            density: yes,
            ..self
        }
        .tabled()
    }

    /// Heights as running sums from the first bin; with `density`, the last
    /// is 1.
    #[must_use]
    pub fn cumulative(self, yes: bool) -> Self {
        Self {
            cumulative: yes,
            ..self
        }
        .tabled()
    }

    /// The outline only, matplotlib's `histtype="step"`: one line up from
    /// 0 at the first edge, along the tops and down to 0 at the last, in
    /// the theme's line width.
    #[must_use]
    pub fn step(self, yes: bool) -> Self {
        Self { step: yes, ..self }
    }

    /// Fills the bars (or strokes the outline) with `palette`'s color for
    /// `key`, looked up when the figure draws; a key the palette lacks
    /// draws the theme's gray.
    #[must_use]
    pub fn fill<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self {
        Self {
            color: Some(keyed(palette, key)),
            ..self
        }
    }

    /// The same with its table made for its options.
    fn tabled(self) -> Self {
        let finite = self.values.iter().filter(|v| v.is_finite()).count();
        let bins = self.bins.clone().unwrap_or_else(|| default_bins(finite));
        let table = table(
            &self.values,
            self.weights.as_deref(),
            &bins,
            self.density,
            self.cumulative,
        );
        Self { table, ..self }
    }

    /// The mark the table draws through; `None` when it could not be made.
    fn drawn(&self) -> Option<Drawn<X, Y>> {
        let t = self.table.as_ref().ok()?;
        let bins = t.edges.windows(2).zip(&t.counts);
        Some(if self.step {
            let first = t.edges.first().map(|&e| (e, 0.0));
            let tops = bins.flat_map(|(e, &c)| [(e[0], c), (e[1], c)]);
            let last = t.edges.last().map(|&e| (e, 0.0));
            let points: Vec<(f64, f64)> = first.into_iter().chain(tops).chain(last).collect();
            let line = mark::line(points, self.x, self.y);
            Drawn::Outline(match &self.color {
                Some(color) => color.line(line),
                None => line,
            })
        } else {
            let rows: Vec<([f64; 2], [f64; 2])> =
                bins.map(|(e, &c)| ([e[0], e[1]], [0.0, c])).collect();
            let rect = mark::rect(rows, self.x, self.y);
            Drawn::Bars(match &self.color {
                Some(color) => color.rect(rect),
                None => rect,
            })
        })
    }
}

impl<'a, X: Position<Domain = f64>, Y: Position<Domain = f64>> Mark<'a> for Hist<X, Y> {
    /// The edges on x, the heights and 0 on y, held at 0; nothing when the
    /// table could not be made.
    fn fit(&self, fits: &mut Fits) {
        if let Ok(t) = &self.table {
            for e in &t.edges {
                fits.add(self.x, e);
            }
            for c in t.counts.iter().chain([&0.0]) {
                fits.add(self.y, c);
            }
            fits.stick(self.y, 0.0);
        }
    }

    /// The bars as `rect` draws them (a bin with no height draws nothing),
    /// or the outline as `line` draws it.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        match self.drawn() {
            Some(Drawn::Bars(m)) => m.draw(ctx, list),
            Some(Drawn::Outline(m)) => m.draw(ctx, list),
            None => {}
        }
    }

    /// A table that could not be made; a fill key the palette lacks.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        if let Err(e) = &self.table {
            issues.push(Issue::Mark {
                panel: ctx.panel(),
                mark: ctx.mark(),
                error: e.clone().into(),
            });
        }
        match self.drawn() {
            Some(Drawn::Bars(m)) => m.issues(ctx, issues),
            Some(Drawn::Outline(m)) => m.issues(ctx, issues),
            None => {}
        }
    }
}
