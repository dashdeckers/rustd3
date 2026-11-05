//! What the widget's scenarios need of the workloads, with the `widget`
//! feature: the size of a figure that keeps its own, where the gestures
//! act, and which scenarios a workload has.

use iced::Size;

use crate::gallery::wheels;
use crate::scenario::{Anchors, Scenario};
use crate::workload::Workload;

/// The wheel figure's logical size: the example's.
const WHEELS: Size = Size::new(wheels::SIZE.width as f32, wheels::SIZE.height as f32);

/// Where the gestures act: near the middle of the front-left scatter's
/// plot area (the wheel, the horizontal drag) and of the rear-left one's
/// (the vertical drag), which zoom vibration and temperature, as
/// fractions of the figure at 960 × 760 and in the 4k window, where the
/// areas' middles lie at 0.33 and 0.27 across, 0.22 and 0.21 down, 0.66
/// and 0.67 down (`gestures_land_on_the_scatters` checks both).
const ANCHORS: Anchors = Anchors {
    x: 0.3,
    top: 0.22,
    bottom: 0.66,
};

impl Workload {
    /// The logical size of its figure where a scenario uses the standard
    /// window: the wheel figure keeps its own; the others take the
    /// scenario's.
    pub fn size(self) -> Option<Size> {
        match self {
            Workload::Wheels => Some(WHEELS),
            Workload::Trace
            | Workload::Scatter100k
            | Workload::Scatter1m
            | Workload::Grid16
            | Workload::LabelChurn => None,
        }
    }

    /// Whether a panel of its figure zooms an x handle, which a
    /// horizontal drag pans: the wheel figure's x is a band scale.
    pub fn pans_x(self) -> bool {
        match self {
            Workload::Trace
            | Workload::Scatter100k
            | Workload::Scatter1m
            | Workload::Grid16
            | Workload::LabelChurn => true,
            Workload::Wheels => false,
        }
    }

    /// Whether it supports `scenario`; a cell it does not is "n/a": `live`
    /// and `soak` need recordings, `pan-x` an x that pans. (`4k`'s pan
    /// half on a figure whose x does not pan moves nothing; its zoom half
    /// does.)
    pub fn supports(self, scenario: Scenario) -> bool {
        (self.has_recordings() || !scenario.needs_recordings())
            && (self.pans_x() || scenario != Scenario::PanX)
    }

    /// Where its scenarios' gestures act: the standard figures' panels,
    /// or the wheel figure's scatters, which zoom.
    pub fn anchors(self) -> Anchors {
        match self {
            Workload::Wheels => ANCHORS,
            Workload::Trace
            | Workload::Scatter100k
            | Workload::Scatter1m
            | Workload::Grid16
            | Workload::LabelChurn => Anchors::STANDARD,
        }
    }
}

#[cfg(test)]
mod tests {
    use rustd3_figure::cache::Cache;
    use rustd3_figure::kurbo::{Point, Size};
    use rustd3_figure::{Held, Target};

    use super::{ANCHORS, WHEELS};
    use crate::scenario::Scenario;
    use crate::workload::Workload;
    use crate::workload::wheels::Wheels;

    #[test]
    fn scatters_have_no_live_cells() {
        for workload in [Workload::Scatter100k, Workload::Scatter1m] {
            assert!(!workload.supports(Scenario::Live));
            assert!(!workload.supports(Scenario::Soak));
            assert!(workload.supports(Scenario::PanX));
        }
        assert!(Workload::Grid16.supports(Scenario::Live));
    }

    #[test]
    fn the_wheel_figure_has_no_live_cells() {
        assert!(!Workload::Wheels.supports(Scenario::Live));
        assert!(!Workload::Wheels.supports(Scenario::Soak));
        assert!(Workload::Wheels.supports(Scenario::ZoomLines));
    }

    #[test]
    fn gestures_land_on_the_scatters() {
        let wheels = Wheels::simulated();
        let fig = wheels.figure();
        for size in [
            Size::new(f64::from(WHEELS.width), f64::from(WHEELS.height)),
            Size::new(1920.0, 1080.0),
        ] {
            let mut cache = Cache::default();
            let target = Target::Screen { scale: 2.0 };
            let resolved = fig.resolve(size, &mut cache, &Held::default(), target);
            let x = f64::from(ANCHORS.x) * size.width;
            for y in [ANCHORS.top, ANCHORS.bottom] {
                let at = Point::new(x, f64::from(y) * size.height);
                let panel = resolved
                    .get_plot_areas()
                    .iter()
                    .position(|area| area.contains(at))
                    .expect("a plot area");
                let (_, ys) = resolved.get_zooms(panel);
                println!(
                    "{size:?}: {at:?} in panel {panel} of {:?}",
                    resolved.get_plot_areas()[panel]
                );
                assert!(
                    !ys.is_empty(),
                    "{size:?}: {at:?} on panel {panel}, which zooms no y"
                );
            }
        }
    }
}
