//! Time: intervals in explicit zones, against d3-time 3.1.0 run in Node
//! under the zone of each row, and the port's own sub-millisecond rows.

use rustd3_core::jiff::Timestamp;
use rustd3_core::jiff::tz::TimeZone;
use rustd3_core::time::Interval;
use rustd3_testkit::zone;

fn at(s: &str) -> Timestamp {
    s.parse().expect("an RFC 3339 instant")
}

fn ms(ms: i64) -> Timestamp {
    Timestamp::from_millisecond(ms).expect("an instant in range")
}

fn range(interval: Interval, start: Timestamp, stop: Timestamp, tz: &TimeZone) -> Vec<Timestamp> {
    interval.range(start, stop, tz).collect()
}

#[test]
fn hours_floor_on_the_local_clock_of_half_and_quarter_hour_zones() {
    let t = ms(1_456_279_843_199);
    let kolkata = zone("Asia/Kolkata");
    assert_eq!(
        Interval::Hour(1).floor(t, &kolkata),
        Some(ms(1_456_277_400_000))
    );
    assert_eq!(
        Interval::Hour(1).ceil(t, &kolkata),
        Some(ms(1_456_281_000_000))
    );
    let kathmandu = zone("Asia/Kathmandu");
    assert_eq!(
        Interval::Hour(1).floor(t, &kathmandu),
        Some(ms(1_456_276_500_000))
    );
    assert_eq!(
        Interval::Hour(1).ceil(t, &kathmandu),
        Some(ms(1_456_280_100_000))
    );
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
