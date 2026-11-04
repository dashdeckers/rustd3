//! What d3's converted tests call beyond rustd3-core: the comparisons
//! d3's assertions make where `assert_eq!` does not express them, and
//! d3's test helpers.

use rustd3_core::color::{Hcl, Hsl, Lab, Rgb};
use rustd3_core::ease::Ease;
use rustd3_core::jiff::civil::{Date, Time};
use rustd3_core::jiff::tz::TimeZone;
use rustd3_core::jiff::{SignedDuration, Timestamp};
use rustd3_core::path::{PathContext, SvgPath};
use rustd3_core::shape;
use rustd3_testkit::obs::{Obs, ToObs, Token, Tolerance, tokenize};
pub use rustd3_testkit::zone;

/// d3's `assert.strictEqual` and `deepStrictEqual` where the two sides have
/// different Rust types (an `Option` against a value, a tuple against an
/// array, `()` against `undefined`) or compare otherwise than by `==`:
/// both as [`Obs`] values, equal under `Object.is` (NaN equals NaN) or
/// within the tolerant comparison of numbers and printed numbers.
#[track_caller]
pub fn assert_d3_eq<A: ToObs + ?Sized, B: ToObs + ?Sized>(actual: &A, expected: &B) {
    let (actual, expected) = (actual.to_obs(), expected.to_obs());
    assert!(
        actual.strict_eq(&expected) || actual.approx_eq(&expected, &Tolerance::default()),
        "assertion `actual == expected` failed\n  actual: {actual:?}\nexpected: {expected:?}"
    );
}

/// d3's `assertInDelta`: numbers within `delta`, arrays element by element.
#[track_caller]
pub fn assert_in_delta<A: ToObs + ?Sized, B: ToObs + ?Sized>(actual: &A, expected: &B, delta: f64) {
    let (actual, expected) = (actual.to_obs(), expected.to_obs());
    assert!(
        in_delta(&actual, &expected, delta),
        "assertion `actual within {delta} of expected` failed\n  actual: {actual:?}\nexpected: {expected:?}"
    );
}

/// d3-shape's `assertPathEqual`: equal after `normalizePath`, which prints
/// numbers within 1e-6 of an integer as that integer and every other with
/// 3 decimals.
#[track_caller]
pub fn assert_path_eq<A: ToObs + ?Sized, B: ToObs + ?Sized>(actual: &A, expected: &B) {
    let (actual, expected) = (actual.to_obs(), expected.to_obs());
    let (a, e) = (normalize_path(&actual), normalize_path(&expected));
    assert!(
        a == e,
        "assertion `actual == expected` failed after normalizing\n  actual: {actual:?}\nexpected: {expected:?}"
    );
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
/// from zero), and −0 for x in [−0.5, −0], which `assert.deepStrictEqual`
/// tells from +0.
pub fn js_round(x: f64) -> f64 {
    let floor = x.floor();
    let rounded = if x - floor >= 0.5 { floor + 1.0 } else { floor };
    if rounded == 0.0 {
        0.0_f64.copysign(x)
    } else {
        rounded
    }
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

/// d3-shape's arcs as `deepStrictEqual` compares them: per slice, its datum
/// (d3's `data`, as its position in the input), index, value and angles.
pub fn slices(slices: &[shape::Slice]) -> Obs {
    let fields = |s: &shape::Slice| {
        let angles = [s.value, s.start_angle, s.end_angle, s.pad_angle];
        [s.datum.to_obs(), s.index.to_obs()]
            .into_iter()
            .chain(angles.iter().map(ToObs::to_obs))
            .collect()
    };
    Obs::List(slices.iter().map(|s| Obs::List(fields(s))).collect())
}

/// d3-shape's test double `polygonContext()`: the points of the last
/// subpath, whose `area()` is the absolute `d3.polygonArea` of them. Curves
/// and arcs, which d3's double does not have, add their end point.
#[derive(Debug, Default)]
pub struct Polygon {
    points: Vec<(f64, f64)>,
}

impl Polygon {
    /// The polygon's area, whichever way it winds.
    pub fn area(&self) -> f64 {
        let Some(&last) = self.points.last() else {
            return 0.0;
        };
        let mut b = last;
        let mut twice = 0.0;
        for &point in &self.points {
            let a = b;
            b = point;
            twice += a.1 * b.0 - a.0 * b.1;
        }
        (twice / 2.0).abs()
    }
}

impl PathContext for Polygon {
    fn move_to(&mut self, x: f64, y: f64) {
        self.points = vec![(x, y)];
    }

    fn line_to(&mut self, x: f64, y: f64) {
        self.points.push((x, y));
    }

    fn quadratic_curve_to(&mut self, _x1: f64, _y1: f64, x: f64, y: f64) {
        self.points.push((x, y));
    }

    fn bezier_curve_to(&mut self, _x1: f64, _y1: f64, _x2: f64, _y2: f64, x: f64, y: f64) {
        self.points.push((x, y));
    }

    fn arc(&mut self, x: f64, y: f64, r: f64, _start: f64, end: f64, _anticlockwise: bool) {
        self.points.push((x + r * end.cos(), y + r * end.sin()));
    }

    fn close_path(&mut self) {}

    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.points = vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h)];
    }
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
    near_channels([c.h, c.s, c.l, c.opacity], [h, s, l, opacity])
}

