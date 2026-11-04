//! d3's tests of d3-scale, as assertions on rustd3-core; each names d3's file, line and column.

#![allow(
    clippy::approx_constant,
    reason = "d3's tests write these constants as decimal literals"
)]

use rustd3_core::color::{Hsl, Rgb};
use rustd3_core::{array, format, interpolate, scale};

use super::support::{assert_d3_eq, assert_in_delta, js_round, round_epsilon};

/// d3-scale's test helper `roundEpsilon`, under the name its tests call.
fn round(x: f64) -> f64 {
    js_round(x * 1e12) / 1e12
}

// d3-scale/test/linear-test.js: scaleLinear() has the expected defaults
#[test]
fn scalelinear_has_the_expected_defaults() {
    let s = scale::linear();
    assert_eq!(s.get_domain().to_vec(), vec![0.0, 1.0]); // d3-scale/test/linear-test.js:7:3
    assert_eq!(s.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/linear-test.js:8:3
}

// d3-scale/test/linear-test.js: scaleLinear(range) sets the range
#[test]
fn scalelinear_range_sets_the_range() {
    let s = scale::linear().range([1.0, 2.0]);
    assert_eq!(s.get_domain().to_vec(), vec![0.0, 1.0]); // d3-scale/test/linear-test.js:16:3
    assert_eq!(s.get_range().to_vec(), vec![1.0, 2.0]); // d3-scale/test/linear-test.js:17:3
    assert_eq!(s.apply(0.5), 1.5); // d3-scale/test/linear-test.js:18:3
}

// d3-scale/test/linear-test.js: scaleLinear(domain, range) sets the domain and range
#[test]
fn scalelinear_domain_range_sets_the_domain_and_range() {
    let s = scale::linear().domain([1.0, 2.0]).range([3.0, 4.0]);
    assert_eq!(s.get_domain().to_vec(), vec![1.0, 2.0]); // d3-scale/test/linear-test.js:23:3
    assert_eq!(s.get_range().to_vec(), vec![3.0, 4.0]); // d3-scale/test/linear-test.js:24:3
    assert_eq!(s.apply(1.5), 3.5); // d3-scale/test/linear-test.js:25:3
}

// d3-scale/test/linear-test.js: linear(x) maps a domain value x to a range value y
#[test]
fn linear_x_maps_a_domain_value_x_to_a_range_value_y() {
    assert_eq!(scale::linear().range([1.0, 2.0]).apply(0.5), 1.5); // d3-scale/test/linear-test.js:29:3
}

// d3-scale/test/linear-test.js: linear(x) maps an empty domain to the middle of the range
#[test]
fn linear_x_maps_an_empty_domain_to_the_middle_of_the_range() {
    assert_eq!(
        scale::linear()
            .domain([0.0, 0.0])
            .range([1.0, 2.0])
            .apply(0.0),
        1.5
    ); // d3-scale/test/linear-test.js:43:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 0.0])
            .range([2.0, 1.0])
            .apply(1.0),
        1.5
    ); // d3-scale/test/linear-test.js:44:3
}

// d3-scale/test/linear-test.js: linear(x) can map a bilinear domain with two values to the corresponding range
#[test]
fn linear_x_can_map_a_bilinear_domain_with_two_values_to_the_corresponding_range() {
    let s = scale::linear().domain([1.0, 2.0]);
    assert_eq!(s.get_domain().to_vec(), vec![1.0, 2.0]); // d3-scale/test/linear-test.js:49:3
    assert_eq!(s.apply(0.5), -0.5); // d3-scale/test/linear-test.js:50:3
    assert_eq!(s.apply(1.0), 0.0); // d3-scale/test/linear-test.js:51:3
    assert_eq!(s.apply(1.5), 0.5); // d3-scale/test/linear-test.js:52:3
    assert_eq!(s.apply(2.0), 1.0); // d3-scale/test/linear-test.js:53:3
    assert_eq!(s.apply(2.5), 1.5); // d3-scale/test/linear-test.js:54:3
    assert_eq!(s.invert(-0.5), 0.5); // d3-scale/test/linear-test.js:55:3
    assert_eq!(s.invert(0.0), 1.0); // d3-scale/test/linear-test.js:56:3
    assert_eq!(s.invert(0.5), 1.5); // d3-scale/test/linear-test.js:57:3
    assert_eq!(s.invert(1.0), 2.0); // d3-scale/test/linear-test.js:58:3
    assert_eq!(s.invert(1.5), 2.5); // d3-scale/test/linear-test.js:59:3
}

// d3-scale/test/linear-test.js: linear(x) can map a polylinear domain with more than two values to the corresponding range
#[test]
fn linear_x_can_map_a_polylinear_domain_with_more_than_two_values_to_the_corresponding_range() {
    let s = scale::piecewise(
        [-10.0, 0.0, 100.0],
        [
            "red".parse::<Rgb>().expect("a CSS color"),
            "white".parse::<Rgb>().expect("a CSS color"),
            "green".parse::<Rgb>().expect("a CSS color"),
        ],
    )
    .domain([-10.0, 0.0, 100.0])
    .range([
        "red".parse::<Rgb>().expect("a CSS color"),
        "white".parse::<Rgb>().expect("a CSS color"),
        "green".parse::<Rgb>().expect("a CSS color"),
    ]);
    assert_eq!(s.get_domain().to_vec(), vec![-10.0, 0.0, 100.0]); // d3-scale/test/linear-test.js:64:3
    assert_eq!(s.apply(-5.0).to_string(), "rgb(255, 128, 128)"); // d3-scale/test/linear-test.js:65:3
    assert_eq!(s.apply(50.0).to_string(), "rgb(128, 192, 128)"); // d3-scale/test/linear-test.js:66:3
    assert_eq!(s.apply(75.0).to_string(), "rgb(64, 160, 64)"); // d3-scale/test/linear-test.js:67:3
    let mut s = s.domain([4.0, 2.0, 1.0]).range([1.0, 2.0, 4.0]);
    assert_eq!(s.apply(1.5), 3.0); // d3-scale/test/linear-test.js:69:3
    assert_eq!(s.apply(3.0), 1.5); // d3-scale/test/linear-test.js:70:3
    assert_eq!(s.invert(1.5), 3.0); // d3-scale/test/linear-test.js:71:3
    assert_eq!(s.invert(3.0), 1.5); // d3-scale/test/linear-test.js:72:3
    s = s.domain([1.0, 2.0, 4.0]).range([4.0, 2.0, 1.0]);
    assert_eq!(s.apply(1.5), 3.0); // d3-scale/test/linear-test.js:74:3
    assert_eq!(s.apply(3.0), 1.5); // d3-scale/test/linear-test.js:75:3
    assert_eq!(s.invert(1.5), 3.0); // d3-scale/test/linear-test.js:76:3
    assert_eq!(s.invert(3.0), 1.5); // d3-scale/test/linear-test.js:77:3
}

// d3-scale/test/linear-test.js: linear.invert(y) maps a range value y to a domain value x
#[test]
fn linear_invert_y_maps_a_range_value_y_to_a_domain_value_x() {
    assert_eq!(scale::linear().range([1.0, 2.0]).invert(1.5), 0.5); // d3-scale/test/linear-test.js:81:3
}

// d3-scale/test/linear-test.js: linear.invert(y) maps an empty range to the middle of the domain
#[test]
fn linear_invert_y_maps_an_empty_range_to_the_middle_of_the_domain() {
    assert_eq!(
        scale::linear()
            .domain([1.0, 2.0])
            .range([0.0, 0.0])
            .invert(0.0),
        1.5
    ); // d3-scale/test/linear-test.js:85:3
    assert_eq!(
        scale::linear()
            .domain([2.0, 1.0])
            .range([0.0, 0.0])
            .invert(1.0),
        1.5
    ); // d3-scale/test/linear-test.js:86:3
}

// d3-scale/test/linear-test.js: linear.domain(domain) accepts an array of numbers
#[test]
fn linear_domain_domain_accepts_an_array_of_numbers() {
    assert_eq!(
        scale::linear().domain([1.0, 0.0]).get_domain().to_vec(),
        vec![1.0, 0.0]
    ); // d3-scale/test/linear-test.js:101:3
}

// d3-scale/test/linear-test.js: linear.range(range) can accept range values as colors
#[test]
fn linear_range_range_can_accept_range_values_as_colors() {
    assert_eq!(
        scale::linear()
            .range([
                "red".parse::<Rgb>().expect("a CSS color"),
                "blue".parse::<Rgb>().expect("a CSS color")
            ])
            .apply(0.5)
            .to_string(),
        "rgb(128, 0, 128)"
    ); // d3-scale/test/linear-test.js:141:3
    assert_eq!(
        scale::linear()
            .range([
                "#ff0000".parse::<Rgb>().expect("a CSS color"),
                "#0000ff".parse::<Rgb>().expect("a CSS color")
            ])
            .apply(0.5)
            .to_string(),
        "rgb(128, 0, 128)"
    ); // d3-scale/test/linear-test.js:142:3
    assert_eq!(
        scale::linear()
            .range([
                "#f00".parse::<Rgb>().expect("a CSS color"),
                "#00f".parse::<Rgb>().expect("a CSS color")
            ])
            .apply(0.5)
            .to_string(),
        "rgb(128, 0, 128)"
    ); // d3-scale/test/linear-test.js:143:3
    assert_eq!(
        scale::linear()
            .range([
                "rgb(255,0,0)".parse::<Rgb>().expect("a CSS color"),
                "hsl(240,100%,50%)".parse::<Rgb>().expect("a CSS color")
            ])
            .apply(0.5)
            .to_string(),
        "rgb(128, 0, 128)"
    ); // d3-scale/test/linear-test.js:144:3
    assert_eq!(
        scale::linear()
            .range([
                "rgb(100%,0%,0%)".parse::<Rgb>().expect("a CSS color"),
                "hsl(240,100%,50%)".parse::<Rgb>().expect("a CSS color")
            ])
            .apply(0.5)
            .to_string(),
        "rgb(128, 0, 128)"
    ); // d3-scale/test/linear-test.js:145:3
    assert_eq!(
        scale::linear()
            .range([
                "hsl(0,100%,50%)".parse::<Rgb>().expect("a CSS color"),
                "hsl(240,100%,50%)".parse::<Rgb>().expect("a CSS color")
            ])
            .apply(0.5)
            .to_string(),
        "rgb(128, 0, 128)"
    ); // d3-scale/test/linear-test.js:146:3
}

// d3-scale/test/linear-test.js: linear.rangeRound(range) is an alias for linear.range(range).interpolate(interpolateRound)
#[test]
fn linear_rangeround_range_is_an_alias_for_linear_range_range_interpolate_interpolateround() {
    assert_eq!(scale::linear().range_round([0.0, 10.0]).apply(0.59), 6.0); // d3-scale/test/linear-test.js:170:3
}

// d3-scale/test/linear-test.js: linear.unknown(value) sets the return value for undefined, null, and NaN input
#[test]
fn linear_unknown_value_sets_the_return_value_for_undefined_null_and_nan_input() {
    let s = scale::linear().unknown(-1.0);
    assert_eq!(s.apply(f64::NAN), -1.0); // d3-scale/test/linear-test.js:181:3
    assert_eq!(s.apply(0.4), 0.4); // d3-scale/test/linear-test.js:183:3
}

// d3-scale/test/linear-test.js: linear.clamp() is false by default
#[test]
fn linear_clamp_is_false_by_default() {
    assert_eq!(scale::linear().range([10.0, 20.0]).apply(2.0), 30.0); // d3-scale/test/linear-test.js:188:3
    assert_eq!(scale::linear().range([10.0, 20.0]).apply(-1.0), 0.0); // d3-scale/test/linear-test.js:189:3
    assert_eq!(scale::linear().range([10.0, 20.0]).invert(30.0), 2.0); // d3-scale/test/linear-test.js:190:3
    assert_eq!(scale::linear().range([10.0, 20.0]).invert(0.0), -1.0); // d3-scale/test/linear-test.js:191:3
}

// d3-scale/test/linear-test.js: linear.clamp(true) restricts output values to the range
#[test]
fn linear_clamp_true_restricts_output_values_to_the_range() {
    assert_eq!(
        scale::linear().clamp(true).range([10.0, 20.0]).apply(2.0),
        20.0
    ); // d3-scale/test/linear-test.js:195:3
    assert_eq!(
        scale::linear().clamp(true).range([10.0, 20.0]).apply(-1.0),
        10.0
    ); // d3-scale/test/linear-test.js:196:3
}

// d3-scale/test/linear-test.js: linear.clamp(true) restricts input values to the domain
#[test]
fn linear_clamp_true_restricts_input_values_to_the_domain() {
    assert_eq!(
        scale::linear().clamp(true).range([10.0, 20.0]).invert(30.0),
        1.0
    ); // d3-scale/test/linear-test.js:200:3
    assert_eq!(
        scale::linear().clamp(true).range([10.0, 20.0]).invert(0.0),
        0.0
    ); // d3-scale/test/linear-test.js:201:3
}

// d3-scale/test/linear-test.js: linear.nice() is an alias for linear.nice(10)
#[test]
fn linear_nice_is_an_alias_for_linear_nice_10() {
    assert_eq!(
        scale::linear()
            .domain([0.0, 0.96])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, 1.0]
    ); // d3-scale/test/linear-test.js:219:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 96.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, 100.0]
    ); // d3-scale/test/linear-test.js:220:3
}

// d3-scale/test/linear-test.js: linear.nice(count) extends the domain to match the desired ticks
#[test]
fn linear_nice_count_extends_the_domain_to_match_the_desired_ticks() {
    assert_eq!(
        scale::linear()
            .domain([0.0, 0.96])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, 1.0]
    ); // d3-scale/test/linear-test.js:224:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 96.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, 100.0]
    ); // d3-scale/test/linear-test.js:225:3
    assert_eq!(
        scale::linear()
            .domain([0.96, 0.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![1.0, 0.0]
    ); // d3-scale/test/linear-test.js:226:3
    assert_eq!(
        scale::linear()
            .domain([96.0, 0.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![100.0, 0.0]
    ); // d3-scale/test/linear-test.js:227:3
    assert_eq!(
        scale::linear()
            .domain([0.0, -0.96])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, -1.0]
    ); // d3-scale/test/linear-test.js:228:3
    assert_eq!(
        scale::linear()
            .domain([0.0, -96.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, -100.0]
    ); // d3-scale/test/linear-test.js:229:3
    assert_eq!(
        scale::linear()
            .domain([-0.96, 0.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![-1.0, 0.0]
    ); // d3-scale/test/linear-test.js:230:3
    assert_eq!(
        scale::linear()
            .domain([-96.0, 0.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![-100.0, 0.0]
    ); // d3-scale/test/linear-test.js:231:3
    assert_eq!(
        scale::linear()
            .domain([-0.1, 51.1])
            .nice(8)
            .get_domain()
            .to_vec(),
        vec![-10.0, 60.0]
    ); // d3-scale/test/linear-test.js:232:3
}

// d3-scale/test/linear-test.js: linear.nice(count) nices the domain, extending it to round numbers
#[test]
fn linear_nice_count_nices_the_domain_extending_it_to_round_numbers() {
    assert_eq!(
        scale::linear()
            .domain([1.1, 10.9])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![1.0, 11.0]
    ); // d3-scale/test/linear-test.js:236:3
    assert_eq!(
        scale::linear()
            .domain([10.9, 1.1])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![11.0, 1.0]
    ); // d3-scale/test/linear-test.js:237:3
    assert_eq!(
        scale::linear()
            .domain([0.7, 11.001])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, 12.0]
    ); // d3-scale/test/linear-test.js:238:3
    assert_eq!(
        scale::linear()
            .domain([123.1, 6.7])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![130.0, 0.0]
    ); // d3-scale/test/linear-test.js:239:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 0.49])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, 0.5]
    ); // d3-scale/test/linear-test.js:240:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 14.1])
            .nice(5)
            .get_domain()
            .to_vec(),
        vec![0.0, 20.0]
    ); // d3-scale/test/linear-test.js:241:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 15.0])
            .nice(5)
            .get_domain()
            .to_vec(),
        vec![0.0, 20.0]
    ); // d3-scale/test/linear-test.js:242:3
}

// d3-scale/test/linear-test.js: linear.nice(count) has no effect on degenerate domains
#[test]
fn linear_nice_count_has_no_effect_on_degenerate_domains() {
    assert_eq!(
        scale::linear()
            .domain([0.0, 0.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, 0.0]
    ); // d3-scale/test/linear-test.js:246:3
    assert_eq!(
        scale::linear()
            .domain([0.5, 0.5])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.5, 0.5]
    ); // d3-scale/test/linear-test.js:247:3
}

// d3-scale/test/linear-test.js: linear.nice(count) accepts a tick count to control nicing step
#[test]
fn linear_nice_count_accepts_a_tick_count_to_control_nicing_step() {
    assert_eq!(
        scale::linear()
            .domain([12.0, 87.0])
            .nice(5)
            .get_domain()
            .to_vec(),
        vec![0.0, 100.0]
    ); // d3-scale/test/linear-test.js:256:3
    assert_eq!(
        scale::linear()
            .domain([12.0, 87.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![10.0, 90.0]
    ); // d3-scale/test/linear-test.js:257:3
    assert_eq!(
        scale::linear()
            .domain([12.0, 87.0])
            .nice(100)
            .get_domain()
            .to_vec(),
        vec![12.0, 87.0]
    ); // d3-scale/test/linear-test.js:258:3
}

// d3-scale/test/linear-test.js: linear.ticks(count) returns the expected ticks for an ascending domain
#[test]
fn linear_ticks_count_returns_the_expected_ticks_for_an_ascending_domain() {
    let mut s = scale::linear();
    assert_eq!(
        s.ticks(10)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    ); // d3-scale/test/linear-test.js:263:3
    assert_eq!(
        s.ticks(9)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    ); // d3-scale/test/linear-test.js:264:3
    assert_eq!(
        s.ticks(8)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    ); // d3-scale/test/linear-test.js:265:3
    assert_eq!(
        s.ticks(7)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
    ); // d3-scale/test/linear-test.js:266:3
    assert_eq!(
        s.ticks(6)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
    ); // d3-scale/test/linear-test.js:267:3
    assert_eq!(
        s.ticks(5)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
    ); // d3-scale/test/linear-test.js:268:3
    assert_eq!(
        s.ticks(4)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
    ); // d3-scale/test/linear-test.js:269:3
    assert_eq!(
        s.ticks(3)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.5, 1.0]
    ); // d3-scale/test/linear-test.js:270:3
    assert_eq!(
        s.ticks(2)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.5, 1.0]
    ); // d3-scale/test/linear-test.js:271:3
    assert_eq!(
        s.ticks(1)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 1.0]
    ); // d3-scale/test/linear-test.js:272:3
    s = s.domain([-100.0, 100.0]);
    assert_eq!(
        s.ticks(10),
        vec![
            -100.0, -80.0, -60.0, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0, 80.0, 100.0
        ]
    ); // d3-scale/test/linear-test.js:274:3
    assert_eq!(
        s.ticks(9),
        vec![
            -100.0, -80.0, -60.0, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0, 80.0, 100.0
        ]
    ); // d3-scale/test/linear-test.js:275:3
    assert_eq!(
        s.ticks(8),
        vec![
            -100.0, -80.0, -60.0, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0, 80.0, 100.0
        ]
    ); // d3-scale/test/linear-test.js:276:3
    assert_eq!(
        s.ticks(7),
        vec![
            -100.0, -80.0, -60.0, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0, 80.0, 100.0
        ]
    ); // d3-scale/test/linear-test.js:277:3
    assert_eq!(s.ticks(6), vec![-100.0, -50.0, 0.0, 50.0, 100.0]); // d3-scale/test/linear-test.js:278:3
    assert_eq!(s.ticks(5), vec![-100.0, -50.0, 0.0, 50.0, 100.0]); // d3-scale/test/linear-test.js:279:3
    assert_eq!(s.ticks(4), vec![-100.0, -50.0, 0.0, 50.0, 100.0]); // d3-scale/test/linear-test.js:280:3
    assert_eq!(s.ticks(3), vec![-100.0, -50.0, 0.0, 50.0, 100.0]); // d3-scale/test/linear-test.js:281:3
    assert_eq!(s.ticks(2), vec![-100.0, 0.0, 100.0]); // d3-scale/test/linear-test.js:282:3
    assert_eq!(s.ticks(1), vec![0.0]); // d3-scale/test/linear-test.js:283:3
}

// d3-scale/test/linear-test.js: linear.ticks(count) returns the expected ticks for a descending domain
#[test]
fn linear_ticks_count_returns_the_expected_ticks_for_a_descending_domain() {
    let mut s = scale::linear().domain([1.0, 0.0]);
    assert_eq!(
        s.ticks(10)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/linear-test.js:288:3
    assert_eq!(
        s.ticks(9)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/linear-test.js:289:3
    assert_eq!(
        s.ticks(8)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/linear-test.js:290:3
    assert_eq!(
        s.ticks(7)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/linear-test.js:291:3
    assert_eq!(
        s.ticks(6)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/linear-test.js:292:3
    assert_eq!(
        s.ticks(5)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/linear-test.js:293:3
    assert_eq!(
        s.ticks(4)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/linear-test.js:294:3
    assert_eq!(
        s.ticks(3)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.5, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/linear-test.js:295:3
    assert_eq!(
        s.ticks(2)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.5, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/linear-test.js:296:3
    assert_eq!(
        s.ticks(1)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/linear-test.js:297:3
    s = s.domain([100.0, -100.0]);
    assert_eq!(s.ticks(10), {
        let mut v = vec![
            -100.0, -80.0, -60.0, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0, 80.0, 100.0,
        ];
        v.reverse();
        v
    }); // d3-scale/test/linear-test.js:299:3
    assert_eq!(s.ticks(9), {
        let mut v = vec![
            -100.0, -80.0, -60.0, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0, 80.0, 100.0,
        ];
        v.reverse();
        v
    }); // d3-scale/test/linear-test.js:300:3
    assert_eq!(s.ticks(8), {
        let mut v = vec![
            -100.0, -80.0, -60.0, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0, 80.0, 100.0,
        ];
        v.reverse();
        v
    }); // d3-scale/test/linear-test.js:301:3
    assert_eq!(s.ticks(7), {
        let mut v = vec![
            -100.0, -80.0, -60.0, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0, 80.0, 100.0,
        ];
        v.reverse();
        v
    }); // d3-scale/test/linear-test.js:302:3
    assert_eq!(s.ticks(6), {
        let mut v = vec![-100.0, -50.0, 0.0, 50.0, 100.0];
        v.reverse();
        v
    }); // d3-scale/test/linear-test.js:303:3
    assert_eq!(s.ticks(5), {
        let mut v = vec![-100.0, -50.0, 0.0, 50.0, 100.0];
        v.reverse();
        v
    }); // d3-scale/test/linear-test.js:304:3
    assert_eq!(s.ticks(4), {
        let mut v = vec![-100.0, -50.0, 0.0, 50.0, 100.0];
        v.reverse();
        v
    }); // d3-scale/test/linear-test.js:305:3
    assert_eq!(s.ticks(3), {
        let mut v = vec![-100.0, -50.0, 0.0, 50.0, 100.0];
        v.reverse();
        v
    }); // d3-scale/test/linear-test.js:306:3
    assert_eq!(s.ticks(2), {
        let mut v = vec![-100.0, 0.0, 100.0];
        v.reverse();
        v
    }); // d3-scale/test/linear-test.js:307:3
    assert_eq!(s.ticks(1), {
        let mut v = vec![0.0];
        v.reverse();
        v
    }); // d3-scale/test/linear-test.js:308:3
}

// d3-scale/test/linear-test.js: linear.ticks(X) spans linear.nice(X).domain()
#[test]
fn linear_ticks_x_spans_linear_nice_x_domain() {
    {
        let s = scale::linear().domain([1.0, 9.0]).nice(2);
        let ticks = s.ticks(2);
        assert_eq!(
            vec![
                ticks[(0.0) as usize],
                ticks[((ticks.len() as f64) - 1.0) as usize]
            ],
            s.get_domain().to_vec()
        ); // d3-scale/test/linear-test.js:340:5
    }
    {
        let s = scale::linear().domain([1.0, 9.0]).nice(3);
        let ticks = s.ticks(3);
        assert_eq!(
            vec![
                ticks[(0.0) as usize],
                ticks[((ticks.len() as f64) - 1.0) as usize]
            ],
            s.get_domain().to_vec()
        ); // d3-scale/test/linear-test.js:340:5
    }
    {
        let s = scale::linear().domain([1.0, 9.0]).nice(4);
        let ticks = s.ticks(4);
        assert_eq!(
            vec![
                ticks[(0.0) as usize],
                ticks[((ticks.len() as f64) - 1.0) as usize]
            ],
            s.get_domain().to_vec()
        ); // d3-scale/test/linear-test.js:340:5
    }
    {
        let s = scale::linear().domain([8.0, 9.0]).nice(2);
        let ticks = s.ticks(2);
        assert_eq!(
            vec![
                ticks[(0.0) as usize],
                ticks[((ticks.len() as f64) - 1.0) as usize]
            ],
            s.get_domain().to_vec()
        ); // d3-scale/test/linear-test.js:340:5
    }
    {
        let s = scale::linear().domain([8.0, 9.0]).nice(3);
        let ticks = s.ticks(3);
        assert_eq!(
            vec![
                ticks[(0.0) as usize],
                ticks[((ticks.len() as f64) - 1.0) as usize]
            ],
            s.get_domain().to_vec()
        ); // d3-scale/test/linear-test.js:340:5
    }
    {
        let s = scale::linear().domain([8.0, 9.0]).nice(4);
        let ticks = s.ticks(4);
        assert_eq!(
            vec![
                ticks[(0.0) as usize],
                ticks[((ticks.len() as f64) - 1.0) as usize]
            ],
            s.get_domain().to_vec()
        ); // d3-scale/test/linear-test.js:340:5
    }
    {
        let s = scale::linear().domain([1.0, 21.0]).nice(2);
        let ticks = s.ticks(2);
        assert_eq!(
            vec![
                ticks[(0.0) as usize],
                ticks[((ticks.len() as f64) - 1.0) as usize]
            ],
            s.get_domain().to_vec()
        ); // d3-scale/test/linear-test.js:340:5
    }
    {
        let s = scale::linear().domain([2.0, 21.0]).nice(2);
        let ticks = s.ticks(2);
        assert_eq!(
            vec![
                ticks[(0.0) as usize],
                ticks[((ticks.len() as f64) - 1.0) as usize]
            ],
            s.get_domain().to_vec()
        ); // d3-scale/test/linear-test.js:340:5
    }
    {
        let s = scale::linear().domain([3.0, 21.0]).nice(2);
        let ticks = s.ticks(2);
        assert_eq!(
            vec![
                ticks[(0.0) as usize],
                ticks[((ticks.len() as f64) - 1.0) as usize]
            ],
            s.get_domain().to_vec()
        ); // d3-scale/test/linear-test.js:340:5
    }
    {
        let s = scale::linear().domain([4.0, 21.0]).nice(2);
        let ticks = s.ticks(2);
        assert_eq!(
            vec![
                ticks[(0.0) as usize],
                ticks[((ticks.len() as f64) - 1.0) as usize]
            ],
            s.get_domain().to_vec()
        ); // d3-scale/test/linear-test.js:340:5
    }
    {
        let s = scale::linear().domain([5.0, 21.0]).nice(2);
        let ticks = s.ticks(2);
        assert_eq!(
            vec![
                ticks[(0.0) as usize],
                ticks[((ticks.len() as f64) - 1.0) as usize]
            ],
            s.get_domain().to_vec()
        ); // d3-scale/test/linear-test.js:340:5
    }
    {
        let s = scale::linear().domain([6.0, 21.0]).nice(2);
        let ticks = s.ticks(2);
        assert_eq!(
            vec![
                ticks[(0.0) as usize],
                ticks[((ticks.len() as f64) - 1.0) as usize]
            ],
            s.get_domain().to_vec()
        ); // d3-scale/test/linear-test.js:340:5
    }
    {
        let s = scale::linear().domain([7.0, 21.0]).nice(2);
        let ticks = s.ticks(2);
        assert_eq!(
            vec![
                ticks[(0.0) as usize],
                ticks[((ticks.len() as f64) - 1.0) as usize]
            ],
            s.get_domain().to_vec()
        ); // d3-scale/test/linear-test.js:340:5
    }
    {
        let s = scale::linear().domain([8.0, 21.0]).nice(2);
        let ticks = s.ticks(2);
        assert_eq!(
            vec![
                ticks[(0.0) as usize],
                ticks[((ticks.len() as f64) - 1.0) as usize]
            ],
            s.get_domain().to_vec()
        ); // d3-scale/test/linear-test.js:340:5
    }
    {
        let s = scale::linear().domain([9.0, 21.0]).nice(2);
        let ticks = s.ticks(2);
        assert_eq!(
            vec![
                ticks[(0.0) as usize],
                ticks[((ticks.len() as f64) - 1.0) as usize]
            ],
            s.get_domain().to_vec()
        ); // d3-scale/test/linear-test.js:340:5
    }
    {
        let s = scale::linear().domain([10.0, 21.0]).nice(2);
        let ticks = s.ticks(2);
        assert_eq!(
            vec![
                ticks[(0.0) as usize],
                ticks[((ticks.len() as f64) - 1.0) as usize]
            ],
            s.get_domain().to_vec()
        ); // d3-scale/test/linear-test.js:340:5
    }
    {
        let s = scale::linear().domain([11.0, 21.0]).nice(2);
        let ticks = s.ticks(2);
        assert_eq!(
            vec![
                ticks[(0.0) as usize],
                ticks[((ticks.len() as f64) - 1.0) as usize]
            ],
            s.get_domain().to_vec()
        ); // d3-scale/test/linear-test.js:340:5
    }
}

// d3-scale/test/linear-test.js: linear.ticks(count) returns the empty array if count is not a positive integer
#[test]
fn linear_ticks_count_returns_the_empty_array_if_count_is_not_a_positive_integer() {
    let s = scale::linear();
    assert_eq!(s.ticks(0), Vec::<f64>::new()); // d3-scale/test/linear-test.js:364:3
}

// d3-scale/test/linear-test.js: linear.ticks() is an alias for linear.ticks(10)
#[test]
fn linear_ticks_is_an_alias_for_linear_ticks_10() {
    let s = scale::linear();
    assert_eq!(s.ticks(10), s.ticks(10)); // d3-scale/test/linear-test.js:371:3
}

// d3-scale/test/linear-test.js: linear.tickFormat() is an alias for linear.tickFormat(10)
#[test]
fn linear_tickformat_is_an_alias_for_linear_tickformat_10() {
    assert_eq!(
        scale::linear()
            .tick_format_in(&format::Locale::EN_US, 10, None)
            .expect("d3 test")
            .format(0.2),
        "0.2"
    ); // d3-scale/test/linear-test.js:375:3
    assert_eq!(
        scale::linear()
            .domain([-100.0, 100.0])
            .tick_format_in(&format::Locale::EN_US, 10, None)
            .expect("d3 test")
            .format(-20.0),
        "−20"
    ); // d3-scale/test/linear-test.js:376:3
}

// d3-scale/test/linear-test.js: linear.tickFormat(count) returns a format suitable for the ticks
#[test]
fn linear_tickformat_count_returns_a_format_suitable_for_the_ticks() {
    assert_eq!(
        scale::linear()
            .tick_format_in(&format::Locale::EN_US, 10, None)
            .expect("d3 test")
            .format(0.2),
        "0.2"
    ); // d3-scale/test/linear-test.js:380:3
    assert_eq!(
        scale::linear()
            .tick_format_in(&format::Locale::EN_US, 20, None)
            .expect("d3 test")
            .format(0.2),
        "0.20"
    ); // d3-scale/test/linear-test.js:381:3
    assert_eq!(
        scale::linear()
            .domain([-100.0, 100.0])
            .tick_format_in(&format::Locale::EN_US, 10, None)
            .expect("d3 test")
            .format(-20.0),
        "−20"
    ); // d3-scale/test/linear-test.js:382:3
}

// d3-scale/test/linear-test.js: linear.tickFormat(count, specifier) sets the appropriate fixed precision if not specified
#[test]
fn linear_tickformat_count_specifier_sets_the_appropriate_fixed_precision_if_not_specified() {
    assert_eq!(
        scale::linear()
            .tick_format_in(&format::Locale::EN_US, 10, Some("+f"))
            .expect("d3 test")
            .format(0.2),
        "+0.2"
    ); // d3-scale/test/linear-test.js:386:3
    assert_eq!(
        scale::linear()
            .tick_format_in(&format::Locale::EN_US, 20, Some("+f"))
            .expect("d3 test")
            .format(0.2),
        "+0.20"
    ); // d3-scale/test/linear-test.js:387:3
    assert_eq!(
        scale::linear()
            .tick_format_in(&format::Locale::EN_US, 10, Some("+%"))
            .expect("d3 test")
            .format(0.2),
        "+20%"
    ); // d3-scale/test/linear-test.js:388:3
    assert_eq!(
        scale::linear()
            .domain([0.19, 0.21])
            .tick_format_in(&format::Locale::EN_US, 10, Some("+%"))
            .expect("d3 test")
            .format(0.2),
        "+20.0%"
    ); // d3-scale/test/linear-test.js:389:3
}

// d3-scale/test/linear-test.js: linear.tickFormat(count, specifier) sets the appropriate round precision if not specified
#[test]
fn linear_tickformat_count_specifier_sets_the_appropriate_round_precision_if_not_specified() {
    assert_eq!(
        scale::linear()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 10, Some(""))
            .expect("d3 test")
            .format(2.10),
        "2"
    ); // d3-scale/test/linear-test.js:393:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 100, Some(""))
            .expect("d3 test")
            .format(2.01),
        "2"
    ); // d3-scale/test/linear-test.js:394:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 100, Some(""))
            .expect("d3 test")
            .format(2.11),
        "2.1"
    ); // d3-scale/test/linear-test.js:395:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 10, Some("e"))
            .expect("d3 test")
            .format(2.10),
        "2e+0"
    ); // d3-scale/test/linear-test.js:396:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 100, Some("e"))
            .expect("d3 test")
            .format(2.01),
        "2.0e+0"
    ); // d3-scale/test/linear-test.js:397:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 100, Some("e"))
            .expect("d3 test")
            .format(2.11),
        "2.1e+0"
    ); // d3-scale/test/linear-test.js:398:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 10, Some("g"))
            .expect("d3 test")
            .format(2.10),
        "2"
    ); // d3-scale/test/linear-test.js:399:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 100, Some("g"))
            .expect("d3 test")
            .format(2.01),
        "2.0"
    ); // d3-scale/test/linear-test.js:400:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 100, Some("g"))
            .expect("d3 test")
            .format(2.11),
        "2.1"
    ); // d3-scale/test/linear-test.js:401:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 10, Some("r"))
            .expect("d3 test")
            .format(2.10e6),
        "2000000"
    ); // d3-scale/test/linear-test.js:402:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 100, Some("r"))
            .expect("d3 test")
            .format(2.01e6),
        "2000000"
    ); // d3-scale/test/linear-test.js:403:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 100, Some("r"))
            .expect("d3 test")
            .format(2.11e6),
        "2100000"
    ); // d3-scale/test/linear-test.js:404:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 0.9])
            .tick_format_in(&format::Locale::EN_US, 10, Some("p"))
            .expect("d3 test")
            .format(0.210),
        "20%"
    ); // d3-scale/test/linear-test.js:405:3
    assert_eq!(
        scale::linear()
            .domain([0.19, 0.21])
            .tick_format_in(&format::Locale::EN_US, 10, Some("p"))
            .expect("d3 test")
            .format(0.201),
        "20.1%"
    ); // d3-scale/test/linear-test.js:406:3
}

