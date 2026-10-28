//! [`Workload`]: the figures the benchmark drives, with their data
//! generated deterministically by xorshift64* from fixed seeds. Their
//! layout and data are fixed here for repeatable timing; they are not the
//! example figures.

mod grid;
mod label_churn;
mod scatter;
mod trace;
mod wheels;

use iced::Size;
use rustd3_core::jiff::Timestamp;
use rustd3_figure::Figure;

use crate::scenario::{Anchors, Scenario};

/// One of the benchmark's figures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Workload {
    /// Two stacked panels sharing x, three channels of 1M samples.
    Trace,
    /// 100k dots in 5 Gaussian clusters.
    Scatter100k,
    /// 1M dots in 5 Gaussian clusters.
    Scatter1m,
    /// 4 × 4 panels, each a 100k-sample line and 2k dots.
    Grid16,
    /// `trace` on a UTC time axis around midnight: labels that change as
    /// pans cross the day and on every zoom.
    LabelChurn,
    /// Part I's wheel figure at 960 × 760.
    Wheels,
}

/// A workload's data, owned by the app between frames.
#[derive(Debug)]
pub enum Data {
    /// `trace`.
    Trace(trace::Trace<f64>),
    /// `label-churn`.
    Churn(trace::Trace<Timestamp>),
    /// The scatters.
    Scatter(scatter::Scatter),
    /// `grid16`.
    Grid(grid::Grid),
    /// `wheels`.
    Wheels(wheels::Wheels),
}

/// xorshift64* (Vigna 2016): the workloads' only source of noise.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Workload {
    /// Every workload, in report order.
    pub const ALL: [Workload; 6] = [
        Workload::Trace,
        Workload::Scatter100k,
        Workload::Scatter1m,
        Workload::Grid16,
        Workload::LabelChurn,
        Workload::Wheels,
    ];

    /// The workload's name on the command line and in file names.
    pub fn name(self) -> &'static str {
        match self {
            Workload::Trace => "trace",
            Workload::Scatter100k => "scatter100k",
            Workload::Scatter1m => "scatter1m",
            Workload::Grid16 => "grid16",
            Workload::LabelChurn => "label-churn",
            Workload::Wheels => "wheels",
        }
    }

    /// The workload named `name`.
    pub fn parse(name: &str) -> Option<Workload> {
        Workload::ALL.into_iter().find(|w| w.name() == name)
    }

    /// Whether its figure draws recordings, which `live` and `soak` append to.
    pub fn has_recordings(self) -> bool {
        match self {
            Workload::Trace | Workload::Grid16 | Workload::LabelChurn => true,
            Workload::Scatter100k | Workload::Scatter1m | Workload::Wheels => false,
        }
    }

    /// The logical size of its figure where a scenario uses the standard
    /// window: the wheel figure keeps its own; the others take the
    /// scenario's.
    pub fn size(self) -> Option<Size> {
        match self {
            Workload::Wheels => Some(wheels::SIZE),
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
            Workload::Wheels => wheels::ANCHORS,
            Workload::Trace
            | Workload::Scatter100k
            | Workload::Scatter1m
            | Workload::Grid16
            | Workload::LabelChurn => Anchors::STANDARD,
        }
    }

    /// What its data holds at `scale` (1 = full size), for listings.
    pub fn describe(self, scale: f64) -> String {
        let n = |full: usize| sized(full, scale);
        match self {
            Workload::Trace => format!(
                "2 panels, 3 channels × {} samples at 10 kHz",
                n(trace::SAMPLES)
            ),
            Workload::LabelChurn => format!(
                "2 panels, 3 channels × {} samples at 10 kHz around midnight UTC",
                n(trace::SAMPLES)
            ),
            Workload::Scatter100k => format!("1 panel, {} dots", n(100_000)),
            Workload::Scatter1m => format!("1 panel, {} dots", n(1_000_000)),
            Workload::Grid16 => format!(
                "4 × 4 panels, each {} samples and {} dots",
                n(grid::SAMPLES),
                n(grid::DOTS)
            ),
            Workload::Wheels => {
                "4 × 4 panels, 52 marks over 5,600 samples at 960 × 760 (fixed size)".to_owned()
            }
        }
    }

    /// Its data at `scale` (1 = full size).
    pub fn data(self, scale: f64) -> Data {
        match self {
            Workload::Trace => Data::Trace(trace::Trace::new(sized(trace::SAMPLES, scale))),
            Workload::LabelChurn => {
                Data::Churn(trace::Trace::around_midnight(sized(trace::SAMPLES, scale)))
            }
            Workload::Scatter100k => Data::Scatter(scatter::Scatter::new(sized(100_000, scale))),
            Workload::Scatter1m => Data::Scatter(scatter::Scatter::new(sized(1_000_000, scale))),
            Workload::Grid16 => Data::Grid(grid::Grid::new(
                sized(grid::SAMPLES, scale),
                sized(grid::DOTS, scale),
            )),
            Workload::Wheels => Data::Wheels(wheels::Wheels::simulated()),
        }
    }
}

