//! What the generated tests call: [`check`], [`skip`], [`throws`], the
//! assertion [`Kind`]s, rustd3-core's modules and d3's test helpers. Each
//! call is recorded for the run to fold.

use std::fmt;

use rustd3_testkit::obs::{Obs, ToObs, Token, Tolerance, tokenize};

use crate::record::{self, Mark, Record};

pub(crate) use rustd3_core::color::{Hsl, Rgb};
pub(crate) use rustd3_core::ease::{Ease, Mode};
pub(crate) use rustd3_core::interpolate::Interpolate;
pub(crate) use rustd3_core::path::{PathContext, SvgPath};
pub(crate) use rustd3_core::shape::{self, Curve};
pub(crate) use rustd3_core::{format, interact, interpolate, scale, ticks};

/// The d3 assertion an assertion was written with, which sets its d3 verdict.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// `strictEqual`, `deepStrictEqual`, `equal`: exact in d3 (`Object.is`).
    Strict,
    /// `assertInDelta`: within an absolute tolerance.
    InDelta(f64),
    /// `assertPathEqual`: equal after d3-shape's `normalizePath` (integers
    /// within 1e-6, everything else to 3 decimals).
    PathEq,
    /// `assert.throws`: the call must return `Err`.
    Throws,
}

/// Records one assertion under d3's semantics and under the tolerant
/// format (`Tolerance::default()`, or `at_least(d)` for `InDelta(d)`).
/// Under `Throws`, both pass iff `actual` is the [`Obs`] of an `Err`.
pub fn check<A: ToObs + ?Sized, B: ToObs + ?Sized>(
    id: &'static str,
    kind: Kind,
    actual: &A,
    expected: &B,
) {
    let (actual, expected) = (actual.to_obs(), expected.to_obs());
    let (d3, tolerant) = match kind {
        Kind::Strict => (
            actual.strict_eq(&expected),
            actual.approx_eq(&expected, &Tolerance::default()),
        ),
        Kind::InDelta(delta) => (
            in_delta(&actual, &expected, delta),
            actual.approx_eq(&expected, &Tolerance::at_least(delta)),
        ),
        Kind::PathEq => (
            d3_normalize(&actual) == d3_normalize(&expected),
            actual.approx_eq(&expected, &Tolerance::default()),
        ),
        Kind::Throws => {
            let thrown = if let Obs::Str(s) = &actual {
                s.starts_with("error: ")
            } else {
                false
            };
            (thrown, thrown)
        }
    };
    record::push(Record {
        id,
        mark: Mark::Checked {
            kind,
            d3,
            tolerant,
            actual,
            expected,
        },
    });
}

/// Marks an assertion the translator skipped. `reason` repeats its
/// `ASSERTS` entry, which the run verifies.
pub fn skip(id: &'static str, reason: &'static str) {
    record::push(Record {
        id,
        mark: Mark::Skipped(reason),
    });
}

/// `assert.throws`: records a `Kind::Throws` record that passes under both
/// verdicts iff `f()` returns `Err`; an `Ok(v)` fails and reports `v`.
pub fn throws<T: ToObs, E: fmt::Debug>(id: &'static str, f: impl FnOnce() -> Result<T, E>) {
    let (thrown, actual) = match f() {
        Ok(value) => (false, value.to_obs()),
        Err(error) => (true, Obs::Str(format!("error: {error:?}"))),
    };
    let expected = Obs::Str("an error".to_owned());
    let mark = Mark::Checked {
        kind: Kind::Throws,
        d3: thrown,
        tolerant: thrown,
        actual,
        expected,
    };
    record::push(Record { id, mark });
}

