//! d3's continuous scale (continuous.js), generic over its transform, its
//! range type and its number of stops.

use std::fmt;
use std::sync::Arc;

use crate::format::{InvalidFormat, Locale};
use crate::interpolate::{self, Interpolate};
use crate::js;
use crate::scale::{TickFormat, TickRule, Transform, transform};

/// A continuous scale: `N` domain stops mapped through a transform onto `N`
/// range values of any interpolatable type `R` (d3's scaleLinear, scalePow,
/// scaleLog and their kin). Builders consume and return the value; ticks,
/// nice and tick formats act on the end stops, as in d3.
///
/// The constructors make two stops and [`piecewise`](crate::scale::piecewise)
/// any N ≥ 2; `domain` and `range` keep `N`.
#[derive(Clone)]
pub struct Continuous<T, R = f64, const N: usize = 2> {
    transform: T,
    domain: [f64; N],
    /// `transform.forward` of each stop, kept current by the builders.
    forward: [f64; N],
    range: [R; N],
    interpolate: Interpolator<R>,
    clamp: bool,
    unknown: Option<R>,
    /// The rule that replaces the transform's own, when set.
    tick_rule: Option<TickRule>,
}

/// How two range values blend: their own `lerp`, or the caller's function.
#[derive(Clone)]
enum Interpolator<R> {
    Lerp,
    Custom(Blend<R>),
}

/// A caller's blend of two range values at `t`, shared between clones.
type Blend<R> = Arc<dyn Fn(&R, &R, f64) -> R + Send + Sync>;

impl<T: fmt::Debug, R: fmt::Debug, const N: usize> fmt::Debug for Continuous<T, R, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Continuous")
            .field("transform", &self.transform)
            .field("domain", &self.domain)
            .field("range", &self.range)
            .field("interpolate", &self.interpolate)
            .field("clamp", &self.clamp)
            .field("unknown", &self.unknown)
            .field("tick_rule", &self.tick_rule)
            .finish_non_exhaustive()
    }
}

impl<R> fmt::Debug for Interpolator<R> {
    /// `lerp`, or `custom` for a function, which has no Debug.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Interpolator::Lerp => "lerp",
            Interpolator::Custom(_) => "custom",
        })
    }
}

impl<T: Transform> Continuous<T> {
    /// Domain [0, 1] and range [0, 1] under `transform`.
    pub(super) fn new(transform: T) -> Continuous<T> {
        Continuous::with_stops(transform, [0.0, 1.0], [0.0, 1.0])
    }
}

impl<T, R, const N: usize> Continuous<T, R, N> {
    /// The scale under the transform `f` makes of this one's, every other
    /// field moved across; `f` must keep the map, since the forward cache
    /// is kept.
    pub(crate) fn map_transform<U>(self, f: impl FnOnce(T) -> U) -> Continuous<U, R, N> {
        Continuous {
            transform: f(self.transform),
            domain: self.domain,
            forward: self.forward,
            range: self.range,
            interpolate: self.interpolate,
            clamp: self.clamp,
            unknown: self.unknown,
            tick_rule: self.tick_rule,
        }
    }
}

impl<T: Transform, R: Interpolate, const N: usize> Continuous<T, R, N> {
    /// `domain` mapped onto `range` under `transform`, by `lerp`, unclamped.
    pub(super) fn with_stops(transform: T, domain: [f64; N], range: [R; N]) -> Self {
        Continuous {
            transform: transform.clone(),
            domain,
            forward: domain,
            range,
            interpolate: Interpolator::Lerp,
            clamp: false,
            unknown: None,
            tick_rule: None,
        }
        .transformed(transform)
    }

    /// d3's `domain(d)`: the stops, mapped onto the range's in order.
    pub fn domain(self, domain: [f64; N]) -> Self {
        let transform = self.transform.clone();
        Continuous { domain, ..self }.transformed(transform)
    }

    /// d3's `range(r)`: the values the stops map to, of any interpolatable
    /// type; a new type resets the interpolator to `lerp` and clears the
    /// unknown value.
    pub fn range<S: Interpolate>(self, range: [S; N]) -> Continuous<T, S, N> {
        Continuous {
            transform: self.transform,
            domain: self.domain,
            forward: self.forward,
            range,
            interpolate: Interpolator::Lerp,
            clamp: self.clamp,
            unknown: None,
            tick_rule: self.tick_rule,
        }
    }

    /// d3's `interpolate(f)`: blend neighbouring range values `a` and `b`
    /// at `t` with `f(a, b, t)` instead of `lerp`.
    pub fn interpolate(self, f: impl Fn(&R, &R, f64) -> R + Send + Sync + 'static) -> Self {
        Continuous {
            interpolate: Interpolator::Custom(Arc::new(f)),
            ..self
        }
    }

    /// d3's `clamp(b)`: whether `apply` and `invert` keep to the domain.
    pub fn clamp(self, clamp: bool) -> Self {
        Continuous { clamp, ..self }
    }

