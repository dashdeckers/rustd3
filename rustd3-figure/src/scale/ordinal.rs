//! The capability traits for core's ordinal scales over colors: palettes.

use std::any::Any;

use rustd3_core::color::Rgb;
use rustd3_core::scale::Ordinal;

use crate::scale::{Kind, Paint, Scale};

/// A palette has no range in px and ignores gestures.
impl<K: PartialEq + Clone + 'static> Scale for Ordinal<K, Rgb> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn boxed(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }

    /// The palette as it is.
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

impl<K: PartialEq + Clone + 'static> crate::sealed::Scale for Ordinal<K, Rgb> {}

/// Fitting keeps the keys the palette was built with.
impl<K: PartialEq + Clone + 'static> Kind for Ordinal<K, Rgb> {
    type Domain = K;
    type Output = Rgb;
    type Span = ();

    /// Whether the palette colors `key`: one of its keys, or any key when
    /// it has an unknown color.
    fn admits(&self, key: &K) -> bool {
        self.apply(key).is_some()
    }

    fn merge(_extent: &mut Option<[K; 2]>, _key: &K) {}

    fn fitted(&self, _extent: Option<&[K; 2]>, _sticky: &[K], _margin: f64) -> Self {
        self.clone()
    }

    fn ends(&self) -> Option<[K; 2]> {
        None
    }
}

impl<K: PartialEq + Clone + 'static> Paint for Ordinal<K, Rgb> {
    fn paint(&self, key: &K) -> Option<Rgb> {
        self.apply(key).copied()
    }

    /// The key's index among the domain's keys, and their count.
    fn slot(&self, key: &K) -> Option<(usize, usize)> {
        let keys = self.get_domain();
        let i = keys.iter().position(|k| k == key)?;
        Some((i, keys.len()))
    }
}

#[cfg(test)]
mod tests {
    use rustd3_core::scale;
    use rustd3_core::scheme::COLORBLIND;

    use super::*;

    #[test]
    fn a_palette_colors_and_slots_its_keys() {
        let color = scale::ordinal(["x", "z", "t"], COLORBLIND);
        assert_eq!(color.paint(&"z"), Some(COLORBLIND[1]));
        assert_eq!(color.slot(&"t"), Some((2, 3)));
        assert_eq!((color.paint(&"w"), color.slot(&"w")), (None, None));
        assert!(Scale::zoom_px(&color, [0.0, 1.0], 0.5, 2.0).is_none());
    }
}
