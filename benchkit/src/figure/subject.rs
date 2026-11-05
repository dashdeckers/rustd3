//! [`Subject`]: the figures the figure profile times, and their data,
//! generated deterministically before the timings: the widget's workloads
//! where they serve (`trace`, the scatters, `grid16`), figures a library
//! user makes that the widget's profiles do not (labels with typst math, a
//! statistical figure, a heatmap), lines and scatters along a ladder of
//! sizes, and the gallery's presets and wheel figure as real-world
//! figures at their own sizes.

use std::f64::consts::TAU;

use rustd3_core::scale;
use rustd3_core::scheme::{self, COLORBLIND};
use rustd3_core::series::{Clock, Recording};
use rustd3_figure::colorbar::colorbar;
use rustd3_figure::kurbo::Size;
use rustd3_figure::legend::{Slot, legend};
use rustd3_figure::mark::{Grid, density_x, histogram, image, line, note};
use rustd3_figure::text::{Text, typst};
use rustd3_figure::{Figure, axis};

use crate::figure::Stage;
use crate::gallery::{presets, wheels};
use crate::workload::{self, Rng, Workload};

/// One figure of the profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subject {
    /// The widget's `trace`: 2 panels, 3 channels × 1M samples on a time
    /// axis in seconds, a legend.
    Trace,
    /// `n` markers in 5 clusters: `scatter100k` is the widget's.
    Scatter(usize),
    /// The widget's `grid16`: 4 × 4 panels, each a 100k-sample line and
    /// 2k markers.
    Grid16,
    /// 3 × 3 panels titled with typst math, math axis labels with minor
    /// ticks, a math note per panel, a legend of nine keys.
    Labels,
    /// A histogram and a density of 1M values in one panel.
    Stats,
    /// A 1000 × 1000 heatmap with a colorbar.
    Heatmap,
    /// One line of `n` samples of `trace`'s signal.
    Line(usize),
    /// A chart preset of the gallery (one of `presets::NAMES`).
    Preset(&'static str),
    /// The wheel figure from the gallery.
    Wheels,
}

/// A subject's data, owned while its figure is timed.
pub enum Data {
    /// `trace`, the scatters, `grid16`.
    Workload(workload::Data),
    /// The label figure draws its own small data.
    Labels,
    /// The statistical figure's values.
    Stats(Vec<f64>),
    /// The heatmap's cells, row-major.
    Heatmap(Vec<f64>),
    /// A ladder's line.
    Line(Recording<f64>),
    /// A preset's data and name.
    Preset(presets::Data, &'static str),
    /// The wheel figure's samples.
    Wheels(Vec<wheels::Sample>),
}

/// The values of the statistical figure.
const VALUES: usize = 1_000_000;

/// The heatmap's side, in cells.
const SIDE: usize = 1_000;

impl Subject {
    /// Its name in the reports: `trace`, `scatter100k`, `line10m`,
    /// `preset-pie`, …
    pub fn name(self) -> String {
        match self {
            Subject::Trace => "trace".to_owned(),
            Subject::Scatter(n) => format!("scatter{}", count(n)),
            Subject::Grid16 => "grid16".to_owned(),
            Subject::Labels => "labels".to_owned(),
            Subject::Stats => "stats".to_owned(),
            Subject::Heatmap => "heatmap".to_owned(),
            Subject::Line(n) => format!("line{}", count(n)),
            Subject::Preset(name) => format!("preset-{name}"),
            Subject::Wheels => "wheels".to_owned(),
        }
    }

    /// The ladder it is a rung of (`lines`, `scatters`; `scatter100k` is
    /// one), and its size.
    pub fn rung(self) -> Option<(&'static str, usize)> {
        match self {
            Subject::Line(n) => Some(("lines", n)),
            Subject::Scatter(n) => Some(("scatters", n)),
            Subject::Trace
            | Subject::Grid16
            | Subject::Labels
            | Subject::Stats
            | Subject::Heatmap
            | Subject::Preset(_)
            | Subject::Wheels => None,
        }
    }

