//! Port of d3-ease: easing functions on normalized time `t` in `[0, 1]`.

use std::f64::consts::{FRAC_PI_2, PI, TAU};

/// An easing function.
pub trait Ease {
    fn ease(&self, t: f64) -> f64;
}

macro_rules! simple {
    ($($name:ident => |$t:ident| $body:expr;)*) => {$(
        #[derive(Debug, Clone, Copy, Default, PartialEq)]
        pub struct $name;
        impl Ease for $name {
            fn ease(&self, $t: f64) -> f64 { $body }
        }
    )*};
}

/// `(2^(-10x) - 2^-10) * 1.0009775171065494`: two to the power minus ten times `x`, scaled to `[0, 1]`.
fn tpmt(x: f64) -> f64 {
    (2f64.powf(-10.0 * x) - 0.0009765625) * 1.0009775171065494
}

const B1: f64 = 4.0 / 11.0;
const B2: f64 = 6.0 / 11.0;
const B3: f64 = 8.0 / 11.0;
const B4: f64 = 3.0 / 4.0;
const B5: f64 = 9.0 / 11.0;
const B6: f64 = 10.0 / 11.0;
const B7: f64 = 15.0 / 16.0;
const B8: f64 = 21.0 / 22.0;
const B9: f64 = 63.0 / 64.0;
const B0: f64 = 1.0 / B1 / B1;

fn bounce_out(t: f64) -> f64 {
    if t < B1 {
        B0 * t * t
    } else if t < B3 {
        B0 * (t - B2) * (t - B2) + B4
    } else if t < B6 {
        B0 * (t - B5) * (t - B5) + B7
    } else {
        B0 * (t - B8) * (t - B8) + B9
    }
}

simple! {
    Linear => |t| t;
    QuadIn => |t| t * t;
    QuadOut => |t| t * (2.0 - t);
    QuadInOut => |t| {
        let t = t * 2.0;
        (if t <= 1.0 { t * t } else { (t - 1.0) * (2.0 - (t - 1.0)) + 1.0 }) / 2.0
    };
    CubicIn => |t| t * t * t;
    CubicOut => |t| { let t = t - 1.0; t * t * t + 1.0 };
    CubicInOut => |t| {
        let t = t * 2.0;
        (if t <= 1.0 { t * t * t } else { let t = t - 2.0; t * t * t + 2.0 }) / 2.0
    };
    SinIn => |t| if t == 1.0 { 1.0 } else { 1.0 - (t * FRAC_PI_2).cos() };
    SinOut => |t| (t * FRAC_PI_2).sin();
    SinInOut => |t| (1.0 - (PI * t).cos()) / 2.0;
    ExpIn => |t| tpmt(1.0 - t);
    ExpOut => |t| 1.0 - tpmt(t);
    ExpInOut => |t| {
        let t = t * 2.0;
        (if t <= 1.0 { tpmt(1.0 - t) } else { 2.0 - tpmt(t - 1.0) }) / 2.0
    };
    CircleIn => |t| 1.0 - (1.0 - t * t).sqrt();
    CircleOut => |t| { let t = t - 1.0; (1.0 - t * t).sqrt() };
    CircleInOut => |t| {
        let t = t * 2.0;
        (if t <= 1.0 { 1.0 - (1.0 - t * t).sqrt() } else { let t = t - 2.0; (1.0 - t * t).sqrt() + 1.0 }) / 2.0
    };
    BounceIn => |t| 1.0 - bounce_out(1.0 - t);
    BounceOut => |t| bounce_out(t);
    BounceInOut => |t| {
        let t = t * 2.0;
        (if t <= 1.0 { 1.0 - bounce_out(1.0 - t) } else { bounce_out(t - 1.0) + 1.0 }) / 2.0
    };
}

macro_rules! parametric {
    ($($name:ident { $field:ident = $default:expr } => |$s:ident, $t:ident| $body:expr;)*) => {$(
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $name { $field: f64 }
        impl Default for $name {
            fn default() -> Self { Self { $field: $default } }
        }
        impl $name {
            pub fn $field(self, $field: f64) -> Self { Self { $field } }
        }
        impl Ease for $name {
            fn ease(&self, $t: f64) -> f64 { let $s = self.$field; $body }
        }
    )*};
}

parametric! {
    PolyIn { exponent = 3.0 } => |e, t| t.powf(e);
    PolyOut { exponent = 3.0 } => |e, t| 1.0 - (1.0 - t).powf(e);
    PolyInOut { exponent = 3.0 } => |e, t| {
        let t = t * 2.0;
        (if t <= 1.0 { t.powf(e) } else { 2.0 - (2.0 - t).powf(e) }) / 2.0
    };
    BackIn { overshoot = 1.70158 } => |s, t| t * t * (s * (t - 1.0) + t);
    BackOut { overshoot = 1.70158 } => |s, t| { let t = t - 1.0; t * t * ((t + 1.0) * s + t) + 1.0 };
    BackInOut { overshoot = 1.70158 } => |s, t| {
        let t = t * 2.0;
        (if t < 1.0 { t * t * ((s + 1.0) * t - s) } else { let t = t - 2.0; t * t * ((s + 1.0) * t + s) + 2.0 }) / 2.0
    };
}

/// Shared parameters of the elastic easings: amplitude is clamped to at least 1.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Elastic {
    a: f64,
    p: f64,
    s: f64,
}

impl Elastic {
    fn new(amplitude: f64, period: f64) -> Self {
        let a = amplitude.max(1.0);
        let p = period / TAU;
        Self { a, p, s: (1.0 / a).asin() * p }
    }
}

macro_rules! elastic {
    ($($name:ident => |$e:ident, $t:ident| $body:expr;)*) => {$(
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $name { amplitude: f64, period: f64 }
        impl Default for $name {
            fn default() -> Self { Self { amplitude: 1.0, period: 0.3 } }
        }
        impl $name {
            pub fn amplitude(self, amplitude: f64) -> Self { Self { amplitude: amplitude.max(1.0), ..self } }
            pub fn period(self, period: f64) -> Self { Self { period, ..self } }
        }
        impl Ease for $name {
            fn ease(&self, $t: f64) -> f64 { let $e = Elastic::new(self.amplitude, self.period); $body }
        }
    )*};
}

elastic! {
    ElasticIn => |e, t| { let t = t - 1.0; e.a * tpmt(-t) * ((e.s - t) / e.p).sin() };
    ElasticOut => |e, t| 1.0 - e.a * tpmt(t) * ((t + e.s) / e.p).sin();
    ElasticInOut => |e, t| {
        let t = t * 2.0 - 1.0;
        (if t < 0.0 { e.a * tpmt(-t) * ((e.s - t) / e.p).sin() } else { 2.0 - e.a * tpmt(t) * ((e.s + t) / e.p).sin() }) / 2.0
    };
}
