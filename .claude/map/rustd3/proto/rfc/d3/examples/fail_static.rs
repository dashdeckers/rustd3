// Accessors stored as 'static boxes cannot borrow a scale built in the view.
use d3::scale;
struct StaticLine<D> {
    x: Box<dyn Fn(&D) -> f64>,
}
fn main() {
    let x = scale::linear().domain([0.0, 10.0]).range([0.0, 100.0]);
    let _line: StaticLine<f64> = StaticLine { x: Box::new(|d: &f64| x.apply(*d)) };
}
