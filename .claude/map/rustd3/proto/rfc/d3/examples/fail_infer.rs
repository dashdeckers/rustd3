// The same API without a type annotation on the closure.
pub enum Accessor<'a, D> { Const(f64), Fn(Box<dyn Fn(&D) -> f64 + 'a>) }
impl<D> From<f64> for Accessor<'_, D> { fn from(v: f64) -> Self { Accessor::Const(v) } }
impl<'a, D, F: Fn(&D) -> f64 + 'a> From<F> for Accessor<'a, D> { fn from(f: F) -> Self { Accessor::Fn(Box::new(f)) } }
pub struct Bar<'a, D> { pub y1: Accessor<'a, D> }
impl<'a, D> Bar<'a, D> {
    pub fn y1(self, a: impl Into<Accessor<'a, D>>) -> Self { Self { y1: a.into() } }
}
struct Row { v: f64 }
fn main() {
    let bar = Bar::<Row> { y1: 0.0.into() };
    let _ = bar.y1(|r| r.v);
}
