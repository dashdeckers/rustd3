//! The power transform, under d3's scalePow and scaleSqrt (pow.js).

use crate::scale::Transform;
use crate::ticks;

/// d3's power transform: `x^e` keeping the sign of `x`, with exponent 1 the
/// identity and 0.5 an exact square root. Ticks are d3's linear ticks in
/// data space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pow {
    exponent: f64,
}

impl Pow {
    /// The transform of exponent `exponent`.
    pub(crate) fn new(exponent: f64) -> Pow {
        Pow { exponent }
    }
}

impl Transform for Pow {
    fn forward(&self, x: f64) -> f64 {
        if self.exponent == 1.0 {
            x
        } else if self.exponent == 0.5 {
            signed(x, f64::sqrt)
        } else {
            signed(x, |x| x.powf(self.exponent))
        }
    }

    fn inverse(&self, y: f64) -> f64 {
        if self.exponent == 1.0 {
            y
        } else if self.exponent == 0.5 {
            signed(y, |y| y * y)
        } else {
            signed(y, |y| y.powf(1.0 / self.exponent))
        }
    }

    fn ticks(&self, domain: [f64; 2], count: usize) -> Vec<f64> {
        ticks::linear(domain, count)
    }
}

/// `f` of `x`, reflected through the origin for negative `x`.
fn signed(x: f64, f: impl Fn(f64) -> f64) -> f64 {
    if x < 0.0 { -f(-x) } else { f(x) }
}
