//! [`Pad`]: a domain fitted to data, as matplotlib's autoscaling fits an
//! axis (margins, `nonsingular`, sticky edges), in each scale's transformed
//! space.

use crate::scale::{self, Continuous};

/// matplotlib's `Locator.nonsingular` expander: a flat extent widens by
/// 5 % of its magnitude per side, an empty or zero one to ±0.05.
const EXPANDER: f64 = 0.05;
/// `nonsingular`'s `tiny`: a span within this fraction of the magnitude is
/// flat.
const TINY: f64 = 1e-15;
/// How far inside the extent a sticky value still stops the margin,
/// relative to the span (matplotlib's streamplot tolerance).
const STICKY_TOLERANCE: f64 = 1e-5;
/// The domains tried, in order, when there is no data: matplotlib's
/// linear start [0, 1], `LogLocator.nonsingular`'s [1, 10] and
/// `LogitLocator.nonsingular`'s [1e-7, 1 − 1e-7] (D10.1).
const EMPTY: [[f64; 2]; 3] = [[0.0, 1.0], [1.0, 10.0], [1e-7, 1.0 - 1e-7]];

/// Domain fitting for one scale type: margins, nonsingular, sticky edges.
///
/// It is matplotlib's `autoscale_view` for one axis, in the transform's
/// space: an empty extent gets a default domain; a flat or tiny one is
/// widened by `nonsingular` (expander 0.05) in transformed space, which for
/// a log scale widens by 5 % of the logarithm where matplotlib widens by
/// decades; then each side gets `margin` of the transformed span, except
/// where a sticky value stops it. The result keeps the orientation of the
/// scale's domain, as an inverted matplotlib axis keeps its inversion.
///
/// ```
/// use rustd3_core::scale::{self, Pad};
///
/// assert_eq!(scale::linear().pad(Some([0.0, 10.0]), 0.05, &[]), [-0.5, 10.5]);
/// assert_eq!(scale::linear().pad(Some([0.0, 4.0]), 0.05, &[0.0]), [0.0, 4.2]);
/// assert_eq!(scale::log().pad(None, 0.05, &[]), [1.0, 10.0]);
/// let inverted = scale::linear().domain([10.0, 0.0]);
/// assert_eq!(inverted.pad(Some([0.0, 10.0]), 0.05, &[]), [10.5, -0.5]);
/// ```
pub trait Pad {
    /// The type of the domain's values.
    type Domain;

    /// `extent` is the merged data extent (`None`: no valid data); `margin`
    /// is a fraction of the span per side (themes use 0.05); `sticky`
    /// values are not crossed by the margin (matplotlib's sticky edges).
    /// The result has the scale's domain orientation.
    fn pad(
        &self,
        extent: Option<[Self::Domain; 2]>,
        margin: f64,
        sticky: &[Self::Domain],
    ) -> [Self::Domain; 2];
}

/// Fitted in the space of the transform as it applies to the fitted
/// domain; for N > 2 the end stops decide the orientation.
impl<T: scale::Transform, R, const N: usize> Pad for Continuous<T, R, N> {
    type Domain = f64;

    fn pad(&self, extent: Option<[f64; 2]>, margin: f64, sticky: &[f64]) -> [f64; 2] {
        let domain = self.get_domain();
        let fitted = fit(self.transform(), extent, margin, sticky);
        oriented(fitted, domain[0] > domain[N - 1])
    }
}

/// Fitted as a linear scale over its one array.
impl Pad for scale::Identity {
    type Domain = f64;

    fn pad(&self, extent: Option<[f64; 2]>, margin: f64, sticky: &[f64]) -> [f64; 2] {
        let [first, last] = self.get_domain();
        let fitted = fit(&scale::transform::Identity, extent, margin, sticky);
        oriented(fitted, first > last)
    }
}

