//! d3-ease: easing curves over normalized time t ∈ [0, 1].
//!
//! ```
//! use rustd3_core::ease::{Ease, Mode};
//!
//! assert_eq!(Ease::default().apply(0.25), 0.0625);
//! assert_eq!(Ease::Cubic(Mode::In).apply(0.25), 0.015625);
//! let custom = Ease::Poly { mode: Mode::In, exponent: 2.0 };
//! assert_eq!(custom.apply(0.5), 0.25);
//! ```

use std::f64::consts::{FRAC_PI_2, PI, TAU};

/// d3-ease's curves as one closed set.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ease {
    /// `easeLinear`: t.
    Linear,
    /// `easeQuad*`: t².
    Quad(Mode),
    /// `easeCubic*`: t³.
    Cubic(Mode),
    /// `easeSin*`: a quarter sine wave.
    Sin(Mode),
    /// `easeExp*`: 2^(10t − 10), scaled to reach 0 and 1.
    Exp(Mode),
    /// `easeCircle*`: a quarter circle.
    Circle(Mode),
    /// `easeBounce*`: a bouncing ball.
    Bounce(Mode),
    /// `easePoly*`: t^exponent (d3's default 3).
    Poly {
        /// Which end the curve eases.
        mode: Mode,
        /// The power.
        exponent: f64,
    },
    /// `easeBack*`: an anticipatory dip (d3's default overshoot 1.70158).
    Back {
        /// Which end the curve eases.
        mode: Mode,
        /// How far it dips.
        overshoot: f64,
    },
    /// `easeElastic*`: a rubber band (d3's defaults: amplitude 1, period 0.3).
    Elastic {
        /// Which end the curve eases.
        mode: Mode,
        /// The overshoot; below 1 it acts as 1.
        amplitude: f64,
        /// The oscillation's period.
        period: f64,
    },
}

/// Which end of the interval a curve eases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Slow at the start (d3's `*In`).
    In,
    /// Slow at the end (d3's `*Out`).
    Out,
    /// Slow at both ends (d3's `*InOut`).
    InOut,
}

/// The bounce's breakpoints and offsets (d3's b1 … b9) and its scale b0.
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

impl Default for Ease {
    /// `Cubic(InOut)`, d3's transition default.
    fn default() -> Self {
        Ease::Cubic(Mode::InOut)
    }
}

impl Ease {
    /// The eased value at `t`, by d3's formulas.
    #[must_use]
    pub fn apply(&self, t: f64) -> f64 {
        match *self {
            Ease::Linear => t,
            Ease::Quad(mode) => quad(mode, t),
            Ease::Cubic(mode) => cubic(mode, t),
            Ease::Sin(mode) => sin(mode, t),
            Ease::Exp(mode) => exp(mode, t),
            Ease::Circle(mode) => circle(mode, t),
            Ease::Bounce(mode) => bounce(mode, t),
            Ease::Poly { mode, exponent } => poly(mode, exponent, t),
            Ease::Back { mode, overshoot } => back(mode, overshoot, t),
            Ease::Elastic {
                mode,
                amplitude,
                period,
            } => elastic(mode, amplitude, period, t),
        }
    }

    /// `Poly` with d3's default exponent, 3.
    #[must_use]
    pub const fn poly(mode: Mode) -> Ease {
        Ease::Poly {
            mode,
            exponent: 3.0,
        }
    }

    /// `Back` with d3's default overshoot, 1.70158.
    #[must_use]
    pub const fn back(mode: Mode) -> Ease {
        Ease::Back {
            mode,
            overshoot: 1.70158,
        }
    }

    /// `Elastic` with d3's default amplitude 1 and period 0.3.
    #[must_use]
    pub const fn elastic(mode: Mode) -> Ease {
        Ease::Elastic {
            mode,
            amplitude: 1.0,
            period: 0.3,
        }
    }
}

fn quad(mode: Mode, t: f64) -> f64 {
    match mode {
        Mode::In => t * t,
        Mode::Out => t * (2.0 - t),
        Mode::InOut => {
            let t = t * 2.0;
            (if t <= 1.0 {
                t * t
            } else {
                let t = t - 1.0;
                t * (2.0 - t) + 1.0
            }) / 2.0
        }
    }
}

