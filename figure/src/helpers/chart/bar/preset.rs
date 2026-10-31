//! [`Bar`]: bars of one or more series per key.

use std::fmt::Display;

use rustd3_core::scale;
use rustd3_core::shape::{self, stack};

use crate::Figure;
use crate::helpers::chart::bar::Mode;
use crate::helpers::chart::setup::Setup;
use crate::legend::{self, Slot};
use crate::mark::{bar_y, bar_y_spans};

/// The share of a key's step left between neighbouring bands: seaborn's
/// bar width of 0.8, half the gap outside the first and last band.
const GAP: f64 = 0.2;

/// A bar chart: per key, one bar per series, grouped, stacked or diverging
/// (see [`bar`]).
pub struct Bar<K> {
    /// Each key and its values, one per series.
    rows: Vec<(K, Vec<f64>)>,
    series: Vec<String>,
    mode: Mode,
    setup: Setup,
}

/// A bar chart of `rows`, each a key and its values in the order of
/// `series`, the series' names: the keys on a band scale whose bars take
/// 0.8 of each step (seaborn's), a fitted linear y, each series in its
/// name's color, the bars [`Grouped`](Mode::Grouped) by default, a bottom
/// and a left axis (minor ticks on y), and a legend right of the plot area
/// when there is more than one series. A row with fewer values than series
/// has no bar (NaN) for the rest.
///
/// ```
/// use rustd3_figure::helpers::chart;
///
/// let fig = chart::bar(
///     [("mon", vec![3.0, 2.5]), ("tue", vec![4.0, 1.0]), ("wed", vec![2.0, 3.5])],
///     vec!["north".into(), "south".into()],
/// )
/// .y_label("orders")
/// .figure();
/// ```
pub fn bar<K>(rows: impl IntoIterator<Item = (K, Vec<f64>)>, series: Vec<String>) -> Bar<K> {
    Bar {
        rows: rows.into_iter().collect(),
        series,
        mode: Mode::default(),
        setup: Setup::default(),
    }
}

impl<K> Bar<K> {
    /// How the series' bars stand at each key: side by side (the default),
    /// stacked, or diverging from 0.
    ///
    /// ```
    /// use rustd3_figure::helpers::chart::{self, bar::Mode};
    ///
    /// let rows = [("k0", vec![3.0, -1.0, 2.0]), ("k1", vec![-2.0, 4.0, 1.0])];
    /// let series = vec!["a".to_string(), "b".into(), "c".into()];
    /// let fig = chart::bar(rows, series).mode(Mode::Diverging).figure();
    /// ```
    #[must_use]
    pub fn mode(self, m: Mode) -> Self {
        Bar { mode: m, ..self }
    }
}

impl<K: PartialEq + Clone + Display + 'static> Bar<K> {
    /// The figure: one panel with the bars (a dodged
    /// [`bar_y`] per series when grouped; a [`bar_y_spans`] per series of
    /// `shape::stack`'s spans when stacked or diverging), its axes and
    /// title, and the legend when there is more than one series.
    pub fn figure<'a>(self) -> Figure<'a> {
        let Bar {
            rows,
            series,
            mode,
            setup,
        } = self;
        let mut fig = setup.figure();
        let keys = rows.iter().map(|(k, _)| k.clone());
        let band = scale::band(keys)
            .padding_inner(GAP)
            .padding_outer(GAP / 2.0);
        let (x, y) = (fig.scale(band), fig.fit(scale::linear()));
        let color = fig.scale(setup.palette(series.iter().cloned()));
        let panel = fig.panel(0, 0);
        let offset = match mode {
            Mode::Grouped => None,
            Mode::Stacked => Some(stack::Offset::None),
            Mode::Diverging => Some(stack::Offset::Diverging),
        };
        match offset {
            None => {
                for (i, name) in series.iter().enumerate() {
                    let bars = rows.iter().map(|(k, row)| (k.clone(), value(row, i)));
                    panel.mark(bar_y(bars, x, y).fill(color, name.clone()).dodge());
                }
            }
            Some(offset) => {
                let columns: Vec<Vec<f64>> = (0..series.len())
                    .map(|i| rows.iter().map(|(_, row)| value(row, i)).collect())
                    .collect();
                let columns: Vec<&[f64]> = columns.iter().map(Vec::as_slice).collect();
                let layers = shape::Stack::default().offset(offset).layout(&columns);
                for (layer, name) in layers.iter().zip(&series) {
                    let spans = rows.iter().zip(layer).map(|((k, _), s)| (k.clone(), *s));
                    panel.mark(bar_y_spans(spans, x, y).fill(color, name.clone()));
                }
            }
        }
        setup.axes(panel, x, y);
        if series.len() > 1 {
            fig.legend(Slot::Right, legend::legend(color));
        }
        fig
    }
}

finish!(impl[K: PartialEq + Clone + Display + 'static] Bar<K>);

/// Series `i`'s value in `row`; NaN, which draws no bar, past its end.
fn value(row: &[f64], i: usize) -> f64 {
    row.get(i).copied().unwrap_or(f64::NAN)
}
