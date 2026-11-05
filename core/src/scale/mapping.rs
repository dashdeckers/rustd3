//! [`Transform`]: how a continuous scale maps its domain before
//! interpolating, and the ticks that suit it; the built-in transforms are
//! in [`scale::transform`](crate::scale::transform).

use crate::format::{InvalidFormat, Locale};
use crate::scale::{self, TickFormat};
use crate::ticks;

/// How a continuous scale maps its domain before interpolating, and the
/// ticks that suit it. A transform states `forward`, `inverse` and its tick
/// rule; minor ticks, nice and tick formats default to d3's linear scale.
///
/// ```
/// use rustd3_core::{scale, ticks};
///
/// /// Frequency laid out by period.
/// #[derive(Clone, Copy)]
/// struct Reciprocal;
///
/// impl scale::Transform for Reciprocal {
///     fn forward(&self, x: f64) -> f64 {
///         1.0 / x
///     }
///     fn inverse(&self, y: f64) -> f64 {
///         1.0 / y
///     }
///     fn ticks(&self, d: [f64; 2], n: usize) -> Vec<f64> {
///         ticks::log(10.0, d, n)
///     }
/// }
///
/// let f = scale::continuous(Reciprocal).domain([1.0, 1e6]);
/// assert_eq!(f.ticks(5), [1.0, 10.0, 100.0, 1e3, 1e4, 1e5, 1e6]);
/// ```
pub trait Transform: Clone {
    /// The transformed value of a domain value.
    fn forward(&self, x: f64) -> f64;

    /// The domain value of a transformed value.
    fn inverse(&self, y: f64) -> f64;

    /// About `count` ticks over `domain`, in its order. There is no
    /// default, and the rule is usually one of the `ticks::` functions.
    fn ticks(&self, domain: [f64; 2], count: usize) -> Vec<f64>;

    /// Minor ticks between `major`. The default is five equal steps in
    /// transformed space between neighboring majors, and log-like
    /// transforms supply their own.
    fn minor(&self, major: &[f64]) -> Vec<f64> {
        ticks::subdivide(self, major, 5)
    }

    /// This transform as it applies to `domain`, the scale's end stops;
    /// the scale calls it whenever its domain changes. The default returns
    /// a clone.
    #[must_use]
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
    ///
    /// With no specifier the result must be `Ok`: an override's default
    /// specifier must parse, since [`scale::Ticks::labels`] labels axes
    /// through it and panics otherwise.
    ///
    /// # Errors
    ///
    /// [`InvalidFormat`](crate::format::InvalidFormat) when `specifier`
    /// does not parse; never for `None`.
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
