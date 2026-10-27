//! [`Workload`]: the figures the benchmark drives, with their data
//! generated deterministically by xorshift64* from fixed seeds. Their
//! layout and data are fixed here for repeatable timing; they are not the
//! example figures.

mod grid;
mod label_churn;
mod scatter;
mod trace;

use rustd3_figure::Figure;

use crate::scenario::Scenario;

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
    /// `trace` with x labels that change on every pan step.
    LabelChurn,
}

/// A workload's data, owned by the app between frames.
#[derive(Debug)]
pub enum Data {
    /// `trace` and `label-churn`.
    Trace(trace::Trace),
    /// The scatters.
    Scatter(scatter::Scatter),
    /// `grid16`.
    Grid(grid::Grid),
}

/// xorshift64* (Vigna 2016): the workloads' only source of noise.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Workload {
    /// Every workload, in report order.
    pub const ALL: [Workload; 5] = [
        Workload::Trace,
        Workload::Scatter100k,
        Workload::Scatter1m,
        Workload::Grid16,
        Workload::LabelChurn,
    ];

    /// The workload's name on the command line and in file names.
    pub fn name(self) -> &'static str {
        match self {
            Workload::Trace => "trace",
            Workload::Scatter100k => "scatter100k",
            Workload::Scatter1m => "scatter1m",
            Workload::Grid16 => "grid16",
            Workload::LabelChurn => "label-churn",
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
            Workload::Scatter100k | Workload::Scatter1m => false,
        }
    }

    /// Whether it supports `scenario`; a cell it does not is "n/a".
    pub fn supports(self, scenario: Scenario) -> bool {
        self.has_recordings() || !scenario.needs_recordings()
    }

    /// What its data holds at `scale` (1 = full size), for listings.
    pub fn describe(self, scale: f64) -> String {
        let n = |full: usize| sized(full, scale);
        match self {
            Workload::Trace | Workload::LabelChurn => format!(
                "2 panels, 3 channels × {} samples at 10 kHz",
                n(trace::SAMPLES)
            ),
            Workload::Scatter100k => format!("1 panel, {} dots", n(100_000)),
            Workload::Scatter1m => format!("1 panel, {} dots", n(1_000_000)),
            Workload::Grid16 => format!(
                "4 × 4 panels, each {} samples and {} dots",
                n(grid::SAMPLES),
                n(grid::DOTS)
            ),
        }
    }

    /// Its data at `scale` (1 = full size).
    pub fn data(self, scale: f64) -> Data {
        match self {
            Workload::Trace | Workload::LabelChurn => {
                Data::Trace(trace::Trace::new(sized(trace::SAMPLES, scale)))
            }
            Workload::Scatter100k => Data::Scatter(scatter::Scatter::new(sized(100_000, scale))),
            Workload::Scatter1m => Data::Scatter(scatter::Scatter::new(sized(1_000_000, scale))),
            Workload::Grid16 => Data::Grid(grid::Grid::new(
                sized(grid::SAMPLES, scale),
                sized(grid::DOTS, scale),
            )),
        }
    }
}

impl Data {
    /// The figure of `workload` over this data; with `windowed`, x follows
    /// the newest 10 s (`Fit::Window(10.0)`).
    pub fn figure(&self, workload: Workload, windowed: bool) -> Figure<'_> {
        match self {
            Data::Trace(trace) => {
                let churn = workload == Workload::LabelChurn;
                trace.figure(windowed, churn.then_some(label_churn::LABELS))
            }
            Data::Scatter(scatter) => scatter.figure(),
            Data::Grid(grid) => grid.figure(windowed),
        }
    }

    /// Appends `n` samples to every channel of every recording; a scatter
    /// has none.
    pub fn append(&mut self, n: usize) {
        match self {
            Data::Trace(trace) => trace.append(n),
            Data::Grid(grid) => grid.append(n),
            Data::Scatter(_) => {}
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
    fn xorshift_is_deterministic_and_uniform() {
        let (mut a, mut b) = (Rng::new(7), Rng::new(7));
        let draws: Vec<f64> = (0..10_000).map(|_| a.unit()).collect();
        assert!(draws.iter().all(|&u| (0.0..1.0).contains(&u)));
        assert_eq!(draws[..3], [b.unit(), b.unit(), b.unit()]);
        let mean = draws.iter().sum::<f64>() / draws.len() as f64;
        assert!((mean - 0.5).abs() < 0.01, "{mean}");
    }
}
