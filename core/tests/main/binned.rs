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
