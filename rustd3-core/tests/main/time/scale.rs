//! `time::Time`: d3's time scale in an explicit zone, against d3-scale
//! 4.0.2 in JavaScript under the zone of each row, and rustd3-core's own
//! rows in nanoseconds.

use rustd3_core::interact::{self, Rescale};
use rustd3_core::jiff::Timestamp;
use rustd3_core::jiff::civil::Weekday;
use rustd3_core::jiff::tz::TimeZone;
use rustd3_core::scale::Pad;
use rustd3_core::time::{self, Interval};
use rustd3_testkit::zone;

use super::{at, local, local_hour};

/// `t0 + ns` nanoseconds.
fn plus(t0: Timestamp, ns: i128) -> Timestamp {
    Timestamp::from_nanosecond(t0.as_nanosecond() + ns).expect("an instant in range")
}

/// A day in Berlin: the end of daylight saving time on 800 px.
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
    assert_eq!(
        x.zoom_px(500.0, 2.0).map(|x| x.get_domain()),
        Some([plus(t0, 250), plus(t0, 750)])
    );
    assert!(x.zoom_px(500.0, 2000.0).is_none());
    assert_eq!(
        x.pan_px(100.0).map(|x| x.get_domain()),
        Some([plus(t0, -100), plus(t0, 900)])
    );
    for ns in 0..=1_000 {
        let t = plus(t0, ns);
        assert_eq!(x.invert(x.apply(t)), t, "{ns} ns");
    }
}

#[test]
fn berlin_ticks() {
    let x = berlin_day();
    assert_eq!(x.tick_interval(8), Interval::Hour(3));
    let ticks = [
        "2026-10-24T22:00:00Z",
        "2026-10-25T02:00:00Z",
        "2026-10-25T05:00:00Z",
        "2026-10-25T08:00:00Z",
        "2026-10-25T11:00:00Z",
        "2026-10-25T14:00:00Z",
        "2026-10-25T17:00:00Z",
        "2026-10-25T20:00:00Z",
        "2026-10-25T23:00:00Z",
    ];
    assert_eq!(x.ticks(8), ticks.map(at));
    let berlin = zone("Europe/Berlin");
    let hours: Vec<i8> = x
        .ticks(8)
        .iter()
        .map(|&t| berlin.to_datetime(t).hour())
        .collect();
    assert_eq!(hours, [0, 3, 6, 9, 12, 15, 18, 21, 0]);
}

#[test]
fn ticks_lay_out_in_the_scales_zone_or_count_days_from_1970_in_utc() {
    let la = zone("America/Los_Angeles");
    let span = [local(&la, 2011, 1, 1), local(&la, 2011, 1, 9)];
    let x = time::time(la.clone()).domain(span);
    assert_eq!(x.tick_interval(4), Interval::Day(2));
    assert_eq!(x.ticks(4), [1, 3, 5, 7, 9].map(|d| local(&la, 2011, 1, d)));
    let u = time::utc().domain(span);
    assert_eq!(u.tick_interval(4), Interval::UnixDay(2));
    let utc_days = [2, 4, 6, 8].map(|d| at(&format!("2011-01-{d:02}T00:00:00Z")));
    assert_eq!(u.ticks(4), utc_days);
    let years = time::time(la.clone()).domain([local(&la, 2000, 1, 1), local(&la, 2100, 1, 1)]);
    let every_20 = [2000, 2020, 2040, 2060, 2080, 2100].map(|y| local(&la, y, 1, 1));
    assert_eq!(years.ticks(5), every_20);
    let noon = local_hour(&la, 2011, 1, 1, 12);
    let second = time::time(la.clone()).domain([noon, plus(noon, 1_000_000_000)]);
    let fifths = [0, 200, 400, 600, 800, 1_000].map(|ms| plus(noon, ms * 1_000_000));
    assert_eq!(second.ticks(4), fifths);
    let minutes = second
        .clone()
        .domain([noon, plus(noon, 33 * 60_000_000_000 + 4_000_000_000)]);
    let every_10 = minutes.ticks_with(Interval::Minute(10));
    assert_eq!(
        every_10,
        [0, 10, 20, 30].map(|m| plus(noon, m * 60_000_000_000))
    );
}

#[test]
fn equal_ends_tick_once_and_reversed_domains_tick_backwards() {
    let la = zone("America/Los_Angeles");
    let t = at("2009-01-01T08:12:00Z");
    let flat = time::time(la.clone()).domain([t, t]);
    assert_eq!(flat.ticks(10), [t]);
    assert_eq!(flat.clone().nice(10).get_domain(), [t, t]);
    let back = time::time(la.clone()).domain([local(&la, 2014, 3, 2), local(&la, 2010, 12, 18)]);
    let years = [2014, 2013, 2012, 2011].map(|y| local(&la, y, 1, 1));
    assert_eq!(back.ticks(4), years);
}

