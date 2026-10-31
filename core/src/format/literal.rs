//! Format specifiers checked at compile time: [`is_valid`], d3's specifier
//! grammar as a `const fn`, and the [`fmt!`](crate::fmt) macro built on it.

use crate::format::specifier::MAX_WIDTH;

/// Whether `spec` is a valid d3 format specifier,
/// `[[fill]align][sign][symbol][0][width][,][.precision][~][type]`, as
/// [`Specifier`](super::Specifier)'s `FromStr` judges it: d3's
/// `formatSpecifier.js` pattern, case-insensitive, whose fill is any one
/// character but a line terminator, and a width of at most 2²⁹ − 24.
/// Usable in `const`.
///
/// ```
/// use rustd3_core::format::literal::is_valid;
///
/// const MONEY: bool = is_valid("$,.2f");
/// assert!(MONEY);
/// assert!(is_valid("€^12,.3~s"));
/// assert!(!is_valid("10..f"));
/// assert!(!is_valid("536870889"));
/// ```
#[must_use]
pub const fn is_valid(spec: &str) -> bool {
    let b = spec.as_bytes();
    let mut i = after_align(b);
    if i < b.len() && matches!(b[i], b'+' | b'-' | b'(' | b' ') {
        i += 1;
    }
    if i < b.len() && matches!(b[i], b'$' | b'#') {
        i += 1;
    }
    if i < b.len() && b[i] == b'0' {
        i += 1;
    }
    let (end, width) = digits(b, i);
    if width > MAX_WIDTH {
        return false;
    }
    i = end;
    if i < b.len() && b[i] == b',' {
        i += 1;
    }
    if i < b.len() && b[i] == b'.' {
        let (end, _) = digits(b, i + 1);
        if end == i + 1 {
            return false;
        }
        i = end;
    }
    if i < b.len() && b[i] == b'~' {
        i += 1;
    }
    if i < b.len() && (b[i].is_ascii_alphabetic() || b[i] == b'%') {
        i += 1;
    }
    i == b.len()
}

/// A [`Format`](crate::format::Format) under the neutral locale from a
/// specifier literal that the compiler checks: an invalid one is a
/// compile-time error, `error[E0080]: evaluation panicked: invalid d3
/// format specifier: …`. Specifiers built at run time go through
/// [`format::format`](crate::format::format), which returns a `Result`; a
/// locale's own formats through [`Locale::format`](crate::format::Locale::format).
///
/// `fmt!` formats under [`Locale::NEUTRAL`](crate::format::Locale::NEUTRAL),
/// which has no currency symbol: `$` in a specifier writes nothing there.
/// For currency, format under a locale that has one, through the
/// locale's own `.format(…)` (the specifier then checked at run time):
/// `Locale::EN_US.format("$,.2f")` writes "$1,234.50".
///
/// ```
/// use rustd3_core::fmt;
/// use rustd3_core::format::Locale;
///
/// // No currency under the neutral locale.
/// let money = fmt!("$,.2f");
/// assert_eq!(money.format(-1234.5), "−1234.50");
/// // A locale with a currency writes it.
/// assert_eq!(Locale::EN_US.format("$,.2f")?.format(1234.5), "$1,234.50");
/// assert_eq!(Locale::DE_DE.format("$,.2f")?.format(1234.5), "1.234,50\u{a0}€");
/// assert_eq!(fmt!(",.1f").format(1234567.5), "1\u{202f}234\u{202f}567.5");
/// assert_eq!(fmt!(".3s").format(0.000_042), "42.0µ");
/// # Ok::<(), rustd3_core::format::InvalidFormat>(())
/// ```
///
/// ```compile_fail
/// // error[E0080]: evaluation panicked: invalid d3 format specifier: 10..f
/// let f = rustd3_core::fmt!("10..f");
/// ```
///
/// ```
/// let f = rustd3_core::fmt!("10.1f");
/// assert_eq!(f.format(2.25), "       2.3");
/// ```
///
/// Widths stop where d3's padding would exceed V8's longest string:
///
/// ```compile_fail
/// // error[E0080]: evaluation panicked: invalid d3 format specifier: 536870889
/// let f = rustd3_core::fmt!("536870889");
/// ```
///
/// ```
/// let f = rustd3_core::fmt!("536870888");
/// assert_eq!(f.to_string(), " >-536870888");
/// ```
#[macro_export]
macro_rules! fmt {
    ($spec:literal) => {{
        const _: () = if !$crate::format::literal::is_valid($spec) {
            ::core::panic!(
                "{}",
                ::core::concat!("invalid d3 format specifier: ", $spec)
            )
        };
        $crate::format::Locale::NEUTRAL
            .format($spec)
            .expect("invariant: fmt! checked the specifier at compile time")
    }};
}

/// Where the sign may start: after a fill and an align, after an align
/// alone, or at 0. The fill is one UTF-8 character, its length read from
/// its first byte, and not a line terminator.
const fn after_align(b: &[u8]) -> usize {
    if b.is_empty() {
        return 0;
    }
    let fill = match b[0].leading_ones() {
        0 => 1,
        n => n as usize,
    };
    if fill < b.len() && is_align(b[fill]) && !ends_line(b, fill) {
        fill + 1
    } else if is_align(b[0]) {
        1
    } else {
        0
    }
}

/// Whether `c` is one of `<>=^`.
const fn is_align(c: u8) -> bool {
    matches!(c, b'<' | b'>' | b'=' | b'^')
}

/// Whether the first `n` bytes of `b`, one character, are one of
/// JavaScript's line terminators: `\n`, `\r`, U+2028 or U+2029.
const fn ends_line(b: &[u8], n: usize) -> bool {
    match n {
        1 => b[0] == b'\n' || b[0] == b'\r',
        3 => b[0] == 0xe2 && b[1] == 0x80 && (b[2] == 0xa8 || b[2] == 0xa9),
        _ => false,
    }
}

/// The end of the run of ASCII digits from `i`, and its value, saturating.
const fn digits(b: &[u8], mut i: usize) -> (usize, usize) {
    let mut n: usize = 0;
    while i < b.len() && b[i].is_ascii_digit() {
        n = n.saturating_mul(10).saturating_add((b[i] - b'0') as usize);
        i += 1;
    }
    (i, n)
}
