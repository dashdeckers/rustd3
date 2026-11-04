//! Time tick labels: d3's `multi` rows (d3-time-format 4.1.0's
//! `format-test.js`), d3-scale's `tickFormat` choice under both ladders,
//! and the port's sub-second rungs.

use rustd3_core::jiff::Timestamp;
use rustd3_core::jiff::tz::TimeZone;
use rustd3_core::locale as figure;
use rustd3_core::scale::Ticks;
use rustd3_core::time::{self, Interval, Locale};
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

/// A day in Berlin: the end of daylight saving time on 800 px.
fn berlin_day() -> time::Time {
    time::time(zone("Europe/Berlin"))
        .domain([at("2026-10-24T22:00:00Z"), at("2026-10-25T23:00:00Z")])
        .range([0.0, 800.0])
}

#[test]
fn berlin_example() {
    let st = berlin_day();
    assert_eq!(st.tick_interval(8), Interval::Hour(3));
    let set = st.tick_set(8);
    assert_eq!(set.get_step(), &Interval::Hour(3));
    assert_eq!(set.get_count(), 8);
    assert_eq!(
        st.labels(&set, &figure::Locale::default()),
        [
            "10-25", "03:00", "06:00", "09:00", "12:00", "15:00", "18:00", "21:00", "10-26"
        ]
    );
    // d3-scale 4.0.2's tickFormat in Node under TZ=Europe/Berlin.
    assert_eq!(
        st.labels(&set, &figure::Locale::EN_US),
        [
            "Oct 25", "03 AM", "06 AM", "09 AM", "12 PM", "03 PM", "06 PM", "09 PM", "Mon 26"
        ]
    );
    let positions: Vec<f64> = set
        .get_values()
        .iter()
        .map(|t| st.position(t, 0.5).unwrap())
        .collect();
    let hours = [0.0, 4.0, 7.0, 10.0, 13.0, 16.0, 19.0, 22.0, 25.0];
    for (x, h) in positions.iter().zip(hours) {
        assert!((x - 800.0 * h / 25.0).abs() < 1e-9, "{x} for hour {h}");
    }
    assert_eq!(st.ends(), [0.0, 800.0]);
    assert!(st.minor(&set).is_empty());
}

#[test]
fn sub_second_labels() {
    let t0 = at("2026-10-25T12:00:00Z");
    let micro = time::utc().domain([t0, plus(t0, 10_000)]);
    let set = micro.tick_set(5);
    assert_eq!(set.get_step(), &Interval::Microsecond(2));
    assert_eq!(
        micro.labels(&set, &figure::Locale::default()),
        [
            "12:00",
            "00.000002",
            "00.000004",
            "00.000006",
            "00.000008",
            "00.000010"
        ]
    );
    // A microsecond tick on an exact millisecond takes the millisecond rung.
    let across = time::utc().domain([plus(t0, 600_000), plus(t0, 1_400_000)]);
    let set = across.tick_set(4);
    assert_eq!(set.get_step(), &Interval::Microsecond(200));
    assert_eq!(
        across.labels(&set, &figure::Locale::default()),
        ["00.000600", "00.000800", "00.001", "00.001200", "00.001400"]
    );
    let nano = time::utc().domain([plus(t0, 1_000), plus(t0, 1_010)]);
    let set = nano.tick_set(5);
    assert_eq!(set.get_step(), &Interval::Nanosecond(2));
    assert_eq!(
        nano.labels(&set, &figure::Locale::default()),
        [
            "00.000001",
            "00.000001002",
            "00.000001004",
            "00.000001006",
            "00.000001008",
            "00.000001010"
        ]
    );
    assert_eq!(
        nano.labels(&set, &figure::Locale::EN_US),
        [
            ".000001",
            ".000001002",
            ".000001004",
            ".000001006",
            ".000001008",
            ".000001010"
        ]
    );
}

#[test]
fn spans_tick_and_label_as_d3s_scale_time() {
    // d3-scale 4.0.2 in Node under each zone; de-DE as d3's default locale.
    let de = super::locales::from_json(
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../dev/harness/d3/d3-time-format/locale/de-DE.json"
        )),
        rustd3_core::time::Ladder::D3,
    );
    // A zone, a domain, the locale when not EN_US through `labels`, d3's labels.
    type Row<'a> = (&'a str, [&'a str; 2], Option<&'a Locale>, &'a [&'a str]);
    let rows: [Row; 5] = [
        (
            "America/Los_Angeles",
            ["2019-06-01T07:00:00Z", "2031-03-01T08:00:00Z"],
            None,
            &["2020", "2022", "2024", "2026", "2028", "2030"],
        ),
        (
            "America/Los_Angeles",
            ["2026-03-08T07:59:59.950Z", "2026-03-08T08:00:00.040Z"],
            None,
            &[".960", ".980", "Mar 08", ".020", ".040"],
        ),
        (
            "Europe/Berlin",
            ["2026-03-07T22:58:30Z", "2026-03-07T23:01:30Z"],
            None,
            &[":30", "11:59", ":30", "Mar 08", ":30", "12:01", ":30"],
        ),
        (
            "Europe/Berlin",
            ["2026-02-03T04:00:00Z", "2026-03-29T10:00:00Z"],
            Some(&de),
            &[
                "Feb 08", "Feb 15", "Feb 22", "März", "Mrz 08", "Mrz 15", "Mrz 22", "Mrz 29",
            ],
        ),
        (
            "Europe/Berlin",
            ["2025-11-19T23:00:00Z", "2027-02-09T23:00:00Z"],
            Some(&de),
            &["2026", "April", "Juli", "Oktober", "2027"],
        ),
    ];
    for (name, [a, b], locale, d3) in rows {
        let x = time::time(zone(name)).domain([at(a), at(b)]);
        let set = x.tick_set(5);
        let labels: Vec<String> = match locale {
            Some(locale) => set
                .get_values()
                .iter()
                .map(|&t| x.tick_label(t, locale))
                .collect(),
            None => x.labels(&set, &figure::Locale::EN_US),
        };
        assert_eq!(labels, d3, "{name} {a}");
    }
}
