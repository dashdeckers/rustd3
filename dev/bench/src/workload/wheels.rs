//! The `wheels` workload: Part I's wheel figure from
//! `rustd3_examples::wheels`, 4 × 4 panels and 52 marks over its 5,600
//! samples, at its own 960 × 760; the wheel zooms vibration and
//! temperature on the scatters. It holds no recording, and its data does
//! not scale with the plan. Its x is a band scale, so nothing pans along
//! x: the gestures act on the scatters, the wheel and the horizontal drag
//! on the front-left one, the vertical drag on the rear-left one.

use std::fmt;

use iced::Size;
use rustd3_examples::wheels::{self, Sample};
use rustd3_figure::Figure;

use crate::scenario::Anchors;

/// The wheel figure's samples.
pub struct Wheels {
    samples: Vec<Sample>,
}

/// The figure's logical size: the example's.
pub const SIZE: Size = Size::new(wheels::SIZE.width as f32, wheels::SIZE.height as f32);

/// Where the gestures act: near the middle of the front-left scatter's
/// plot area (the wheel, the horizontal drag) and of the rear-left one's
/// (the vertical drag), which zoom vibration and temperature, as
/// fractions of the figure at 960 × 760 and in the 4k window, where the
/// areas' middles lie at 0.33 and 0.27 across, 0.22 and 0.21 down, 0.66
/// and 0.67 down (`gestures_land_on_the_scatters` checks both).
pub const ANCHORS: Anchors = Anchors {
    x: 0.3,
    top: 0.22,
    bottom: 0.66,
};

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

#[cfg(test)]
mod tests {
    use rustd3_figure::kurbo::{Point, Size};
    use rustd3_figure::{Cache, Held, Target};

    use super::{ANCHORS, SIZE, Wheels};

    #[test]
    fn gestures_land_on_the_scatters() {
        let wheels = Wheels::simulated();
        let fig = wheels.figure();
        for size in [
            Size::new(f64::from(SIZE.width), f64::from(SIZE.height)),
            Size::new(1920.0, 1080.0),
        ] {
            let mut cache = Cache::default();
            let target = Target::Screen { scale: 2.0 };
            let resolved = fig.resolve(size, &mut cache, &Held::default(), target);
            let x = f64::from(ANCHORS.x) * size.width;
            for y in [ANCHORS.top, ANCHORS.bottom] {
                let at = Point::new(x, f64::from(y) * size.height);
                let panel = resolved
                    .plot_areas()
                    .iter()
                    .position(|area| area.contains(at))
                    .expect("a plot area");
                let (_, ys) = resolved.zooms(panel);
                println!(
                    "{size:?}: {at:?} in panel {panel} of {:?}",
                    resolved.plot_areas()[panel]
                );
                assert!(
                    !ys.is_empty(),
                    "{size:?}: {at:?} on panel {panel}, which zooms no y"
                );
            }
        }
    }
}
