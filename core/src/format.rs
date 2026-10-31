//! d3-format: numbers formatted by d3's specifier mini-language under a
//! locale.
//!
//! The free functions use the neutral locale, SI style: a point, digits
//! grouped by a narrow no-break space from five integer digits on, no
//! currency, and − for minus. d3's own default is [`Locale::EN_US`].
//! d3-format's tables for en-US and the ten languages of the time tables
//! are constants of [`Locale`] (`DE_DE`, `FR_FR`, `ES_ES`, `IT_IT`,
//! `PT_BR`, `NL_NL`, `PL_PL`, `RU_RU`, `JA_JP`, `ZH_CN`); its other tables
//! load at run time through [`Locale::new`]. A specifier literal is checked
//! by the compiler through [`fmt!`](crate::fmt).
//!
//! ```
//! use rustd3_core::format::{self, Locale};
//!
//! assert_eq!(format::format(".1%")?.format(0.123), "12.3%");
//! assert_eq!(format::format(",.2f")?.format(-12345.678), "−12\u{202f}345.68");
//! assert_eq!(Locale::EN_US.format("$,.2f")?.format(-1234.5), "−$1,234.50");
//! assert_eq!(Locale::DE_DE.format(",.1f")?.format(1234567.5), "1.234.567,5");
//! assert!(format::format("10..f").is_err());
//! assert_eq!(rustd3_core::fmt!(".3s").format(0.000_042), "42.0µ");
//! # Ok::<(), format::InvalidFormat>(())
//! ```

mod decimal;
// `fmt!`'s compile-time check, which the macro's expansion names from the
// caller's crate: public for that alone.
#[doc(hidden)]
pub mod literal;
mod locale;
#[rustfmt::skip]
mod locales;
mod precision;
mod specifier;

use std::fmt;

use crate::js;

pub use locale::{Definition, Locale};
pub(crate) use precision::precision_fixed_between;
pub use precision::{precision_fixed, precision_prefix, precision_round};
pub use specifier::{Align, InvalidFormat, Kind, Sign, Specifier, Symbol};

/// d3's SI prefixes, yocto (10⁻²⁴) to yotta (10²⁴).
const PREFIXES: [&str; 17] = [
    "y", "z", "a", "f", "p", "n", "µ", "m", "", "k", "M", "G", "T", "P", "E", "Z", "Y",
];

/// A number formatter: a specifier bound to a locale.
#[derive(Debug, Clone, PartialEq)]
pub struct Format {
    locale: Locale,
    specifier: Specifier,
    kind: Type,
    precision: usize,
    trim: bool,
    comma: bool,
    zero: bool,
    fill: char,
    align: Align,
    /// Written before the digits, after the sign: currency or `0x`.
    prefix: String,
    /// Written after the digits: currency, percent, a fixed SI symbol.
    suffix: String,
    /// formatPrefix's factor to the fixed SI prefix, and its symbol.
    si: Option<(f64, &'static str)>,
}

/// A [`Kind`] as d3 normalizes it: `n` is `,g`, and none or an unknown
/// letter is `~g` at precision 12.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Type {
    Exponent,
    Fixed,
    General,
    Rounded,
    Si,
    Percent,
    PercentRounded,
    Binary,
    Octal,
    Decimal,
    Hex,
    HexUpper,
    Char,
}

/// A value split as d3 writes it: sign and prefix, the integer digits that
/// grouping applies to, and the rest.
struct Parts {
    prefix: String,
    digits: String,
    suffix: String,
}

impl fmt::Display for Format {
    /// Its specifier, as d3's `format.toString()`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.specifier.fmt(f)
    }
}

impl Format {
    /// The formatter `specifier` describes under `locale`.
    #[must_use]
    pub fn new(locale: &Locale, specifier: Specifier) -> Format {
        Format::with_scale(locale, specifier, None)
    }

    /// d3's formatPrefix: fixed-point, scaled to `value`'s SI prefix.
    #[must_use]
    pub fn prefixed(locale: &Locale, specifier: Specifier, value: f64) -> Format {
        // d3's exponent of 0, NaN or ±∞ is NaN, which propagates: every
        // value then formats as NaN, with no SI symbol.
        let (scale, symbol) = match decimal::exponent(value) {
            Some(e) => {
                let e = decimal::si_exponent(e);
                (10f64.powf(f64::from(-e)), si_symbol(e))
            }
            None => (f64::NAN, ""),
        };
        let specifier = specifier.with_kind(Some(Kind::Fixed));
        Format::with_scale(locale, specifier, Some((scale, symbol)))
    }

