//! Review F follow-up: tick rules as building blocks,
//! a required `ticks` on Transform, an override on
//! any continuous scale, and a user transform.
use std::sync::Arc;

pub mod ticks {
    use super::Transform;
    /// d3's linear ticks in data space.
    pub fn linear(d: [f64; 2], n: usize) -> Vec<f64> {
        d3_core::ticks::ticks(d[0], d[1], n as f64)
    }
    /// Nice ticks in transformed space, mapped back.
    pub fn transformed<T: Transform>(t: &T, d: [f64; 2], n: usize) -> Vec<f64> {
        linear(d.map(|v| t.forward(v)), n)
            .into_iter()
            .map(|v| t.inverse(v))
            .collect()
    }
    /// 1-2-…-9 per decade, or decades only when
    /// that would exceed the count (d3's log ticks).
    pub fn log(base: f64, d: [f64; 2], n: usize) -> Vec<f64> {
        let (a, b) = (d[0].min(d[1]), d[0].max(d[1]));
        let (i, j) = (a.log(base).floor() as i32, b.log(base).ceil() as i32);
        let mut t = Vec::new();
        let all = ((j - i) as usize) < n;
        for k in i..=j {
            let p = base.powi(k);
            let steps: Vec<f64> = if all { (1..base as i32).map(f64::from).collect() } else { vec![1.0] };
            for s in steps {
                let v = s * p;
                if v >= a && v <= b {
                    t.push(v);
                }
            }
        }
        t
    }
    /// 0 and ±c·10^k, every k-th decade to stay
    /// near the count (symlog).
    pub fn decades(d: [f64; 2], c: f64, n: usize) -> Vec<f64> {
        let m = d[0].abs().max(d[1].abs());
        let mut t = if d[0] <= 0.0 && 0.0 <= d[1] { vec![0.0] } else { vec![] };
        let mut ks = Vec::new();
        let mut k = 0;
        while c * 10f64.powi(k) <= m {
            ks.push(k);
            k += 1;
        }
        let sides = if d[0] < 0.0 && d[1] > 0.0 { 2 } else { 1 };
        let step = (ks.len() * sides).div_ceil(n.max(1)).max(1);
        for k in ks.into_iter().filter(|k| *k as usize % step == 0) {
            for v in [c * 10f64.powi(k), -c * 10f64.powi(k)] {
                if v >= d[0] && v <= d[1] {
                    t.push(v);
                }
            }
        }
        t.sort_by(f64::total_cmp);
        t
    }
    /// Minor ticks: k equal steps in transformed
    /// space between neighbouring majors.
    pub fn subdivide<T: Transform>(t: &T, major: &[f64], k: usize) -> Vec<f64> {
        major
            .windows(2)
            .flat_map(|w| {
                let (a, b) = (t.forward(w[0]), t.forward(w[1]));
                (1..k).map(move |i| a + (b - a) * i as f64 / k as f64)
            })
            .map(|v| t.inverse(v))
            .collect()
    }
}

pub trait Transform: Clone {
    fn forward(&self, x: f64) -> f64;
    fn inverse(&self, y: f64) -> f64;
    /// Required: a transform states its tick rule,
    /// usually by calling one from `ticks`.
    fn ticks(&self, domain: [f64; 2], count: usize) -> Vec<f64>;
    /// Minor ticks between the majors; override for
    /// log-like rules (2–9 per decade).
    fn minor(&self, major: &[f64]) -> Vec<f64> {
        ticks::subdivide(self, major, 5)
    }
}

#[derive(Clone, Copy)]
pub struct Identity;
impl Transform for Identity {
    fn forward(&self, x: f64) -> f64 { x }
    fn inverse(&self, y: f64) -> f64 { y }
    fn ticks(&self, d: [f64; 2], n: usize) -> Vec<f64> { ticks::linear(d, n) }
}

#[derive(Clone, Copy)]
pub struct Symlog(pub f64);
impl Transform for Symlog {
    fn forward(&self, x: f64) -> f64 { x.signum() * (x.abs() / self.0).ln_1p() }
    fn inverse(&self, y: f64) -> f64 { y.signum() * y.abs().exp_m1() * self.0 }
    fn ticks(&self, d: [f64; 2], n: usize) -> Vec<f64> {
        if d[0].abs().max(d[1].abs()) / self.0 < 100.0 {
            ticks::linear(d, n)
        } else {
            ticks::decades(d, self.0, n)
        }
    }
}

#[derive(Clone)]
pub enum TickRule {
    Transformed,
    Data,
    Custom(Arc<dyn Fn([f64; 2], usize) -> Vec<f64> + Send + Sync>),
}

