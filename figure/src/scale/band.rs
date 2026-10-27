//! The capability traits for core's band scales: positions, no gestures,
//! and the domain they were built with.

use std::any::Any;

use rustd3_core::scale::Band;

use crate::scale::{Kind, Position, Scale};

/// Band scales ignore gestures: their keys are the domain.
impl<K: PartialEq + Clone + 'static> Scale for Band<K> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn boxed(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }

    /// d3's `range(r)`, which keeps padding, alignment and rounding.
    fn with_range(&self, range: [f64; 2]) -> Box<dyn Scale> {
        Box::new(self.clone().range(range))
    }

    fn pan_px(&self, _range: [f64; 2], _dx: f64) -> Option<Box<dyn Scale>> {
        None
    }

    fn zoom_px(&self, _range: [f64; 2], _anchor: f64, _factor: f64) -> Option<Box<dyn Scale>> {
        None
    }
}

/// Fitting keeps the keys the scale was built with.
impl<K: PartialEq + Clone + 'static> Kind for Band<K> {
    type Domain = K;
    type Output = f64;
    type Span = ();

    /// Whether `key` is one of the domain's keys.
    fn admits(&self, key: &K) -> bool {
        self.apply(key).is_some()
    }

    fn merge(_extent: &mut Option<[K; 2]>, _key: &K) {}

    fn fitted(&self, _extent: Option<&[K; 2]>, _sticky: &[K], _margin: f64) -> Self {
        self.clone()
    }

    fn domain(&self) -> Option<[K; 2]> {
        None
    }
}

impl<K: PartialEq + Clone + 'static> Position for Band<K> {
    /// The start of the key's band.
    fn position(&self, key: &K) -> Option<f64> {
        self.apply(key)
    }

    fn bandwidth(&self) -> f64 {
        Band::bandwidth(self)
    }
}

#[cfg(test)]
mod tests {
    use rustd3_core::scale;

    use super::*;

    #[test]
    fn indexed_is_none_for_band_scales() {
        let speed = scale::band(0..20).padding(0.1);
        assert!(speed.indexed().is_none());
    }

    #[test]
    fn band_scales_keep_their_keys_and_ignore_gestures() {
        let b = scale::band(["a", "b"]).padding(0.1);
        assert_eq!(b.fitted(None, &[], 0.05), b);
        assert!(Scale::pan_px(&b, [0.0, 100.0], 5.0).is_none());
        assert!(Scale::zoom_px(&b, [0.0, 100.0], 50.0, 2.0).is_none());
        assert!(b.admits(&"a") && !b.admits(&"c"));
        let ranged = Scale::with_range(&b, [0.0, 210.0]);
        let ranged = ranged
            .as_any()
            .downcast_ref::<Band<&str>>()
            .expect("its type");
        assert_eq!(
            (ranged.position(&"b"), ranged.bandwidth()),
            (Some(110.0), 90.0)
        );
    }
}
