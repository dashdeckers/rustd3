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
    fn fitted(
        &self,
        extent: Option<&[Self::Domain; 2]>,
        sticky: &[Self::Domain],
        margin: f64,
    ) -> Self;

    /// The end stops of a continuous domain, in its own type (numbers,
    /// instants, a color scale's end stops); `None` for band, point,
    /// ordinal and binned scales.
    fn domain(&self) -> Option<[Self::Domain; 2]>;
}
