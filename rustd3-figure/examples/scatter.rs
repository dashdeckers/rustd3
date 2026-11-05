//! Writes `scatter.svg` and `scatter.png` into the directory given on the
//! command line: 5,600 points in three palette keys, three `Markers`
//! commands. rustd3-figure's tests draw the same figure, [`scatter`].

use std::path::PathBuf;

use rustd3_core::scale;
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::kurbo::Size;
use rustd3_figure::mark::dots;
use rustd3_figure::{Figure, axis};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir: PathBuf = std::env::args_os()
        .nth(1)
        .ok_or("usage: scatter <directory>")?
        .into();
    let fig = scatter(POINTS);
    for warning in fig.save_svg(dir.join("scatter.svg"), SIZE)? {
        eprintln!("scatter.svg: {warning:?}");
    }
    fig.save_png(dir.join("scatter.png"), SIZE, SCALE)?;
    Ok(())
}

/// The example's points: as many as the wheel figure's dots.
pub const POINTS: usize = 5_600;

/// The example's size in logical px.
pub const SIZE: Size = Size::new(640.0, 480.0);

/// Device px per logical px of the example's PNG.
pub const SCALE: f64 = 2.0;

/// The clusters' keys and centers.
const CLUSTERS: [(&str, [f64; 2]); 3] = [("a", [0.0, 0.0]), ("b", [2.5, 1.0]), ("c", [1.0, 2.5])];

/// A figure of `n` points in three Gaussian clusters, point `i` in cluster
/// `i % 3`. Each cluster is one dots mark filled with its palette color,
/// so the figure draws three `Markers` commands however many points there
/// are. The normal deviates come from the wheel figure's LCG (seed 11)
/// through Box–Muller.
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
