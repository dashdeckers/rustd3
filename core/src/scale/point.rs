//! d3's scalePoint: [`Point`], a band scale whose bands have no width.

use std::fmt;

use crate::locale::Locale;
use crate::scale::{Band, Ticks, band};
use crate::ticks;

/// d3.scalePoint: a band scale with zero bandwidth (inner padding 1), so
/// each key maps to a point; its `padding` is the outer padding, and it
/// has no inner or outer padding builders, as in d3.
///
/// ```
/// use rustd3_core::scale;
///
/// let p = scale::point(["a", "b", "c"]).range([0.0, 1.0]);
/// assert_eq!(p.apply(&"b"), Some(0.5));
/// assert_eq!((p.step(), p.bandwidth()), (0.5, 0.0));
/// assert_eq!(p.padding(1.0).apply(&"a"), Some(0.25));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Point<K> {
    band: Band<K>,
}

/// d3's `scalePoint().domain(keys)`: range [0, 1], no padding, centered;
/// duplicates keep their first position.
#[doc(alias = "scalePoint")]
pub fn point<K: PartialEq>(keys: impl IntoIterator<Item = K>) -> Point<K> {
    Point {
        band: band(keys).padding_inner(1.0),
    }
}

impl<K: PartialEq> Point<K> {
    /// d3's `domain(keys)`: one point per key, duplicates keeping their
    /// first position.
    pub fn domain(self, keys: impl IntoIterator<Item = K>) -> Self {
        self.with(|b| b.domain(keys))
    }

    /// d3's `range(r)`: the extent the points divide.
    pub fn range(self, range: [f64; 2]) -> Self {
        self.with(|b| b.range(range))
    }

    /// d3's `rangeRound(r)`: `range(r)` with `round(true)`.
    pub fn range_round(self, range: [f64; 2]) -> Self {
        self.with(|b| b.range_round(range))
    }

    /// d3's `round(b)`: whether the step is floored and the first point
    /// rounded.
    pub fn round(self, round: bool) -> Self {
        self.with(|b| b.round(round))
    }

    /// The outer padding, in steps: d3's `pointish` maps `padding` to
    /// `paddingOuter`.
    pub fn padding(self, p: f64) -> Self {
        self.with(|b| b.padding_outer(p))
    }

    /// d3's `align(a)`: where the outer space goes, 0 all after the
    /// points, 1 all before; clamped to [0, 1].
    pub fn align(self, a: f64) -> Self {
        self.with(|b| b.align(a))
    }

    /// The position of `key` (d3's `scale(key)`); `None` for a key outside
    /// the domain.
    pub fn apply(&self, key: &K) -> Option<f64> {
        self.band.apply(key)
    }

    /// d3's `bandwidth()`: 0, as the inner padding is 1.
    pub fn bandwidth(&self) -> f64 {
        self.band.bandwidth()
    }

    /// d3's `step()`: the distance between neighbouring points.
    pub fn step(&self) -> f64 {
        self.band.step()
    }

    /// d3's `domain()`: the keys, in order.
    pub fn get_domain(&self) -> &[K] {
        self.band.get_domain()
    }

    /// d3's `range()`.
    pub fn get_range(&self) -> [f64; 2] {
        self.band.get_range()
    }

    /// The point scale over `f` of its band scale.
    fn with(self, f: impl FnOnce(Band<K>) -> Band<K>) -> Self {
        Point { band: f(self.band) }
    }
}

/// The keys label themselves at their points.
impl<K: Clone + PartialEq + fmt::Display> Ticks for Point<K> {
    type Domain = K;
    type Step = ();

    /// Every key, whatever the count.
    fn tick_set(&self, count: usize) -> ticks::Set<K, ()> {
        self.band.tick_set(count)
    }

    /// d3-axis's `center` over a zero bandwidth: the point itself for any
    /// offset of at least 0.
    fn position(&self, value: &K, offset: f64) -> Option<f64> {
        self.band.position(value, offset)
    }

    /// Each key's `Display`; the locale is unused.
    fn labels(&self, ticks: &ticks::Set<K, ()>, locale: &Locale) -> Vec<String> {
        self.band.labels(ticks, locale)
    }

    fn extent(&self) -> [f64; 2] {
        self.band.extent()
    }
}
