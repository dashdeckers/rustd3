//! What measured layout plus the collision pass cost, as
//! the grid grows: every panel has a title, a left and a
//! bottom axis, x shared per column, y per row.

use d3::scale;
use figmodel::plot::{Figure, Grid, bottom, dots, left};
use std::hint::black_box;
use std::time::Instant;

fn main() {
    for n in [2usize, 4, 8, 16] {
        let mut fig = Figure::new();
        let xs: Vec<_> = (0..n).map(|_| fig.fit(scale::linear())).collect();
        let ys: Vec<_> = (0..n).map(|_| fig.fit(scale::linear())).collect();
        fig.layout(Grid::new());
        for r in 0..n {
            for c in 0..n {
                let k = (1 + r * n + c) as f64;
                let pts: Vec<(f64, f64)> =
                    (0..20).map(|i| (i as f64 * k, (i as f64).sin() * k * 100.0)).collect();
                fig.panel(r, c)
                    .title(format!("panel {r},{c}"))
                    .mark(dots(pts, xs[c], ys[r]))
                    .axis(left(ys[r]).label("y"))
                    .axis(bottom(xs[c]).label("x"));
            }
        }
        let size = [240.0 * n as f64, 200.0 * n as f64];
        let reps = if n < 16 { 200 } else { 40 };
        let t = Instant::now();
        let mut kept = 0;
        for _ in 0..reps {
            let r = black_box(fig.resolve(size));
            kept = r.keep.iter().flatten().flatten().filter(|k| **k).count();
        }
        let ms = t.elapsed().as_secs_f64() * 1e3 / reps as f64;
        let labels: usize = {
            let r = fig.resolve(size);
            r.keep.iter().flatten().map(|a| a.len()).sum()
        };
        let passes = fig.resolve(size).passes;
        println!("{n:>2}x{n:<2} {:>4} panels: resolve {ms:7.3} ms, {passes} passes  ({kept} of {labels} labels kept)", n * n);
    }
}
