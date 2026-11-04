//! d3's scaleSequential (sequential.js): a domain mapped to [0, 1] through
//! a transform, then through an interpolator.

use std::fmt;
use std::sync::Arc;

use crate::format::{InvalidFormat, Locale};
use crate::interpolate::Interpolate;
use crate::js;
use crate::locale;
use crate::scale::{Pad, TickFormat, Ticks, Transform, transform};
use crate::ticks;

/// d3's scaleSequential: the domain's two ends mapped to 0 and 1 through a
/// transform, then an interpolator of `t`, such as a color scheme. The
/// interpolator sits behind an `Arc`, so the type stays nameable
/// (`Sequential<Rgb>`) and clones share it. Log, pow, sqrt and symlog
/// variants are [`transform`](Sequential::transform) with their builders;
/// ticks, nice and tick formats act on the domain through the transform.
/// There is no `invert`, as in d3.
///
/// ```
/// use rustd3_core::scale::{self, Ticks};
/// use rustd3_core::scheme;
///
/// let seq = scale::sequential([0.0, 50.0], scheme::blues);
/// assert_eq!(seq.apply(25.0).to_string(), "rgb(109, 174, 213)");
/// assert_eq!(seq.ticks(5), [0.0, 10.0, 20.0, 30.0, 40.0, 50.0]);
/// assert_eq!((seq.position(&25.0, 0.0), seq.ends()), (Some(0.5), [0.0, 1.0]));
/// ```
#[derive(Clone)]
pub struct Sequential<R, T = transform::Identity> {
    transform: T,
    domain: [f64; 2],
    /// `transform.forward` of each end, kept current by the builders.
    forward: [f64; 2],
    interpolator: Arc<dyn Fn(f64) -> R + Send + Sync>,
    clamp: bool,
    unknown: Option<R>,
}

/// d3.scaleSequential(domain, interpolator): `domain` mapped onto [0, 1],
/// then through `interpolator`; unclamped, under the identity transform.
#[doc(alias = "scaleSequential")]
pub fn sequential<R>(
    domain: [f64; 2],
    interpolator: impl Fn(f64) -> R + Send + Sync + 'static,
) -> Sequential<R> {
    Sequential {
        transform: transform::Identity,
        domain,
        forward: domain,
        interpolator: Arc::new(interpolator),
        clamp: false,
        unknown: None,
    }
}

impl<R: fmt::Debug, T: fmt::Debug> fmt::Debug for Sequential<R, T> {
    /// The domain, transform, clamp and unknown value; the interpolator has
    /// no Debug.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sequential")
            .field("domain", &self.domain)
            .field("transform", &self.transform)
            .field("clamp", &self.clamp)
            .field("unknown", &self.unknown)
            .finish_non_exhaustive()
    }
}

impl<R: Clone, T: Transform> Sequential<R, T> {
    /// d3's `domain(d)`: the values that map to 0 and 1.
    #[must_use]
    pub fn domain(self, domain: [f64; 2]) -> Self {
        let transform = self.transform.clone();
        Sequential { domain, ..self }.transformed(transform)
    }

    /// d3's `domain()`.
    pub fn get_domain(&self) -> [f64; 2] {
        self.domain
    }

    /// d3's `interpolator(f)`: what `t` in [0, 1] maps to.
    #[must_use]
    pub fn interpolator(self, f: impl Fn(f64) -> R + Send + Sync + 'static) -> Self {
        Sequential {
            interpolator: Arc::new(f),
            ..self
        }
    }

    /// d3's `clamp(b)`: whether `t` is kept in [0, 1] before the
    /// interpolator.
    #[must_use]
    pub fn clamp(self, clamp: bool) -> Self {
        Sequential { clamp, ..self }
    }

    /// d3's `unknown(v)`: what `apply` returns for NaN.
    #[must_use]
    pub fn unknown(self, value: R) -> Self {
        Sequential {
            unknown: Some(value),
            ..self
        }
    }

