//! d3's scaleDiverging (diverging.js): a three-stop domain mapped to 0,
//! 0.5 and 1 through a transform, then through an interpolator.

use std::fmt;
use std::sync::Arc;

use crate::format::{InvalidFormat, Locale};
use crate::interpolate::{self, Interpolate};
use crate::js;
use crate::locale;
use crate::scale::{Pad, TickFormat, Ticks, Transform, transform};
use crate::ticks;

/// d3's scaleDiverging: [`Sequential`](crate::scale::Sequential) with a
/// middle stop. The domain's stops map to 0, 0.5 and 1 through a
/// transform, each half linearly in transformed space, then through an
/// interpolator such as a diverging scheme; the domain may descend. Log,
/// pow, sqrt and symlog variants are
/// [`transform`](Diverging::transform) with their builders; ticks, nice
/// and tick formats act on the end stops. There is no `invert`, as in d3.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_core::scheme;
///
/// let s = scale::diverging([-1.0, 0.0, 4.0], |t| t);
/// assert_eq!((s.apply(-0.5), s.apply(2.0)), (0.25, 0.75));
/// let heat = scale::diverging([-1.0, 0.0, 4.0], scheme::rd_bu);
/// assert_eq!(heat.apply(2.0).to_string(), "rgb(107, 172, 208)");
/// ```
#[derive(Clone)]
pub struct Diverging<R, T = transform::Identity> {
    transform: T,
    domain: [f64; 3],
    /// `transform.forward` of each stop, kept current by the builders.
    forward: [f64; 3],
    interpolator: Arc<dyn Fn(f64) -> R + Send + Sync>,
    clamp: bool,
    unknown: Option<R>,
}

/// d3.scaleDiverging(domain, interpolator): `domain`'s stops mapped onto
/// 0, 0.5 and 1, then through `interpolator`; unclamped, under the
/// identity transform.
#[doc(alias = "scaleDiverging")]
pub fn diverging<R>(
    domain: [f64; 3],
    interpolator: impl Fn(f64) -> R + Send + Sync + 'static,
) -> Diverging<R> {
    Diverging {
        transform: transform::Identity,
        domain,
        forward: domain,
        interpolator: Arc::new(interpolator),
        clamp: false,
        unknown: None,
    }
}

impl<R: fmt::Debug, T: fmt::Debug> fmt::Debug for Diverging<R, T> {
    /// The domain, transform, clamp and unknown value; the interpolator has
    /// no Debug.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Diverging")
            .field("domain", &self.domain)
            .field("transform", &self.transform)
            .field("clamp", &self.clamp)
            .field("unknown", &self.unknown)
            .finish_non_exhaustive()
    }
}

impl<R: Clone, T: Transform> Diverging<R, T> {
    /// d3's `domain(d)`: the values that map to 0, 0.5 and 1.
    pub fn domain(self, domain: [f64; 3]) -> Self {
        let transform = self.transform.clone();
        Diverging { domain, ..self }.transformed(transform)
    }

    /// d3's `domain()`.
    pub fn get_domain(&self) -> [f64; 3] {
        self.domain
    }

    /// d3's `interpolator(f)`: what `t` in [0, 1] maps to.
    pub fn interpolator(self, f: impl Fn(f64) -> R + Send + Sync + 'static) -> Self {
        Diverging {
            interpolator: Arc::new(f),
            ..self
        }
    }

    /// d3's `clamp(b)`: whether `t` is kept in [0, 1] before the
    /// interpolator.
    pub fn clamp(self, clamp: bool) -> Self {
        Diverging { clamp, ..self }
    }

    /// d3's `unknown(v)`: what `apply` returns for NaN.
    pub fn unknown(self, value: R) -> Self {
        Diverging {
            unknown: Some(value),
            ..self
        }
    }

