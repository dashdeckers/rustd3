//! Time: intervals in explicit zones, against d3-time 3.1.0 run in Node
//! under the zone of each row, and the port's own sub-millisecond rows.

use rustd3_core::jiff::Timestamp;
use rustd3_core::jiff::civil::{Weekday, date};
use rustd3_core::jiff::tz::TimeZone;
use rustd3_core::time::Interval;
use rustd3_testkit::zone;

fn at(s: &str) -> Timestamp {
    s.parse().expect("an RFC 3339 instant")
}

fn ms(ms: i64) -> Timestamp {
    Timestamp::from_millisecond(ms).expect("an instant in range")
}

/// `new Date(year, month − 1, day, hour)` in `tz`: a local time resolved
/// as `Date` resolves it.
fn local_hour(tz: &TimeZone, year: i16, month: i8, day: i8, hour: i8) -> Timestamp {
    tz.to_timestamp(date(year, month, day).at(hour, 0, 0, 0))
        .expect("a local date in range")
}

/// Local midnight of a civil date in `tz`.
fn local(tz: &TimeZone, year: i16, month: i8, day: i8) -> Timestamp {
    local_hour(tz, year, month, day, 0)
}

fn range(interval: Interval, start: Timestamp, stop: Timestamp, tz: &TimeZone) -> Vec<Timestamp> {
    interval.range(start, stop, tz).collect()
}

#[test]
fn hours_in_los_angeles_floor_across_daylight_saving_and_round_ties_up() {
    let la = zone("America/Los_Angeles");
    let hour = Interval::Hour(1);
    assert_eq!(
        hour.floor(at("2011-03-13T09:30:00Z"), &la),
        Some(at("2011-03-13T09:00:00Z"))
    );
    assert_eq!(
        hour.round(at("2011-01-01T08:30:00Z"), &la),
        Some(at("2011-01-01T09:00:00Z"))
    );
}

#[test]
fn every_third_hour_in_berlin_keeps_local_multiples_across_the_fall_back() {
    let berlin = zone("Europe/Berlin");
    let hours = range(
        Interval::Hour(3),
        at("2026-10-24T22:00:00Z"),
        at("2026-10-25T06:00:00Z"),
        &berlin,
    );
    assert_eq!(
        hours,
        [
            at("2026-10-24T22:00:00Z"),
            at("2026-10-25T02:00:00Z"),
            at("2026-10-25T05:00:00Z")
        ]
    );
}

#[test]
fn fixed_units_step_by_multiples_since_the_epoch() {
    let utc = TimeZone::UTC;
    let millis = range(Interval::Millisecond(50), ms(1_000), ms(1_200), &utc);
    assert_eq!(millis, [ms(1_000), ms(1_050), ms(1_100), ms(1_150)]);
    let t0 = at("2026-10-25T12:00:00Z");
    let stop = Timestamp::from_nanosecond(t0.as_nanosecond() + 3).unwrap();
    let nanos: Vec<i128> = range(Interval::Nanosecond(1), t0, stop, &utc)
        .iter()
        .map(|t| t.as_nanosecond() - t0.as_nanosecond())
        .collect();
    assert_eq!(nanos, [0, 1, 2]);
}

#[test]
fn every_returns_none_for_zero_and_a_zero_step_behaves_as_one() {
    assert_eq!(Interval::Minute(1).every(0), None);
    assert_eq!(Interval::Minute(1).every(15), Some(Interval::Minute(15)));
    let t = at("2026-10-25T12:34:56.789Z");
    let utc = TimeZone::UTC;
    assert_eq!(
        Interval::Second(0).floor(t, &utc),
        Interval::Second(1).floor(t, &utc)
    );
    assert_eq!(
        Interval::Millisecond(0).floor(t, &utc),
        Some(at("2026-10-25T12:34:56.789Z"))
    );
    assert_eq!(
        Interval::Minute(15).floor(t, &utc),
        Some(at("2026-10-25T12:30:00Z"))
    );
    assert_eq!(
        Interval::Minute(15).offset(at("2026-10-25T12:34:00Z"), -3, &utc),
        Some(at("2026-10-25T12:00:00Z"))
    );
}

