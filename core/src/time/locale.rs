//! [`Locale`], d3's timeFormatLocale plus the tick-label ladder, and the
//! crate-private [`Table`] its constants are built from.

mod ladder;

use std::borrow::Cow;

use crate::time::{Definition, Format};

pub use ladder::Ladder;
pub(crate) use ladder::Rung;

/// d3's timeFormatLocale plus the tick-label ladder: the patterns `%c`,
/// `%x` and `%X` expand to, the names of periods, weekdays and months, and
/// the [`Ladder`] tick labels climb.
///
/// `NEUTRAL`, the default, writes English names, ISO composites and a
/// 24-hour clock and labels ticks with the ISO ladder. d3-time-format's
/// tables for en-US and ten languages are constants generated from its
/// JSON: `EN_US`, d3's default definition with d3's ladder, and `DE_DE`,
/// `FR_FR`, `ES_ES`, `IT_IT`, `PT_BR`, `NL_NL`, `PL_PL`, `RU_RU`, `JA_JP`
/// and `ZH_CN` with the ISO ladder. Its other 24 tables load at run time
/// through [`Locale::new`].
///
/// ```
/// use rustd3_core::jiff::{Timestamp, tz::TimeZone};
/// use rustd3_core::time::Locale;
///
/// let t: Timestamp = "2026-03-08T07:59:30.123Z".parse()?;
/// let la = TimeZone::posix("PST8PDT,M3.2.0,M11.1.0")?;
/// assert_eq!(Locale::NEUTRAL.format("%c").format(t, &la), "2026-03-07 23:59:30");
/// assert_eq!(Locale::EN_US.format("%c").format(t, &la), "3/7/2026, 11:59:30 PM");
/// assert_eq!(Locale::DE_DE.format("%a %d %b").format(t, &la), "Sa 07 Mrz");
/// assert_eq!(Locale::default().format("%a %d %b").format(t, &la), "Sat 07 Mar");
/// # Ok::<(), rustd3_core::jiff::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Locale {
    date_time: Cow<'static, str>,
    date: Cow<'static, str>,
    time: Cow<'static, str>,
    periods: [Cow<'static, str>; 2],
    days: [Cow<'static, str>; 7],
    short_days: [Cow<'static, str>; 7],
    months: [Cow<'static, str>; 12],
    short_months: [Cow<'static, str>; 12],
    ladder: Ladder,
}

/// A locale table of `&'static` parts, from which [`Locale::from_table`]
/// builds a constant.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Table {
    pub(crate) date_time: &'static str,
    pub(crate) date: &'static str,
    pub(crate) time: &'static str,
    pub(crate) periods: [&'static str; 2],
    pub(crate) days: [&'static str; 7],
    pub(crate) short_days: [&'static str; 7],
    pub(crate) months: [&'static str; 12],
    pub(crate) short_months: [&'static str; 12],
    pub(crate) ladder: Ladder,
}

/// English weekday names, Sunday first.
const DAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
/// English month names.
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
/// English weekday abbreviations, Sunday first.
const SHORT_DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
/// English month abbreviations.
const SHORT_MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

impl Default for Locale {
    /// [`Locale::NEUTRAL`].
    fn default() -> Self {
        Locale::NEUTRAL
    }
}

impl Locale {
    /// English names, ISO composites (`%c` as `%Y-%m-%d %H:%M:%S`, `%x` as
    /// `%Y-%m-%d`, `%X` as `%H:%M:%S`), a 24-hour clock and the ISO ladder.
    /// `Default` returns it.
    pub const NEUTRAL: Locale = Locale::from_table(Table {
        date_time: "%x %X",
        date: "%Y-%m-%d",
        time: "%H:%M:%S",
        periods: ["AM", "PM"],
        days: DAYS,
        short_days: SHORT_DAYS,
        months: MONTHS,
        short_months: SHORT_MONTHS,
        ladder: Ladder::Iso,
    });

    /// The locale a definition states (d3's `timeFormatLocale`).
    pub fn new(definition: Definition) -> Locale {
        let Definition {
            date_time,
            date,
            time,
            periods,
            days,
            short_days,
            months,
            short_months,
            ladder,
        } = definition;
        Locale {
            date_time,
            date,
            time,
            periods,
            days,
            short_days,
            months,
            short_months,
            ladder,
        }
    }

    /// d3's `locale.format(specifier)`: `spec` compiled under this locale.
    /// Formatting cannot fail: an unknown directive writes its letter, as
    /// d3 does.
    pub fn format(&self, spec: &str) -> Format {
        Format::new(spec, self.clone())
    }

    /// A constant locale from a table of static parts (`Locale::new` cannot
    /// be `const`: dropping a `Definition` is not).
    pub(crate) const fn from_table(table: Table) -> Locale {
        Locale {
            date_time: Cow::Borrowed(table.date_time),
            date: Cow::Borrowed(table.date),
            time: Cow::Borrowed(table.time),
            periods: [
                Cow::Borrowed(table.periods[0]),
                Cow::Borrowed(table.periods[1]),
            ],
            days: week(table.days),
            short_days: week(table.short_days),
            months: year(table.months),
            short_months: year(table.short_months),
            ladder: table.ladder,
        }
    }

    /// The pattern `%c` expands to.
    pub(crate) fn date_time(&self) -> &str {
        &self.date_time
    }

    /// The pattern `%x` expands to.
    pub(crate) fn date(&self) -> &str {
        &self.date
    }

    /// The pattern `%X` expands to.
    pub(crate) fn time(&self) -> &str {
        &self.time
    }

    /// The name of the morning (`false`) or afternoon (`true`) period.
    pub(crate) fn period(&self, afternoon: bool) -> &str {
        &self.periods[usize::from(afternoon)]
    }

    /// A weekday's name, Sunday 0, abbreviated or not.
    pub(crate) fn weekday(&self, day: usize, short: bool) -> &str {
        if short {
            &self.short_days[day]
        } else {
            &self.days[day]
        }
    }

    /// A month's name, January 0, abbreviated or not.
    pub(crate) fn month(&self, month: usize, short: bool) -> &str {
        if short {
            &self.short_months[month]
        } else {
            &self.months[month]
        }
    }

    /// The ladder tick labels climb.
    pub(crate) fn ladder(&self) -> Ladder {
        self.ladder
    }
}

/// Seven static names as borrowed `Cow`s.
const fn week(names: [&'static str; 7]) -> [Cow<'static, str>; 7] {
    [
        Cow::Borrowed(names[0]),
        Cow::Borrowed(names[1]),
        Cow::Borrowed(names[2]),
        Cow::Borrowed(names[3]),
        Cow::Borrowed(names[4]),
        Cow::Borrowed(names[5]),
        Cow::Borrowed(names[6]),
    ]
}

/// Twelve static names as borrowed `Cow`s.
const fn year(names: [&'static str; 12]) -> [Cow<'static, str>; 12] {
    [
        Cow::Borrowed(names[0]),
        Cow::Borrowed(names[1]),
        Cow::Borrowed(names[2]),
        Cow::Borrowed(names[3]),
        Cow::Borrowed(names[4]),
        Cow::Borrowed(names[5]),
        Cow::Borrowed(names[6]),
        Cow::Borrowed(names[7]),
        Cow::Borrowed(names[8]),
        Cow::Borrowed(names[9]),
        Cow::Borrowed(names[10]),
        Cow::Borrowed(names[11]),
    ]
}
