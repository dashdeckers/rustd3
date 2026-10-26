//! Digit strings as JavaScript's `toFixed`, `toExponential` and
//! `toPrecision` print them, and d3-format's helpers over them.
//!
//! std's float formatting rounds a value exactly halfway between two
//! results to even; JavaScript rounds it up (`(2.5).toFixed(0)` is "3",
//! `format!("{:.0}", 2.5)` is "2"). Ties are detected exactly from the
//! float's bits and rounded up here. Every function takes a finite x ≥ 0.

use crate::js;

/// `n`, or 0 when it is negative, as a count.
pub(super) fn count(n: i32) -> usize {
    usize::try_from(n).unwrap_or(0)
}

/// d3's SI prefix exponent for a decimal exponent: ⌊e/3⌋·3 within [−24, 24].
pub(super) fn si_exponent(exponent: i32) -> i32 {
    exponent.div_euclid(3).clamp(-8, 8) * 3
}

/// d3's exponent: the decimal exponent of |x|'s leading digit, `None` for
/// 0, NaN and ±∞ (where d3's is NaN).
pub(super) fn exponent(x: f64) -> Option<i32> {
    let x = x.abs();
    (x.is_finite() && x != 0.0).then(|| shortest(x).1)
}

/// d3's formatDecimalParts: x's significant digits and the decimal exponent
/// of the first, rounded to `sig` digits, or as many as needed for `sig` 0;
/// `None` for 0 (and non-finite x).
pub(super) fn parts(x: f64, sig: usize) -> Option<(String, i32)> {
    if !x.is_finite() || x == 0.0 {
        None
    } else if sig == 0 {
        Some(shortest(x))
    } else {
        Some(significant(x, sig))
    }
}

/// JavaScript's `x.toFixed(places)` (below 10²¹, where JavaScript switches
/// to exponent notation and this keeps all digits), `Infinity` for ∞.
pub(super) fn fixed(x: f64, places: usize) -> String {
    if x.is_infinite() {
        // `%` multiplies by 100 first, which can overflow.
        return js::Number(x).to_string();
    }
    let tie = exact(x).is_some_and(|(s, q)| s % 10 == 5 && count(-q) == places + 1);
    if !tie {
        return format!("{x:.places$}");
    }
    // The expansion ends one place further, in 5: drop it and round up.
    let text = format!("{x:.*}", places + 1);
    let kept = text[..text.len() - 1].trim_end_matches('.');
    round_up(kept)
}

/// JavaScript's `x.toExponential(fraction)`: `d.ddde±e`.
pub(super) fn exponential(x: f64, fraction: usize) -> String {
    let (digits, exponent) = significant(x, fraction + 1);
    let (head, tail) = digits.split_at(1);
    let point = if tail.is_empty() { "" } else { "." };
    let sign = if exponent < 0 { '-' } else { '+' };
    format!("{head}{point}{tail}e{sign}{}", exponent.unsigned_abs())
}

/// JavaScript's `x.toPrecision(sig)`: fixed notation unless the exponent is
/// below −6 or at least `sig`.
pub(super) fn precision(x: f64, sig: usize) -> String {
    if x == 0.0 {
        return if sig > 1 {
            format!("0.{}", "0".repeat(sig - 1))
        } else {
            "0".to_owned()
        };
    }
    let (digits, exponent) = significant(x, sig);
    match usize::try_from(exponent) {
        Ok(e) if e >= sig => exponential(x, sig - 1),
        Ok(e) => point(&digits, e + 1),
        Err(_) if exponent < -6 => exponential(x, sig - 1),
        Err(_) => format!("0.{}{digits}", "0".repeat(count(-exponent - 1))),
    }
}

/// d3's formatRounded: `sig` significant digits in fixed notation.
pub(super) fn rounded(x: f64, sig: usize) -> String {
    let Some((digits, exponent)) = parts(x, sig) else {
        return js::Number(x).to_string();
    };
    match usize::try_from(exponent) {
        Ok(e) if digits.len() > e + 1 => point(&digits, e + 1),
        Ok(e) => format!("{digits}{}", "0".repeat(e + 1 - digits.len())),
        Err(_) => format!("0.{}{digits}", "0".repeat(count(-exponent - 1))),
    }
}