// d3-scale/test/linear-test.js: linear.tickFormat(count, specifier) sets the appropriate prefix precision if not specified
#[test]
fn linear_tickformat_count_specifier_sets_the_appropriate_prefix_precision_if_not_specified() {
    assert_eq!(
        scale::linear()
            .domain([0.0, 1e6])
            .tick_format_in(&format::Locale::EN_US, 10, Some("$s"))
            .expect("d3 test")
            .format(0.51e6),
        "$0.5M"
    ); // d3-scale/test/linear-test.js:410:3
    assert_eq!(
        scale::linear()
            .domain([0.0, 1e6])
            .tick_format_in(&format::Locale::EN_US, 100, Some("$s"))
            .expect("d3 test")
            .format(0.501e6),
        "$0.50M"
    ); // d3-scale/test/linear-test.js:411:3
}

// d3-scale/test/linear-test.js: linear.tickFormat() uses the default precision when the domain is invalid
#[test]
fn linear_tickformat_uses_the_default_precision_when_the_domain_is_invalid() {
    let f = scale::linear()
        .domain([0.0, f64::NAN])
        .tick_format_in(&format::Locale::EN_US, 10, None)
        .expect("d3 test");
    assert_eq!(f.format(0.12), "0.120000"); // d3-scale/test/linear-test.js:417:3
}

// d3-scale/test/linear-test.js: linear.copy() returns a copy with changes to the domain are isolated
#[test]
fn linear_copy_returns_a_copy_with_changes_to_the_domain_are_isolated() {
    let mut x = scale::linear();
    let mut y = x.clone();
    x = x.domain([1.0, 2.0]);
    assert_eq!(y.get_domain().to_vec(), vec![0.0, 1.0]); // d3-scale/test/linear-test.js:423:3
    assert_eq!(x.apply(1.0), 0.0); // d3-scale/test/linear-test.js:424:3
    assert_eq!(y.apply(1.0), 1.0); // d3-scale/test/linear-test.js:425:3
    y = y.domain([2.0, 3.0]);
    assert_eq!(x.apply(2.0), 1.0); // d3-scale/test/linear-test.js:427:3
    assert_eq!(y.apply(2.0), 0.0); // d3-scale/test/linear-test.js:428:3
    assert_eq!(x.get_domain().to_vec(), vec![1.0, 2.0]); // d3-scale/test/linear-test.js:429:3
    assert_eq!(y.get_domain().to_vec(), vec![2.0, 3.0]); // d3-scale/test/linear-test.js:430:3
    let y2 = {
        x = x.domain([1.0, 1.9]);
        x.clone()
    };
    x = x.nice(5);
    assert_eq!(x.get_domain().to_vec(), vec![1.0, 2.0]); // d3-scale/test/linear-test.js:433:3
    assert_eq!(y2.get_domain().to_vec(), vec![1.0, 1.9]); // d3-scale/test/linear-test.js:434:3
}

