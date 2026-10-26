//! Option A for time locales: an optional table of
//! names. Without one, formatting is pure jiff plus
//! ISO meanings for %c, %x and %X.

use jiff::civil::DateTime;

/// d3's `timeFormatLocale` definition, as constants.
pub struct TimeLocale {
    pub date_time: &'static str,
    pub date: &'static str,
    pub time: &'static str,
    pub periods: [&'static str; 2],
    pub days: [&'static str; 7],
    pub short_days: [&'static str; 7],
    pub months: [&'static str; 12],
    pub short_months: [&'static str; 12],
}

/// One table, converted from d3-time-format's
/// de-DE.json; the other 33 are the same shape.
pub const DE_DE: TimeLocale = TimeLocale {
    date_time: "%A, der %e. %B %Y, %X",
    date: "%d.%m.%Y",
    time: "%H:%M:%S",
    periods: ["AM", "PM"],
    days: [
        "Sonntag", "Montag", "Dienstag", "Mittwoch",
        "Donnerstag", "Freitag", "Samstag",
    ],
    short_days: ["So", "Mo", "Di", "Mi", "Do", "Fr", "Sa"],
    months: [
        "Januar", "Februar", "März", "April", "Mai", "Juni",
        "Juli", "August", "September", "Oktober",
        "November", "Dezember",
    ],
    short_months: [
        "Jan", "Feb", "Mrz", "Apr", "Mai", "Jun", "Jul",
        "Aug", "Sep", "Okt", "Nov", "Dez",
    ],
};

/// Formats `dt` with d3 specifiers. Names come from
/// `locale` when given, else from jiff (English).
pub fn format(
    dt: DateTime,
    spec: &str,
    locale: Option<&TimeLocale>,
) -> String {
    dt.strftime(&rewrite(dt, spec, locale)).to_string()
}

/// Replaces the locale-dependent directives with
/// literal text or with other directives; everything
/// else passes to jiff untouched.
fn rewrite(
    dt: DateTime,
    spec: &str,
    loc: Option<&TimeLocale>,
) -> String {
    let mut out = String::with_capacity(spec.len());
    let mut chars = spec.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        let Some(d) = chars.next() else {
            out.push('%');
            break;
        };
        // ISO without a locale; the locale's own
        // patterns with one.
        let nested = match (d, loc) {
            ('c', None) => Some("%F %T"),
            ('x', None) => Some("%F"),
            ('X', None) => Some("%T"),
            ('c', Some(l)) => Some(l.date_time),
            ('x', Some(l)) => Some(l.date),
            ('X', Some(l)) => Some(l.time),
            _ => None,
        };
        if let Some(n) = nested {
            out += &rewrite(dt, n, loc);
            continue;
        }
        let name = loc.and_then(|l| {
            let wd = dt.weekday().to_sunday_zero_offset() as usize;
            let mo = dt.month() as usize - 1;
            Some(match d {
                'a' => l.short_days[wd],
                'A' => l.days[wd],
                'b' => l.short_months[mo],
                'B' => l.months[mo],
                'p' => l.periods[usize::from(dt.hour() >= 12)],
                _ => return None,
            })
        });
        match name {
            // Literal text: escape any % in it.
            Some(s) => out += &s.replace('%', "%%"),
            None => {
                out.push('%');
                out.push(d);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    #[test]
    fn neutral_and_german() {
        let dt = date(2026, 3, 7).at(23, 59, 30, 0);
        let f = |s, l| format(dt, s, l);
        assert_eq!(f("%Y-%m-%d %H:%M", None), "2026-03-07 23:59");
        assert_eq!(f("%c", None), "2026-03-07 23:59:30");
        assert_eq!(f("%a %d %b", None), "Sat 07 Mar");
        assert_eq!(f("%a %d %b", Some(&DE_DE)), "Sa 07 Mrz");
        assert_eq!(
            f("%c", Some(&DE_DE)),
            "Samstag, der  7. März 2026, 23:59:30"
        );
    }
}
