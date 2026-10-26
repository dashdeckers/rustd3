//! Polylinear domains: const generic stops vs a
//! runtime list, for f64 and color ranges.

use smallvec::SmallVec;
use std::hint::black_box;
use std::time::Instant;

trait Lerp: Copy {
    fn lerp(self, b: Self, t: f64) -> Self;
}
impl Lerp for f64 {
    fn lerp(self, b: f64, t: f64) -> f64 {
        self + (b - self) * t
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct Rgb(f64, f64, f64);
impl Lerp for Rgb {
    fn lerp(self, b: Rgb, t: f64) -> Rgb {
        Rgb(self.0.lerp(b.0, t), self.1.lerp(b.1, t), self.2.lerp(b.2, t))
    }
}

/// Today's two-stop scale.
#[derive(Clone, Copy)]
struct Two<R> {
    d: [f64; 2],
    r: [R; 2],
}
impl<R: Lerp> Two<R> {
    fn apply(&self, x: f64) -> R {
        let t = (x - self.d[0]) / (self.d[1] - self.d[0]);
        self.r[0].lerp(self.r[1], t)
    }
}

/// Stops as a const generic: N is part of the type.
#[derive(Clone, Copy)]
struct Stops<R, const N: usize> {
    d: [f64; N],
    r: [R; N],
}
fn piecewise<R, const N: usize>(d: [f64; N], r: [R; N]) -> Stops<R, N> {
    assert!(N >= 2);
    Stops { d, r }
}
impl<R: Lerp, const N: usize> Stops<R, N> {
    fn apply(&self, x: f64) -> R {
        // d3: bisect the inner stops, then interpolate.
        let i = if N == 2 {
            0
        } else {
            self.d[1..N - 1].partition_point(|s| *s <= x)
        };
        let t = (x - self.d[i]) / (self.d[i + 1] - self.d[i]);
        self.r[i].lerp(self.r[i + 1], t)
    }
}

/// Stops as a runtime list, inline up to 2 without
/// allocating.
#[derive(Clone)]
struct List<R> {
    d: SmallVec<[f64; 2]>,
    r: SmallVec<[R; 2]>,
}
impl<R: Lerp> List<R> {
    fn new(d: &[f64], r: &[R]) -> Self {
        let n = d.len().min(r.len()); // d3 truncates
        List { d: d[..n].into(), r: r[..n].into() }
    }
    fn apply(&self, x: f64) -> R {
        let n = self.d.len();
        let i = if n == 2 { 0 } else { self.d[1..n - 1].partition_point(|s| *s <= x) };
        let t = (x - self.d[i]) / (self.d[i + 1] - self.d[i]);
        self.r[i].lerp(self.r[i + 1], t)
    }
}

fn bench<R>(name: &str, xs: &[f64], f: impl Fn(f64) -> R) {
    let t = Instant::now();
    for _ in 0..10 {
        for &x in xs {
            black_box(f(black_box(x)));
        }
    }
    let ns = t.elapsed().as_secs_f64() * 1e9 / (10 * xs.len()) as f64;
    println!("  {name:<28} {ns:5.2} ns/point");
}

fn main() {
    let mut s = 3u64;
    let xs: Vec<f64> = (0..1_000_000)
        .map(|_| {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
            (s >> 11) as f64 / (1u64 << 53) as f64 * 100.0
        })
        .collect();
    let (red, white, green) = (Rgb(0.8, 0.2, 0.2), Rgb(1.0, 1.0, 1.0), Rgb(0.2, 0.6, 0.3));
    println!("sizes: Two<f64> {}, Stops<f64,2> {}, List<f64> {}; Two<Rgb> {}, Stops<Rgb,3> {}, List<Rgb> {}",
        size_of::<Two<f64>>(), size_of::<Stops<f64, 2>>(), size_of::<List<f64>>(),
        size_of::<Two<Rgb>>(), size_of::<Stops<Rgb, 3>>(), size_of::<List<Rgb>>());
    println!("two stops, f64:");
    let two = Two { d: [0.0, 100.0], r: [0.0, 960.0] };
    let st2 = piecewise([0.0, 100.0], [0.0, 960.0]);
    let li2 = List::new(&[0.0, 100.0], &[0.0, 960.0]);
    bench("[f64; 2] (today)", &xs, |x| two.apply(x));
    bench("const N = 2", &xs, |x| st2.apply(x));
    bench("SmallVec, 2", &xs, |x| li2.apply(x));
    println!("three stops, Rgb:");
    let st3 = piecewise([0.0, 50.0, 100.0], [red, white, green]);
    let li3 = List::new(&[0.0, 50.0, 100.0], &[red, white, green]);
    bench("const N = 3", &xs, |x| st3.apply(x));
    bench("Vec, 3 (spilled)", &xs, |x| li3.apply(x));
    assert_eq!(st3.apply(25.0), li3.apply(25.0));
    assert_eq!(st3.apply(75.0), Rgb(0.6, 0.8, 0.65));
    // Mismatched lengths: d3 truncates silently.
    let bad = List::new(&[0.0, 50.0, 100.0], &[red, green]);
    println!("List with 3 domain stops, 2 colors: kept {} stops (d3 truncates)", bad.d.len());
}
