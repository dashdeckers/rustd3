use d3::scale;
use figmodel::plot::{Figure, Grid, bottom, histogram, label_trace, left};

fn main() {
    let mut fig = Figure::new();
    let bins: Vec<String> = (0..20).map(|i| format!("{}–{}", i * 10, i * 10 + 10)).collect();
    let x = fig.scale(scale::band(bins.clone()).padding(0.1));
    let y = fig.fit(scale::linear());
    fig.layout(Grid::new());
    fig.panel(0, 0).mark(histogram(bins.clone(), x, y)).axis(bottom(x)).axis(left(y));
    let r = fig.resolve([320.0, 240.0]);
    let q = r.rects[0];
    println!("rect {:.1} {:.1} {:.1} {:.1}", q.x, q.y, q.w, q.h);
    let v: Vec<String> = label_trace(&fig, &r)
        .into_iter()
        .filter(|l| l.1 == 0)
        .map(|(_, _, t, b, k)| format!("[\"{t}\",{:.1},{:.1},{k}]", b[0], b[2]))
        .collect();
    println!("[{}]", v.join(","));
}
