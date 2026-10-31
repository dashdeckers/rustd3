//! The facade's paths: each re-export is the item rustd3-core or
//! rustd3-figure defines, one path per item, under the features that
//! enable it.

#[cfg(feature = "derive")]
mod derive;
#[cfg(feature = "figure")]
mod figure;
mod readme;

use rustd3::jiff::Timestamp;
use rustd3::jiff::tz::TimeZone;
use rustd3::{axis, scale, time};

/// `doc` (a crate's `//!` lines) as words one space apart.
fn flat(doc: &str) -> String {
    let lines = doc
        .lines()
        .map(|l| l.trim_start().trim_start_matches("//!"));
    let words: Vec<&str> = lines.flat_map(str::split_whitespace).collect();
    words.join(" ")
}

#[test]
fn the_crate_docs_map_the_component_crates_names_before_their_first_example() {
    let facade = flat(include_str!("../../src/lib.rs"));
    let figure = flat(include_str!("../../figure/src/lib.rs"));
    let notes = [
        (&facade, "`rustd3_iced` is not re-exported"),
        (
            &figure,
            "`rustd3_iced` (the `Plot` widget) is used directly",
        ),
    ];
    for (doc, iced) in notes {
        let first = doc
            .find("```rust")
            .or_else(|| doc.find("```"))
            .expect("an example");
        for note in [
            "`rustd3_core::x` is `rustd3::x`",
            "`rustd3_figure::x` is `rustd3::figure::x`",
            iced,
        ] {
            let at = doc.find(note).unwrap_or_else(|| panic!("no {note:?}"));
            assert!(at < first, "{note:?} after the first example");
        }
    }
}

#[test]
fn a_linear_scale_maps_25_to_240_and_inverts_480_to_50() {
    let x = scale::linear().domain([0.0, 100.0]).range([0.0, 960.0]);
    assert_eq!(x.apply(25.0), 240.0);
    assert_eq!(x.invert(480.0), 50.0);
}

#[test]
fn rustd3_axis_bottom_is_the_core_axis_of_a_scale() {
    let x = scale::linear().domain([0.0, 100.0]).range([0.0, 960.0]);
    let bottom: rustd3_core::axis::Axis<'_, rustd3_core::scale::Linear> = axis::bottom(&x);
    let labels: Vec<String> = bottom
        .ticks(5)
        .labels()
        .iter()
        .map(|l| l.get_text().to_owned())
        .collect();
    assert_eq!(labels, ["0", "20", "40", "60", "80", "100"]);
}

#[test]
fn fmt_is_the_core_macro() {
    let f: rustd3_core::format::Format = rustd3::fmt!(".1%");
    assert_eq!(f.format(0.123), "12.3%");
}

#[test]
fn jiff_is_the_core_reexport() {
    let t: rustd3_core::jiff::Timestamp = Timestamp::UNIX_EPOCH;
    assert_eq!(time::iso_format(t), "1970-01-01T00:00:00.000Z");
}

#[test]
fn time_formats_and_locales_resolve_through_the_core_path() {
    let t = time::iso_parse("2026-10-28T12:34:56Z").unwrap();
    let f: time::Format = time::format("%Y-%m-%d %H:%M");
    assert_eq!(f.format(t, &TimeZone::UTC), "2026-10-28 12:34");
    let de: rustd3_core::time::Locale = time::Locale::DE_DE;
    assert_eq!(de.format("%B").format(t, &TimeZone::UTC), "Oktober");
    let from_definition: fn(time::Definition) -> time::Locale = time::Locale::new;
    let _ = from_definition;
    assert_eq!(time::Ladder::default(), rustd3_core::time::Ladder::Iso);
}
