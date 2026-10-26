//! What the figure needs from a scale: an object-safe face
//! for its table, and typed traits for marks and axes.

use d3::color::Rgb;
use d3::scale::{Band, Linear, Ordinal};
use std::any::Any;
use std::fmt::Display;

/// The erased face, so one table holds every kind.
pub trait Scale: Any {
    fn as_any(&self) -> &dyn Any;
    fn boxed(&self) -> Box<dyn Scale>;
    /// The domain as numbers, if continuous: for fitting,
    /// zooming and density grids.
    fn span(&self) -> Option<[f64; 2]> {
        None
    }
    fn with_span(&self, _: [f64; 2]) -> Box<dyn Scale> {
        self.boxed()
    }
    fn with_range(&self, r: [f64; 2]) -> Box<dyn Scale>;
    /// Pixels back to a domain number, if continuous.
    fn invert(&self, _px: f64) -> Option<f64> {
        None
    }
}

/// What a handle knows at compile time.
pub trait Kind: Scale + Clone {
    type Domain: Clone + 'static;
    type Output;
    /// A number to fit the domain to, if continuous.
    fn number(v: &Self::Domain) -> Option<f64>;
}

/// Scales that place values along x or y.
pub trait Position: Kind<Output = f64> {
    fn position(&self, v: &Self::Domain) -> Option<f64>;
    fn bandwidth(&self) -> f64 {
        0.0
    }
}

/// Scales that color values.
pub trait Paint: Kind<Output = Rgb> {
    fn paint(&self, v: &Self::Domain) -> Option<Rgb>;
    /// The key's slot among all keys, for dodging.
    fn slot(&self, _: &Self::Domain) -> Option<(usize, usize)> {
        None
    }
}

/// Scales an axis can label.
pub trait Ticks: Position {
    fn tick_values(&self, count: usize) -> Vec<Self::Domain>;
    fn tick_label(&self, v: &Self::Domain, count: usize) -> String;
}

impl Scale for Linear {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn boxed(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }
    fn span(&self) -> Option<[f64; 2]> {
        Some(self.get_domain())
    }
    fn with_span(&self, d: [f64; 2]) -> Box<dyn Scale> {
        Box::new(self.clone().domain(d))
    }
    fn with_range(&self, r: [f64; 2]) -> Box<dyn Scale> {
        Box::new(self.clone().range(r))
    }
    fn invert(&self, px: f64) -> Option<f64> {
        Some(Linear::invert(self, px))
    }
}

impl Kind for Linear {
    type Domain = f64;
    type Output = f64;
    fn number(v: &f64) -> Option<f64> {
        Some(*v)
    }
}

impl Position for Linear {
    fn position(&self, v: &f64) -> Option<f64> {
        Some(self.apply(*v))
    }
}

impl Ticks for Linear {
    fn tick_values(&self, count: usize) -> Vec<f64> {
        self.ticks(count)
    }
    fn tick_label(&self, v: &f64, count: usize) -> String {
        self.tick_format(count, None)
            .map(|f| f.format(*v))
            .unwrap_or_default()
    }
}

impl<K> Scale for Band<K>
where
    K: PartialEq + Clone + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn boxed(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }
    fn with_range(&self, r: [f64; 2]) -> Box<dyn Scale> {
        Box::new(self.clone().range(r))
    }
}

impl<K> Kind for Band<K>
where
    K: PartialEq + Clone + 'static,
{
    type Domain = K;
    type Output = f64;
    fn number(_: &K) -> Option<f64> {
        None
    }
}

impl<K> Position for Band<K>
where
    K: PartialEq + Clone + 'static,
{
    fn position(&self, k: &K) -> Option<f64> {
        self.apply(k)
    }
    fn bandwidth(&self) -> f64 {
        Band::bandwidth(self)
    }
}

impl<K> Ticks for Band<K>
where
    K: PartialEq + Clone + Display + 'static,
{
    fn tick_values(&self, _: usize) -> Vec<K> {
        self.keys().to_vec()
    }
    fn tick_label(&self, k: &K, _: usize) -> String {
        k.to_string()
    }
}

impl<K> Scale for Ordinal<K, Rgb>
where
    K: PartialEq + Clone + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn boxed(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }
    fn with_range(&self, _: [f64; 2]) -> Box<dyn Scale> {
        self.boxed()
    }
}

impl<K> Kind for Ordinal<K, Rgb>
where
    K: PartialEq + Clone + 'static,
{
    type Domain = K;
    type Output = Rgb;
    fn number(_: &K) -> Option<f64> {
        None
    }
}

impl<K> Paint for Ordinal<K, Rgb>
where
    K: PartialEq + Clone + 'static,
{
    fn paint(&self, k: &K) -> Option<Rgb> {
        self.apply(k).copied()
    }
    fn slot(&self, k: &K) -> Option<(usize, usize)> {
        let i = self.keys().iter().position(|x| x == k)?;
        Some((i, self.keys().len()))
    }
}