    /// The same scale under `transform`, set for the current domain: d3's
    /// scaleSequentialLog, Pow, Sqrt and Symlog.
    ///
    /// ```
    /// use rustd3_core::scale::{self, transform};
    /// use rustd3_core::scheme;
    ///
    /// let s = scale::sequential([1.0, 1000.0], scheme::viridis).transform(transform::Log::new(10.0));
    /// assert_eq!(s.apply(10.0).format_hex(), "#31688e");
    /// assert_eq!(s.ticks(3), [1.0, 10.0, 100.0, 1000.0]);
    /// ```
    pub fn transform<U: Transform>(self, transform: U) -> Sequential<R, U> {
        Sequential {
            transform: transform.clone(),
            domain: self.domain,
            forward: self.forward,
            interpolator: self.interpolator,
            clamp: self.clamp,
            unknown: self.unknown,
        }
        .transformed(transform)
    }

    /// d3's `scale(x)`: the interpolator at `x`'s place between the domain
    /// ends in transformed space, clamped to [0, 1] when set, or at 0.5 when
    /// the ends transform alike. NaN gives the unknown value when set and
    /// the interpolator at NaN otherwise (d3 returns undefined).
    ///
    /// ```
    /// use rustd3_core::scale;
    ///
    /// let s = scale::sequential([0.0, 1.0], |t| t);
    /// assert_eq!((s.apply(2.0), s.clone().clamp(true).apply(2.0)), (2.0, 1.0));
    /// assert_eq!(s.unknown(-1.0).apply(f64::NAN), -1.0);
    /// ```
    pub fn apply(&self, x: f64) -> R {
        if x.is_nan() {
            return self
                .unknown
                .clone()
                .unwrap_or_else(|| (self.interpolator)(f64::NAN));
        }
        let t = self.fraction(x);
        (self.interpolator)(if self.clamp {
            js::max(0.0, js::min(1.0, t))
        } else {
            t
        })
    }

    /// Whether `x` is finite and the transform, as set for the current
    /// domain, maps it to a finite value: the data a domain can be fitted
    /// to, as [`Continuous::admits`](crate::scale::Continuous::admits).
    ///
    /// ```
    /// use rustd3_core::scale::{self, transform};
    ///
    /// let s = scale::sequential([1.0, 100.0], |t| t).transform(transform::Log::new(10.0));
    /// assert!(s.admits(0.5) && !s.admits(0.0) && !s.admits(-1.0));
    /// assert!(!scale::sequential([0.0, 1.0], |t| t).admits(f64::NAN));
    /// ```
    pub fn admits(&self, x: f64) -> bool {
        x.is_finite() && self.transform.forward(x).is_finite()
    }

    /// d3's `range()`: the interpolator at 0 and at 1.
    pub fn get_range(&self) -> [R; 2] {
        [(self.interpolator)(0.0), (self.interpolator)(1.0)]
    }

    /// d3's `ticks(count)`: about `count` ticks over the domain by the
    /// transform's rule.
    pub fn ticks(&self, count: usize) -> Vec<f64> {
        self.transform.ticks(self.domain, count)
    }

    /// d3's `nice(count)`: the domain extended to round values by the
    /// transform's rule.
    #[must_use]
    pub fn nice(self, count: usize) -> Self {
        let domain = self.transform.nice(self.domain, count);
        self.domain(domain)
    }

    /// d3's `tickFormat(count, specifier)` under the neutral locale; see
    /// [`tick_format_in`](Sequential::tick_format_in).
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
        self.tick_format_in(&Locale::NEUTRAL, count, specifier)
    }

    /// d3's `tickFormat(count, specifier)` under `locale`: the transform's
    /// tick format over the domain.
    ///
    /// # Errors
    ///
    /// [`InvalidFormat`](crate::format::InvalidFormat) when `specifier`
    /// does not parse.
    pub fn tick_format_in(
        &self,
        locale: &Locale,
        count: usize,
        specifier: Option<&str>,
    ) -> Result<TickFormat, InvalidFormat> {
        self.transform
            .tick_format(locale, self.domain, count, specifier)
    }

    /// Where `x` lies between the domain ends in transformed space (d3's
    /// `(T(x) − t0)·k10`), before clamping: 0.5 when the ends transform
    /// alike.
    fn fraction(&self, x: f64) -> f64 {
        let [t0, t1] = self.forward;
        let k10 = if t0 == t1 { 0.0 } else { 1.0 / (t1 - t0) };
        if k10 == 0.0 {
            0.5
        } else {
            (self.transform.forward(x) - t0) * k10
        }
    }

    /// The scale with `transform`, set for the current domain, and the
    /// forward cache refreshed.
    fn transformed(mut self, transform: T) -> Self {
        self.transform = transform.for_domain(self.domain);
        self.forward = self.domain.map(|x| self.transform.forward(x));
        self
    }
}

