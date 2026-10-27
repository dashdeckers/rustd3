//! The scatter figure: Gaussian clusters in three palette keys.

use rustd3_core::scale;
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::mark::dots;
use rustd3_figure::{Figure, axis};

/// The clusters' keys and centres.
const CLUSTERS: [(&str, [f64; 2]); 3] = [("a", [0.0, 0.0]), ("b", [2.5, 1.0]), ("c", [1.0, 2.5])];

/// `n` points in three Gaussian clusters, point `i` in cluster `i % 3`, as
/// one dots mark per cluster filled with its palette color: three
/// `Markers` commands however many points there are. The normal deviates
/// come from the wheel figure's LCG (seed 11) through Box–Muller
/// (`proto/figmodel/examples/wheels_auto.rs`).
pub fn scatter(n: usize) -> Figure<'static> {
    let mut seed = 11_u32;
    let mut u = move || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        f64::from(seed >> 8) / f64::from(1_u32 << 24)
    };
    let mut gauss = move || {
        let (a, b) = (u().max(1e-12), u());
        (-2.0 * a.ln()).sqrt() * (std::f64::consts::TAU * b).cos()
    };
    let mut points: [Vec<(f64, f64)>; 3] = Default::default();
    for i in 0..n {
        let [cx, cy] = CLUSTERS[i % 3].1;
        let p = (cx + gauss(), cy + 0.6 * gauss());
        points[i % 3].push(p);
    }
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let color = fig.scale(scale::ordinal(CLUSTERS.map(|c| c.0), COLORBLIND));
    let panel = fig
        .panel(0, 0)
        .axis(axis::left(y).label("y"))
        .axis(axis::bottom(x).label("x"));
    for ((key, _), points) in CLUSTERS.into_iter().zip(points) {
        panel.mark(dots(points, x, y).fill(color, key).opacity(0.7));
    }
    fig
}
