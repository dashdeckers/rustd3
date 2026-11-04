//! [`Directive`], one of d3's format directives and what it writes
//! (`locale.js:57–91`, helpers `400–697`), on jiff's civil fields and the
//! crate's intervals.

use jiff::Zoned;
use jiff::civil::Weekday;

use crate::time::format::Pad;
use crate::time::{self, Interval, Locale};

/// A directive other than the composites `%c %x %X`, which a format
/// expands, and `%%`, which it reads as text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Directive {
    /// `%a`: the abbreviated weekday name.
    ShortWeekday,
    /// `%A`: the weekday name.
    Weekday,
    /// `%b`: the abbreviated month name.
    ShortMonth,
    /// `%B`: the month name.
    Month,
    /// `%d`, `%e`: the day of the month, 2 digits.
    DayOfMonth,
    /// `%f`: microseconds of the second, 6 digits.
    Microsecond,
    /// `%g`: the ISO 8601 week-based year mod 100, 2 digits.
    IsoYearOfCentury,
    /// `%G`: the ISO 8601 week-based year, 4 digits.
    IsoYear,
    /// `%H`: the hour of a 24-hour clock, 2 digits.
    Hour24,
    /// `%I`: the hour of a 12-hour clock, 2 digits.
    Hour12,
    /// `%j`: the day of the year, 3 digits.
    DayOfYear,
    /// `%L`: milliseconds of the second, 3 digits.
    Millisecond,
    /// `%m`: the month number, 2 digits.
    MonthNumber,
    /// `%M`: the minute, 2 digits.
    Minute,
    /// `%N`: nanoseconds of the second, 9 digits (jiff's meaning; d3 has
    /// no `%N`).
    Nanosecond,
    /// `%p`: the period, AM or PM.
    Period,
    /// `%q`: the quarter of the year, 1 to 4.
    Quarter,
    /// `%Q`: milliseconds since the epoch, floored.
    EpochMillisecond,
    /// `%s`: seconds since the epoch, floored.
    EpochSecond,
    /// `%S`: the second, 2 digits.
    Second,
    /// `%u`: the weekday from Monday 1 to Sunday 7.
    WeekdayFromMonday,
    /// `%U`: the week of the year, weeks starting on Sunday, 2 digits.
    WeekFromSunday,
    /// `%V`: the ISO 8601 week of the year, 2 digits.
    IsoWeek,
    /// `%w`: the weekday from Sunday 0 to Saturday 6.
    WeekdayFromSunday,
    /// `%W`: the week of the year, weeks starting on Monday, 2 digits.
    WeekFromMonday,
    /// `%y`: the year mod 100 with its sign, 2 digits.
    YearOfCentury,
    /// `%Y`: the year with its sign, 4 digits.
    Year,
    /// `%Z`: the offset from UTC as ±HHMM, truncated to minutes.
    Zone,
}

impl Directive {
    /// The directive a letter names; `None` for the composites, `%` and
    /// letters d3 does not know (a format writes those as text).
    pub(crate) fn from_letter(c: char) -> Option<Directive> {
        Some(match c {
            'a' => Directive::ShortWeekday,
            'A' => Directive::Weekday,
            'b' => Directive::ShortMonth,
            'B' => Directive::Month,
            'd' | 'e' => Directive::DayOfMonth,
            'f' => Directive::Microsecond,
            'g' => Directive::IsoYearOfCentury,
            'G' => Directive::IsoYear,
            'H' => Directive::Hour24,
            'I' => Directive::Hour12,
            'j' => Directive::DayOfYear,
            'L' => Directive::Millisecond,
            'm' => Directive::MonthNumber,
            'M' => Directive::Minute,
            'N' => Directive::Nanosecond,
            'p' => Directive::Period,
            'q' => Directive::Quarter,
            'Q' => Directive::EpochMillisecond,
            's' => Directive::EpochSecond,
            'S' => Directive::Second,
            'u' => Directive::WeekdayFromMonday,
            'U' => Directive::WeekFromSunday,
            'V' => Directive::IsoWeek,
            'w' => Directive::WeekdayFromSunday,
            'W' => Directive::WeekFromMonday,
            'y' => Directive::YearOfCentury,
            'Y' => Directive::Year,
            'Z' => Directive::Zone,
            _ => return None,
        })
    }

