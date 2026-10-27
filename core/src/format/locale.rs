//! d3's formatLocale: the separators, grouping, currency and numerals a
//! number format writes.

use std::borrow::Cow;

use crate::format::{Format, InvalidFormat};

/// d3's formatLocale: separators, grouping, currency, numerals.
#[derive(Debug, Clone, PartialEq)]
pub struct Locale {
    pub(super) decimal: Cow<'static, str>,
    pub(super) thousands: Cow<'static, str>,
    pub(super) grouping: Cow<'static, [usize]>,
    pub(super) currency: [Cow<'static, str>; 2],
    pub(super) numerals: Option<[Cow<'static, str>; 10]>,
    pub(super) percent: Cow<'static, str>,
    pub(super) minus: Cow<'static, str>,
    pub(super) nan: Cow<'static, str>,
    pub(super) group_from: usize,
}

/// What a locale table states; `Default` is d3's fallbacks (locale.js 12–21):
/// ".", no grouping (empty), ["", ""], no numerals, "%", "\u{2212}", "NaN",
/// group_from 1.
#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    /// The decimal point.
    pub decimal: Cow<'static, str>,
    /// The group separator, written only where `grouping` is not empty.
    pub thousands: Cow<'static, str>,
    /// Group sizes from the decimal point leftward, the last repeating;
    /// empty for no grouping.
    pub grouping: Cow<'static, [usize]>,
    /// The currency prefix and suffix that `$` writes.
    pub currency: [Cow<'static, str>; 2],
    /// The ten digits, when not 0–9.
    pub numerals: Option<[Cow<'static, str>; 10]>,
    /// The percent sign that `%` and `p` write.
    pub percent: Cow<'static, str>,
    /// The minus sign.
    pub minus: Cow<'static, str>,
    /// What NaN formats as.
    pub nan: Cow<'static, str>,
    /// `,` groups only values with at least this many integer digits; 0 or
    /// 1 always groups, as d3 does.
    pub group_from: usize,
}

/// A locale table of `&'static` parts, from which [`Locale::from_table`]
/// builds a constant.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Table {
    pub(crate) decimal: &'static str,
    pub(crate) thousands: &'static str,
    pub(crate) grouping: &'static [usize],
    pub(crate) currency: [&'static str; 2],
    pub(crate) numerals: Option<&'static [&'static str; 10]>,
    pub(crate) percent: &'static str,
    pub(crate) minus: &'static str,
    pub(crate) nan: &'static str,
    pub(crate) group_from: usize,
}

impl Default for Definition {
    fn default() -> Self {
        Definition {
            decimal: Cow::Borrowed("."),
            thousands: Cow::Borrowed(""),
            grouping: Cow::Borrowed(&[]),
            currency: [Cow::Borrowed(""), Cow::Borrowed("")],
            numerals: None,
            percent: Cow::Borrowed("%"),
            minus: Cow::Borrowed("\u{2212}"),
            nan: Cow::Borrowed("NaN"),
            group_from: 1,
        }
    }
}

impl Default for Locale {
    /// [`Locale::NEUTRAL`].
    fn default() -> Self {
        Locale::NEUTRAL
    }
}

impl Locale {
    /// SI style (CGPM 2003, Res. 10): ".", U+202F groups of three from five
    /// integer digits, no currency, U+2212. `Default` returns it.
    pub const NEUTRAL: Locale = Locale::from_table(Table {
        decimal: ".",
        thousands: "\u{202f}",
        grouping: &[3],
        currency: ["", ""],
        numerals: None,
        percent: "%",
        minus: "\u{2212}",
        nan: "NaN",
        group_from: 5,
    });

    /// The locale a definition states.
    pub fn new(definition: Definition) -> Locale {
        let Definition {
            decimal,
            thousands,
            grouping,
            currency,
            numerals,
            percent,
            minus,
            nan,
            group_from,
        } = definition;
        Locale {
            decimal,
            thousands,
            grouping,
            currency,
            numerals,
            percent,
            minus,
            nan,
            group_from,
        }
    }

    /// d3's `locale.format(specifier)`.
    pub fn format(&self, specifier: &str) -> Result<Format, InvalidFormat> {
        Ok(Format::new(self, specifier.parse()?))
    }

    /// d3's `locale.formatPrefix(specifier, value)`: fixed-point, scaled to
    /// `value`'s SI prefix.
    pub fn format_prefix(&self, specifier: &str, value: f64) -> Result<Format, InvalidFormat> {
        Ok(Format::prefixed(self, specifier.parse()?, value))
    }

    /// A constant locale from a table of static parts (`Locale::new` cannot
    /// be `const`: dropping a `Definition` is not).
    pub(crate) const fn from_table(table: Table) -> Locale {
        let numerals = match table.numerals {
            Some(n) => Some([
                Cow::Borrowed(n[0]),
                Cow::Borrowed(n[1]),
                Cow::Borrowed(n[2]),
                Cow::Borrowed(n[3]),
                Cow::Borrowed(n[4]),
                Cow::Borrowed(n[5]),
                Cow::Borrowed(n[6]),
                Cow::Borrowed(n[7]),
                Cow::Borrowed(n[8]),
                Cow::Borrowed(n[9]),
            ]),
            None => None,
        };
        Locale {
            decimal: Cow::Borrowed(table.decimal),
            thousands: Cow::Borrowed(table.thousands),
            grouping: Cow::Borrowed(table.grouping),
            currency: [
                Cow::Borrowed(table.currency[0]),
                Cow::Borrowed(table.currency[1]),
            ],
            numerals,
            percent: Cow::Borrowed(table.percent),
            minus: Cow::Borrowed(table.minus),
            nan: Cow::Borrowed(table.nan),
            group_from: table.group_from,
        }
    }

    /// d3's formatGroup: `digits` split into groups from the right, at most
    /// `width` characters long counting separators as one.
    pub(super) fn group(&self, digits: &str, width: usize) -> String {
        let Some(&first) = self.grouping.first() else {
            return digits.to_owned();
        };
        let chars: Vec<char> = digits.chars().collect();
        let mut groups = Vec::new();
        let (mut i, mut j, mut size, mut length) = (chars.len(), 0, first, 0usize);
        while i > 0 && size > 0 {
            if length.saturating_add(size).saturating_add(1) > width {
                size = width.saturating_sub(length).max(1);
            }
            let from = i.saturating_sub(size);
            groups.push(chars[from..i].iter().collect::<String>());
            i = from;
            length = length.saturating_add(size).saturating_add(1);
            if length > width {
                break;
            }
            j = (j + 1) % self.grouping.len();
            size = self.grouping[j];
        }
        groups.reverse();
        groups.join(&*self.thousands)
    }

    /// d3's formatNumerals: every ASCII digit replaced by the locale's.
    pub(super) fn numerals(&self, text: String) -> String {
        let Some(numerals) = &self.numerals else {
            return text;
        };
        let mut out = String::with_capacity(text.len());
        for c in text.chars() {
            match c
                .to_digit(10)
                .and_then(|d| usize::try_from(d).ok())
                .and_then(|d| numerals.get(d))
            {
                Some(numeral) => out.push_str(numeral),
                None => out.push(c),
            }
        }
        out
    }
}