/// d3-path's test helper `precision(str, p)`: every decimal `\d+\.\d+` in
/// `s` rounded to `p` places, then printed as the shortest number
/// (`+parseFloat(s).toFixed(p)`).
pub fn precision(s: &str, p: f64) -> String {
    // The tests pass small whole numbers of places.
    let places = p as usize;
    let mut out = String::new();
    let mut rest = s;
    while let Some(start) = rest.find(|c: char| c.is_ascii_digit()) {
        out.push_str(&rest[..start]);
        let digits = &rest[start..];
        let int = digits
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(digits.len());
        let frac = digits[int..].strip_prefix('.').map_or(0, |f| {
            f.find(|c: char| !c.is_ascii_digit()).unwrap_or(f.len())
        });
        if frac == 0 {
            out.push_str(&digits[..int]);
            rest = &digits[int..];
            continue;
        }
        let end = int + 1 + frac;
        let value: f64 = digits[..end].parse().expect("digits, a point and digits");
        let fixed: f64 = format!("{value:.places$}")
            .parse()
            .expect("a fixed-point number");
        out.push_str(&fixed.to_string());
        rest = &digits[end..];
    }
    out.push_str(rest);
    out
}

/// JavaScript's `Math.round`: ties toward +∞ (`f64::round` takes them away
/// from zero).
pub fn js_round(x: f64) -> f64 {
    let floor = x.floor();
    if x - floor >= 0.5 { floor + 1.0 } else { floor }
}

/// d3-scale's test helper `roundEpsilon(x)`: `x` rounded to 12 decimals.
pub fn round_epsilon(x: f64) -> f64 {
    js_round(x * 1e12) / 1e12
}

/// What d3-shape's generators return: the path `draw` writes into a fresh
/// [`SvgPath`], or `None` (d3's `null`) when it is empty.
pub fn drawn(draw: impl FnOnce(&mut SvgPath)) -> Option<String> {
    let mut path = SvgPath::new();
    draw(&mut path);
    let data = path.to_string();
    (!data.is_empty()).then_some(data)
}

/// d3.interpolateRgb(a, b) at gamma 1, a closure of `t`.
pub fn interpolate_rgb(a: Rgb, b: Rgb) -> impl Fn(f64) -> Rgb {
    move |t| a.lerp(&b, t)
}

/// d3-color's test helper `assertRgbEqual(c, r, g, b, opacity)`: each
/// channel equal, NaN matching NaN.
pub fn rgb_equal(c: &Rgb, r: f64, g: f64, b: f64, opacity: f64) -> bool {
    let same = |a: f64, e: f64| if e.is_nan() { a.is_nan() } else { a == e };
    same(c.r, r) && same(c.g, g) && same(c.b, b) && same(c.opacity, opacity)
}

/// d3-color's test helper `assertRgbApproxEqual(c, r, g, b, opacity)`:
/// channels equal after `Math.round`, the opacity exactly, NaN matching
/// NaN.
pub fn rgb_approx_equal(c: &Rgb, r: f64, g: f64, b: f64, opacity: f64) -> bool {
    let near = |a: f64, e: f64| {
        if e.is_nan() {
            a.is_nan()
        } else {
            js_round(a) == js_round(e)
        }
    };
    let same = |a: f64, e: f64| if e.is_nan() { a.is_nan() } else { a == e };
    near(c.r, r) && near(c.g, g) && near(c.b, b) && same(c.opacity, opacity)
}

/// d3-color's test helper `assertHslEqual(c, h, s, l, opacity)`: each
/// component within 1e-6, the opacity exactly, NaN matching NaN.
pub fn hsl_equal(c: &Hsl, h: f64, s: f64, l: f64, opacity: f64) -> bool {
    let near = |a: f64, e: f64| {
        if e.is_nan() {
            a.is_nan()
        } else {
            e - 1e-6 <= a && a <= e + 1e-6
        }
    };
    let same = |a: f64, e: f64| if e.is_nan() { a.is_nan() } else { a == e };
    near(c.h, h) && near(c.s, s) && near(c.l, l) && same(c.opacity, opacity)
}