/// d3-color's test helper `assertLabEqual(c, l, a, b, opacity)`: as
/// [`hsl_equal`].
pub fn lab_equal(c: &Lab, l: f64, a: f64, b: f64, opacity: f64) -> bool {
    near_channels([c.l, c.a, c.b, c.opacity], [l, a, b, opacity])
}

/// d3-color's test helper `assertHclEqual(c, h, c, l, opacity)`: as
/// [`hsl_equal`].
pub fn hcl_equal(c: &Hcl, h: f64, chroma: f64, l: f64, opacity: f64) -> bool {
    near_channels([c.h, c.c, c.l, c.opacity], [h, chroma, l, opacity])
}

/// d3-color's asserts: three channels within 1e-6, the opacity exactly,
/// NaN matching NaN.
fn near_channels(actual: [f64; 4], expected: [f64; 4]) -> bool {
    let near = |a: f64, e: f64| {
        if e.is_nan() {
            a.is_nan()
        } else {
            e - 1e-6 <= a && a <= e + 1e-6
        }
    };
    let same = |a: f64, e: f64| if e.is_nan() { a.is_nan() } else { a == e };
    let [c0, c1, c2, opacity] = actual;
    let [e0, e1, e2, expected_opacity] = expected;
    near(c0, e0) && near(c1, e1) && near(c2, e2) && same(opacity, expected_opacity)
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

/// d3-time's test helper `local(year, month, day, hours, minutes, seconds,
/// milliseconds)` in `zone`: ECMAScript's MakeDay, MakeTime and MakeDate
/// over 1 to 7 fields (the month from 0; a missing day is 1, other missing
/// fields 0; any field may carry into the next, negative ones back; years
/// 0–99 literal), resolved as `Date` resolves a local time, by jiff's
/// compatible disambiguation.
pub fn local(zone: &TimeZone, fields: &[f64]) -> Timestamp {
    zone.to_timestamp(make_date(fields))
        .expect("d3 test: a date in jiff's range")
}

/// d3-time's test helper `utc(…)`: [`local`] in UTC (`Date.UTC`).
pub fn utc(fields: &[f64]) -> Timestamp {
    local(&TimeZone::UTC, fields)
}

/// `new Date(ms)`: the instant `ms` milliseconds from the epoch.
pub fn millis(ms: f64) -> Timestamp {
    // Tests pass whole numbers, which `as` keeps exactly below 2^53.
    Timestamp::from_millisecond(ms as i64).expect("d3 test: a date in jiff's range")
}

/// A date as the tests compare it: RFC 3339 with nine fraction digits,
/// exact to the nanosecond, or null for d3's Invalid Date.
pub fn ts(t: impl Into<Option<Timestamp>>) -> Obs {
    t.into().map_or(Obs::Null, |t| Obs::Str(format!("{t:.9}")))
}

/// Dates as [`ts`] compares each.
pub fn tss(dates: &[Timestamp]) -> Obs {
    Obs::List(dates.iter().map(|&t| ts(t)).collect())
}

/// The civil date-time `local` and `utc` resolve: the date from year and
/// month (carried by whole years) plus `day − 1` days, then the time fields
/// as one millisecond offset, all in civil arithmetic.
fn make_date(fields: &[f64]) -> rustd3_core::jiff::civil::DateTime {
    // Tests pass whole numbers; ECMAScript truncates fields as `as` does.
    let field = |i: usize, missing: i64| fields.get(i).map_or(missing, |&f| f as i64);
    let (year, month) = (field(0, 0), field(1, 0));
    let first = Date::new(
        i16::try_from(year + month.div_euclid(12)).expect("d3 test: a year jiff holds"),
        i8::try_from(month.rem_euclid(12) + 1).expect("invariant: a month from 1 to 12"),
        1,
    )
    .expect("d3 test: a year jiff holds");
    let ms = field(3, 0) * 3_600_000 + field(4, 0) * 60_000 + field(5, 0) * 1_000 + field(6, 0);
    let offset =
        SignedDuration::from_hours(24 * (field(2, 1) - 1)) + SignedDuration::from_millis(ms);
    first
        .to_datetime(Time::midnight())
        .checked_add(offset)
        .expect("d3 test: a date in jiff's range")
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
fn normalize_path(path: &Obs) -> Obs {
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
