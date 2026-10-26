// d3 accessors receive (d, i, data); Rust closures have one fixed arity.
use d3::shape;
fn main() {
    let data = [1.0, 2.0, 3.0];
    let _ = shape::line_of().x(|d: &f64, i: usize| i as f64 * d);
    let _ = data;
}
