//! d3's scaleBand: [`Band`], discrete bands of a continuous range, one per
//! key of an explicit domain.

use std::fmt;

use crate::js;
use crate::locale::Locale;
use crate::scale::{self, Ticks};
use crate::ticks;

/// d3.scaleBand: each key of the domain maps to the start of its band, the
/// bands spaced by `step` and `bandwidth` wide. Lookups are a linear search
/// over the keys, so keys need only `PartialEq`; a key outside the domain
/// maps to `None`, and nothing is added to the domain. Keys compare with
/// `PartialEq`, so a NaN key never matches, where d3's SameValueZero finds
/// it; the same holds for deduplicating the domain.
///
/// ```
/// use rustd3_core::scale;
///
/// let bands = scale::band(["a", "b", "c"]).range([0.0, 300.0]).padding(0.1);
/// assert_eq!(bands.apply(&"a"), Some(9.677419354838719));
/// assert_eq!(bands.apply(&"z"), None);
/// assert_eq!(bands.bandwidth(), 87.09677419354838);
/// ```
///
/// An axis labels keys with their `Display`, so a key without one has no
/// axis:
///
/// ```compile_fail
/// use rustd3_core::{axis, scale};
///
/// #[derive(Clone, PartialEq)]
/// struct Bin(u8);
///
/// axis::bottom(&scale::band((0..20).map(Bin)));
/// ```
///
/// ```
/// use std::fmt;
///
/// use rustd3_core::{axis, scale};
///
/// #[derive(Clone, PartialEq)]
/// struct Bin(u8);
///
/// impl fmt::Display for Bin {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "{}–{}", self.0 * 10, self.0 * 10 + 10)
///     }
/// }
///
/// axis::bottom(&scale::band((0..20).map(Bin)));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Band<K> {
    keys: Vec<K>,
    range: [f64; 2],
    round: bool,
    padding_inner: f64,
    padding_outer: f64,
    align: f64,
    /// The first band's start in range order, as d3's `rescale` leaves it.
    start: f64,
    step: f64,
    bandwidth: f64,
}

/// d3's `scaleBand().domain(keys)`: range [0, 1], no padding, centered;
/// duplicates keep their first position.
#[doc(alias = "scaleBand")]
pub fn band<K: PartialEq>(keys: impl IntoIterator<Item = K>) -> Band<K> {
    Band {
        keys: Vec::new(),
        range: [0.0, 1.0],
        round: false,
        padding_inner: 0.0,
        padding_outer: 0.0,
        align: 0.5,
        start: 0.0,
        step: 1.0,
        bandwidth: 1.0,
    }
    .domain(keys)
}

impl<K: PartialEq> Band<K> {
    /// d3's `domain(keys)`: one band per key, duplicates keeping their
    /// first position.
    #[must_use]
    pub fn domain(self, keys: impl IntoIterator<Item = K>) -> Self {
        Band {
            keys: scale::unique(keys),
            ..self
        }
        .rescaled()
    }

    /// d3's `range(r)`: the extent the bands divide; a descending range
    /// puts the first key last.
    #[must_use]
    pub fn range(self, range: [f64; 2]) -> Self {
        Band { range, ..self }.rescaled()
    }

    /// d3's `rangeRound(r)`: `range(r)` with `round(true)`.
    #[must_use]
    pub fn range_round(self, range: [f64; 2]) -> Self {
        Band {
            range,
            round: true,
            ..self
        }
        .rescaled()
    }

    /// d3's `round(b)`: whether the step is floored and the start and
    /// bandwidth rounded to whole pixels.
    #[must_use]
    pub fn round(self, round: bool) -> Self {
        Band { round, ..self }.rescaled()
    }

    /// d3's `padding(p)`: the outer padding `p` and the inner padding
    /// `min(1, p)`, as fractions of the step.
    #[must_use]
    pub fn padding(self, p: f64) -> Self {
        Band {
            padding_inner: js::min(1.0, p),
            padding_outer: p,
            ..self
        }
        .rescaled()
    }