/// d3's formatPrefixAuto: `sig` significant digits scaled to an SI prefix,
/// and the prefix's exponent (`None` for 0, where d3 has none).
pub(super) fn prefix_auto(x: f64, sig: usize) -> (String, Option<i32>) {
    let Some((digits, exponent)) = parts(x, sig) else {
        return (precision(x, sig), None);
    };
    let prefix = si_exponent(exponent);
    let integer = exponent - prefix + 1;
    let n = digits.len();
    let text = match usize::try_from(integer) {
        Ok(i) if i == n => digits,
        Ok(i) if i > n => format!("{digits}{}", "0".repeat(i - n)),
        Ok(i) if i > 0 => point(&digits, i),
        // Below 1y: more digits than `sig`, after −integer zeros.
        Ok(_) | Err(_) => {
            let sig = i32::try_from(sig).expect("invariant: d3 clamps precision to 21");
            let more = parts(x, count(sig + integer - 1)).map_or_else(String::new, |p| p.0);
            format!("0.{}{more}", "0".repeat(count(-integer)))
        }
    };
    (text, Some(prefix))
}

/// d3's formatTrim: insignificant trailing zeros removed (`1.2000k` → `1.2k`).
pub(super) fn trim(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let (mut i0, mut i1) = (None, 0);
    for (i, &c) in chars.iter().enumerate().skip(1) {
        match c {
            '.' => (i0, i1) = (Some(i), i),
            '0' => {
                if i0 == Some(0) {
                    i0 = Some(i);
                }
                i1 = i;
            }
            '1'..='9' => {
                if i0.is_some_and(|i0| i0 > 0) {
                    i0 = Some(0);
                }
            }
            _ => break,
        }
    }
    match i0 {
        Some(i0) if i0 > 0 => chars[..i0].iter().chain(&chars[i1 + 1..]).collect(),
        Some(_) | None => text.to_owned(),
    }
}

/// `Math.round(x).toString(radix)` for a radix of 2, 8 or 16: long division,
/// exact in `f64` because the radix is a power of two.
pub(super) fn radix(x: f64, radix: u32) -> String {
    let base = f64::from(radix);
    let mut rest = js::round(x);
    let mut digits = Vec::new();
    while rest >= 1.0 {
        let digit = rest % base;
        let symbol = js::to_int(digit)
            .and_then(|d| u32::try_from(d).ok())
            .and_then(|d| char::from_digit(d, radix))
            .expect("invariant: a whole remainder below the radix");
        digits.push(symbol);
        rest = (rest - digit) / base;
    }
    if digits.is_empty() {
        "0".to_owned()
    } else {
        digits.iter().rev().collect()
    }
}

/// `digits` with a point after the first `k` (when any remain).
fn point(digits: &str, k: usize) -> String {
    if digits.len() > k {
        format!("{}.{}", &digits[..k], &digits[k..])
    } else {
        digits.to_owned()
    }
}

/// x > 0 rounded half up to `sig` ≥ 1 significant digits, and the decimal
/// exponent of the first.
fn significant(x: f64, sig: usize) -> (String, i32) {
    let tie = exact(x).is_some_and(|(s, _)| s % 10 == 5 && s.ilog10() + 1 == width(sig + 1));
    let shown = if tie { sig } else { sig - 1 };
    let (mut digits, mut exponent) = mantissa(&format!("{x:.shown$e}"));
    if tie {
        digits.pop();
        digits = round_up(&digits);
        // 9.5 → 10: one digit too many, a power of ten more.
        if digits.len() > sig {
            digits.pop();
            exponent += 1;
        }
    }
    (digits, exponent)
}

/// x > 0's shortest round-trip digits and the decimal exponent of the first.
fn shortest(x: f64) -> (String, i32) {
    mantissa(&format!("{x:e}"))
}

/// The digits and exponent of std's `{:e}` output (`1.25e-3` → `125`, −3).
fn mantissa(text: &str) -> (String, i32) {
    let (mantissa, exponent) = text
        .split_once('e')
        .expect("invariant: {:e} writes an exponent");
    let digits = mantissa.chars().filter(char::is_ascii_digit).collect();
    let exponent = exponent
        .parse()
        .expect("invariant: {:e} writes an integer exponent");
    (digits, exponent)
}

