//! Number formatting (d3-format), with specifiers checkable at compile time.

pub use d3_core::format::{Format, InvalidFormat, Locale, format, format_prefix};

/// Whether `s` matches `[[fill]align][sign][symbol][0][width][,][.precision][~][type]`, in a `const` context.
pub const fn is_valid(s: &str) -> bool {
    let b = s.as_bytes();
    let n = b.len();
    let mut i = 0;
    if n >= 2 && matches!(b[1], b'<' | b'>' | b'=' | b'^') {
        i = 2;
    } else if n >= 1 && matches!(b[0], b'<' | b'>' | b'=' | b'^') {
        i = 1;
    }
    if i < n && matches!(b[i], b'+' | b'-' | b'(' | b' ') { i += 1; }
    if i < n && matches!(b[i], b'$' | b'#') { i += 1; }
    if i < n && b[i] == b'0' { i += 1; }
    while i < n && b[i].is_ascii_digit() { i += 1; }
    if i < n && b[i] == b',' { i += 1; }
    if i < n && b[i] == b'.' {
        i += 1;
        if !(i < n && b[i].is_ascii_digit()) { return false; }
        while i < n && b[i].is_ascii_digit() { i += 1; }
    }
    if i < n && b[i] == b'~' { i += 1; }
    if i < n && (b[i].is_ascii_alphabetic() || b[i] == b'%') { i += 1; }
    i == n
}

/// A formatter whose specifier is checked by the compiler: `fmt!(",.2f")`.
#[macro_export]
macro_rules! fmt {
    ($spec:literal) => {{
        const _: () = assert!($crate::format::is_valid($spec), concat!("invalid d3 format specifier: ", $spec));
        $crate::format::format($spec).expect("checked at compile time")
    }};
}