impl<R: Interpolate + Send + Sync + 'static, T> Sequential<R, T> {
    /// d3's `range([a, b])`: the interpolator becomes `a.lerp(&b, t)`.
    ///
    /// ```
    /// use rustd3_core::scale;
    ///
    /// let s = scale::sequential([0.0, 1.0], |t| t).range([0.0, 10.0]);
    /// assert_eq!((s.apply(0.5), s.get_range()), (5.0, [0.0, 10.0]));
    /// ```
    #[must_use]
    pub fn range(self, range: [R; 2]) -> Self {
        let [a, b] = range;
        Sequential {
            interpolator: Arc::new(move |t| a.lerp(&b, t)),
            ..self
        }
    }
}

impl<R: Clone> Sequential<R, transform::Log> {
    /// d3's `base(b)`: the base of the ticks, nice and labels.
    #[must_use]
    pub fn base(self, base: f64) -> Self {
        self.transform(transform::Log::new(base))
    }
}

impl<R: Clone> Sequential<R, transform::Pow> {
    /// d3's `exponent(e)`: the power the domain is raised to.
    #[must_use]
    pub fn exponent(self, exponent: f64) -> Self {
        self.transform(transform::Pow::new(exponent))
    }
}

impl<R: Clone> Sequential<R, transform::Symlog> {
    /// d3's `constant(c)`: where the symlog turns from linear to
    /// logarithmic.
    #[must_use]
    pub fn constant(self, constant: f64) -> Self {
        self.transform(transform::Symlog::new(constant))
    }
}

/// The domain fitted as a continuous scale's under the same transform:
/// with no margin it is the data's extent, a flat one widened by
/// matplotlib's `nonsingular`, as `Normalize.autoscale_None` fits a color
/// map.
///
/// ```
/// use rustd3_core::scale::{self, Pad};
///
/// let s = scale::sequential([0.0, 80.0], |t| t);
/// assert_eq!(s.pad(Some([3.0, 7.0]), 0.0, &[]), [3.0, 7.0]);
/// assert_eq!(s.pad(Some([3.0, 3.0]), 0.0, &[]), [2.85, 3.15]);
/// assert_eq!(s.clone().domain([1.0, 0.0]).pad(Some([3.0, 7.0]), 0.0, &[]), [7.0, 3.0]);
/// ```
impl<R, T: Transform> Pad for Sequential<R, T> {
    type Domain = f64;

    fn pad(&self, extent: Option<[f64; 2]>, margin: f64, sticky: &[f64]) -> [f64; 2] {
        super::pad::ends(&self.transform, self.domain, extent, margin, sticky)
    }
}

impl<R, T: Transform> crate::sealed::Pad for Sequential<R, T> {}

/// Ticks over the domain, positioned in [0, 1] for a colorbar.
impl<R: Clone, T: Transform> Ticks for Sequential<R, T> {
    type Domain = f64;
    type Step = f64;

    /// The scale's `ticks(count)`, with d3's tick step over the domain.
    fn tick_set(&self, count: usize) -> ticks::Set<f64, f64> {
        let [a, b] = self.domain;
        ticks::Set::new(
            self.ticks(count),
            ticks::tick_step(a, b, count as f64),
            count,
        )
    }

    /// Where `value` lies between the domain ends in transformed space,
    /// before clamping and the interpolator; `None` for NaN.
    fn position(&self, value: &f64, _offset: f64) -> Option<f64> {
        Some(self.fraction(*value)).filter(|t| !value.is_nan() && !t.is_nan())
    }

    /// The transform's default tick format for the set's count, under
    /// `locale`.
    fn labels(&self, ticks: &ticks::Set<f64, f64>, locale: &locale::Locale) -> Vec<String> {
        super::ticks::labels(
            self.tick_format_in(locale.get_number(), ticks.get_count(), None),
            ticks,
            locale,
        )
    }

    /// The transform's minor ticks between the majors, inside the domain.
    fn minor(&self, ticks: &ticks::Set<f64, f64>) -> Vec<f64> {
        super::ticks::minor(&self.transform, self.domain, ticks.get_values())
    }

    /// `[0, 1]`: a colorbar maps it to pixels.
    fn ends(&self) -> [f64; 2] {
        [0.0, 1.0]
    }
}

impl<R: Clone, T: Transform> crate::sealed::Ticks for Sequential<R, T> {}
