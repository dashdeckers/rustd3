//! `scale::Ordinal`, d3's scaleOrdinal with an explicit domain: lookups,
//! the unknown value, opt-in growth, its `Ticks` over numbers and its use
//! over a categorical scheme.

use rustd3_core::locale::Locale;
use rustd3_core::scale::{self, Ticks};
use rustd3_core::{axis, scheme};

#[test]
fn keys_map_to_the_range_value_at_their_index() {
    let s = scale::ordinal([0, 1], ["foo", "bar"]);
    assert_eq!(s.apply(&0), Some(&"foo"));
    assert_eq!(s.apply(&1), Some(&"bar"));
    assert_eq!(s.apply(&2), None);
    assert_eq!(s.get_domain(), [0, 1]);
    assert_eq!(s.get_range(), ["foo", "bar"]);
}

#[test]
fn a_new_range_keeps_the_keys() {
    let s = scale::ordinal([0, 1], ["foo", "bar"]).range(["a", "b", "c"]);
    assert_eq!(s.apply(&0), Some(&"a"));
    assert_eq!(s.apply(&1), Some(&"b"));
}

#[test]
fn keys_past_the_range_reuse_it_in_turn() {
    let s = scale::ordinal(0..6, ["a", "b", "c"]);
    let values: Vec<_> = (0..6).filter_map(|k| s.apply(&k).copied()).collect();
    assert_eq!(values, ["a", "b", "c", "a", "b", "c"]);
}

#[test]
fn the_unknown_value_answers_for_keys_outside_the_domain() {
    let s = scale::ordinal(["foo", "bar"], ["red", "blue"]).unknown("gray");
    assert_eq!(s.apply(&"foo"), Some(&"red"));
    assert_eq!(s.apply(&"baz"), Some(&"gray"));
    assert_eq!(s.apply(&"quux"), Some(&"gray"));
    assert_eq!(s.get_domain(), ["foo", "bar"]);
}

#[test]
fn growth_appends_unseen_keys_in_the_order_they_arrive() {
    let mut s = scale::ordinal(Vec::<&str>::new(), ["a", "b"]);
    assert_eq!(s.apply_or_insert("foo"), Some(&"a"));
    assert_eq!(s.apply_or_insert("bar"), Some(&"b"));
    assert_eq!(s.apply_or_insert("foo"), Some(&"a"));
    assert_eq!(s.apply_or_insert("baz"), Some(&"a"));
    assert_eq!(s.get_domain(), ["foo", "bar", "baz"]);
    let mut t = s.domain(["baz", "bar"]);
    assert_eq!(t.apply_or_insert("foo"), Some(&"a"));
    assert_eq!(t.get_domain(), ["baz", "bar", "foo"]);
}

#[test]
fn an_empty_range_maps_every_key_to_none_but_still_grows() {
    let mut s = scale::ordinal(["a"], Vec::<f64>::new());
    assert_eq!(s.apply(&"a"), None);
    assert_eq!(s.apply_or_insert("b"), None);
    assert_eq!(s.get_domain(), ["a", "b"]);
    assert_eq!(s.clone().unknown(1.0).apply(&"z"), Some(&1.0));
    assert_eq!(s.unknown(1.0).apply(&"a"), None);
}

#[test]
fn duplicate_keys_keep_their_first_position() {
    let s = scale::ordinal(["a", "b", "a", "c"], [1.0, 2.0, 3.0]);
    assert_eq!(s.get_domain(), ["a", "b", "c"]);
    assert_eq!(s.apply(&"c"), Some(&3.0));
    let t = s.domain(["c", "c", "a"]);
    assert_eq!(t.get_domain(), ["c", "a"]);
}

#[test]
fn an_ordinal_scale_over_numbers_places_its_keys_on_an_axis() {
    let s = scale::ordinal(["lo", "mid", "hi"], [10.0, 50.0, 90.0]);
    let set = s.tick_set(10);
    assert_eq!(set.get_values(), ["lo", "mid", "hi"]);
    assert_eq!(s.labels(&set, &Locale::NEUTRAL), ["lo", "mid", "hi"]);
    assert_eq!(s.position(&"mid", 0.5), Some(50.0));
    assert_eq!(s.position(&"x", 0.5), None);
    assert_eq!(s.ends(), [10.0, 90.0]);
    let xs: Vec<f64> = axis::bottom(&s)
        .labels()
        .iter()
        .map(|l| l.get_x())
        .collect();
    assert_eq!(xs, [10.5, 50.5, 90.5]);
}

#[test]
fn an_empty_range_has_a_nan_extent() {
    let [a, b] = scale::ordinal(["a"], Vec::<f64>::new()).ends();
    assert!(a.is_nan() && b.is_nan());
    assert_eq!(scale::ordinal(["a"], [4.0]).ends(), [4.0, 4.0]);
}

#[test]
fn an_ordinal_scale_over_category10_colors_its_keys() {
    let color = scale::ordinal(["cpu", "mem"], scheme::CATEGORY10);
    assert_eq!(
        color.apply(&"cpu").map(ToString::to_string).as_deref(),
        Some("rgb(31, 119, 180)")
    );
    assert_eq!(color.apply(&"disk"), None);
}

#[test]
fn an_empty_domain_grows_through_category10_in_order() {
    let mut grow = scale::ordinal(Vec::<&str>::new(), scheme::CATEGORY10);
    let disk = grow.apply_or_insert("disk").copied();
    let net = grow.apply_or_insert("net").copied();
    assert_eq!(
        disk.map(|c| c.to_string()).as_deref(),
        Some("rgb(31, 119, 180)")
    );
    assert_eq!(
        net.map(|c| c.to_string()).as_deref(),
        Some("rgb(255, 127, 14)")
    );
    assert_eq!(grow.get_domain(), ["disk", "net"]);
}