// d3-scale/test/linear-test.js: linear.copy() returns a copy with changes to the range are isolated
#[test]
fn linear_copy_returns_a_copy_with_changes_to_the_range_are_isolated() {
    let mut x = scale::linear();
    let mut y = x.clone();
    x = x.range([1.0, 2.0]);
    assert_eq!(x.invert(1.0), 0.0); // d3-scale/test/linear-test.js:440:3
    assert_eq!(y.invert(1.0), 1.0); // d3-scale/test/linear-test.js:441:3
    assert_eq!(y.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/linear-test.js:442:3
    y = y.range([2.0, 3.0]);
    assert_eq!(x.invert(2.0), 1.0); // d3-scale/test/linear-test.js:444:3
    assert_eq!(y.invert(2.0), 0.0); // d3-scale/test/linear-test.js:445:3
    assert_eq!(x.get_range().to_vec(), vec![1.0, 2.0]); // d3-scale/test/linear-test.js:446:3
    assert_eq!(y.get_range().to_vec(), vec![2.0, 3.0]); // d3-scale/test/linear-test.js:447:3
}

// d3-scale/test/linear-test.js: linear.copy() returns a copy with changes to clamping are isolated
#[test]
fn linear_copy_returns_a_copy_with_changes_to_clamping_are_isolated() {
    let mut x = scale::linear().clamp(true);
    let mut y = x.clone();
    x = x.clamp(false);
    assert_eq!(x.apply(2.0), 2.0); // d3-scale/test/linear-test.js:464:3
    assert_eq!(y.apply(2.0), 1.0); // d3-scale/test/linear-test.js:465:3
    y = y.clamp(false);
    assert_eq!(x.apply(2.0), 2.0); // d3-scale/test/linear-test.js:468:3
    assert_eq!(y.apply(2.0), 2.0); // d3-scale/test/linear-test.js:469:3
}

// d3-scale/test/linear-test.js: linear.copy() returns a copy with changes to the unknown value are isolated
#[test]
fn linear_copy_returns_a_copy_with_changes_to_the_unknown_value_are_isolated() {
    let mut x = scale::linear();
    let mut y = x.clone();
    x = x.unknown(2.0);
    assert_eq!(x.apply(f64::NAN), 2.0); // d3-scale/test/linear-test.js:476:3
    assert!((y.apply(f64::NAN)).is_nan()); // d3-scale/test/linear-test.js:477:3
    y = y.unknown(3.0);
    assert_eq!(x.apply(f64::NAN), 2.0); // d3-scale/test/linear-test.js:480:3
    assert_eq!(y.apply(f64::NAN), 3.0); // d3-scale/test/linear-test.js:481:3
}

// d3-scale/test/pow-test.js: scalePow() has the expected defaults
#[test]
fn scalepow_has_the_expected_defaults() {
    let s = scale::pow();
    assert_eq!(s.get_domain().to_vec(), vec![0.0, 1.0]); // d3-scale/test/pow-test.js:8:3
    assert_eq!(s.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/pow-test.js:9:3
}

// d3-scale/test/pow-test.js: pow(x) maps a domain value x to a range value y
#[test]
fn pow_x_maps_a_domain_value_x_to_a_range_value_y() {
    assert_eq!(
        scale::pow().exponent(0.5).apply(0.5),
        std::f64::consts::FRAC_1_SQRT_2
    ); // d3-scale/test/pow-test.js:16:3
}

// d3-scale/test/pow-test.js: pow(x) maps an empty domain to the middle of the range
#[test]
fn pow_x_maps_an_empty_domain_to_the_middle_of_the_range() {
    assert_eq!(
        scale::pow().domain([0.0, 0.0]).range([1.0, 2.0]).apply(0.0),
        1.5
    ); // d3-scale/test/pow-test.js:30:3
    assert_eq!(
        scale::pow().domain([0.0, 0.0]).range([2.0, 1.0]).apply(1.0),
        1.5
    ); // d3-scale/test/pow-test.js:31:3
}

// d3-scale/test/pow-test.js: pow(x) can map a bipow domain with two values to the corresponding range
#[test]
fn pow_x_can_map_a_bipow_domain_with_two_values_to_the_corresponding_range() {
    let s = scale::pow().domain([1.0, 2.0]);
    assert_eq!(s.get_domain().to_vec(), vec![1.0, 2.0]); // d3-scale/test/pow-test.js:36:3
    assert_eq!(s.apply(0.5), -0.5); // d3-scale/test/pow-test.js:37:3
    assert_eq!(s.apply(1.0), 0.0); // d3-scale/test/pow-test.js:38:3
    assert_eq!(s.apply(1.5), 0.5); // d3-scale/test/pow-test.js:39:3
    assert_eq!(s.apply(2.0), 1.0); // d3-scale/test/pow-test.js:40:3
    assert_eq!(s.apply(2.5), 1.5); // d3-scale/test/pow-test.js:41:3
    assert_eq!(s.invert(-0.5), 0.5); // d3-scale/test/pow-test.js:42:3
    assert_eq!(s.invert(0.0), 1.0); // d3-scale/test/pow-test.js:43:3
    assert_eq!(s.invert(0.5), 1.5); // d3-scale/test/pow-test.js:44:3
    assert_eq!(s.invert(1.0), 2.0); // d3-scale/test/pow-test.js:45:3
    assert_eq!(s.invert(1.5), 2.5); // d3-scale/test/pow-test.js:46:3
}

// d3-scale/test/pow-test.js: pow.invert(y) maps a range value y to a domain value x
#[test]
fn pow_invert_y_maps_a_range_value_y_to_a_domain_value_x() {
    assert_eq!(scale::pow().range([1.0, 2.0]).invert(1.5), 0.5); // d3-scale/test/pow-test.js:68:3
}

// d3-scale/test/pow-test.js: pow.invert(y) maps an empty range to the middle of the domain
#[test]
fn pow_invert_y_maps_an_empty_range_to_the_middle_of_the_domain() {
    assert_eq!(
        scale::pow()
            .domain([1.0, 2.0])
            .range([0.0, 0.0])
            .invert(0.0),
        1.5
    ); // d3-scale/test/pow-test.js:72:3
    assert_eq!(
        scale::pow()
            .domain([2.0, 1.0])
            .range([0.0, 0.0])
            .invert(1.0),
        1.5
    ); // d3-scale/test/pow-test.js:73:3
}

// d3-scale/test/pow-test.js: pow.exponent(exponent) sets the exponent to the specified value
#[test]
fn pow_exponent_exponent_sets_the_exponent_to_the_specified_value() {
    let mut x = scale::pow().exponent(0.5).domain([1.0, 2.0]);
    assert_in_delta(&x.apply(1.0), &0.0, 1e-6); // d3-scale/test/pow-test.js:88:3
    assert_in_delta(&x.apply(1.5), &0.5425821, 1e-6); // d3-scale/test/pow-test.js:89:3
    assert_in_delta(&x.apply(2.0), &1.0, 1e-6); // d3-scale/test/pow-test.js:90:3
    x = x.exponent(2.0).domain([1.0, 2.0]);
    assert_in_delta(&x.apply(1.0), &0.0, 1e-6); // d3-scale/test/pow-test.js:93:3
    assert_in_delta(&x.apply(1.5), &0.41666667, 1e-6); // d3-scale/test/pow-test.js:94:3
    assert_in_delta(&x.apply(2.0), &1.0, 1e-6); // d3-scale/test/pow-test.js:95:3
    x = x.exponent(-1.0).domain([1.0, 2.0]);
    assert_in_delta(&x.apply(1.0), &0.0, 1e-6); // d3-scale/test/pow-test.js:98:3
    assert_in_delta(&x.apply(1.5), &0.6666667, 1e-6); // d3-scale/test/pow-test.js:99:3
    assert_in_delta(&x.apply(2.0), &1.0, 1e-6); // d3-scale/test/pow-test.js:100:3
}

// d3-scale/test/pow-test.js: pow.exponent(exponent) changing the exponent does not change the domain or range
#[test]
fn pow_exponent_exponent_changing_the_exponent_does_not_change_the_domain_or_range() {
    let mut x = scale::pow().domain([1.0, 2.0]).range([3.0, 4.0]);
    x = x.exponent(0.5);
    assert_eq!(x.get_domain().to_vec(), vec![1.0, 2.0]); // d3-scale/test/pow-test.js:107:3
    assert_eq!(x.get_range().to_vec(), vec![3.0, 4.0]); // d3-scale/test/pow-test.js:108:3
    x = x.exponent(2.0);
    assert_eq!(x.get_domain().to_vec(), vec![1.0, 2.0]); // d3-scale/test/pow-test.js:110:3
    assert_eq!(x.get_range().to_vec(), vec![3.0, 4.0]); // d3-scale/test/pow-test.js:111:3
    x = x.exponent(-1.0);
    assert_eq!(x.get_domain().to_vec(), vec![1.0, 2.0]); // d3-scale/test/pow-test.js:113:3
    assert_eq!(x.get_range().to_vec(), vec![3.0, 4.0]); // d3-scale/test/pow-test.js:114:3
}

// d3-scale/test/pow-test.js: pow.domain(domain) accepts an array of numbers
#[test]
fn pow_domain_domain_accepts_an_array_of_numbers() {
    assert_eq!(
        scale::pow().domain([1.0, 0.0]).get_domain().to_vec(),
        vec![1.0, 0.0]
    ); // d3-scale/test/pow-test.js:119:3
}

// d3-scale/test/pow-test.js: pow.range(range) can accept range values as colors
#[test]
fn pow_range_range_can_accept_range_values_as_colors() {
    assert_eq!(
        scale::pow()
            .range([
                "red".parse::<Rgb>().expect("a CSS color"),
                "blue".parse::<Rgb>().expect("a CSS color")
            ])
            .apply(0.5)
            .to_string(),
        "rgb(128, 0, 128)"
    ); // d3-scale/test/pow-test.js:151:3
    assert_eq!(
        scale::pow()
            .range([
                "#ff0000".parse::<Rgb>().expect("a CSS color"),
                "#0000ff".parse::<Rgb>().expect("a CSS color")
            ])
            .apply(0.5)
            .to_string(),
        "rgb(128, 0, 128)"
    ); // d3-scale/test/pow-test.js:152:3
    assert_eq!(
        scale::pow()
            .range([
                "#f00".parse::<Rgb>().expect("a CSS color"),
                "#00f".parse::<Rgb>().expect("a CSS color")
            ])
            .apply(0.5)
            .to_string(),
        "rgb(128, 0, 128)"
    ); // d3-scale/test/pow-test.js:153:3
    assert_eq!(
        scale::pow()
            .range([
                "rgb(255,0,0)".parse::<Rgb>().expect("a CSS color"),
                "hsl(240,100%,50%)".parse::<Rgb>().expect("a CSS color")
            ])
            .apply(0.5)
            .to_string(),
        "rgb(128, 0, 128)"
    ); // d3-scale/test/pow-test.js:154:3
    assert_eq!(
        scale::pow()
            .range([
                "rgb(100%,0%,0%)".parse::<Rgb>().expect("a CSS color"),
                "hsl(240,100%,50%)".parse::<Rgb>().expect("a CSS color")
            ])
            .apply(0.5)
            .to_string(),
        "rgb(128, 0, 128)"
    ); // d3-scale/test/pow-test.js:155:3
    assert_eq!(
        scale::pow()
            .range([
                "hsl(0,100%,50%)".parse::<Rgb>().expect("a CSS color"),
                "hsl(240,100%,50%)".parse::<Rgb>().expect("a CSS color")
            ])
            .apply(0.5)
            .to_string(),
        "rgb(128, 0, 128)"
    ); // d3-scale/test/pow-test.js:156:3
}

// d3-scale/test/pow-test.js: pow.rangeRound(range) is an alias for pow.range(range).interpolate(interpolateRound)
#[test]
fn pow_rangeround_range_is_an_alias_for_pow_range_range_interpolate_interpolateround() {
    assert_eq!(scale::pow().range_round([0.0, 10.0]).apply(0.59), 6.0); // d3-scale/test/pow-test.js:180:3
}

// d3-scale/test/pow-test.js: pow.clamp() is false by default
#[test]
fn pow_clamp_is_false_by_default() {
    assert_eq!(scale::pow().range([10.0, 20.0]).apply(2.0), 30.0); // d3-scale/test/pow-test.js:185:3
    assert_eq!(scale::pow().range([10.0, 20.0]).apply(-1.0), 0.0); // d3-scale/test/pow-test.js:186:3
    assert_eq!(scale::pow().range([10.0, 20.0]).invert(30.0), 2.0); // d3-scale/test/pow-test.js:187:3
    assert_eq!(scale::pow().range([10.0, 20.0]).invert(0.0), -1.0); // d3-scale/test/pow-test.js:188:3
}

// d3-scale/test/pow-test.js: pow.clamp(true) restricts output values to the range
#[test]
fn pow_clamp_true_restricts_output_values_to_the_range() {
    assert_eq!(
        scale::pow().clamp(true).range([10.0, 20.0]).apply(2.0),
        20.0
    ); // d3-scale/test/pow-test.js:192:3
    assert_eq!(
        scale::pow().clamp(true).range([10.0, 20.0]).apply(-1.0),
        10.0
    ); // d3-scale/test/pow-test.js:193:3
}

// d3-scale/test/pow-test.js: pow.clamp(true) restricts input values to the domain
#[test]
fn pow_clamp_true_restricts_input_values_to_the_domain() {
    assert_eq!(
        scale::pow().clamp(true).range([10.0, 20.0]).invert(30.0),
        1.0
    ); // d3-scale/test/pow-test.js:197:3
    assert_eq!(
        scale::pow().clamp(true).range([10.0, 20.0]).invert(0.0),
        0.0
    ); // d3-scale/test/pow-test.js:198:3
}

// d3-scale/test/pow-test.js: pow.nice() is an alias for pow.nice(10)
#[test]
fn pow_nice_is_an_alias_for_pow_nice_10() {
    assert_eq!(
        scale::pow()
            .domain([0.0, 0.96])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, 1.0]
    ); // d3-scale/test/pow-test.js:216:3
    assert_eq!(
        scale::pow()
            .domain([0.0, 96.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, 100.0]
    ); // d3-scale/test/pow-test.js:217:3
}

// d3-scale/test/pow-test.js: pow.nice(count) extends the domain to match the desired ticks
#[test]
fn pow_nice_count_extends_the_domain_to_match_the_desired_ticks() {
    assert_eq!(
        scale::pow()
            .domain([0.0, 0.96])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, 1.0]
    ); // d3-scale/test/pow-test.js:221:3
    assert_eq!(
        scale::pow()
            .domain([0.0, 96.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, 100.0]
    ); // d3-scale/test/pow-test.js:222:3
    assert_eq!(
        scale::pow()
            .domain([0.96, 0.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![1.0, 0.0]
    ); // d3-scale/test/pow-test.js:223:3
    assert_eq!(
        scale::pow()
            .domain([96.0, 0.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![100.0, 0.0]
    ); // d3-scale/test/pow-test.js:224:3
    assert_eq!(
        scale::pow()
            .domain([0.0, -0.96])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, -1.0]
    ); // d3-scale/test/pow-test.js:225:3
    assert_eq!(
        scale::pow()
            .domain([0.0, -96.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, -100.0]
    ); // d3-scale/test/pow-test.js:226:3
    assert_eq!(
        scale::pow()
            .domain([-0.96, 0.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![-1.0, 0.0]
    ); // d3-scale/test/pow-test.js:227:3
    assert_eq!(
        scale::pow()
            .domain([-96.0, 0.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![-100.0, 0.0]
    ); // d3-scale/test/pow-test.js:228:3
}

// d3-scale/test/pow-test.js: pow.nice(count) nices the domain, extending it to round numbers
#[test]
fn pow_nice_count_nices_the_domain_extending_it_to_round_numbers() {
    assert_eq!(
        scale::pow()
            .domain([1.1, 10.9])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![1.0, 11.0]
    ); // d3-scale/test/pow-test.js:232:3
    assert_eq!(
        scale::pow()
            .domain([10.9, 1.1])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![11.0, 1.0]
    ); // d3-scale/test/pow-test.js:233:3
    assert_eq!(
        scale::pow()
            .domain([0.7, 11.001])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, 12.0]
    ); // d3-scale/test/pow-test.js:234:3
    assert_eq!(
        scale::pow()
            .domain([123.1, 6.7])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![130.0, 0.0]
    ); // d3-scale/test/pow-test.js:235:3
    assert_eq!(
        scale::pow()
            .domain([0.0, 0.49])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, 0.5]
    ); // d3-scale/test/pow-test.js:236:3
}

// d3-scale/test/pow-test.js: pow.nice(count) has no effect on degenerate domains
#[test]
fn pow_nice_count_has_no_effect_on_degenerate_domains() {
    assert_eq!(
        scale::pow()
            .domain([0.0, 0.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.0, 0.0]
    ); // d3-scale/test/pow-test.js:240:3
    assert_eq!(
        scale::pow()
            .domain([0.5, 0.5])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![0.5, 0.5]
    ); // d3-scale/test/pow-test.js:241:3
}

// d3-scale/test/pow-test.js: pow.nice(count) accepts a tick count to control nicing step
#[test]
fn pow_nice_count_accepts_a_tick_count_to_control_nicing_step() {
    assert_eq!(
        scale::pow()
            .domain([12.0, 87.0])
            .nice(5)
            .get_domain()
            .to_vec(),
        vec![0.0, 100.0]
    ); // d3-scale/test/pow-test.js:250:3
    assert_eq!(
        scale::pow()
            .domain([12.0, 87.0])
            .nice(10)
            .get_domain()
            .to_vec(),
        vec![10.0, 90.0]
    ); // d3-scale/test/pow-test.js:251:3
    assert_eq!(
        scale::pow()
            .domain([12.0, 87.0])
            .nice(100)
            .get_domain()
            .to_vec(),
        vec![12.0, 87.0]
    ); // d3-scale/test/pow-test.js:252:3
}

// d3-scale/test/pow-test.js: pow.ticks(count) returns the expected ticks for an ascending domain
#[test]
fn pow_ticks_count_returns_the_expected_ticks_for_an_ascending_domain() {
    let mut s = scale::pow();
    assert_eq!(
        s.ticks(10)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    ); // d3-scale/test/pow-test.js:257:3
    assert_eq!(
        s.ticks(9)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    ); // d3-scale/test/pow-test.js:258:3
    assert_eq!(
        s.ticks(8)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    ); // d3-scale/test/pow-test.js:259:3
    assert_eq!(
        s.ticks(7)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
    ); // d3-scale/test/pow-test.js:260:3
    assert_eq!(
        s.ticks(6)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
    ); // d3-scale/test/pow-test.js:261:3
    assert_eq!(
        s.ticks(5)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
    ); // d3-scale/test/pow-test.js:262:3
    assert_eq!(
        s.ticks(4)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
    ); // d3-scale/test/pow-test.js:263:3
    assert_eq!(
        s.ticks(3)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.5, 1.0]
    ); // d3-scale/test/pow-test.js:264:3
    assert_eq!(
        s.ticks(2)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 0.5, 1.0]
    ); // d3-scale/test/pow-test.js:265:3
    assert_eq!(
        s.ticks(1)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        vec![0.0, 1.0]
    ); // d3-scale/test/pow-test.js:266:3
    s = s.domain([-100.0, 100.0]);
    assert_eq!(
        s.ticks(10),
        vec![
            -100.0, -80.0, -60.0, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0, 80.0, 100.0
        ]
    ); // d3-scale/test/pow-test.js:268:3
    assert_eq!(
        s.ticks(9),
        vec![
            -100.0, -80.0, -60.0, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0, 80.0, 100.0
        ]
    ); // d3-scale/test/pow-test.js:269:3
    assert_eq!(
        s.ticks(8),
        vec![
            -100.0, -80.0, -60.0, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0, 80.0, 100.0
        ]
    ); // d3-scale/test/pow-test.js:270:3
    assert_eq!(
        s.ticks(7),
        vec![
            -100.0, -80.0, -60.0, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0, 80.0, 100.0
        ]
    ); // d3-scale/test/pow-test.js:271:3
    assert_eq!(s.ticks(6), vec![-100.0, -50.0, 0.0, 50.0, 100.0]); // d3-scale/test/pow-test.js:272:3
    assert_eq!(s.ticks(5), vec![-100.0, -50.0, 0.0, 50.0, 100.0]); // d3-scale/test/pow-test.js:273:3
    assert_eq!(s.ticks(4), vec![-100.0, -50.0, 0.0, 50.0, 100.0]); // d3-scale/test/pow-test.js:274:3
    assert_eq!(s.ticks(3), vec![-100.0, -50.0, 0.0, 50.0, 100.0]); // d3-scale/test/pow-test.js:275:3
    assert_eq!(s.ticks(2), vec![-100.0, 0.0, 100.0]); // d3-scale/test/pow-test.js:276:3
    assert_eq!(s.ticks(1), vec![0.0]); // d3-scale/test/pow-test.js:277:3
}

// d3-scale/test/pow-test.js: pow.ticks(count) returns the expected ticks for a descending domain
#[test]
fn pow_ticks_count_returns_the_expected_ticks_for_a_descending_domain() {
    let mut s = scale::pow().domain([1.0, 0.0]);
    assert_eq!(
        s.ticks(10)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/pow-test.js:282:3
    assert_eq!(
        s.ticks(9)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/pow-test.js:283:3
    assert_eq!(
        s.ticks(8)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/pow-test.js:284:3
    assert_eq!(
        s.ticks(7)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/pow-test.js:285:3
    assert_eq!(
        s.ticks(6)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/pow-test.js:286:3
    assert_eq!(
        s.ticks(5)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/pow-test.js:287:3
    assert_eq!(
        s.ticks(4)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/pow-test.js:288:3
    assert_eq!(
        s.ticks(3)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.5, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/pow-test.js:289:3
    assert_eq!(
        s.ticks(2)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 0.5, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/pow-test.js:290:3
    assert_eq!(
        s.ticks(1)
            .iter()
            .map(|v| round_epsilon(*v))
            .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.0, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/pow-test.js:291:3
    s = s.domain([100.0, -100.0]);
    assert_eq!(s.ticks(10), {
        let mut v = vec![
            -100.0, -80.0, -60.0, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0, 80.0, 100.0,
        ];
        v.reverse();
        v
    }); // d3-scale/test/pow-test.js:293:3
    assert_eq!(s.ticks(9), {
        let mut v = vec![
            -100.0, -80.0, -60.0, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0, 80.0, 100.0,
        ];
        v.reverse();
        v
    }); // d3-scale/test/pow-test.js:294:3
    assert_eq!(s.ticks(8), {
        let mut v = vec![
            -100.0, -80.0, -60.0, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0, 80.0, 100.0,
        ];
        v.reverse();
        v
    }); // d3-scale/test/pow-test.js:295:3
    assert_eq!(s.ticks(7), {
        let mut v = vec![
            -100.0, -80.0, -60.0, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0, 80.0, 100.0,
        ];
        v.reverse();
        v
    }); // d3-scale/test/pow-test.js:296:3
    assert_eq!(s.ticks(6), {
        let mut v = vec![-100.0, -50.0, 0.0, 50.0, 100.0];
        v.reverse();
        v
    }); // d3-scale/test/pow-test.js:297:3
    assert_eq!(s.ticks(5), {
        let mut v = vec![-100.0, -50.0, 0.0, 50.0, 100.0];
        v.reverse();
        v
    }); // d3-scale/test/pow-test.js:298:3
    assert_eq!(s.ticks(4), {
        let mut v = vec![-100.0, -50.0, 0.0, 50.0, 100.0];
        v.reverse();
        v
    }); // d3-scale/test/pow-test.js:299:3
    assert_eq!(s.ticks(3), {
        let mut v = vec![-100.0, -50.0, 0.0, 50.0, 100.0];
        v.reverse();
        v
    }); // d3-scale/test/pow-test.js:300:3
    assert_eq!(s.ticks(2), {
        let mut v = vec![-100.0, 0.0, 100.0];
        v.reverse();
        v
    }); // d3-scale/test/pow-test.js:301:3
    assert_eq!(s.ticks(1), {
        let mut v = vec![0.0];
        v.reverse();
        v
    }); // d3-scale/test/pow-test.js:302:3
}

// d3-scale/test/pow-test.js: pow.ticks(count) returns the empty array if count is not a positive integer
#[test]
fn pow_ticks_count_returns_the_empty_array_if_count_is_not_a_positive_integer() {
    let s = scale::pow();
    assert_eq!(s.ticks(0), Vec::<f64>::new()); // d3-scale/test/pow-test.js:333:3
}

// d3-scale/test/pow-test.js: pow.ticks() is an alias for pow.ticks(10)
#[test]
fn pow_ticks_is_an_alias_for_pow_ticks_10() {
    let s = scale::pow();
    assert_eq!(s.ticks(10), s.ticks(10)); // d3-scale/test/pow-test.js:340:3
}

// d3-scale/test/pow-test.js: pow.tickFormat() is an alias for pow.tickFormat(10)
#[test]
fn pow_tickformat_is_an_alias_for_pow_tickformat_10() {
    assert_eq!(
        scale::pow()
            .tick_format_in(&format::Locale::EN_US, 10, None)
            .expect("d3 test")
            .format(0.2),
        "0.2"
    ); // d3-scale/test/pow-test.js:344:3
    assert_eq!(
        scale::pow()
            .domain([-100.0, 100.0])
            .tick_format_in(&format::Locale::EN_US, 10, None)
            .expect("d3 test")
            .format(-20.0),
        "−20"
    ); // d3-scale/test/pow-test.js:345:3
}

// d3-scale/test/pow-test.js: pow.tickFormat(count) returns a format suitable for the ticks
#[test]
fn pow_tickformat_count_returns_a_format_suitable_for_the_ticks() {
    assert_eq!(
        scale::pow()
            .tick_format_in(&format::Locale::EN_US, 10, None)
            .expect("d3 test")
            .format(0.2),
        "0.2"
    ); // d3-scale/test/pow-test.js:349:3
    assert_eq!(
        scale::pow()
            .tick_format_in(&format::Locale::EN_US, 20, None)
            .expect("d3 test")
            .format(0.2),
        "0.20"
    ); // d3-scale/test/pow-test.js:350:3
    assert_eq!(
        scale::pow()
            .domain([-100.0, 100.0])
            .tick_format_in(&format::Locale::EN_US, 10, None)
            .expect("d3 test")
            .format(-20.0),
        "−20"
    ); // d3-scale/test/pow-test.js:351:3
}

// d3-scale/test/pow-test.js: pow.tickFormat(count, specifier) sets the appropriate fixed precision if not specified
#[test]
fn pow_tickformat_count_specifier_sets_the_appropriate_fixed_precision_if_not_specified() {
    assert_eq!(
        scale::pow()
            .tick_format_in(&format::Locale::EN_US, 10, Some("+f"))
            .expect("d3 test")
            .format(0.2),
        "+0.2"
    ); // d3-scale/test/pow-test.js:355:3
    assert_eq!(
        scale::pow()
            .tick_format_in(&format::Locale::EN_US, 20, Some("+f"))
            .expect("d3 test")
            .format(0.2),
        "+0.20"
    ); // d3-scale/test/pow-test.js:356:3
    assert_eq!(
        scale::pow()
            .tick_format_in(&format::Locale::EN_US, 10, Some("+%"))
            .expect("d3 test")
            .format(0.2),
        "+20%"
    ); // d3-scale/test/pow-test.js:357:3
    assert_eq!(
        scale::pow()
            .domain([0.19, 0.21])
            .tick_format_in(&format::Locale::EN_US, 10, Some("+%"))
            .expect("d3 test")
            .format(0.2),
        "+20.0%"
    ); // d3-scale/test/pow-test.js:358:3
}

// d3-scale/test/pow-test.js: pow.tickFormat(count, specifier) sets the appropriate round precision if not specified
#[test]
fn pow_tickformat_count_specifier_sets_the_appropriate_round_precision_if_not_specified() {
    assert_eq!(
        scale::pow()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 10, Some(""))
            .expect("d3 test")
            .format(2.10),
        "2"
    ); // d3-scale/test/pow-test.js:362:3
    assert_eq!(
        scale::pow()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 100, Some(""))
            .expect("d3 test")
            .format(2.01),
        "2"
    ); // d3-scale/test/pow-test.js:363:3
    assert_eq!(
        scale::pow()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 100, Some(""))
            .expect("d3 test")
            .format(2.11),
        "2.1"
    ); // d3-scale/test/pow-test.js:364:3
    assert_eq!(
        scale::pow()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 10, Some("e"))
            .expect("d3 test")
            .format(2.10),
        "2e+0"
    ); // d3-scale/test/pow-test.js:365:3
    assert_eq!(
        scale::pow()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 100, Some("e"))
            .expect("d3 test")
            .format(2.01),
        "2.0e+0"
    ); // d3-scale/test/pow-test.js:366:3
    assert_eq!(
        scale::pow()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 100, Some("e"))
            .expect("d3 test")
            .format(2.11),
        "2.1e+0"
    ); // d3-scale/test/pow-test.js:367:3
    assert_eq!(
        scale::pow()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 10, Some("g"))
            .expect("d3 test")
            .format(2.10),
        "2"
    ); // d3-scale/test/pow-test.js:368:3
    assert_eq!(
        scale::pow()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 100, Some("g"))
            .expect("d3 test")
            .format(2.01),
        "2.0"
    ); // d3-scale/test/pow-test.js:369:3
    assert_eq!(
        scale::pow()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 100, Some("g"))
            .expect("d3 test")
            .format(2.11),
        "2.1"
    ); // d3-scale/test/pow-test.js:370:3
    assert_eq!(
        scale::pow()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 10, Some("r"))
            .expect("d3 test")
            .format(2.10e6),
        "2000000"
    ); // d3-scale/test/pow-test.js:371:3
    assert_eq!(
        scale::pow()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 100, Some("r"))
            .expect("d3 test")
            .format(2.01e6),
        "2000000"
    ); // d3-scale/test/pow-test.js:372:3
    assert_eq!(
        scale::pow()
            .domain([0.0, 9.0])
            .tick_format_in(&format::Locale::EN_US, 100, Some("r"))
            .expect("d3 test")
            .format(2.11e6),
        "2100000"
    ); // d3-scale/test/pow-test.js:373:3
    assert_eq!(
        scale::pow()
            .domain([0.0, 0.9])
            .tick_format_in(&format::Locale::EN_US, 10, Some("p"))
            .expect("d3 test")
            .format(0.210),
        "20%"
    ); // d3-scale/test/pow-test.js:374:3
    assert_eq!(
        scale::pow()
            .domain([0.19, 0.21])
            .tick_format_in(&format::Locale::EN_US, 10, Some("p"))
            .expect("d3 test")
            .format(0.201),
        "20.1%"
    ); // d3-scale/test/pow-test.js:375:3
}

// d3-scale/test/pow-test.js: pow.tickFormat(count, specifier) sets the appropriate prefix precision if not specified
#[test]
fn pow_tickformat_count_specifier_sets_the_appropriate_prefix_precision_if_not_specified() {
    assert_eq!(
        scale::pow()
            .domain([0.0, 1e6])
            .tick_format_in(&format::Locale::EN_US, 10, Some("$s"))
            .expect("d3 test")
            .format(0.51e6),
        "$0.5M"
    ); // d3-scale/test/pow-test.js:379:3
    assert_eq!(
        scale::pow()
            .domain([0.0, 1e6])
            .tick_format_in(&format::Locale::EN_US, 100, Some("$s"))
            .expect("d3 test")
            .format(0.501e6),
        "$0.50M"
    ); // d3-scale/test/pow-test.js:380:3
}

// d3-scale/test/pow-test.js: pow.copy() returns a copy with changes to the domain are isolated
#[test]
fn pow_copy_returns_a_copy_with_changes_to_the_domain_are_isolated() {
    let mut x = scale::pow();
    let mut y = x.clone();
    x = x.domain([1.0, 2.0]);
    assert_eq!(y.get_domain().to_vec(), vec![0.0, 1.0]); // d3-scale/test/pow-test.js:386:3
    assert_eq!(x.apply(1.0), 0.0); // d3-scale/test/pow-test.js:387:3
    assert_eq!(y.apply(1.0), 1.0); // d3-scale/test/pow-test.js:388:3
    y = y.domain([2.0, 3.0]);
    assert_eq!(x.apply(2.0), 1.0); // d3-scale/test/pow-test.js:390:3
    assert_eq!(y.apply(2.0), 0.0); // d3-scale/test/pow-test.js:391:3
    assert_eq!(x.get_domain().to_vec(), vec![1.0, 2.0]); // d3-scale/test/pow-test.js:392:3
    assert_eq!(y.get_domain().to_vec(), vec![2.0, 3.0]); // d3-scale/test/pow-test.js:393:3
    let y2 = {
        x = x.domain([1.0, 1.9]);
        x.clone()
    };
    x = x.nice(5);
    assert_eq!(x.get_domain().to_vec(), vec![1.0, 2.0]); // d3-scale/test/pow-test.js:396:3
    assert_eq!(y2.get_domain().to_vec(), vec![1.0, 1.9]); // d3-scale/test/pow-test.js:397:3
}

// d3-scale/test/pow-test.js: pow.copy() returns a copy with changes to the range are isolated
#[test]
fn pow_copy_returns_a_copy_with_changes_to_the_range_are_isolated() {
    let mut x = scale::pow();
    let mut y = x.clone();
    x = x.range([1.0, 2.0]);
    assert_eq!(x.invert(1.0), 0.0); // d3-scale/test/pow-test.js:403:3
    assert_eq!(y.invert(1.0), 1.0); // d3-scale/test/pow-test.js:404:3
    assert_eq!(y.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/pow-test.js:405:3
    y = y.range([2.0, 3.0]);
    assert_eq!(x.invert(2.0), 1.0); // d3-scale/test/pow-test.js:407:3
    assert_eq!(y.invert(2.0), 0.0); // d3-scale/test/pow-test.js:408:3
    assert_eq!(x.get_range().to_vec(), vec![1.0, 2.0]); // d3-scale/test/pow-test.js:409:3
    assert_eq!(y.get_range().to_vec(), vec![2.0, 3.0]); // d3-scale/test/pow-test.js:410:3
}

// d3-scale/test/pow-test.js: pow.copy() returns a copy with changes to clamping are isolated
#[test]
fn pow_copy_returns_a_copy_with_changes_to_clamping_are_isolated() {
    let mut x = scale::pow().clamp(true);
    let mut y = x.clone();
    x = x.clamp(false);
    assert_eq!(x.apply(2.0), 2.0); // d3-scale/test/pow-test.js:427:3
    assert_eq!(y.apply(2.0), 1.0); // d3-scale/test/pow-test.js:428:3
    y = y.clamp(false);
    assert_eq!(x.apply(2.0), 2.0); // d3-scale/test/pow-test.js:431:3
    assert_eq!(y.apply(2.0), 2.0); // d3-scale/test/pow-test.js:432:3
}

// d3-scale/test/pow-test.js: pow().clamp(true).invert(x) cannot return a value outside the domain
#[test]
fn pow_clamp_true_invert_x_cannot_return_a_value_outside_the_domain() {
    let x = scale::pow().exponent(0.5).domain([1.0, 20.0]).clamp(true);
    assert_eq!(x.invert(0.0), 1.0); // d3-scale/test/pow-test.js:438:3
    assert_eq!(x.invert(1.0), 20.0); // d3-scale/test/pow-test.js:439:3
}

// d3-scale/test/pow-test.js: scaleSqrt() is an alias for pow().exponent(0.5)
#[test]
fn scalesqrt_is_an_alias_for_pow_exponent_0_5() {
    let s = scale::sqrt();
    assert_in_delta(&s.apply(0.5), &std::f64::consts::FRAC_1_SQRT_2, 1e-6); // d3-scale/test/pow-test.js:445:3
    assert_in_delta(&s.invert(std::f64::consts::FRAC_1_SQRT_2), &0.5, 1e-6); // d3-scale/test/pow-test.js:446:3
}

// d3-scale/test/sqrt-test.js: scaleSqrt() has the expected defaults
#[test]
fn scalesqrt_has_the_expected_defaults() {
    let s = scale::sqrt();
    assert_eq!(s.get_domain().to_vec(), vec![0.0, 1.0]); // d3-scale/test/sqrt-test.js:6:3
    assert_eq!(s.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/sqrt-test.js:7:3
}

// d3-scale/test/sqrt-test.js: sqrt(x) maps a domain value x to a range value y
#[test]
fn sqrt_x_maps_a_domain_value_x_to_a_range_value_y() {
    assert_eq!(scale::sqrt().apply(0.5), std::f64::consts::FRAC_1_SQRT_2); // d3-scale/test/sqrt-test.js:14:3
}

// d3-scale/test/log-test.js: scaleLog() has the expected defaults
#[test]
fn scalelog_has_the_expected_defaults() {
    let x = scale::log();
    assert_eq!(x.get_domain().to_vec(), vec![1.0, 10.0]); // d3-scale/test/log-test.js:10:3
    assert_eq!(x.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/log-test.js:11:3
    assert_in_delta(&x.apply(5.0), &0.69897, 1e-6); // d3-scale/test/log-test.js:16:3
    assert_in_delta(&x.invert(0.69897), &5.0, 1e-6); // d3-scale/test/log-test.js:17:3
    assert_in_delta(&x.apply(3.162278), &0.5, 1e-6); // d3-scale/test/log-test.js:18:3
    assert_in_delta(&x.invert(0.5), &3.162278, 1e-6); // d3-scale/test/log-test.js:19:3
}

// d3-scale/test/log-test.js: log.domain(…) can take negative values
#[test]
fn log_domain_can_take_negative_values() {
    let x = scale::log().domain([-100.0, -1.0]);
    assert_in_delta(&x.apply(-50.0), &0.150515, 1e-6); // d3-scale/test/log-test.js:49:3
}

// d3-scale/test/log-test.js: log.domain(…) preserves specified domain exactly, with no floating point error
#[test]
fn log_domain_preserves_specified_domain_exactly_with_no_floating_point_error() {
    let x = scale::log().domain([0.1, 1000.0]);
    assert_eq!(x.get_domain().to_vec(), vec![0.1, 1000.0]); // d3-scale/test/log-test.js:61:3
}

// d3-scale/test/log-test.js: log.ticks(…) returns exact ticks, with no floating point error
#[test]
fn log_ticks_returns_exact_ticks_with_no_floating_point_error() {
    assert_eq!(
        scale::log().domain([0.15, 0.68]).ticks(10),
        vec![0.2, 0.3, 0.4, 0.5, 0.6]
    ); // d3-scale/test/log-test.js:65:3
    assert_eq!(
        scale::log().domain([0.68, 0.15]).ticks(10),
        vec![0.6, 0.5, 0.4, 0.3, 0.2]
    ); // d3-scale/test/log-test.js:66:3
    assert_eq!(
        scale::log().domain([-0.15, -0.68]).ticks(10),
        vec![-0.2, -0.3, -0.4, -0.5, -0.6]
    ); // d3-scale/test/log-test.js:67:3
    assert_eq!(
        scale::log().domain([-0.68, -0.15]).ticks(10),
        vec![-0.6, -0.5, -0.4, -0.3, -0.2]
    ); // d3-scale/test/log-test.js:68:3
}

// d3-scale/test/log-test.js: log.range(…) can take colors
#[test]
fn log_range_can_take_colors() {
    let mut x = scale::log().range([
        "red".parse::<Rgb>().expect("a CSS color"),
        "blue".parse::<Rgb>().expect("a CSS color"),
    ]);
    assert_eq!(x.apply(5.0).to_string(), "rgb(77, 0, 178)"); // d3-scale/test/log-test.js:79:3
    x = x.range([
        "#ff0000".parse::<Rgb>().expect("a CSS color"),
        "#0000ff".parse::<Rgb>().expect("a CSS color"),
    ]);
    assert_eq!(x.apply(5.0).to_string(), "rgb(77, 0, 178)"); // d3-scale/test/log-test.js:81:3
    x = x.range([
        "#f00".parse::<Rgb>().expect("a CSS color"),
        "#00f".parse::<Rgb>().expect("a CSS color"),
    ]);
    assert_eq!(x.apply(5.0).to_string(), "rgb(77, 0, 178)"); // d3-scale/test/log-test.js:83:3
    x = x.range([
        Rgb::new(255.0, 0.0, 0.0),
        Rgb::from(Hsl {
            h: 240.0,
            s: 1.0,
            l: 0.5,
            opacity: 1.0,
        }),
    ]);
    assert_eq!(x.apply(5.0).to_string(), "rgb(77, 0, 178)"); // d3-scale/test/log-test.js:85:3
    x = x.range([
        "hsl(0,100%,50%)".parse::<Rgb>().expect("a CSS color"),
        "hsl(240,100%,50%)".parse::<Rgb>().expect("a CSS color"),
    ]);
    assert_eq!(x.apply(5.0).to_string(), "rgb(77, 0, 178)"); // d3-scale/test/log-test.js:87:3
}

// d3-scale/test/log-test.js: log.interpolate(f) sets the interpolator
#[test]
fn log_interpolate_f_sets_the_interpolator() {
    let mut x = scale::log().range([
        "red".parse::<Rgb>().expect("a CSS color"),
        "blue".parse::<Rgb>().expect("a CSS color"),
    ]);
    assert_eq!(x.apply(5.0).to_string(), "rgb(77, 0, 178)"); // d3-scale/test/log-test.js:100:3
    x = x.interpolate(|a: &Rgb, b: &Rgb, t: f64| interpolate::hsl(Hsl::from(*a), Hsl::from(*b))(t));
    assert_eq!(x.apply(5.0).to_string(), "rgb(154, 0, 255)"); // d3-scale/test/log-test.js:102:3
}

// d3-scale/test/log-test.js: log(x) does not clamp by default
#[test]
fn log_x_does_not_clamp_by_default() {
    let x = scale::log();
    assert_in_delta(&x.apply(0.5), &-0.3010299, 1e-6); // d3-scale/test/log-test.js:108:3
    assert_in_delta(&x.apply(15.0), &1.1760913, 1e-6); // d3-scale/test/log-test.js:109:3
}

// d3-scale/test/log-test.js: log.clamp(true)(x) clamps to the domain
#[test]
fn log_clamp_true_x_clamps_to_the_domain() {
    let mut x = scale::log().clamp(true);
    assert_in_delta(&x.apply(-1.0), &0.0, 1e-6); // d3-scale/test/log-test.js:114:3
    assert_in_delta(&x.apply(5.0), &0.69897, 1e-6); // d3-scale/test/log-test.js:115:3
    assert_in_delta(&x.apply(15.0), &1.0, 1e-6); // d3-scale/test/log-test.js:116:3
    x = x.domain([10.0, 1.0]);
    assert_in_delta(&x.apply(-1.0), &1.0, 1e-6); // d3-scale/test/log-test.js:118:3
    assert_in_delta(&x.apply(5.0), &0.30103, 1e-6); // d3-scale/test/log-test.js:119:3
    assert_in_delta(&x.apply(15.0), &0.0, 1e-6); // d3-scale/test/log-test.js:120:3
}

// d3-scale/test/log-test.js: log.clamp(true).invert(y) clamps to the range
#[test]
fn log_clamp_true_invert_y_clamps_to_the_range() {
    let mut x = scale::log().clamp(true);
    assert_in_delta(&x.invert(-0.1), &1.0, 1e-6); // d3-scale/test/log-test.js:125:3
    assert_in_delta(&x.invert(0.69897), &5.0, 1e-6); // d3-scale/test/log-test.js:126:3
    assert_in_delta(&x.invert(1.5), &10.0, 1e-6); // d3-scale/test/log-test.js:127:3
    x = x.domain([10.0, 1.0]);
    assert_in_delta(&x.invert(-0.1), &10.0, 1e-6); // d3-scale/test/log-test.js:129:3
    assert_in_delta(&x.invert(0.30103), &5.0, 1e-6); // d3-scale/test/log-test.js:130:3
    assert_in_delta(&x.invert(1.5), &1.0, 1e-6); // d3-scale/test/log-test.js:131:3
}

// d3-scale/test/log-test.js: log(x) maps a number x to a number y
#[test]
fn log_x_maps_a_number_x_to_a_number_y() {
    let x = scale::log().domain([1.0, 2.0]);
    assert_in_delta(&x.apply(0.5), &-1.0000000, 1e-6); // d3-scale/test/log-test.js:136:3
    assert_in_delta(&x.apply(1.0), &0.0000000, 1e-6); // d3-scale/test/log-test.js:137:3
    assert_in_delta(&x.apply(1.5), &0.5849625, 1e-6); // d3-scale/test/log-test.js:138:3
    assert_in_delta(&x.apply(2.0), &1.0000000, 1e-6); // d3-scale/test/log-test.js:139:3
    assert_in_delta(&x.apply(2.5), &1.3219281, 1e-6); // d3-scale/test/log-test.js:140:3
}

// d3-scale/test/log-test.js: log.invert(y) maps a number y to a number x
#[test]
fn log_invert_y_maps_a_number_y_to_a_number_x() {
    let x = scale::log().domain([1.0, 2.0]);
    assert_in_delta(&x.invert(-1.0000000), &0.5, 1e-6); // d3-scale/test/log-test.js:145:3
    assert_in_delta(&x.invert(0.0000000), &1.0, 1e-6); // d3-scale/test/log-test.js:146:3
    assert_in_delta(&x.invert(0.5849625), &1.5, 1e-6); // d3-scale/test/log-test.js:147:3
    assert_in_delta(&x.invert(1.0000000), &2.0, 1e-6); // d3-scale/test/log-test.js:148:3
    assert_in_delta(&x.invert(1.3219281), &2.5, 1e-6); // d3-scale/test/log-test.js:149:3
}

// d3-scale/test/log-test.js: log.base(b) sets the log base, changing the ticks
#[test]
fn log_base_b_sets_the_log_base_changing_the_ticks() {
    let mut x = scale::log().domain([1.0, 32.0]);
    assert_eq!(
        {
            let values_ = &({
                x = x.base(2.0);
                x.ticks(10)
            });
            let format_ = x
                .tick_format_in(&format::Locale::EN_US, 10, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec!["1", "2", "4", "8", "16", "32"]
    ); // d3-scale/test/log-test.js:163:3
    assert_eq!(
        {
            let values_ = &({
                x = x.base(std::f64::consts::E);
                x.ticks(10)
            });
            let format_ = x
                .tick_format_in(&format::Locale::EN_US, 10, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec!["1", "2.71828182846", "7.38905609893", "20.0855369232"]
    ); // d3-scale/test/log-test.js:164:3
}

// d3-scale/test/log-test.js: log.nice() nices the domain, extending it to powers of ten
#[test]
fn log_nice_nices_the_domain_extending_it_to_powers_of_ten() {
    let mut x = scale::log().domain([1.1, 10.9]).nice(10);
    assert_eq!(x.get_domain().to_vec(), vec![1.0, 100.0]); // d3-scale/test/log-test.js:169:3
    x = x.domain([10.9, 1.1]).nice(10);
    assert_eq!(x.get_domain().to_vec(), vec![100.0, 1.0]); // d3-scale/test/log-test.js:171:3
    x = x.domain([0.7, 11.001]).nice(10);
    assert_eq!(x.get_domain().to_vec(), vec![0.1, 100.0]); // d3-scale/test/log-test.js:173:3
    x = x.domain([123.1, 6.7]).nice(10);
    assert_eq!(x.get_domain().to_vec(), vec![1000.0, 1.0]); // d3-scale/test/log-test.js:175:3
    x = x.domain([0.01, 0.49]).nice(10);
    assert_eq!(x.get_domain().to_vec(), vec![0.01, 1.0]); // d3-scale/test/log-test.js:177:3
    x = x.domain([1.5, 50.0]).nice(10);
    assert_eq!(x.get_domain().to_vec(), vec![1.0, 100.0]); // d3-scale/test/log-test.js:179:3
    assert_in_delta(&x.apply(1.0), &0.0, 1e-6); // d3-scale/test/log-test.js:180:3
    assert_in_delta(&x.apply(100.0), &1.0, 1e-6); // d3-scale/test/log-test.js:181:3
}

// d3-scale/test/log-test.js: log.nice() works on degenerate domains
#[test]
fn log_nice_works_on_degenerate_domains() {
    let mut x = scale::log().domain([0.0, 0.0]).nice(10);
    assert_eq!(x.get_domain().to_vec(), vec![0.0, 0.0]); // d3-scale/test/log-test.js:186:3
    x = x.domain([0.5, 0.5]).nice(10);
    assert_eq!(x.get_domain().to_vec(), vec![0.1, 1.0]); // d3-scale/test/log-test.js:188:3
}

// d3-scale/test/log-test.js: log.copy() isolates changes to the domain
#[test]
fn log_copy_isolates_changes_to_the_domain() {
    let mut x = scale::log();
    let mut y = x.clone();
    x = x.domain([10.0, 100.0]);
    assert_eq!(y.get_domain().to_vec(), vec![1.0, 10.0]); // d3-scale/test/log-test.js:201:3
    assert_in_delta(&x.apply(10.0), &0.0, 1e-6); // d3-scale/test/log-test.js:202:3
    assert_in_delta(&y.apply(1.0), &0.0, 1e-6); // d3-scale/test/log-test.js:203:3
    y = y.domain([100.0, 1000.0]);
    assert_in_delta(&x.apply(100.0), &1.0, 1e-6); // d3-scale/test/log-test.js:205:3
    assert_in_delta(&y.apply(100.0), &0.0, 1e-6); // d3-scale/test/log-test.js:206:3
    assert_eq!(x.get_domain().to_vec(), vec![10.0, 100.0]); // d3-scale/test/log-test.js:207:3
    assert_eq!(y.get_domain().to_vec(), vec![100.0, 1000.0]); // d3-scale/test/log-test.js:208:3
}

// d3-scale/test/log-test.js: log.copy() isolates changes to the domain via nice
#[test]
fn log_copy_isolates_changes_to_the_domain_via_nice() {
    let x = scale::log().domain([1.5, 50.0]);
    let y = x.clone().nice(10);
    assert_eq!(x.get_domain().to_vec(), vec![1.5, 50.0]); // d3-scale/test/log-test.js:213:3
    assert_in_delta(&x.apply(1.5), &0.0, 1e-6); // d3-scale/test/log-test.js:214:3
    assert_in_delta(&x.apply(50.0), &1.0, 1e-6); // d3-scale/test/log-test.js:215:3
    assert_in_delta(&x.invert(0.0), &1.5, 1e-6); // d3-scale/test/log-test.js:216:3
    assert_in_delta(&x.invert(1.0), &50.0, 1e-6); // d3-scale/test/log-test.js:217:3
    assert_eq!(y.get_domain().to_vec(), vec![1.0, 100.0]); // d3-scale/test/log-test.js:218:3
    assert_in_delta(&y.apply(1.0), &0.0, 1e-6); // d3-scale/test/log-test.js:219:3
    assert_in_delta(&y.apply(100.0), &1.0, 1e-6); // d3-scale/test/log-test.js:220:3
    assert_in_delta(&y.invert(0.0), &1.0, 1e-6); // d3-scale/test/log-test.js:221:3
    assert_in_delta(&y.invert(1.0), &100.0, 1e-6); // d3-scale/test/log-test.js:222:3
}

// d3-scale/test/log-test.js: log.copy() isolates changes to the range
#[test]
fn log_copy_isolates_changes_to_the_range() {
    let mut x = scale::log();
    let mut y = x.clone();
    x = x.range([1.0, 2.0]);
    assert_in_delta(&x.invert(1.0), &1.0, 1e-6); // d3-scale/test/log-test.js:228:3
    assert_in_delta(&y.invert(1.0), &10.0, 1e-6); // d3-scale/test/log-test.js:229:3
    assert_eq!(y.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/log-test.js:230:3
    y = y.range([2.0, 3.0]);
    assert_in_delta(&x.invert(2.0), &10.0, 1e-6); // d3-scale/test/log-test.js:232:3
    assert_in_delta(&y.invert(2.0), &1.0, 1e-6); // d3-scale/test/log-test.js:233:3
    assert_eq!(x.get_range().to_vec(), vec![1.0, 2.0]); // d3-scale/test/log-test.js:234:3
    assert_eq!(y.get_range().to_vec(), vec![2.0, 3.0]); // d3-scale/test/log-test.js:235:3
}

// d3-scale/test/log-test.js: log.copy() isolates changes to the interpolator
#[test]
fn log_copy_isolates_changes_to_the_interpolator() {
    let mut x = scale::log().range([
        "red".parse::<Rgb>().expect("a CSS color"),
        "blue".parse::<Rgb>().expect("a CSS color"),
    ]);
    let y = x.clone();
    x = x.interpolate(|a: &Rgb, b: &Rgb, t: f64| interpolate::hsl(Hsl::from(*a), Hsl::from(*b))(t));
    assert_eq!(x.apply(5.0).to_string(), "rgb(154, 0, 255)"); // d3-scale/test/log-test.js:241:3
    assert_eq!(y.apply(5.0).to_string(), "rgb(77, 0, 178)"); // d3-scale/test/log-test.js:242:3
}

// d3-scale/test/log-test.js: log.copy() isolates changes to clamping
#[test]
fn log_copy_isolates_changes_to_clamping() {
    let mut x = scale::log().clamp(true);
    let mut y = x.clone();
    x = x.clamp(false);
    assert_in_delta(&x.apply(0.5), &-0.30103, 1e-6); // d3-scale/test/log-test.js:249:3
    assert_in_delta(&y.apply(0.5), &0.0, 1e-6); // d3-scale/test/log-test.js:250:3
    y = y.clamp(false);
    assert_in_delta(&x.apply(20.0), &1.30103, 1e-6); // d3-scale/test/log-test.js:253:3
    assert_in_delta(&y.apply(20.0), &1.30103, 1e-6); // d3-scale/test/log-test.js:254:3
}

// d3-scale/test/log-test.js: log.ticks() generates the expected power-of-ten for ascending ticks
#[test]
fn log_ticks_generates_the_expected_power_of_ten_for_ascending_ticks() {
    let mut s = scale::log();
    assert_eq!(
        ({
            s = s.domain([1e-1, 1e1]);
            s.ticks(10)
        })
        .iter()
        .map(|v| round(*v))
        .collect::<Vec<f64>>(),
        vec![
            0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0,
            9.0, 10.0
        ]
    ); // d3-scale/test/log-test.js:260:3
    assert_eq!(
        ({
            s = s.domain([1e-1, 1e0]);
            s.ticks(10)
        })
        .iter()
        .map(|v| round(*v))
        .collect::<Vec<f64>>(),
        vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    ); // d3-scale/test/log-test.js:261:3
    assert_eq!(
        ({
            s = s.domain([-1e0, -1e-1]);
            s.ticks(10)
        })
        .iter()
        .map(|v| round(*v))
        .collect::<Vec<f64>>(),
        vec![-1.0, -0.9, -0.8, -0.7, -0.6, -0.5, -0.4, -0.3, -0.2, -0.1]
    ); // d3-scale/test/log-test.js:262:3
}

// d3-scale/test/log-test.js: log.ticks() generates the expected power-of-ten ticks for descending domains
#[test]
fn log_ticks_generates_the_expected_power_of_ten_ticks_for_descending_domains() {
    let mut s = scale::log();
    assert_eq!(
        ({
            s = s.domain([-1e-1, -1e1]);
            s.ticks(10)
        })
        .iter()
        .map(|v| round(*v))
        .collect::<Vec<f64>>(),
        {
            let mut v = vec![
                -10.0, -9.0, -8.0, -7.0, -6.0, -5.0, -4.0, -3.0, -2.0, -1.0, -0.9, -0.8, -0.7,
                -0.6, -0.5, -0.4, -0.3, -0.2, -0.1,
            ];
            v.reverse();
            v
        }
    ); // d3-scale/test/log-test.js:268:3
    assert_eq!(
        ({
            s = s.domain([-1e-1, -1e0]);
            s.ticks(10)
        })
        .iter()
        .map(|v| round(*v))
        .collect::<Vec<f64>>(),
        {
            let mut v = vec![-1.0, -0.9, -0.8, -0.7, -0.6, -0.5, -0.4, -0.3, -0.2, -0.1];
            v.reverse();
            v
        }
    ); // d3-scale/test/log-test.js:269:3
    assert_eq!(
        ({
            s = s.domain([1e0, 1e-1]);
            s.ticks(10)
        })
        .iter()
        .map(|v| round(*v))
        .collect::<Vec<f64>>(),
        {
            let mut v = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
            v.reverse();
            v
        }
    ); // d3-scale/test/log-test.js:270:3
}

// d3-scale/test/log-test.js: log.ticks() generates the expected power-of-ten ticks for small domains
#[test]
fn log_ticks_generates_the_expected_power_of_ten_ticks_for_small_domains() {
    let mut s = scale::log();
    assert_eq!(
        {
            s = s.domain([1.0, 5.0]);
            s.ticks(10)
        },
        vec![1.0, 2.0, 3.0, 4.0, 5.0]
    ); // d3-scale/test/log-test.js:275:3
    assert_eq!(
        {
            s = s.domain([5.0, 1.0]);
            s.ticks(10)
        },
        vec![5.0, 4.0, 3.0, 2.0, 1.0]
    ); // d3-scale/test/log-test.js:276:3
    assert_eq!(
        {
            s = s.domain([-1.0, -5.0]);
            s.ticks(10)
        },
        vec![-1.0, -2.0, -3.0, -4.0, -5.0]
    ); // d3-scale/test/log-test.js:277:3
    assert_eq!(
        {
            s = s.domain([-5.0, -1.0]);
            s.ticks(10)
        },
        vec![-5.0, -4.0, -3.0, -2.0, -1.0]
    ); // d3-scale/test/log-test.js:278:3
    assert_eq!(
        {
            s = s.domain([286.9252014, 329.4978332]);
            s.ticks(1)
        },
        vec![300.0]
    ); // d3-scale/test/log-test.js:279:3
    assert_eq!(
        {
            s = s.domain([286.9252014, 329.4978332]);
            s.ticks(2)
        },
        vec![300.0]
    ); // d3-scale/test/log-test.js:280:3
    assert_eq!(
        {
            s = s.domain([286.9252014, 329.4978332]);
            s.ticks(3)
        },
        vec![300.0, 320.0]
    ); // d3-scale/test/log-test.js:281:3
    assert_eq!(
        {
            s = s.domain([286.9252014, 329.4978332]);
            s.ticks(4)
        },
        vec![290.0, 300.0, 310.0, 320.0]
    ); // d3-scale/test/log-test.js:282:3
    assert_eq!(
        {
            s = s.domain([286.9252014, 329.4978332]);
            s.ticks(10)
        },
        vec![290.0, 295.0, 300.0, 305.0, 310.0, 315.0, 320.0, 325.0]
    ); // d3-scale/test/log-test.js:283:3
}

// d3-scale/test/log-test.js: log.ticks() generates linear ticks when the domain extent is small
#[test]
fn log_ticks_generates_linear_ticks_when_the_domain_extent_is_small() {
    let mut s = scale::log();
    assert_eq!(
        {
            s = s.domain([41.0, 42.0]);
            s.ticks(10)
        },
        vec![
            41.0, 41.1, 41.2, 41.3, 41.4, 41.5, 41.6, 41.7, 41.8, 41.9, 42.0
        ]
    ); // d3-scale/test/log-test.js:288:3
    assert_eq!(
        {
            s = s.domain([42.0, 41.0]);
            s.ticks(10)
        },
        vec![
            42.0, 41.9, 41.8, 41.7, 41.6, 41.5, 41.4, 41.3, 41.2, 41.1, 41.0
        ]
    ); // d3-scale/test/log-test.js:289:3
    assert_eq!(
        {
            s = s.domain([1600.0, 1400.0]);
            s.ticks(10)
        },
        vec![
            1600.0, 1580.0, 1560.0, 1540.0, 1520.0, 1500.0, 1480.0, 1460.0, 1440.0, 1420.0, 1400.0
        ]
    ); // d3-scale/test/log-test.js:290:3
}

// d3-scale/test/log-test.js: log.base(base).ticks() generates the expected power-of-base ticks
#[test]
fn log_base_base_ticks_generates_the_expected_power_of_base_ticks() {
    let mut s = scale::log().base(std::f64::consts::E);
    assert_eq!(
        ({
            s = s.domain([0.1, 100.0]);
            s.ticks(10)
        })
        .iter()
        .map(|v| round(*v))
        .collect::<Vec<f64>>(),
        vec![
            0.135335283237,
            0.367879441171,
            1.0,
            2.718281828459,
            7.389056098931,
            20.085536923188,
            54.598150033144
        ]
    ); // d3-scale/test/log-test.js:295:3
}

// d3-scale/test/log-test.js: log.tickFormat() is equivalent to log.tickFormat(10)
#[test]
fn log_tickformat_is_equivalent_to_log_tickformat_10() {
    let mut s = scale::log();
    assert_eq!(
        {
            let values_ = &({
                s = s.domain([1e-1, 1e1]);
                s.ticks(10)
            });
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 10, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "100m", "200m", "300m", "400m", "500m", "", "", "", "", "1", "2", "3", "4", "5", "",
            "", "", "", "10"
        ]
    ); // d3-scale/test/log-test.js:300:3
}

// d3-scale/test/log-test.js: log.tickFormat(count) returns a filtered "s" format
#[test]
fn log_tickformat_count_returns_a_filtered_s_format() {
    let mut s = scale::log();
    let t = {
        s = s.domain([1e-1, 1e1]);
        s.ticks(10)
    };
    assert_eq!(
        {
            let values_ = &t;
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 10, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "100m", "200m", "300m", "400m", "500m", "", "", "", "", "1", "2", "3", "4", "5", "",
            "", "", "", "10"
        ]
    ); // d3-scale/test/log-test.js:305:3
    assert_eq!(
        {
            let values_ = &t;
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 5, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "100m", "200m", "", "", "", "", "", "", "", "1", "2", "", "", "", "", "", "", "", "10"
        ]
    ); // d3-scale/test/log-test.js:306:2
    assert_eq!(
        {
            let values_ = &t;
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 1, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "100m", "", "", "", "", "", "", "", "", "1", "", "", "", "", "", "", "", "", "10"
        ]
    ); // d3-scale/test/log-test.js:307:2
    assert_eq!(
        {
            let values_ = &t;
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 0, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "100m", "", "", "", "", "", "", "", "", "1", "", "", "", "", "", "", "", "", "10"
        ]
    ); // d3-scale/test/log-test.js:308:2
}

// d3-scale/test/log-test.js: log.tickFormat(count, format) returns the specified format, filtered
#[test]
fn log_tickformat_count_format_returns_the_specified_format_filtered() {
    let mut s = scale::log();
    let t = {
        s = s.domain([1e-1, 1e1]);
        s.ticks(10)
    };
    assert_eq!(
        {
            let values_ = &t;
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 10, Some("+"))
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "+0.1", "+0.2", "+0.3", "+0.4", "+0.5", "", "", "", "", "+1", "+2", "+3", "+4", "+5",
            "", "", "", "", "+10"
        ]
    ); // d3-scale/test/log-test.js:313:3
}

// d3-scale/test/log-test.js: log.base(base).tickFormat() returns the "," format
#[test]
fn log_base_base_tickformat_returns_the_format() {
    let mut s = scale::log().base(std::f64::consts::E);
    assert_eq!(
        {
            let values_ = &({
                s = s.domain([1e-1, 1e1]);
                s.ticks(10)
            });
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 10, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "0.135335283237",
            "0.367879441171",
            "1",
            "2.71828182846",
            "7.38905609893"
        ]
    ); // d3-scale/test/log-test.js:318:3
}

// d3-scale/test/log-test.js: log.base(base).tickFormat(count) returns a filtered "," format
#[test]
fn log_base_base_tickformat_count_returns_a_filtered_format() {
    let mut s = scale::log().base(16.0);
    let t = {
        s = s.domain([1e-1, 1e1]);
        s.ticks(10)
    };
    assert_eq!(
        {
            let values_ = &t;
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 10, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "0.125", "0.1875", "0.25", "0.3125", "0.375", "", "", "", "", "", "", "", "", "", "1",
            "2", "3", "4", "5", "6", "", "", "", ""
        ]
    ); // d3-scale/test/log-test.js:323:3
    assert_eq!(
        {
            let values_ = &t;
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 5, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "0.125", "0.1875", "", "", "", "", "", "", "", "", "", "", "", "", "1", "2", "3", "",
            "", "", "", "", "", ""
        ]
    ); // d3-scale/test/log-test.js:324:3
    assert_eq!(
        {
            let values_ = &t;
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 1, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "", "", "", "", "", "", "", "", "", "", "", "", "", "", "1", "", "", "", "", "", "",
            "", "", ""
        ]
    ); // d3-scale/test/log-test.js:325:3
}

// d3-scale/test/log-test.js: log.tickFormat(count) filters ticks to about count
#[test]
fn log_tickformat_count_filters_ticks_to_about_count() {
    let mut x = scale::log();
    assert_eq!(
        {
            let values_ = &x.ticks(10);
            let format_ = x
                .tick_format_in(&format::Locale::EN_US, 5, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec!["1", "2", "3", "4", "5", "", "", "", "", "10"]
    ); // d3-scale/test/log-test.js:354:3
    x = x.domain([100.0, 1.0]);
    assert_eq!(
        {
            let values_ = &x.ticks(10);
            let format_ = x
                .tick_format_in(&format::Locale::EN_US, 10, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "100", "", "", "", "", "50", "40", "30", "20", "10", "", "", "", "", "5", "4", "3",
            "2", "1"
        ]
    ); // d3-scale/test/log-test.js:359:3
}

// d3-scale/test/log-test.js: log.ticks(count) filters powers-of-ten ticks for huge domains
#[test]
fn log_ticks_count_filters_powers_of_ten_ticks_for_huge_domains() {
    let mut x = scale::log().domain([1e10, 1.0]);
    assert_eq!(
        {
            let values_ = &x.ticks(10);
            let format_ = x
                .tick_format_in(&format::Locale::EN_US, 10, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "10G", "1G", "100M", "10M", "1M", "100k", "10k", "1k", "100", "10", "1"
        ]
    ); // d3-scale/test/log-test.js:368:3
    x = x.domain([1e-29, 1e-1]);
    assert_eq!(
        {
            let values_ = &x.ticks(10);
            let format_ = x
                .tick_format_in(&format::Locale::EN_US, 10, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "0.0001y", "0.01y", "1y", "100y", "10z", "1a", "100a", "10f", "1p", "100p", "10n",
            "1µ", "100µ", "10m"
        ]
    ); // d3-scale/test/log-test.js:370:3
}

// d3-scale/test/log-test.js: log.ticks() generates ticks that cover the domain
#[test]
fn log_ticks_generates_ticks_that_cover_the_domain() {
    let x = scale::log().domain([0.01, 10000.0]);
    assert_eq!(
        {
            let values_ = &x.ticks(20);
            let format_ = x
                .tick_format_in(&format::Locale::EN_US, 20, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "10m", "20m", "30m", "", "", "", "", "", "", "100m", "200m", "300m", "", "", "", "",
            "", "", "1", "2", "3", "", "", "", "", "", "", "10", "20", "30", "", "", "", "", "",
            "", "100", "200", "300", "", "", "", "", "", "", "1k", "2k", "3k", "", "", "", "", "",
            "", "10k"
        ]
    ); // d3-scale/test/log-test.js:375:3
}

// d3-scale/test/log-test.js: log.ticks() generates ticks that cover the niced domain
#[test]
fn log_ticks_generates_ticks_that_cover_the_niced_domain() {
    let x = scale::log().domain([0.0124123, 1230.4]).nice(10);
    assert_eq!(
        {
            let values_ = &x.ticks(20);
            let format_ = x
                .tick_format_in(&format::Locale::EN_US, 20, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "10m", "20m", "30m", "", "", "", "", "", "", "100m", "200m", "300m", "", "", "", "",
            "", "", "1", "2", "3", "", "", "", "", "", "", "10", "20", "30", "", "", "", "", "",
            "", "100", "200", "300", "", "", "", "", "", "", "1k", "2k", "3k", "", "", "", "", "",
            "", "10k"
        ]
    ); // d3-scale/test/log-test.js:388:3
}

// d3-scale/test/log-test.js: log.tickFormat(count, specifier) returns a filtered format
#[test]
fn log_tickformat_count_specifier_returns_a_filtered_format() {
    let x = scale::log().domain([1000.1, 1.0]);
    assert_eq!(
        {
            let values_ = &x.ticks(10);
            let format_ = x
                .tick_format_in(&format::Locale::EN_US, 10, Some("s"))
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "1k", "", "", "", "", "", "", "300", "200", "100", "", "", "", "", "", "", "30", "20",
            "10", "", "", "", "", "", "", "3", "2", "1"
        ]
    ); // d3-scale/test/log-test.js:411:3
}

// d3-scale/test/log-test.js: log.tickFormat(count, specifier) trims trailing zeroes by default
#[test]
fn log_tickformat_count_specifier_trims_trailing_zeroes_by_default() {
    let x = scale::log().domain([100.1, 0.02]);
    assert_eq!(
        {
            let values_ = &x.ticks(10);
            let format_ = x
                .tick_format_in(&format::Locale::EN_US, 10, Some("f"))
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "100", "", "", "", "", "", "", "", "20", "10", "", "", "", "", "", "", "", "2", "1",
            "", "", "", "", "", "", "", "0.2", "0.1", "", "", "", "", "", "", "", "0.02"
        ]
    ); // d3-scale/test/log-test.js:421:3
}

// d3-scale/test/log-test.js: log.tickFormat(count, specifier) with base two trims trailing zeroes by default
#[test]
fn log_tickformat_count_specifier_with_base_two_trims_trailing_zeroes_by_default() {
    let x = scale::log().base(2.0).domain([100.1, 0.02]);
    assert_eq!(
        {
            let values_ = &x.ticks(10);
            let format_ = x
                .tick_format_in(&format::Locale::EN_US, 10, Some("f"))
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "64", "32", "16", "8", "4", "2", "1", "0.5", "0.25", "0.125", "0.0625", "0.03125"
        ]
    ); // d3-scale/test/log-test.js:432:3
}

// d3-scale/test/log-test.js: log.tickFormat(count, specifier) preserves trailing zeroes if needed
#[test]
fn log_tickformat_count_specifier_preserves_trailing_zeroes_if_needed() {
    let x = scale::log().domain([100.1, 0.02]);
    assert_eq!(
        {
            let values_ = &x.ticks(10);
            let format_ = x
                .tick_format_in(&format::Locale::EN_US, 10, Some(".1f"))
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "100.0", "", "", "", "", "", "", "", "20.0", "10.0", "", "", "", "", "", "", "", "2.0",
            "1.0", "", "", "", "", "", "", "", "0.2", "0.1", "", "", "", "", "", "", "", "0.0"
        ]
    ); // d3-scale/test/log-test.js:439:3
}

// d3-scale/test/log-test.js: log.ticks() returns the empty array when the domain is degenerate
#[test]
fn log_ticks_returns_the_empty_array_when_the_domain_is_degenerate() {
    let mut x = scale::log();
    assert_eq!(
        {
            x = x.domain([0.0, 1.0]);
            x.ticks(10)
        },
        Vec::<f64>::new()
    ); // d3-scale/test/log-test.js:450:3
    assert_eq!(
        {
            x = x.domain([1.0, 0.0]);
            x.ticks(10)
        },
        Vec::<f64>::new()
    ); // d3-scale/test/log-test.js:451:3
    assert_eq!(
        {
            x = x.domain([0.0, -1.0]);
            x.ticks(10)
        },
        Vec::<f64>::new()
    ); // d3-scale/test/log-test.js:452:3
    assert_eq!(
        {
            x = x.domain([-1.0, 0.0]);
            x.ticks(10)
        },
        Vec::<f64>::new()
    ); // d3-scale/test/log-test.js:453:3
    assert_eq!(
        {
            x = x.domain([-1.0, 1.0]);
            x.ticks(10)
        },
        Vec::<f64>::new()
    ); // d3-scale/test/log-test.js:454:3
    assert_eq!(
        {
            x = x.domain([0.0, 0.0]);
            x.ticks(10)
        },
        Vec::<f64>::new()
    ); // d3-scale/test/log-test.js:455:3
}

// d3-scale/test/symlog-test.js: scaleSymlog() has the expected defaults
#[test]
fn scalesymlog_has_the_expected_defaults() {
    let s = scale::symlog();
    assert_eq!(s.get_domain().to_vec(), vec![0.0, 1.0]); // d3-scale/test/symlog-test.js:7:3
    assert_eq!(s.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/symlog-test.js:8:3
}

// d3-scale/test/symlog-test.js: symlog(x) maps a domain value x to a range value y
#[test]
fn symlog_x_maps_a_domain_value_x_to_a_range_value_y() {
    let s = scale::symlog().domain([-100.0, 100.0]);
    assert_eq!(s.apply(-100.0), 0.0); // d3-scale/test/symlog-test.js:15:3
    assert_eq!(s.apply(100.0), 1.0); // d3-scale/test/symlog-test.js:16:3
    assert_eq!(s.apply(0.0), 0.5); // d3-scale/test/symlog-test.js:17:3
}

// d3-scale/test/symlog-test.js: symlog.invert(y) maps a range value y to a domain value x
#[test]
fn symlog_invert_y_maps_a_range_value_y_to_a_domain_value_x() {
    let s = scale::symlog().domain([-100.0, 100.0]);
    assert_in_delta(&s.invert(1.0), &100.0, 1e-6); // d3-scale/test/symlog-test.js:22:3
}

// d3-scale/test/symlog-test.js: symlog.constant(constant) changing the constant does not change the domain or range
#[test]
fn symlog_constant_constant_changing_the_constant_does_not_change_the_domain_or_range() {
    let s = scale::symlog().constant(2.0);
    assert_eq!(s.get_domain().to_vec(), vec![0.0, 1.0]); // d3-scale/test/symlog-test.js:42:3
    assert_eq!(s.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/symlog-test.js:43:3
}

// d3-scale/test/symlog-test.js: symlog.domain(domain) accepts an array of numbers
#[test]
fn symlog_domain_domain_accepts_an_array_of_numbers() {
    assert_eq!(
        scale::symlog().domain([1.0, 0.0]).get_domain().to_vec(),
        vec![1.0, 0.0]
    ); // d3-scale/test/symlog-test.js:48:3
}

// d3-scale/test/symlog-test.js: symlog.clamp() is false by default
#[test]
fn symlog_clamp_is_false_by_default() {
    assert_eq!(scale::symlog().range([10.0, 20.0]).apply(3.0), 30.0); // d3-scale/test/symlog-test.js:101:3
    assert_eq!(scale::symlog().range([10.0, 20.0]).apply(-1.0), 0.0); // d3-scale/test/symlog-test.js:102:3
    assert_eq!(scale::symlog().range([10.0, 20.0]).invert(30.0), 3.0); // d3-scale/test/symlog-test.js:103:3
    assert_eq!(scale::symlog().range([10.0, 20.0]).invert(0.0), -1.0); // d3-scale/test/symlog-test.js:104:3
}

// d3-scale/test/symlog-test.js: symlog.clamp(true) restricts output values to the range
#[test]
fn symlog_clamp_true_restricts_output_values_to_the_range() {
    assert_eq!(
        scale::symlog().clamp(true).range([10.0, 20.0]).apply(2.0),
        20.0
    ); // d3-scale/test/symlog-test.js:108:3
    assert_eq!(
        scale::symlog().clamp(true).range([10.0, 20.0]).apply(-1.0),
        10.0
    ); // d3-scale/test/symlog-test.js:109:3
}

// d3-scale/test/symlog-test.js: symlog.clamp(true) restricts input values to the domain
#[test]
fn symlog_clamp_true_restricts_input_values_to_the_domain() {
    assert_eq!(
        scale::symlog().clamp(true).range([10.0, 20.0]).invert(30.0),
        1.0
    ); // d3-scale/test/symlog-test.js:113:3
    assert_eq!(
        scale::symlog().clamp(true).range([10.0, 20.0]).invert(0.0),
        0.0
    ); // d3-scale/test/symlog-test.js:114:3
}

// d3-scale/test/symlog-test.js: symlog.copy() returns a copy with changes to the domain are isolated
#[test]
fn symlog_copy_returns_a_copy_with_changes_to_the_domain_are_isolated() {
    let mut x = scale::symlog();
    let mut y = x.clone();
    x = x.domain([1.0, 2.0]);
    assert_eq!(y.get_domain().to_vec(), vec![0.0, 1.0]); // d3-scale/test/symlog-test.js:127:3
    assert_eq!(x.apply(1.0), 0.0); // d3-scale/test/symlog-test.js:128:3
    assert_eq!(y.apply(1.0), 1.0); // d3-scale/test/symlog-test.js:129:3
    y = y.domain([2.0, 3.0]);
    assert_eq!(x.apply(2.0), 1.0); // d3-scale/test/symlog-test.js:131:3
    assert_eq!(y.apply(2.0), 0.0); // d3-scale/test/symlog-test.js:132:3
    assert_eq!(x.get_domain().to_vec(), vec![1.0, 2.0]); // d3-scale/test/symlog-test.js:133:3
    assert_eq!(y.get_domain().to_vec(), vec![2.0, 3.0]); // d3-scale/test/symlog-test.js:134:3
    let y2 = {
        x = x.domain([1.0, 1.9]);
        x.clone()
    };
    x = x.nice(5);
    assert_eq!(x.get_domain().to_vec(), vec![1.0, 2.0]); // d3-scale/test/symlog-test.js:137:3
    assert_eq!(y2.get_domain().to_vec(), vec![1.0, 1.9]); // d3-scale/test/symlog-test.js:138:3
}

// d3-scale/test/symlog-test.js: symlog.copy() returns a copy with changes to the range are isolated
#[test]
fn symlog_copy_returns_a_copy_with_changes_to_the_range_are_isolated() {
    let mut x = scale::symlog();
    let mut y = x.clone();
    x = x.range([1.0, 2.0]);
    assert_eq!(x.invert(1.0), 0.0); // d3-scale/test/symlog-test.js:144:3
    assert_eq!(y.invert(1.0), 1.0); // d3-scale/test/symlog-test.js:145:3
    assert_eq!(y.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/symlog-test.js:146:3
    y = y.range([2.0, 3.0]);
    assert_eq!(x.invert(2.0), 1.0); // d3-scale/test/symlog-test.js:148:3
    assert_eq!(y.invert(2.0), 0.0); // d3-scale/test/symlog-test.js:149:3
    assert_eq!(x.get_range().to_vec(), vec![1.0, 2.0]); // d3-scale/test/symlog-test.js:150:3
    assert_eq!(y.get_range().to_vec(), vec![2.0, 3.0]); // d3-scale/test/symlog-test.js:151:3
}

// d3-scale/test/symlog-test.js: symlog.copy() returns a copy with changes to clamping are isolated
#[test]
fn symlog_copy_returns_a_copy_with_changes_to_clamping_are_isolated() {
    let mut x = scale::symlog().clamp(true);
    let mut y = x.clone();
    x = x.clamp(false);
    assert_eq!(x.apply(3.0), 2.0); // d3-scale/test/symlog-test.js:157:3
    assert_eq!(y.apply(2.0), 1.0); // d3-scale/test/symlog-test.js:158:3
    y = y.clamp(false);
    assert_eq!(x.apply(3.0), 2.0); // d3-scale/test/symlog-test.js:161:3
    assert_eq!(y.apply(3.0), 2.0); // d3-scale/test/symlog-test.js:162:3
}

// d3-scale/test/symlog-test.js: symlog().clamp(true).invert(x) cannot return a value outside the domain
#[test]
fn symlog_clamp_true_invert_x_cannot_return_a_value_outside_the_domain() {
    let x = scale::symlog().domain([1.0, 20.0]).clamp(true);
    assert_eq!(x.invert(0.0), 1.0); // d3-scale/test/symlog-test.js:168:3
    assert_eq!(x.invert(1.0), 20.0); // d3-scale/test/symlog-test.js:169:3
}

// d3-scale/test/tickFormat-test.js: tickFormat(start, stop, count) returns a format suitable for the ticks
#[test]
fn tickformat_start_stop_count_returns_a_format_suitable_for_the_ticks() {
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 1.0], 10, None)
            .expect("d3 test")
            .format(0.2),
        "0.2"
    ); // d3-scale/test/tickFormat-test.js:5:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 1.0], 20, None)
            .expect("d3 test")
            .format(0.2),
        "0.20"
    ); // d3-scale/test/tickFormat-test.js:6:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [-100.0, 100.0], 10, None)
            .expect("d3 test")
            .format(-20.0),
        "−20"
    ); // d3-scale/test/tickFormat-test.js:7:3
}

// d3-scale/test/tickFormat-test.js: tickFormat(start, stop, count, specifier) sets the appropriate fixed precision if not specified
#[test]
fn tickformat_start_stop_count_specifier_sets_the_appropriate_fixed_precision_if_not_specified() {
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 1.0], 10, Some("+f"))
            .expect("d3 test")
            .format(0.2),
        "+0.2"
    ); // d3-scale/test/tickFormat-test.js:11:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 1.0], 20, Some("+f"))
            .expect("d3 test")
            .format(0.2),
        "+0.20"
    ); // d3-scale/test/tickFormat-test.js:12:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 1.0], 10, Some("+%"))
            .expect("d3 test")
            .format(0.2),
        "+20%"
    ); // d3-scale/test/tickFormat-test.js:13:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.19, 0.21], 10, Some("+%"))
            .expect("d3 test")
            .format(0.2),
        "+20.0%"
    ); // d3-scale/test/tickFormat-test.js:14:3
}

// d3-scale/test/tickFormat-test.js: tickFormat(start, stop, count, specifier) sets the appropriate round precision if not specified
#[test]
fn tickformat_start_stop_count_specifier_sets_the_appropriate_round_precision_if_not_specified() {
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 9.0], 10, Some(""))
            .expect("d3 test")
            .format(2.10),
        "2"
    ); // d3-scale/test/tickFormat-test.js:18:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 9.0], 100, Some(""))
            .expect("d3 test")
            .format(2.01),
        "2"
    ); // d3-scale/test/tickFormat-test.js:19:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 9.0], 100, Some(""))
            .expect("d3 test")
            .format(2.11),
        "2.1"
    ); // d3-scale/test/tickFormat-test.js:20:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 9.0], 10, Some("e"))
            .expect("d3 test")
            .format(2.10),
        "2e+0"
    ); // d3-scale/test/tickFormat-test.js:21:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 9.0], 100, Some("e"))
            .expect("d3 test")
            .format(2.01),
        "2.0e+0"
    ); // d3-scale/test/tickFormat-test.js:22:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 9.0], 100, Some("e"))
            .expect("d3 test")
            .format(2.11),
        "2.1e+0"
    ); // d3-scale/test/tickFormat-test.js:23:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 9.0], 10, Some("g"))
            .expect("d3 test")
            .format(2.10),
        "2"
    ); // d3-scale/test/tickFormat-test.js:24:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 9.0], 100, Some("g"))
            .expect("d3 test")
            .format(2.01),
        "2.0"
    ); // d3-scale/test/tickFormat-test.js:25:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 9.0], 100, Some("g"))
            .expect("d3 test")
            .format(2.11),
        "2.1"
    ); // d3-scale/test/tickFormat-test.js:26:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 9.0], 10, Some("r"))
            .expect("d3 test")
            .format(2.10e6),
        "2000000"
    ); // d3-scale/test/tickFormat-test.js:27:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 9.0], 100, Some("r"))
            .expect("d3 test")
            .format(2.01e6),
        "2000000"
    ); // d3-scale/test/tickFormat-test.js:28:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 9.0], 100, Some("r"))
            .expect("d3 test")
            .format(2.11e6),
        "2100000"
    ); // d3-scale/test/tickFormat-test.js:29:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 0.9], 10, Some("p"))
            .expect("d3 test")
            .format(0.210),
        "20%"
    ); // d3-scale/test/tickFormat-test.js:30:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.19, 0.21], 10, Some("p"))
            .expect("d3 test")
            .format(0.201),
        "20.1%"
    ); // d3-scale/test/tickFormat-test.js:31:3
}

