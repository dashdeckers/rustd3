//! [`Rescale`]: what a zoom does to a scale, through d3's
//! `transform.rescaleX` or the pixel-space gestures, for continuous scales
//! and `scale::Identity`.

use crate::interact;
use crate::interpolate::Interpolate;
use crate::scale::{self, Continuous};

/// The smallest span a gesture leaves, relative to the domain's centre:
/// 2⁻⁴⁰, about 9.1e-13, some 4,000 ulps.
const MIN_SPAN: f64 = 1.0 / (1u64 << 40) as f64;

/// A scale a zoom can rescale.
///
/// [`rescale_x`](Rescale::rescale_x) and [`rescale_y`](Rescale::rescale_y)
/// are d3's, through `invert`, so a clamping scale cannot zoom out, as in
/// d3. The pixel-space gestures [`pan_px`](Rescale::pan_px) and
/// [`zoom_px`](Rescale::zoom_px) work in the transform's space without
/// clamping: each stop becomes the value the piecewise-linear map from the
/// range to transformed values, extended past the end stops, gives at the
/// moved range position. They return `None`, and the gesture keeps its
/// transform, when a new stop or its transformed value is not finite, when
/// the stops leave the old domain's order, when the factor is not finite
/// and positive, or when the span is 0 or below 2⁻⁴⁰ of the domain's
/// centre.
///
/// ```
/// use rustd3_core::interact::{self, Rescale};
/// use rustd3_core::scale;
///
/// let x = scale::linear().domain([0.0, 100.0]).range([0.0, 960.0]);
/// let t = interact::Transform::IDENTITY.translate(-480.0, 0.0).scale(2.0);
/// assert_eq!(x.rescale_x(&t).get_domain(), [25.0, 75.0]);
/// assert_eq!(x.zoom_px(480.0, 2.0).map(|x| x.get_domain()), Some([25.0, 75.0]));
/// assert_eq!(x.pan_px(96.0).map(|x| x.get_domain()), Some([-10.0, 90.0]));
/// assert!(scale::linear().range([0.0, 960.0]).zoom_px(480.0, 1e13).is_none());
/// ```
pub trait Rescale: Sized {
    /// d3's `transform.rescaleX(scale)`: domain =
    /// range.map(invertX).map(scale.invert).
    #[must_use]
    fn rescale_x(&self, t: &interact::Transform) -> Self;

    /// d3's `transform.rescaleY(scale)`.
    #[must_use]
    fn rescale_y(&self, t: &interact::Transform) -> Self;

    /// Content follows the pointer by `dx` range units (pixels): the value
    /// shown at range position r moves to r + dx. `None` if degenerate.
    fn pan_px(&self, dx: f64) -> Option<Self>;

    /// Scales by `factor` around the range position `anchor`; > 1 zooms in.
    /// `None` if degenerate.
    fn zoom_px(&self, anchor: f64, factor: f64) -> Option<Self>;
}

impl<T: scale::Transform, const N: usize> Rescale for Continuous<T, f64, N> {
    fn rescale_x(&self, t: &interact::Transform) -> Self {
        let domain = self.get_range().map(|r| self.invert(t.invert_x(r)));
        self.clone().domain(domain)
    }

    fn rescale_y(&self, t: &interact::Transform) -> Self {
        let domain = self.get_range().map(|r| self.invert(t.invert_y(r)));
        self.clone().domain(domain)
    }

    fn pan_px(&self, dx: f64) -> Option<Self> {
        moved(self, |r| r - dx)
    }

    fn zoom_px(&self, anchor: f64, factor: f64) -> Option<Self> {
        zoomable(factor)?;
        moved(self, |r| anchor + (r - anchor) / factor)
    }
}

/// d3's scaleIdentity: setting the domain sets the range, so a rescaled
/// identity scale shows the new values at the new positions.
impl Rescale for scale::Identity {
    fn rescale_x(&self, t: &interact::Transform) -> Self {
        let domain = self.get_range().map(|r| self.invert(t.invert_x(r)));
        self.clone().domain(domain)
    }

    fn rescale_y(&self, t: &interact::Transform) -> Self {
        let domain = self.get_range().map(|r| self.invert(t.invert_y(r)));
        self.clone().domain(domain)
    }

