//! Observed values and their tolerant comparison.
//!
//! Both sides of a translated d3 assertion become an [`Obs`]. Numbers
//! compare within a [`Tolerance`]; strings split into text and number
//! [`Token`]s, so text must match exactly while numbers compare as values:
//! `1e-7` equals `0.0000001`, and `0.12` equals `0.13` (one unit in the last
//! printed place absorbs round-half-even against round-half-up).

mod json;
mod token;
mod tolerance;

use std::fmt;

pub use json::Error;
pub use token::{Token, tokenize};
pub use tolerance::Tolerance;

/// A value observed in a test, independent of how either language prints it.
#[derive(Debug, Clone, PartialEq)]
pub enum Obs {
    /// JavaScript's `null` and `undefined`, Rust's `None` and `()`.
    Null,
    /// A boolean.
    Bool(bool),
    /// A number; JavaScript has no other kind.
    Num(f64),
    /// A string, compared token by token.
    Str(String),
    /// An array, compared element by element.
    List(Vec<Obs>),
}

/// Converts a Rust value into an [`Obs`].
pub trait ToObs {
    /// The value as an [`Obs`].
    fn to_obs(&self) -> Obs;
}

impl ToObs for Obs {
    fn to_obs(&self) -> Obs {
        self.clone()
    }
}

impl ToObs for () {
    fn to_obs(&self) -> Obs {
        Obs::Null
    }
}

impl ToObs for f64 {
    fn to_obs(&self) -> Obs {
        Obs::Num(*self)
    }
}

impl ToObs for f32 {
    fn to_obs(&self) -> Obs {
        Obs::Num(f64::from(*self))
    }
}

impl ToObs for bool {
    fn to_obs(&self) -> Obs {
        Obs::Bool(*self)
    }
}

impl ToObs for i32 {
    fn to_obs(&self) -> Obs {
        Obs::Num(f64::from(*self))
    }
}

impl ToObs for u32 {
    fn to_obs(&self) -> Obs {
        Obs::Num(f64::from(*self))
    }
}

// JavaScript numbers are f64: integers beyond 2^53 round here as they do in d3.
impl ToObs for i64 {
    fn to_obs(&self) -> Obs {
        Obs::Num(*self as f64)
    }
}

impl ToObs for usize {
    fn to_obs(&self) -> Obs {
        Obs::Num(*self as f64)
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

impl<A: ToObs, B: ToObs> ToObs for (A, B) {
    fn to_obs(&self) -> Obs {
        Obs::List(vec![self.0.to_obs(), self.1.to_obs()])
    }
}

/// An `Err` becomes the string `error: <message>`, so a `Result` graded
/// exactly fails loudly, with the error's text, when it is `Err`.
impl<T: ToObs, E: fmt::Display> ToObs for Result<T, E> {
    fn to_obs(&self) -> Obs {
        match self {
            Ok(value) => value.to_obs(),
            Err(error) => Obs::Str(format!("error: {error}")),
        }
    }
}

impl Obs {
    /// Whether `self` equals `other` within `tolerance`. Numbers compare
    /// within max(abs, rel × magnitude) and strings token by token, where a
    /// decimal may differ by one unit in its last printed place.
    pub fn approx_eq(&self, other: &Obs, tolerance: &Tolerance) -> bool {
        match (self, other) {
            (Obs::Num(a), Obs::Num(b)) => tolerance.numbers(*a, *b),
            (Obs::Str(a), Obs::Str(b)) => tolerance.texts(a, b),
            (Obs::List(a), Obs::List(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(p, q)| p.approx_eq(q, tolerance))
            }
            (Obs::Null | Obs::Bool(_) | Obs::Num(_) | Obs::Str(_) | Obs::List(_), _) => {
                self == other
            }
        }
    }

    /// JavaScript's `Object.is`: NaN equals NaN, 0 differs from −0.
    pub fn strict_eq(&self, other: &Obs) -> bool {
        match (self, other) {
            (Obs::Num(a), Obs::Num(b)) => (a.is_nan() && b.is_nan()) || a.to_bits() == b.to_bits(),
            (Obs::List(a), Obs::List(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(p, q)| p.strict_eq(q))
            }
            (Obs::Null | Obs::Bool(_) | Obs::Num(_) | Obs::Str(_) | Obs::List(_), _) => {
                self == other
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: &str, b: &str) -> bool {
        a.to_obs().approx_eq(&b.to_obs(), &Tolerance::default())
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
        assert!(!close("inf", "Infinity"));
    }

    #[test]
    fn floats_compare_within_tolerance() {
        let tolerance = Tolerance::default();
        assert!(Obs::Num(0.1 + 0.2).approx_eq(&Obs::Num(0.3), &tolerance));
        assert!(Obs::Num(f64::NAN).approx_eq(&Obs::Num(f64::NAN), &tolerance));
        assert!(!Obs::Num(f64::INFINITY).approx_eq(&Obs::Num(f64::NEG_INFINITY), &tolerance));
    }

    #[test]
    fn strict_equality_is_object_is() {
        assert!(!Obs::Num(-0.0).strict_eq(&Obs::Num(0.0)));
        assert!(Obs::Num(-0.0).approx_eq(&Obs::Num(0.0), &Tolerance::default()));
        assert!(Obs::Num(f64::NAN).strict_eq(&Obs::Num(f64::NAN)));
        assert!(!Obs::Num(0.1 + 0.2).strict_eq(&Obs::Num(0.3)));
    }

    #[test]
    fn values_of_different_kinds_differ() {
        let tolerance = Tolerance::default();
        assert!(!Obs::Num(1.0).approx_eq(&Obs::Str("1".to_owned()), &tolerance));
        assert!(!Obs::Null.strict_eq(&Obs::Bool(false)));
        assert!(Obs::Null.approx_eq(&Obs::Null, &tolerance));
    }

    #[test]
    fn lists_compare_element_by_element() {
        let tolerance = Tolerance::default();
        assert!(
            vec![0.1 + 0.2, 1.0]
                .to_obs()
                .approx_eq(&[0.3, 1.0].to_obs(), &tolerance)
        );
        assert!(
            !vec![1.0]
                .to_obs()
                .approx_eq(&vec![1.0, 2.0].to_obs(), &tolerance)
        );
    }

    #[test]
    fn results_become_their_value_or_their_error_text() {
        assert_eq!(Ok::<f64, String>(2.0).to_obs(), Obs::Num(2.0));
        assert_eq!(
            Err::<f64, &str>("bad").to_obs(),
            Obs::Str("error: bad".to_owned())
        );
    }

    #[test]
    fn unit_none_and_pairs_have_obs_forms() {
        assert_eq!(().to_obs(), Obs::Null);
        assert_eq!(None::<f64>.to_obs(), Obs::Null);
        assert_eq!(Some(3_u32).to_obs(), Obs::Num(3.0));
        assert_eq!(
            (1_i64, "a").to_obs(),
            Obs::List(vec![Obs::Num(1.0), Obs::Str("a".to_owned())])
        );
        assert_eq!(2.5_f32.to_obs(), Obs::Num(2.5));
        assert_eq!([true; 2].to_obs(), Obs::List(vec![Obs::Bool(true); 2]));
    }
}