#[test]
fn ticks_go_below_a_millisecond_in_a_scale() {
    let t0 = at("2026-10-25T12:00:00Z");
    let micro = time::utc().domain([t0, plus(t0, 10_000)]);
    assert_eq!(micro.tick_interval(5), Interval::Microsecond(2));
    let expected = [0, 2_000, 4_000, 6_000, 8_000, 10_000].map(|ns| plus(t0, ns));
    assert_eq!(micro.ticks(5), expected);
    let nano = time::time(zone("Europe/Berlin")).domain([t0, plus(t0, 100)]);
    assert_eq!(nano.tick_interval(10), Interval::Nanosecond(10));
    assert_eq!(
        nano.ticks(10),
        (0..=10).map(|i| plus(t0, 10 * i)).collect::<Vec<_>>()
    );
}

#[test]
fn nice_extends_the_ends_to_the_tick_interval() {
    let la = zone("America/Los_Angeles");
    let day =
        time::time(la.clone()).domain([at("2009-01-01T08:12:00Z"), at("2009-01-02T07:48:00Z")]);
    assert_eq!(
        day.clone().nice(10).get_domain(),
        [at("2009-01-01T08:00:00Z"), at("2009-01-02T08:00:00Z")]
    );
    let centuries = time::time(la.clone()).domain([local(&la, 2001, 1, 1), local(&la, 2138, 1, 1)]);
    assert_eq!(
        centuries.nice(10).get_domain(),
        [local(&la, 2000, 1, 1), local(&la, 2140, 1, 1)]
    );
    let weeks = day.clone().nice_with(Interval::Week(Weekday::Sunday, 2));
    assert_eq!(
        weeks.get_domain(),
        [local(&la, 2008, 12, 21), local(&la, 2009, 1, 4)]
    );
    let back = time::time(la.clone())
        .domain([
            plus(local_hour(&la, 2011, 1, 3, 12), 7 * 60_000_000_000),
            local_hour(&la, 2011, 1, 1, 13),
        ])
        .nice(10);
    assert_eq!(
        back.get_domain(),
        [at("2011-01-04T02:00:00Z"), at("2011-01-01T20:00:00Z")]
    );
}

#[test]
fn nice_of_zero_and_past_jiffs_range_leave_the_domain() {
    let la = zone("America/Los_Angeles");
    let noon = local_hour(&la, 2011, 1, 1, 12);
    let x = time::time(la.clone()).domain([plus(noon, 7), plus(noon, 1_000_000_000)]);
    assert_eq!(x.clone().nice(0), x);
    let edge = time::utc().domain([plus(Timestamp::MIN, 1), plus(Timestamp::MAX, -1)]);
    assert_eq!(edge.clone().nice_with(Interval::Year(1)), edge);
    assert_eq!(edge.clone().nice_with(Interval::Nanosecond(1)), edge);
}

#[test]
fn rescaling_berlin_by_a_transform_moves_the_domain_as_d3() {
    let x = berlin_day();
    let pan = interact::Transform {
        k: 1.0,
        x: 80.0,
        y: 0.0,
    };
    let panned = [at("2026-10-24T19:30:00Z"), at("2026-10-25T20:30:00Z")];
    assert_eq!(x.rescale_x(&pan).get_domain(), panned);
    assert_eq!(x.pan_px(80.0).map(|x| x.get_domain()), Some(panned));
    let zoom = interact::Transform {
        k: 2.0,
        x: -400.0,
        y: 0.0,
    };
    let zoomed = [at("2026-10-25T04:15:00Z"), at("2026-10-25T16:45:00Z")];
    assert_eq!(x.rescale_x(&zoom).get_domain(), zoomed);
    assert_eq!(x.zoom_px(400.0, 2.0).map(|x| x.get_domain()), Some(zoomed));
    let vertical = x.clone().range([800.0, 0.0]);
    let flipped = interact::Transform {
        k: 1.0,
        x: 0.0,
        y: -80.0,
    };
    // Content moved up 80 px on an upward axis shows earlier instants.
    assert_eq!(vertical.rescale_y(&flipped).get_domain(), panned);
    assert_eq!(vertical.pan_px(-80.0).map(|x| x.get_domain()), Some(panned));
}

#[test]
fn a_clamping_scale_cannot_zoom_out_through_rescale_but_gestures_do_not_clamp() {
    let x = berlin_day().clamp(true);
    let out = interact::Transform {
        k: 0.5,
        x: 200.0,
        y: 0.0,
    };
    assert_eq!(x.rescale_x(&out).get_domain(), x.get_domain());
    let wide = x.zoom_px(400.0, 0.5).expect("a zoom out of a day");
    assert_eq!(
        wide.get_domain(),
        [at("2026-10-24T09:30:00Z"), at("2026-10-26T11:30:00Z")]
    );
}

