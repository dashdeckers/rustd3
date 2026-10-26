// d3's "constant or function" as `impl Into<Accessor<D>>`. Compiles: Fn traits are #[fundamental].
pub enum Accessor<'a, D> {
    Const(f64),
    Fn(Box<dyn Fn(&D) -> f64 + 'a>),
}
impl<D> From<f64> for Accessor<'_, D> {
    fn from(v: f64) -> Self { Accessor::Const(v) }
}
impl<'a, D, F: Fn(&D) -> f64 + 'a> From<F> for Accessor<'a, D> {
    fn from(f: F) -> Self { Accessor::Fn(Box::new(f)) }
}
pub struct Bar<'a, D> { pub y0: Accessor<'a, D>, pub y1: Accessor<'a, D> }
impl<'a, D> Bar<'a, D> {
    pub fn new() -> Self { Self { y0: 0.0.into(), y1: 0.0.into() } }
    pub fn y0(self, a: impl Into<Accessor<'a, D>>) -> Self { Self { y0: a.into(), ..self } }
    pub fn y1(self, a: impl Into<Accessor<'a, D>>) -> Self { Self { y1: a.into(), ..self } }
}
struct Row { v: f64 }
fn main() {
    let _ok = Bar::<Row>::new().y0(300.0).y1(|r: &Row| r.v);
}