/// d3-ease's test helper `out(easeIn)`: the easing reflected in time and value.
pub fn out(ease: Ease) -> impl Fn(f64) -> f64 {
    move |t| 1.0 - ease.apply(1.0 - t)
}

/// d3-ease's test helper `inOut(easeIn)`: the easing, then its reflection.
pub fn in_out(ease: Ease) -> impl Fn(f64) -> f64 {
    move |t| {
        if t < 0.5 {
            ease.apply(t * 2.0) / 2.0
        } else {
            (2.0 - ease.apply((1.0 - t) * 2.0)) / 2.0
        }
    }
}

/// d3's `assertInDelta`: numbers within `delta`, arrays element by element.
fn in_delta(actual: &Obs, expected: &Obs, delta: f64) -> bool {
    match (actual, expected) {
        (Obs::Num(a), Obs::Num(e)) => *a >= e - delta && *a <= e + delta,
        (Obs::List(a), Obs::List(e)) => {
            a.len() == e.len() && a.iter().zip(e).all(|(p, q)| in_delta(p, q, delta))
        }
        (Obs::Null | Obs::Bool(_) | Obs::Num(_) | Obs::Str(_) | Obs::List(_), _) => {
            actual == expected
        }
    }
}

/// d3-shape's `normalizePath`: numbers within 1e-6 of an integer print as
/// that integer, every other number with 3 decimals.
fn d3_normalize(path: &Obs) -> Obs {
    let Obs::Str(s) = path else {
        return path.clone();
    };
    let mut out = String::new();
    for token in tokenize(s) {
        match token {
            Token::Text(text) => out.push_str(&text),
            Token::Num { value, .. } => {
                let integer = value.round();
                if (value - integer).abs() < 1e-6 {
                    let integer = if integer == 0.0 { 0.0 } else { integer };
                    out.push_str(&integer.to_string());
                } else {
                    out.push_str(&format!("{value:.3}"));
                }
            }
        }
    }
    Obs::Str(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{Mark, take};

    fn verdicts() -> Vec<(bool, bool)> {
        take()
            .into_iter()
            .map(|r| match r.mark {
                Mark::Checked { d3, tolerant, .. } => (d3, tolerant),
                Mark::Skipped(reason) => panic!("unexpected skip: {reason}"),
            })
            .collect()
    }

    #[test]
    fn strict_is_object_is_while_the_tolerant_verdict_allows_rounding() {
        check("a:1:1", Kind::Strict, &(0.1 + 0.2), &0.3);
        check("a:2:1", Kind::Strict, &f64::NAN, &f64::NAN);
        check("a:3:1", Kind::Strict, &-0.0, &0.0);
        check("a:4:1", Kind::Strict, "1e+21", "1000000000000000000000");
        assert_eq!(
            verdicts(),
            [(false, true), (true, true), (false, true), (false, true)]
        );
    }

    #[test]
    fn in_delta_keeps_d3s_delta_in_both_verdicts() {
        check("a:1:1", Kind::InDelta(1e-6), &0.1000005, &0.1);
        check("a:2:1", Kind::InDelta(1e-6), &0.100002, &0.1);
        check(
            "a:3:1",
            Kind::InDelta(0.5),
            &vec![1.0, 2.4],
            &vec![1.0, 2.0],
        );
        assert_eq!(verdicts(), [(true, true), (false, false), (true, true)]);
    }

    #[test]
    fn path_equality_normalizes_numbers_to_three_decimals() {
        check(
            "a:1:1",
            Kind::PathEq,
            "M0.0000001,1.00049L2",
            "M0,1.0004L2.0000000001",
        );
        check("a:2:1", Kind::PathEq, "M0,1.0006", "M0,1.0004");
        assert_eq!(verdicts(), [(true, false), (false, false)]);
    }

    #[test]
    fn throws_passes_only_on_err() {
        throws("a:1:1", || Err::<f64, &str>("negative radius"));
        throws("a:2:1", || Ok::<f64, &str>(2.0));
        check("a:3:1", Kind::Throws, &Err::<(), &str>("bad"), &());
        check("a:4:1", Kind::Throws, &Ok::<(), &str>(()), &());
        let records = take();
        let marks: Vec<_> = records
            .iter()
            .map(|r| match &r.mark {
                Mark::Checked {
                    kind,
                    d3,
                    tolerant,
                    actual,
                    ..
                } => (*kind, *d3, *tolerant, actual.clone()),
                Mark::Skipped(reason) => panic!("unexpected skip: {reason}"),
            })
            .collect();
        assert_eq!(
            marks,
            [
                (
                    Kind::Throws,
                    true,
                    true,
                    Obs::Str("error: \"negative radius\"".to_owned())
                ),
                (Kind::Throws, false, false, Obs::Num(2.0)),
                (Kind::Throws, true, true, Obs::Str("error: bad".to_owned())),
                (Kind::Throws, false, false, Obs::Null),
            ]
        );
    }

    #[test]
    fn skip_records_its_reason() {
        skip("a:1:1", "js: coercion");
        let records = take();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, "a:1:1");
        assert!(matches!(records[0].mark, Mark::Skipped("js: coercion")));
    }

    #[test]
    fn precision_rounds_only_decimals_and_prints_them_shortest() {
        assert_eq!(
            precision("M10.0001,10A123.456,1e-7,0,0,1,3.06e-15", 1.0),
            "M10,10A123.5,1e-7,0,0,1,3.1e-15"
        );
    }

    #[test]
    fn out_and_in_out_reflect_an_easing() {
        let quad = Ease::Quad(Mode::In);
        assert_eq!(out(quad)(0.25), Ease::Quad(Mode::Out).apply(0.25));
        assert_eq!(in_out(quad)(0.25), 0.125);
        assert_eq!(in_out(quad)(0.75), 0.875);
    }

    #[test]
    fn color_helpers_follow_d3_colors_asserts() {
        let c = Rgb::new(10.4, f64::NAN, 3.0).with_opacity(0.5);
        assert!(rgb_approx_equal(&c, 10.0, f64::NAN, 3.0, 0.5));
        assert!(!rgb_approx_equal(&c, 11.0, f64::NAN, 3.0, 0.5));
        assert!(!rgb_approx_equal(&c, 10.0, 0.0, 3.0, 0.5));
        assert!(!rgb_equal(&c, 10.0, f64::NAN, 3.0, 0.5));
        assert!(rgb_equal(&c, 10.4, f64::NAN, 3.0, 0.5));
        let h = Hsl {
            h: 120.0000005,
            s: f64::NAN,
            l: 0.5,
            opacity: 1.0,
        };
        assert!(hsl_equal(&h, 120.0, f64::NAN, 0.5, 1.0));
        assert!(!hsl_equal(&h, 120.0, f64::NAN, 0.5, 0.9));
        assert_eq!(
            interpolate_rgb(Rgb::new(0.0, 0.0, 0.0), Rgb::new(255.0, 0.0, 0.0))(0.5),
            Rgb::new(127.5, 0.0, 0.0)
        );
    }

    #[test]
    fn drawn_is_none_for_an_empty_path_as_d3_shape_returns_null() {
        assert_eq!(drawn(|_| {}), None);
        assert_eq!(drawn(|p| p.move_to(0.0, 1.0)), Some("M0,1".to_owned()));
    }

    #[test]
    fn d3_normalize_rounds_like_d3_shape() {
        let normalized = d3_normalize(&Obs::Str("M-0.0000001,2.00049Z".to_owned()));
        assert_eq!(normalized, Obs::Str("M0,2.000Z".to_owned()));
        assert_eq!(d3_normalize(&Obs::Num(1.5)), Obs::Num(1.5));
    }
}