#[test]
fn degenerate_gestures_return_none() {
    let t0 = at("2026-10-25T12:00:00Z");
    let x = time::utc()
        .domain([t0, plus(t0, 1_000)])
        .range([0.0, 1000.0]);
    assert!(x.zoom_px(500.0, 2000.0).is_none());
    assert!(x.zoom_px(500.0, 1001.0).is_none());
    assert!(x.zoom_px(500.0, 999.0).is_some());
    for factor in [0.0, -2.0, f64::NAN, f64::INFINITY] {
        assert!(x.zoom_px(500.0, factor).is_none(), "{factor}");
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(x.zoom_px(value, 2.0).is_none(), "{value}");
        assert!(x.pan_px(value).is_none(), "{value}");
    }
    let flat = time::utc().domain([t0, t0]).range([0.0, 1000.0]);
    assert!(flat.pan_px(10.0).is_none() && flat.zoom_px(500.0, 2.0).is_none());
    let edge = time::utc().domain([Timestamp::MIN, plus(Timestamp::MIN, 1_000)]);
    assert!(edge.pan_px(0.5).is_none());
    assert!(edge.pan_px(-0.5).is_some());
    let wide = time::utc().domain([Timestamp::MIN, Timestamp::MAX]);
    assert!(wide.zoom_px(0.5, 0.5).is_none());
    let flat_range = x.clone().range([3.0, 3.0]);
    assert!(flat_range.pan_px(1.0).is_none());
    let reversed = time::utc()
        .domain([plus(t0, 1_000), t0])
        .range([0.0, 1000.0]);
    assert_eq!(
        reversed.zoom_px(500.0, 2.0).map(|x| x.get_domain()),
        Some([plus(t0, 750), plus(t0, 250)])
    );
}

#[test]
fn pad_fits_dates_as_matplotlib() {
    let x = time::utc();
    assert_eq!(
        x.pad(None, 0.05, &[]),
        [at("1970-01-01T00:00:00Z"), at("1970-01-02T00:00:00Z")]
    );
    // An hour per side, then 5% of the two hours.
    let noon = at("2026-01-01T12:00:00Z");
    assert_eq!(
        x.pad(Some([noon, noon]), 0.05, &[]),
        [at("2026-01-01T10:54:00Z"), at("2026-01-01T13:06:00Z")]
    );
    let ten_days = [at("2026-01-01T00:00:00Z"), at("2026-01-11T00:00:00Z")];
    let padded = [at("2025-12-31T12:00:00Z"), at("2026-01-11T12:00:00Z")];
    assert_eq!(x.pad(Some(ten_days), 0.05, &[]), padded);
    assert_eq!(x.pad(Some([ten_days[1], ten_days[0]]), 0.05, &[]), padded);
    let stuck = x.pad(Some(ten_days), 0.05, &[ten_days[0]]);
    assert_eq!(stuck, [ten_days[0], padded[1]]);
    let reversed = time::utc().domain([ten_days[1], ten_days[0]]);
    assert_eq!(
        reversed.pad(Some(ten_days), 0.05, &[]),
        [padded[1], padded[0]]
    );
    assert_eq!(
        x.pad(Some([Timestamp::MIN, Timestamp::MAX]), 0.05, &[]),
        [Timestamp::MIN, Timestamp::MAX]
    );
}

#[test]
fn timestamps_2026_over_one_microsecond() {
    let t0 = at("2026-10-25T12:00:00.000000001Z");
    let x = time::utc()
        .domain([t0, plus(t0, 1_000)])
        .range([0.0, 1000.0]);
    let ns = |t: Timestamp| t.as_nanosecond() - t0.as_nanosecond();
    for offset in 0..=1_000 {
        assert_eq!(x.apply(plus(t0, offset)), offset as f64, "apply {offset}");
        assert_eq!(x.invert(offset as f64), plus(t0, offset), "invert {offset}");
    }
    let panned = x.pan_px(100.0).expect("a pan").get_domain();
    assert_eq!(panned.map(ns), [-100, 900]);
    let zoomed = x.zoom_px(500.0, 2.0).expect("a zoom").get_domain();
    assert_eq!(zoomed.map(ns), [250, 750]);
    let off_centre = x.zoom_px(333.0, 3.0).expect("a zoom").get_domain();
    assert_eq!(off_centre.map(ns), [222, 555]);
    let padded = x.pad(Some(x.get_domain()), 0.05, &[]);
    assert_eq!(padded.map(ns), [-50, 1_050]);
    // The i128 path keeps every probe to the nanosecond; an f64 of epoch
    // nanoseconds (a 256 ns ulp in 2026) moves at least one.
    let probes = [x.get_domain(), panned, zoomed, off_centre, padded].concat();
    let offsets: Vec<i128> = probes.iter().map(|&t| ns(t)).collect();
    assert_eq!(
        offsets,
        [0, 1_000, -100, 900, 250, 750, 222, 555, -50, 1_050]
    );
    let through_f64 = |t: Timestamp| ((t.as_nanosecond() as f64) as i128) - t.as_nanosecond();
    assert!(probes.iter().any(|&t| through_f64(t).abs() >= 1));
}
