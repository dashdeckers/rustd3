//! d3's scaleQuantize (quantize.js): [`Quantize`], a continuous domain cut
//! into equal bins, one per range value.

use crate::array;
use crate::format::{self, Format, InvalidFormat};
use crate::locale::Locale;
use crate::scale::{self, Pad, Ticks, transform};
use crate::ticks;

/// d3's scaleQuantize: the domain `[x0, x1]` cut into as many equal bins as
/// the range has values, each mapped to its value; values past the ends
/// take the first or last. Ticks, `nice` and the tick format are a linear
/// scale's over the domain; as an axis or a colorbar ([`Ticks`]), its ticks
/// are the bin edges.
///
/// ```
/// use rustd3_core::scale::{self, Ticks};
///
/// let q = scale::quantize([0.0, 1.0], ["a", "b", "c"]);
/// assert_eq!((q.apply(0.2), q.apply(0.5), q.apply(0.9)), (Some(&"a"), Some(&"b"), Some(&"c")));
/// assert_eq!(q.thresholds(), [0.3333333333333333, 0.6666666666666666]);
/// assert_eq!(q.invert_extent(&"b"), Some([0.3333333333333333, 0.6666666666666666]));
/// assert_eq!(q.tick_set(5).values(), [0.0, 0.3333333333333333, 0.6666666666666666, 1.0]);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Quantize<R> {
    domain: [f64; 2],
    range: Vec<R>,
    /// The inner bin edges, recomputed when the domain or range changes
    /// (d3's `rescale`).
    thresholds: Vec<f64>,
}

/// d3.scaleQuantize(domain, range): `domain` cut into one bin per value of
/// `range`.
#[doc(alias = "scaleQuantize")]
pub fn quantize<R>(domain: [f64; 2], range: impl IntoIterator<Item = R>) -> Quantize<R> {
    Quantize {
        domain,
        range: range.into_iter().collect(),
        thresholds: Vec::new(),
    }
    .rescale()
}

impl<R> Quantize<R> {
    /// d3's `domain([x0, x1])`.
    pub fn domain(self, domain: [f64; 2]) -> Self {
        Quantize { domain, ..self }.rescale()
    }

    /// d3's `domain()`.
    pub fn get_domain(&self) -> [f64; 2] {
        self.domain
    }

    /// d3's `range(values)`: one bin per value.
    pub fn range(self, range: impl IntoIterator<Item = R>) -> Self {
        Quantize {
            range: range.into_iter().collect(),
            ..self
        }
        .rescale()
    }

    /// d3's `range()`.
    pub fn get_range(&self) -> &[R] {
        &self.range
    }

    /// The range value of the bin `x` falls in, the first or last past the
    /// ends; `None` for NaN or an empty range, where d3 returns its unknown
    /// value.
    pub fn apply(&self, x: f64) -> Option<&R> {
        if x.is_nan() {
            return None;
        }
        self.range.get(array::bisect_right(&self.thresholds, x))
    }

    /// d3's `thresholds()`: the n − 1 inner bin edges for n range values.
    pub fn thresholds(&self) -> Vec<f64> {
        self.thresholds.clone()
    }

    /// d3's `nice(count)`: the domain extended to the ticks of a linear
    /// scale.
    pub fn nice(self, count: usize) -> Self {
        let domain = scale::nice_linear(self.domain, count);
        self.domain(domain)
    }

    /// About `count` of d3's linear ticks over the domain.
    pub fn ticks(&self, count: usize) -> Vec<f64> {
        ticks::linear(self.domain, count)
    }

    /// d3's `tickFormat(count, specifier)` over the domain under the
    /// neutral locale.
    pub fn tick_format(
        &self,
        count: usize,
        specifier: Option<&str>,
    ) -> Result<Format, InvalidFormat> {
        scale::tick_format(&format::Locale::NEUTRAL, self.domain, count, specifier)
    }

    /// The edges `((i + 1)·x1 − (i − n)·x0) / (n + 1)` for `i` in `0..n`,
    /// n + 1 the range's length (quantize.js `rescale`).
    fn rescale(self) -> Self {
        let [x0, x1] = self.domain;
        let n = self.range.len().saturating_sub(1);
        let thresholds = (0..n)
            .map(|i| ((i + 1) as f64 * x1 - (i as f64 - n as f64) * x0) / (n + 1) as f64)
            .collect();
        Quantize { thresholds, ..self }
    }
}

impl<R: PartialEq> Quantize<R> {
    /// d3's `invertExtent(y)`: the bin of the first range value equal to
    /// `y`, from its lower to its upper edge (the domain's ends at either
    /// end); `None` when no value is equal, where d3 returns `[NaN, NaN]`.
    pub fn invert_extent(&self, y: &R) -> Option<[f64; 2]> {
        let i = self.range.iter().position(|r| r == y)?;
        let [x0, x1] = self.domain;
        let lower = i.checked_sub(1).map_or(x0, |j| self.thresholds[j]);
        let upper = self.thresholds.get(i).copied().unwrap_or(x1);
        Some([lower, upper])
    }
}

/// The domain fitted as a linear scale's: with no margin, the data's
/// extent, a flat one widened by matplotlib's `nonsingular`.
///
/// ```
/// use rustd3_core::scale::{self, Pad};
///
/// let q = scale::quantize([0.0, 1.0], ["low", "high"]);
/// assert_eq!(q.pad(Some([3.0, 7.0]), 0.0, &[]), [3.0, 7.0]);
/// assert_eq!(q.pad(None, 0.0, &[]), [0.0, 1.0]);
/// ```
impl<R> Pad for Quantize<R> {
    type Domain = f64;

    fn pad(&self, extent: Option<[f64; 2]>, margin: f64, sticky: &[f64]) -> [f64; 2] {
        super::pad::ends(&transform::Identity, self.domain, extent, margin, sticky)
    }
}

/// The ticks of a colorbar or an axis are the bin edges, positioned in [0,
/// 1] along the domain.
impl<R> Ticks for Quantize<R> {
    type Domain = f64;
    type Step = f64;

    /// `[x0, thresholds…, x1]`, the smallest gap between them as the step;
    /// `count` is only stored.
    fn tick_set(&self, count: usize) -> ticks::Set<f64, f64> {
        let [x0, x1] = self.domain;
        let edges = std::iter::once(x0)
            .chain(self.thresholds.iter().copied())
            .chain([x1])
            .collect();
        super::ticks::edges(edges, count)
    }

    /// `(value − x0)/(x1 − x0)`; `None` for NaN.
    fn position(&self, value: &f64, _offset: f64) -> Option<f64> {
        let [x0, x1] = self.domain;
        Some((value - x0) / (x1 - x0)).filter(|t| !t.is_nan())
    }

    /// d3's linear format at the set's step: `,f` to the places of the
    /// smallest gap between edges.
    fn labels(&self, ticks: &ticks::Set<f64, f64>, locale: &Locale) -> Vec<String> {
        super::ticks::edge_labels(ticks, locale)
    }

    /// `[0, 1]`: a colorbar maps it to pixels.
    fn extent(&self) -> [f64; 2] {
        [0.0, 1.0]
    }
}
