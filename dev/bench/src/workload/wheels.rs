//! The `wheels` workload: Part I's wheel figure from
//! `rustd3_examples::wheels`, 4 × 4 panels and 52 marks over its 5,600
//! samples, at its own 960 × 760; the wheel zooms vibration and
//! temperature on the scatters. It holds no recording, and its data does
//! not scale with the plan. Its x is a band scale, so nothing pans along
//! x: the gestures act on the scatters, the wheel and the horizontal drag
//! on the front-left one, the vertical drag on the rear-left one.

use std::fmt;

use rustd3_examples::wheels::{self, Sample};
use rustd3_figure::Figure;

/// The wheel figure's samples.
pub struct Wheels {
    samples: Vec<Sample>,
}

impl fmt::Debug for Wheels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wheels")
            .field("samples", &self.samples.len())
            .finish()
    }
}

impl Wheels {
    /// The example's samples, from its `simulate`.
    pub fn simulated() -> Wheels {
        Wheels {
            samples: wheels::simulate(),
        }
    }

    /// The figure, its legend below the grid.
    pub fn figure(&self) -> Figure<'_> {
        wheels::figure(&self.samples, false)
    }
}
