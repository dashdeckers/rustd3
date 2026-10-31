//! Histogram statistics and stacking: [`Bins`], [`Table`], [`table`],
//! [`Error`] and [`stacked`]; the chart is [`stats::hist`](super::hist()).

pub(super) mod chart;
mod stacked;

use rustd3_core::array;
use rustd3_core::scale;

use crate::mark::{self, Binned};

pub use stacked::{Stacked, stacked};

/// The most bins `Auto` makes, as `Count` (the histogram mark's cap).
const COUNT: usize = 10_000;

/// How a histogram bins its values: [`mark::Bins`]' two rules plus
/// caller edges and numpy's `"auto"`.
#[derive(Debug, Clone, PartialEq)]
pub enum Bins {
    /// d3.bin's nice bins over about `n` ticks ([`mark::Bins::Nice`]); the
    /// default is Sturges' count, ⌈log₂ n⌉ + 1 for n values.
    Nice(usize),
    /// `n` equal-width bins over `[min, max]`, numpy's and matplotlib's
    /// rule ([`mark::Bins::Count`]).
    Count(usize),
    /// The caller's edges, finite and strictly increasing; values outside
    /// them are dropped, as numpy and matplotlib's `hist` drop them.
    Edges(Vec<f64>),
    /// numpy's `"auto"` (2.x): equal-width bins at the smaller of Sturges'
    /// width (`range / (log₂ n + 1)`) and Freedman–Diaconis' (`2·IQR·n^(−1/3)`),
    /// the latter at least half the square-root rule's (`range / √n`), so
    /// an IQR of 0 or a long tail does not make thousands of bins; at most
    /// 10,000.
    Auto,
}

/// A histogram's bins: `edges` (one more than `counts`) and the height of
/// each bin, a count, a weight, a density or their running sum. Plain
/// data.
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    /// The bin edges, increasing; bins are left-closed, the last one
    /// closed.
    pub edges: Vec<f64>,
    /// Each bin's height.
    pub counts: Vec<f64>,
}

/// Why a histogram's table could not be made.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A caller's edge that is not finite or not above the one before it.
    #[error("edge {index} is not finite or not above the one before: {value}")]
    Edge {
        /// The first bad edge's index.
        index: usize,
        /// Its value.
        value: f64,
    },
    /// Weights whose count differs from the values'.
    #[error("{weights} weights for {values} values")]
    Weights {
        /// The values given.
        values: usize,
        /// The weights given.
        weights: usize,
    },
}

impl From<Error> for mark::Error {
    /// The same error, as [`Figure::check`](crate::Figure::check) reports
    /// it for a mark.
    fn from(e: Error) -> mark::Error {
        match e {
            Error::Edge { index, value } => mark::Error::Edge { index, value },
            Error::Weights { values, weights } => mark::Error::Weights { values, weights },
        }
    }
}

/// The statistic behind [`hist`](super::hist()) and [`stacked`]: the finite
/// `values` (each with its weight, 1 without `weights`) binned by `bins`,
/// left-closed with the last bin closed, as numpy's `histogram` bins them.
/// With `density` each height is divided by the total weight counted and
/// the bin's width, so the bars' area sums to 1; with `cumulative` each
/// height is the running sum (of the bins' areas with `density`, so the
/// last is 1), matplotlib's `hist`. No finite values give no bins, except
/// with caller edges.
///
/// ```
/// use rustd3_figure::helpers::stats::hist::{self, Bins};
///
/// let x = [1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 4.7, 5.1, 5.6, 6.2, 7.9, 9.5, 14.0];
/// let t = hist::table(&x, None, &Bins::Count(5), false, true)?;
/// assert_eq!(t.counts, [5.0, 12.0, 13.0, 14.0, 15.0]);
/// let bad = hist::table(&x, None, &Bins::Edges(vec![0.0, 1.0, 1.0, 2.0]), false, false);
/// assert_eq!(bad, Err(hist::Error::Edge { index: 2, value: 1.0 }));
/// # Ok::<(), hist::Error>(())
/// ```
///
/// # Errors
///
/// [`Error::Edge`] for caller's edges that are not finite and increasing,
/// [`Error::Weights`] for weights of another count than the values.
pub fn table(
    values: &[f64],
    weights: Option<&[f64]>,
    bins: &Bins,
    density: bool,
    cumulative: bool,
) -> Result<Table, Error> {
    if let Some(w) = weights
        && w.len() != values.len()
    {
        return Err(Error::Weights {
            values: values.len(),
            weights: w.len(),
        });
    }
    let ones = std::iter::repeat(1.0);
    let weighted: Vec<(f64, f64)> = match weights {
        Some(w) => values.iter().copied().zip(w.iter().copied()).collect(),
        None => values.iter().copied().zip(ones).collect(),
    };
    let finite: Vec<f64> = weighted
        .iter()
        .map(|(v, _)| *v)
        .filter(|v| v.is_finite())
        .collect();
    let edges = match bins {
        Bins::Nice(n) => binned(&finite, mark::Bins::Nice(*n)),
        Bins::Count(n) => binned(&finite, mark::Bins::Count(*n)),
        Bins::Auto => binned(&finite, mark::Bins::Count(auto(&finite))),
        Bins::Edges(edges) => checked(edges)?,
    };
    let mut counts = vec![0.0; edges.len().saturating_sub(1)];
    for (v, w) in weighted {
        if let Some(i) = locate(&edges, v) {
            counts[i] += w;
        }
    }
    let widths: Vec<f64> = edges.windows(2).map(|e| e[1] - e[0]).collect();
    if density {
        let total: f64 = counts.iter().sum();
        for (c, w) in counts.iter_mut().zip(&widths) {
            *c /= total * w;
        }
    }
    if cumulative {
        let mut sum = 0.0;
        for (c, w) in counts.iter_mut().zip(&widths) {
            sum += if density { *c * w } else { *c };
            *c = sum;
        }
    }
    Ok(Table { edges, counts })
}

