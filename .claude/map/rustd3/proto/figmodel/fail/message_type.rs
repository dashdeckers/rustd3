use d3::time::time;
use figmodel::fig::Figure;
use figmodel::live::Chart;
use jiff::tz::TimeZone;
enum Msg {
    Range([f64; 2]),
}
fn main() {
    let mut fig = Figure::new();
    let x = fig.fit(time(TimeZone::UTC));
    let _ = Chart::new(&fig, [1.0, 1.0])
        .on_change(x, Msg::Range);
}
