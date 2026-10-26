//! d3's continuous scale (continuous.js), generic over its transform, its
//! range type and its number of stops.

use std::fmt;
use std::sync::Arc;

use crate::format::{InvalidFormat, Locale};
use crate::interpolate::{self, Interpolate};
use crate::js;
use crate::scale::{TickFormat, Transform};

/// A continuous scale: `N` domain stops mapped through a transform onto `N`
/// range values of any interpolatable type `R` (d3's scaleLinear, scalePow,
/// scaleLog and their kin). Builders consume and return the value; ticks,
/// nice and tick formats act on the end stops, as in d3.
///
/// Only two stops can be built in this version: the constructors make
/// them, and `domain` and `range` keep `N`.
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
        Continuous {
            transform: transform.clone(),
            domain: [0.0, 1.0],
            forward: [0.0, 1.0],
            range: [0.0, 1.0],
            interpolate: Interpolator::Lerp,
            clamp: false,
            unknown: None,
        }
        .transformed(transform)
    }
}

impl<T: Transform, R: Interpolate, const N: usize> Continuous<T, R, N> {
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
        piecewise(&self.forward, &self.range, u, |a, b, t| {
            match &self.interpolate {
                Interpolator::Lerp => a.lerp(b, t),
                Interpolator::Custom(f) => f(a, b, t),
            }
        })
    }

    /// About `count` ticks over the end stops, by the transform's rule.
    pub fn ticks(&self, count: usize) -> Vec<f64> {
        self.transform.ticks(self.ends(), count)
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
        let u = piecewise(&self.range, &self.forward, y, |a, b, t| a.lerp(b, t));
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

/// d3's bimap and polymap: `u` placed among `stops` (both arrays reversed
/// when the last stop is below the first), normalized within its segment,
/// and the segment's two `values` blended at that fraction.
fn piecewise<V, const N: usize>(
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
    use crate::scale::{self, transform};

    /// A three-stop linear scale, which only Phase 5's `piecewise` builds
    /// from outside.
    fn three(domain: [f64; 3], range: [f64; 3]) -> Continuous<transform::Identity, f64, 3> {
        let two = scale::linear();
        Continuous {
            transform: two.transform,
            domain,
            forward: domain,
            range,
            interpolate: Interpolator::Lerp,
            clamp: false,
            unknown: None,
        }
    }

    #[test]
    fn three_stops_pick_the_segment_of_the_value() {
        let s = three([-10.0, 0.0, 100.0], [0.0, 1.0, 2.0]);
        assert_eq!(s.apply(-5.0), 0.5);
        assert_eq!(s.apply(50.0), 1.5);
        assert_eq!(s.invert(1.5), 50.0);
    }

    #[test]
    fn three_descending_stops_are_reversed_together_with_the_range() {
        let s = three([4.0, 2.0, 1.0], [1.0, 2.0, 4.0]);
        assert_eq!(s.apply(1.5), 3.0);
        assert_eq!(s.apply(3.0), 1.5);
        assert_eq!(s.invert(1.5), 3.0);
        assert_eq!(s.invert(3.0), 1.5);
    }

    #[test]
    fn nice_on_three_stops_moves_only_the_ends() {
        let s = three([1.1, 2.0, 10.9], [0.0, 1.0, 2.0]).nice(10);
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