fn cubic(mode: Mode, t: f64) -> f64 {
    match mode {
        Mode::In => t * t * t,
        Mode::Out => {
            let t = t - 1.0;
            t * t * t + 1.0
        }
        Mode::InOut => {
            let t = t * 2.0;
            (if t <= 1.0 {
                t * t * t
            } else {
                let t = t - 2.0;
                t * t * t + 2.0
            }) / 2.0
        }
    }
}

fn sin(mode: Mode, t: f64) -> f64 {
    match mode {
        Mode::In if t == 1.0 => 1.0,
        Mode::In => 1.0 - (t * FRAC_PI_2).cos(),
        Mode::Out => (t * FRAC_PI_2).sin(),
        Mode::InOut => (1.0 - (PI * t).cos()) / 2.0,
    }
}

fn exp(mode: Mode, t: f64) -> f64 {
    match mode {
        Mode::In => tpmt(1.0 - t),
        Mode::Out => 1.0 - tpmt(t),
        Mode::InOut => {
            let t = t * 2.0;
            (if t <= 1.0 {
                tpmt(1.0 - t)
            } else {
                2.0 - tpmt(t - 1.0)
            }) / 2.0
        }
    }
}

fn circle(mode: Mode, t: f64) -> f64 {
    match mode {
        Mode::In => 1.0 - (1.0 - t * t).sqrt(),
        Mode::Out => {
            let t = t - 1.0;
            (1.0 - t * t).sqrt()
        }
        Mode::InOut => {
            let t = t * 2.0;
            (if t <= 1.0 {
                1.0 - (1.0 - t * t).sqrt()
            } else {
                let t = t - 2.0;
                (1.0 - t * t).sqrt() + 1.0
            }) / 2.0
        }
    }
}

fn bounce(mode: Mode, t: f64) -> f64 {
    match mode {
        Mode::In => 1.0 - bounce_out(1.0 - t),
        Mode::Out => bounce_out(t),
        Mode::InOut => {
            let t = t * 2.0;
            (if t <= 1.0 {
                1.0 - bounce_out(1.0 - t)
            } else {
                bounce_out(t - 1.0) + 1.0
            }) / 2.0
        }
    }
}

fn bounce_out(t: f64) -> f64 {
    if t < B1 {
        B0 * t * t
    } else if t < B3 {
        let t = t - B2;
        B0 * t * t + B4
    } else if t < B6 {
        let t = t - B5;
        B0 * t * t + B7
    } else {
        let t = t - B8;
        B0 * t * t + B9
    }
}

fn poly(mode: Mode, exponent: f64, t: f64) -> f64 {
    match mode {
        Mode::In => t.powf(exponent),
        Mode::Out => 1.0 - (1.0 - t).powf(exponent),
        Mode::InOut => {
            let t = t * 2.0;
            (if t <= 1.0 {
                t.powf(exponent)
            } else {
                2.0 - (2.0 - t).powf(exponent)
            }) / 2.0
        }
    }
}

fn back(mode: Mode, s: f64, t: f64) -> f64 {
    match mode {
        Mode::In => t * t * (s * (t - 1.0) + t),
        Mode::Out => {
            let t = t - 1.0;
            t * t * ((t + 1.0) * s + t) + 1.0
        }
        Mode::InOut => {
            let t = t * 2.0;
            (if t < 1.0 {
                t * t * ((s + 1.0) * t - s)
            } else {
                let t = t - 2.0;
                t * t * ((s + 1.0) * t + s) + 2.0
            }) / 2.0
        }
    }
}

fn elastic(mode: Mode, amplitude: f64, period: f64, t: f64) -> f64 {
    // d3 clamps with Math.max(1, a), which keeps NaN.
    let a = if amplitude < 1.0 { 1.0 } else { amplitude };
    let p = period / TAU;
    let s = (1.0 / a).asin() * p;
    match mode {
        Mode::In => {
            let t = t - 1.0;
            a * tpmt(-t) * ((s - t) / p).sin()
        }
        Mode::Out => 1.0 - a * tpmt(t) * ((t + s) / p).sin(),
        Mode::InOut => {
            let t = t * 2.0 - 1.0;
            (if t < 0.0 {
                a * tpmt(-t) * ((s - t) / p).sin()
            } else {
                2.0 - a * tpmt(t) * ((s + t) / p).sin()
            }) / 2.0
        }
    }
}

/// d3's tpmt: two to the power minus ten times `x`, scaled to [0, 1].
fn tpmt(x: f64) -> f64 {
    (2f64.powf(-10.0 * x) - 0.000_976_562_5) * 1.000_977_517_106_549_4
}
