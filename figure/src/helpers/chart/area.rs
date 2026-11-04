//! [`Area`]: a stacked area chart.

use std::marker::PhantomData;

use rustd3_core::scale;
use rustd3_core::shape;

use crate::Figure;
use crate::helpers::chart::setup::{self, Setup};
use crate::legend::{self, Slot};
use crate::mark::ribbon;

/// A stacked area chart: one ribbon per column stacked from 0 (see
/// [`area`]).
pub struct Area<'a> {
    /// One series per column, each a value per x.
    columns: Vec<Vec<f64>>,
    /// The columns' names, in order; columns past them keep their index.
    keys: Vec<String>,
    x: Vec<f64>,
    setup: Setup,
    /// The figure it ends in: `Figure<'a>`.
    figure: PhantomData<&'a ()>,
}

impl<'a> std::fmt::Debug for Area<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Area").finish_non_exhaustive()
    }
}

/// A stacked area chart of `columns` over `x`, each column a series with a
/// value per x, stacked from 0 in the columns' order by `shape::stack`
/// (offset none; matplotlib's `stackplot`): one opaque [`ribbon`] per
/// column between its stacked edges, in its key's color, on linear x and
/// y scales fitted to them, a bottom and a left axis with minor ticks, and a
/// legend right of the plot area when there is more than one column.
/// Columns are keyed by their index ("0", "1", …, pandas' names for
/// unnamed columns) until [`keys`](Area::keys) names them; keys are the
/// legend's labels. Rows are the shortest of `x` and the columns. The
/// figure's one panel holds one ribbon per column, in their order: mark *i*
/// is column *i*, so a mark index the figure reports (a `check` issue's
/// `mark`) is named from the app's own keys, `keys[mark]`. Ribbons report
/// no hit points, so an area chart is not hovered.
///
/// ```
/// use rustd3_figure::helpers::chart;
///
/// let x: Vec<f64> = (0..12).map(f64::from).collect();
/// let solar = x.iter().map(|m| 2.0 + (m * 0.5).sin()).collect();
/// let wind = x.iter().map(|m| 3.0 - 0.1 * m).collect();
/// let fig = chart::area(vec![solar, wind], x).y_label("GW").figure();
/// ```
#[must_use]
pub fn area<'a>(columns: Vec<Vec<f64>>, x: Vec<f64>) -> Area<'a> {
    Area {
        columns,
        keys: Vec::new(),
        x,
        setup: Setup::default(),
        figure: PhantomData,
    }
}

impl<'a> Area<'a> {
    /// Names the columns in order, in place of their indices: a column past
    /// the names keeps its index, a name past the columns is unused. A key
    /// given twice is an
    /// [`Issue::DuplicateKey`](crate::check::Issue::DuplicateKey) that
    /// [`Figure::check`] reports; its columns share a color and a legend
    /// entry.
    ///
    /// ```
    /// use rustd3_figure::helpers::chart;
    ///
    /// let x: Vec<f64> = (0..12).map(f64::from).collect();
    /// let solar = x.iter().map(|m| 2.0 + (m * 0.5).sin()).collect();
    /// let wind = x.iter().map(|m| 3.0 - 0.1 * m).collect();
    /// let fig = chart::area(vec![solar, wind], x).keys(["solar", "wind"]).figure();
    /// ```
    #[must_use]
    pub fn keys(self, names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Area {
            keys: names.into_iter().map(Into::into).collect(),
            ..self
        }
    }

    /// The figure: one panel with the ribbons, its axes and title, and the
    /// legend when there is more than one column.
    pub fn figure(self) -> Figure<'a> {
        let Area {
            columns,
            keys: names,
            x,
            setup,
            ..
        } = self;
        let mut fig = setup.figure();
        let keys: Vec<String> = (0..columns.len())
            .map(|i| names.get(i).cloned().unwrap_or_else(|| i.to_string()))
            .collect();
        setup::duplicates(&mut fig, &keys);
        let (sx, sy) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
        let color = fig.scale(setup.palette(keys.iter().cloned()));
        let columns: Vec<&[f64]> = columns.iter().map(Vec::as_slice).collect();
        let panel = fig.panel(0, 0);
        for (layer, key) in shape::stack(&columns).iter().zip(&keys) {
            let rows = x.iter().zip(layer).map(|(x, [y0, y1])| (*x, *y0, *y1));
            panel.mark(ribbon(rows, sx, sy).fill(color, key.clone()).opacity(1.0));
        }
        setup.axes(panel, sx, sy);
        if keys.len() > 1 {
            fig.legend(Slot::Right, legend::legend(color));
        }
        fig
    }
}

finish!(impl['a] Area<'a>);
