//! Can generated code pick "lerp if the type can, else
//! jump at t = 1" without an annotation? Autoref-based
//! specialization, as a derive would emit it.

pub trait Interpolate {
    fn lerp(&self, to: &Self, t: f64) -> Self;
}
impl Interpolate for f64 {
    fn lerp(&self, to: &f64, t: f64) -> f64 {
        self + (to - self) * t
    }
}

pub struct Pick<'a, T>(pub &'a T, pub &'a T);

// Chosen when T: Interpolate (one autoref level).
pub trait ViaLerp<T> {
    fn blend(&self, t: f64) -> T;
}
impl<T: Interpolate> ViaLerp<T> for Pick<'_, T> {
    fn blend(&self, t: f64) -> T {
        self.0.lerp(self.1, t)
    }
}
// Fallback, one more autoref: jump at t = 1.
pub trait ViaJump<T> {
    fn blend(&self, t: f64) -> T;
}
impl<T: Clone> ViaJump<T> for &Pick<'_, T> {
    fn blend(&self, t: f64) -> T {
        if t < 1.0 { self.0.clone() } else { self.1.clone() }
    }
}

// What the derive would generate for each field:
macro_rules! field {
    ($a:expr, $b:expr, $t:expr) => {{
        #[allow(unused_imports)]
        use crate::{ViaJump, ViaLerp};
        (&Pick(&$a, &$b)).blend($t)
    }};
}

#[derive(Debug, Clone)]
struct Label {
    x: f64,
    width: f32, // no impl for f32 here: a typo-like gap
    text: String,
}

fn lerp_label(a: &Label, b: &Label, t: f64) -> Label {
    Label {
        x: field!(a.x, b.x, t),
        width: field!(a.width, b.width, t),
        text: field!(a.text, b.text, t),
    }
}

// The same inside generic code: T is unknown at
// expansion, so method lookup cannot see T's impls.
fn lerp_generic<T: Clone + Interpolate>(a: &T, b: &T, t: f64) -> T {
    field!(*a, *b, t)
}

fn main() {
    let a = Label { x: 0.0, width: 0.0, text: "a".into() };
    let b = Label { x: 10.0, width: 10.0, text: "b".into() };
    println!("t=0.5 {:?}", lerp_label(&a, &b, 0.5));
    println!("generic f64 t=0.5: {}", lerp_generic(&0.0f64, &10.0, 0.5));
}

// A generic mark whose derive adds only `T: Clone`:
// even when T = f64, the fallback was fixed at
// expansion time.
#[derive(Debug, Clone)]
struct Mark<T> { v: T }
fn lerp_mark<T: Clone>(a: &Mark<T>, b: &Mark<T>, t: f64) -> Mark<T> {
    Mark { v: field!(a.v, b.v, t) }
}
#[test]
fn generic_falls_back() {
    let m = lerp_mark(&Mark { v: 0.0f64 }, &Mark { v: 10.0 }, 0.5);
    println!("generic Mark<f64> t=0.5: {:?}", m);
}
