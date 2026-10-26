//! Does the plot area move while zooming? Relayout on
//! every event, against layout frozen for the gesture.

use d3::scale;
use figmodel::plot::{Figure, Grid, bottom, dots, left};
use figmodel::plot::{label_trace, protrusion_of};
use std::collections::HashMap;

fn shown(fig: &Figure, r: &figmodel::plot::Resolved) -> String {
    let t: Vec<String> = label_trace(fig, r)
        .into_iter()
        .filter(|l| l.1 == 0)
        .map(|l| l.2)
        .collect();
    format!("{} .. {}", t[0], t[t.len() - 1])
}

#[allow(dead_code)]
fn labels(d: [f64; 2]) -> String {
    let s = scale::linear().domain(d);
    let f = s.tick_format(5, None).unwrap();
    let t: Vec<String> = s.ticks(5).iter().map(|v| f.format(*v)).collect();
    format!("{} .. {}", t[0], t[t.len() - 1])
}

fn main() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let pts: Vec<(f64, f64)> = (0..50)
        .map(|i| (i as f64, (i as f64 / 5.0).sin() * 4.0 + 4.5))
        .collect();
    fig.layout(Grid::new());
    fig.panel(0, 0)
        .mark(dots(pts, x, y))
        .axis(left(y).label("value"))
        .axis(bottom(x))
        .zoom_y(y);
    let size = [480.0, 320.0];
    let mut views: HashMap<usize, [f64; 2]> = HashMap::new();
    let start = fig.resolve(size);
    let frozen = start.rects.clone();
    let reach0 = protrusion_of(&fig, &start)[0][3];
    println!("step  y domain           ticks          plot x (live)  overflow (frozen)");
    for step in 0..8 {
        let live = fig.resolve_with(size, &views);
        let held = fig.resolve_in(size, &views, Some(&frozen));
        let reach = protrusion_of(&fig, &held)[0][3];
        let d = live.raw(y).get_domain();
        println!(
            "{step:>4}  [{:>6.3}, {:>6.3}]  {:<14} {:6.1}         {:5.1} px",
            d[0], d[1], shown(&fig, &live), live.rects[0].x, (reach - reach0).max(0.0)
        );
        let q = live.rects[0];
        let at = (q.x + q.w / 2.0, q.y + 0.6 * q.h);
        for (i, d) in fig.wheel(&live, at, -20.0) {
            views.insert(i, d);
        }
    }
}
