//! How close two observed values must be to count as equal.

use super::token::{Token, tokenize};

/// How close two values must be to count as equal.
#[derive(Debug, Clone, Copy)]
pub struct Tolerance {
    rel: f64,
    abs: f64,
    last_place: bool,
}

impl Default for Tolerance {
    /// Relative and absolute bounds of 1e-9, with last-place slack.
    fn default() -> Self {
        Self {
            rel: 1e-9,
            abs: 1e-9,
            last_place: true,
        }
    }
}

impl Tolerance {
    /// The default with the absolute bound raised to `abs` (d3's assertInDelta).
    pub fn at_least(abs: f64) -> Self {
        let default = Self::default();
        Self {
            abs: default.abs.max(abs),
            ..default
        }
    }

    /// Numbers within `rel` × magnitude and no absolute bound, for values
    /// near 0 whose scale matters (fixtures of tiny tick values).
    pub fn relative(rel: f64) -> Self {
        Self {
            rel,
            abs: 0.0,
            ..Self::default()
        }
    }

    /// Exact strings: no last-place slack.
    pub fn without_last_place(self) -> Self {
        Self {
            last_place: false,
            ..self
        }
    }

    /// NaN equals only NaN; infinities equal only themselves.
    pub(super) fn numbers(&self, a: f64, b: f64) -> bool {
        if a.is_nan() || b.is_nan() {
            return a.is_nan() && b.is_nan();
        }
        if a == b {
            return true;
        }
        a.is_finite()
            && b.is_finite()
            && (a - b).abs() <= self.abs.max(self.rel * a.abs().max(b.abs()))
    }

    pub(super) fn texts(&self, a: &str, b: &str) -> bool {
        if a == b {
            return true;
        }
        let (a, b) = (tokenize(a), tokenize(b));
        a.len() == b.len() && a.iter().zip(&b).all(|pair| self.tokens(pair))
    }

    fn tokens(&self, pair: (&Token, &Token)) -> bool {
        match pair {
            (Token::Text(a), Token::Text(b)) => a == b,
            (
                Token::Num {
                    value: a,
                    places: pa,
                },
                Token::Num {
                    value: b,
                    places: pb,
                },
            ) => {
                if self.numbers(*a, *b) {
                    return true;
                }
                let places = (*pa).min(*pb);
                let unit = 10f64.powi(-i32::try_from(places).unwrap_or(i32::MAX));
                self.last_place && places > 0 && (a - b).abs() <= unit * (1.0 + 1e-9)
            }
            (Token::Text(_), Token::Num { .. }) | (Token::Num { .. }, Token::Text(_)) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_least_raises_only_the_absolute_bound() {
        let tolerance = Tolerance::at_least(0.5);
        assert!(tolerance.numbers(1.0, 1.4));
        assert!(!tolerance.numbers(1.0, 1.6));
        assert!(Tolerance::at_least(1e-12).numbers(0.0, 1e-10));
    }

    #[test]
    fn without_last_place_compares_printed_decimals_exactly() {
        let exact = Tolerance::default().without_last_place();
        assert!(!exact.texts("0.13", "0.12"));
        assert!(exact.texts("1e+21", "1000000000000000000000"));
    }

    #[test]
    fn relative_has_no_absolute_floor() {
        let tolerance = Tolerance::relative(1e-9);
        assert!(!tolerance.numbers(1e-9, 2e-9));
        assert!(tolerance.numbers(1e-9, 1e-9 * (1.0 + 1e-12)));
        assert!(tolerance.numbers(0.0, 0.0));
        assert!(Tolerance::default().numbers(1e-9, 2e-9));
    }

    #[test]
    fn relative_bound_scales_with_magnitude() {
        let tolerance = Tolerance::default();
        assert!(tolerance.numbers(1e12, 1e12 + 1e2));
        assert!(!tolerance.numbers(1e12, 1e12 + 1e4));
    }
}
