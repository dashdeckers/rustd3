//! [`Format`], a d3 time specifier compiled under a locale, and
//! [`format()`], the same under the neutral locale.

mod directive;
mod iso;
mod item;
mod pad;

use jiff::Timestamp;
use jiff::tz::TimeZone;

use crate::time::Locale;

pub(crate) use directive::Directive;
pub use iso::{iso_format, iso_parse};
pub(crate) use item::Item;
pub(crate) use pad::Pad;

/// A compiled d3 time specifier with its locale (d3's `locale.format`):
/// every directive of d3-time-format, `%L %f %Q %s %Z` included, plus `%N`
/// for nanoseconds. Instants format in the zone a call names; jiff supplies
/// the civil fields, once per formatted value.
///
/// Flags pad numbers: `%-d` none, `%_d` spaces, `%0d` zeros, the default
/// (spaces for `%e`). An unknown directive writes its letter, as d3 does,
/// so formatting cannot fail.
///
/// ```
/// use rustd3_core::jiff::{Timestamp, tz::TimeZone};
/// use rustd3_core::time;
///
/// let t: Timestamp = "2026-03-08T07:59:30.123456789Z".parse()?;
/// let la = TimeZone::posix("PST8PDT,M3.2.0,M11.1.0")?;
/// let f = time::format("%Y-%m-%d %H:%M:%S.%L %Z");
/// assert_eq!(f.format(t, &la), "2026-03-07 23:59:30.123 -0800");
/// assert_eq!(f.format(t, &TimeZone::UTC), "2026-03-08 07:59:30.123 +0000");
/// assert_eq!(time::format("%f %N %Q").format(t, &la), "123456 123456789 1772956770123");
/// assert_eq!(time::format("%e|%-e|%_H|%k").format(t, &la), " 7|7|23|k");
/// # Ok::<(), rustd3_core::jiff::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Format {
    items: Vec<Item>,
    locale: Locale,
}

impl Format {
    /// `spec` compiled under `locale`, its composites expanded.
    pub(crate) fn new(spec: &str, locale: Locale) -> Format {
        let mut items = Vec::new();
        compile(spec, &locale, &mut Vec::new(), &mut items);
        Format { items, locale }
    }

    /// `t` written in `zone`.
    pub fn format(&self, t: Timestamp, zone: &TimeZone) -> String {
        let at = t.to_zoned(zone.clone());
        let mut out = String::new();
        for item in &self.items {
            item.write(&at, &self.locale, &mut out);
        }
        out
    }
}

/// `spec` compiled under [`Locale::NEUTRAL`]: English names, ISO
/// composites, a 24-hour clock.
pub fn format(spec: &str) -> Format {
    Locale::NEUTRAL.format(spec)
}

/// Appends `spec`'s items to `items`, as d3's `newFormat` reads a
/// specifier: text; `%` with an optional pad flag and a letter; `%%` as a
/// percent sign; `%c %x %X` as the locale's patterns, compiled in turn; a
/// letter d3 does not know as text; a trailing `%` as nothing. A composite
/// inside its own expansion, where d3 recurses without end, writes its
/// letter; `expanding` holds the composites being expanded.
fn compile(spec: &str, locale: &Locale, expanding: &mut Vec<char>, items: &mut Vec<Item>) {
    let mut chars = spec.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            text(items, c);
            continue;
        }
        let mut letter = chars.next();
        let flag = letter.and_then(Pad::from_flag);
        if flag.is_some() {
            letter = chars.next();
        }
        let Some(letter) = letter else {
            break;
        };
        let pattern = match letter {
            'c' => Some(locale.date_time()),
            'x' => Some(locale.date()),
            'X' => Some(locale.time()),
            _ => None,
        };
        match (pattern, Directive::from_letter(letter)) {
            (Some(pattern), _) if !expanding.contains(&letter) => {
                expanding.push(letter);
                compile(pattern, locale, expanding, items);
                expanding.pop();
            }
            (_, Some(directive)) => {
                let default = if letter == 'e' { Pad::Space } else { Pad::Zero };
                items.push(Item::Directive(directive, flag.unwrap_or(default)));
            }
            (_, None) => text(items, letter),
        }
    }
}

/// Appends `c` to the text item `items` ends with, or as a new one.
fn text(items: &mut Vec<Item>, c: char) {
    match items.last_mut() {
        Some(Item::Literal(text)) => text.push(c),
        Some(Item::Directive(..)) | None => items.push(Item::Literal(c.to_string())),
    }
}
