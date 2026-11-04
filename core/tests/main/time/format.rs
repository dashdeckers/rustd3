//! Time formats: spot rows, computed with d3-time-format
//! 4.1.0 in JavaScript under `TZ=America/Los_Angeles`, and rustd3-core's own rows
//! (`%N`, composites that refer to themselves).

use std::borrow::Cow;

use rustd3_core::jiff::Timestamp;
use rustd3_core::jiff::civil::date;
use rustd3_core::jiff::tz::TimeZone;
use rustd3_core::time::{self, Definition, Ladder, Locale};
use rustd3_testkit::zone;

use super::at;

/// The section's `t`: local 2026-03-07 23:59:30.123 in Los Angeles, a
/// Saturday.
fn t() -> Timestamp {
    at("2026-03-08T07:59:30.123Z")
}

/// `spec` under d3's en-US of `t` in Los Angeles.
fn en_us(spec: &str, t: Timestamp) -> String {
    Locale::EN_US
        .format(spec)
        .format(t, &zone("America/Los_Angeles"))
}

/// Local midnight of January 1 of `year` in Los Angeles.
fn new_year(year: i16) -> Timestamp {
    zone("America/Los_Angeles")
        .to_timestamp(date(year, 1, 1).at(0, 0, 0, 0))
        .expect("a year in jiff's range")
}

#[test]
fn en_us_composites_match_d3() {
    assert_eq!(en_us("%c", t()), "3/7/2026, 11:59:30 PM");
    assert_eq!(en_us("%x %X", t()), "3/7/2026 11:59:30 PM");
}

#[test]
fn the_neutral_locale_writes_iso_composites_and_english_names() {
    let la = zone("America/Los_Angeles");
    assert_eq!(Locale::default(), Locale::NEUTRAL);
    assert_eq!(time::format("%c").format(t(), &la), "2026-03-07 23:59:30");
    assert_eq!(
        time::format("%x|%X").format(t(), &la),
        "2026-03-07|23:59:30"
    );
    assert_eq!(time::format("%a %d %b").format(t(), &la), "Sat 07 Mar");
}

#[test]
fn directives_match_d3_on_the_spot_instant() {
    let rows = [
        ("%Y-%m-%d %H:%M:%S.%L", "2026-03-07 23:59:30.123"),
        ("%f", "123000"),
        ("%Q", "1772956770123"),
        ("%s", "1772956770"),
        ("%Z", "-0800"),
        ("%j", "066"),
        ("%U", "09"),
        ("%W", "09"),
        ("%V", "10"),
        ("%u", "6"),
        ("%w", "6"),
        ("%G-%g", "2026-26"),
        ("%q", "1"),
        ("%e", " 7"),
        ("%-d", "7"),
        ("%_H", "23"),
        ("%y", "26"),
        ("%I %p", "11 PM"),
        ("%A %B %a %b", "Saturday March Sat Mar"),
    ];
    for (spec, expected) in rows {
        assert_eq!(en_us(spec, t()), expected, "{spec}");
    }
}

#[test]
fn nanoseconds_are_the_ports_own_directive() {
    let t = Timestamp::from_nanosecond(t().as_nanosecond() + 456_789).unwrap();
    assert_eq!(en_us("%N", t), "123456789");
    assert_eq!(en_us("%L %f", t), "123 123456");
    assert_eq!(en_us("%-N", Timestamp::from_nanosecond(5).unwrap()), "5");
}

#[test]
fn utc_formats_write_the_utc_fields() {
    let f = Locale::EN_US.format("%Y-%m-%dT%H:%M:%S.%LZ");
    assert_eq!(f.format(t(), &TimeZone::UTC), "2026-03-08T07:59:30.123Z");
    assert_eq!(en_us("%Z", t()), "-0800");
    assert_eq!(
        Locale::EN_US.format("%Z").format(t(), &TimeZone::UTC),
        "+0000"
    );
}

#[test]
fn years_keep_their_sign_and_four_digits() {
    assert_eq!(en_us("%y", new_year(-2)), "-02");
    assert_eq!(en_us("%Y", new_year(123)), "0123");
    assert_eq!(en_us("%Y", new_year(-2)), "-0002");
}

#[test]
fn iso_weeks_follow_d3s_thursdays() {
    let la = zone("America/Los_Angeles");
    let day = |y, m, d| la.to_timestamp(date(y, m, d).at(0, 0, 0, 0)).unwrap();
    assert_eq!(en_us("%V", day(2015, 12, 31)), "53");
    assert_eq!(en_us("%V", day(2016, 1, 1)), "53");
    assert_eq!(en_us("%G %g", day(2018, 12, 31)), "2019 19");
}

#[test]
fn offsets_east_of_utc_and_in_minutes_are_signed_hours_and_minutes() {
    let kathmandu = zone("Asia/Kathmandu");
    assert_eq!(Locale::EN_US.format("%Z").format(t(), &kathmandu), "+0545");
    // Los Angeles's local mean time, −07:52:58, truncated to minutes.
    assert_eq!(en_us("%Z", new_year(1800)), "-0752");
}

#[test]
fn unknown_directives_write_their_letter_and_a_trailing_percent_nothing() {
    assert_eq!(en_us("%k|%-k|%%|%-%", t()), "k|k|%|%");
    assert_eq!(en_us("a%", t()), "a");
    assert_eq!(en_us("a%-", t()), "a");
}

#[test]
fn a_composite_inside_its_own_expansion_writes_its_letter() {
    let names = |n: &[&'static str]| n.iter().map(|&s| Cow::Borrowed(s)).collect::<Vec<_>>();
    let looping = Locale::new(Definition {
        date_time: "%x %c".into(),
        date: "%Y %X".into(),
        time: "%x!".into(),
        periods: ["am".into(), "pm".into()],
        days: names(&["S", "M", "T", "W", "T", "F", "S"])
            .try_into()
            .unwrap(),
        short_days: names(&["S", "M", "T", "W", "T", "F", "S"])
            .try_into()
            .unwrap(),
        months: names(&[
            "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12",
        ])
        .try_into()
        .unwrap(),
        short_months: names(&[
            "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12",
        ])
        .try_into()
        .unwrap(),
        ladder: Ladder::Iso,
    });
    let utc = &TimeZone::UTC;
    assert_eq!(looping.format("%c").format(t(), utc), "2026 x! c");
    assert_eq!(looping.format("%X").format(t(), utc), "2026 X!");
}

#[test]
fn iso_format_writes_utc_to_the_millisecond_and_iso_parse_reads_it_back() {
    assert_eq!(time::iso_format(t()), "2026-03-08T07:59:30.123Z");
    let later = Timestamp::from_nanosecond(t().as_nanosecond() + 999_999).unwrap();
    assert_eq!(time::iso_format(later), "2026-03-08T07:59:30.123Z");
    assert_eq!(
        time::iso_parse("2026-03-07T23:59:30.123Z").ok(),
        Some(at("2026-03-07T23:59:30.123Z"))
    );
    assert_eq!(time::iso_parse("2026-03-08T07:59:30.123Z").ok(), Some(t()));
    assert_eq!(
        time::iso_parse("2026-03-07T23:59:30.123-08:00").ok(),
        Some(t())
    );
    assert!(time::iso_parse("2026-03-07T23:59:30").is_err());
}
