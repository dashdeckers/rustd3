//! Time tick labels: d3's `multi` rows (d3-time-format 4.1.0's
//! `format-test.js`), d3-scale's `tickFormat` choice under both ladders,
//! and the port's sub-second rungs.

use rustd3_core::jiff::Timestamp;
use rustd3_core::jiff::tz::TimeZone;
use rustd3_core::time::{self, Locale};
use rustd3_testkit::zone;

use super::{at, local, local_hour};

/// `t0 + ns` nanoseconds.
fn plus(t0: Timestamp, ns: i128) -> Timestamp {
    Timestamp::from_nanosecond(t0.as_nanosecond() + ns).expect("an instant in range")
}

#[test]
fn en_us_labels_ticks_as_d3s_multi_format() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone());
    let label = |t| x.tick_label(t, &Locale::EN_US);
    let new_year = local(&la, 1990, 1, 1);
    assert_eq!(label(plus(new_year, 12_000_000)), ".012");
    assert_eq!(label(plus(new_year, 1_000_000_000)), ":01");
    assert_eq!(label(plus(new_year, 60_000_000_000)), "12:01");
    assert_eq!(label(local_hour(&la, 1990, 1, 1, 1)), "01 AM");
    assert_eq!(label(local(&la, 1990, 1, 2)), "Tue 02");
    assert_eq!(label(local(&la, 1990, 2, 1)), "February");
    assert_eq!(label(new_year), "1990");
    assert_eq!(label(local(&la, 2011, 2, 6)), "Feb 06");
}

#[test]
fn the_neutral_ladder_labels_each_rung_in_iso_form() {
    let x = time::utc();
    let label = |s: &str| x.tick_label(at(s), &Locale::NEUTRAL);
    assert_eq!(label("2026-01-01T00:00:00Z"), "2026");
    assert_eq!(label("2026-03-01T00:00:00Z"), "2026-03");
    assert_eq!(label("2026-03-08T00:00:00Z"), "03-08");
    assert_eq!(label("2026-03-09T00:00:00Z"), "03-09");
    assert_eq!(label("2026-03-09T13:00:00Z"), "13:00");
    assert_eq!(label("2026-03-09T13:05:00Z"), "13:05");
    assert_eq!(label("2026-03-09T13:05:07Z"), "13:05:07");
    assert_eq!(label("2026-03-09T13:05:07.250Z"), "07.250");
    assert_eq!(label("2026-03-09T13:05:07.000250Z"), "07.000250");
    assert_eq!(label("2026-03-09T13:05:07.000000250Z"), "07.000000250");
}

#[test]
fn labels_read_the_scales_zone() {
    let t = at("2026-10-24T22:00:00Z");
    let berlin = time::time(zone("Europe/Berlin"));
    assert_eq!(berlin.tick_label(t, &Locale::NEUTRAL), "10-25");
    assert_eq!(berlin.tick_label(t, &Locale::EN_US), "Oct 25");
    assert_eq!(time::utc().tick_label(t, &Locale::NEUTRAL), "22:00");
    assert_eq!(
        time::time(TimeZone::UTC).tick_label(t, &Locale::EN_US),
        "10 PM"
    );
}