    /// d3's `paddingInner(p)`: the gap between bands, `min(1, p)` of the
    /// step.
    #[must_use]
    pub fn padding_inner(self, p: f64) -> Self {
        Band {
            padding_inner: js::min(1.0, p),
            ..self
        }
        .rescaled()
    }

    /// d3's `paddingOuter(p)`: the space before the first and after the
    /// last band, in steps.
    #[must_use]
    pub fn padding_outer(self, p: f64) -> Self {
        Band {
            padding_outer: p,
            ..self
        }
        .rescaled()
    }

    /// d3's `align(a)`: where the outer space goes, 0 all after the bands,
    /// 1 all before; clamped to [0, 1].
    #[must_use]
    pub fn align(self, a: f64) -> Self {
        Band {
            align: js::max(0.0, js::min(1.0, a)),
            ..self
        }
        .rescaled()
    }

    /// The start of `key`'s band (d3's `scale(key)`); `None` for a key
    /// outside the domain.
    pub fn apply(&self, key: &K) -> Option<f64> {
        let i = self.keys.iter().position(|k| k == key)?;
        let i = if self.range[1] < self.range[0] {
            self.keys.len() - 1 - i
        } else {
            i
        };
        Some(self.start + self.step * i as f64)
    }

    /// d3's `bandwidth()`: the width of each band.
    #[must_use]
    pub fn bandwidth(&self) -> f64 {
        self.bandwidth
    }

    /// d3's `step()`: the distance between the starts of neighbouring
    /// bands.
    #[must_use]
    pub fn step(&self) -> f64 {
        self.step
    }

    /// d3's `domain()`: the keys, in order.
    #[must_use]
    pub fn get_domain(&self) -> &[K] {
        &self.keys
    }

    /// d3's `range()`.
    #[must_use]
    pub fn get_range(&self) -> [f64; 2] {
        self.range
    }

    /// d3's `rescale` (band.js): the step, start and bandwidth for the
    /// keys, range, padding, alignment and rounding.
    fn rescaled(self) -> Self {
        let n = self.keys.len() as f64;
        let [r0, r1] = self.range;
        let (start, stop) = if r1 < r0 { (r1, r0) } else { (r0, r1) };
        let fit = n - self.padding_inner + self.padding_outer * 2.0;
        let mut step = (stop - start) / js::max(1.0, fit);
        if self.round {
            step = step.floor();
        }
        let mut start = start + (stop - start - step * (n - self.padding_inner)) * self.align;
        let mut bandwidth = step * (1.0 - self.padding_inner);
        if self.round {
            start = js::round(start);
            bandwidth = js::round(bandwidth);
        }
        Band {
            start,
            step,
            bandwidth,
            ..self
        }
    }
}

/// The keys label themselves and sit at the band centers, as d3-axis's
/// `center`.
impl<K: Clone + PartialEq + fmt::Display> Ticks for Band<K> {
    type Domain = K;
    type Step = ();

    /// Every key, whatever the count.
    fn tick_set(&self, count: usize) -> ticks::Set<K, ()> {
        ticks::Set::new(self.keys.clone(), (), count)
    }

    /// The band's start plus half of what is left of the bandwidth after
    /// `offset` on each side, that half rounded when the scale rounds.
    fn position(&self, value: &K, offset: f64) -> Option<f64> {
        let mut inset = js::max(0.0, self.bandwidth - offset * 2.0) / 2.0;
        if self.round {
            inset = js::round(inset);
        }
        Some(self.apply(value)? + inset)
    }

    /// Each key's `Display`, d3's identity format; the locale is unused.
    fn labels(&self, ticks: &ticks::Set<K, ()>, _locale: &Locale) -> Vec<String> {
        ticks.values().iter().map(ToString::to_string).collect()
    }

    fn extent(&self) -> [f64; 2] {
        self.range
    }
}

impl<K: Clone + PartialEq + fmt::Display> crate::sealed::Ticks for Band<K> {}
