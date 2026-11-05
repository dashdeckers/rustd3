//! The `wheels` workload: the wheel figure from `gallery::wheels`, with
//! 4 × 4 panels and 52 marks over its 5,600 samples, at its own 960 × 760.
//! It holds no recording, and its data does not scale with the plan. Its x
//! is a band scale, so nothing pans along x and the gestures act on the
//! scatters. The scroll wheel zooms vibration and temperature, the
//! horizontal drag acts on the front-left scatter, and the vertical drag on
//! the rear-left one.

use std::fmt;

use rustd3_figure::Figure;

use crate::gallery::wheels::{self, Sample};

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
