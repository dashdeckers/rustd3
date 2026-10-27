//! `time::Time`: d3's time scale in an explicit zone, against d3-scale
//! 4.0.2 in Node under the zone of each row, and the port's own rows in
//! nanoseconds.

use rustd3_core::jiff::Timestamp;
use rustd3_core::jiff::tz::TimeZone;
use rustd3_core::time;
use rustd3_testkit::zone;

use super::{at, local, local_hour};

/// `t0 + ns` nanoseconds.
fn plus(t0: Timestamp, ns: i128) -> Timestamp {
    Timestamp::from_nanosecond(t0.as_nanosecond() + ns).expect("an instant in range")
}

/// Part I's Berlin day: the end of daylight saving time on 800 px.
fn berlin_day() -> time::Time {
    time::time(zone("Europe/Berlin"))
        .domain([at("2026-10-24T22:00:00Z"), at("2026-10-25T23:00:00Z")])
        .range([0.0, 800.0])
}

#[test]
fn the_default_domains_are_the_first_day_of_2000() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone());
    assert_eq!(
        x.get_domain(),
        [at("2000-01-01T08:00:00Z"), at("2000-01-02T08:00:00Z")]
    );
    assert_eq!(x.get_range(), [0.0, 1.0]);
    assert_eq!(x.get_zone(), &la);
    let u = time::utc();
    assert_eq!(
        u.get_domain(),
        [at("2000-01-01T00:00:00Z"), at("2000-01-02T00:00:00Z")]
    );
    assert_eq!(u.get_zone(), &TimeZone::UTC);
    assert_ne!(u, time::time(TimeZone::UTC));
}

#[test]
fn apply_and_invert_map_offsets_from_the_domain_start() {
    let x = berlin_day();
    assert!((x.apply(at("2026-10-25T12:00:00Z")) - 448.0).abs() < 1e-9);
    assert_eq!(x.invert(400.0), at("2026-10-25T10:30:00Z"));
    assert_eq!(x.apply(at("2026-10-24T22:00:00Z")), 0.0);
    assert_eq!(x.apply(at("2026-10-25T23:00:00Z")), 800.0);
    assert_eq!(x.apply(at("2026-10-26T23:00:00Z")), 800.0 * 49.0 / 25.0);
    let la = zone("America/Los_Angeles");
    let year = time::time(la.clone())
        .domain([local(&la, 2009, 1, 1), local(&la, 2010, 1, 1)])
        .range([0.0, 100.0]);
    // 181 days less the hour daylight saving time takes, of 365 days.
    assert_eq!(year.apply(local(&la, 2009, 7, 1)), 49.577625570776256);
}

#[test]
fn clamping_keeps_apply_and_invert_within_the_domain() {
    let la = zone("America/Los_Angeles");
    let year = time::time(la.clone())
        .domain([local(&la, 2009, 1, 1), local(&la, 2010, 1, 1)])
        .range([0.0, 100.0]);
    assert_eq!(year.invert(-50.0), local_hour(&la, 2008, 7, 2, 13));
    let clamped = year.clone().clamp(true);
    assert_eq!(clamped.invert(-50.0), at("2009-01-01T08:00:00Z"));
    assert_eq!(clamped.invert(150.0), local(&la, 2010, 1, 1));
    assert_eq!(clamped.apply(local(&la, 2011, 1, 1)), 100.0);
    assert_eq!(year.apply(local(&la, 2011, 1, 1)), 200.0);
    let default = time::time(la.clone()).clamp(true);
    let [d0, d1] = default.get_domain();
    assert_eq!(
        [-1.0, 0.0, 1.0, 2.0].map(|y| default.invert(y)),
        [d0, d0, d1, d1]
    );
}

#[test]
fn reversed_domains_and_ranges_map_end_to_end() {
    let [a, b] = [at("2026-10-25T00:00:00Z"), at("2026-10-26T00:00:00Z")];
    let x = time::utc().domain([b, a]).range([0.0, 24.0]);
    assert_eq!(x.apply(at("2026-10-25T06:00:00Z")), 18.0);
    assert_eq!(x.invert(18.0), at("2026-10-25T06:00:00Z"));
    let clamped = x.clone().clamp(true);
    assert_eq!(clamped.apply(at("2026-10-24T00:00:00Z")), 24.0);
    assert_eq!(clamped.invert(-1.0), b);
    let y = time::utc().domain([a, b]).range([600.0, 0.0]);
    assert_eq!(y.apply(at("2026-10-25T06:00:00Z")), 450.0);
    assert_eq!(y.invert(450.0), at("2026-10-25T06:00:00Z"));
}

#[test]
fn flat_domains_and_ranges_map_to_the_middle_as_d3_normalizes() {
    let t = at("2026-10-25T12:00:00Z");
    let flat = time::utc().domain([t, t]).range([0.0, 10.0]);
    assert_eq!(flat.apply(at("1999-01-01T00:00:00Z")), 5.0);
    assert_eq!(flat.clone().clamp(true).apply(t), 5.0);
    assert_eq!(flat.invert(3.0), t);
    let [a, b] = [t, at("2026-10-25T14:00:00Z")];
    let point = time::utc().domain([a, b]).range([4.0, 4.0]);
    assert_eq!(point.invert(4.0), at("2026-10-25T13:00:00Z"));
    assert_eq!(point.invert(-7.0), at("2026-10-25T13:00:00Z"));
}

#[test]
fn invert_saturates_at_jiffs_range_and_takes_nan_to_the_start() {
    let [a, b] = [at("2026-10-25T00:00:00Z"), at("2027-10-25T00:00:00Z")];
    let x = time::utc().domain([a, b]).range([0.0, 1.0]);
    assert_eq!(x.invert(1e9), Timestamp::MAX);
    assert_eq!(x.invert(-1e9), Timestamp::MIN);
    assert_eq!(x.invert(f64::INFINITY), Timestamp::MAX);
    assert_eq!(x.invert(f64::NEG_INFINITY), Timestamp::MIN);
    assert_eq!(x.invert(f64::NAN), a);
    let y = time::utc().domain([Timestamp::MIN, Timestamp::MAX]);
    assert_eq!(y.invert(0.0), Timestamp::MIN);
    assert_eq!(y.apply(Timestamp::MAX), 1.0);
}

#[test]
fn a_microsecond_in_2026_maps_to_the_nanosecond() {
    let t0 = at("2026-10-25T12:00:00Z");
    let x = time::utc()
        .domain([t0, plus(t0, 1_000)])
        .range([0.0, 1000.0]);
    assert_eq!(x.apply(plus(t0, 1)), 1.0);
    assert_eq!(x.invert(500.0), plus(t0, 500));
    for ns in 0..=1_000 {
        let t = plus(t0, ns);
        assert_eq!(x.invert(x.apply(t)), t, "{ns} ns");
    }
}
