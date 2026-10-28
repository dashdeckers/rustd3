//! The capability traits for core's binned color scales: quantize,
//! quantile and threshold scales over colors. They paint, ignore gestures
//! and tick at their tick sets. Quantize fits its domain to the data's
//! extent with no margin; a threshold scale's thresholds are its domain.
//!
//! A quantile scale's domain is the data sample itself, while fitting
//! keeps an extent and sticky values per handle and never collects
//! samples, so a quantile scale is not `Fittable`: build it from the data
//! and add it with `fig.scale` (E0277 with `fig.fit`).
//!
//! ```compile_fail,E0277
//! # use rustd3_figure::Figure;
//! # use rustd3_core::{scale, scheme};
//! let mut fig = Figure::new();
//! let c = fig.fit(scale::quantile([1.0, 2.0, 3.0, 10.0], scheme::CATEGORY10)); // E0277
//! ```
//!
//! ```
//! # use rustd3_figure::Figure;
//! # use rustd3_core::{scale, scheme};
//! let mut fig = Figure::new();
//! let c = fig.scale(scale::quantile([1.0, 2.0, 3.0, 10.0], scheme::CATEGORY10));
//! ```

use std::any::Any;

use rustd3_core::color::Rgb;
use rustd3_core::scale::{Pad, Quantile, Quantize, Threshold};

use crate::scale::{Kind, Paint, Scale};

/// A binned scale has no range in px and ignores gestures.
impl Scale for Quantize<Rgb> {
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

/// Fitted to the data's extent through core's `Pad` with no margin (the
/// theme's is ignored), the bins following; no data keeps the domain as
/// built.
impl Kind for Quantize<Rgb> {
    type Domain = f64;
    type Output = Rgb;
    type Span = ();

    /// Finite values.
    fn admits(&self, v: &f64) -> bool {
        v.is_finite()
    }

    fn merge(extent: &mut Option<[f64; 2]>, &v: &f64) {
        let [low, high] = extent.get_or_insert([v, v]);
        *low = low.min(v);
        *high = high.max(v);
    }

    fn fitted(&self, extent: Option<&[f64; 2]>, sticky: &[f64], _margin: f64) -> Self {
        match extent {
            Some(extent) => self.clone().domain(self.pad(Some(*extent), 0.0, sticky)),
            None => self.clone(),
        }
    }

    /// `None`: a binned scale has bins, not a continuous domain.
    fn domain(&self) -> Option<[f64; 2]> {
        None
    }
}

impl Paint for Quantize<Rgb> {
    fn paint(&self, v: &f64) -> Option<Rgb> {
        self.apply(*v).copied()
    }
}

/// A binned scale has no range in px and ignores gestures.
impl Scale for Quantile<Rgb> {
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

/// Its sample is its domain: not fitted (no `Fittable`), so `fitted`
/// keeps it.
impl Kind for Quantile<Rgb> {
    type Domain = f64;
    type Output = Rgb;
    type Span = ();

    /// Finite values.
    fn admits(&self, v: &f64) -> bool {
        v.is_finite()
    }

    fn merge(_extent: &mut Option<[f64; 2]>, _v: &f64) {}

    fn fitted(&self, _extent: Option<&[f64; 2]>, _sticky: &[f64], _margin: f64) -> Self {
        self.clone()
    }

    /// `None`: a binned scale has bins, not a continuous domain.
    fn domain(&self) -> Option<[f64; 2]> {
        None
    }
}

impl Paint for Quantile<Rgb> {
    fn paint(&self, v: &f64) -> Option<Rgb> {
        self.apply(*v).copied()
    }
}

/// A binned scale has no range in px and ignores gestures.
impl Scale for Threshold<Rgb> {
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

/// Its thresholds are its domain: fitting keeps them.
impl Kind for Threshold<Rgb> {
    type Domain = f64;
    type Output = Rgb;
    type Span = ();

    /// Values that are not NaN: every other value falls in a bin.
    fn admits(&self, v: &f64) -> bool {
        !v.is_nan()
    }

    fn merge(_extent: &mut Option<[f64; 2]>, _v: &f64) {}

    fn fitted(&self, _extent: Option<&[f64; 2]>, _sticky: &[f64], _margin: f64) -> Self {
        self.clone()
    }

    /// `None`: a binned scale has bins, not a continuous domain.
    fn domain(&self) -> Option<[f64; 2]> {
        None
    }
}

impl Paint for Threshold<Rgb> {
    fn paint(&self, v: &f64) -> Option<Rgb> {
        self.apply(*v).copied()
    }
}

#[cfg(test)]
mod tests {
    use rustd3_core::scale;

    use super::*;

    #[test]
    fn binned_scales_paint_ignore_gestures_and_keep_or_fit_their_domains() {
        let blues5 = ["#eff3ff", "#bdd7e7", "#6baed6", "#3182bd", "#08519c"]
            .map(|c| c.parse::<Rgb>().expect("a color"));
        let q = scale::quantize([0.0, 100.0], blues5);
        assert!(Scale::zoom_px(&q, [0.0, 1.0], 0.5, 2.0).is_none());
        assert_eq!(q.fitted(None, &[], 0.05), q);
        let fitted = q.fitted(Some(&[10.0, 60.0]), &[], 0.05);
        assert_eq!(fitted.get_domain(), [10.0, 60.0]);
        assert_eq!(fitted.paint(&f64::NAN), None);
        let red = Rgb::new(255.0, 0.0, 0.0);
        let t = scale::threshold([0.0, 1.0], [red, red, red]);
        assert_eq!(t.fitted(Some(&[-5.0, 5.0]), &[], 0.05), t);
        assert!(t.admits(&f64::INFINITY) && !t.admits(&f64::NAN));
        let quantile = scale::quantile([1.0, 2.0, 3.0, 10.0], [red, red]);
        assert_eq!(quantile.fitted(Some(&[0.0, 1.0]), &[], 0.05), quantile);
        assert_eq!((Kind::domain(&q), Kind::domain(&t)), (None, None));
    }
}
