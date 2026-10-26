//! Scales as plain values: built per view, cloned freely, never mutated by being called.

use crate::interpolate::Interpolate;
use d3_core::format::{self, Format, InvalidFormat};
use d3_core::ticks;

/// A continuous linear map from a numeric domain to any interpolatable range.
#[derive(Debug, Clone, PartialEq)]
pub struct Linear<R = f64> {
    domain: [f64; 2],
    range: [R; 2],
    clamp: bool,
}

/// `d3.scaleLinear()`: domain and range both `[0, 1]`.
pub fn linear() -> Linear<f64> {
    Linear { domain: [0.0, 1.0], range: [0.0, 1.0], clamp: false }
}

impl<R: Interpolate> Linear<R> {
    pub fn domain(self, domain: [f64; 2]) -> Self {
        Self { domain, ..self }
    }

    /// Changing the range can change the output type: `f64` to `Rgb`, say.
    pub fn range<S: Interpolate>(self, range: [S; 2]) -> Linear<S> {
        Linear { domain: self.domain, range, clamp: self.clamp }
    }

    pub fn clamp(self, clamp: bool) -> Self {
        Self { clamp, ..self }
    }

    /// Extends the domain to nicely rounded values.
    pub fn nice(self, count: usize) -> Self {
        let [d0, d1] = self.domain;
        Self { domain: ticks::nice(d0, d1, count as f64), ..self }
    }

    pub fn apply(&self, x: f64) -> R {
        let [d0, d1] = self.domain;
        let mut t = if d1 != d0 { (x - d0) / (d1 - d0) } else { 0.5 };
        if self.clamp {
            t = t.clamp(0.0, 1.0);
        }
        self.range[0].lerp(&self.range[1], t)
    }

    pub fn get_domain(&self) -> [f64; 2] {
        self.domain
    }

    pub fn ticks(&self, count: usize) -> Vec<f64> {
        ticks::ticks(self.domain[0], self.domain[1], count as f64)
    }

    /// A number format suited to these ticks: d3's `scale.tickFormat(count, specifier)`.
    pub fn tick_format(&self, count: usize, specifier: Option<&str>) -> Result<Format, InvalidFormat> {
        let [d0, d1] = self.domain;
        let step = ticks::tick_step(d0, d1, count as f64);
        match specifier {
            Some(s) => format::format(s),
            None => format::format(&format!(",.{}f", format::precision_fixed(step).max(0.0))),
        }
    }
}

impl Linear<f64> {
    /// Only numeric ranges can be inverted, so this exists only for `Linear<f64>`.
    pub fn invert(&self, y: f64) -> f64 {
        let [r0, r1] = self.range;
        let t = (y - r0) / (r1 - r0);
        self.domain[0].lerp(&self.domain[1], t)
    }

    pub fn get_range(&self) -> [f64; 2] {
        self.range
    }
}

/// Discrete bands for categorical data, as `d3.scaleBand()`.
#[derive(Debug, Clone, PartialEq)]
pub struct Band<K> {
    keys: Vec<K>,
    range: [f64; 2],
    padding_inner: f64,
    padding_outer: f64,
    align: f64,
}

pub fn band<K: PartialEq>(keys: impl IntoIterator<Item = K>) -> Band<K> {
    Band { keys: keys.into_iter().collect(), range: [0.0, 1.0], padding_inner: 0.0, padding_outer: 0.0, align: 0.5 }
}

impl<K: PartialEq> Band<K> {
    pub fn range(self, range: [f64; 2]) -> Self {
        Self { range, ..self }
    }

    pub fn padding(self, p: f64) -> Self {
        Self { padding_inner: p, padding_outer: p, ..self }
    }

    fn layout(&self) -> (f64, f64, bool) {
        let n = self.keys.len() as f64;
        let [r0, r1] = self.range;
        let reverse = r1 < r0;
        let (start, stop) = if reverse { (r1, r0) } else { (r0, r1) };
        let step = (stop - start) / (n - self.padding_inner + self.padding_outer * 2.0).max(1.0);
        let start = start + (stop - start - step * (n - self.padding_inner)) * self.align;
        (start, step, reverse)
    }

