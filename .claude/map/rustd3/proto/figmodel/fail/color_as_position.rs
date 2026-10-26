use d3::color::Rgb;
use d3::scale;
use figmodel::fig::Figure;
fn main() {
    let mut fig = Figure::new();
    let ends = [Rgb::hex(0x000000), Rgb::hex(0xffffff)];
    let c = fig.fit(scale::linear().range(ends));
    let y = fig.fit(scale::linear());
    fig.panel(0, 0).line([(0.0, 1.0)], c, y);
}
