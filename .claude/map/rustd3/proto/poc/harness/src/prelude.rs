//! What the generated tests call: `check`, `skip`, and d3's own test helpers.

pub use d3_core::ease::{self, Ease};
pub use d3_core::path::Path;
pub use d3_core::ticks::{nice, tick_increment, tick_step, ticks};
#[allow(unused_imports)]
pub use d3_core::format::{Definition, Locale, Specifier, format, format_prefix, precision_fixed, precision_prefix, precision_round};
use obs::{Obs, ToObs, Tolerance, approx_eq, strict_eq, tokenize, Token};
pub use std::f64::consts::{E, PI};
use std::cell::RefCell;

/// The assertion helper the d3 test used.
#[derive(Debug, Clone, Copy)]
pub enum Kind {
    /// `assert.strictEqual` / `assert.deepStrictEqual`: exact in d3.
    Strict,
    /// `assertInDelta`: absolute delta in d3.
    InDelta(f64),
    /// `assertPathEqual`: numbers rounded to 3 decimals in d3.
    PathEq,
}

#[derive(Debug, Clone)]
pub struct Record {
    pub id: usize,
    pub kind: Kind,
    pub d3_pass: bool,
    pub tolerant_pass: bool,
    pub actual: Obs,
    pub expected: Obs,
}

thread_local! {
    pub static RECORDS: RefCell<Vec<Record>> = const { RefCell::new(Vec::new()) };
}

/// Records one assertion under d3's own semantics and under the tolerant intermediate format.
pub fn check<A: ToObs + ?Sized, B: ToObs + ?Sized>(id: usize, kind: Kind, actual: &A, expected: &B) {
    let (actual, expected) = (actual.to_obs(), expected.to_obs());
    let d3_pass = match kind {
        Kind::Strict => strict_eq(&actual, &expected),
        Kind::InDelta(d) => in_delta(&actual, &expected, d),
        Kind::PathEq => d3_normalize(&actual) == d3_normalize(&expected),
    };
    let tolerance = match kind {
        Kind::InDelta(d) => Tolerance::at_least(d),
        _ => Tolerance::default(),
    };
    let tolerant_pass = approx_eq(&actual, &expected, &tolerance);
    RECORDS.with(|r| r.borrow_mut().push(Record { id, kind, d3_pass, tolerant_pass, actual, expected }));
}

/// Marks an assertion that only tests JavaScript semantics.
pub fn skip(_id: usize, _reason: &str) {}

fn in_delta(a: &Obs, e: &Obs, d: f64) -> bool {
    match (a, e) {
        (Obs::Num(x), Obs::Num(y)) => *x >= y - d && *x <= y + d,
        (Obs::List(x), Obs::List(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| in_delta(p, q, d)),
        _ => a == e,
    }
}

/// d3-shape's `normalizePath`: integers within 1e-6 print as integers, everything else with 3 decimals.
fn d3_normalize(o: &Obs) -> Obs {
    let Obs::Str(s) = o else { return o.clone() };
    let mut out = String::new();
    for token in tokenize(s) {
        match token {
            Token::Text(t) => out.push_str(&t),
            Token::Num { value, .. } => {
                let r = value.round();
                if (value - r).abs() < 1e-6 {
                    out.push_str(&format!("{}", if r == 0.0 { 0.0 } else { r }));
                } else {
                    out.push_str(&format!("{value:.3}"));
                }
            }
        }
    }
    Obs::Str(out)
}

/// d3-path's test helper: `str.replace(/\d+\.\d+/g, s => +parseFloat(s).toFixed(p))`.
pub fn precision(s: &str, p: f64) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let start = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        if i > start && i + 1 < chars.len() && chars[i] == '.' && chars[i + 1].is_ascii_digit() {
            i += 1;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let v: f64 = chars[start..i].iter().collect::<String>().parse().expect("digits");
            let fixed: f64 = format!("{v:.*}", p as usize).parse().expect("formatted number");
            out.push_str(&format!("{fixed}"));
        } else if i > start {
            out.extend(&chars[start..i]);
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// d3-ease's test helper `out(easeIn)`.
pub fn out<E: Ease + Copy>(e: E) -> impl Fn(f64) -> f64 {
    move |t| 1.0 - e.ease(1.0 - t)
}

/// d3-ease's test helper `inOut(easeIn)`.
pub fn in_out<E: Ease + Copy>(e: E) -> impl Fn(f64) -> f64 {
    move |t| (if t < 0.5 { e.ease(t * 2.0) } else { 2.0 - e.ease((1.0 - t) * 2.0) }) / 2.0
}
