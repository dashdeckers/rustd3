use d3::scale;
use figmodel::fig::Figure;
use jiff::Timestamp;
fn main() {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let t = Timestamp::UNIX_EPOCH;
    fig.panel(0, 0).line([(t, 1.0)], x, y);
}