    /// d3's `unknown(v)`: what `apply` returns for NaN.
    pub fn unknown(self, value: R) -> Self {
        Continuous {
            unknown: Some(value),
            ..self
        }
    }

    /// d3's `nice(count)`: the end stops extended to round values by the
    /// transform's rule.
    pub fn nice(self, count: usize) -> Self {
        let [start, stop] = self.transform.nice(self.ends(), count);
        let mut domain = self.domain;
        domain[0] = start;
        domain[N - 1] = stop;
        self.domain(domain)
    }

    /// The range value of `x` (d3's `scale(x)`): `unknown` for NaN when set,
    /// otherwise NaN flows through as d3's arithmetic carries it.
    pub fn apply(&self, x: f64) -> R {
        if x.is_nan()
            && let Some(unknown) = &self.unknown
        {
            return unknown.clone();
        }
        let x = if self.clamp { self.clamped(x) } else { x };
        let u = self.transform.forward(x);
        polymap(&self.forward, &self.range, u, |a, b, t| {
            match &self.interpolate {
                Interpolator::Lerp => a.lerp(b, t),
                Interpolator::Custom(f) => f(a, b, t),
            }
        })
    }

    /// Replaces the transform's tick rule with `rule` (not `.ticks(…)`,
    /// which is d3's and takes a count).
    ///
    /// ```
    /// use rustd3_core::scale::{self, TickRule};
    ///
    /// let s = scale::linear()
    ///     .domain([0.0, 12.0])
    ///     .tick_rule(TickRule::custom(|[a, b], _| {
    ///         (0..).map(|i| a + 3.0 * f64::from(i)).take_while(|v| *v <= b).collect()
    ///     }));
    /// assert_eq!(s.ticks(5), [0.0, 3.0, 6.0, 9.0, 12.0]);
    /// ```
    pub fn tick_rule(self, rule: TickRule) -> Self {
        Continuous {
            tick_rule: Some(rule),
            ..self
        }
    }

    /// About `count` ticks over the end stops, by the tick rule when one is
    /// set and the transform's rule otherwise.
    pub fn ticks(&self, count: usize) -> Vec<f64> {
        match &self.tick_rule {
            Some(rule) => rule.ticks(&self.transform, self.ends(), count),
            None => self.transform.ticks(self.ends(), count),
        }
    }

    /// The transform's minor ticks between `ticks(count)`.
    pub fn minor(&self, count: usize) -> Vec<f64> {
        self.transform.minor(&self.ticks(count))
    }

    /// d3's `tickFormat(count, specifier)` under the neutral locale.
    pub fn tick_format(
        &self,
        count: usize,
        specifier: Option<&str>,
    ) -> Result<TickFormat, InvalidFormat> {
        self.tick_format_in(&Locale::NEUTRAL, count, specifier)
    }

    /// d3's `tickFormat(count, specifier)` under `locale`.
    pub fn tick_format_in(
        &self,
        locale: &Locale,
        count: usize,
        specifier: Option<&str>,
    ) -> Result<TickFormat, InvalidFormat> {
        self.transform
            .tick_format(locale, self.ends(), count, specifier)
    }

    /// d3's `domain()`.
    pub fn get_domain(&self) -> [f64; N] {
        self.domain
    }

    /// d3's `range()`.
    pub fn get_range(&self) -> &[R; N] {
        &self.range
    }

    /// The transform, as set for the current domain.
    pub(crate) fn transform(&self) -> &T {
        &self.transform
    }

    /// The first and last stop, which ticks, nice and formats act on.
    fn ends(&self) -> [f64; 2] {
        [self.domain[0], self.domain[N - 1]]
    }

    /// `x` kept between the end stops, as d3's clamper: NaN stays NaN.
    fn clamped(&self, x: f64) -> f64 {
        let [a, b] = self.ends();
        let (low, high) = if a > b { (b, a) } else { (a, b) };
        js::max(low, js::min(high, x))
    }

    /// The scale with `transform`, set for the current domain, and the
    /// forward cache refreshed.
    fn transformed(mut self, transform: T) -> Self {
        self.transform = transform.for_domain(self.ends());
        self.forward = self.domain.map(|x| self.transform.forward(x));
        self
    }
}

impl<T: Transform, const N: usize> Continuous<T, f64, N> {
    /// d3's `invert(y)`: the domain value that maps to `y`. It exists only
    /// for `f64` ranges; d3 returns NaN for others at run time.
    ///
    /// ```compile_fail
    /// use rustd3_core::scale;
    ///
    /// scale::linear().range([0.0f32, 1.0]).invert(0.5);
    /// ```
    ///
    /// ```
    /// use rustd3_core::scale;
    ///
    /// assert_eq!(scale::linear().range([0.0f64, 1.0]).invert(0.5), 0.5);
    /// ```
    pub fn invert(&self, y: f64) -> f64 {
        let u = polymap(&self.range, &self.forward, y, |a, b, t| a.lerp(b, t));
        let x = self.transform.inverse(u);
        if self.clamp { self.clamped(x) } else { x }
    }

