//! d3-interpolate's channel rules (color.js): how one channel of a color
//! blends, crate-private.

use crate::js;

/// One channel of a color blend, as d3's color.js builds it: a constant,
/// a line, or d3's gamma curve.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Channel {
    /// d3's `constant(v)`: `v` at every `t`.
    Constant(f64),
    /// d3's `linear(a, d)`: `a + t·d`.
    Linear { a: f64, d: f64 },
    /// d3's `exponential(a, b, y)`: `(a^y + t·(b^y − a^y))^(1/y)`, kept as
    /// `a^y`, `b^y − a^y` and `1/y`.
    Exponential { a: f64, d: f64, inverse: f64 },
}

impl Channel {
    /// d3's `nogamma(a, b)`: linear in `b − a`. When that is 0 or NaN, it is
    /// the constant `a` (`b` when `a` is NaN), so an undefined channel takes
    /// the other side's value.
    pub(crate) fn nogamma(a: f64, b: f64) -> Channel {
        let d = b - a;
        if d != 0.0 && !d.is_nan() {
            Channel::Linear { a, d }
        } else {
            Channel::constant(a, b)
        }
    }

    /// d3's `gamma(y)(a, b)`: [`nogamma`](Channel::nogamma) at `y = 1`,
    /// otherwise d3's exponential curve, with the same constants.
    pub(crate) fn gamma(y: f64, a: f64, b: f64) -> Channel {
        if y == 1.0 {
            return Channel::nogamma(a, b);
        }
        let d = b - a;
        if d != 0.0 && !d.is_nan() {
            let a = a.powf(y);
            Channel::Exponential {
                a,
                d: b.powf(y) - a,
                inverse: 1.0 / y,
            }
        } else {
            Channel::constant(a, b)
        }
    }

    /// d3's `hue(a, b)`: linear along the shorter arc, with a difference `d`
    /// beyond ±180° taken as `d − 360·round(d / 360)`. It is constant, as in
    /// [`nogamma`](Channel::nogamma), when the difference is 0 or NaN. The
    /// result is not wrapped into [0, 360).
    pub(crate) fn hue(a: f64, b: f64) -> Channel {
        let d = b - a;
        if d != 0.0 && !d.is_nan() {
            let d = if !(-180.0..=180.0).contains(&d) {
                d - 360.0 * js::round(d / 360.0)
            } else {
                d
            };
            Channel::Linear { a, d }
        } else {
            Channel::constant(a, b)
        }
    }

    /// The four channels of a blend from `a` to `b`: the first by `first`
    /// (the hue rule or `nogamma`), the others, opacity last, by `nogamma`.
    pub(crate) fn four(a: [f64; 4], b: [f64; 4], first: fn(f64, f64) -> Channel) -> [Channel; 4] {
        [
            first(a[0], b[0]),
            Channel::nogamma(a[1], b[1]),
            Channel::nogamma(a[2], b[2]),
            Channel::nogamma(a[3], b[3]),
        ]
    }

    /// The channel's value at `t`.
    pub(crate) fn at(self, t: f64) -> f64 {
        match self {
            Channel::Constant(v) => v,
            Channel::Linear { a, d } => a + t * d,
            Channel::Exponential { a, d, inverse } => (a + t * d).powf(inverse),
        }
    }

    /// d3's `constant(isNaN(a) ? b : a)`.
    fn constant(a: f64, b: f64) -> Channel {
        Channel::Constant(if a.is_nan() { b } else { a })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_undefined_side_takes_the_other_sides_value() {
        assert_eq!(Channel::nogamma(f64::NAN, 3.0).at(0.2), 3.0);
        assert_eq!(Channel::nogamma(3.0, f64::NAN).at(0.8), 3.0);
        assert!(Channel::nogamma(f64::NAN, f64::NAN).at(0.5).is_nan());
        assert_eq!(Channel::gamma(2.0, f64::NAN, 4.0).at(0.5), 4.0);
    }

    #[test]
    fn hues_take_the_shorter_arc_without_wrapping() {
        assert_eq!(Channel::hue(10.0, 350.0).at(0.5), 0.0);
        assert_eq!(Channel::hue(350.0, 10.0).at(0.5), 360.0);
        assert_eq!(Channel::hue(10.0, 380.0).at(1.0), 20.0);
        assert_eq!(Channel::hue(10.0, 190.0).at(0.5), 100.0);
    }

    #[test]
    fn gamma_curves_blend_powers_of_the_ends() {
        let c = Channel::gamma(2.0, 0.0, 10.0);
        assert_eq!(c.at(0.25), 5.0);
        assert_eq!(c.at(1.0), 10.0);
    }
}