    /// The same scale under `transform`, set for the current end stops:
    /// d3's scaleDivergingLog, Pow, Sqrt and Symlog.
    ///
    /// ```
    /// use rustd3_core::scale::{self, transform};
    ///
    /// let s = scale::diverging([-100.0, 0.0, 100.0], |t| t).transform(transform::Symlog::new(1.0));
    /// assert_eq!(s.apply(10.0), 0.7597868532412204);
    /// let log = scale::diverging([3.0, 2.0, 1.0], |t| t).transform(transform::Log::new(10.0));
    /// assert_eq!(log.apply(2.0), 0.5);
    /// ```
    pub fn transform<U: Transform>(self, transform: U) -> Diverging<R, U> {
        Diverging {
            transform: transform.clone(),
            domain: self.domain,
            forward: self.forward,
            interpolator: self.interpolator,
            clamp: self.clamp,
            unknown: self.unknown,
        }
        .transformed(transform)
    }

    /// d3's `scale(x)`: the interpolator at `x`'s place on its half of the
    /// domain in transformed space, clamped to [0, 1] when set; a half whose
    /// stops transform alike maps to 0.5. NaN gives the unknown value when
    /// set and the interpolator at NaN otherwise (d3 returns undefined).
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

    /// Whether `x` is finite and the transform, as set for the current end
    /// stops, maps it to a finite value: the data a domain can be fitted
    /// to, as [`Continuous::admits`](crate::scale::Continuous::admits).
    ///
    /// ```
    /// use rustd3_core::scale;
    ///
    /// let s = scale::diverging([-1.0, 0.0, 1.0], |t| t);
    /// assert!(s.admits(-5.0) && !s.admits(f64::INFINITY));
    /// ```
    pub fn admits(&self, x: f64) -> bool {
        x.is_finite() && self.transform.forward(x).is_finite()
    }

    /// d3's `range()`: the interpolator at 0, 0.5 and 1.
    pub fn get_range(&self) -> [R; 3] {
        [0.0, 0.5, 1.0].map(|t| (self.interpolator)(t))
    }

    /// d3's `ticks(count)`: about `count` ticks between the end stops by
    /// the transform's rule.
    pub fn ticks(&self, count: usize) -> Vec<f64> {
        self.transform.ticks(self.ends(), count)
    }

    /// d3's `nice(count)`: the end stops extended to round values by the
    /// transform's rule; the middle stop stays.
    pub fn nice(self, count: usize) -> Self {
        let [start, stop] = self.transform.nice(self.ends(), count);
        let [_, middle, _] = self.domain;
        self.domain([start, middle, stop])
    }

    /// d3's `tickFormat(count, specifier)` under the neutral locale; see
    /// [`tick_format_in`](Diverging::tick_format_in).
    pub fn tick_format(
        &self,
        count: usize,
        specifier: Option<&str>,
    ) -> Result<TickFormat, InvalidFormat> {
        self.tick_format_in(&Locale::NEUTRAL, count, specifier)
    }

    /// d3's `tickFormat(count, specifier)` under `locale`: the transform's
    /// tick format over the end stops.
    pub fn tick_format_in(
        &self,
        locale: &Locale,
        count: usize,
        specifier: Option<&str>,
    ) -> Result<TickFormat, InvalidFormat> {
        self.transform
            .tick_format(locale, self.ends(), count, specifier)
    }

    /// The first and last stop, which ticks, nice and formats act on.
    fn ends(&self) -> [f64; 2] {
        [self.domain[0], self.domain[2]]
    }

    /// d3's `0.5 + (u − t1)·(s·u < s·t1 ? k10 : k21)` with `u = T(x)`, before
    /// clamping: each half spans 0.5, or none when its stops transform
    /// alike, and `s` follows a descending domain.
    fn fraction(&self, x: f64) -> f64 {
        let [t0, t1, t2] = self.forward;
        let k10 = if t0 == t1 { 0.0 } else { 0.5 / (t1 - t0) };
        let k21 = if t1 == t2 { 0.0 } else { 0.5 / (t2 - t1) };
        let s = if t1 < t0 { -1.0 } else { 1.0 };
        let u = self.transform.forward(x);
        0.5 + (u - t1) * if s * u < s * t1 { k10 } else { k21 }
    }

    /// The scale with `transform`, set for the current end stops, and the
    /// forward cache refreshed.
    fn transformed(mut self, transform: T) -> Self {
        self.transform = transform.for_domain(self.ends());
        self.forward = self.domain.map(|x| self.transform.forward(x));
        self
    }
}