#[test]
fn results_outside_jiffs_range_are_none_and_end_a_range() {
    let utc = TimeZone::UTC;
    assert_eq!(Interval::Hour(1).floor(Timestamp::MIN, &utc), None);
    assert_eq!(Interval::Second(1).offset(Timestamp::MAX, 1, &utc), None);
    assert_eq!(
        Interval::Nanosecond(1).count(Timestamp::MIN, Timestamp::MAX, &utc),
        None
    );
    let last = Interval::Second(1).floor(Timestamp::MAX, &utc).unwrap();
    let start = Timestamp::from_nanosecond(last.as_nanosecond() - 2_000_000_000).unwrap();
    assert_eq!(
        range(Interval::Second(1), start, Timestamp::MAX, &utc).len(),
        3
    );
}

#[test]
fn counts_divide_the_difference_of_the_floors() {
    let la = zone("America/Los_Angeles");
    let (start, end) = (at("2011-03-13T08:00:00Z"), at("2011-03-13T11:59:59Z"));
    assert_eq!(Interval::Hour(1).count(start, end, &la), Some(3));
    assert_eq!(Interval::Hour(6).count(start, end, &la), Some(3));
    assert_eq!(Interval::Minute(1).count(start, end, &la), Some(239));
    assert_eq!(Interval::Second(1).count(end, start, &la), Some(-14_399));
    assert_eq!(Interval::Millisecond(7).count(ms(0), ms(10), &la), Some(10));
}

#[test]
fn calendar_units_in_los_angeles_follow_d3() {
    let la = zone("America/Los_Angeles");
    let day = Interval::Day(1);
    assert_eq!(
        day.floor(at("2011-03-13T12:00:00Z"), &la),
        Some(at("2011-03-13T08:00:00Z"))
    );
    assert_eq!(
        day.offset(at("2011-03-12T08:00:00Z"), 1, &la),
        Some(at("2011-03-13T08:00:00Z"))
    );
    assert_eq!(
        day.count(local(&la, 2011, 1, 1), local(&la, 2011, 3, 14), &la),
        Some(72)
    );
    let month = Interval::Month(1);
    assert_eq!(
        month.count(local(&la, 2011, 1, 15), local(&la, 2012, 3, 2), &la),
        Some(14)
    );
    let sunday = Interval::Week(Weekday::Sunday, 1);
    assert_eq!(
        sunday.count(local(&la, 2011, 1, 1), local(&la, 2011, 12, 31), &la),
        Some(52)
    );
    let years = range(
        Interval::Year(5),
        local(&la, 2006, 1, 1),
        local(&la, 2026, 1, 1),
        &la,
    );
    assert_eq!(years, [2010, 2015, 2020, 2025].map(|y| local(&la, y, 1, 1)));
}

#[test]
fn calendar_offsets_carry_past_the_end_of_the_month_as_date_does() {
    let la = zone("America/Los_Angeles");
    let dec30 = local(&la, 2010, 12, 30);
    assert_eq!(
        Interval::Month(1).offset(dec30, 2, &la),
        Some(at("2011-03-02T08:00:00Z"))
    );
    let leap_noon = at("2012-02-29T20:00:00Z");
    assert_eq!(
        Interval::Year(1).offset(leap_noon, 1, &la),
        Some(at("2013-03-01T20:00:00Z"))
    );
    let nov6 = local(&la, 2011, 11, 6);
    assert_eq!(
        Interval::Month(5).floor(nov6, &la),
        Some(at("2011-11-01T07:00:00Z"))
    );
}

#[test]
fn stepped_days_keep_days_of_the_month_or_days_since_1970() {
    let utc = TimeZone::UTC;
    let (start, stop) = (at("2011-01-01T00:00:00Z"), at("2011-01-09T00:00:00Z"));
    let days = |interval| -> Vec<i8> {
        range(interval, start, stop, &utc)
            .iter()
            .map(|&t| utc.to_datetime(t).day())
            .collect()
    };
    assert_eq!(days(Interval::Day(2)), [1, 3, 5, 7]);
    assert_eq!(days(Interval::UnixDay(2)), [2, 4, 6, 8]);
}

