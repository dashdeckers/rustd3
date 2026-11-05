//! The `scatter100k` and `scatter1m` workloads: one panel of dots in five
//! Gaussian clusters, a dots mark per cluster in its palette color, zoomed
//! on both handles.

use rustd3_core::scheme::COLORBLIND;
use rustd3_core::{scale, series};
use rustd3_figure::mark::dots;
use rustd3_figure::{Figure, axis};

use crate::workload::Rng;

/// The clusters: key, centre and standard deviations.
const CLUSTERS: [(&str, [f64; 2], [f64; 2]); 5] = [
    ("a", [0.0, 0.0], [1.0, 0.6]),
    ("b", [3.0, 1.0], [0.8, 0.8]),
    ("c", [1.0, 3.5], [0.5, 1.2]),
    ("d", [4.5, 4.0], [1.2, 0.5]),
    ("e", [-2.0, 2.5], [0.7, 0.7]),
];

/// The dots, as two columns per cluster.
#[derive(Debug)]
pub struct Scatter {
    clusters: Vec<[Vec<f64>; 2]>,
}

impl Scatter {
    /// `n` dots, dot `i` in cluster `i % 5`, seed 11.
    pub fn new(n: usize) -> Scatter {
        let mut rng = Rng::new(11);
        let mut clusters: Vec<[Vec<f64>; 2]> =
            CLUSTERS.iter().map(|_| Default::default()).collect();
        for i in 0..n {
            let (_, [cx, cy], [sx, sy]) = CLUSTERS[i % CLUSTERS.len()];
            let [xs, ys] = &mut clusters[i % CLUSTERS.len()];
            xs.push(cx + sx * rng.gauss());
            ys.push(cy + sy * rng.gauss());
        }
        Scatter { clusters }
    }

    /// The figure: fitted x and y, left and bottom axes, both zoomed.
    pub fn figure(&self) -> Figure<'_> {
        let mut fig = Figure::new();
        let x = fig.fit(scale::linear());
        let y = fig.fit(scale::linear());
        let color = fig.scale(scale::ordinal(CLUSTERS.map(|c| c.0), COLORBLIND));
        let panel = fig
            .panel(0, 0)
            .axis(axis::left(y).label("y"))
            .axis(axis::bottom(x).label("x"))
            .zoom_x(x)
            .zoom_y(y);
        for ((key, _, _), [xs, ys]) in CLUSTERS.iter().zip(&self.clusters) {
            panel.mark(dots(series::slices(xs, ys), x, y).fill(color, *key));
        }
        fig
    }
}
