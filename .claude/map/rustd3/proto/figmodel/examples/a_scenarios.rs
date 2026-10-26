//! Option A on four layouts: crossing shares, a scatter
//! matrix, twin axes, and two heatmaps with one colorbar.

use d3::axis::Orient::{Bottom, Left, Right};
use d3::color::Rgb;
use d3::scale;
use figmodel::fig::{Figure, Resolved};
use std::collections::HashMap;

fn noise(seed: &mut u32) -> f64 {
    *seed = seed
        .wrapping_mul(1_664_525)
        .wrapping_add(1_013_904_223);
    (*seed >> 8) as f64 / (1u32 << 24) as f64 - 0.5
}

fn report(name: &str, fig: &Figure) -> Resolved {
    let r = fig.resolve([720.0, 480.0], &HashMap::new());
    let domains: Vec<String> = r
        .scales
        .iter()
        .map(|s| format!("{:?}", s.domain()))
        .collect();
    println!("{name}: {}", domains.join(" "));
    let path = format!("out/a_{name}.svg");
    std::fs::write(path, r.svg(fig)).unwrap();
    r
}

fn main() {
    let mut seed = 3;
    let mut n = || noise(&mut seed);

    // 1. x shared within each column, y within each row.
    let mut fig = Figure::new();
    let g = fig.subplots(2, 2);
    for (r, c) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
        let (kx, ky) = ([1.0, 10.0][c], [1.0, 10.0][r]);
        let span = kx * (1.0 + r as f64 / 2.0);
        let data: Vec<(f64, f64)> = (0..50)
            .map(|i| i as f64 * span)
            .map(|x| (x, (x / 9.0 / kx).sin() * ky + n()))
            .collect();
        fig.panel(r, c)
            .line(data, g.xs[c], g.ys[r])
            .axis(Bottom, g.xs[c])
            .axis(Left, g.ys[r]);
    }
    report("grid", &fig);

    // 2. A scatter matrix: variable i's scale is x in
    //    column i and y in row i.
    let cols: Vec<Vec<f64>> = [1.0, 10.0, 100.0]
        .iter()
        .map(|s| (0..80).map(|_| n() * s).collect())
        .collect();
    let mut fig = Figure::new();
    let vars: Vec<_> = cols
        .iter()
        .map(|_| fig.fit(scale::linear()))
        .collect();
    for i in 0..3 {
        for j in (0..3).filter(|&j| j != i) {
            let pts = cols[j]
                .iter()
                .copied()
                .zip(cols[i].iter().copied());
            fig.panel(i, j)
                .dots(pts, vars[j], vars[i])
                .axis(Bottom, vars[j])
                .axis(Left, vars[i])
                .zoom_x(vars[j]);
        }
    }
    report("splom", &fig);

    // 3. Twin axes: one panel, two y scales.
    let mut fig = Figure::new();
    let day = fig.fit(scale::linear());
    let temp = fig.fit(scale::linear());
    let rain = fig.fit(scale::linear());
    let t: Vec<(f64, f64)> = (0..120)
        .map(|d| d as f64)
        .map(|d| (d, 8.0 + 10.0 * (d / 19.0).sin()))
        .collect();
    let mm: Vec<(f64, f64)> = (0..120)
        .map(|d| (d as f64, 40.0 + 30.0 * n()))
        .collect();
    fig.panel(0, 0)
        .line(t, day, temp)
        .line(mm, day, rain)
        .axis(Bottom, day)
        .axis(Left, temp)
        .axis(Right, rain);
    report("twin", &fig);

    // 4. Two heatmaps, one color scale, one colorbar.
    let mut fig = Figure::new();
    let viridis_ends = [Rgb::hex(0x440154), Rgb::hex(0xfde725)];
    let color = fig.fit(scale::linear().range(viridis_ends));
    let small = vec![vec![0.0, 1.0], vec![2.0, 3.0]];
    let large = vec![vec![0.0, 5.0], vec![10.0, 20.0]];
    fig.panel(0, 0).heatmap(small, color);
    fig.panel(0, 1).heatmap(large, color);
    fig.panel(0, 2).colorbar(color);
    report("colorbar", &fig);
}
