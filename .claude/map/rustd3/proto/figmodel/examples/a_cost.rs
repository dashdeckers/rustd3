//! What rebuilding and resolving a figure costs per frame.

use d3::axis::Orient::{Bottom, Left};
use d3::scale;
use figmodel::fig::Figure;
use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

fn main() {
    for n in [10_000usize, 1_000_000] {
        let data: Vec<(f64, f64)> = (0..n)
            .map(|i| (i as f64, (i as f64 / 1e3).sin()))
            .collect();
        let reps = 20;
        let (mut build, mut resolve) = (0.0, 0.0);
        for _ in 0..reps {
            let t = Instant::now();
            let mut fig = Figure::new();
            let x = fig.fit(scale::linear());
            let y = fig.fit(scale::linear());
            fig.panel(0, 0)
                .line(data.iter().copied(), x, y)
                .axis(Bottom, x)
                .axis(Left, y);
            fig.panel(1, 0)
                .line(data.iter().copied(), x, y)
                .axis(Bottom, x);
            build += t.elapsed().as_secs_f64();
            let t = Instant::now();
            black_box(fig.resolve([800.0, 600.0], &HashMap::new()));
            resolve += t.elapsed().as_secs_f64();
        }
        let ms = |s: f64| s / reps as f64 * 1e3;
        println!(
            "{n:>9} points x 2 panels: build {:.3} ms, resolve {:.3} ms",
            ms(build),
            ms(resolve)
        );
    }
}
