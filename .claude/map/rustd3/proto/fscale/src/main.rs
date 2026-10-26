//! Sketch: continuous scales generic over a public transform, with a tick hook.
use std::sync::Arc;

pub trait Transform: Clone {
    fn forward(&self, x: f64) -> f64;
    fn inverse(&self, y: f64) -> f64;
    /// Default: nice ticks in transformed space, mapped back.
    fn ticks(&self, d: [f64; 2], n: usize) -> Vec<f64> {
        let [a, b] = d.map(|v| self.forward(v));
        d3_core::ticks::ticks(a, b, n as f64)
            .into_iter()
            .map(|t| self.inverse(t))
            .collect()
    }
}

#[derive(Clone, Copy)]
pub struct Identity;
impl Transform for Identity {
    fn forward(&self, x: f64) -> f64 { x }
    fn inverse(&self, y: f64) -> f64 { y }
    fn ticks(&self, [a, b]: [f64; 2], n: usize) -> Vec<f64> {
        d3_core::ticks::ticks(a, b, n as f64)
    }
}

#[derive(Clone, Copy)]
pub struct Logit;
impl Transform for Logit {
    fn forward(&self, p: f64) -> f64 { (p / (1.0 - p)).ln() }
    fn inverse(&self, y: f64) -> f64 { 1.0 / (1.0 + (-y).exp()) }
    /// matplotlib-style: 0.5, then decades towards 0 and 1.
    fn ticks(&self, [a, b]: [f64; 2], _: usize) -> Vec<f64> {
        let mut t = vec![0.5];
        for k in 1..=6 {
            let e = 10f64.powi(-k);
            t.push(e);
            t.push(1.0 - e);
        }
        t.retain(|v| *v >= a && *v <= b);
        t.sort_by(f64::total_cmp);
        t
    }
}

#[derive(Clone, Copy)]
pub struct Asinh(pub f64);
impl Transform for Asinh {
    fn forward(&self, x: f64) -> f64 { (x / self.0).asinh() }
    fn inverse(&self, y: f64) -> f64 { y.sinh() * self.0 }
}

/// Any pair of closures, for one-off transforms.
#[derive(Clone)]
pub struct Func(Arc<dyn Fn(f64) -> f64 + Send + Sync>, Arc<dyn Fn(f64) -> f64 + Send + Sync>);
impl Transform for Func {
    fn forward(&self, x: f64) -> f64 { (self.0)(x) }
    fn inverse(&self, y: f64) -> f64 { (self.1)(y) }
}

#[derive(Clone)]
pub struct Continuous<T> { t: T, domain: [f64; 2], range: [f64; 2] }
impl<T: Transform> Continuous<T> {
    pub fn domain(self, domain: [f64; 2]) -> Self { Self { domain, ..self } }
    pub fn range(self, range: [f64; 2]) -> Self { Self { range, ..self } }
    pub fn apply(&self, x: f64) -> f64 {
        let [a, b] = self.domain.map(|v| self.t.forward(v));
        let u = (self.t.forward(x) - a) / (b - a);
        self.range[0] + u * (self.range[1] - self.range[0])
    }
    /// Every continuous scale inverts, because every transform has an inverse.
    pub fn invert(&self, y: f64) -> f64 {
        let [a, b] = self.domain.map(|v| self.t.forward(v));
        let u = (y - self.range[0]) / (self.range[1] - self.range[0]);
        self.t.inverse(a + u * (b - a))
    }
    pub fn ticks(&self, n: usize) -> Vec<f64> { self.t.ticks(self.domain, n) }
}
pub fn continuous<T: Transform>(t: T) -> Continuous<T> {
    Continuous { t, domain: [0.0, 1.0], range: [0.0, 1.0] }
}
pub type Linear = Continuous<Identity>;
pub fn func(
    f: impl Fn(f64) -> f64 + Send + Sync + 'static,
    g: impl Fn(f64) -> f64 + Send + Sync + 'static,
) -> Continuous<Func> {
    continuous(Func(Arc::new(f), Arc::new(g)))
}

fn main() {
    let lin: Linear = continuous(Identity).domain([0.13, 9.71]);
    println!("linear ticks      {:?}", lin.ticks(5));
    let p = continuous(Logit).domain([0.001, 0.999]).range([0.0, 400.0]);
    println!("logit ticks       {:?}", p.ticks(10));
    println!("logit apply(0.5) = {}, invert(200) = {}", p.apply(0.5), p.invert(200.0));
    let a = continuous(Asinh(1.0)).domain([-1000.0, 1000.0]);
    let r = |v: Vec<f64>| v.iter().map(|x| (x * 100.0).round() / 100.0).collect::<Vec<_>>();
    println!("asinh default     {:?}", r(a.ticks(10)));
    let sq = func(|x| x.sqrt(), |y| y * y).domain([0.0, 100.0]);
    println!("func(sqrt) ticks  {:?}", r(sq.ticks(5)));
}