/// matplotlib's `handle_single_axis` under `transform`, ascending.
fn fit<T: scale::Transform>(
    transform: &T,
    extent: Option<[f64; 2]>,
    margin: f64,
    sticky: &[f64],
) -> [f64; 2] {
    let Some(extent) = extent else {
        return empty(transform);
    };
    let extent = ascending(extent);
    let transform = transform.for_domain(extent);
    let [u0, u1] = nonsingular(extent.map(|x| transform.forward(x)));
    let [x0, x1] = ascending([transform.inverse(u0), transform.inverse(u1)]);
    // matplotlib's searchsorted: the largest sticky value below x0 + tol and
    // the smallest at or above x1 − tol.
    let tolerance = STICKY_TOLERANCE * (x1 - x0).abs();
    let low = sticky.iter().copied().filter(|&s| s < x0 + tolerance);
    let high = sticky.iter().copied().filter(|&s| s >= x1 - tolerance);
    let delta = (u1 - u0) * margin;
    let delta = if delta.is_finite() { delta } else { 0.0 };
    let [x0, x1] = ascending([transform.inverse(u0 - delta), transform.inverse(u1 + delta)]);
    [low.fold(x0, f64::max), high.fold(x1, f64::min)]
}

/// The first of [`EMPTY`] where the transform, as it applies there, is
/// finite at both ends; else the inverse of [0, 1].
fn empty<T: scale::Transform>(transform: &T) -> [f64; 2] {
    let finite = |ends: &[f64; 2]| {
        let at = transform.for_domain(*ends);
        ends.iter().all(|&x| at.forward(x).is_finite())
    };
    EMPTY
        .into_iter()
        .find(finite)
        .unwrap_or_else(|| [transform.inverse(0.0), transform.inverse(1.0)])
}

/// matplotlib's `nonsingular(u0, u1, expander=0.05)`, ascending: ±0.05 for
/// non-finite or vanishing ends (both 0 among them); a span within 1e-15
/// of the magnitude widened by 5 % of each end's magnitude.
fn nonsingular(u: [f64; 2]) -> [f64; 2] {
    if !u.iter().all(|v| v.is_finite()) {
        return [-EXPANDER, EXPANDER];
    }
    let [u0, u1] = ascending(u);
    let largest = u0.abs().max(u1.abs());
    if largest < (1e6 / TINY) * f64::MIN_POSITIVE {
        [-EXPANDER, EXPANDER]
    } else if u1 - u0 <= largest * TINY {
        [u0 - EXPANDER * u0.abs(), u1 + EXPANDER * u1.abs()]
    } else {
        [u0, u1]
    }
}

/// The pair in ascending order (a NaN stays where it is).
fn ascending([a, b]: [f64; 2]) -> [f64; 2] {
    if b < a { [b, a] } else { [a, b] }
}

/// The ascending pair, reversed for a descending domain.
fn oriented([low, high]: [f64; 2], descending: bool) -> [f64; 2] {
    if descending { [high, low] } else { [low, high] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonsingular_widens_flat_and_vanishing_pairs_as_matplotlib() {
        assert_eq!(nonsingular([0.0, 10.0]), [0.0, 10.0]);
        assert_eq!(nonsingular([10.0, 0.0]), [0.0, 10.0]);
        assert_eq!(nonsingular([3.0, 3.0]), [2.85, 3.15]);
        assert_eq!(nonsingular([-2.0, -2.0]), [-2.1, -1.9]);
        assert_eq!(nonsingular([0.0, 0.0]), [-0.05, 0.05]);
        assert_eq!(nonsingular([1e-300, 2e-300]), [-0.05, 0.05]);
        assert_eq!(nonsingular([f64::NAN, 1.0]), [-0.05, 0.05]);
        assert_eq!(nonsingular([0.0, f64::INFINITY]), [-0.05, 0.05]);
        let above = 1.0 + f64::EPSILON;
        assert_eq!(nonsingular([1.0, above]), [0.95, above + 0.05 * above]);
    }

    #[test]
    fn the_empty_domain_is_the_first_the_transform_maps() {
        use crate::scale::transform;
        assert_eq!(empty(&transform::Identity), [0.0, 1.0]);
        assert_eq!(empty(&transform::Log::new(10.0)), [1.0, 10.0]);
        assert_eq!(empty(&transform::Logit), [1e-7, 1.0 - 1e-7]);
        assert_eq!(empty(&transform::Symlog::new(1.0)), [0.0, 1.0]);
    }
}