    /// d3's `rangeRound(r)`: `range(r)` with values rounded by
    /// [`interpolate::round`].
    pub fn range_round(self, range: [f64; N]) -> Self {
        self.range(range)
            .interpolate(|a: &f64, b: &f64, t| interpolate::round(*a, *b)(t))
    }
}

impl<R: Interpolate, const N: usize> Continuous<transform::Pow, R, N> {
    /// d3's `exponent(e)`: the power the domain is raised to; the domain
    /// and range stay as they are.
    pub fn exponent(self, exponent: f64) -> Self {
        self.transformed(transform::Pow::new(exponent))
    }
}

impl<R: Interpolate, const N: usize> Continuous<transform::Log, R, N> {
    /// d3's `base(b)`: the base of the ticks, nice and labels; the map is
    /// the same in every base. It exists only for log scales:
    ///
    /// ```compile_fail
    /// use rustd3_core::scale;
    ///
    /// scale::linear().base(2.0);
    /// ```
    ///
    /// ```
    /// use rustd3_core::scale;
    ///
    /// let x = scale::log().base(2.0).domain([1.0, 1024.0]);
    /// assert_eq!(x.ticks(10)[..4], [1.0, 2.0, 4.0, 8.0]);
    /// ```
    pub fn base(self, base: f64) -> Self {
        self.transformed(transform::Log::new(base))
    }
}

impl<R: Interpolate, const N: usize> Continuous<transform::Symlog, R, N> {
    /// d3's `constant(c)`: where the symlog turns from linear to
    /// logarithmic; the domain and range stay as they are. It exists only
    /// for symlog scales.
    pub fn constant(self, constant: f64) -> Self {
        self.transformed(transform::Symlog::new(constant))
    }
}

impl<R: Interpolate, const N: usize> Continuous<transform::Asinh, R, N> {
    /// The width around 0 within which the asinh scale is nearly linear;
    /// the domain and range stay as they are. It exists only for asinh
    /// scales.
    pub fn linear_width(self, linear_width: f64) -> Self {
        self.transformed(transform::Asinh::new(linear_width))
    }
}

/// d3's bimap and polymap: `u` placed among `stops` (both arrays reversed
/// when the last stop is below the first), normalized within its segment,
/// and the segment's two `values` blended at that fraction.
fn polymap<V, const N: usize>(
    stops: &[f64; N],
    values: &[V; N],
    u: f64,
    blend: impl Fn(&V, &V, f64) -> V,
) -> V {
    let last = N - 1;
    let reversed = stops[last] < stops[0];
    let at = |k: usize| if reversed { last - k } else { k };
    let i = if N > 2 {
        let mut ordered = *stops;
        if reversed {
            ordered.reverse();
        }
        ordered[1..last].partition_point(|&stop| stop <= u)
    } else {
        0
    };
    let (a, b) = (at(i), at(i + 1));
    blend(&values[a], &values[b], normalize(stops[a], stops[b], u))
}

/// d3's normalize: where `x` lies from `a` (0) to `b` (1); 0.5 for an
/// empty span and NaN for a NaN one.
fn normalize(a: f64, b: f64, x: f64) -> f64 {
    let span = b - a;
    if span.is_nan() {
        f64::NAN
    } else if span == 0.0 {
        0.5
    } else {
        (x - a) / span
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale;

    #[test]
    fn three_stops_pick_the_segment_of_the_value() {
        let s = scale::piecewise([-10.0, 0.0, 100.0], [0.0, 1.0, 2.0]);
        assert_eq!(s.apply(-5.0), 0.5);
        assert_eq!(s.apply(50.0), 1.5);
        assert_eq!(s.invert(1.5), 50.0);
    }

    #[test]
    fn three_descending_stops_are_reversed_together_with_the_range() {
        let s = scale::piecewise([4.0, 2.0, 1.0], [1.0, 2.0, 4.0]);
        assert_eq!(s.apply(1.5), 3.0);
        assert_eq!(s.apply(3.0), 1.5);
        assert_eq!(s.invert(1.5), 3.0);
        assert_eq!(s.invert(3.0), 1.5);
    }

    #[test]
    fn nice_on_three_stops_moves_only_the_ends() {
        let s = scale::piecewise([1.1, 2.0, 10.9], [0.0, 1.0, 2.0]).nice(10);
        assert_eq!(s.get_domain(), [1.0, 2.0, 11.0]);
    }

    #[test]
    fn normalize_maps_an_empty_span_to_the_middle() {
        assert_eq!(normalize(1.0, 1.0, 7.0), 0.5);
        assert!(normalize(f64::NAN, 1.0, 0.0).is_nan());
        assert_eq!(normalize(0.0, 4.0, 1.0), 0.25);
    }

    #[test]
    fn a_custom_interpolator_prints_as_custom() {
        let s = scale::linear().interpolate(|a: &f64, _: &f64, _| *a);
        assert!(format!("{s:?}").contains("interpolate: custom"));
        assert!(format!("{:?}", scale::linear()).contains("interpolate: lerp"));
    }
}
