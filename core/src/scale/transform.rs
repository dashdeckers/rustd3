//! Transforms: what a continuous scale applies to its domain before
//! interpolating, together with the tick rule, nice, minor ticks and tick
//! format that suit it. The trait's canonical path is `scale::Transform`.

mod identity;

pub use identity::Identity;

use crate::format::{InvalidFormat, Locale};
use crate::scale::{self, TickFormat};
use crate::ticks;

/// How a continuous scale maps its domain before interpolating, and the
/// ticks that suit it. A transform states `forward`, `inverse` and its tick
/// rule; minor ticks, nice and tick formats default to d3's linear scale.
pub trait Transform: Clone {
    /// The transformed value of a domain value.
    fn forward(&self, x: f64) -> f64;

    /// The domain value of a transformed value.
    fn inverse(&self, y: f64) -> f64;

    /// About `count` ticks over `domain`, in its order: every transform
    /// states its rule, usually one of `ticks::`.
    fn ticks(&self, domain: [f64; 2], count: usize) -> Vec<f64>;

    /// Minor ticks between `major`: by default five equal steps in
    /// transformed space between neighbouring majors; log-like transforms
    /// supply their own.
    fn minor(&self, major: &[f64]) -> Vec<f64> {
        ticks::subdivide(self, major, 5)
    }

    /// This transform as it applies to `domain`, the scale's end stops;
    /// the scale calls it whenever its domain changes. The default returns
    /// a clone.
    fn for_domain(&self, _domain: [f64; 2]) -> Self {
        self.clone()
    }

    /// d3's nice for this transform: `domain` extended to round values.
    /// The default is d3-scale's linear nice.
    fn nice(&self, domain: [f64; 2], count: usize) -> [f64; 2] {
        scale::nice_linear(domain, count)
    }

    /// d3's tickFormat for this transform under `locale`. The default is
    /// d3-scale's tickFormat.js.
    fn tick_format(
        &self,
        locale: &Locale,
        domain: [f64; 2],
        count: usize,
        specifier: Option<&str>,
    ) -> Result<TickFormat, InvalidFormat> {
        scale::tick_format(locale, domain, count, specifier).map(TickFormat::from)
    }
}