#[test]
fn stepped_weeks_count_from_the_week_that_holds_the_epochs_local_date() {
    let berlin = zone("Europe/Berlin");
    let sundays = range(
        Interval::Week(Weekday::Sunday, 2),
        local(&berlin, 2026, 1, 1),
        local(&berlin, 2026, 3, 1),
        &berlin,
    );
    let expected = [
        "2026-01-10T23:00:00Z",
        "2026-01-24T23:00:00Z",
        "2026-02-07T23:00:00Z",
        "2026-02-21T23:00:00Z",
    ];
    assert_eq!(sundays, expected.map(at));
    let la = zone("America/Los_Angeles");
    let thursdays = range(
        Interval::Week(Weekday::Thursday, 3),
        local(&la, 1969, 12, 1),
        local(&la, 1970, 2, 1),
        &la,
    );
    let expected = [
        "1969-12-04T08:00:00Z",
        "1969-12-25T08:00:00Z",
        "1970-01-15T08:00:00Z",
    ];
    assert_eq!(thursdays, expected.map(at));
}

#[test]
fn offsets_beyond_any_calendar_are_none() {
    let utc = TimeZone::UTC;
    let t = at("2026-10-25T12:00:00Z");
    for interval in [
        Interval::Day(1),
        Interval::Week(Weekday::Monday, 1),
        Interval::Month(1),
        Interval::Year(7),
    ] {
        assert_eq!(interval.offset(t, i64::MAX, &utc), None);
        assert_eq!(interval.offset(t, i64::MIN, &utc), None);
    }
    assert_eq!(Interval::Millisecond(1).offset(t, i64::MIN, &utc), None);
}

/// The 32 active cases of d3-time 3.1.0's `test/test-time-zones`, with the
/// milliseconds Node prints in each zone.
#[test]
fn time_zones_script_cases() {
    let hour = Interval::Hour(1);
    let t = ms(1_456_279_843_199);
    let offsets: [(&str, i64, i64, [i64; 3]); 2] = [
        (
            "Asia/Kolkata",
            1_456_277_400_000,
            1_456_281_000_000,
            [1_320_604_200_000, 1_320_607_800_000, 1_320_611_400_000],
        ),
        (
            "Asia/Kathmandu",
            1_456_276_500_000,
            1_456_280_100_000,
            [1_320_603_300_000, 1_320_606_900_000, 1_320_610_500_000],
        ),
    ];
    for (name, floor, ceil, hours) in offsets {
        let tz = zone(name);
        assert_eq!(hour.floor(t, &tz), Some(ms(floor)), "{name} floor");
        assert_eq!(hour.ceil(t, &tz), Some(ms(ceil)), "{name} ceil");
        let (start, stop) = (
            local_hour(&tz, 2011, 11, 7, 0),
            local_hour(&tz, 2011, 11, 7, 3),
        );
        assert_eq!(range(hour, start, stop, &tz), hours.map(ms), "{name} range");
    }

    // São Paulo: daylight saving ends at midnight on 2016-02-21 and begins
    // at midnight on 2016-10-16.
    let tz = zone("America/Sao_Paulo");
    let day = Interval::Day(1);
    let utc = |month, day, hour| local_hour(&TimeZone::UTC, 2016, month, day, hour);
    struct Transition {
        month: i8,
        day: i8,
        floors: [i64; 4],
        ceils: [i64; 4],
        rounds: [i64; 4],
        range: [i64; 4],
    }
    let transitions = [
        Transition {
            month: 2,
            day: 21,
            floors: [
                1_455_933_600_000,
                1_455_933_600_000,
                1_456_023_600_000,
                1_456_023_600_000,
            ],
            ceils: [
                1_456_023_600_000,
                1_456_023_600_000,
                1_456_023_600_000,
                1_456_110_000_000,
            ],
            rounds: [
                1_455_933_600_000,
                1_456_023_600_000,
                1_456_023_600_000,
                1_456_110_000_000,
            ],
            range: [
                1_455_847_200_000,
                1_455_933_600_000,
                1_456_023_600_000,
                1_456_110_000_000,
            ],
        },
        Transition {
            month: 10,
            day: 16,
            floors: [
                1_476_500_400_000,
                1_476_500_400_000,
                1_476_586_800_000,
                1_476_586_800_000,
            ],
            ceils: [
                1_476_586_800_000,
                1_476_586_800_000,
                1_476_586_800_000,
                1_476_669_600_000,
            ],
            rounds: [
                1_476_500_400_000,
                1_476_586_800_000,
                1_476_586_800_000,
                1_476_669_600_000,
            ],
            range: [
                1_476_414_000_000,
                1_476_500_400_000,
                1_476_586_800_000,
                1_476_669_600_000,
            ],
        },
    ];
    for x in transitions {
        for (h, (floor, ceil)) in (1..=4).zip(x.floors.into_iter().zip(x.ceils)) {
            assert_eq!(
                day.floor(utc(x.month, x.day, h), &tz),
                Some(ms(floor)),
                "{}-{} {h}:00Z floor",
                x.month,
                x.day
            );
            assert_eq!(
                day.ceil(utc(x.month, x.day, h), &tz),
                Some(ms(ceil)),
                "{}-{} {h}:00Z ceil",
                x.month,
                x.day
            );
        }
        let probes = [(x.day - 1, 14), (x.day - 1, 15), (x.day, 14), (x.day, 15)];
        for ((d, h), round) in probes.into_iter().zip(x.rounds) {
            assert_eq!(
                day.round(utc(x.month, d, h), &tz),
                Some(ms(round)),
                "{}-{d} {h}:00Z round",
                x.month
            );
        }
        let (start, stop) = (
            local(&tz, 2016, x.month, x.day - 2),
            local(&tz, 2016, x.month, x.day + 2),
        );
        assert_eq!(
            range(day, start, stop, &tz),
            x.range.map(ms),
            "{} range",
            x.month
        );
    }
}