impl Data {
    /// The figure over this data; with `windowed`, x follows the newest
    /// 10 s (`Fit::Window`) of the workloads with recordings.
    pub fn figure(&self, windowed: bool) -> Figure<'_> {
        match self {
            Data::Trace(trace) => trace.figure(windowed),
            Data::Churn(trace) => trace.figure(windowed),
            Data::Scatter(scatter) => scatter.figure(),
            Data::Grid(grid) => grid.figure(windowed),
            Data::Wheels(wheels) => wheels.figure(),
        }
    }

    /// Appends `n` samples to every channel of every recording; the
    /// scatters and the wheel figure have none.
    pub fn append(&mut self, n: usize) {
        match self {
            Data::Trace(trace) => trace.append(n),
            Data::Churn(trace) => trace.append(n),
            Data::Grid(grid) => grid.append(n),
            Data::Scatter(_) | Data::Wheels(_) => {}
        }
    }
}

impl Rng {
    /// A generator seeded with `seed` (0 becomes 1: xorshift's zero state
    /// is fixed).
    pub fn new(seed: u64) -> Rng {
        Rng(seed.max(1))
    }

    /// The next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in [0, 1), 53 bits.
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1_u64 << 53) as f64
    }

    /// A standard normal deviate (Box–Muller).
    pub fn gauss(&mut self) -> f64 {
        let (a, b) = (self.unit().max(1e-300), self.unit());
        (-2.0 * a.ln()).sqrt() * (std::f64::consts::TAU * b).cos()
    }
}

/// `full` scaled by `scale`, at least 1.
fn sized(full: usize, scale: f64) -> usize {
    ((full as f64 * scale).round() as usize).max(1)
}

#[cfg(test)]
mod tests {
    use rustd3_core::jiff::{SignedDuration, Timestamp};
    use rustd3_figure::kurbo::Size;
    use rustd3_figure::list::Command;

    use super::{Data, Rng, Workload};
    use crate::scenario::Scenario;

    #[test]
    fn trace_has_three_channels_of_one_million() {
        let Data::Trace(trace) = Workload::Trace.data(1.0) else {
            panic!("trace data");
        };
        let rec = trace.recording();
        assert_eq!((rec.channels(), rec.len()), (3, 1_000_000));
        // 100 s at 10 kHz.
        assert_eq!(rec.x(0), Some(0.0));
        assert!((rec.x(999_999).unwrap() - 99.9999).abs() < 1e-9);
        for channel in 0..3 {
            assert_eq!(trace.impacts(channel), 50);
        }
        // A burst opens every 10 s: 10 of them.
        assert_eq!(trace.bursts(), 10);
    }

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
    fn the_wheel_figure_has_no_recordings_and_does_not_scale() {
        assert!(!Workload::Wheels.supports(Scenario::Live));
        assert!(!Workload::Wheels.supports(Scenario::Soak));
        assert!(Workload::Wheels.supports(Scenario::ZoomLines));
        let Data::Wheels(wheels) = Workload::Wheels.data(0.01) else {
            panic!("wheel data");
        };
        assert_eq!(format!("{wheels:?}"), "Wheels { samples: 5600 }");
    }

    #[test]
    fn label_churn_spans_midnight_on_a_time_axis_labelled_with_the_date() {
        let Data::Churn(churn) = Workload::LabelChurn.data(0.01) else {
            panic!("label-churn data");
        };
        let rec = churn.recording();
        let midnight: Timestamp = "2026-10-28T00:00:00Z".parse().expect("an instant");
        assert_eq!(rec.len(), 10_000);
        assert_eq!(rec.x(5_000), Some(midnight));
        let first = rec.x(0).expect("a sample");
        assert_eq!(
            midnight.duration_since(first),
            SignedDuration::from_millis(500)
        );
        let list = Workload::LabelChurn
            .data(0.01)
            .figure(false)
            .draw(Size::new(1600.0, 1000.0));
        let labels: Vec<&str> = list
            .commands
            .iter()
            .filter_map(|c| {
                if let Command::Glyphs(run) = c {
                    Some(&*run.text)
                } else {
                    None
                }
            })
            .collect();
        // The ISO ladder: the day at midnight, seconds and milliseconds
        // around it.
        assert!(labels.contains(&"10-28"), "{labels:?}");
        assert!(labels.iter().any(|l| l.starts_with("59.")), "{labels:?}");
    }

    #[test]
    fn xorshift_is_deterministic_and_uniform() {
        let (mut a, mut b) = (Rng::new(7), Rng::new(7));
        let draws: Vec<f64> = (0..10_000).map(|_| a.unit()).collect();
        assert!(draws.iter().all(|&u| (0.0..1.0).contains(&u)));
        assert_eq!(draws[..3], [b.unit(), b.unit(), b.unit()]);
        let mean = draws.iter().sum::<f64>() / draws.len() as f64;
        assert!((mean - 0.5).abs() < 0.01, "{mean}");
    }
}
