//! d3's scaleThreshold (threshold.js): [`Threshold`], arbitrary bin edges,
//! one range value per bin.

use crate::array;
use crate::locale::Locale;
use crate::scale::Ticks;
use crate::ticks;

/// d3's scaleThreshold: n ascending thresholds cut the line into n + 1
/// bins, open at both ends, mapped in order to the range's values. The
/// domain must be ascending; an unsorted one gives unspecified bins, as in
/// d3, and is neither sorted nor checked. Nor is the range's length: a bin
/// without a range value maps to `None`, as d3's to undefined.
///
/// ```
/// use rustd3_core::scale;
///
/// let t = scale::threshold([0.0, 1.0], ["a", "b", "c"]);
/// let mapped: Vec<_> = [-1.0, 0.0, 0.5, 1.0, 2.0].map(|x| t.apply(x).copied()).to_vec();
/// assert_eq!(mapped, [Some("a"), Some("b"), Some("b"), Some("c"), Some("c")]);
/// assert_eq!(t.invert_extent(&"a"), Some([None, Some(0.0)]));
/// assert_eq!(t.invert_extent(&"c"), Some([Some(1.0), None]));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Threshold<R> {
    domain: Vec<f64>,
    range: Vec<R>,
}

/// d3.scaleThreshold(domain, range): the ascending thresholds `domain`, the
/// value of each bin in `range`.
#[doc(alias = "scaleThreshold")]
pub fn threshold<R>(
    domain: impl IntoIterator<Item = f64>,
    range: impl IntoIterator<Item = R>,
) -> Threshold<R> {
    Threshold {
        domain: domain.into_iter().collect(),
        range: range.into_iter().collect(),
    }
}

impl<R> Threshold<R> {
    /// d3's `domain(values)`: the ascending thresholds.
    #[must_use]
    pub fn domain(self, domain: impl IntoIterator<Item = f64>) -> Self {
        Threshold {
            domain: domain.into_iter().collect(),
            ..self
        }
    }

    /// d3's `domain()`.
    #[must_use]
    pub fn get_domain(&self) -> &[f64] {
        &self.domain
    }

    /// d3's `range(values)`: the value of each bin, in order.
    #[must_use]
    pub fn range(self, range: impl IntoIterator<Item = R>) -> Self {
        Threshold {
            range: range.into_iter().collect(),
            ..self
        }
    }

    /// d3's `range()`.
    #[must_use]
    pub fn get_range(&self) -> &[R] {
        &self.range
    }

    /// The range value of the bin `x` falls in, by bisection over the
    /// thresholds that have a bin after them in the range; `None` for NaN
    /// or a bin without a value.
    #[must_use]
    pub fn apply(&self, x: f64) -> Option<&R> {
        if x.is_nan() {
            return None;
        }
        let n = self.domain.len().min(self.range.len().saturating_sub(1));
        self.range.get(array::bisect_right(&self.domain[..n], x))
    }
}

impl<R: PartialEq> Threshold<R> {
    /// d3's `invertExtent(y)`: the thresholds around the bin of the first
    /// range value equal to `y`, `None` at an open end where d3 has
    /// undefined; `None` when no value is equal.
    pub fn invert_extent(&self, y: &R) -> Option<[Option<f64>; 2]> {
        let i = self.range.iter().position(|r| r == y)?;
        let lower = i.checked_sub(1).and_then(|j| self.domain.get(j)).copied();
        Some([lower, self.domain.get(i).copied()])
    }
}

/// The ticks of a colorbar or an axis are the thresholds; the open ends
/// have no edge.
impl<R> Ticks for Threshold<R> {
    type Domain = f64;
    type Step = f64;

    /// The domain, the smallest gap between thresholds as the step; `count`
    /// is only stored.
    fn tick_set(&self, count: usize) -> ticks::Set<f64, f64> {
        super::ticks::edges(self.domain.clone(), count)
    }

    /// The default placement, equal-width bins with the open ends as wide
    /// as the others: the i-th of n thresholds at `(i + 1)/(n + 1)`. `None`
    /// for a value that is not a threshold, NaN included. This is the
    /// geometry of a threshold scale's colorbar in rustd3-figure: its open
    /// ends have no edge in domain units, and equal rows invent none.
    fn position(&self, value: &f64, _offset: f64) -> Option<f64> {
        let i = self.domain.iter().position(|d| d == value)?;
        Some((i + 1) as f64 / (self.domain.len() + 1) as f64)
    }

    /// d3's linear format at the set's step: `,f` to the places of the
    /// smallest gap between thresholds.
    fn labels(&self, ticks: &ticks::Set<f64, f64>, locale: &Locale) -> Vec<String> {
        super::ticks::edge_labels(ticks, locale)
    }

    /// `[0, 1]`: a colorbar maps it to pixels.
    fn extent(&self) -> [f64; 2] {
        [0.0, 1.0]
    }
}