    /// Writes the directive for the instant `at` in its zone, with `pad`,
    /// names from `locale`. Week numbers and ISO years whose interval
    /// boundary leaves jiff's range (only in years −9999 and 9999) write
    /// `NaN`, as d3 writes an Invalid Date's.
    pub(crate) fn write(self, pad: Pad, at: &Zoned, locale: &Locale, out: &mut String) {
        let local = at.datetime();
        let weekday = local.weekday().to_sunday_zero_offset();
        let month = local.month();
        let ns = at.timestamp().as_nanosecond();
        let mut number = |value: i64, width: usize| pad.write(Some(value), width, out);
        match self {
            Directive::ShortWeekday => out.push_str(locale.weekday(index(weekday), true)),
            Directive::Weekday => out.push_str(locale.weekday(index(weekday), false)),
            Directive::ShortMonth => out.push_str(locale.month(index(month - 1), true)),
            Directive::Month => out.push_str(locale.month(index(month - 1), false)),
            Directive::DayOfMonth => number(local.day().into(), 2),
            Directive::Microsecond => number((local.subsec_nanosecond() / 1_000).into(), 6),
            Directive::IsoYearOfCentury => pad.write(iso_year(at).map(|y| y % 100), 2, out),
            Directive::IsoYear => pad.write(iso_year(at).map(|y| y % 10_000), 4, out),
            Directive::Hour24 => number(local.hour().into(), 2),
            Directive::Hour12 => number(twelve(local.hour()).into(), 2),
            Directive::DayOfYear => pad.write(day_of_year(at), 3, out),
            Directive::Millisecond => number((local.subsec_nanosecond() / 1_000_000).into(), 3),
            Directive::MonthNumber => number(month.into(), 2),
            Directive::Minute => number(local.minute().into(), 2),
            Directive::Nanosecond => number(local.subsec_nanosecond().into(), 9),
            Directive::Period => out.push_str(locale.period(local.hour() >= 12)),
            Directive::Quarter => out.push_str(&(1 + (month - 1) / 3).to_string()),
            Directive::EpochMillisecond => out.push_str(&ns.div_euclid(1_000_000).to_string()),
            Directive::EpochSecond => out.push_str(&ns.div_euclid(1_000_000_000).to_string()),
            Directive::Second => number(local.second().into(), 2),
            Directive::WeekdayFromMonday => {
                out.push_str(&(if weekday == 0 { 7 } else { weekday }).to_string());
            }
            Directive::WeekFromSunday => pad.write(week_of_year(at, Weekday::Sunday), 2, out),
            Directive::IsoWeek => pad.write(iso_week(at), 2, out),
            Directive::WeekdayFromSunday => out.push_str(&weekday.to_string()),
            Directive::WeekFromMonday => pad.write(week_of_year(at, Weekday::Monday), 2, out),
            Directive::YearOfCentury => number((local.year() % 100).into(), 2),
            Directive::Year => number((local.year() % 10_000).into(), 4),
            Directive::Zone => zone(at, out),
        }
    }
}

/// A weekday or month number from 0 as an index into a locale's names.
fn index(n: i8) -> usize {
    usize::try_from(n).expect("invariant: weekday and month indices are 0 to 11")
}

/// The hour of a 12-hour clock: 12, then 1 to 11.
fn twelve(hour: i8) -> i8 {
    match hour % 12 {
        0 => 12,
        h => h,
    }
}

/// `%Z` (`formatZone`): the sign of the offset east of UTC (+ for none),
/// then its whole hours and minutes, two digits each.
fn zone(at: &Zoned, out: &mut String) {
    let minutes = at.offset().seconds() / 60;
    out.push(if minutes < 0 { '-' } else { '+' });
    let minutes = i64::from(minutes.unsigned_abs());
    Pad::Zero.write(Some(minutes / 60), 2, out);
    Pad::Zero.write(Some(minutes % 60), 2, out);
}

/// `%j` (`formatDayOfYear`): 1 + local days since the year's first.
fn day_of_year(at: &Zoned) -> Option<i64> {
    let (t, tz) = (at.timestamp(), at.time_zone());
    Some(1 + Interval::Day(1).count(Interval::Year(1).floor(t, tz)?, t, tz)?)
}

/// `%U` and `%W` (`formatWeekNumberSunday`, `…Monday`): weeks starting on
/// `first` from the one that holds the eve of the year's first day.
fn week_of_year(at: &Zoned, first: Weekday) -> Option<i64> {
    let (t, tz) = (at.timestamp(), at.time_zone());
    let year = Interval::Year(1).floor(t, tz)?;
    let eve = time::at(year.as_nanosecond() - 1)?;
    Interval::Week(first, 1).count(eve, t, tz)
}

/// `dISO`: the Thursday of `at`'s ISO week, the one before for Thursday to
/// Sunday, the next for Monday to Wednesday.
fn iso_thursday(at: &Zoned) -> Option<jiff::Timestamp> {
    let (t, tz) = (at.timestamp(), at.time_zone());
    let thursday = Interval::Week(Weekday::Thursday, 1);
    match at.weekday() {
        Weekday::Thursday | Weekday::Friday | Weekday::Saturday | Weekday::Sunday => {
            thursday.floor(t, tz)
        }
        Weekday::Monday | Weekday::Tuesday | Weekday::Wednesday => thursday.ceil(t, tz),
    }
}

/// `%V` (`formatWeekNumberISO`): Thursdays from the year of `at`'s ISO
/// Thursday to it, plus one when that year starts on a Thursday.
fn iso_week(at: &Zoned) -> Option<i64> {
    let tz = at.time_zone();
    let thursday = iso_thursday(at)?;
    let year = Interval::Year(1).floor(thursday, tz)?;
    let weeks = Interval::Week(Weekday::Thursday, 1).count(year, thursday, tz)?;
    Some(weeks + i64::from(tz.to_datetime(year).weekday() == Weekday::Thursday))
}

/// `%G`'s year: the local year of `at`'s ISO Thursday.
fn iso_year(at: &Zoned) -> Option<i64> {
    Some(at.time_zone().to_datetime(iso_thursday(at)?).year().into())
}
