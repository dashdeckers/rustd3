//! `scale::Pad` against matplotlib 3.11.2's autoscaled limits: the spot
//! checks, and every case of `tests/fixtures/pad.json`.

use rustd3_core::interact::Rescale;
use rustd3_core::scale::{self, AnyContinuous, Pad, TickRule};
use rustd3_testkit::obs::{ToObs, Tolerance};
use serde_json::Value;

/// The fixture, read at compile time so the test needs no working directory.
const PAD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pad.json"
));

/// Whether `a` and `b` agree within 1e-12 relative.
fn near(a: [f64; 2], b: [f64; 2]) -> bool {
    a.to_obs()
        .approx_eq(&b.to_obs(), &Tolerance::relative(1e-12))
}

#[test]
fn margins_widen_each_side_by_a_fraction_of_the_span() {
    let x = scale::linear();
    assert_eq!(x.pad(Some([0.0, 10.0]), 0.05, &[]), [-0.5, 10.5]);
    assert_eq!(x.pad(Some([10.0, 0.0]), 0.05, &[]), [-0.5, 10.5]);
    assert_eq!(x.pad(Some([0.1, 0.3]), 0.0, &[]), [0.1, 0.3]);
}

#[test]
fn flat_extents_widen_as_matplotlibs_nonsingular() {
    let x = scale::linear();
    assert_eq!(x.pad(Some([3.0, 3.0]), 0.05, &[]), [2.835, 3.165]);
    let zero = x.pad(Some([0.0, 0.0]), 0.05, &[]);
    assert_eq!(zero, [-0.05500000000000001, 0.05500000000000001]);
    assert!(near(zero, [-0.055, 0.055]));
}

#[test]
fn sticky_values_stop_the_margin() {
    let x = scale::linear();
    assert_eq!(x.pad(Some([0.0, 4.0]), 0.05, &[0.0]), [0.0, 4.2]);
    assert_eq!(x.pad(Some([-4.0, 0.0]), 0.05, &[0.0]), [-4.2, 0.0]);
    assert_eq!(x.pad(Some([-2.0, 4.0]), 0.05, &[0.0]), [-2.3, 4.3]);
    assert_eq!(
        x.pad(Some([0.0, 4.0]), 0.05, &[4.0, f64::NAN, 0.0]),
        [0.0, 4.0]
    );
}

#[test]
fn log_margins_are_taken_in_log_space() {
    let padded = scale::log().pad(Some([1.0, 1000.0]), 0.05, &[]);
    assert!(near(padded, [0.7079457843841379, 1412.5375446227545]));
}

#[test]
fn the_domains_orientation_is_kept() {
    let inverted = scale::linear().domain([10.0, 0.0]);
    assert_eq!(inverted.pad(Some([0.0, 10.0]), 0.05, &[]), [10.5, -0.5]);
    let log = scale::log().domain([1000.0, 1.0]);
    let padded = log.pad(Some([1.0, 1000.0]), 0.05, &[]);
    assert!(near(padded, [1412.5375446227545, 0.7079457843841379]));
    let three = scale::piecewise([0.0, 10.0, 5.0], [0.0, 1.0, 2.0]);
    assert_eq!(three.pad(Some([0.0, 10.0]), 0.05, &[]), [-0.5, 10.5]);
    let three = scale::piecewise([10.0, 20.0, 0.0], [0.0, 1.0, 2.0]);
    assert_eq!(three.pad(Some([0.0, 10.0]), 0.05, &[]), [10.5, -0.5]);
    let identity = scale::identity().domain([1.0, 0.0]);
    assert_eq!(identity.pad(Some([0.0, 10.0]), 0.05, &[]), [10.5, -0.5]);
}