#[derive(Clone)]
pub struct Func {
    f: Arc<dyn Fn(f64) -> f64 + Send + Sync>,
    g: Arc<dyn Fn(f64) -> f64 + Send + Sync>,
    rule: TickRule,
}
impl Transform for Func {
    fn forward(&self, x: f64) -> f64 { (self.f)(x) }
    fn inverse(&self, y: f64) -> f64 { (self.g)(y) }
    fn ticks(&self, d: [f64; 2], n: usize) -> Vec<f64> {
        apply_rule(&self.rule, self, d, n)
    }
}

fn apply_rule<T: Transform>(r: &TickRule, t: &T, d: [f64; 2], n: usize) -> Vec<f64> {
    match r {
        TickRule::Transformed => ticks::transformed(t, d, n),
        TickRule::Data => ticks::linear(d, n),
        TickRule::Custom(f) => f(d, n),
    }
}

#[derive(Clone)]
pub struct Continuous<T> {
    t: T,
    domain: [f64; 2],
    rule: Option<TickRule>,
}
impl<T: Transform> Continuous<T> {
    pub fn domain(self, domain: [f64; 2]) -> Self { Self { domain, ..self } }
    /// Overrides the transform's own rule.
    pub fn tick_rule(self, r: TickRule) -> Self { Self { rule: Some(r), ..self } }
    pub fn ticks(&self, n: usize) -> Vec<f64> {
        match &self.rule {
            None => self.t.ticks(self.domain, n),
            Some(r) => apply_rule(r, &self.t, self.domain, n),
        }
    }
    pub fn minor(&self, n: usize) -> Vec<f64> { self.t.minor(&self.ticks(n)) }
}
pub fn continuous<T: Transform>(t: T) -> Continuous<T> {
    Continuous { t, domain: [0.0, 1.0], rule: None }
}
pub fn symlog(c: f64) -> Continuous<Symlog> { continuous(Symlog(c)) }
/// The rule is an argument, so the choice is
/// required by the signature, not by a runtime check.
pub fn func(
    f: impl Fn(f64) -> f64 + Send + Sync + 'static,
    g: impl Fn(f64) -> f64 + Send + Sync + 'static,
    rule: TickRule,
) -> Continuous<Func> {
    continuous(Func { f: Arc::new(f), g: Arc::new(g), rule })
}

// ---- user code: a transform the library lacks ----

/// Frequency on an axis laid out by period (1/x).
#[derive(Clone, Copy)]
struct Reciprocal;
impl Transform for Reciprocal {
    fn forward(&self, x: f64) -> f64 { 1.0 / x }
    fn inverse(&self, y: f64) -> f64 { 1.0 / y }
    fn ticks(&self, d: [f64; 2], n: usize) -> Vec<f64> {
        ticks::log(10.0, d, n)
    }
}

fn show(name: &str, v: Vec<f64>) {
    let s: Vec<String> = v.iter().map(|x| format!("{}", (x * 1000.0).round() / 1000.0)).collect();
    println!("{name:28} {}", s.join(" "));
}

fn main() {
    show("symlog [0, 1e6]", symlog(1.0).domain([0.0, 1e6]).ticks(10));
    show("symlog, d3's rule", symlog(1.0).domain([0.0, 1e6]).tick_rule(TickRule::Data).ticks(10));
    show("symlog [-3, 3]", symlog(1.0).domain([-3.0, 3.0]).ticks(10));
    show("symlog minor [-1e3, 1e3]", symlog(1.0).domain([-1e3, 1e3]).minor(10).into_iter().filter(|v| v.abs() < 12.0).collect());
    show("func sqrt, Data", func(f64::sqrt, |y| y * y, TickRule::Data).domain([0.0, 100.0]).ticks(5));
    show("func sqrt, Transformed", func(f64::sqrt, |y| y * y, TickRule::Transformed).domain([0.0, 100.0]).ticks(5));
    let every_3 = TickRule::Custom(Arc::new(|d: [f64; 2], _| {
        (0..).map(|i| d[0] + 3.0 * i as f64).take_while(|v| *v <= d[1]).collect()
    }));
    show("linear, custom closure", continuous(Identity).domain([0.0, 12.0]).tick_rule(every_3).ticks(5));
    show("user Reciprocal [1, 100]", continuous(Reciprocal).domain([1.0, 100.0]).ticks(10));
    show("user Reciprocal [1, 1e6]", continuous(Reciprocal).domain([1.0, 1e6]).ticks(5));
}