// d3-scale/test/tickFormat-test.js: tickFormat(start, stop, count, specifier) sets the appropriate prefix precision if not specified
#[test]
fn tickformat_start_stop_count_specifier_sets_the_appropriate_prefix_precision_if_not_specified() {
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 1e6], 10, Some("$s"))
            .expect("d3 test")
            .format(0.51e6),
        "$0.5M"
    ); // d3-scale/test/tickFormat-test.js:35:3
    assert_eq!(
        scale::tick_format(&format::Locale::EN_US, [0.0, 1e6], 100, Some("$s"))
            .expect("d3 test")
            .format(0.501e6),
        "$0.50M"
    ); // d3-scale/test/tickFormat-test.js:36:3
}

// d3-scale/test/tickFormat-test.js: tickFormat(start, stop, count) uses the default precision when the domain is invalid
#[test]
fn tickformat_start_stop_count_uses_the_default_precision_when_the_domain_is_invalid() {
    let f = scale::tick_format(&format::Locale::EN_US, [0.0, f64::NAN], 10, None).expect("d3 test");
    assert_eq!(f.to_string(), " >-,f"); // d3-scale/test/tickFormat-test.js:41:3
    assert_eq!(f.format(0.12), "0.120000"); // d3-scale/test/tickFormat-test.js:42:3
}

