//! How layout learns text widths. The figure layer's
//! default estimates; a typst-backed measurer replaces
//! it so the engine that sizes a label is the one that
//! draws it.

use super::text::Text;
use std::rc::Rc;

pub trait Measure {
    /// Advance width in px of `t` at `size` px.
    fn width(&self, t: &Text, size: f64, bold: bool) -> f64;
}

/// 0.6 em per character: the sketch's old rule.
pub struct Estimate;

impl Measure for Estimate {
    fn width(&self, t: &Text, size: f64, _bold: bool) -> f64 {
        0.6 * size * t.len() as f64
    }
}

#[derive(Clone)]
pub struct Measurer(pub Rc<dyn Measure>);

impl Default for Measurer {
    fn default() -> Self {
        Measurer(Rc::new(Estimate))
    }
}
