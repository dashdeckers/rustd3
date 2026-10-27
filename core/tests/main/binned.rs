//! The binned scales `Quantize`, `Quantile` and `Threshold`, and their
//! `Ticks`, against d3-scale 4.0.2 (d3 7.9.0 in Node).

use rustd3_core::locale::Locale;
use rustd3_core::scale::{self, Ticks};

#[test]
fn quantize_maps_equal_bins_to_the_range() {
    let q = scale::quantize([0.0, 1.0], ["a", "b", "c"]);
    assert_eq!(
        [q.apply(0.2), q.apply(0.5), q.apply(0.9)],
        [Some(&"a"), Some(&"b"), Some(&"c")]
    );
    assert_eq!(q.apply(f64::NAN), None);
    assert_eq!(q.thresholds(), [0.3333333333333333, 0.6666666666666666]);
    assert_eq!(
        q.invert_extent(&"b"),
        Some([0.3333333333333333, 0.6666666666666666])
    );
    assert_eq!(q.invert_extent(&"d"), None);
}

#[test]
fn quantize_ticks_are_its_bin_edges() {
    let q = scale::quantize([0.0, 1.0], ["a", "b", "c"]);
    let set = q.tick_set(5);
    assert_eq!(
        set.values(),
        [0.0, 0.3333333333333333, 0.6666666666666666, 1.0]
    );
    assert_eq!(*set.step(), 0.3333333333333333);
    assert_eq!(q.position(&0.5, 0.0), Some(0.5));
    assert_eq!(
        q.labels(&set, &Locale::NEUTRAL),
        ["0.0", "0.3", "0.7", "1.0"]
    );
    assert_eq!(q.extent(), [0.0, 1.0]);
}

#[test]
fn quantize_nices_and_ticks_as_a_linear_scale() {
    let q = scale::quantize([0.13, 9.71], [0.0, 1.0, 2.0, 3.0]).nice(10);
    assert_eq!(q.get_domain(), [0.0, 10.0]);
    let q = scale::quantize([0.0, 10.0], [0.0, 1.0]);
    assert_eq!(q.ticks(5), [0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
    assert_eq!(
        q.tick_format(5, None).map(|f| f.format(4.0)),
        Ok("4".to_owned())
    );
}

const SAMPLE: [f64; 10] = [3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0];

#[test]
fn quantile_cuts_a_sample_at_its_quantiles() {
    let q = scale::quantile(SAMPLE, [0, 1, 2, 3]);
    assert_eq!(q.quantiles(), [7.25, 9.0, 14.5]);
    assert_eq!(
        [q.apply(7.9), q.apply(8.0), q.apply(15.5)],
        [Some(&1), Some(&1), Some(&3)]
    );
    assert_eq!(q.invert_extent(&2), Some([9.0, 14.5]));
    assert_eq!(q.invert_extent(&0), Some([3.0, 7.25]));
    assert_eq!(q.apply(f64::NAN), None);
}

#[test]
fn quantile_drops_nan_from_its_sorted_domain() {
    let q = scale::quantile([6.0, f64::NAN, 3.0, 8.0], [0, 1]);
    assert_eq!(q.get_domain(), [3.0, 6.0, 8.0]);
}

#[test]
fn quantile_ticks_run_from_the_least_to_the_greatest_value() {
    let q = scale::quantile(SAMPLE, [0, 1, 2, 3]);
    let set = q.tick_set(5);
    assert_eq!(set.values(), [3.0, 7.25, 9.0, 14.5, 20.0]);
    assert_eq!(*set.step(), 1.75);
    assert_eq!(q.position(&20.0, 0.0), Some(1.0));
    // precision_fixed(1.75) is 0 places, as d3 would label ticks 1.75 apart.
    assert_eq!(
        q.labels(&set, &Locale::NEUTRAL),
        ["3", "7", "9", "15", "20"]
    );
}

#[test]
fn an_empty_quantile_sample_maps_to_the_first_value_and_has_no_ticks() {
    let q = scale::quantile(Vec::new(), [0, 1, 2]);
    assert_eq!(q.apply(5.0), Some(&0));
    assert!(q.quantiles().iter().all(|t| t.is_nan()));
    assert!(q.tick_set(5).values().is_empty());
}

#[test]
fn threshold_maps_open_ended_bins() {
    let t = scale::threshold([0.0, 1.0], ["a", "b", "c"]);
    let mapped = [-1.0, 0.0, 0.5, 1.0, 2.0].map(|x| t.apply(x).copied());
    assert_eq!(
        mapped,
        [Some("a"), Some("b"), Some("b"), Some("c"), Some("c")]
    );
    let extents = ["a", "b", "c"].map(|y| t.invert_extent(&y));
    let expected = [[None, Some(0.0)], [Some(0.0), Some(1.0)], [Some(1.0), None]];
    assert_eq!(extents, expected.map(Some));
    assert_eq!(t.invert_extent(&"d"), None);
}

#[test]
fn threshold_places_its_thresholds_as_equal_width_bins() {
    let t = scale::threshold([0.0, 1.0], ["a", "b", "c"]);
    assert_eq!(t.position(&0.0, 0.0), Some(1.0 / 3.0));
    assert_eq!(t.position(&1.0, 0.0), Some(2.0 / 3.0));
    assert_eq!(t.position(&0.5, 0.0), None);
    assert_eq!(t.tick_set(5).values(), [0.0, 1.0]);
}

#[test]
fn a_threshold_without_a_range_maps_nothing() {
    let t = scale::threshold([0.0, 1.0], Vec::<&str>::new());
    assert_eq!(t.apply(0.5), None);
}