#[test]
fn no_data_gives_the_first_default_domain_the_transform_maps() {
    assert_eq!(scale::linear().pad(None, 0.05, &[]), [0.0, 1.0]);
    assert_eq!(scale::log().pad(None, 0.05, &[]), [1.0, 10.0]);
    let negative = scale::log().domain([-10.0, -1.0]);
    assert_eq!(negative.pad(None, 0.05, &[]), [1.0, 10.0]);
    assert_eq!(scale::logit().pad(None, 0.05, &[]), [1e-7, 1.0 - 1e-7]);
    assert_eq!(scale::symlog().pad(None, 0.05, &[]), [0.0, 1.0]);
    let inverted = scale::identity().domain([5.0, 2.0]);
    assert_eq!(inverted.pad(None, 0.05, &[]), [1.0, 0.0]);
    let nowhere = scale::func(|_| f64::NAN, |u| u + 7.0, TickRule::Data);
    assert_eq!(nowhere.pad(None, 0.05, &[]), [7.0, 8.0]);
}

#[test]
fn non_finite_extents_widen_from_zero_in_transformed_space() {
    let x = scale::linear();
    let widened = [-0.05500000000000001, 0.05500000000000001];
    assert_eq!(x.pad(Some([f64::NAN, 5.0]), 0.05, &[]), widened);
    assert_eq!(x.pad(Some([0.0, f64::INFINITY]), 0.05, &[]), widened);
    let log = scale::log().pad(Some([0.0, 10.0]), 0.05, &[]);
    assert!(near(log, [(-0.055f64).exp(), 0.055f64.exp()]));
}

#[test]
fn a_negative_log_extent_pads_under_the_reflected_transform() {
    let padded = scale::log().pad(Some([-1000.0, -1.0]), 0.05, &[]);
    assert!(near(padded, [-1412.5375446227545, -0.7079457843841379]));
}

#[test]
fn a_decreasing_transform_pads_its_ascending_extent() {
    /// Frequency laid out by period.
    #[derive(Clone, Copy)]
    struct Reciprocal;

    impl scale::Transform for Reciprocal {
        fn forward(&self, x: f64) -> f64 {
            1.0 / x
        }
        fn inverse(&self, y: f64) -> f64 {
            1.0 / y
        }
        fn ticks(&self, d: [f64; 2], n: usize) -> Vec<f64> {
            rustd3_core::ticks::linear(d, n)
        }
    }

    let s = scale::continuous(Reciprocal).domain([1.0, 4.0]);
    let padded = s.pad(Some([4.0, 1.0]), 0.05, &[]);
    assert!(near(padded, [1.0 / 1.0375, 1.0 / 0.2125]));
}

#[test]
fn any_continuous_and_rescaled_scales_pad_as_their_transform() {
    let any: AnyContinuous = scale::log().into();
    let log = scale::log().pad(Some([3.0, 7.0]), 0.05, &[]);
    assert_eq!(any.pad(Some([3.0, 7.0]), 0.05, &[]), log);
    let zoomed = scale::log()
        .range([0.0, 300.0])
        .zoom_px(10.0, 4.0)
        .expect("a valid zoom");
    assert_eq!(zoomed.pad(Some([3.0, 7.0]), 0.05, &[]), log);
}

#[test]
fn pad_matches_every_fixture_case_of_matplotlib() {
    let fixture: Value = serde_json::from_str(PAD).expect("valid JSON");
    assert_eq!(fixture["matplotlib"], "3.11.2");
    let numbers = |v: &Value| -> Vec<f64> {
        v.as_array()
            .expect("an array")
            .iter()
            .map(|n| n.as_f64().expect("a number"))
            .collect()
    };
    let pair = |v: &Value| -> [f64; 2] { numbers(v).try_into().expect("two numbers") };
    let cases = fixture["cases"].as_array().expect("an array of cases");
    let mut failures = Vec::new();
    for case in cases {
        let extent = pair(&case["extent"]);
        let margin = case["margin"].as_f64().expect("a margin");
        let sticky = numbers(&case["sticky"]);
        let inverted = case["inverted"].as_bool().expect("an inverted flag");
        let s: AnyContinuous = match case["scale"].as_str().expect("a scale name") {
            "linear" => scale::linear().into(),
            "log" => scale::log().into(),
            other => panic!("unexpected scale {other}"),
        };
        let s = if inverted {
            let [a, b] = s.get_domain();
            s.domain([b, a])
        } else {
            s
        };
        let ours = s.pad(Some(extent), margin, &sticky);
        if !near(ours, pair(&case["limits"])) {
            failures.push(format!("{case}: ours {ours:?}"));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
    assert_eq!(cases.len(), 26);
}
