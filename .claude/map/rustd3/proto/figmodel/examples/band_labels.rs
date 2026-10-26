//! All 20 speed bins labeled "40–50": which labels does
//! the greedy pass keep at different widths?

use d3::scale;
use figmodel::plot::{Figure, Grid, bottom, histogram, left};

fn main() {
    for w in [320.0, 480.0, 960.0, 1400.0] {
        let mut fig = Figure::new();
        let bins: Vec<String> = (0..20)
            .map(|i| format!("{}–{}", i * 10, i * 10 + 10))
            .collect();
        let x = fig.scale(scale::band(bins.clone()).padding(0.1));
        let y = fig.fit(scale::linear());
        fig.layout(Grid::new());
        fig.panel(0, 0)
            .mark(histogram(bins.clone(), x, y))
            .axis(bottom(x))
            .axis(left(y));
        let r = fig.resolve([w, 240.0]);
        let keep = &r.keep[0][0];
        let kept: Vec<&str> = bins
            .iter()
            .zip(keep)
            .filter(|(_, k)| **k)
            .map(|(b, _)| b.as_str())
            .collect();
        println!("{w:>5} px: kept {:>2} of 20: {}", kept.len(), kept.join(" "));
    }
}