    /// The locale this format writes in.
    #[must_use]
    pub fn get_locale(&self) -> &Locale {
        &self.locale
    }

    /// This format under `locale`: the same specifier and fixed SI prefix,
    /// with the separators, currency and numerals `locale` gives them.
    pub(crate) fn with_locale(&self, locale: &Locale) -> Format {
        Format::with_scale(locale, self.specifier.clone(), self.si)
    }

    /// `value` formatted. With `,`, it is grouped when its integer digits
    /// as printed reach the locale's threshold (`NEUTRAL`: 5).
    #[must_use]
    pub fn format(&self, value: f64) -> String {
        let parts = self.parts(value);
        let grouped = self.groups(&parts);
        self.finish(parts, grouped)
    }

    /// `values` formatted as one axis. With `,` under a locale that groups
    /// from N integer digits (`NEUTRAL`: 5), either every value is grouped,
    /// when some value has at least N integer digits as printed, or none
    /// is; under a locale that always groups (`EN_US`, d3's tables) each
    /// value reads as [`format`](Format::format) writes it.
    ///
    /// ```
    /// use rustd3_core::format;
    ///
    /// let f = format::format(",")?;
    /// assert_eq!(f.format_all(&[8000.0, 9000.0]), ["8000", "9000"]);
    /// assert_eq!(f.format_all(&[8000.0, 10000.0]), ["8\u{202f}000", "10\u{202f}000"]);
    /// # Ok::<(), format::InvalidFormat>(())
    /// ```
    #[must_use]
    pub fn format_all(&self, values: &[f64]) -> Vec<String> {
        let parts: Vec<Parts> = values.iter().map(|&v| self.parts(v)).collect();
        let grouped = parts.iter().any(|p| self.groups(p));
        parts.into_iter().map(|p| self.finish(p, grouped)).collect()
    }

    /// Whether `,` groups a value of these parts on its own: its integer
    /// digits reach the locale's threshold, which 0 or 1 always meets.
    fn groups(&self, parts: &Parts) -> bool {
        let from = self.locale.group_from;
        self.comma && (from <= 1 || parts.digits.chars().count() >= from)
    }

    /// The parts of `value` as printed, before grouping and padding.
    fn parts(&self, value: f64) -> Parts {
        let value = self.si.map_or(value, |(k, _)| k * value);
        match self.kind {
            Type::Char => Parts {
                prefix: self.prefix.clone(),
                digits: String::new(),
                suffix: format!(
                    "{}{}",
                    self.kind.render(value, self.precision).0,
                    self.suffix
                ),
            },
            Type::Exponent
            | Type::Fixed
            | Type::General
            | Type::Rounded
            | Type::Si
            | Type::Percent
            | Type::PercentRounded
            | Type::Binary
            | Type::Octal
            | Type::Decimal
            | Type::Hex
            | Type::HexUpper => self.number(value),
        }
    }

    /// `parts` grouped when `grouped`, padded to the width, aligned and
    /// written in the locale's numerals.
    fn finish(&self, parts: Parts, grouped: bool) -> String {
        let Parts {
            prefix,
            mut digits,
            suffix,
        } = parts;
        if grouped && !self.zero {
            digits = self.locale.group(&digits, usize::MAX);
        }
        let width = self.specifier.get_width().unwrap_or(0);
        let length = prefix.chars().count() + digits.chars().count() + suffix.chars().count();
        let mut padding = width.saturating_sub(length);
        if grouped && self.zero {
            // Zeros are grouped too, within the width.
            let limit = if padding > 0 {
                width.saturating_sub(suffix.chars().count())
            } else {
                usize::MAX
            };
            digits = self.locale.group(&("0".repeat(padding) + &digits), limit);
            padding = 0;
        }
        let pad = |n: usize| self.fill.to_string().repeat(n);
        let out = match self.align {
            Align::Left => format!("{prefix}{digits}{suffix}{}", pad(padding)),
            Align::AfterSign => format!("{prefix}{}{digits}{suffix}", pad(padding)),
            Align::Center => {
                let half = padding / 2;
                format!(
                    "{}{prefix}{digits}{suffix}{}",
                    pad(half),
                    pad(padding - half)
                )
            }
            Align::Right => format!("{}{prefix}{digits}{suffix}", pad(padding)),
        };
        self.locale.numerals(out)
    }

