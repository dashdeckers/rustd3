//! Zooming variable a where it is x also rescales row a,
//! where it is y: one scale, two roles.

use d3::axis::Orient::{Bottom, Left};
use d3::interact::Pointer;
use d3::scale;
use figmodel::fig::Figure;
use figmodel::live::{Chart, State};

fn main() {
    let mut seed = 3u32;
    let mut n = || {
        seed = seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        (seed >> 8) as f64 / (1u32 << 24) as f64 - 0.5
    };
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
    let chart: Chart<'_, ()> = Chart::new(&fig, [720.0, 480.0]);
    let mut state = State::default();
    let r = chart.resolved(&state);
    let (p, _) = fig_panel(&r, 1, 0);
    let c = (p.x + p.w / 2.0, p.y + p.h / 2.0);
    let before = r.scales[0].domain();
    let wheel = Pointer::Wheel { x: c.0, y: c.1, lines: -20.0 };
    chart.update(&mut state, wheel);
    let r = chart.resolved(&state);
    println!("variable a: {before:?} -> {:?}", r.scales[0].domain());
    let a_as_y = r.along(1, 0, false);
    println!(
        "panel (0, 1) y axis now spans {:?}",
        [a_as_y.invert(r.rects[1].h), a_as_y.invert(0.0)]
    );
    std::fs::write("out/a_splom_zoomed.svg", r.svg(&fig)).unwrap();
}

/// The rect of the panel at a cell: panels are numbered
/// in creation order, row by row here.
fn fig_panel(
    r: &figmodel::fig::Resolved,
    row: usize,
    col: usize,
) -> (figmodel::fig::Rect, usize) {
    let order = [(0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)];
    let i = order.iter().position(|&c| c == (row, col)).unwrap();
    (r.rects[i], i)
}
