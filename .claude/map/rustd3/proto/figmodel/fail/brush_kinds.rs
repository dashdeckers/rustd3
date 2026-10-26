use d3::scale;
use d3::time::time;
use figmodel::fig::Figure;
use jiff::tz::TimeZone;
fn main() {
    let mut fig = Figure::new();
    let full = fig.fit(scale::linear());
    let focus = fig.fit(time(TimeZone::UTC));
    fig.panel(1, 0).brush_x(full, focus);
}
