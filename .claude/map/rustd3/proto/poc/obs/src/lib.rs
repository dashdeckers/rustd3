//! An intermediate format for test observations, with "almost equal" comparison.
//!
//! Both sides of a ported d3 assertion are converted to [`Obs`]. Numbers compare
//! within a relative/absolute tolerance. Strings are split into text and number
//! tokens: text must match exactly, numbers compare numerically, so `1e-7` equals
//! `0.0000001` and `0.12` equals `0.13` (one unit in the last printed decimal place,
//! which absorbs round-half-even versus round-half-up). Path strings are strings.

/// A value observed in a test, independent of how either language prints it.
#[derive(Debug, Clone, PartialEq)]
pub enum Obs {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    List(Vec<Obs>),
}

/// Converts a Rust value into an [`Obs`].
pub trait ToObs {
    fn to_obs(&self) -> Obs;
}

impl ToObs for Obs {
    fn to_obs(&self) -> Obs {
        self.clone()
    }
}
impl ToObs for f64 {
    fn to_obs(&self) -> Obs {
        Obs::Num(*self)
    }
}
impl ToObs for bool {
    fn to_obs(&self) -> Obs {
        Obs::Bool(*self)
    }
}
impl ToObs for str {
    fn to_obs(&self) -> Obs {
        Obs::Str(self.to_owned())
    }
}
impl ToObs for String {
    fn to_obs(&self) -> Obs {
        Obs::Str(self.clone())
    }
}
impl<T: ToObs> ToObs for [T] {
    fn to_obs(&self) -> Obs {
        Obs::List(self.iter().map(ToObs::to_obs).collect())
    }
}
impl<T: ToObs> ToObs for Vec<T> {
    fn to_obs(&self) -> Obs {
        self.as_slice().to_obs()
    }
}
impl<T: ToObs, const N: usize> ToObs for [T; N] {
    fn to_obs(&self) -> Obs {
        self.as_slice().to_obs()
    }
}
impl<T: ToObs + ?Sized> ToObs for &T {
    fn to_obs(&self) -> Obs {
        (**self).to_obs()
    }
}
impl<T: ToObs> ToObs for Option<T> {
    fn to_obs(&self) -> Obs {
        self.as_ref().map_or(Obs::Null, ToObs::to_obs)
    }
}

/// How close two values must be to count as equal.
#[derive(Debug, Clone, Copy)]
pub struct Tolerance {
    /// Relative tolerance, scaled by the larger magnitude.
    pub rel: f64,
    /// Absolute tolerance, for values near zero.
    pub abs: f64,
    /// Accept numbers in strings that differ by one unit in their last printed decimal place.
    pub last_place: bool,
}

impl Default for Tolerance {
    fn default() -> Self {
        Self { rel: 1e-9, abs: 1e-9, last_place: true }
    }
}

impl Tolerance {
    /// The default tolerance with the absolute bound raised to `abs`.
    pub fn at_least(abs: f64) -> Self {
        let default = Self::default();
        Self { abs: default.abs.max(abs), ..default }
    }
}

/// Whether `a` and `b` are equal within `tol`.
pub fn approx_eq(a: &Obs, b: &Obs, tol: &Tolerance) -> bool {
    match (a, b) {
        (Obs::Num(x), Obs::Num(y)) => num_close(*x, *y, tol),
        (Obs::Str(x), Obs::Str(y)) => text_close(x, y, tol),
        (Obs::List(x), Obs::List(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| approx_eq(p, q, tol))
        }
        _ => a == b,
    }
}

/// Exact comparison with JavaScript's `Object.is` semantics: `NaN` equals `NaN`, `0` does not equal `-0`.
pub fn strict_eq(a: &Obs, b: &Obs) -> bool {
    match (a, b) {
        (Obs::Num(x), Obs::Num(y)) => (x.is_nan() && y.is_nan()) || x.to_bits() == y.to_bits(),
        (Obs::List(x), Obs::List(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| strict_eq(p, q))
        }
        _ => a == b,
    }
}

fn num_close(a: f64, b: f64, tol: &Tolerance) -> bool {
    if a.is_nan() || b.is_nan() {
        return a.is_nan() && b.is_nan();
    }
    if a == b {
        return true;
    }
    if !a.is_finite() || !b.is_finite() {
        return false;
    }
    (a - b).abs() <= tol.abs.max(tol.rel * a.abs().max(b.abs()))
}

