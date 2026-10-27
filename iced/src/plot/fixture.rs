//! Figures the widget's unit tests show.

use rustd3_core::scale;
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::legend::{self, Slot};
use rustd3_figure::mark::{dots, line};
use rustd3_figure::{Figure, axis};

/// A 2 × 2 grid: a line, dots, a line and dots, each panel with left and
/// bottom axes, a figure title and a legend right of the grid.
pub(crate) fn grid() -> Figure<'static> {
    let mut fig = Figure::new();
    let color = fig.scale(scale::ordinal(["sine", "noise"], COLORBLIND));
    fig.title("A grid of four");
    fig.legend(Slot::Right, legend::legend(color).title("series"));
    for (i, (row, col)) in [(0, 0), (0, 1), (1, 0), (1, 1)].into_iter().enumerate() {
        let x = fig.fit(scale::linear());
        let y = fig.fit(scale::linear());
        let samples = sine(200, i);
        let panel = fig.panel(row, col);
        if i % 2 == 0 {
            panel.mark(line(samples, x, y).stroke(color, "sine"));
        } else {
            panel.mark(dots(samples, x, y).fill(color, "noise"));
        }
        panel.axis(axis::left(y)).axis(axis::bottom(x));
    }
    fig
}

/// Two panels side by side: a line of 10k samples and 2k dots, each with
/// left and bottom axes, under a title.
pub(crate) fn two_panels() -> Figure<'static> {
    let mut fig = Figure::new();
    fig.title("Line and dots");
    let (x0, y0) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    let (x1, y1) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.panel(0, 0)
        .mark(line(sine(10_000, 0), x0, y0))
        .axis(axis::left(y0))
        .axis(axis::bottom(x0));
    fig.panel(0, 1)
        .mark(dots(sine(2_000, 1), x1, y1))
        .axis(axis::left(y1))
        .axis(axis::bottom(x1));
    fig
}

/// `n` samples of a sine with a little deterministic wobble, phase `seed`.
fn sine(n: usize, seed: usize) -> Vec<(f64, f64)> {
    (0..n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let wobble = ((i * 7919 + seed * 104_729) % 1000) as f64 / 5000.0;
            (t * 10.0, (t * 25.0 + seed as f64).sin() + wobble)
        })
        .collect()
}