    /// d3's newFormat: the specifier normalized, with the prefix and suffix
    /// `locale` gives it, and formatPrefix's scale and symbol.
    fn with_scale(
        locale: &Locale,
        specifier: Specifier,
        si: Option<(f64, &'static str)>,
    ) -> Format {
        let (mut comma, mut trim, mut precision) = (
            specifier.get_comma(),
            specifier.get_trim(),
            specifier.get_precision(),
        );
        let kind = match specifier.get_kind() {
            Some(Kind::Exponent) => Type::Exponent,
            Some(Kind::Fixed) => Type::Fixed,
            Some(Kind::General) => Type::General,
            Some(Kind::Rounded) => Type::Rounded,
            Some(Kind::Si) => Type::Si,
            Some(Kind::Percent) => Type::Percent,
            Some(Kind::PercentRounded) => Type::PercentRounded,
            Some(Kind::Binary) => Type::Binary,
            Some(Kind::Octal) => Type::Octal,
            Some(Kind::Decimal) => Type::Decimal,
            Some(Kind::Hex) => Type::Hex,
            Some(Kind::HexUpper) => Type::HexUpper,
            Some(Kind::Char) => Type::Char,
            Some(Kind::Number) => {
                comma = true;
                Type::General
            }
            Some(Kind::Other(_)) | None => {
                precision.get_or_insert(12);
                trim = true;
                Type::General
            }
        };
        let (zero, fill, align) = if specifier.get_zero()
            || (specifier.get_fill() == '0' && specifier.get_align() == Align::AfterSign)
        {
            (true, '0', Align::AfterSign)
        } else {
            (false, specifier.get_fill(), specifier.get_align())
        };
        let prefix = match specifier.get_symbol() {
            Some(Symbol::Currency) => locale.currency[0].to_string(),
            Some(Symbol::Base) => kind.base().to_owned(),
            None => String::new(),
        };
        let mut suffix = match specifier.get_symbol() {
            Some(Symbol::Currency) => locale.currency[1].to_string(),
            Some(Symbol::Base) | None if kind.percent() => locale.percent.to_string(),
            Some(Symbol::Base) | None => String::new(),
        };
        suffix.push_str(si.map_or("", |(_, symbol)| symbol));
        let precision = match precision {
            None => 6,
            Some(p) if kind.significant() => p.clamp(1, 21),
            Some(p) => p.min(20),
        };
        Format {
            locale: locale.clone(),
            specifier,
            kind,
            precision,
            trim,
            comma,
            zero,
            fill,
            align,
            prefix,
            suffix,
            si,
        }
    }

    /// A number's sign, digits and suffix, before grouping and padding.
    fn number(&self, value: f64) -> Parts {
        let sign = self.specifier.get_sign();
        let negative = value < 0.0 || 1.0 / value < 0.0;
        let (mut text, si) = if value.is_nan() {
            (self.locale.nan.to_string(), None)
        } else if value.is_infinite() {
            ("Infinity".to_owned(), None)
        } else {
            self.kind.render(value.abs(), self.precision)
        };
        if self.trim {
            text = decimal::trim(&text);
        }
        // A negative value that formats as zero loses its sign, unless `+` asks for one.
        let negative = negative && !(text.parse::<f64>() == Ok(0.0) && sign != Sign::Plus);
        let (open, close) = match (negative, sign) {
            (true, Sign::Parens) => ("(", ")"),
            (true, Sign::Minus | Sign::Plus | Sign::Space) => (&*self.locale.minus, ""),
            (false, Sign::Minus | Sign::Parens) => ("", ""),
            (false, Sign::Plus) => ("+", ""),
            (false, Sign::Space) => (" ", ""),
        };
        let symbol = si.map_or("", si_symbol);
        let mut parts = Parts {
            prefix: format!("{open}{}", self.prefix),
            digits: text,
            suffix: format!("{symbol}{}{close}", self.suffix),
        };
        if self.kind.splits() {
            // The integer digits are grouped; the point, fraction or exponent are not.
            if let Some(i) = parts.digits.find(|c: char| !c.is_ascii_digit()) {
                let rest = match parts.digits[i..].strip_prefix('.') {
                    Some(fraction) => format!("{}{fraction}", self.locale.decimal),
                    None => parts.digits[i..].to_owned(),
                };
                parts.suffix = rest + &parts.suffix;
                parts.digits.truncate(i);
            }
        }
        parts
    }
}

impl Type {
    /// d3's formatTypes: `x` (≥ 0 and finite for every type but `c`) at
    /// precision `p`, and the SI prefix exponent `s` chose.
    fn render(self, x: f64, p: usize) -> (String, Option<i32>) {
        let text = match self {
            Type::Exponent => decimal::exponential(x, p),
            Type::Fixed => decimal::fixed(x, p),
            Type::General => decimal::precision(x, p),
            Type::Rounded => decimal::rounded(x, p),
            Type::Si => return decimal::prefix_auto(x, p),
            Type::Percent => decimal::fixed(x * 100.0, p),
            Type::PercentRounded => decimal::rounded(x * 100.0, p),
            Type::Binary => decimal::radix(x, 2),
            Type::Octal => decimal::radix(x, 8),
            Type::Decimal => js::Number(js::round(x)).to_string(),
            Type::Hex => decimal::radix(x, 16),
            Type::HexUpper => decimal::radix(x, 16).to_ascii_uppercase(),
            Type::Char => js::Number(x).to_string(),
        };
        (text, None)
    }

