//! The capability traits for core's point scales: positions, no gestures,
//! and the domain they were built with.

use std::any::Any;

use rustd3_core::scale::Point;

use crate::scale::{Kind, Position, Scale};

/// Point scales ignore gestures: their keys are the domain.
impl<K: PartialEq + Clone + 'static> Scale for Point<K> {
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
impl<K: PartialEq + Clone + 'static> Kind for Point<K> {
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

/// Points have no width: the default bandwidth of 0.
impl<K: PartialEq + Clone + 'static> Position for Point<K> {
    fn position(&self, key: &K) -> Option<f64> {
        self.apply(key)
    }
}

#[cfg(test)]
mod tests {
    use rustd3_core::scale;

    use super::*;

    #[test]
    fn point_scales_place_keys_and_ignore_gestures() {
        let p = scale::point(["a", "b", "c"]);
        let ranged = Scale::with_range(&p, [0.0, 100.0]);
        let ranged = ranged
            .as_any()
            .downcast_ref::<Point<&str>>()
            .expect("its type");
        assert_eq!(ranged.position(&"b"), Some(50.0));
        assert_eq!(ranged.bandwidth(), 0.0);
        assert!(ranged.indexed().is_none());
        assert!(Scale::pan_px(&p, [0.0, 100.0], 5.0).is_none());
        assert_eq!(p.fitted(None, &[], 0.05), p);
    }
}
