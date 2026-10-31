//! The capability traits for core's continuous color scales: sequential
//! and diverging scales over colors, and continuous scales with a color
//! range (`Linear<Rgb>`, piecewise or under a transform). Their output is
//! a color, so they ignore gestures, and they fit to the data's extent
//! with no margin, as matplotlib's `Normalize.autoscale_None` does: a
//! padded color domain wastes ramp.

use std::any::Any;

use rustd3_core::color::Rgb;
use rustd3_core::scale::{self, Continuous, Diverging, Pad, Sequential};

use crate::scale::{Kind, Paint, Scale, stops};

/// A color scale has no range in px and ignores gestures.
impl<T: scale::Transform + 'static> Scale for Sequential<Rgb, T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn boxed(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }

    /// The scale as it is.
    fn with_range(&self, _range: [f64; 2]) -> Box<dyn Scale> {
        self.boxed()
    }

    fn pan_px(&self, _range: [f64; 2], _dx: f64) -> Option<Box<dyn Scale>> {
        None
    }

    fn zoom_px(&self, _range: [f64; 2], _anchor: f64, _factor: f64) -> Option<Box<dyn Scale>> {
        None
    }
}

impl<T: scale::Transform + 'static> crate::sealed::Scale for Sequential<Rgb, T> {}

/// Fitted to the data's extent through core's `Pad` with no margin (the
/// theme's is ignored): a flat extent widens by `nonsingular`; no data
/// keeps the domain as built.
impl<T: scale::Transform + 'static> Kind for Sequential<Rgb, T> {
    type Domain = f64;
    type Output = Rgb;
    type Span = ();

    fn admits(&self, v: &f64) -> bool {
        Sequential::admits(self, *v)
    }

    fn merge(extent: &mut Option<[f64; 2]>, v: &f64) {
        widen(extent, *v);
    }

    fn fitted(&self, extent: Option<&[f64; 2]>, sticky: &[f64], _margin: f64) -> Self {
        match extent {
            Some(extent) => self.clone().domain(self.pad(Some(*extent), 0.0, sticky)),
            None => self.clone(),
        }
    }

    fn domain(&self) -> Option<[f64; 2]> {
        Some(self.get_domain())
    }
}

impl<T: scale::Transform + 'static> Paint for Sequential<Rgb, T> {
    /// The interpolator at `v`'s place; `None` for a value the scale does
    /// not admit (NaN, infinities, what its transform does not take, as
    /// matplotlib masks them) or a color that is not finite.
    fn paint(&self, v: &f64) -> Option<Rgb> {
        Sequential::admits(self, *v)
            .then(|| self.apply(*v))
            .and_then(finite)
    }
}

/// A color scale has no range in px and ignores gestures.
impl<T: scale::Transform + 'static> Scale for Diverging<Rgb, T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn boxed(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }

    /// The scale as it is.
    fn with_range(&self, _range: [f64; 2]) -> Box<dyn Scale> {
        self.boxed()
    }

    fn pan_px(&self, _range: [f64; 2], _dx: f64) -> Option<Box<dyn Scale>> {
        None
    }

    fn zoom_px(&self, _range: [f64; 2], _anchor: f64, _factor: f64) -> Option<Box<dyn Scale>> {
        None
    }
}

impl<T: scale::Transform + 'static> crate::sealed::Scale for Diverging<Rgb, T> {}

/// The end stops fitted to the data's extent through core's `Pad` with no
/// margin, the middle stop kept and never crossed: data entirely on one
/// side of it (`[2, 5]` over a middle of 0) gives `[0, 0, 5]`, so the ramp
/// never reverses. A descending domain stays descending; no data keeps
/// the domain as built.
impl<T: scale::Transform + 'static> Kind for Diverging<Rgb, T> {
    type Domain = f64;
    type Output = Rgb;
    type Span = ();

    fn admits(&self, v: &f64) -> bool {
        Diverging::admits(self, *v)
    }

    fn merge(extent: &mut Option<[f64; 2]>, v: &f64) {
        widen(extent, *v);
    }

    fn fitted(&self, extent: Option<&[f64; 2]>, sticky: &[f64], _margin: f64) -> Self {
        let Some(extent) = extent else {
            return self.clone();
        };
        let [a, b] = self.pad(Some(*extent), 0.0, sticky);
        let [_, middle, _] = self.get_domain();
        let low = a.min(b).min(middle);
        let high = a.max(b).max(middle);
        let domain = if a > b {
            [high, middle, low]
        } else {
            [low, middle, high]
        };
        self.clone().domain(domain)
    }

    /// The end stops.
    fn domain(&self) -> Option<[f64; 2]> {
        let [start, _, stop] = self.get_domain();
        Some([start, stop])
    }
}

impl<T: scale::Transform + 'static> Paint for Diverging<Rgb, T> {
    /// The interpolator at `v`'s place; `None` for a value the scale does
    /// not admit (NaN, infinities, what its transform does not take, as
    /// matplotlib masks them) or a color that is not finite.
    fn paint(&self, v: &f64) -> Option<Rgb> {
        Diverging::admits(self, *v)
            .then(|| self.apply(*v))
            .and_then(finite)
    }
}