impl<R: Interpolate + Send + Sync + 'static, T> Diverging<R, T> {
    /// d3's `range([a, b, c])`: the interpolator becomes
    /// [`interpolate::piecewise`] over the three values by `lerp`.
    ///
    /// ```
    /// use rustd3_core::scale;
    ///
    /// let s = scale::diverging([0.0, 0.5, 1.0], |t| t).range([1.0, 3.0, 10.0]);
    /// assert_eq!((s.apply(0.5), s.apply(0.75)), (3.0, 6.5));
    /// assert_eq!(s.get_range(), [1.0, 3.0, 10.0]);
    /// ```
    pub fn range(self, range: [R; 3]) -> Self {
        let blend = interpolate::piecewise(&range, |a: &R, b: &R| {
            let (a, b) = (a.clone(), b.clone());
            move |t| a.lerp(&b, t)
        });
        Diverging {
            interpolator: Arc::new(blend),
            ..self
        }
    }
}

impl<R: Clone> Diverging<R, transform::Log> {
    /// d3's `base(b)`: the base of the ticks, nice and labels.
    pub fn base(self, base: f64) -> Self {
        self.transform(transform::Log::new(base))
    }
}

impl<R: Clone> Diverging<R, transform::Pow> {
    /// d3's `exponent(e)`: the power the domain is raised to.
    pub fn exponent(self, exponent: f64) -> Self {
        self.transform(transform::Pow::new(exponent))
    }
}

impl<R: Clone> Diverging<R, transform::Symlog> {
    /// d3's `constant(c)`: where the symlog turns from linear to
    /// logarithmic.
    pub fn constant(self, constant: f64) -> Self {
        self.transform(transform::Symlog::new(constant))
    }
}

/// The end stops fitted as a continuous scale's under the same
/// transform, in their order: with no margin, the data's extent, a flat
/// one widened by matplotlib's `nonsingular`. The middle stop is the
/// caller's to keep.
///
/// ```
/// use rustd3_core::scale::{self, Pad};
///
/// let s = scale::diverging([-1.0, 0.0, 1.0], |t| t);
/// assert_eq!(s.pad(Some([2.0, 5.0]), 0.0, &[]), [2.0, 5.0]);
/// let reversed = s.domain([1.0, 0.0, -1.0]);
/// assert_eq!(reversed.pad(Some([2.0, 5.0]), 0.0, &[]), [5.0, 2.0]);
/// ```
impl<R, T: Transform> Pad for Diverging<R, T> {
    type Domain = f64;

    fn pad(&self, extent: Option<[f64; 2]>, margin: f64, sticky: &[f64]) -> [f64; 2] {
        let [start, _, stop] = self.domain;
        super::pad::ends(&self.transform, [start, stop], extent, margin, sticky)
    }
}

/// Ticks between the end stops, positioned in [0, 1] for a colorbar.
impl<R: Clone, T: Transform> Ticks for Diverging<R, T> {
    type Domain = f64;
    type Step = f64;

    /// The scale's `ticks(count)`, with d3's tick step over the end stops.
    fn tick_set(&self, count: usize) -> ticks::Set<f64, f64> {
        let [a, b] = self.ends();
        ticks::Set::new(
            self.ticks(count),
            ticks::tick_step(a, b, count as f64),
            count,
        )
    }

    /// Where `value` maps before clamping and the interpolator, the middle
    /// stop at 0.5; `None` for NaN.
    fn position(&self, value: &f64, _offset: f64) -> Option<f64> {
        Some(self.fraction(*value)).filter(|t| !value.is_nan() && !t.is_nan())
    }

    /// The transform's default tick format for the set's count, under
    /// `locale`.
    fn labels(&self, ticks: &ticks::Set<f64, f64>, locale: &locale::Locale) -> Vec<String> {
        super::ticks::labels(
            self.tick_format_in(locale.number(), ticks.count(), None),
            ticks,
            locale,
        )
    }

    /// The transform's minor ticks between the majors, inside the end
    /// stops.
    fn minor(&self, ticks: &ticks::Set<f64, f64>) -> Vec<f64> {
        super::ticks::minor(&self.transform, self.ends(), ticks.values())
    }

    /// `[0, 1]`: a colorbar maps it to pixels.
    fn extent(&self) -> [f64; 2] {
        [0.0, 1.0]
    }
}
