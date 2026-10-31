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
    pub const NEUTRAL: Locale = Locale::new(format::Locale::NEUTRAL, time::Locale::NEUTRAL);

    /// d3's default locale: commas in groups of three, a dollar currency;
    /// d3's en-US dates and d3's ladder.
    pub const EN_US: Locale = Locale::new(format::Locale::EN_US, time::Locale::EN_US);

    /// German (Germany): d3's de-DE number and time tables, the ISO ladder.
    pub const DE_DE: Locale = Locale::new(format::Locale::DE_DE, time::Locale::DE_DE);

    /// French (France): d3's fr-FR number and time tables, the ISO ladder.
    pub const FR_FR: Locale = Locale::new(format::Locale::FR_FR, time::Locale::FR_FR);

    /// Spanish (Spain): d3's es-ES number and time tables, the ISO ladder.
    pub const ES_ES: Locale = Locale::new(format::Locale::ES_ES, time::Locale::ES_ES);

    /// Italian (Italy): d3's it-IT number and time tables, the ISO ladder.
    pub const IT_IT: Locale = Locale::new(format::Locale::IT_IT, time::Locale::IT_IT);

    /// Portuguese (Brazil): d3's pt-BR number and time tables, the ISO
    /// ladder.
    pub const PT_BR: Locale = Locale::new(format::Locale::PT_BR, time::Locale::PT_BR);

    /// Dutch (Netherlands): d3's nl-NL number and time tables, the ISO
    /// ladder.
    pub const NL_NL: Locale = Locale::new(format::Locale::NL_NL, time::Locale::NL_NL);

    /// Polish (Poland): d3's pl-PL number and time tables, the ISO ladder.
    pub const PL_PL: Locale = Locale::new(format::Locale::PL_PL, time::Locale::PL_PL);

    /// Russian (Russia): d3's ru-RU number and time tables, the ISO ladder.
    pub const RU_RU: Locale = Locale::new(format::Locale::RU_RU, time::Locale::RU_RU);

    /// Japanese (Japan): d3's ja-JP number and time tables, the ISO ladder.
    pub const JA_JP: Locale = Locale::new(format::Locale::JA_JP, time::Locale::JA_JP);

    /// Chinese (China): d3's zh-CN number and time tables, the ISO ladder.
    pub const ZH_CN: Locale = Locale::new(format::Locale::ZH_CN, time::Locale::ZH_CN);

    /// The figure locale of two tables: numbers as `number` writes them,
    /// dates as `time` does. Every constant is such a pair; this one takes
    /// runtime tables, such as d3-time-format's without a constant.
    ///
    /// ```
    /// use rustd3_core::jiff::{Timestamp, tz::TimeZone};
    /// use rustd3_core::locale::Locale;
    /// use rustd3_core::scale::Ticks;
    /// use rustd3_core::time::{self, Definition, Ladder};
    /// use rustd3_core::format;
    ///
    /// let names = |s: &str| s.split(' ').map(|n| n.to_owned().into()).collect::<Vec<_>>();
    /// let nl = time::Locale::new(Definition {
    ///     date_time: "%a %e %B %Y %X".into(),
    ///     date: "%d-%m-%Y".into(),
    ///     time: "%H:%M:%S".into(),
    ///     periods: ["AM".into(), "PM".into()],
    ///     days: names("zondag maandag dinsdag woensdag donderdag vrijdag zaterdag").try_into().unwrap(),
    ///     short_days: names("zo ma di wo do vr za").try_into().unwrap(),
    ///     months: names("januari februari maart april mei juni juli augustus september oktober november december")
    ///         .try_into()
    ///         .unwrap(),
    ///     short_months: names("jan feb mrt apr mei jun jul aug sep okt nov dec").try_into().unwrap(),
    ///     ladder: Ladder::D3,
    /// });
    /// let locale = Locale::new(format::Locale::NL_NL, nl);
    /// let month: [Timestamp; 2] = ["2026-03-01T00:00:00Z".parse()?, "2026-04-01T00:00:00Z".parse()?];
    /// let t = time::time(TimeZone::UTC).domain(month);
    /// assert_eq!(t.labels(&t.tick_set(4), &locale)[..2], ["maart", "mrt 08"]);
    /// # Ok::<(), rustd3_core::jiff::Error>(())
    /// ```
    #[must_use]
    pub const fn new(number: format::Locale, time: time::Locale) -> Locale {
        Locale { number, time }
    }

    /// The locale with its time half `time` and its number half as it was:
    /// a constant writing dates on another ladder, or in another table.
    ///
    /// ```
    /// use rustd3_core::jiff::{Timestamp, tz::TimeZone};
    /// use rustd3_core::locale::Locale;
    /// use rustd3_core::scale::Ticks;
    /// use rustd3_core::time::{self, Ladder};
    ///
    /// let de = Locale::DE_DE.with_time(time::Locale::DE_DE.ladder(Ladder::D3));
    /// let span: [Timestamp; 2] = ["2026-10-29T00:00:00Z".parse()?, "2026-11-03T00:00:00Z".parse()?];
    /// let t = time::utc().domain(span);
    /// assert_eq!(t.labels(&t.tick_set(5), &de), ["Do 29", "Fr 30", "Sa 31", "November", "Mo 02", "Di 03"]);
    /// # Ok::<(), rustd3_core::jiff::Error>(())
    /// ```
    #[must_use]
    pub fn with_time(self, time: time::Locale) -> Locale {
        Locale { time, ..self }
    }

    /// The number half, d3's `formatLocale`.
    #[must_use]
    pub fn get_number(&self) -> &format::Locale {
        &self.number
    }

    /// The time half, d3's `timeFormatLocale` with the tick-label ladder.
    #[must_use]
    pub fn get_time(&self) -> &time::Locale {
        &self.time
    }
}