/// A digit count as `ilog10` reports it, saturating.
fn width(digits: usize) -> u32 {
    u32::try_from(digits).unwrap_or(u32::MAX)
}

/// Adds one unit in the last place to ASCII digits with at most one point.
fn round_up(text: &str) -> String {
    let mut bytes = text.as_bytes().to_vec();
    let mut carry = true;
    for b in bytes.iter_mut().rev() {
        match *b {
            b'.' => {}
            b'9' => *b = b'0',
            _ => {
                *b += 1;
                carry = false;
                break;
            }
        }
    }
    if carry {
        bytes.insert(0, b'1');
    }
    String::from_utf8(bytes).expect("invariant: ASCII digits stay ASCII")
}

/// x > 0's exact decimal value s·10^q, s without trailing zeros, when s
/// fits a `u128` (a tie at up to 21 digits always does).
fn exact(x: f64) -> Option<(u128, i32)> {
    if !x.is_finite() || x <= 0.0 {
        return None;
    }
    let bits = x.to_bits();
    let biased = i32::try_from((bits >> 52) & 0x7ff).expect("invariant: 11 exponent bits");
    let fraction = bits & ((1 << 52) - 1);
    let (mantissa, power) = if biased == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1 << 52), biased - 1075)
    };
    let zeros = mantissa.trailing_zeros();
    let odd = u128::from(mantissa >> zeros);
    let power = power + i32::try_from(zeros).expect("invariant: at most 64 zeros");
    if power < 0 {
        // odd · 2^power = odd · 5^−power / 10^−power, and odd · 5^k ends in 5.
        let s = 5u128.checked_pow(power.unsigned_abs())?.checked_mul(odd)?;
        Some((s, power))
    } else {
        let mut s = 1u128.checked_shl(power.unsigned_abs())?.checked_mul(odd)?;
        let mut q = 0;
        while s % 10 == 0 {
            s /= 10;
            q += 1;
        }
        Some((s, q))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_rounds_exact_ties_up_like_javascript() {
        assert_eq!(fixed(0.5, 0), "1");
        assert_eq!(fixed(2.5, 0), "3");
        assert_eq!(fixed(0.25, 1), "0.3");
        assert_eq!(fixed(9.995, 2), "9.99");
        assert_eq!(fixed(9.5, 0), "10");
        assert_eq!(fixed(1.005, 2), "1.00");
        assert_eq!(fixed(12.5, 0), "13");
        assert_eq!(fixed(2_251_799_813_685_248.5, 0), "2251799813685249");
    }

    #[test]
    fn exponential_and_precision_round_exact_ties_up() {
        assert_eq!(exponential(2.5, 0), "3e+0");
        assert_eq!(exponential(0.25, 0), "3e-1");
        assert_eq!(exponential(9.5, 0), "1e+1");
        assert_eq!(exponential(0.0, 2), "0.00e+0");
        assert_eq!(precision(0.125, 2), "0.13");
        assert_eq!(precision(12.5, 2), "13");
        assert_eq!(precision(125.0, 2), "1.3e+2");
        assert_eq!(precision(0.000_000_123, 2), "1.2e-7");
    }

    #[test]
    fn exact_values_strip_trailing_zeros() {
        assert_eq!(exact(1500.0), Some((15, 2)));
        assert_eq!(exact(0.125), Some((125, -3)));
        assert_eq!(exact(0.5), Some((5, -1)));
        assert_eq!(exact(0.1), None);
        assert_eq!(exact(f64::MAX), None);
        assert_eq!(exact(0.0), None);
    }

    #[test]
    fn trim_drops_insignificant_zeros() {
        assert_eq!(trim("1.2000"), "1.2");
        assert_eq!(trim("1.000"), "1");
        assert_eq!(trim("1.0200e+3"), "1.02e+3");
        assert_eq!(trim("100"), "100");
        assert_eq!(trim("Infinity"), "Infinity");
    }

    #[test]
    fn radix_digits_are_exact_beyond_two_to_the_53() {
        assert_eq!(radix(10.0, 2), "1010");
        assert_eq!(radix(255.4, 16), "ff");
        assert_eq!(radix(0.2, 8), "0");
        assert_eq!(radix(2f64.powi(60), 16), "1000000000000000");
        assert_eq!(radix(1e21, 16), "3635c9adc5dea00000");
    }
}