    /// What it draws, in a phrase.
    pub fn what(self) -> String {
        match self {
            Subject::Trace => {
                "2 panels, 3 channels × 1M samples on a time axis, a legend".to_owned()
            }
            Subject::Scatter(n) => format!("1 panel, {} markers in 5 clusters", count(n)),
            Subject::Grid16 => "4 × 4 panels, each a 100k-sample line and 2k markers".to_owned(),
            Subject::Labels => "3 × 3 panels, titles, axis labels and notes in typst math, \
                                minor ticks, a legend"
                .to_owned(),
            Subject::Stats => "a histogram and a density of 1M values".to_owned(),
            Subject::Heatmap => "a 1000 × 1000 heatmap with a colorbar".to_owned(),
            Subject::Line(n) => format!("1 panel, one line of {} samples", count(n)),
            Subject::Preset(name) => format!("the `{name}` chart preset"),
            Subject::Wheels => "the wheel figure: 4 × 4 panels, 52 marks".to_owned(),
        }
    }

    /// Its size, logical px: the widget's standard window, at which the
    /// CPU reference draws, or a real-world figure's own.
    pub fn size(self) -> Size {
        match self {
            Subject::Preset(_) => presets::SIZE,
            Subject::Wheels => wheels::SIZE,
            Subject::Trace
            | Subject::Scatter(_)
            | Subject::Grid16
            | Subject::Labels
            | Subject::Stats
            | Subject::Heatmap
            | Subject::Line(_) => workload::WINDOW,
        }
    }

    /// Starting its process and generating its data, s, on two cores at
    /// 2.1 GHz, where the estimates were measured.
    pub fn data_s(self) -> f64 {
        let data = match self {
            Subject::Trace | Subject::Grid16 => 0.3,
            Subject::Scatter(n) | Subject::Line(n) => 1e-7 * n as f64,
            Subject::Stats | Subject::Heatmap => 0.1,
            Subject::Labels | Subject::Preset(_) | Subject::Wheels => 0.0,
        };
        0.15 + data
    }

    /// One repetition of build, resolve, list and `stages`, s, there.
    pub fn repetition_s(self, stages: &[Stage]) -> f64 {
        let (base, gif) = match self {
            Subject::Trace | Subject::Scatter(100_000) | Subject::Grid16 => (1.0, 0.9),
            Subject::Scatter(n) => ((1e-5 * n as f64).max(0.1), 0.0),
            Subject::Line(n) if n <= 10_000 => (0.15, 0.0),
            Subject::Line(n) if n <= 1_000_000 => (0.35, 0.0),
            Subject::Line(_) => (0.6, 0.0),
            Subject::Labels => (0.15, 0.5),
            Subject::Stats => (4.1, 0.5),
            Subject::Heatmap => (0.85, 0.9),
            Subject::Preset(_) => (0.03, 0.1),
            Subject::Wheels => (0.2, 0.3),
        };
        base + if stages.contains(&Stage::Gif) {
            gif
        } else {
            0.0
        }
    }

    /// The memory its process needs at its peak, MiB, as measured here:
    /// about 1.4 KiB a marker for the scatters (vello_cpu's PNG at 2× and
    /// the PDF hold the most), 250 MiB at most for the others.
    pub fn memory_mb(self) -> f64 {
        match self {
            Subject::Scatter(n) => 100.0 + 1.4e-3 * n as f64,
            Subject::Line(n) => 70.0 + 1.6e-5 * n as f64,
            Subject::Trace
            | Subject::Grid16
            | Subject::Labels
            | Subject::Stats
            | Subject::Heatmap
            | Subject::Preset(_)
            | Subject::Wheels => 250.0,
        }
    }