    /// `#`'s prefix for this type.
    fn base(self) -> &'static str {
        match self {
            Type::Binary => "0b",
            Type::Octal => "0o",
            Type::Hex | Type::HexUpper => "0x",
            Type::Exponent
            | Type::Fixed
            | Type::General
            | Type::Rounded
            | Type::Si
            | Type::Percent
            | Type::PercentRounded
            | Type::Decimal
            | Type::Char => "",
        }
    }

    /// Whether the precision counts significant digits (`g p r s`), in
    /// [1, 21], rather than decimal places, in [0, 20].
    fn significant(self) -> bool {
        match self {
            Type::General | Type::PercentRounded | Type::Rounded | Type::Si => true,
            Type::Exponent
            | Type::Fixed
            | Type::Percent
            | Type::Binary
            | Type::Octal
            | Type::Decimal
            | Type::Hex
            | Type::HexUpper
            | Type::Char => false,
        }
    }

    /// Whether the locale's percent sign follows (`%` and `p`).
    fn percent(self) -> bool {
        match self {
            Type::Percent | Type::PercentRounded => true,
            Type::Exponent
            | Type::Fixed
            | Type::General
            | Type::Rounded
            | Type::Si
            | Type::Binary
            | Type::Octal
            | Type::Decimal
            | Type::Hex
            | Type::HexUpper
            | Type::Char => false,
        }
    }

    /// Whether a point, fraction or exponent may follow the integer digits
    /// (`d e f g p r s %`).
    fn splits(self) -> bool {
        match self {
            Type::Decimal
            | Type::Exponent
            | Type::Fixed
            | Type::General
            | Type::PercentRounded
            | Type::Rounded
            | Type::Si
            | Type::Percent => true,
            Type::Binary | Type::Octal | Type::Hex | Type::HexUpper | Type::Char => false,
        }
    }
}

/// `specifier` under the neutral default locale.
///
/// # Errors
///
/// [`InvalidFormat`] when `specifier` does not parse.
pub fn format(specifier: &str) -> Result<Format, InvalidFormat> {
    Locale::NEUTRAL.format(specifier)
}

/// d3's formatPrefix under the neutral default locale: fixed-point, scaled
/// to `value`'s SI prefix.
///
/// # Errors
///
/// [`InvalidFormat`] when `specifier` does not parse.
pub fn format_prefix(specifier: &str, value: f64) -> Result<Format, InvalidFormat> {
    Locale::NEUTRAL.format_prefix(specifier, value)
}

/// The SI symbol of a prefix exponent from [`decimal::si_exponent`].
fn si_symbol(exponent: i32) -> &'static str {
    usize::try_from(exponent / 3 + 8)
        .ok()
        .and_then(|i| PREFIXES.get(i))
        .copied()
        .expect("invariant: a prefix exponent in [-24, 24]")
}