/// Sturges' bin count for `n` values: ⌈log₂ n⌉ + 1, d3's
/// `thresholdSturges`; 1 for none.
fn sturges(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    (n as f64).log2().ceil() as usize + 1
}

/// The default bins of `n` values: [`Bins::Nice`] with Sturges' count.
pub(super) fn default_bins(n: usize) -> Bins {
    Bins::Nice(sturges(n))
}

/// The edges the histogram mark makes for `finite` values by `rule`:
/// the same bins, read back from them.
fn binned(finite: &[f64], rule: mark::Bins) -> Vec<f64> {
    let bars = <scale::Linear as Binned>::bars(finite, Some(rule));
    let first = bars.first().map(|([lo, _], _)| *lo);
    first
        .into_iter()
        .chain(bars.iter().map(|([_, hi], _)| *hi))
        .collect()
}

/// numpy 2's `"auto"` bin count for `finite` values: the range over the
/// smaller of Sturges' width and Freedman–Diaconis' (at least half the
/// square-root rule's), rounded up; one bin for values that are all equal.
fn auto(finite: &[f64]) -> usize {
    let Some([lo, hi]) = array::extent(finite.iter().copied()) else {
        return 1;
    };
    let n = finite.len() as f64;
    let range = hi - lo;
    let sturges = range / (n.log2() + 1.0);
    let quartile = |p: f64| array::quantile(finite.iter().copied(), p).unwrap_or(lo);
    let iqr = quartile(0.75) - quartile(0.25);
    let fd = 2.0 * iqr * n.powf(-1.0 / 3.0);
    let width = fd.max(range / n.sqrt() / 2.0).min(sturges);
    if width > 0.0 {
        ((range / width).ceil() as usize).clamp(1, COUNT)
    } else {
        1
    }
}

/// The caller's `edges` when each is finite and above the one before it.
fn checked(edges: &[f64]) -> Result<Vec<f64>, Error> {
    let bad = edges
        .iter()
        .enumerate()
        .find(|&(i, &v)| !v.is_finite() || (i > 0 && v <= edges[i - 1]));
    match bad {
        Some((index, &value)) => Err(Error::Edge { index, value }),
        None => Ok(edges.to_vec()),
    }
}

/// The bin of `v` among `edges`: left-closed, the last bin closed; `None`
/// outside them or for fewer than two edges.
fn locate(edges: &[f64], v: f64) -> Option<usize> {
    let (&first, &last) = (edges.first()?, edges.last()?);
    if edges.len() < 2 || !(first..=last).contains(&v) {
        return None;
    }
    Some((array::bisect_right(edges, v) - 1).min(edges.len() - 2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_sit_in_left_closed_bins_and_the_last_is_closed() {
        let edges = [0.0, 1.0, 2.0];
        let at = [-0.5, 0.0, 0.5, 1.0, 2.0, 2.5].map(|v| locate(&edges, v));
        assert_eq!(at, [None, Some(0), Some(0), Some(1), Some(1), None]);
        assert_eq!(locate(&[1.0], 1.0), None);
        assert_eq!(locate(&[], 1.0), None);
    }

    #[test]
    fn sturges_counts_as_d3_does() {
        assert_eq!([0, 1, 2, 15, 16, 17].map(sturges), [1, 1, 2, 5, 5, 6]);
    }

    #[test]
    fn edges_must_be_finite_and_increasing() {
        assert_eq!(checked(&[0.0, 1.0]), Ok(vec![0.0, 1.0]));
        assert!(matches!(
            checked(&[0.0, f64::NAN]),
            Err(Error::Edge { index: 1, value }) if value.is_nan()
        ));
        assert!(matches!(
            checked(&[f64::INFINITY]),
            Err(Error::Edge { index: 0, .. })
        ));
        assert_eq!(
            checked(&[2.0, 1.0]),
            Err(Error::Edge {
                index: 1,
                value: 1.0
            })
        );
    }

    #[test]
    fn weights_of_another_length_are_an_error() {
        let t = table(&[1.0, 2.0], Some(&[1.0]), &Bins::Count(1), false, false);
        assert_eq!(
            t,
            Err(Error::Weights {
                values: 2,
                weights: 1
            })
        );
    }

    #[test]
    fn no_finite_values_give_no_bins_unless_edges_are_given() {
        let empty = table(&[f64::NAN], None, &Bins::Auto, false, false);
        assert_eq!(
            empty,
            Ok(Table {
                edges: Vec::new(),
                counts: Vec::new()
            })
        );
        let edged = table(&[], None, &Bins::Edges(vec![0.0, 1.0]), false, false);
        assert_eq!(edged.map(|t| t.counts), Ok(vec![0.0]));
    }
}