    pub fn step(&self) -> f64 {
        self.layout().1
    }

    pub fn bandwidth(&self) -> f64 {
        self.step() * (1.0 - self.padding_inner)
    }

    /// The band's start, or `None` for a key outside the domain (d3 returns `undefined`).
    pub fn apply(&self, key: &K) -> Option<f64> {
        let i = self.keys.iter().position(|k| k == key)?;
        let (start, step, reverse) = self.layout();
        let i = if reverse { self.keys.len() - 1 - i } else { i };
        Some(start + step * i as f64)
    }

    pub fn keys(&self) -> &[K] {
        &self.keys
    }
}

/// A lookup from keys to a cycled range, as `d3.scaleOrdinal()`, with an explicit domain.
#[derive(Debug, Clone, PartialEq)]
pub struct Ordinal<K, V> {
    keys: Vec<K>,
    range: Vec<V>,
    unknown: Option<V>,
}

pub fn ordinal<K, V>(keys: impl IntoIterator<Item = K>, range: impl IntoIterator<Item = V>) -> Ordinal<K, V> {
    Ordinal { keys: keys.into_iter().collect(), range: range.into_iter().collect(), unknown: None }
}

impl<K: PartialEq, V> Ordinal<K, V> {
    pub fn unknown(self, value: V) -> Self {
        Self { unknown: Some(value), ..self }
    }

    pub fn apply(&self, key: &K) -> Option<&V> {
        match self.keys.iter().position(|k| k == key) {
            Some(i) if !self.range.is_empty() => self.range.get(i % self.range.len()),
            _ => self.unknown.as_ref(),
        }
    }

    /// d3's `ordinal.domain()`.
    pub fn keys(&self) -> &[K] {
        &self.keys
    }

    /// d3's implicit domain growth: an unseen key is appended. Needs `&mut`, so it is opt-in.
    pub fn apply_or_insert(&mut self, key: K) -> &V {
        let i = match self.keys.iter().position(|k| *k == key) {
            Some(i) => i,
            None => {
                self.keys.push(key);
                self.keys.len() - 1
            }
        };
        &self.range[i % self.range.len()]
    }
}

/// A sequential scale: a numeric domain into an interpolator such as a color scheme.
pub struct Sequential<R> {
    domain: [f64; 2],
    interpolator: Box<dyn Fn(f64) -> R + Send + Sync>,
}

pub fn sequential<R>(domain: [f64; 2], interpolator: impl Fn(f64) -> R + Send + Sync + 'static) -> Sequential<R> {
    Sequential { domain, interpolator: Box::new(interpolator) }
}

impl<R> Sequential<R> {
    pub fn apply(&self, x: f64) -> R {
        let [d0, d1] = self.domain;
        (self.interpolator)(((x - d0) / (d1 - d0)).clamp(0.0, 1.0))
    }
}

/// What an axis needs from a scale: d3 duck-types this; here it is a trait.
pub trait TickSource {
    type Value;
    fn tick_values(&self, count: usize) -> Vec<Self::Value>;
    fn position(&self, value: &Self::Value) -> f64;
    fn label(&self, value: &Self::Value, count: usize) -> String;
    fn extent(&self) -> [f64; 2];
}

impl TickSource for Linear<f64> {
    type Value = f64;
    fn tick_values(&self, count: usize) -> Vec<f64> {
        self.ticks(count)
    }
    fn position(&self, v: &f64) -> f64 {
        self.apply(*v)
    }
    fn label(&self, v: &f64, count: usize) -> String {
        self.tick_format(count, None).map(|f| f.format(*v)).unwrap_or_default()
    }
    fn extent(&self) -> [f64; 2] {
        self.range
    }
}

impl<K: PartialEq + Clone + std::fmt::Display> TickSource for Band<K> {
    type Value = K;
    fn tick_values(&self, _count: usize) -> Vec<K> {
        self.keys.clone()
    }
    fn position(&self, k: &K) -> f64 {
        self.apply(k).map_or(f64::NAN, |x| x + self.bandwidth() / 2.0)
    }
    fn label(&self, k: &K, _count: usize) -> String {
        k.to_string()
    }
    fn extent(&self) -> [f64; 2] {
        self.range
    }
}
