fn piecewise<R, const N: usize>(d: [f64; N], r: [R; N]) -> ([f64; N], [R; N]) { (d, r) }
fn main() { let _ = piecewise([0.0, 50.0, 100.0], [1.0, 2.0]); }