/// A piece of a string: literal text, or a number with its count of printed decimal places.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Text(String),
    Num { value: f64, places: u32 },
}

/// Splits a string into text and number tokens. U+2212 (d3-format's minus sign) reads as `-`.
pub fn tokenize(s: &str) -> Vec<Token> {
    let chars: Vec<char> = s.chars().map(|c| if c == '\u{2212}' { '-' } else { c }).collect();
    let mut tokens = Vec::new();
    let mut text = String::new();
    let mut i = 0;
    while i < chars.len() {
        if let Some((end, value, places)) = number_at(&chars, i) {
            if !text.is_empty() {
                tokens.push(Token::Text(std::mem::take(&mut text)));
            }
            tokens.push(Token::Num { value, places });
            i = end;
        } else {
            text.push(chars[i]);
            i += 1;
        }
    }
    if !text.is_empty() {
        tokens.push(Token::Text(text));
    }
    tokens
}

/// Parses `[-+]?(digits[.digits]|.digits)([eE][-+]?digits)?` starting at `start`.
fn number_at(chars: &[char], start: usize) -> Option<(usize, f64, u32)> {
    let digits = |mut i: usize| {
        let from = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        (i, i - from)
    };
    let mut i = start;
    if i < chars.len() && (chars[i] == '-' || chars[i] == '+') {
        i += 1;
    }
    let (after_int, int_len) = digits(i);
    i = after_int;
    let mut frac_len = 0;
    if i < chars.len() && chars[i] == '.' {
        let (after_frac, len) = digits(i + 1);
        if int_len > 0 || len > 0 {
            i = after_frac;
            frac_len = len;
        }
    }
    if int_len == 0 && frac_len == 0 {
        return None;
    }
    let mut exponent = 0i32;
    if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
        let mut j = i + 1;
        if j < chars.len() && (chars[j] == '-' || chars[j] == '+') {
            j += 1;
        }
        let (after_exp, exp_len) = digits(j);
        if exp_len > 0 {
            exponent = chars[i + 1..after_exp].iter().collect::<String>().parse().ok()?;
            i = after_exp;
        }
    }
    let literal: String = chars[start..i].iter().collect();
    let value = literal.parse::<f64>().ok()?;
    let places = (frac_len as i32 - exponent).max(0) as u32;
    Some((i, value, places))
}

fn text_close(a: &str, b: &str, tol: &Tolerance) -> bool {
    if a == b {
        return true;
    }
    let (ta, tb) = (tokenize(a), tokenize(b));
    ta.len() == tb.len()
        && ta.iter().zip(&tb).all(|pair| match pair {
            (Token::Text(x), Token::Text(y)) => x == y,
            (Token::Num { value: x, places: px }, Token::Num { value: y, places: py }) => {
                if num_close(*x, *y, tol) {
                    return true;
                }
                let places = (*px).min(*py);
                tol.last_place && places > 0 && (x - y).abs() <= 10f64.powi(-(places as i32)) * (1.0 + 1e-9)
            }
            _ => false,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: &str, b: &str) -> bool {
        approx_eq(&a.to_obs(), &b.to_obs(), &Tolerance::default())
    }

    #[test]
    fn exponent_and_plain_notation_are_equal() {
        assert!(close("M1e-7,0", "M0.0000001,0"));
        assert!(close("1e+21", "1000000000000000000000"));
    }

    #[test]
    fn rounding_ties_differ_by_one_last_place() {
        assert!(close("0.13", "0.12"));
        assert!(!close("0.14", "0.12"));
    }

    #[test]
    fn integers_get_no_last_place_slack() {
        assert!(!close("2010-01-01", "2010-01-02"));
        assert!(close("-0", "0"));
    }

    #[test]
    fn text_must_match_exactly() {
        assert!(!close("M0,0L1,1", "M0,0C1,1"));
        assert!(close("\u{2212}1,234.50", "-1,234.50"));
    }

    #[test]
    fn floats_compare_within_tolerance() {
        let tol = Tolerance::default();
        assert!(approx_eq(&Obs::Num(0.1 + 0.2), &Obs::Num(0.3), &tol));
        assert!(approx_eq(&Obs::Num(f64::NAN), &Obs::Num(f64::NAN), &tol));
        assert!(!approx_eq(&Obs::Num(f64::INFINITY), &Obs::Num(f64::NEG_INFINITY), &tol));
    }
}
