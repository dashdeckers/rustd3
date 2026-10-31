//! d3's scaleIdentity: [`Identity`], a scale whose domain and range are
//! one array.

use crate::format::{self, InvalidFormat};
use crate::scale::{self, TickFormat};
use crate::ticks;

/// d3.scaleIdentity: `apply` and `invert` return their input; domain and
/// range are one array. Ticks, nice and tick formats are a linear scale's
/// over that array. Not the transform under linear scales,
/// [`scale::transform::Identity`].
///
/// ```
/// use rustd3_core::scale;
///
/// let s = scale::identity().domain([1.0, 2.0]);
/// assert_eq!(s.apply(0.5), 0.5);
/// assert_eq!(s.invert(3.0), 3.0);
/// assert_eq!(s.get_range(), [1.0, 2.0]);
/// assert_eq!(s.unknown(-1.0).apply(f64::NAN), -1.0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Identity {
    stops: [f64; 2],
    unknown: Option<f64>,
}

/// d3.scaleIdentity(): domain and range [0, 1].
#[doc(alias = "scaleIdentity")]
#[must_use]
pub fn identity() -> Identity {
    Identity {
        stops: [0.0, 1.0],
        unknown: None,
    }
}

impl Identity {
    /// d3's `domain(d)`: sets the range too, which is the same array.
    #[must_use]
    pub fn domain(self, domain: [f64; 2]) -> Self {
        Identity {
            stops: domain,
            ..self
        }
    }

    /// d3's `range(r)`: the same as [`domain`](Identity::domain).
    #[must_use]
    pub fn range(self, range: [f64; 2]) -> Self {
        self.domain(range)
    }

    /// d3's `unknown(v)`: what `apply` and `invert` return for NaN.
    #[must_use]
    pub fn unknown(self, value: f64) -> Self {
        Identity {
            unknown: Some(value),
            ..self
        }
    }

    /// d3's `nice(count)`: the stops extended to round values, as a linear
    /// scale's.
    #[must_use]
    pub fn nice(self, count: usize) -> Self {
        Identity {
            stops: scale::nice_linear(self.stops, count),
            ..self
        }
    }

    /// `x` itself (d3's `scale(x)`); for NaN the unknown value when set,
    /// otherwise NaN, where d3 returns `undefined`.
    #[must_use]
    pub fn apply(&self, x: f64) -> f64 {
        if x.is_nan() {
            self.unknown.unwrap_or(f64::NAN)
        } else {
            x
        }
    }

    /// d3's `invert(y)`: the same as [`apply`](Identity::apply).
    #[must_use]
    pub fn invert(&self, y: f64) -> f64 {
        self.apply(y)
    }

    /// About `count` of d3's linear ticks between the stops.
    #[must_use]
    pub fn ticks(&self, count: usize) -> Vec<f64> {
        ticks::linear(self.stops, count)
    }

    /// d3's `tickFormat(count, specifier)` under the neutral locale.
    ///
    /// # Errors
    ///
    /// [`InvalidFormat`](crate::format::InvalidFormat) when `specifier`
    /// does not parse.
    pub fn tick_format(
        &self,
        count: usize,
        specifier: Option<&str>,
    ) -> Result<TickFormat, InvalidFormat> {
        self.tick_format_in(&format::Locale::NEUTRAL, count, specifier)
    }

    /// d3's `tickFormat(count, specifier)` under `locale`.
    ///
    /// # Errors
    ///
    /// [`InvalidFormat`](crate::format::InvalidFormat) when `specifier`
    /// does not parse.
    pub fn tick_format_in(
        &self,
        locale: &format::Locale,
        count: usize,
        specifier: Option<&str>,
    ) -> Result<TickFormat, InvalidFormat> {
        scale::tick_format(locale, self.stops, count, specifier).map(TickFormat::from)
    }

    /// d3's `domain()`.
    #[must_use]
    pub fn get_domain(&self) -> [f64; 2] {
        self.stops
    }

    /// d3's `range()`: the same array as the domain.
    #[must_use]
    pub fn get_range(&self) -> [f64; 2] {
        self.stops
    }
}
