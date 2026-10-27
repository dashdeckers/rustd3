//! `scale::Identity`, d3's scaleIdentity: the spot checks of d3-scale
//! 4.0.2, and its `Ticks` impl against a linear scale over the same stops.

use rustd3_core::format::InvalidFormat;
use rustd3_core::locale::Locale;
use rustd3_core::scale::{self, Ticks};

#[test]
fn identity_returns_its_input_whatever_the_stops() {
    let s = scale::identity().domain([1.0, 2.0]);
    assert_eq!(s.apply(0.5), 0.5);
    assert_eq!(s.invert(3.0), 3.0);
    assert_eq!(s.get_range(), [1.0, 2.0]);
    assert_eq!(s.get_domain(), [1.0, 2.0]);
}

#[test]
fn nan_maps_to_the_unknown_value_or_stays_nan() {
    assert!(scale::identity().apply(f64::NAN).is_nan());
    let s = scale::identity().unknown(-1.0);
    assert_eq!(s.apply(f64::NAN), -1.0);
    assert_eq!(s.invert(f64::NAN), -1.0);
    assert_eq!(s.apply(0.4), 0.4);
}

#[test]
fn domain_and_range_are_one_array() {
    assert_eq!(scale::identity().get_domain(), [0.0, 1.0]);
    assert_eq!(scale::identity().range([3.0, 4.0]).get_domain(), [3.0, 4.0]);
    assert_eq!(scale::identity().domain([3.0, 4.0]).get_range(), [3.0, 4.0]);
}

#[test]
fn nice_and_tick_formats_follow_a_linear_scale() -> Result<(), InvalidFormat> {
    let s = scale::identity().domain([0.13, 9.71]).nice(5);
    assert_eq!(s.get_domain(), [0.0, 10.0]);
    let f = s.tick_format(5, None)?;
    assert_eq!(f.format_all(&s.ticks(5)), ["0", "2", "4", "6", "8", "10"]);
    let precise = scale::identity().domain([0.123456789, 1.23456789]);
    let first = |count| -> Result<String, InvalidFormat> {
        let f = precise.tick_format(count, None)?;
        Ok(f.format(precise.ticks(count)[0]))
    };
    assert_eq!(first(16)?, "0.15");
    assert_eq!(first(256)?, "0.125");
    Ok(())
}

#[test]
fn the_ticks_impl_matches_a_linear_scale_over_the_same_stops() {
    for (stops, count) in [([0.0, 1.0], 10), ([-3.0, 250.0], 7), ([12.0, 0.5], 4)] {
        let id = scale::identity().domain(stops);
        let linear = scale::linear().domain(stops).range(stops);
        let (a, b) = (id.tick_set(count), linear.tick_set(count));
        assert_eq!(a, b);
        for locale in [Locale::NEUTRAL, Locale::EN_US] {
            assert_eq!(id.labels(&a, &locale), linear.labels(&b, &locale));
        }
        assert_eq!(Ticks::minor(&id, &a), Ticks::minor(&linear, &b));
        assert_eq!(id.extent(), linear.extent());
        for v in a.values() {
            // The identity is exact where linear interpolation rounds.
            assert_eq!(id.position(v, 0.5), Some(*v));
            let p = linear
                .position(v, 0.5)
                .expect("a continuous scale places every value");
            assert!((p - v).abs() <= 1e-12 * v.abs().max(1.0), "{p} against {v}");
        }
    }
}

#[test]
fn identity_minor_ticks_reach_the_stops() {
    let s = scale::identity().domain([0.5, 9.5]);
    let set = s.tick_set(5);
    let minor = Ticks::minor(&s, &set);
    assert_eq!(minor.len(), 18);
    assert_eq!(minor[0], 0.8);
}