    fn pan_px(&self, dx: f64) -> Option<Self> {
        let stops = self.get_range().map(|r| r - dx);
        accepted(self.get_domain(), stops).map(|domain| self.clone().domain(domain))
    }

    fn zoom_px(&self, anchor: f64, factor: f64) -> Option<Self> {
        zoomable(factor)?;
        let stops = self.get_range().map(|r| anchor + (r - anchor) / factor);
        accepted(self.get_domain(), stops).map(|domain| self.clone().domain(domain))
    }
}

/// `scale` with each stop moved to the value its range position `to(r)`
/// shows, unclamped; `None` if degenerate.
fn moved<T: scale::Transform, const N: usize>(
    scale: &Continuous<T, f64, N>,
    to: impl Fn(f64) -> f64,
) -> Option<Continuous<T, f64, N>> {
    let range = scale.get_range();
    let stops = range.map(|r| {
        let u = scale::polymap(range, scale.forward(), to(r), |a, b, t| a.lerp(b, t));
        scale.transform().inverse(u)
    });
    let moved = scale.clone().domain(accepted(scale.get_domain(), stops)?);
    moved
        .forward()
        .iter()
        .all(|u| u.is_finite())
        .then_some(moved)
}

/// `Some` for a finite, positive zoom factor.
fn zoomable(factor: f64) -> Option<()> {
    (factor.is_finite() && factor > 0.0).then_some(())
}

/// `stops` if they can replace `old`: all finite, in `old`'s order (end
/// to end, and no neighbours reversed), and spanning more than 0 and at
/// least [`MIN_SPAN`] of their centre.
fn accepted<const N: usize>(old: [f64; N], stops: [f64; N]) -> Option<[f64; N]> {
    let direction = old[N - 1].partial_cmp(&old[0])?;
    let (first, last) = (stops[0], stops[N - 1]);
    let ordered = last.partial_cmp(&first) == Some(direction)
        && stops
            .windows(2)
            .all(|pair| pair[1].partial_cmp(&pair[0]) != Some(direction.reverse()));
    let span = (last - first).abs();
    let wide = span > 0.0 && span >= MIN_SPAN * ((first + last) / 2.0).abs();
    (stops.iter().all(|d| d.is_finite()) && ordered && wide).then_some(stops)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_span_is_two_to_the_minus_forty() {
        assert_eq!(MIN_SPAN, 2f64.powi(-40));
    }

    #[test]
    fn accepted_keeps_the_order_of_the_old_domain() {
        assert_eq!(accepted([0.0, 1.0], [2.0, 3.0]), Some([2.0, 3.0]));
        assert_eq!(accepted([1.0, 0.0], [3.0, 2.0]), Some([3.0, 2.0]));
        assert_eq!(accepted([0.0, 1.0], [3.0, 2.0]), None);
        assert_eq!(accepted([0.0, 5.0, 10.0], [9.0, 4.0, 49.0]), None);
        assert_eq!(
            accepted([0.0, 5.0, 5.0, 10.0], [1.0, 3.0, 3.0, 4.0]),
            Some([1.0, 3.0, 3.0, 4.0])
        );
        assert_eq!(accepted([0.0, f64::NAN], [0.0, 1.0]), None);
    }

    #[test]
    fn accepted_rejects_flat_tiny_and_non_finite_spans() {
        assert_eq!(accepted([5.0, 5.0], [5.0, 5.0]), None);
        assert_eq!(accepted([0.0, 1.0], [1.0, 1.0]), None);
        assert_eq!(accepted([0.0, 1.0], [0.0, f64::INFINITY]), None);
        assert_eq!(accepted([0.0, 1.0], [f64::NAN, 1.0]), None);
        assert_eq!(
            accepted([0.0, 1.0], [-1e-300, 1e-300]),
            Some([-1e-300, 1e-300])
        );
        assert!(accepted([0.0, 1.0], [1e6 - 1e-6, 1e6 + 1e-6]).is_some());
        assert_eq!(accepted([0.0, 1.0], [1e6 - 1e-7, 1e6 + 1e-7]), None);
    }
}