#[test]
fn sub_millisecond_intervals_step_exactly() {
    let utc = TimeZone::UTC;
    let t0 = at("2026-10-25T12:00:00.000000007Z").as_nanosecond();
    let ns = |n: i128| Timestamp::from_nanosecond(t0 + n).expect("an instant in 2026");
    let offsets = |interval, start, stop| -> Vec<i128> {
        range(interval, ns(start), ns(stop), &utc)
            .iter()
            .map(|t| t.as_nanosecond() - t0)
            .collect()
    };
    assert_eq!(offsets(Interval::Nanosecond(1), 0, 5), [0, 1, 2, 3, 4]);
    assert_eq!(offsets(Interval::Nanosecond(10), 0, 40), [3, 13, 23, 33]);
    assert_eq!(
        offsets(Interval::Microsecond(1), 0, 4_000),
        [993, 1_993, 2_993, 3_993]
    );
    assert_eq!(
        Interval::Microsecond(1).floor(ns(1_500), &utc),
        Some(ns(993))
    );
    assert_eq!(
        Interval::Nanosecond(1).offset(ns(0), 1_000_000_007, &utc),
        Some(ns(1_000_000_007))
    );
    // The same instants through an f64 epoch value lose their nanoseconds.
    let through_f64 = |n: i128| ((t0 + n) as f64) as i128 - t0;
    assert!((0..5).any(|n| through_f64(n) != n));
}

#[test]
fn interval_range_is_lazy() {
    let utc = TimeZone::UTC;
    let (start, stop) = (at("2026-10-25T00:00:00Z"), at("2026-10-26T00:00:00Z"));
    let mut nanoseconds = Interval::Nanosecond(1).range(start, stop, &utc);
    let first: Vec<i128> = nanoseconds
        .by_ref()
        .take(10)
        .map(|t| t.as_nanosecond() - start.as_nanosecond())
        .collect();
    assert_eq!(first, (0..10).collect::<Vec<i128>>());
    let eleventh = nanoseconds.next().expect("86.4e12 boundaries in a day");
    assert_eq!(eleventh.as_nanosecond() - start.as_nanosecond(), 10);
    // A range holds its state, not its boundaries.
    assert!(size_of_val(&nanoseconds) <= 128);
}
