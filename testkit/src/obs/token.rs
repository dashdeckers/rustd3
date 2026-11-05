//! Strings split into text and number tokens.

/// A piece of a string: literal text, or a number with its count of
/// printed decimal places.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Text between numbers, compared exactly.
    Text(String),
    /// A number and the decimal places it was printed with (`1.50` has 2,
    /// `1e-7` has 7, `1e+21` has 0).
    Num {
        /// The number's value.
        value: f64,
        /// Printed decimal places, net of the exponent, never below 0.
        places: u32,
    },
}

/// Splits `s` into text and number tokens. U+2212 reads as `-`.
///
/// A number is `[-+]?(digits[.digits]|.digits)([eE][-+]?digits)?`, so a
/// sign always belongs to the number after it.
pub fn tokenize(s: &str) -> Vec<Token> {
    let s = s.replace('\u{2212}', "-");
    let mut tokens = Vec::new();
    let (mut text, mut i) = (0, 0);
    while i < s.len() {
        let Some((end, places)) = number_at(s.as_bytes(), i) else {
            i += 1;
            continue;
        };
        if text < i {
            tokens.push(Token::Text(s[text..i].to_owned()));
        }
        let value = s[i..end]
            .parse()
            .expect("invariant: the scanned literal is a Rust float");
        tokens.push(Token::Num { value, places });
        (text, i) = (end, end);
    }
    if text < s.len() {
        tokens.push(Token::Text(s[text..].to_owned()));
    }
    tokens
}

/// The end of the number starting at `start` and its decimal places. Every
/// byte it accepts is ASCII, so both ends fall on character boundaries.
fn number_at(bytes: &[u8], start: usize) -> Option<(usize, u32)> {
    let digits = |from: usize| {
        bytes[from..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count()
    };
    let mut i = start;
    if matches!(bytes.get(i), Some(b'-' | b'+')) {
        i += 1;
    }
    let integer = digits(i);
    i += integer;
    let mut fraction = 0;
    if bytes.get(i) == Some(&b'.') {
        let len = digits(i + 1);
        if integer > 0 || len > 0 {
            i += 1 + len;
            fraction = len;
        }
    }
    if integer == 0 && fraction == 0 {
        return None;
    }
    let mut exponent = 0_i64;
    if matches!(bytes.get(i), Some(b'e' | b'E')) {
        let negative = bytes.get(i + 1) == Some(&b'-');
        let sign = usize::from(matches!(bytes.get(i + 1), Some(b'-' | b'+')));
        let len = digits(i + 1 + sign);
        if len > 0 {
            let magnitude = bytes[i + 1 + sign..i + 1 + sign + len]
                .iter()
                .fold(0_i64, |n, b| {
                    n.saturating_mul(10).saturating_add(i64::from(b - b'0'))
                });
            exponent = if negative { -magnitude } else { magnitude };
            i += 1 + sign + len;
        }
    }
    let places = i64::try_from(fraction)
        .unwrap_or(i64::MAX)
        .saturating_sub(exponent);
    Some((i, u32::try_from(places.max(0)).unwrap_or(u32::MAX)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num(value: f64, places: u32) -> Token {
        Token::Num { value, places }
    }

    fn text(s: &str) -> Token {
        Token::Text(s.to_owned())
    }

    #[test]
    fn numbers_split_from_text() {
        assert_eq!(
            tokenize("M1.5,-2L3e2,0"),
            vec![
                text("M"),
                num(1.5, 1),
                text(","),
                num(-2.0, 0),
                text("L"),
                num(300.0, 0),
                text(","),
                num(0.0, 0)
            ]
        );
    }

    #[test]
    fn exponents_shift_the_decimal_places() {
        assert_eq!(tokenize("1e-7"), vec![num(1e-7, 7)]);
        assert_eq!(tokenize("1.25e+1"), vec![num(12.5, 1)]);
        assert_eq!(tokenize("1e+21"), vec![num(1e21, 0)]);
    }

    #[test]
    fn unicode_minus_reads_as_hyphen_minus() {
        assert_eq!(tokenize("\u{2212}1"), vec![num(-1.0, 0)]);
    }

    #[test]
    fn text_without_digits_is_one_token() {
        assert_eq!(tokenize("Infinity µs"), vec![text("Infinity µs")]);
        assert_eq!(tokenize("e.-"), vec![text("e.-")]);
        assert!(tokenize("").is_empty());
    }

    #[test]
    fn a_trailing_dot_joins_the_number_but_a_bare_exponent_marker_does_not() {
        assert_eq!(tokenize("5.x"), vec![num(5.0, 0), text("x")]);
        assert_eq!(tokenize("2em"), vec![num(2.0, 0), text("em")]);
        assert_eq!(tokenize(".5"), vec![num(0.5, 1)]);
    }

    #[test]
    fn huge_exponents_saturate() {
        assert_eq!(
            tokenize("1e999999999999999999999"),
            vec![num(f64::INFINITY, 0)]
        );
        assert_eq!(
            tokenize("1e-999999999999999999999"),
            vec![num(0.0, u32::MAX)]
        );
    }
}
