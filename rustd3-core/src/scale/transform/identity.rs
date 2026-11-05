//! The identity transform, under d3's linear scale.

use crate::scale::Transform;
use crate::ticks;

/// The identity transform under [`scale::Linear`](crate::scale::Linear):
/// forward and inverse return their argument, and ticks are d3's linear
/// ticks. Not d3's scaleIdentity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Identity;

impl Transform for Identity {
    fn forward(&self, x: f64) -> f64 {
        x
    }

    fn inverse(&self, y: f64) -> f64 {
        y
    }

    fn ticks(&self, domain: [f64; 2], count: usize) -> Vec<f64> {
        ticks::linear(domain, count)
    }
}
