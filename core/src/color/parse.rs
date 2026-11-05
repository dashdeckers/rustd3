//! d3-color 3.1.0's `color(format)` (color.js), hand-written: the CSS
//! syntaxes d3's regular expressions accept, in d3's order, read into the
//! color space each syntax names.

use crate::color::{Hsl, Rgb, named};

/// A CSS color in the space its syntax names: `hsl(…)` and `hsla(…)` keep
/// their components, everything else is sRGB.
pub(super) enum Parsed {
    Rgb(Rgb),
    Hsl(Hsl),
}

/// The color `input` names, or `None`. The input is trimmed of
/// JavaScript's whitespace and ASCII-lowercased, then read as `#rgb`,
/// `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb(…)` with integers or with
/// percentages, `rgba(…)` in both forms, `hsl(…)`, `hsla(…)`, one of the
/// 148 names, or `transparent`.
pub(super) fn parse(input: &str) -> Option<Parsed> {
    let s = input.trim_matches(is_space).to_ascii_lowercase();
    if let Some(digits) = s.strip_prefix('#') {
        return hex(digits).map(Parsed::Rgb);
    }
    if let Some(color) = functional(&s) {
        return Some(color);
    }
    if let Some(value) = named::lookup(&s) {
        return Some(Parsed::Rgb(Rgb::hex(value)));
    }
    (s == "transparent").then_some(Parsed::Rgb(Rgb {
        r: f64::NAN,
        g: f64::NAN,
        b: f64::NAN,
        opacity: 0.0,
    }))
}

/// `#` followed by 3, 4, 6 or 8 hexadecimal digits; a short form's digits
/// are doubled, and an alpha byte is divided by 255.
fn hex(digits: &str) -> Option<Rgb> {
    if !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let m = u32::from_str_radix(digits, 16).ok()?;
    let byte = |k: u32| f64::from((m >> (8 * k)) & 0xff);
    let doubled = |k: u32| f64::from(((m >> (4 * k)) & 0xf) * 0x11);
    match digits.len() {
        3 => Some(Rgb::new(doubled(2), doubled(1), doubled(0))),
        4 => Some(rgba(
            [doubled(3), doubled(2), doubled(1)],
            doubled(0) / 255.0,
        )),
        6 => Some(Rgb::hex(m)),
        8 => Some(rgba([byte(3), byte(2), byte(1)], byte(0) / 255.0)),
        _ => None,
    }
}

/// `rgb(…)`, `rgba(…)`, `hsl(…)` or `hsla(…)`: the name, an opening
/// parenthesis without space before it, comma-separated arguments with
/// optional whitespace around each, and a closing parenthesis last.
fn functional(s: &str) -> Option<Parsed> {
    let (name, rest) = s.split_once('(')?;
    let args: Vec<&str> = rest.strip_suffix(')')?.split(',').collect();
    match (name, args.as_slice()) {
        ("rgb", &[r, g, b]) => channels([r, g, b]).map(|[r, g, b]| Parsed::Rgb(Rgb::new(r, g, b))),
        ("rgba", &[r, g, b, a]) => {
            let [r, g, b] = channels([r, g, b])?;
            Some(Parsed::Rgb(rgba([r, g, b], number(a)?)))
        }
        ("hsl", &[h, s, l]) => Some(Parsed::Hsl(hsla(
            [number(h)?, percent(s)? / 100.0, percent(l)? / 100.0],
            1.0,
        ))),
        ("hsla", &[h, s, l, a]) => Some(Parsed::Hsl(hsla(
            [number(h)?, percent(s)? / 100.0, percent(l)? / 100.0],
            number(a)?,
        ))),
        _ => None,
    }
}

/// Three integers, or three percentages of 255 (d3's `m * 255 / 100`).
fn channels(args: [&str; 3]) -> Option<[f64; 3]> {
    let all = |read: fn(&str) -> Option<f64>| {
        let [r, g, b] = args.map(read);
        Some([r?, g?, b?])
    };
    all(integer).or_else(|| all(|a| percent(a).map(|p| p * 255.0 / 100.0)))
}