    /// Its data.
    pub fn data(self) -> Data {
        match self {
            Subject::Trace => Data::Workload(Workload::Trace.data(1.0)),
            Subject::Scatter(n) => Data::Workload(Workload::Scatter100k.data(n as f64 / 1e5)),
            Subject::Grid16 => Data::Workload(Workload::Grid16.data(1.0)),
            Subject::Labels => Data::Labels,
            Subject::Stats => {
                // Two Gaussians, 70:30, seed 21.
                let mut rng = Rng::new(21);
                let values = (0..VALUES)
                    .map(|_| {
                        if rng.unit() < 0.7 {
                            rng.gauss()
                        } else {
                            3.0 + 0.5 * rng.gauss()
                        }
                    })
                    .collect();
                Data::Stats(values)
            }
            Subject::Heatmap => {
                let mut rng = Rng::new(31);
                let cells = (0..SIDE * SIDE)
                    .map(|k| {
                        let (r, c) = ((k / SIDE) as f64, (k % SIDE) as f64);
                        (r / 37.0).sin() * (c / 53.0).cos() + 0.2 * rng.unit()
                    })
                    .collect();
                Data::Heatmap(cells)
            }
            Subject::Line(n) => {
                let samples = workload::trace::Signal::new(1, n).take(n);
                let clock = Clock::Regular {
                    start: 0.0,
                    period: 1.0 / workload::trace::RATE,
                };
                Data::Line(Recording::new(clock, [samples]).expect("invariant: finite samples"))
            }
            Subject::Preset(name) => Data::Preset(presets::Data::new(), name),
            Subject::Wheels => Data::Wheels(wheels::simulate()),
        }
    }
}

impl Data {
    /// The figure over this data, built as a user builds it.
    pub fn figure(&self) -> Figure<'_> {
        match self {
            Data::Workload(data) => data.figure(false),
            Data::Labels => labels(),
            Data::Stats(values) => stats(values),
            Data::Heatmap(cells) => heatmap(cells),
            Data::Line(rec) => long_line(rec),
            Data::Preset(data, name) => {
                presets::figure(data, name).expect("invariant: a preset's own name")
            }
            Data::Wheels(samples) => wheels::figure(samples, false),
        }
    }
}

/// `n` as the workloads name sizes: `1k`, `100k`, `1m`, `10m`.
pub fn count(n: usize) -> String {
    match n {
        n if n >= 1_000_000 && n % 1_000_000 == 0 => format!("{}m", n / 1_000_000),
        n if n >= 1_000 && n % 1_000 == 0 => format!("{}k", n / 1_000),
        n => n.to_string(),
    }
}

/// The label-heavy figure: partial sums of a square wave's Fourier series,
/// one per panel of a 3 × 3 grid, each titled with its sum and labelled
/// in typst math, with minor ticks and a math note at its peak.
fn labels<'a>() -> Figure<'a> {
    let mut fig = Figure::new();
    let x = fig.scale(scale::linear().domain([0.0, TAU]));
    let keys: Vec<String> = (1..=9).map(|k| format!("{k} terms")).collect();
    let color = fig.scale(scale::ordinal(keys.clone(), COLORBLIND));
    fig.title(Text::from("Partial sums of a square wave: ").push(typst(
        "$f_k (x) = 4 / pi sum_(i=1)^k sin((2i - 1) x) / (2i - 1)$",
    )));
    fig.legend(Slot::Right, legend(color).title("terms"));
    for (k, key) in (1..=9).zip(keys) {
        let y = fig.fit(scale::linear());
        let points: Vec<(f64, f64)> = (0..=400)
            .map(|i| {
                let t = TAU * f64::from(i) / 400.0;
                let sum: f64 = (1..=k)
                    .map(|j| {
                        let odd = f64::from(2 * j - 1);
                        (odd * t).sin() / odd
                    })
                    .sum();
                (t, 4.0 / std::f64::consts::PI * sum)
            })
            .collect();
        let peak = points
            .iter()
            .copied()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .expect("invariant: 401 points");
        let title = Text::from(format!("k = {k}: "))
            .push(typst(format!("$sum_(i=1)^{k} sin((2i - 1) x) / (2i - 1)$")));
        fig.panel(((k - 1) / 3) as usize, ((k - 1) % 3) as usize)
            .title(title)
            .mark(line(points, x, y).stroke(color, key))
            .mark(note(
                peak,
                typst(format!("$max f_{k} approx {:.4}$", peak.1)),
                x,
                y,
            ))
            .axis(axis::bottom(x).label(typst("$x$ [rad]")).minor(true))
            .axis(
                axis::left(y)
                    .label(typst(format!("$f_{k} (x)$")))
                    .minor(true),
            );
    }
    fig
}

