pub trait Interpolate {
    fn lerp(&self, to: &Self, t: f64) -> Self;
}
impl Interpolate for f64 {
    fn lerp(&self, to: &f64, t: f64) -> f64 {
        self + (to - self) * t
    }
}

/// The macro_rules alternative: no proc-macro crate.
macro_rules! interpolate {
    ($t:ident { $($f:ident),* $(,)? }) => {
        impl Interpolate for $t {
            fn lerp(&self, to: &Self, t: f64) -> Self {
                $t { $($f: self.$f.lerp(&to.$f, t)),* }
            }
        }
    };
}

struct Bar {
    y: f64,
    width: f64,
    opacity: f64,
    value: f64,
    
    name: String,
}

#[derive(Debug, Clone, PartialEq)]
struct Dot {
    x: f64,
    y: f64,
    r: f64,
}
interpolate!(Dot { x, y, r });

fn main() {
    let a = Bar { y: 0.0, width: 0.0, opacity: 0.0, value: 0.0, name: "a".into() };
    let b = Bar { y: 40.0, width: 200.0, opacity: 1.0, value: 10.0, name: "b".into() };
    println!("{:?}", a.lerp(&b, 0.25));
    println!("{:?}", a.lerp(&b, 1.0).name);
    let d = Dot { x: 0.0, y: 0.0, r: 1.0 }.lerp(&Dot { x: 10.0, y: 20.0, r: 3.0 }, 0.5);
    println!("{d:?}");
}
