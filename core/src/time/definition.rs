//! [`Definition`], what a time locale table states.

use std::borrow::Cow;

use crate::time::Ladder;

/// The fields of d3's timeFormatLocale definition, plus the ladder: what
/// [`Locale::new`](crate::time::Locale::new) builds a locale from. d3-time-format's
/// JSON tables map onto it key by key (`dateTime` is `date_time`,
/// `shortDays` is `short_days`, `shortMonths` is `short_months`); d3's
/// tables label ticks with [`Ladder::D3`].
///
/// ```
/// use rustd3_core::jiff::{Timestamp, tz::TimeZone};
/// use rustd3_core::time::{Definition, Ladder, Locale};
///
/// let names = |s: &str| s.split(' ').map(|n| n.to_owned().into()).collect::<Vec<_>>();
/// let nl = Locale::new(Definition {
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
/// let t: Timestamp = "2026-03-07T23:59:30Z".parse()?;
/// assert_eq!(nl.format("%x %A").format(t, &TimeZone::UTC), "07-03-2026 zaterdag");
/// # Ok::<(), rustd3_core::jiff::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    /// The pattern `%c` writes (d3's `dateTime`).
    pub date_time: Cow<'static, str>,
    /// The pattern `%x` writes.
    pub date: Cow<'static, str>,
    /// The pattern `%X` writes.
    pub time: Cow<'static, str>,
    /// What `%p` writes before noon and from noon.
    pub periods: [Cow<'static, str>; 2],
    /// Weekday names, Sunday first (`%A`).
    pub days: [Cow<'static, str>; 7],
    /// Abbreviated weekday names, Sunday first (`%a`).
    pub short_days: [Cow<'static, str>; 7],
    /// Month names (`%B`).
    pub months: [Cow<'static, str>; 12],
    /// Abbreviated month names (`%b`).
    pub short_months: [Cow<'static, str>; 12],
    /// The formats tick labels climb.
    pub ladder: Ladder,
}
