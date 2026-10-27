//! [`Ladder`], the tick-label formats a locale climbs, and their rungs.

/// Which tick-label ladder a locale uses: the formats a time axis labels
/// its ticks with, one per unit, each tick taking the finest unit it is
/// not aligned to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ladder {
    /// Part I's neutral ladder: `%Y`, `%Y-%m`, `%m-%d`, `%H:%M`,
    /// `%H:%M:%S`, then `%S.%L`, `%S.%f` and `%S.%N` below the second.
    #[default]
    Iso,
    /// d3-scale's (`time.js:15–36`): `%Y`, `%B`, `%b %d`, `%a %d`, `%I %p`,
    /// `%I:%M`, `:%S`, `.%L`, extended with `.%f` and `.%N`.
    D3,
}

/// A ladder's formats, coarsest first: year, month, week, day, hour,
/// minute, second, millisecond, microsecond, nanosecond.
const ISO: [&str; 10] = [
    "%Y", "%Y-%m", "%m-%d", "%m-%d", "%H:%M", "%H:%M", "%H:%M:%S", "%S.%L", "%S.%f", "%S.%N",
];
/// d3's ladder, in [`ISO`]'s order.
const D3: [&str; 10] = [
    "%Y", "%B", "%b %d", "%a %d", "%I %p", "%I:%M", ":%S", ".%L", ".%f", ".%N",
];

/// A rung of a ladder: the unit a tick is labelled at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Rung {
    /// On a year's first instant.
    Year,
    /// On a month's first instant, not a year's.
    Month,
    /// On a week's first day, not a month's.
    Week,
    /// On a day's first instant, not a week's or a month's.
    Day,
    /// On the hour, not at a day's start.
    Hour,
    /// On the minute, not the hour.
    Minute,
    /// On the second, not the minute.
    Second,
    /// Off the second, on the millisecond.
    Millisecond,
    /// Off the millisecond, on the microsecond.
    Microsecond,
    /// Off the microsecond.
    Nanosecond,
}

impl Ladder {
    /// The ladder's ten formats, coarsest first.
    pub(crate) fn rungs(self) -> &'static [&'static str; 10] {
        match self {
            Ladder::Iso => &ISO,
            Ladder::D3 => &D3,
        }
    }

    /// The format of `rung`.
    pub(crate) fn rung(self, rung: Rung) -> &'static str {
        let index = match rung {
            Rung::Year => 0,
            Rung::Month => 1,
            Rung::Week => 2,
            Rung::Day => 3,
            Rung::Hour => 4,
            Rung::Minute => 5,
            Rung::Second => 6,
            Rung::Millisecond => 7,
            Rung::Microsecond => 8,
            Rung::Nanosecond => 9,
        };
        self.rungs()[index]
    }
}