/// d3's `rgba(r, g, b, a)`: no channels when the alpha is at most 0.
fn rgba([r, g, b]: [f64; 3], opacity: f64) -> Rgb {
    if opacity <= 0.0 {
        Rgb {
            r: f64::NAN,
            g: f64::NAN,
            b: f64::NAN,
            opacity,
        }
    } else {
        Rgb { r, g, b, opacity }
    }
}

/// d3's `hsla(h, s, l, a)`: every component NaN when the alpha is at most
/// 0, no hue or saturation when the lightness is at most 0 or at least 1,
/// and no hue when the saturation is at most 0.
fn hsla([h, s, l]: [f64; 3], opacity: f64) -> Hsl {
    let (h, s, l) = if opacity <= 0.0 {
        (f64::NAN, f64::NAN, f64::NAN)
    } else if l <= 0.0 || l >= 1.0 {
        (f64::NAN, f64::NAN, l)
    } else if s <= 0.0 {
        (f64::NAN, s, l)
    } else {
        (h, s, l)
    };
    Hsl { h, s, l, opacity }
}

/// d3's `reI`: an optional sign and digits.
fn integer(arg: &str) -> Option<f64> {
    let t = arg.trim_matches(is_space);
    let digits = t.strip_prefix(['+', '-']).unwrap_or(t);
    let valid = !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit());
    valid.then(|| t.parse().ok()).flatten()
}

/// d3's `reN`: an optional sign, digits with an optional point before
/// the last of them, and an optional exponent.
fn number(arg: &str) -> Option<f64> {
    let t = arg.trim_matches(is_space);
    is_number(t).then(|| t.parse().ok()).flatten()
}

/// d3's `reP`: a number with `%` right after it.
fn percent(arg: &str) -> Option<f64> {
    let t = arg.trim_matches(is_space).strip_suffix('%')?;
    is_number(t).then(|| t.parse().ok()).flatten()
}

/// Whether `t` is `[+-]?(\d*\.)?\d+([eE][+-]?\d+)?` in full.
fn is_number(t: &str) -> bool {
    let unsigned = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let t = t.strip_prefix(['+', '-']).unwrap_or(t);
    let (mantissa, exponent) = match t.split_once(['e', 'E']) {
        Some((m, e)) => (m, Some(e.strip_prefix(['+', '-']).unwrap_or(e))),
        None => (t, None),
    };
    let mantissa = match mantissa.split_once('.') {
        Some((whole, fraction)) => whole.bytes().all(|b| b.is_ascii_digit()) && unsigned(fraction),
        None => unsigned(mantissa),
    };
    mantissa && exponent.is_none_or(unsigned)
}

/// JavaScript's whitespace, which `trim` and `\s` match.
fn is_space(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_follow_d3s_pattern() {
        for ok in ["1", "+1", "-1", ".5", "1.5", "1e1", "1E-1", "+.5e+2", "007"] {
            assert!(is_number(ok), "{ok}");
        }
        for bad in [
            "", "1.", ".", "e1", "1e", "1e+", "++1", "1.5.1", " 1", "1 ", "inf", "nan",
        ] {
            assert!(!is_number(bad), "{bad}");
        }
    }

    #[test]
    fn integers_take_a_sign_and_digits_only() {
        assert_eq!(integer(" +120 "), Some(120.0));
        assert_eq!(integer("-0"), Some(-0.0));
        assert_eq!(integer("1.5"), None);
        assert_eq!(integer("1e2"), None);
        assert_eq!(integer("+"), None);
    }

    #[test]
    fn whitespace_is_javascripts() {
        assert!(is_space('\u{feff}') && is_space('\u{2028}') && is_space('\u{b}'));
        assert!(!is_space('\u{85}') && !is_space('\u{200b}'));
    }
}
