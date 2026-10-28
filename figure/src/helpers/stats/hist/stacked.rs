//! [`Stacked`]: histograms of groups stacked over one set of edges.

use rustd3_core::shape;

use crate::Panel;
use crate::helpers::stats::hist::chart::hist;
use crate::helpers::stats::hist::{Bins, Error, default_bins, table};
use crate::mark::rect;
use crate::scale::{Paint, Position, ScaleRef};

/// One group's rows: `([edge_i, edge_{i+1}], [y0, y1])` per bin.
type Rows = Vec<([f64; 2], [f64; 2])>;

/// Histograms of groups stacked over one set of edges, one
/// [`rect`](crate::mark::rect) mark per group (see [`stacked`]).
pub struct Stacked<K, X, Y, C>
where
    C: Paint<Domain = K>,
    X: Position<Domain = f64>,
    Y: Position<Domain = f64>,
{
    groups: Vec<(K, Vec<f64>)>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    color: ScaleRef<C>,
    bins: Option<Bins>,
    density: bool,
}

/// Groups `(key, values)` stacked over one set of edges, matplotlib's
/// `hist(…, stacked=True)`: the edges come from the union of every group's
/// finite values (d3's nice bins with Sturges' count by default), each
/// group is counted per bin, the count columns are stacked with core's
/// [`shape::stack()`] (the groups in their order from the bottom, no offset)
/// and each group becomes one [`rect`](crate::mark::rect) mark with rows
/// `([edge_i, edge_{i+1}], [y0, y1])`, filled with `color`'s color for its
/// key. As rects, the bars fit both of their ends and hold nothing.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_core::scheme::COLORBLIND;
/// use rustd3_figure::Figure;
/// use rustd3_figure::helpers::stats::hist::{self, Bins};
///
/// let mut fig = Figure::new();
/// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
/// let color = fig.scale(scale::ordinal(["a", "b"], COLORBLIND));
/// let groups = [("a", vec![1.0, 2.0, 2.0, 3.0]), ("b", vec![2.0, 3.0, 3.0])];
/// hist::stacked(groups, x, y, color)
///     .bins(Bins::Edges(vec![0.5, 1.5, 2.5, 3.5]))
///     .add_to(fig.panel(0, 0));
/// ```
pub fn stacked<K, X, Y, C>(
    groups: impl IntoIterator<Item = (K, Vec<f64>)>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    color: ScaleRef<C>,
) -> Stacked<K, X, Y, C>
where
    C: Paint<Domain = K>,
    X: Position<Domain = f64>,
    Y: Position<Domain = f64>,
{
    Stacked {
        groups: groups.into_iter().collect(),
        x,
        y,
        color,
        bins: None,
        density: false,
    }
}

impl<K, X, Y, C> Stacked<K, X, Y, C>
where
    C: Paint<Domain = K>,
    X: Position<Domain = f64>,
    Y: Position<Domain = f64>,
{
    /// How the union of the values is binned.
    pub fn bins(self, b: Bins) -> Self {
        Self {
            bins: Some(b),
            ..self
        }
    }

    /// Heights as densities over all groups (matplotlib's `stacked=True,
    /// density=True`): counts divided by the total count of every group and
    /// the bin's width, so the stack's area sums to 1.
    pub fn density(self, yes: bool) -> Self {
        Self {
            density: yes,
            ..self
        }
    }

    /// Adds one `rect` mark per group to `panel`, bottom group first. Edges
    /// that cannot be used add instead one [`hist`](super::super::hist()) of
    /// the union, which draws nothing and reports the error to
    /// [`Figure::check`](crate::Figure::check).
    pub fn add_to(self, panel: &mut Panel<'_>) {
        let union: Vec<f64> = self
            .groups
            .iter()
            .flat_map(|(_, values)| values.iter().copied())
            .collect();
        let finite = union.iter().filter(|v| v.is_finite()).count();
        let bins = self.bins.clone().unwrap_or_else(|| default_bins(finite));
        match self.layers(&union, &bins) {
            Ok(layers) => {
                for ((key, _), rows) in self.groups.into_iter().zip(layers) {
                    panel.mark(rect(rows, self.x, self.y).fill(self.color, key));
                }
            }
            Err(_) => {
                panel.mark(hist(union, self.x, self.y).bins(bins));
            }
        }
    }

    /// Per group, its rows over the edges of the `union` of the values by
    /// `bins`.
    fn layers(&self, union: &[f64], bins: &Bins) -> Result<Vec<Rows>, Error> {
        let edges = table(union, None, bins, false, false)?.edges;
        let shared = Bins::Edges(edges.clone());
        let mut columns = self
            .groups
            .iter()
            .map(|(_, values)| Ok(table(values, None, &shared, false, false)?.counts))
            .collect::<Result<Vec<Vec<f64>>, Error>>()?;
        if self.density {
            let total: f64 = columns.iter().flatten().sum();
            let widths: Vec<f64> = edges.windows(2).map(|e| e[1] - e[0]).collect();
            for column in &mut columns {
                for (c, w) in column.iter_mut().zip(&widths) {
                    *c /= total * w;
                }
            }
        }
        let columns: Vec<&[f64]> = columns.iter().map(Vec::as_slice).collect();
        let rows = shape::stack(&columns).into_iter().map(|layer| {
            let bins = edges.windows(2).map(|e| [e[0], e[1]]);
            bins.zip(layer).collect()
        });
        Ok(rows.collect())
    }
}