/// A color scale has no range in px and ignores gestures.
impl<T: scale::Transform + 'static, const N: usize> Scale for Continuous<T, Rgb, N> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn boxed(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }

    /// The scale as it is.
    fn with_range(&self, _range: [f64; 2]) -> Box<dyn Scale> {
        self.boxed()
    }

    fn pan_px(&self, _range: [f64; 2], _dx: f64) -> Option<Box<dyn Scale>> {
        None
    }

    fn zoom_px(&self, _range: [f64; 2], _anchor: f64, _factor: f64) -> Option<Box<dyn Scale>> {
        None
    }
}

impl<T: scale::Transform + 'static, const N: usize> crate::sealed::Scale for Continuous<T, Rgb, N> {}

/// Fitted to the data's extent through core's `Pad` with no margin, inner
/// stops keeping their share of the way between the end stops; no data
/// keeps the domain as built.
impl<T: scale::Transform + 'static, const N: usize> Kind for Continuous<T, Rgb, N> {
    type Domain = f64;
    type Output = Rgb;
    type Span = ();

    fn admits(&self, v: &f64) -> bool {
        Continuous::admits(self, *v)
    }

    fn merge(extent: &mut Option<[f64; 2]>, v: &f64) {
        widen(extent, *v);
    }

    fn fitted(&self, extent: Option<&[f64; 2]>, sticky: &[f64], _margin: f64) -> Self {
        let Some(extent) = extent else {
            return self.clone();
        };
        let ends = self.pad(Some(*extent), 0.0, sticky);
        self.clone().domain(stops(&self.get_domain(), ends))
    }

    fn domain(&self) -> Option<[f64; 2]> {
        let d = self.get_domain();
        Some([d[0], d[N - 1]])
    }
}

impl<T: scale::Transform + 'static, const N: usize> Paint for Continuous<T, Rgb, N> {
    /// The range interpolated at `v`; `None` for a value the scale does
    /// not admit (NaN, infinities, what its transform does not take, as
    /// matplotlib masks them) or a color that is not finite.
    fn paint(&self, v: &f64) -> Option<Rgb> {
        Continuous::admits(self, *v)
            .then(|| self.apply(*v))
            .and_then(finite)
    }
}

/// Widens `extent` by `v`.
fn widen(extent: &mut Option<[f64; 2]>, v: f64) {
    let [low, high] = extent.get_or_insert([v, v]);
    *low = low.min(v);
    *high = high.max(v);
}

/// `c` when every channel is finite.
fn finite(c: Rgb) -> Option<Rgb> {
    [c.r, c.g, c.b, c.opacity]
        .iter()
        .all(|v| v.is_finite())
        .then_some(c)
}

#[cfg(test)]
mod tests {
    use rustd3_core::scheme;

    use super::*;

    #[test]
    fn color_scales_ignore_gestures_and_ranges() {
        let s = scale::sequential([0.0, 80.0], scheme::viridis);
        assert!(Scale::pan_px(&s, [0.0, 100.0], 5.0).is_none());
        assert!(Scale::zoom_px(&s, [0.0, 100.0], 50.0, 2.0).is_none());
        let ranged = Scale::with_range(&s, [0.0, 300.0]);
        let ranged = ranged
            .as_any()
            .downcast_ref::<Sequential<Rgb>>()
            .expect("its type");
        assert_eq!(ranged.get_domain(), [0.0, 80.0]);
    }

    #[test]
    fn a_diverging_middle_is_never_crossed() {
        let d = scale::diverging([-1.0, 0.0, 1.0], scheme::rd_bu);
        assert_eq!(
            d.fitted(Some(&[2.0, 5.0]), &[], 0.05).get_domain(),
            [0.0, 0.0, 5.0]
        );
        assert_eq!(
            d.fitted(Some(&[-3.0, 2.0]), &[], 0.05).get_domain(),
            [-3.0, 0.0, 2.0]
        );
        assert_eq!(
            d.fitted(Some(&[-3.0, -2.0]), &[], 0.05).get_domain(),
            [-3.0, 0.0, 0.0]
        );
        let reversed = d.clone().domain([1.0, 0.0, -1.0]);
        let fitted = reversed.fitted(Some(&[2.0, 5.0]), &[], 0.05).get_domain();
        assert_eq!(fitted, [5.0, 0.0, 0.0]);
        assert_eq!(d.fitted(None, &[], 0.05).get_domain(), [-1.0, 0.0, 1.0]);
        assert_eq!(Kind::domain(&d), Some([-1.0, 1.0]));
    }

    #[test]
    fn color_ranges_fit_without_margin_and_keep_inner_stops_in_proportion() {
        let red = Rgb::new(255.0, 0.0, 0.0);
        let blue = Rgb::new(0.0, 0.0, 255.0);
        let white = Rgb::new(255.0, 255.0, 255.0);
        let l = scale::piecewise([-1.0, 0.0, 1.0], [red, white, blue]);
        let fitted = l.fitted(Some(&[2.0, 6.0]), &[], 0.05);
        assert_eq!(fitted.get_domain(), [2.0, 4.0, 6.0]);
        assert_eq!(fitted.paint(&4.0), Some(white));
        assert_eq!(l.fitted(None, &[], 0.05).get_domain(), [-1.0, 0.0, 1.0]);
        let flat = scale::linear()
            .range([red, blue])
            .fitted(Some(&[3.0, 3.0]), &[], 0.05);
        assert_eq!(Kind::domain(&flat), Some([2.85, 3.15]));
        assert_eq!(scale::linear().range([red, blue]).paint(&f64::NAN), None);
    }
}
