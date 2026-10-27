//! The figure locale: what labels are written in, one value for every
//! label a scale, an axis or a figure formats: numbers through its number
//! half, d3's formatLocale, and dates through its time half, d3's
//! timeFormatLocale with the tick-label ladder.
//!
//! ```
//! use rustd3_core::jiff::{Timestamp, tz::TimeZone};
//! use rustd3_core::locale::Locale;
//! use rustd3_core::scale::{self, Ticks};
//! use rustd3_core::time;
//!
//! let a = scale::linear().domain([0.0, 12000.0]);
//! let set = a.tick_set(5);
//! assert_eq!(a.labels(&set, &Locale::NEUTRAL)[1], "2\u{202f}000");
//! assert_eq!(a.labels(&set, &Locale::EN_US)[1], "2,000");
//! assert_eq!(a.labels(&set, &Locale::DE_DE)[1], "2.000");
//!
//! let day: [Timestamp; 2] = ["2026-03-01T00:00:00Z".parse()?, "2026-04-01T00:00:00Z".parse()?];
//! let t = time::time(TimeZone::UTC).domain(day);
//! let set = t.tick_set(4);
//! assert_eq!(t.labels(&set, &Locale::NEUTRAL)[..2], ["2026-03", "03-08"]);
//! assert_eq!(t.labels(&set, &Locale::EN_US)[..2], ["March", "Mar 08"]);
//! assert_eq!(t.labels(&set, &Locale::DE_DE)[..2], ["2026-03", "03-08"]);
//! # Ok::<(), rustd3_core::jiff::Error>(())
//! ```

use crate::{format, time};

/// The figure locale: the number and time formats of its labels.
#[derive(Debug, Clone, PartialEq)]
pub struct Locale {
    number: format::Locale,
    time: time::Locale,
}

impl Default for Locale {
    /// [`Locale::NEUTRAL`].
    fn default() -> Self {
        Locale::NEUTRAL
    }
}

impl From<format::Locale> for Locale {
    /// The figure locale whose labels write numbers as `number` does, and
    /// dates as the neutral locale: a runtime table, such as one of
    /// d3-format's, labelling an axis.
    ///
    /// ```
    /// use std::borrow::Cow;
    ///
    /// use rustd3_core::format::{self, Definition};
    /// use rustd3_core::locale::Locale;
    /// use rustd3_core::scale::{self, Ticks};
    ///
    /// let de = Locale::from(format::Locale::new(Definition {
    ///     decimal: Cow::Borrowed(","),
    ///     thousands: Cow::Borrowed("."),
    ///     grouping: Cow::Borrowed(&[3]),
    ///     ..Definition::default()
    /// }));
    /// let x = scale::linear().domain([0.0, 12000.0]);
    /// assert_eq!(x.labels(&x.tick_set(5), &de)[1], "2.000");
    /// ```
    fn from(number: format::Locale) -> Self {
        Locale {
            number,
            time: time::Locale::NEUTRAL,
        }
    }
}

impl Locale {
    /// SI style: a point, U+202F groups of three from five integer digits,
    /// U+2212 for minus; English names, ISO dates and the ISO ladder.
    /// `Default` returns it.
    pub const NEUTRAL: Locale = Locale::pair(format::Locale::NEUTRAL, time::Locale::NEUTRAL);

    /// d3's default locale: commas in groups of three, a dollar currency;
    /// d3's en-US dates and d3's ladder.
    pub const EN_US: Locale = Locale::pair(format::Locale::EN_US, time::Locale::EN_US);

    /// German (Germany): d3's de-DE number and time tables, the ISO ladder.
    pub const DE_DE: Locale = Locale::pair(format::Locale::DE_DE, time::Locale::DE_DE);

    /// French (France): d3's fr-FR number and time tables, the ISO ladder.
    pub const FR_FR: Locale = Locale::pair(format::Locale::FR_FR, time::Locale::FR_FR);

    /// Spanish (Spain): d3's es-ES number and time tables, the ISO ladder.
    pub const ES_ES: Locale = Locale::pair(format::Locale::ES_ES, time::Locale::ES_ES);

    /// Italian (Italy): d3's it-IT number and time tables, the ISO ladder.
    pub const IT_IT: Locale = Locale::pair(format::Locale::IT_IT, time::Locale::IT_IT);

    /// Portuguese (Brazil): d3's pt-BR number and time tables, the ISO
    /// ladder.
    pub const PT_BR: Locale = Locale::pair(format::Locale::PT_BR, time::Locale::PT_BR);

    /// Dutch (Netherlands): d3's nl-NL number and time tables, the ISO
    /// ladder.
    pub const NL_NL: Locale = Locale::pair(format::Locale::NL_NL, time::Locale::NL_NL);

    /// Polish (Poland): d3's pl-PL number and time tables, the ISO ladder.
    pub const PL_PL: Locale = Locale::pair(format::Locale::PL_PL, time::Locale::PL_PL);

    /// Russian (Russia): d3's ru-RU number and time tables, the ISO ladder.
    pub const RU_RU: Locale = Locale::pair(format::Locale::RU_RU, time::Locale::RU_RU);

    /// Japanese (Japan): d3's ja-JP number and time tables, the ISO ladder.
    pub const JA_JP: Locale = Locale::pair(format::Locale::JA_JP, time::Locale::JA_JP);

    /// Chinese (China): d3's zh-CN number and time tables, the ISO ladder.
    pub const ZH_CN: Locale = Locale::pair(format::Locale::ZH_CN, time::Locale::ZH_CN);

    /// The number half, d3's `formatLocale`.
    pub fn number(&self) -> &format::Locale {
        &self.number
    }

    /// The time half, d3's `timeFormatLocale` with the tick-label ladder.
    pub fn time(&self) -> &time::Locale {
        &self.time
    }

    /// The constant of two tables.
    const fn pair(number: format::Locale, time: time::Locale) -> Locale {
        Locale { number, time }
    }
}