// d3-scale/test/identity-test.js: scaleIdentity() has the expected defaults
#[test]
fn scaleidentity_has_the_expected_defaults() {
    let s = scale::identity();
    assert_eq!(s.get_domain().to_vec(), vec![0.0, 1.0]); // d3-scale/test/identity-test.js:6:3
    assert_eq!(s.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/identity-test.js:7:3
}

// d3-scale/test/identity-test.js: scaleIdentity(range) sets the domain and range
#[test]
fn scaleidentity_range_sets_the_domain_and_range() {
    let s = scale::identity().domain([1.0, 2.0]);
    assert_eq!(s.get_domain().to_vec(), vec![1.0, 2.0]); // d3-scale/test/identity-test.js:12:3
    assert_eq!(s.get_range().to_vec(), vec![1.0, 2.0]); // d3-scale/test/identity-test.js:13:3
}

// d3-scale/test/identity-test.js: identity(x) is the identity function
#[test]
fn identity_x_is_the_identity_function() {
    let s = scale::identity().domain([1.0, 2.0]);
    assert_eq!(s.apply(0.5), 0.5); // d3-scale/test/identity-test.js:18:3
    assert_eq!(s.apply(1.0), 1.0); // d3-scale/test/identity-test.js:19:3
    assert_eq!(s.apply(1.5), 1.5); // d3-scale/test/identity-test.js:20:3
    assert_eq!(s.apply(2.0), 2.0); // d3-scale/test/identity-test.js:21:3
    assert_eq!(s.apply(2.5), 2.5); // d3-scale/test/identity-test.js:22:3
}

// d3-scale/test/identity-test.js: identity(undefined) returns unknown
#[test]
fn identity_undefined_returns_unknown() {
    let s = scale::identity().unknown(-1.0);
    assert_eq!(s.apply(f64::NAN), -1.0); // d3-scale/test/identity-test.js:34:3
    assert_eq!(s.apply(0.4), 0.4); // d3-scale/test/identity-test.js:36:3
}

// d3-scale/test/identity-test.js: identity.invert(y) is the identity function
#[test]
fn identity_invert_y_is_the_identity_function() {
    let s = scale::identity().domain([1.0, 2.0]);
    assert_eq!(s.invert(0.5), 0.5); // d3-scale/test/identity-test.js:41:3
    assert_eq!(s.invert(1.0), 1.0); // d3-scale/test/identity-test.js:42:3
    assert_eq!(s.invert(1.5), 1.5); // d3-scale/test/identity-test.js:43:3
    assert_eq!(s.invert(2.0), 2.0); // d3-scale/test/identity-test.js:44:3
    assert_eq!(s.invert(2.5), 2.5); // d3-scale/test/identity-test.js:45:3
}

// d3-scale/test/identity-test.js: identity.domain() defaults to [0, 1]
#[test]
fn identity_domain_defaults_to_0_1() {
    let s = scale::identity();
    assert_eq!(s.get_domain().to_vec(), vec![0.0, 1.0]); // d3-scale/test/identity-test.js:74:3
    assert_eq!(s.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/identity-test.js:75:3
    assert_eq!(s.apply(0.5), 0.5); // d3-scale/test/identity-test.js:76:3
}

// d3-scale/test/identity-test.js: identity.domain() does not affect the identity function
#[test]
fn identity_domain_does_not_affect_the_identity_function() {
    let s = scale::identity().domain([f64::INFINITY, f64::NAN]);
    assert_eq!(s.apply(42.0), 42.0); // d3-scale/test/identity-test.js:133:3
    assert_eq!(s.invert(-42.0), -42.0); // d3-scale/test/identity-test.js:134:3
}

// d3-scale/test/identity-test.js: identity.ticks(count) generates ticks of varying degree
#[test]
fn identity_ticks_count_generates_ticks_of_varying_degree() {
    let mut s = scale::identity();
    assert_eq!(
        {
            let values_ = &s.ticks(1);
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 1, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec!["0", "1"]
    ); // d3-scale/test/identity-test.js:139:3
    assert_eq!(
        {
            let values_ = &s.ticks(2);
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 2, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec!["0.0", "0.5", "1.0"]
    ); // d3-scale/test/identity-test.js:140:3
    assert_eq!(
        {
            let values_ = &s.ticks(5);
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 5, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec!["0.0", "0.2", "0.4", "0.6", "0.8", "1.0"]
    ); // d3-scale/test/identity-test.js:141:3
    assert_eq!(
        {
            let values_ = &s.ticks(10);
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 10, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        vec![
            "0.0", "0.1", "0.2", "0.3", "0.4", "0.5", "0.6", "0.7", "0.8", "0.9", "1.0"
        ]
    ); // d3-scale/test/identity-test.js:142:3
    s = s.domain([1.0, 0.0]);
    assert_eq!(
        {
            let values_ = &s.ticks(1);
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 1, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        {
            let mut v = vec!["0", "1"];
            v.reverse();
            v
        }
    ); // d3-scale/test/identity-test.js:144:3
    assert_eq!(
        {
            let values_ = &s.ticks(2);
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 2, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        {
            let mut v = vec!["0.0", "0.5", "1.0"];
            v.reverse();
            v
        }
    ); // d3-scale/test/identity-test.js:145:3
    assert_eq!(
        {
            let values_ = &s.ticks(5);
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 5, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        {
            let mut v = vec!["0.0", "0.2", "0.4", "0.6", "0.8", "1.0"];
            v.reverse();
            v
        }
    ); // d3-scale/test/identity-test.js:146:3
    assert_eq!(
        {
            let values_ = &s.ticks(10);
            let format_ = s
                .tick_format_in(&format::Locale::EN_US, 10, None)
                .expect("d3 test");
            values_
                .iter()
                .map(|v| format_.format(*v))
                .collect::<Vec<_>>()
        },
        {
            let mut v = vec![
                "0.0", "0.1", "0.2", "0.3", "0.4", "0.5", "0.6", "0.7", "0.8", "0.9", "1.0",
            ];
            v.reverse();
            v
        }
    ); // d3-scale/test/identity-test.js:147:3
}

// d3-scale/test/identity-test.js: identity.tickFormat(count) formats ticks with the appropriate precision
#[test]
fn identity_tickformat_count_formats_ticks_with_the_appropriate_precision() {
    let s = scale::identity().domain([0.123456789, 1.23456789]);
    assert_eq!(
        s.tick_format_in(&format::Locale::EN_US, 1, None)
            .expect("d3 test")
            .format(s.ticks(1)[(0.0) as usize]),
        "1"
    ); // d3-scale/test/identity-test.js:152:3
    assert_eq!(
        s.tick_format_in(&format::Locale::EN_US, 2, None)
            .expect("d3 test")
            .format(s.ticks(2)[(0.0) as usize]),
        "0.5"
    ); // d3-scale/test/identity-test.js:153:3
    assert_eq!(
        s.tick_format_in(&format::Locale::EN_US, 4, None)
            .expect("d3 test")
            .format(s.ticks(4)[(0.0) as usize]),
        "0.2"
    ); // d3-scale/test/identity-test.js:154:3
    assert_eq!(
        s.tick_format_in(&format::Locale::EN_US, 8, None)
            .expect("d3 test")
            .format(s.ticks(8)[(0.0) as usize]),
        "0.2"
    ); // d3-scale/test/identity-test.js:155:3
    assert_eq!(
        s.tick_format_in(&format::Locale::EN_US, 16, None)
            .expect("d3 test")
            .format(s.ticks(16)[(0.0) as usize]),
        "0.15"
    ); // d3-scale/test/identity-test.js:156:3
    assert_eq!(
        s.tick_format_in(&format::Locale::EN_US, 32, None)
            .expect("d3 test")
            .format(s.ticks(32)[(0.0) as usize]),
        "0.15"
    ); // d3-scale/test/identity-test.js:157:3
    assert_eq!(
        s.tick_format_in(&format::Locale::EN_US, 64, None)
            .expect("d3 test")
            .format(s.ticks(64)[(0.0) as usize]),
        "0.14"
    ); // d3-scale/test/identity-test.js:158:3
    assert_eq!(
        s.tick_format_in(&format::Locale::EN_US, 128, None)
            .expect("d3 test")
            .format(s.ticks(128)[(0.0) as usize]),
        "0.13"
    ); // d3-scale/test/identity-test.js:159:3
    assert_eq!(
        s.tick_format_in(&format::Locale::EN_US, 256, None)
            .expect("d3 test")
            .format(s.ticks(256)[(0.0) as usize]),
        "0.125"
    ); // d3-scale/test/identity-test.js:160:3
}

// d3-scale/test/identity-test.js: identity.copy() isolates changes to the domain or range
#[test]
fn identity_copy_isolates_changes_to_the_domain_or_range() {
    let mut s1 = scale::identity();
    let mut s2 = s1.clone();
    let mut s3 = s1.clone();
    s1 = s1.domain([1.0, 2.0]);
    assert_eq!(s2.get_domain().to_vec(), vec![0.0, 1.0]); // d3-scale/test/identity-test.js:168:3
    s2 = s2.domain([2.0, 3.0]);
    assert_eq!(s1.get_domain().to_vec(), vec![1.0, 2.0]); // d3-scale/test/identity-test.js:170:3
    assert_eq!(s2.get_domain().to_vec(), vec![2.0, 3.0]); // d3-scale/test/identity-test.js:171:3
    let mut s4 = s3.clone();
    s3 = s3.range([1.0, 2.0]);
    assert_eq!(s4.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/identity-test.js:174:3
    s4 = s4.range([2.0, 3.0]);
    assert_eq!(s3.get_range().to_vec(), vec![1.0, 2.0]); // d3-scale/test/identity-test.js:176:3
    assert_eq!(s4.get_range().to_vec(), vec![2.0, 3.0]); // d3-scale/test/identity-test.js:177:3
}

// d3-scale/test/band-test.js: scaleBand() has the expected defaults
#[test]
fn scaleband_has_the_expected_defaults() {
    let s = scale::band(Vec::<&str>::new());
    assert_d3_eq(&s.get_domain().to_vec(), &Vec::<f64>::new()); // d3-scale/test/band-test.js:6:3
    assert_eq!(s.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/band-test.js:7:3
    assert_eq!(s.bandwidth(), 1.0); // d3-scale/test/band-test.js:8:3
    assert_eq!(s.step(), 1.0); // d3-scale/test/band-test.js:9:3
}

// d3-scale/test/band-test.js: band(value) computes discrete bands in a continuous range
#[test]
fn band_value_computes_discrete_bands_in_a_continuous_range() {
    let mut s = scale::band(Vec::<&str>::new()).range([0.0, 960.0]);
    assert_eq!(s.apply(&"foo"), None); // d3-scale/test/band-test.js:18:3
    s = s.domain(vec!["foo", "bar"]);
    assert_eq!(s.apply(&"foo"), Some(0.0)); // d3-scale/test/band-test.js:20:3
    assert_eq!(s.apply(&"bar"), Some(480.0)); // d3-scale/test/band-test.js:21:3
    s = s.domain(vec!["a", "b", "c"]).range([0.0, 120.0]);
    assert_d3_eq(
        &s.get_domain()
            .to_vec()
            .iter()
            .map(|k| s.apply(k))
            .collect::<Vec<_>>(),
        &vec![0.0, 40.0, 80.0],
    ); // d3-scale/test/band-test.js:23:3
    assert_eq!(s.bandwidth(), 40.0); // d3-scale/test/band-test.js:24:3
    s = s.padding(0.2);
    assert_d3_eq(
        &s.get_domain()
            .to_vec()
            .iter()
            .map(|k| s.apply(k))
            .collect::<Vec<_>>(),
        &vec![7.5, 45.0, 82.5],
    ); // d3-scale/test/band-test.js:26:3
    assert_eq!(s.bandwidth(), 30.0); // d3-scale/test/band-test.js:27:3
}

// d3-scale/test/band-test.js: band(value) returns undefined for values outside the domain
#[test]
fn band_value_returns_undefined_for_values_outside_the_domain() {
    let s = scale::band(Vec::<&str>::new())
        .domain(vec!["a", "b", "c"])
        .range([0.0, 1.0]);
    assert_eq!(s.apply(&"d"), None); // d3-scale/test/band-test.js:32:3
    assert_eq!(s.apply(&"e"), None); // d3-scale/test/band-test.js:33:3
    assert_eq!(s.apply(&"f"), None); // d3-scale/test/band-test.js:34:3
}

// d3-scale/test/band-test.js: band(value) does not implicitly add values to the domain
#[test]
fn band_value_does_not_implicitly_add_values_to_the_domain() {
    let s = scale::band(Vec::<&str>::new())
        .domain(vec!["a", "b", "c"])
        .range([0.0, 1.0]);
    s.apply(&"d");
    s.apply(&"e");
    assert_eq!(s.get_domain().to_vec(), vec!["a", "b", "c"]); // d3-scale/test/band-test.js:41:3
}

// d3-scale/test/band-test.js: band.step() returns the distance between the starts of adjacent bands
#[test]
fn band_step_returns_the_distance_between_the_starts_of_adjacent_bands() {
    let mut s = scale::band(Vec::<&str>::new()).range([0.0, 960.0]);
    assert_eq!(
        {
            s = s.domain(vec!["foo"]);
            s.step()
        },
        960.0
    ); // d3-scale/test/band-test.js:46:3
    assert_eq!(
        {
            s = s.domain(vec!["foo", "bar"]);
            s.step()
        },
        480.0
    ); // d3-scale/test/band-test.js:47:3
    assert_eq!(
        {
            s = s.domain(vec!["foo", "bar", "baz"]);
            s.step()
        },
        320.0
    ); // d3-scale/test/band-test.js:48:3
    s = s.padding(0.5);
    assert_eq!(
        {
            s = s.domain(vec!["foo"]);
            s.step()
        },
        640.0
    ); // d3-scale/test/band-test.js:50:3
    assert_eq!(
        {
            s = s.domain(vec!["foo", "bar"]);
            s.step()
        },
        384.0
    ); // d3-scale/test/band-test.js:51:3
}

// d3-scale/test/band-test.js: band.bandwidth() returns the width of the band
#[test]
fn band_bandwidth_returns_the_width_of_the_band() {
    let mut s = scale::band(Vec::<&str>::new()).range([0.0, 960.0]);
    assert_eq!(
        {
            s = s.domain(Vec::new());
            s.bandwidth()
        },
        960.0
    ); // d3-scale/test/band-test.js:56:3
    assert_eq!(
        {
            s = s.domain(vec!["foo"]);
            s.bandwidth()
        },
        960.0
    ); // d3-scale/test/band-test.js:57:3
    assert_eq!(
        {
            s = s.domain(vec!["foo", "bar"]);
            s.bandwidth()
        },
        480.0
    ); // d3-scale/test/band-test.js:58:3
    assert_eq!(
        {
            s = s.domain(vec!["foo", "bar", "baz"]);
            s.bandwidth()
        },
        320.0
    ); // d3-scale/test/band-test.js:59:3
    s = s.padding(0.5);
    assert_eq!(
        {
            s = s.domain(Vec::new());
            s.bandwidth()
        },
        480.0
    ); // d3-scale/test/band-test.js:61:3
    assert_eq!(
        {
            s = s.domain(vec!["foo"]);
            s.bandwidth()
        },
        320.0
    ); // d3-scale/test/band-test.js:62:3
    assert_eq!(
        {
            s = s.domain(vec!["foo", "bar"]);
            s.bandwidth()
        },
        192.0
    ); // d3-scale/test/band-test.js:63:3
}

// d3-scale/test/band-test.js: band.domain([]) computes reasonable band and step values
#[test]
fn band_domain_computes_reasonable_band_and_step_values() {
    let mut s = scale::band(Vec::<&str>::new())
        .range([0.0, 960.0])
        .domain(Vec::new());
    assert_eq!(s.step(), 960.0); // d3-scale/test/band-test.js:68:3
    assert_eq!(s.bandwidth(), 960.0); // d3-scale/test/band-test.js:69:3
    s = s.padding(0.5);
    assert_eq!(s.step(), 960.0); // d3-scale/test/band-test.js:71:3
    assert_eq!(s.bandwidth(), 480.0); // d3-scale/test/band-test.js:72:3
    s = s.padding(1.0);
    assert_eq!(s.step(), 960.0); // d3-scale/test/band-test.js:74:3
    assert_eq!(s.bandwidth(), 0.0); // d3-scale/test/band-test.js:75:3
}

// d3-scale/test/band-test.js: band.domain([value]) computes a reasonable singleton band, even with padding
#[test]
fn band_domain_value_computes_a_reasonable_singleton_band_even_with_padding() {
    let mut s = scale::band(Vec::<&str>::new())
        .range([0.0, 960.0])
        .domain(vec!["foo"]);
    assert_eq!(s.apply(&"foo"), Some(0.0)); // d3-scale/test/band-test.js:80:3
    assert_eq!(s.step(), 960.0); // d3-scale/test/band-test.js:81:3
    assert_eq!(s.bandwidth(), 960.0); // d3-scale/test/band-test.js:82:3
    s = s.padding(0.5);
    assert_eq!(s.apply(&"foo"), Some(320.0)); // d3-scale/test/band-test.js:84:3
    assert_eq!(s.step(), 640.0); // d3-scale/test/band-test.js:85:3
    assert_eq!(s.bandwidth(), 320.0); // d3-scale/test/band-test.js:86:3
    s = s.padding(1.0);
    assert_eq!(s.apply(&"foo"), Some(480.0)); // d3-scale/test/band-test.js:88:3
    assert_eq!(s.step(), 480.0); // d3-scale/test/band-test.js:89:3
    assert_eq!(s.bandwidth(), 0.0); // d3-scale/test/band-test.js:90:3
}

// d3-scale/test/band-test.js: band.domain(values) recomputes the bands
#[test]
fn band_domain_values_recomputes_the_bands() {
    let mut s = scale::band(Vec::<&str>::new())
        .domain(vec!["a", "b", "c"])
        .range_round([0.0, 100.0]);
    assert_d3_eq(
        &s.get_domain()
            .to_vec()
            .iter()
            .map(|k| s.apply(k))
            .collect::<Vec<_>>(),
        &vec![1.0, 34.0, 67.0],
    ); // d3-scale/test/band-test.js:95:3
    assert_eq!(s.bandwidth(), 33.0); // d3-scale/test/band-test.js:96:3
    s = s.domain(vec!["a", "b", "c", "d"]);
    assert_d3_eq(
        &s.get_domain()
            .to_vec()
            .iter()
            .map(|k| s.apply(k))
            .collect::<Vec<_>>(),
        &vec![0.0, 25.0, 50.0, 75.0],
    ); // d3-scale/test/band-test.js:98:3
    assert_eq!(s.bandwidth(), 25.0); // d3-scale/test/band-test.js:99:3
}

// d3-scale/test/band-test.js: band.range(values) can be descending
#[test]
fn band_range_values_can_be_descending() {
    let mut s = scale::band(Vec::<&str>::new())
        .domain(vec!["a", "b", "c"])
        .range([120.0, 0.0]);
    assert_d3_eq(
        &s.get_domain()
            .to_vec()
            .iter()
            .map(|k| s.apply(k))
            .collect::<Vec<_>>(),
        &vec![80.0, 40.0, 0.0],
    ); // d3-scale/test/band-test.js:123:3
    assert_eq!(s.bandwidth(), 40.0); // d3-scale/test/band-test.js:124:3
    s = s.padding(0.2);
    assert_d3_eq(
        &s.get_domain()
            .to_vec()
            .iter()
            .map(|k| s.apply(k))
            .collect::<Vec<_>>(),
        &vec![82.5, 45.0, 7.5],
    ); // d3-scale/test/band-test.js:126:3
    assert_eq!(s.bandwidth(), 30.0); // d3-scale/test/band-test.js:127:3
}

// d3-scale/test/band-test.js: band.paddingInner(p) specifies the inner padding p
#[test]
fn band_paddinginner_p_specifies_the_inner_padding_p() {
    let mut s = scale::band(Vec::<&str>::new())
        .domain(vec!["a", "b", "c"])
        .range([120.0, 0.0])
        .padding_inner(0.1)
        .round(true);
    assert_d3_eq(
        &s.get_domain()
            .to_vec()
            .iter()
            .map(|k| s.apply(k))
            .collect::<Vec<_>>(),
        &vec![83.0, 42.0, 1.0],
    ); // d3-scale/test/band-test.js:167:3
    assert_eq!(s.bandwidth(), 37.0); // d3-scale/test/band-test.js:168:3
    s = s.padding_inner(0.2);
    assert_d3_eq(
        &s.get_domain()
            .to_vec()
            .iter()
            .map(|k| s.apply(k))
            .collect::<Vec<_>>(),
        &vec![85.0, 43.0, 1.0],
    ); // d3-scale/test/band-test.js:170:3
    assert_eq!(s.bandwidth(), 34.0); // d3-scale/test/band-test.js:171:3
}

// d3-scale/test/band-test.js: band.paddingOuter(p) specifies the outer padding p
#[test]
fn band_paddingouter_p_specifies_the_outer_padding_p() {
    let mut s = scale::band(Vec::<&str>::new())
        .domain(vec!["a", "b", "c"])
        .range([120.0, 0.0])
        .padding_inner(0.2)
        .padding_outer(0.1);
    assert_d3_eq(
        &s.get_domain()
            .to_vec()
            .iter()
            .map(|k| s.apply(k))
            .collect::<Vec<_>>(),
        &vec![84.0, 44.0, 4.0],
    ); // d3-scale/test/band-test.js:184:3
    assert_eq!(s.bandwidth(), 32.0); // d3-scale/test/band-test.js:185:3
    s = s.padding_outer(1.0);
    assert_d3_eq(
        &s.get_domain()
            .to_vec()
            .iter()
            .map(|k| s.apply(k))
            .collect::<Vec<_>>(),
        &vec![75.0, 50.0, 25.0],
    ); // d3-scale/test/band-test.js:187:3
    assert_eq!(s.bandwidth(), 20.0); // d3-scale/test/band-test.js:188:3
}

// d3-scale/test/band-test.js: band.rangeRound(values) is an alias for band.range(values).round(true)
#[test]
fn band_rangeround_values_is_an_alias_for_band_range_values_round_true() {
    let s = scale::band(Vec::<&str>::new())
        .domain(vec!["a", "b", "c"])
        .range_round([0.0, 100.0]);
    assert_eq!(s.get_range().to_vec(), vec![0.0, 100.0]); // d3-scale/test/band-test.js:201:3
}

// d3-scale/test/band-test.js: band.round(true) computes discrete rounded bands in a continuous range
#[test]
fn band_round_true_computes_discrete_rounded_bands_in_a_continuous_range() {
    let mut s = scale::band(Vec::<&str>::new())
        .domain(vec!["a", "b", "c"])
        .range([0.0, 100.0])
        .round(true);
    assert_d3_eq(
        &s.get_domain()
            .to_vec()
            .iter()
            .map(|k| s.apply(k))
            .collect::<Vec<_>>(),
        &vec![1.0, 34.0, 67.0],
    ); // d3-scale/test/band-test.js:207:3
    assert_eq!(s.bandwidth(), 33.0); // d3-scale/test/band-test.js:208:3
    s = s.padding(0.2);
    assert_d3_eq(
        &s.get_domain()
            .to_vec()
            .iter()
            .map(|k| s.apply(k))
            .collect::<Vec<_>>(),
        &vec![7.0, 38.0, 69.0],
    ); // d3-scale/test/band-test.js:210:3
    assert_eq!(s.bandwidth(), 25.0); // d3-scale/test/band-test.js:211:3
}

// d3-scale/test/band-test.js: band.copy() copies all fields
#[test]
fn band_copy_copies_all_fields() {
    let s1 = scale::band(Vec::<&str>::new())
        .domain(vec!["red", "green"])
        .range([1.0, 2.0])
        .round(true)
        .padding_inner(0.1)
        .padding_outer(0.2);
    let s2 = s1.clone();
    assert_eq!(s2.get_domain().to_vec(), s1.get_domain().to_vec()); // d3-scale/test/band-test.js:217:3
    assert_eq!(s2.get_range().to_vec(), s1.get_range().to_vec()); // d3-scale/test/band-test.js:218:3
}

// d3-scale/test/band-test.js: band.copy() isolates changes to the domain
#[test]
fn band_copy_isolates_changes_to_the_domain() {
    let mut s1 = scale::band(Vec::<&str>::new())
        .domain(vec!["foo", "bar"])
        .range([0.0, 2.0]);
    let mut s2 = s1.clone();
    s1 = s1.domain(vec!["red", "blue"]);
    assert_eq!(s2.get_domain().to_vec(), vec!["foo", "bar"]); // d3-scale/test/band-test.js:228:3
    assert_d3_eq(
        &s1.get_domain()
            .to_vec()
            .iter()
            .map(|k| s1.apply(k))
            .collect::<Vec<_>>(),
        &vec![0.0, 1.0],
    ); // d3-scale/test/band-test.js:229:3
    assert_d3_eq(
        &s2.get_domain()
            .to_vec()
            .iter()
            .map(|k| s2.apply(k))
            .collect::<Vec<_>>(),
        &vec![0.0, 1.0],
    ); // d3-scale/test/band-test.js:230:3
    s2 = s2.domain(vec!["red", "blue"]);
    assert_eq!(s1.get_domain().to_vec(), vec!["red", "blue"]); // d3-scale/test/band-test.js:232:3
    assert_d3_eq(
        &s1.get_domain()
            .to_vec()
            .iter()
            .map(|k| s1.apply(k))
            .collect::<Vec<_>>(),
        &vec![0.0, 1.0],
    ); // d3-scale/test/band-test.js:233:3
    assert_d3_eq(
        &s2.get_domain()
            .to_vec()
            .iter()
            .map(|k| s2.apply(k))
            .collect::<Vec<_>>(),
        &vec![0.0, 1.0],
    ); // d3-scale/test/band-test.js:234:3
}

// d3-scale/test/band-test.js: band.copy() isolates changes to the range
#[test]
fn band_copy_isolates_changes_to_the_range() {
    let mut s1 = scale::band(Vec::<&str>::new())
        .domain(vec!["foo", "bar"])
        .range([0.0, 2.0]);
    let mut s2 = s1.clone();
    s1 = s1.range([3.0, 5.0]);
    assert_eq!(s2.get_range().to_vec(), vec![0.0, 2.0]); // d3-scale/test/band-test.js:241:3
    assert_d3_eq(
        &s1.get_domain()
            .to_vec()
            .iter()
            .map(|k| s1.apply(k))
            .collect::<Vec<_>>(),
        &vec![3.0, 4.0],
    ); // d3-scale/test/band-test.js:242:3
    assert_d3_eq(
        &s2.get_domain()
            .to_vec()
            .iter()
            .map(|k| s2.apply(k))
            .collect::<Vec<_>>(),
        &vec![0.0, 1.0],
    ); // d3-scale/test/band-test.js:243:3
    s2 = s2.range([5.0, 7.0]);
    assert_eq!(s1.get_range().to_vec(), vec![3.0, 5.0]); // d3-scale/test/band-test.js:245:3
    assert_d3_eq(
        &s1.get_domain()
            .to_vec()
            .iter()
            .map(|k| s1.apply(k))
            .collect::<Vec<_>>(),
        &vec![3.0, 4.0],
    ); // d3-scale/test/band-test.js:246:3
    assert_d3_eq(
        &s2.get_domain()
            .to_vec()
            .iter()
            .map(|k| s2.apply(k))
            .collect::<Vec<_>>(),
        &vec![5.0, 6.0],
    ); // d3-scale/test/band-test.js:247:3
}

// d3-scale/test/point-test.js: scalePoint() has the expected defaults
#[test]
fn scalepoint_has_the_expected_defaults() {
    let s = scale::point(Vec::<&str>::new());
    assert_d3_eq(&s.get_domain().to_vec(), &Vec::<f64>::new()); // d3-scale/test/point-test.js:6:3
    assert_eq!(s.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/point-test.js:7:3
    assert_eq!(s.bandwidth(), 0.0); // d3-scale/test/point-test.js:8:3
    assert_eq!(s.step(), 1.0); // d3-scale/test/point-test.js:9:3
}

// d3-scale/test/point-test.js: scalePoint() is similar to scaleBand().paddingInner(1)
#[test]
fn scalepoint_is_similar_to_scaleband_paddinginner_1() {
    let p = scale::point(Vec::<&str>::new())
        .domain(vec!["foo", "bar"])
        .range([0.0, 960.0]);
    let b = scale::band(Vec::<&str>::new())
        .domain(vec!["foo", "bar"])
        .range([0.0, 960.0])
        .padding_inner(1.0);
    assert_eq!(
        p.get_domain()
            .to_vec()
            .iter()
            .map(|k| p.apply(k))
            .collect::<Vec<_>>(),
        b.get_domain()
            .to_vec()
            .iter()
            .map(|k| b.apply(k))
            .collect::<Vec<_>>()
    ); // d3-scale/test/point-test.js:24:3
    assert_eq!(p.bandwidth(), b.bandwidth()); // d3-scale/test/point-test.js:25:3
    assert_eq!(p.step(), b.step()); // d3-scale/test/point-test.js:26:3
}

// d3-scale/test/point-test.js: point.padding(p) sets the band outer padding to p
#[test]
fn point_padding_p_sets_the_band_outer_padding_to_p() {
    let p = scale::point(Vec::<&str>::new())
        .domain(vec!["foo", "bar"])
        .range([0.0, 960.0])
        .padding(0.5);
    let b = scale::band(Vec::<&str>::new())
        .domain(vec!["foo", "bar"])
        .range([0.0, 960.0])
        .padding_inner(1.0)
        .padding_outer(0.5);
    assert_eq!(
        p.get_domain()
            .to_vec()
            .iter()
            .map(|k| p.apply(k))
            .collect::<Vec<_>>(),
        b.get_domain()
            .to_vec()
            .iter()
            .map(|k| b.apply(k))
            .collect::<Vec<_>>()
    ); // d3-scale/test/point-test.js:32:3
    assert_eq!(p.bandwidth(), b.bandwidth()); // d3-scale/test/point-test.js:33:3
    assert_eq!(p.step(), b.step()); // d3-scale/test/point-test.js:34:3
}

// d3-scale/test/point-test.js: point.copy() returns a copy
#[test]
fn point_copy_returns_a_copy() {
    let s = scale::point(Vec::<&str>::new());
    assert_d3_eq(&s.get_domain().to_vec(), &Vec::<f64>::new()); // d3-scale/test/point-test.js:39:3
    assert_eq!(s.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/point-test.js:40:3
    assert_eq!(s.bandwidth(), 0.0); // d3-scale/test/point-test.js:41:3
    assert_eq!(s.step(), 1.0); // d3-scale/test/point-test.js:42:3
}

// d3-scale/test/ordinal-test.js: scaleOrdinal() has the expected defaults
#[test]
fn scaleordinal_has_the_expected_defaults() {
    let mut s = scale::ordinal(Vec::<f64>::new(), Vec::<&str>::new());
    assert_eq!(s.get_domain().to_vec(), Vec::<f64>::new()); // d3-scale/test/ordinal-test.js:6:3
    assert_d3_eq(&s.get_range().to_vec(), &Vec::<f64>::new()); // d3-scale/test/ordinal-test.js:7:3
    assert_eq!(s.apply_or_insert(0.0), None); // d3-scale/test/ordinal-test.js:8:3
    assert_eq!(s.get_domain().to_vec(), vec![0.0]); // d3-scale/test/ordinal-test.js:10:3
}

// d3-scale/test/ordinal-test.js: ordinal(x) maps a unique name x in the domain to the corresponding value y in the range
#[test]
fn ordinal_x_maps_a_unique_name_x_in_the_domain_to_the_corresponding_value_y_in_the_range() {
    let mut s = scale::ordinal(Vec::<f64>::new(), Vec::<&str>::new())
        .domain(vec![0.0, 1.0])
        .range(vec!["foo", "bar"]);
    assert_d3_eq(&s.apply(&0.0), &"foo"); // d3-scale/test/ordinal-test.js:15:3
    assert_d3_eq(&s.apply(&1.0), &"bar"); // d3-scale/test/ordinal-test.js:16:3
    s = s.range(vec!["a", "b", "c"]);
    assert_d3_eq(&s.apply(&0.0), &"a"); // d3-scale/test/ordinal-test.js:18:3
    assert_d3_eq(&s.apply(&1.0), &"b"); // d3-scale/test/ordinal-test.js:21:3
    assert_eq!(s.apply(&2.0), None); // d3-scale/test/ordinal-test.js:23:3
}

// d3-scale/test/ordinal-test.js: ordinal(x) implicitly extends the domain when a range is explicitly specified
#[test]
fn ordinal_x_implicitly_extends_the_domain_when_a_range_is_explicitly_specified() {
    let mut s = scale::ordinal(Vec::<f64>::new(), Vec::<&str>::new()).range(vec!["foo", "bar"]);
    assert_eq!(s.get_domain().to_vec(), Vec::<f64>::new()); // d3-scale/test/ordinal-test.js:28:3
    assert_d3_eq(&s.apply_or_insert(0.0), &"foo"); // d3-scale/test/ordinal-test.js:29:3
    assert_eq!(s.get_domain().to_vec(), vec![0.0]); // d3-scale/test/ordinal-test.js:30:3
    assert_d3_eq(&s.apply_or_insert(1.0), &"bar"); // d3-scale/test/ordinal-test.js:31:3
    assert_eq!(s.get_domain().to_vec(), vec![0.0, 1.0]); // d3-scale/test/ordinal-test.js:32:3
    assert_d3_eq(&s.apply_or_insert(0.0), &"foo"); // d3-scale/test/ordinal-test.js:33:3
    assert_eq!(s.get_domain().to_vec(), vec![0.0, 1.0]); // d3-scale/test/ordinal-test.js:34:3
}

// d3-scale/test/ordinal-test.js: ordinal.domain() returns a copy of the domain
#[test]
fn ordinal_domain_returns_a_copy_of_the_domain() {
    let mut s = scale::ordinal(Vec::<&str>::new(), Vec::<&str>::new()).domain(vec!["red", "green"]);
    let domain = s.get_domain().to_vec();
    s.apply_or_insert("blue");
    assert_eq!(domain, vec!["red", "green"]); // d3-scale/test/ordinal-test.js:48:3
}

// d3-scale/test/ordinal-test.js: ordinal.domain() does not coerce domain values to strings
#[test]
fn ordinal_domain_does_not_coerce_domain_values_to_strings() {
    let s = scale::ordinal(Vec::<f64>::new(), Vec::<&str>::new()).domain(vec![0.0, 1.0]);
    assert_eq!(s.get_domain().to_vec(), vec![0.0, 1.0]); // d3-scale/test/ordinal-test.js:76:3
}

// d3-scale/test/ordinal-test.js: ordinal.domain() does not barf on object built-ins
#[test]
fn ordinal_domain_does_not_barf_on_object_built_ins() {
    let mut s = scale::ordinal(Vec::<&str>::new(), Vec::<f64>::new())
        .domain(vec!["__proto__", "hasOwnProperty"])
        .range(vec![42.0, 43.0]);
    assert_d3_eq(&s.apply_or_insert("__proto__"), &42.0); // d3-scale/test/ordinal-test.js:83:3
    assert_d3_eq(&s.apply_or_insert("hasOwnProperty"), &43.0); // d3-scale/test/ordinal-test.js:84:3
    assert_eq!(s.get_domain().to_vec(), vec!["__proto__", "hasOwnProperty"]); // d3-scale/test/ordinal-test.js:85:3
}

// d3-scale/test/ordinal-test.js: ordinal.domain() does not barf on object built-ins
#[test]
fn ordinal_domain_does_not_barf_on_object_built_ins_2() {
    let mut s = scale::ordinal(Vec::<&str>::new(), Vec::<f64>::new())
        .domain(vec!["__proto__", "hasOwnProperty"])
        .range(vec![42.0, 43.0]);
    assert_d3_eq(&s.apply_or_insert("__proto__"), &42.0); // d3-scale/test/ordinal-test.js:111:3
    assert_d3_eq(&s.apply_or_insert("hasOwnProperty"), &43.0); // d3-scale/test/ordinal-test.js:112:3
    assert_eq!(s.get_domain().to_vec(), vec!["__proto__", "hasOwnProperty"]); // d3-scale/test/ordinal-test.js:113:3
}

// d3-scale/test/ordinal-test.js: ordinal.domain() is ordered by appearance
#[test]
fn ordinal_domain_is_ordered_by_appearance() {
    let mut s = scale::ordinal(Vec::<&str>::new(), Vec::<&str>::new());
    s.apply_or_insert("foo");
    s.apply_or_insert("bar");
    s.apply_or_insert("baz");
    assert_eq!(s.get_domain().to_vec(), vec!["foo", "bar", "baz"]); // d3-scale/test/ordinal-test.js:121:3
    s = s.domain(vec!["baz", "bar"]);
    s.apply_or_insert("foo");
    assert_eq!(s.get_domain().to_vec(), vec!["baz", "bar", "foo"]); // d3-scale/test/ordinal-test.js:124:3
    s = s.domain(vec!["baz", "foo"]);
    assert_eq!(s.get_domain().to_vec(), vec!["baz", "foo"]); // d3-scale/test/ordinal-test.js:126:3
    s = s.domain(Vec::new());
    s.apply_or_insert("foo");
    s.apply_or_insert("bar");
    assert_eq!(s.get_domain().to_vec(), vec!["foo", "bar"]); // d3-scale/test/ordinal-test.js:130:3
}

// d3-scale/test/ordinal-test.js: ordinal.range(values) does not discard implicit domain associations
#[test]
fn ordinal_range_values_does_not_discard_implicit_domain_associations() {
    let mut s = scale::ordinal(Vec::<f64>::new(), Vec::<&str>::new());
    assert_eq!(s.apply_or_insert(0.0), None); // d3-scale/test/ordinal-test.js:155:3
    assert_eq!(s.apply_or_insert(1.0), None); // d3-scale/test/ordinal-test.js:156:3
    s = s.range(vec!["foo", "bar"]);
    assert_d3_eq(&s.apply_or_insert(1.0), &"bar"); // d3-scale/test/ordinal-test.js:158:3
    assert_d3_eq(&s.apply_or_insert(0.0), &"foo"); // d3-scale/test/ordinal-test.js:159:3
}

// d3-scale/test/ordinal-test.js: ordinal(value) recycles values when exhausted
#[test]
fn ordinal_value_recycles_values_when_exhausted() {
    let mut s = scale::ordinal(Vec::<f64>::new(), Vec::<&str>::new()).range(vec!["a", "b", "c"]);
    assert_d3_eq(&s.apply_or_insert(0.0), &"a"); // d3-scale/test/ordinal-test.js:164:3
    assert_d3_eq(&s.apply_or_insert(1.0), &"b"); // d3-scale/test/ordinal-test.js:165:3
    assert_d3_eq(&s.apply_or_insert(2.0), &"c"); // d3-scale/test/ordinal-test.js:166:3
    assert_d3_eq(&s.apply_or_insert(3.0), &"a"); // d3-scale/test/ordinal-test.js:167:3
    assert_d3_eq(&s.apply_or_insert(4.0), &"b"); // d3-scale/test/ordinal-test.js:168:3
    assert_d3_eq(&s.apply_or_insert(5.0), &"c"); // d3-scale/test/ordinal-test.js:169:3
    assert_d3_eq(&s.apply_or_insert(2.0), &"c"); // d3-scale/test/ordinal-test.js:170:3
    assert_d3_eq(&s.apply_or_insert(1.0), &"b"); // d3-scale/test/ordinal-test.js:171:3
    assert_d3_eq(&s.apply_or_insert(0.0), &"a"); // d3-scale/test/ordinal-test.js:172:3
}

// d3-scale/test/ordinal-test.js: ordinal.unknown(x) sets the output value for unknown inputs
#[test]
fn ordinal_unknown_x_sets_the_output_value_for_unknown_inputs() {
    let s = scale::ordinal(Vec::<&str>::new(), Vec::<&str>::new())
        .domain(vec!["foo", "bar"])
        .unknown("gray")
        .range(vec!["red", "blue"]);
    assert_d3_eq(&s.apply(&"foo"), &"red"); // d3-scale/test/ordinal-test.js:177:3
    assert_d3_eq(&s.apply(&"bar"), &"blue"); // d3-scale/test/ordinal-test.js:178:3
    assert_d3_eq(&s.apply(&"baz"), &"gray"); // d3-scale/test/ordinal-test.js:179:3
    assert_d3_eq(&s.apply(&"quux"), &"gray"); // d3-scale/test/ordinal-test.js:180:3
}

// d3-scale/test/ordinal-test.js: ordinal.unknown(x) prevents implicit domain extension if x is not implicit
#[test]
fn ordinal_unknown_x_prevents_implicit_domain_extension_if_x_is_not_implicit() {
    let s = scale::ordinal(Vec::<&str>::new(), Vec::<&str>::new())
        .domain(vec!["foo", "bar"])
        .range(vec!["red", "blue"]);
    assert_eq!(s.apply(&"baz"), None); // d3-scale/test/ordinal-test.js:185:3
    assert_eq!(s.get_domain().to_vec(), vec!["foo", "bar"]); // d3-scale/test/ordinal-test.js:186:3
}

// d3-scale/test/ordinal-test.js: ordinal.copy() copies all fields
#[test]
fn ordinal_copy_copies_all_fields() {
    let s1 = scale::ordinal(Vec::<f64>::new(), Vec::<&str>::new())
        .domain(vec![1.0, 2.0])
        .range(vec!["red", "green"])
        .unknown("gray");
    let s2 = s1.clone();
    assert_eq!(s2.get_domain().to_vec(), s1.get_domain().to_vec()); // d3-scale/test/ordinal-test.js:192:3
    assert_eq!(s2.get_range().to_vec(), s1.get_range().to_vec()); // d3-scale/test/ordinal-test.js:193:3
}

// d3-scale/test/ordinal-test.js: ordinal.copy() changes to the domain are isolated
#[test]
fn ordinal_copy_changes_to_the_domain_are_isolated() {
    let mut s1 = scale::ordinal(Vec::<f64>::new(), Vec::<&str>::new()).range(vec!["foo", "bar"]);
    let mut s2 = s1.clone();
    s1 = s1.domain(vec![1.0, 2.0]);
    assert_eq!(s2.get_domain().to_vec(), Vec::<f64>::new()); // d3-scale/test/ordinal-test.js:201:3
    assert_d3_eq(&s1.apply_or_insert(1.0), &"foo"); // d3-scale/test/ordinal-test.js:202:3
    assert_d3_eq(&s2.apply_or_insert(1.0), &"foo"); // d3-scale/test/ordinal-test.js:203:3
    s2 = s2.domain(vec![2.0, 3.0]);
    assert_d3_eq(&s1.apply_or_insert(2.0), &"bar"); // d3-scale/test/ordinal-test.js:205:3
    assert_d3_eq(&s2.apply_or_insert(2.0), &"foo"); // d3-scale/test/ordinal-test.js:206:3
    assert_eq!(s1.get_domain().to_vec(), vec![1.0, 2.0]); // d3-scale/test/ordinal-test.js:207:3
    assert_eq!(s2.get_domain().to_vec(), vec![2.0, 3.0]); // d3-scale/test/ordinal-test.js:208:3
}

// d3-scale/test/ordinal-test.js: ordinal.copy() changes to the range are isolated
#[test]
fn ordinal_copy_changes_to_the_range_are_isolated() {
    let mut s1 = scale::ordinal(Vec::<f64>::new(), Vec::<&str>::new()).range(vec!["foo", "bar"]);
    let mut s2 = s1.clone();
    s1 = s1.range(vec!["bar", "foo"]);
    assert_d3_eq(&s1.apply_or_insert(1.0), &"bar"); // d3-scale/test/ordinal-test.js:215:3
    assert_d3_eq(&s2.apply_or_insert(1.0), &"foo"); // d3-scale/test/ordinal-test.js:216:3
    assert_eq!(s2.get_range().to_vec(), vec!["foo", "bar"]); // d3-scale/test/ordinal-test.js:217:3
    s2 = s2.range(vec!["foo", "baz"]);
    assert_d3_eq(&s1.apply_or_insert(2.0), &"foo"); // d3-scale/test/ordinal-test.js:219:3
    assert_d3_eq(&s2.apply_or_insert(2.0), &"baz"); // d3-scale/test/ordinal-test.js:220:3
    assert_eq!(s1.get_range().to_vec(), vec!["bar", "foo"]); // d3-scale/test/ordinal-test.js:221:3
    assert_eq!(s2.get_range().to_vec(), vec!["foo", "baz"]); // d3-scale/test/ordinal-test.js:222:3
}

// d3-scale/test/sequential-test.js: scaleSequential() has the expected defaults
#[test]
fn scalesequential_has_the_expected_defaults() {
    let s = scale::sequential([0.0, 1.0], |t: f64| t);
    assert_eq!(s.get_domain().to_vec(), vec![0.0, 1.0]); // d3-scale/test/sequential-test.js:6:3
    assert_eq!(s.apply(-0.5), -0.5); // d3-scale/test/sequential-test.js:10:3
    assert_eq!(s.apply(0.0), 0.0); // d3-scale/test/sequential-test.js:11:3
    assert_eq!(s.apply(0.5), 0.5); // d3-scale/test/sequential-test.js:12:3
    assert_eq!(s.apply(1.0), 1.0); // d3-scale/test/sequential-test.js:13:3
    assert_eq!(s.apply(1.5), 1.5); // d3-scale/test/sequential-test.js:14:3
}

// d3-scale/test/sequential-test.js: sequential.clamp(true) enables clamping
#[test]
fn sequential_clamp_true_enables_clamping() {
    let s = scale::sequential([0.0, 1.0], |t: f64| t).clamp(true);
    assert_eq!(s.apply(-0.5), 0.0); // d3-scale/test/sequential-test.js:20:3
    assert_eq!(s.apply(0.0), 0.0); // d3-scale/test/sequential-test.js:21:3
    assert_eq!(s.apply(0.5), 0.5); // d3-scale/test/sequential-test.js:22:3
    assert_eq!(s.apply(1.0), 1.0); // d3-scale/test/sequential-test.js:23:3
    assert_eq!(s.apply(1.5), 1.0); // d3-scale/test/sequential-test.js:24:3
}

// d3-scale/test/sequential-test.js: sequential.unknown(value) sets the return value for undefined and NaN input
#[test]
fn sequential_unknown_value_sets_the_return_value_for_undefined_and_nan_input() {
    let s = scale::sequential([0.0, 1.0], |t: f64| t).unknown(-1.0);
    assert_eq!(s.apply(f64::NAN), -1.0); // d3-scale/test/sequential-test.js:31:3
    assert_eq!(s.apply(0.4), 0.4); // d3-scale/test/sequential-test.js:33:3
}

// d3-scale/test/sequential-test.js: sequential.domain() handles a degenerate domain
#[test]
fn sequential_domain_handles_a_degenerate_domain() {
    let s = scale::sequential([0.0, 1.0], |t: f64| t).domain([2.0, 2.0]);
    assert_eq!(s.get_domain().to_vec(), vec![2.0, 2.0]); // d3-scale/test/sequential-test.js:51:3
    assert_eq!(s.apply(-1.2), 0.5); // d3-scale/test/sequential-test.js:52:3
    assert_eq!(s.apply(0.6), 0.5); // d3-scale/test/sequential-test.js:53:3
    assert_eq!(s.apply(2.4), 0.5); // d3-scale/test/sequential-test.js:54:3
}

// d3-scale/test/sequential-test.js: sequential.domain() handles a non-numeric domain
#[test]
fn sequential_domain_handles_a_non_numeric_domain() {
    let s = scale::sequential([0.0, 1.0], |t: f64| t).domain([f64::NAN, 2.0]);
    assert!((s.get_domain().to_vec()[(0.0) as usize]).is_nan()); // d3-scale/test/sequential-test.js:59:3
    assert_eq!(s.get_domain().to_vec()[(1.0) as usize], 2.0); // d3-scale/test/sequential-test.js:60:3
    assert!((s.apply(-1.2)).is_nan()); // d3-scale/test/sequential-test.js:61:3
    assert!((s.apply(0.6)).is_nan()); // d3-scale/test/sequential-test.js:62:3
    assert!((s.apply(2.4)).is_nan()); // d3-scale/test/sequential-test.js:63:3
}

// d3-scale/test/sequential-test.js: sequential.copy() returns an isolated copy of the scale
#[test]
fn sequential_copy_returns_an_isolated_copy_of_the_scale() {
    let mut s1 = scale::sequential([0.0, 1.0], |t: f64| t)
        .domain([1.0, 3.0])
        .clamp(true);
    let mut s2 = s1.clone();
    assert_eq!(s2.get_domain().to_vec(), vec![1.0, 3.0]); // d3-scale/test/sequential-test.js:74:3
    s1 = s1.domain([-1.0, 2.0]);
    assert_eq!(s2.get_domain().to_vec(), vec![1.0, 3.0]); // d3-scale/test/sequential-test.js:77:3
    s1 = s1.clamp(false);
    s2 = s2.domain([3.0, 4.0]);
    assert_eq!(s1.get_domain().to_vec(), vec![-1.0, 2.0]); // d3-scale/test/sequential-test.js:81:3
    let _ = s2.clamp(true);
}

// d3-scale/test/sequential-test.js: sequential.interpolator(interpolator) sets the interpolator
#[test]
fn sequential_interpolator_interpolator_sets_the_interpolator() {
    let i0 = |t: f64| t;
    let i1 = |t: f64| t * 2.0;
    let mut s = scale::sequential([0.0, 1.0], |t: f64| t).interpolator(i0);
    s = s.interpolator(i1);
    assert_eq!(s.apply(-0.5), -1.0); // d3-scale/test/sequential-test.js:93:3
    assert_eq!(s.apply(0.0), 0.0); // d3-scale/test/sequential-test.js:94:3
    assert_eq!(s.apply(0.5), 1.0); // d3-scale/test/sequential-test.js:95:3
}

// d3-scale/test/sequential-test.js: sequential.range() returns the computed range
#[test]
fn sequential_range_returns_the_computed_range() {
    let s = scale::sequential([0.0, 1.0], |t: f64| t).interpolator(|t: f64| (t * 2.0) + 1.0);
    assert_eq!(s.get_range().to_vec(), vec![1.0, 3.0]); // d3-scale/test/sequential-test.js:100:3
}

// d3-scale/test/sequential-test.js: sequential.range(range) sets the interpolator
#[test]
fn sequential_range_range_sets_the_interpolator() {
    let s = scale::sequential([0.0, 1.0], |t: f64| t).range([1.0, 3.0]);
    assert_eq!(s.get_range().to_vec(), vec![1.0, 3.0]); // d3-scale/test/sequential-test.js:106:3
}

// d3-scale/test/sequential-test.js: scaleSequential(range) sets the interpolator
#[test]
fn scalesequential_range_sets_the_interpolator() {
    let s = scale::sequential([0.0, 1.0], |t: f64| t).range([1.0, 3.0]);
    assert_eq!(s.get_range().to_vec(), vec![1.0, 3.0]); // d3-scale/test/sequential-test.js:118:3
}

// d3-scale/test/diverging-test.js: scaleDiverging() has the expected defaults
#[test]
fn scalediverging_has_the_expected_defaults() {
    let s = scale::diverging([0.0, 0.5, 1.0], |t: f64| t);
    assert_eq!(s.get_domain().to_vec(), vec![0.0, 0.5, 1.0]); // d3-scale/test/diverging-test.js:6:3
    assert_eq!(s.apply(-0.5), -0.5); // d3-scale/test/diverging-test.js:9:3
    assert_eq!(s.apply(0.0), 0.0); // d3-scale/test/diverging-test.js:10:3
    assert_eq!(s.apply(0.5), 0.5); // d3-scale/test/diverging-test.js:11:3
    assert_eq!(s.apply(1.0), 1.0); // d3-scale/test/diverging-test.js:12:3
    assert_eq!(s.apply(1.5), 1.5); // d3-scale/test/diverging-test.js:13:3
}

// d3-scale/test/diverging-test.js: diverging.clamp(true) enables clamping
#[test]
fn diverging_clamp_true_enables_clamping() {
    let s = scale::diverging([0.0, 0.5, 1.0], |t: f64| t).clamp(true);
    assert_eq!(s.apply(-0.5), 0.0); // d3-scale/test/diverging-test.js:19:3
    assert_eq!(s.apply(0.0), 0.0); // d3-scale/test/diverging-test.js:20:3
    assert_eq!(s.apply(0.5), 0.5); // d3-scale/test/diverging-test.js:21:3
    assert_eq!(s.apply(1.0), 1.0); // d3-scale/test/diverging-test.js:22:3
    assert_eq!(s.apply(1.5), 1.0); // d3-scale/test/diverging-test.js:23:3
}

// d3-scale/test/diverging-test.js: diverging.domain() handles a degenerate domain
#[test]
fn diverging_domain_handles_a_degenerate_domain() {
    let mut s = scale::diverging([0.0, 0.5, 1.0], |t: f64| t).domain([2.0, 2.0, 3.0]);
    assert_eq!(s.get_domain().to_vec(), vec![2.0, 2.0, 3.0]); // d3-scale/test/diverging-test.js:41:3
    assert_eq!(s.apply(-1.2), 0.5); // d3-scale/test/diverging-test.js:42:3
    assert_eq!(s.apply(0.6), 0.5); // d3-scale/test/diverging-test.js:43:3
    assert_eq!(s.apply(2.4), 0.7); // d3-scale/test/diverging-test.js:44:3
    assert_eq!(
        {
            s = s.domain([1.0, 2.0, 2.0]);
            s.get_domain().to_vec()
        },
        vec![1.0, 2.0, 2.0]
    ); // d3-scale/test/diverging-test.js:45:3
    assert_eq!(s.apply(-1.0), -1.0); // d3-scale/test/diverging-test.js:46:3
    assert_eq!(s.apply(0.5), -0.25); // d3-scale/test/diverging-test.js:47:3
    assert_eq!(s.apply(2.4), 0.5); // d3-scale/test/diverging-test.js:48:3
    assert_eq!(
        {
            s = s.domain([2.0, 2.0, 2.0]);
            s.get_domain().to_vec()
        },
        vec![2.0, 2.0, 2.0]
    ); // d3-scale/test/diverging-test.js:49:3
    assert_eq!(s.apply(-1.0), 0.5); // d3-scale/test/diverging-test.js:50:3
    assert_eq!(s.apply(0.5), 0.5); // d3-scale/test/diverging-test.js:51:3
    assert_eq!(s.apply(2.4), 0.5); // d3-scale/test/diverging-test.js:52:3
}

// d3-scale/test/diverging-test.js: diverging.domain() handles a descending domain
#[test]
fn diverging_domain_handles_a_descending_domain() {
    let s = scale::diverging([0.0, 0.5, 1.0], |t: f64| t).domain([4.0, 2.0, 1.0]);
    assert_eq!(s.get_domain().to_vec(), vec![4.0, 2.0, 1.0]); // d3-scale/test/diverging-test.js:57:3
    assert_eq!(s.apply(1.2), 0.9); // d3-scale/test/diverging-test.js:58:3
    assert_eq!(s.apply(2.0), 0.5); // d3-scale/test/diverging-test.js:59:3
    assert_eq!(s.apply(3.0), 0.25); // d3-scale/test/diverging-test.js:60:3
}

// d3-scale/test/diverging-test.js: divergingLog.domain() handles a descending domain
#[test]
fn diverginglog_domain_handles_a_descending_domain() {
    let s = scale::diverging([0.1, 1.0, 10.0], |t: f64| t)
        .transform(scale::transform::Log::new(10.0))
        .domain([3.0, 2.0, 1.0]);
    assert_eq!(s.get_domain().to_vec(), vec![3.0, 2.0, 1.0]); // d3-scale/test/diverging-test.js:65:3
    assert_eq!(s.apply(1.2), 1.0 - 0.1315172029168969); // d3-scale/test/diverging-test.js:66:3
    assert_eq!(s.apply(2.0), 1.0 - 0.5); // d3-scale/test/diverging-test.js:67:3
    assert_d3_eq(&s.apply(2.8), &(1.0 - 0.9149213210862197)); // d3-scale/test/diverging-test.js:68:3
}

// d3-scale/test/diverging-test.js: divergingLog.domain() handles a descending negative domain
#[test]
fn diverginglog_domain_handles_a_descending_negative_domain() {
    let s = scale::diverging([0.1, 1.0, 10.0], |t: f64| t)
        .transform(scale::transform::Log::new(10.0))
        .domain([-1.0, -2.0, -3.0]);
    assert_eq!(s.get_domain().to_vec(), vec![-1.0, -2.0, -3.0]); // d3-scale/test/diverging-test.js:73:3
    assert_eq!(s.apply(-1.2), 0.1315172029168969); // d3-scale/test/diverging-test.js:74:3
    assert_eq!(s.apply(-2.0), 0.5); // d3-scale/test/diverging-test.js:75:3
    assert_d3_eq(&s.apply(-2.8), &0.9149213210862197); // d3-scale/test/diverging-test.js:76:3
}

// d3-scale/test/diverging-test.js: diverging.domain() handles a non-numeric domain
#[test]
fn diverging_domain_handles_a_non_numeric_domain() {
    let mut s = scale::diverging([0.0, 0.5, 1.0], |t: f64| t).domain([f64::NAN, 2.0, 3.0]);
    assert!((s.get_domain().to_vec()[(0.0) as usize]).is_nan()); // d3-scale/test/diverging-test.js:81:3
    assert!((s.apply(-1.2)).is_nan()); // d3-scale/test/diverging-test.js:82:3
    assert!((s.apply(0.6)).is_nan()); // d3-scale/test/diverging-test.js:83:3
    assert_eq!(s.apply(2.4), 0.7); // d3-scale/test/diverging-test.js:84:3
    assert!(
        (({
            s = s.domain([1.0, f64::NAN, 2.0]);
            s.get_domain().to_vec()
        })[(1.0) as usize])
            .is_nan()
    ); // d3-scale/test/diverging-test.js:85:3
    assert!((s.apply(-1.0)).is_nan()); // d3-scale/test/diverging-test.js:86:3
    assert!((s.apply(0.5)).is_nan()); // d3-scale/test/diverging-test.js:87:3
    assert!((s.apply(2.4)).is_nan()); // d3-scale/test/diverging-test.js:88:3
    assert!(
        (({
            s = s.domain([0.0, 1.0, f64::NAN]);
            s.get_domain().to_vec()
        })[(2.0) as usize])
            .is_nan()
    ); // d3-scale/test/diverging-test.js:89:3
    assert_eq!(s.apply(-1.0), -0.5); // d3-scale/test/diverging-test.js:90:3
    assert_eq!(s.apply(0.5), 0.25); // d3-scale/test/diverging-test.js:91:3
    assert!((s.apply(2.4)).is_nan()); // d3-scale/test/diverging-test.js:92:3
}

// d3-scale/test/diverging-test.js: diverging.copy() returns an isolated copy of the scale
#[test]
fn diverging_copy_returns_an_isolated_copy_of_the_scale() {
    let mut s1 = scale::diverging([0.0, 0.5, 1.0], |t: f64| t)
        .domain([1.0, 2.0, 3.0])
        .clamp(true);
    let mut s2 = s1.clone();
    assert_eq!(s2.get_domain().to_vec(), vec![1.0, 2.0, 3.0]); // d3-scale/test/diverging-test.js:103:3
    s1 = s1.domain([-1.0, 1.0, 2.0]);
    assert_eq!(s2.get_domain().to_vec(), vec![1.0, 2.0, 3.0]); // d3-scale/test/diverging-test.js:106:3
    s1 = s1.clamp(false);
    s2 = s2.domain([3.0, 4.0, 5.0]);
    assert_eq!(s1.get_domain().to_vec(), vec![-1.0, 1.0, 2.0]); // d3-scale/test/diverging-test.js:110:3
    let _ = s2.clamp(true);
}

// d3-scale/test/diverging-test.js: diverging.range() returns the computed range
#[test]
fn diverging_range_returns_the_computed_range() {
    let s = scale::diverging([0.0, 0.5, 1.0], |t: f64| t).interpolator(|t: f64| (t * 2.0) + 1.0);
    assert_eq!(s.get_range().to_vec(), vec![1.0, 2.0, 3.0]); // d3-scale/test/diverging-test.js:117:3
}

// d3-scale/test/diverging-test.js: diverging.interpolator(interpolator) sets the interpolator
#[test]
fn diverging_interpolator_interpolator_sets_the_interpolator() {
    let i0 = |t: f64| t;
    let i1 = |t: f64| t * 2.0;
    let mut s = scale::diverging([0.0, 0.5, 1.0], |t: f64| t).interpolator(i0);
    s = s.interpolator(i1);
    assert_eq!(s.apply(-0.5), -1.0); // d3-scale/test/diverging-test.js:127:3
    assert_eq!(s.apply(0.0), 0.0); // d3-scale/test/diverging-test.js:128:3
    assert_eq!(s.apply(0.5), 1.0); // d3-scale/test/diverging-test.js:129:3
}

// d3-scale/test/diverging-test.js: diverging.range(range) sets the interpolator
#[test]
fn diverging_range_range_sets_the_interpolator() {
    let s = scale::diverging([0.0, 0.5, 1.0], |t: f64| t).range([1.0, 3.0, 10.0]);
    assert_eq!(s.get_range().to_vec(), vec![1.0, 3.0, 10.0]); // d3-scale/test/diverging-test.js:135:3
}

// d3-scale/test/diverging-test.js: scaleDiverging(range) sets the interpolator
#[test]
fn scalediverging_range_sets_the_interpolator() {
    let s = scale::diverging([0.0, 0.5, 1.0], |t: f64| t).range([1.0, 3.0, 10.0]);
    assert_eq!(s.get_range().to_vec(), vec![1.0, 3.0, 10.0]); // d3-scale/test/diverging-test.js:147:3
}

// d3-scale/test/quantize-test.js: scaleQuantize() has the expected defaults
#[test]
fn scalequantize_has_the_expected_defaults() {
    let s = scale::quantize([0.0, 1.0], vec![0.0, 1.0]);
    assert_eq!(s.get_domain().to_vec(), vec![0.0, 1.0]); // d3-scale/test/quantize-test.js:8:3
    assert_eq!(s.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/quantize-test.js:9:3
    assert_eq!(s.thresholds().to_vec(), vec![0.5]); // d3-scale/test/quantize-test.js:10:3
    assert_eq!(s.apply(0.25).copied(), Some(0.0)); // d3-scale/test/quantize-test.js:11:3
    assert_eq!(s.apply(0.75).copied(), Some(1.0)); // d3-scale/test/quantize-test.js:12:3
}

// d3-scale/test/quantize-test.js: quantize(value) maps a number to a discrete value in the range
#[test]
fn quantize_value_maps_a_number_to_a_discrete_value_in_the_range() {
    let s = scale::quantize([0.0, 1.0], vec![0.0, 1.0, 2.0]);
    assert_eq!(s.thresholds().to_vec(), vec![(1.0 / 3.0), (2.0 / 3.0)]); // d3-scale/test/quantize-test.js:17:3
    assert_eq!(s.apply(0.0).copied(), Some(0.0)); // d3-scale/test/quantize-test.js:18:3
    assert_eq!(s.apply(0.2).copied(), Some(0.0)); // d3-scale/test/quantize-test.js:19:3
    assert_eq!(s.apply(0.4).copied(), Some(1.0)); // d3-scale/test/quantize-test.js:20:3
    assert_eq!(s.apply(0.6).copied(), Some(1.0)); // d3-scale/test/quantize-test.js:21:3
    assert_eq!(s.apply(0.8).copied(), Some(2.0)); // d3-scale/test/quantize-test.js:22:3
    assert_eq!(s.apply(1.0).copied(), Some(2.0)); // d3-scale/test/quantize-test.js:23:3
}

// d3-scale/test/quantize-test.js: quantize.range() cardinality determines the degree of quantization
#[test]
fn quantize_range_cardinality_determines_the_degree_of_quantization() {
    let mut s = scale::quantize([0.0, 1.0], vec![0.0, 1.0]);
    assert_in_delta(
        &({
            s = s.range(array::range(0.0, 1.001, 0.001));
            s.clone()
        })
        .apply(1.0 / 3.0)
        .copied(),
        &0.333,
        1e-6,
    ); // d3-scale/test/quantize-test.js:63:3
    assert_in_delta(
        &({
            s = s.range(array::range(0.0, 1.010, 0.010));
            s.clone()
        })
        .apply(1.0 / 3.0)
        .copied(),
        &0.330,
        1e-6,
    ); // d3-scale/test/quantize-test.js:64:3
    assert_in_delta(
        &({
            s = s.range(array::range(0.0, 1.100, 0.100));
            s.clone()
        })
        .apply(1.0 / 3.0)
        .copied(),
        &0.300,
        1e-6,
    ); // d3-scale/test/quantize-test.js:65:3
    assert_in_delta(
        &({
            s = s.range(array::range(0.0, 1.200, 0.200));
            s.clone()
        })
        .apply(1.0 / 3.0)
        .copied(),
        &0.400,
        1e-6,
    ); // d3-scale/test/quantize-test.js:66:3
    assert_in_delta(
        &({
            s = s.range(array::range(0.0, 1.250, 0.250));
            s.clone()
        })
        .apply(1.0 / 3.0)
        .copied(),
        &0.250,
        1e-6,
    ); // d3-scale/test/quantize-test.js:67:3
    assert_in_delta(
        &({
            s = s.range(array::range(0.0, 1.500, 0.500));
            s.clone()
        })
        .apply(1.0 / 3.0)
        .copied(),
        &0.500,
        1e-6,
    ); // d3-scale/test/quantize-test.js:68:3
    assert_in_delta(
        &({
            s = s.range(array::range(0.0, 1.0, 1.0));
            s.clone()
        })
        .apply(1.0 / 3.0)
        .copied(),
        &0.0,
        1e-6,
    ); // d3-scale/test/quantize-test.js:69:3
}

// d3-scale/test/quantize-test.js: quantize.invertExtent() maps a value in the range to a domain extent
#[test]
fn quantize_invertextent_maps_a_value_in_the_range_to_a_domain_extent() {
    let s = scale::quantize([0.0, 1.0], vec![0.0, 1.0, 2.0, 3.0]);
    assert_d3_eq(&s.invert_extent(&0.0), &vec![0.00, 0.25]); // d3-scale/test/quantize-test.js:87:3
    assert_d3_eq(&s.invert_extent(&1.0), &vec![0.25, 0.50]); // d3-scale/test/quantize-test.js:88:3
    assert_d3_eq(&s.invert_extent(&2.0), &vec![0.50, 0.75]); // d3-scale/test/quantize-test.js:89:3
    assert_d3_eq(&s.invert_extent(&3.0), &vec![0.75, 1.00]); // d3-scale/test/quantize-test.js:90:3
}

// d3-scale/test/quantize-test.js: quantize.invertExtent() returns [NaN, NaN] when the given value is not in the range
#[test]
fn quantize_invertextent_returns_nan_nan_when_the_given_value_is_not_in_the_range() {
    let s = scale::quantize([0.0, 1.0], vec![0.0, 1.0]);
    assert!(s.invert_extent(&-1.0).is_none()); // d3-scale/test/quantize-test.js:103:3
    assert!(s.invert_extent(&0.5).is_none()); // d3-scale/test/quantize-test.js:104:3
    assert!(s.invert_extent(&2.0).is_none()); // d3-scale/test/quantize-test.js:105:3
}

// d3-scale/test/quantize-test.js: quantize.invertExtent() returns the first match if duplicate values exist in the range
#[test]
fn quantize_invertextent_returns_the_first_match_if_duplicate_values_exist_in_the_range() {
    let s = scale::quantize([0.0, 1.0], vec![0.0, 1.0, 2.0, 0.0]);
    assert_d3_eq(&s.invert_extent(&0.0), &vec![0.00, 0.25]); // d3-scale/test/quantize-test.js:111:3
    assert_d3_eq(&s.invert_extent(&1.0), &vec![0.25, 0.50]); // d3-scale/test/quantize-test.js:112:3
}

// d3-scale/test/quantile-test.js: scaleQuantile() has the expected default
#[test]
fn scalequantile_has_the_expected_default() {
    let s = scale::quantile(Vec::<f64>::new(), Vec::<f64>::new());
    assert_eq!(s.get_domain().to_vec(), Vec::<f64>::new()); // d3-scale/test/quantile-test.js:6:3
    assert_eq!(s.get_range().to_vec(), Vec::<f64>::new()); // d3-scale/test/quantile-test.js:7:3
}

// d3-scale/test/quantile-test.js: quantile(x) uses the R-7 algorithm to compute quantiles
#[test]
fn quantile_x_uses_the_r_7_algorithm_to_compute_quantiles() {
    let mut s = scale::quantile(
        vec![3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0],
        vec![0.0, 1.0, 2.0, 3.0],
    );
    assert_d3_eq(
        &[3.0, 6.0, 6.9, 7.0, 7.1]
            .iter()
            .map(|&v| s.apply(v).copied())
            .collect::<Vec<_>>(),
        &vec![0.0, 0.0, 0.0, 0.0, 0.0],
    ); // d3-scale/test/quantile-test.js:13:3
    assert_d3_eq(
        &[8.0, 8.9]
            .iter()
            .map(|&v| s.apply(v).copied())
            .collect::<Vec<_>>(),
        &vec![1.0, 1.0],
    ); // d3-scale/test/quantile-test.js:14:3
    assert_d3_eq(
        &[9.0, 9.1, 10.0, 13.0]
            .iter()
            .map(|&v| s.apply(v).copied())
            .collect::<Vec<_>>(),
        &vec![2.0, 2.0, 2.0, 2.0],
    ); // d3-scale/test/quantile-test.js:15:3
    assert_d3_eq(
        &[14.9, 15.0, 15.1, 16.0, 20.0]
            .iter()
            .map(|&v| s.apply(v).copied())
            .collect::<Vec<_>>(),
        &vec![3.0, 3.0, 3.0, 3.0, 3.0],
    ); // d3-scale/test/quantile-test.js:16:3
    s = s
        .domain(vec![
            3.0, 6.0, 7.0, 8.0, 8.0, 9.0, 10.0, 13.0, 15.0, 16.0, 20.0,
        ])
        .range(vec![0.0, 1.0, 2.0, 3.0]);
    assert_d3_eq(
        &[3.0, 6.0, 6.9, 7.0, 7.1]
            .iter()
            .map(|&v| s.apply(v).copied())
            .collect::<Vec<_>>(),
        &vec![0.0, 0.0, 0.0, 0.0, 0.0],
    ); // d3-scale/test/quantile-test.js:18:3
    assert_d3_eq(
        &[8.0, 8.9]
            .iter()
            .map(|&v| s.apply(v).copied())
            .collect::<Vec<_>>(),
        &vec![1.0, 1.0],
    ); // d3-scale/test/quantile-test.js:19:3
    assert_d3_eq(
        &[9.0, 9.1, 10.0, 13.0]
            .iter()
            .map(|&v| s.apply(v).copied())
            .collect::<Vec<_>>(),
        &vec![2.0, 2.0, 2.0, 2.0],
    ); // d3-scale/test/quantile-test.js:20:3
    assert_d3_eq(
        &[14.9, 15.0, 15.1, 16.0, 20.0]
            .iter()
            .map(|&v| s.apply(v).copied())
            .collect::<Vec<_>>(),
        &vec![3.0, 3.0, 3.0, 3.0, 3.0],
    ); // d3-scale/test/quantile-test.js:21:3
}

// d3-scale/test/quantile-test.js: quantile(x) returns undefined if the input value is NaN
#[test]
fn quantile_x_returns_undefined_if_the_input_value_is_nan() {
    let s = scale::quantile(
        vec![3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0],
        vec![0.0, 1.0, 2.0, 3.0],
    );
    assert_eq!(s.apply(f64::NAN).copied(), None); // d3-scale/test/quantile-test.js:26:3
}

// d3-scale/test/quantile-test.js: quantile.domain() values are sorted in ascending order
#[test]
fn quantile_domain_values_are_sorted_in_ascending_order() {
    let s = scale::quantile(
        vec![6.0, 3.0, 7.0, 8.0, 8.0, 13.0, 20.0, 15.0, 16.0, 10.0],
        Vec::<f64>::new(),
    );
    assert_eq!(
        s.get_domain().to_vec(),
        vec![3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0]
    ); // d3-scale/test/quantile-test.js:31:3
}

// d3-scale/test/quantile-test.js: quantile.domain() values are allowed to be zero
#[test]
fn quantile_domain_values_are_allowed_to_be_zero() {
    let s = scale::quantile(
        [
            Some(1.0),
            Some(2.0),
            Some(0.0),
            Some(0.0),
            Option::<f64>::None,
        ]
        .iter()
        .map(|v| v.unwrap_or(f64::NAN))
        .collect::<Vec<f64>>(),
        Vec::<f64>::new(),
    );
    assert_eq!(s.get_domain().to_vec(), vec![0.0, 0.0, 1.0, 2.0]); // d3-scale/test/quantile-test.js:46:3
}

// d3-scale/test/quantile-test.js: quantile.domain() non-numeric values are ignored
#[test]
fn quantile_domain_non_numeric_values_are_ignored() {
    let s = scale::quantile(
        [
            Some(6.0),
            Some(3.0),
            Some(f64::NAN),
            Option::<f64>::None,
            Some(7.0),
            Some(8.0),
            Some(8.0),
            Some(13.0),
            Option::<f64>::None,
            Some(20.0),
            Some(15.0),
            Some(16.0),
            Some(10.0),
            Some(f64::NAN),
        ]
        .iter()
        .map(|v| v.unwrap_or(f64::NAN))
        .collect::<Vec<f64>>(),
        Vec::<f64>::new(),
    );
    assert_eq!(
        s.get_domain().to_vec(),
        vec![3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0]
    ); // d3-scale/test/quantile-test.js:51:3
}

// d3-scale/test/quantile-test.js: quantile.quantiles() returns the inner thresholds
#[test]
fn quantile_quantiles_returns_the_inner_thresholds() {
    let mut s = scale::quantile(
        vec![3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0],
        vec![0.0, 1.0, 2.0, 3.0],
    );
    assert_eq!(s.quantiles().to_vec(), vec![7.25, 9.0, 14.5]); // d3-scale/test/quantile-test.js:56:3
    s = s
        .domain(vec![
            3.0, 6.0, 7.0, 8.0, 8.0, 9.0, 10.0, 13.0, 15.0, 16.0, 20.0,
        ])
        .range(vec![0.0, 1.0, 2.0, 3.0]);
    assert_eq!(s.quantiles().to_vec(), vec![7.5, 9.0, 14.0]); // d3-scale/test/quantile-test.js:58:3
}

// d3-scale/test/quantile-test.js: quantile.range() cardinality determines the number of quantiles
#[test]
fn quantile_range_cardinality_determines_the_number_of_quantiles() {
    let mut s = scale::quantile(
        vec![3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0],
        Vec::<f64>::new(),
    );
    assert_eq!(
        {
            s = s.range(vec![0.0, 1.0, 2.0, 3.0]);
            s.quantiles().to_vec()
        },
        vec![7.25, 9.0, 14.5]
    ); // d3-scale/test/quantile-test.js:63:3
    assert_eq!(
        {
            s = s.range(vec![0.0, 1.0]);
            s.quantiles().to_vec()
        },
        vec![9.0]
    ); // d3-scale/test/quantile-test.js:64:3
}

// d3-scale/test/quantile-test.js: quantile.invertExtent() maps a value in the range to a domain extent
#[test]
fn quantile_invertextent_maps_a_value_in_the_range_to_a_domain_extent() {
    let s = scale::quantile(
        vec![3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0],
        vec![0.0, 1.0, 2.0, 3.0],
    );
    assert_d3_eq(&s.invert_extent(&0.0), &vec![3.0, 7.25]); // d3-scale/test/quantile-test.js:87:3
    assert_d3_eq(&s.invert_extent(&1.0), &vec![7.25, 9.0]); // d3-scale/test/quantile-test.js:88:3
    assert_d3_eq(&s.invert_extent(&2.0), &vec![9.0, 14.5]); // d3-scale/test/quantile-test.js:89:3
    assert_d3_eq(&s.invert_extent(&3.0), &vec![14.5, 20.0]); // d3-scale/test/quantile-test.js:90:3
}

// d3-scale/test/quantile-test.js: quantile.invertExtent() returns [NaN, NaN] when the given value is not in the range
#[test]
fn quantile_invertextent_returns_nan_nan_when_the_given_value_is_not_in_the_range() {
    let s = scale::quantile(
        vec![3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0],
        Vec::<f64>::new(),
    );
    assert!(s.invert_extent(&-1.0).is_none()); // d3-scale/test/quantile-test.js:103:3
    assert!(s.invert_extent(&0.5).is_none()); // d3-scale/test/quantile-test.js:104:3
    assert!(s.invert_extent(&2.0).is_none()); // d3-scale/test/quantile-test.js:105:3
}

// d3-scale/test/quantile-test.js: quantile.invertExtent() returns the first match if duplicate values exist in the range
#[test]
fn quantile_invertextent_returns_the_first_match_if_duplicate_values_exist_in_the_range() {
    let s = scale::quantile(
        vec![3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0],
        vec![0.0, 1.0, 2.0, 0.0],
    );
    assert_d3_eq(&s.invert_extent(&0.0), &vec![3.0, 7.25]); // d3-scale/test/quantile-test.js:111:3
    assert_d3_eq(&s.invert_extent(&1.0), &vec![7.25, 9.0]); // d3-scale/test/quantile-test.js:112:3
    assert_d3_eq(&s.invert_extent(&2.0), &vec![9.0, 14.5]); // d3-scale/test/quantile-test.js:113:3
}

// d3-scale/test/threshold-test.js: scaleThreshold() has the expected defaults
#[test]
fn scalethreshold_has_the_expected_defaults() {
    let x = scale::threshold(vec![0.5], vec![0.0, 1.0]);
    assert_eq!(x.get_domain().to_vec(), vec![0.5]); // d3-scale/test/threshold-test.js:6:3
    assert_eq!(x.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/threshold-test.js:7:3
    assert_eq!(x.apply(0.50).copied(), Some(1.0)); // d3-scale/test/threshold-test.js:8:3
    assert_eq!(x.apply(0.49).copied(), Some(0.0)); // d3-scale/test/threshold-test.js:9:3
}

// d3-scale/test/threshold-test.js: threshold(x) maps a number to a discrete value in the range
#[test]
fn threshold_x_maps_a_number_to_a_discrete_value_in_the_range() {
    let x = scale::threshold(vec![(1.0 / 3.0), (2.0 / 3.0)], vec!["a", "b", "c"]);
    assert_eq!(x.apply(0.0).copied(), Some("a")); // d3-scale/test/threshold-test.js:14:3
    assert_eq!(x.apply(0.2).copied(), Some("a")); // d3-scale/test/threshold-test.js:15:3
    assert_eq!(x.apply(0.4).copied(), Some("b")); // d3-scale/test/threshold-test.js:16:3
    assert_eq!(x.apply(0.6).copied(), Some("b")); // d3-scale/test/threshold-test.js:17:3
    assert_eq!(x.apply(0.8).copied(), Some("c")); // d3-scale/test/threshold-test.js:18:3
    assert_eq!(x.apply(1.0).copied(), Some("c")); // d3-scale/test/threshold-test.js:19:3
}

// d3-scale/test/threshold-test.js: threshold(x) returns undefined if the specified value x is not orderable
#[test]
fn threshold_x_returns_undefined_if_the_specified_value_x_is_not_orderable() {
    let x = scale::threshold(vec![(1.0 / 3.0), (2.0 / 3.0)], vec!["a", "b", "c"]);
    assert_eq!(x.apply(f64::NAN).copied(), None); // d3-scale/test/threshold-test.js:25:3
    assert_eq!(x.apply(f64::NAN).copied(), None); // d3-scale/test/threshold-test.js:26:3
}
