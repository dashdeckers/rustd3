//! d3's scaleQuantile (quantile.js): [`Quantile`], a sample's quantiles as
//! bin edges, one bin per range value.

use std::cmp::Ordering;

use crate::locale::Locale;
use crate::scale::Ticks;
use crate::{array, ticks};

/// d3's scaleQuantile: a sample of the domain, sorted, cut at its
/// quantiles into as many bins as the range has values, so each holds an
/// equal share of the sample. As an axis or a colorbar ([`Ticks`]), its
/// ticks are the sample's least value, the quantiles and its greatest.
///
/// ```
/// use rustd3_core::scale;
///
/// let q = scale::quantile([3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0], [0, 1, 2, 3]);
/// assert_eq!(q.quantiles(), [7.25, 9.0, 14.5]);
/// assert_eq!((q.apply(7.9), q.apply(8.0), q.apply(15.5)), (Some(&1), Some(&1), Some(&3)));
/// assert_eq!(q.invert_extent(&2), Some([9.0, 14.5]));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Quantile<R> {
    sorted: Vec<f64>,
    range: Vec<R>,
    thresholds: Vec<f64>,
}

/// d3.scaleQuantile(domain, range): the sample `domain`, NaN dropped and
/// sorted, cut into one bin per value of `range`.
#[doc(alias = "scaleQuantile")]
pub fn quantile<R>(
    domain: impl IntoIterator<Item = f64>,
    range: impl IntoIterator<Item = R>,
) -> Quantile<R> {
    Quantile {
        sorted: Vec::new(),
        range: range.into_iter().collect(),
        thresholds: Vec::new(),
    }
    .domain(domain)
}

impl<R> Quantile<R> {
    /// d3's `domain(values)`: the sample, NaN dropped (as d3 drops null,
    /// undefined and NaN), sorted ascending with equal values in their
    /// given order.
    #[must_use]
    pub fn domain(self, domain: impl IntoIterator<Item = f64>) -> Self {
        let mut sorted: Vec<f64> = domain.into_iter().filter(|d| !d.is_nan()).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        Quantile { sorted, ..self }.rescale()
    }

    /// d3's `domain()`: the sorted sample.
    #[must_use]
    pub fn get_domain(&self) -> &[f64] {
        &self.sorted
    }

    /// d3's `range(values)`: one bin per value.
    #[must_use]
    pub fn range(self, range: impl IntoIterator<Item = R>) -> Self {
        Quantile {
            range: range.into_iter().collect(),
            ..self
        }
        .rescale()
    }

    /// d3's `range()`.
    #[must_use]
    pub fn get_range(&self) -> &[R] {
        &self.range
    }

    /// The range value of the bin `x` falls in; `None` for NaN or an empty
    /// range, where d3 returns its unknown value.
    #[must_use]
    pub fn apply(&self, x: f64) -> Option<&R> {
        if x.is_nan() {
            return None;
        }
        self.range.get(array::bisect_right(&self.thresholds, x))
    }

    /// d3's `quantiles()`: the n − 1 inner bin edges for n range values,
    /// NaN for an empty sample.
    #[must_use]
    pub fn quantiles(&self) -> &[f64] {
        &self.thresholds
    }

    /// The `i/n`-quantiles of the sample for `i` in `1..n`, n the range's
    /// length (quantile.js `rescale`).
    fn rescale(self) -> Self {
        let n = self.range.len().max(1);
        let thresholds = (1..n)
            .map(|i| array::quantile_sorted(&self.sorted, i as f64 / n as f64).unwrap_or(f64::NAN))
            .collect();
        Quantile { thresholds, ..self }
    }
}

impl<R: PartialEq> Quantile<R> {
    /// d3's `invertExtent(y)`: the bin of the first range value equal to
    /// `y`, from its lower to its upper edge. The outer bins end at the
    /// sample's least and greatest values, or NaN for an empty sample.
    /// `None` when no value is equal, where d3 returns `[NaN, NaN]`.
    pub fn invert_extent(&self, y: &R) -> Option<[f64; 2]> {
        let i = self.range.iter().position(|r| r == y)?;
        let end = |value: Option<&f64>| value.copied().unwrap_or(f64::NAN);
        let lower = match i.checked_sub(1) {
            Some(j) => self.thresholds[j],
            None => end(self.sorted.first()),
        };
        let upper = match self.thresholds.get(i) {
            Some(&t) => t,
            None => end(self.sorted.last()),
        };
        Some([lower, upper])
    }
}

/// The ticks of a colorbar or an axis are the bin edges, positioned in [0,
/// 1] between the sample's least and greatest values.
impl<R> Ticks for Quantile<R> {
    type Domain = f64;
    type Step = f64;

    /// `[min, quantiles…, max]` of the sample (none for an empty one), the
    /// smallest gap between them as the step; `count` is only stored.
    fn tick_set(&self, count: usize) -> ticks::Set<f64, f64> {
        let edges = match (self.sorted.first(), self.sorted.last()) {
            (Some(&min), Some(&max)) => std::iter::once(min)
                .chain(self.thresholds.iter().copied())
                .chain([max])
                .collect(),
            (None, _) | (_, None) => Vec::new(),
        };
        super::ticks::edges(edges, count)
    }

    /// `(value − min)/(max − min)` over the sample; `None` for NaN or an
    /// empty sample.
    fn position(&self, value: &f64, _offset: f64) -> Option<f64> {
        let (min, max) = (self.sorted.first()?, self.sorted.last()?);
        Some((value - min) / (max - min)).filter(|t| !t.is_nan())
    }

    /// d3's linear format at the set's step: `,f` to the places of the
    /// smallest gap between edges.
    fn labels(&self, ticks: &ticks::Set<f64, f64>, locale: &Locale) -> Vec<String> {
        super::ticks::edge_labels(ticks, locale)
    }

    /// `[0, 1]`: a colorbar maps it to pixels.
    fn ends(&self) -> [f64; 2] {
        [0.0, 1.0]
    }
}

impl<R> crate::sealed::Ticks for Quantile<R> {}