/// A histogram of `values` (counts on the left) and their density (peak
/// 1, on the right) in one panel.
fn stats(values: &[f64]) -> Figure<'_> {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let count = fig.fit(scale::linear());
    let h = fig.fit(scale::linear());
    fig.title("1,000,000 values: a histogram and a density");
    fig.panel(0, 0)
        .mark(histogram(values.iter().copied(), x, count))
        .mark(density_x(values.iter().copied(), x, h))
        .axis(axis::bottom(x).label("value"))
        .axis(axis::left(count).label("count"))
        .axis(axis::right(h).label("density (peak = 1)"));
    fig
}

/// The `SIDE` × `SIDE` heatmap of `cells` in viridis, with a colorbar.
fn heatmap<'a>(cells: &[f64]) -> Figure<'a> {
    let grid = Grid::new(SIDE, SIDE, cells.to_vec()).expect("invariant: SIDE × SIDE cells");
    let mut fig = Figure::new();
    let heat = fig.fit(scale::sequential([0.0, 1.0], scheme::viridis));
    let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.title("A 1000 × 1000 heatmap");
    fig.panel(0, 0)
        .mark(image(grid, x, y, heat))
        .axis(axis::bottom(x).label("column"))
        .axis(axis::left(y).label("row"));
    fig.legend(Slot::Right, colorbar(heat));
    fig
}

/// One line over `rec`'s channel in one panel.
fn long_line(rec: &Recording<f64>) -> Figure<'_> {
    let mut fig = Figure::new();
    let (t, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.panel(0, 0)
        .mark(line(rec.series(0).expect("invariant: one channel"), t, y))
        .axis(axis::bottom(t).label("time [s]"))
        .axis(axis::left(y).label("amplitude"));
    fig
}

#[cfg(test)]
mod tests {
    use rustd3_figure::stats::Kde;

    use super::{Data, Subject, count};

    #[test]
    fn the_stats_figures_binned_density_is_within_a_thousandth_of_the_exact_one() {
        // The stats figure's 1M values take the binned path;
        // the exact sum, 200 points × 1M kernels, is split over the cores.
        let Data::Stats(values) = Subject::Stats.data() else {
            panic!("the stats figure's values");
        };
        let kde = Kde::new(values.iter().copied()).expect("values that vary");
        assert!(values.len() > Kde::BINNED);
        let binned = kde.grid(200, 3.0);
        assert_eq!(binned, kde.binned(200, 3.0));
        let threads = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
        let chunk = binned.len().div_ceil(threads);
        let exact: Vec<f64> = std::thread::scope(|scope| {
            let parts: Vec<_> = binned
                .chunks(chunk)
                .map(|part| {
                    scope.spawn(|| part.iter().map(|&(x, _)| kde.eval(x)).collect::<Vec<_>>())
                })
                .collect();
            parts
                .into_iter()
                .flat_map(|part| part.join().expect("a chunk of the exact sum"))
                .collect()
        });
        let worst = binned
            .iter()
            .zip(&exact)
            .map(|(&(_, b), e)| ((b - e) / e).abs())
            .fold(0.0, f64::max);
        assert!(worst <= 1e-3, "{worst}");
    }

    #[test]
    fn subjects_are_named_as_the_workloads_name_sizes() {
        assert_eq!(count(1_000), "1k");
        assert_eq!(count(100_000), "100k");
        assert_eq!(count(10_000_000), "10m");
        assert_eq!(count(1_500), "1500");
        assert_eq!(Subject::Scatter(100_000).name(), "scatter100k");
        assert_eq!(Subject::Line(10_000_000).name(), "line10m");
        assert_eq!(Subject::Preset("pie").name(), "preset-pie");
    }

    #[test]
    fn every_subject_builds_a_figure_with_panels() {
        // Small ladders keep the test quick; the large ones share the code.
        let subjects = [
            Subject::Scatter(1_000),
            Subject::Line(10_000),
            Subject::Labels,
            Subject::Preset("donut"),
            Subject::Wheels,
        ];
        for subject in subjects {
            let data = subject.data();
            let list = data.figure().draw(subject.size());
            assert!(!list.commands.is_empty(), "{}", subject.name());
        }
    }
}
