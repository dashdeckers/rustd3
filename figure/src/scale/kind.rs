//! [`Kind`]: what a handle knows about its scale at compile time.

use crate::scale::Scale;

/// What a handle knows about its scale at compile time: the types of its
/// domain, its output and its span, and how its domain is fitted to data.
pub trait Kind: Scale + Clone {
    /// The values the scale takes: `f64`, a band key, a timestamp.
    type Domain: Clone + 'static;
    /// What the scale gives: a position in px, or a color.
    type Output;
    /// What `Fit::Window` measures: `f64` for numeric scales,
    /// `jiff::SignedDuration` for time scales, `()` for categorical ones.
    type Span: Copy + 'static;

    /// Whether `v` is data the domain can be fitted to: finite and valid
    /// under the transform (log: > 0); a key of the domain for categorical
    /// scales.
    fn admits(&self, v: &Self::Domain) -> bool;

    /// Widens `extent` by `v` in the domain's own order; categorical scales
    /// leave it as it is.
    fn merge(extent: &mut Option<[Self::Domain; 2]>, v: &Self::Domain);

    /// This scale with its domain fitted to `extent` (`None`: no admitted
    /// data) through core's `scale::Pad`: `margin` of the span per side,
    /// stopped by `sticky` values. Categorical scales keep their domain.
    #[must_use]
    fn fitted(
        &self,
        extent: Option<&[Self::Domain; 2]>,
        sticky: &[Self::Domain],
        margin: f64,
    ) -> Self;

    /// This scale fitted with its ends rounded outward to `count` nice
    /// ticks instead of margins (the theme's `Domains::Nice`); sticky
    /// values still stop the rounding. Default: `self.fitted(extent,
    /// sticky, 0.0)`, which band, point, ordinal, color and binned scales
    /// keep; continuous scales round with core's `nice(count)`, time
    /// scales to the tick interval for `count`.
    #[must_use]
    fn niced(
        &self,
        extent: Option<&[Self::Domain; 2]>,
        sticky: &[Self::Domain],
        count: usize,
    ) -> Self {
        let _ = count;
        self.fitted(extent, sticky, 0.0)
    }

    /// The two ends of the domain as the scale maps it, in its own type
    /// (numbers, instants, a color scale's end stops); `None` for a
    /// categorical scale (band, point, ordinal) and a binned one, which map
    /// keys and bins, not a span. Unlike the scale's `get_domain`, the
    /// domain as stored (two stops, a diverging scale's three, a piecewise
    /// scale's N, a categorical scale's keys), it is the first and last
    /// stop only. Call it as `Kind::ends(&scale)`: core's
    /// [`Ticks::ends`](rustd3_core::scale::Ticks::ends), on the same scales,
    /// is the range's ends.
    fn ends(&self) -> Option<[Self::Domain; 2]>;
}
