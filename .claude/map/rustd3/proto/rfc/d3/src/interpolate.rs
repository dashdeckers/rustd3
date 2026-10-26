//! Typed interpolation. d3's `interpolate(a, b)` inspects runtime types; here the type decides.

/// A value that can be blended with another of the same type.
pub trait Interpolate: Clone {
    /// The value a fraction `t` of the way from `self` to `to`.
    fn lerp(&self, to: &Self, t: f64) -> Self;
}

impl Interpolate for f64 {
    fn lerp(&self, to: &Self, t: f64) -> Self {
        self * (1.0 - t) + to * t
    }
}

impl Interpolate for f32 {
    fn lerp(&self, to: &Self, t: f64) -> Self {
        (*self as f64).lerp(&(*to as f64), t) as f32
    }
}

impl<T: Interpolate, const N: usize> Interpolate for [T; N] {
    fn lerp(&self, to: &Self, t: f64) -> Self {
        std::array::from_fn(|i| self[i].lerp(&to[i], t))
    }
}

impl<A: Interpolate, B: Interpolate> Interpolate for (A, B) {
    fn lerp(&self, to: &Self, t: f64) -> Self {
        (self.0.lerp(&to.0, t), self.1.lerp(&to.1, t))
    }
}

/// d3's `interpolateRound`.
pub fn round(a: f64, b: f64) -> impl Fn(f64) -> f64 {
    move |t| a.lerp(&b, t).round()
}

/// d3's `interpolate(a, b)` as a closure, for any interpolatable type.
pub fn between<T: Interpolate>(a: T, b: T) -> impl Fn(f64) -> T {
    move |t| a.lerp(&b, t)
}
