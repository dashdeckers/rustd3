//! The `grid16` workload: 4 × 4 panels, x shared per column and y per row;
//! per panel a line over its own recording and dots, with a left and a
//! bottom axis, zoomed on both.

use rustd3_core::scale;
use rustd3_core::series::{self, Clock, Recording};
use rustd3_figure::mark::{dots, line};
use rustd3_figure::scale::Fit;
use rustd3_figure::{Figure, axis};

use crate::workload::Rng;
use crate::workload::trace::{RATE, Signal};

/// Samples per panel's recording at full size: 10 s at 10 kHz.
pub const SAMPLES: usize = 100_000;

/// Dots per panel at full size.
pub const DOTS: usize = 2_000;

/// Panels per row and per column.
const SIDE: usize = 4;

/// One panel's data.
#[derive(Debug)]
struct Cell {
    rec: Recording<f64>,
    signal: Signal,
    dots: [Vec<f64>; 2],
}

/// Sixteen panels of data, row-major.
#[derive(Debug)]
pub struct Grid {
    cells: Vec<Cell>,
}

impl Grid {
    /// Each panel `samples` samples (seed 100 + panel) and `n` dots over
    /// the same span (seed 200 + panel).
    pub fn new(samples: usize, n: usize) -> Grid {
        let cells = (0..SIDE * SIDE)
            .map(|panel| {
                let mut signal = Signal::new(100 + panel as u64, samples);
                let clock = Clock::Regular {
                    start: 0.0,
                    period: 1.0 / RATE,
                };
                let rec = Recording::new(clock, [signal.take(samples)])
                    .expect("invariant: finite samples");
                let mut rng = Rng::new(200 + panel as u64);
                let span = samples as f64 / RATE;
                let xs: Vec<f64> = (0..n).map(|_| rng.unit() * span).collect();
                let ys: Vec<f64> = (0..n).map(|_| 0.5 * rng.gauss()).collect();
                Cell {
                    rec,
                    signal,
                    dots: [xs, ys],
                }
            })
            .collect();
        Grid { cells }
    }

    /// Appends `n` samples to every panel's recording.
    pub fn append(&mut self, n: usize) {
        for cell in &mut self.cells {
            cell.rec
                .extend(&[], [cell.signal.take(n)])
                .expect("invariant: finite samples on a regular clock");
        }
    }

    /// The figure; `windowed` puts every column's x on `Fit::Window(10.0)`.
    pub fn figure(&self, windowed: bool) -> Figure<'_> {
        let mut fig = Figure::new();
        let xs: Vec<_> = (0..SIDE)
            .map(|_| {
                if windowed {
                    fig.fit_with(scale::linear(), Fit::Window(10.0))
                } else {
                    fig.fit(scale::linear())
                }
            })
            .collect();
        let ys: Vec<_> = (0..SIDE).map(|_| fig.fit(scale::linear())).collect();
        for (index, cell) in self.cells.iter().enumerate() {
            let (row, col) = (index / SIDE, index % SIDE);
            let (x, y) = (xs[col], ys[row]);
            let [dx, dy] = &cell.dots;
            fig.panel(row, col)
                .mark(line(
                    cell.rec.series(0).expect("invariant: one channel"),
                    x,
                    y,
                ))
                .mark(dots(series::slices(dx, dy), x, y).r(1.5))
                .axis(axis::left(y))
                .axis(axis::bottom(x))
                .zoom_x(x)
                .zoom_y(y);
        }
        fig
    }
}
