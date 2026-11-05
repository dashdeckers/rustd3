//! d3's tests of d3-interpolate, as assertions on rustd3-core; each names d3's file, line and column.

use rustd3_core::color::{Cubehelix, Hcl, Hsl, Lab, Rgb};
use rustd3_core::interpolate;

use super::support::assert_in_delta;

// d3-interpolate/test/number-test.js: interpolateNumber(a, b) interpolates between two numbers a and b
#[test]
fn interpolatenumber_a_b_interpolates_between_two_numbers_a_and_b() {
    let i = interpolate::number(10.0, 42.0);
    assert_in_delta(&i(0.0), &10.0, 1e-6); // d3-interpolate/test/number-test.js:7:3
    assert_in_delta(&i(0.1), &13.2, 1e-6); // d3-interpolate/test/number-test.js:8:3
    assert_in_delta(&i(0.2), &16.4, 1e-6); // d3-interpolate/test/number-test.js:9:3
    assert_in_delta(&i(0.3), &19.6, 1e-6); // d3-interpolate/test/number-test.js:10:3
    assert_in_delta(&i(0.4), &22.8, 1e-6); // d3-interpolate/test/number-test.js:11:3
    assert_in_delta(&i(0.5), &26.0, 1e-6); // d3-interpolate/test/number-test.js:12:3
    assert_in_delta(&i(0.6), &29.2, 1e-6); // d3-interpolate/test/number-test.js:13:3
    assert_in_delta(&i(0.7), &32.4, 1e-6); // d3-interpolate/test/number-test.js:14:3
    assert_in_delta(&i(0.8), &35.6, 1e-6); // d3-interpolate/test/number-test.js:15:3
    assert_in_delta(&i(0.9), &38.8, 1e-6); // d3-interpolate/test/number-test.js:16:3
    assert_in_delta(&i(1.0), &42.0, 1e-6); // d3-interpolate/test/number-test.js:17:3
}

// d3-interpolate/test/number-test.js: interpolateNumber(a, b) gives exact ends for t=0 and t=1
#[test]
fn interpolatenumber_a_b_gives_exact_ends_for_t_0_and_t_1() {
    let a = 2e+42;
    let b = 335.0;
    assert_eq!(interpolate::number(a, b)(1.0), b); // d3-interpolate/test/number-test.js:23:3
    assert_eq!(interpolate::number(a, b)(0.0), a); // d3-interpolate/test/number-test.js:24:3
}

// d3-interpolate/test/round-test.js: interpolateRound(a, b) interpolates between two numbers a and b, and then rounds
#[test]
fn interpolateround_a_b_interpolates_between_two_numbers_a_and_b_and_then_rounds() {
    let i = interpolate::round(10.0, 42.0);
    assert_eq!(i(0.0), 10.0); // d3-interpolate/test/round-test.js:6:3
    assert_eq!(i(0.1), 13.0); // d3-interpolate/test/round-test.js:7:3
    assert_eq!(i(0.2), 16.0); // d3-interpolate/test/round-test.js:8:3
    assert_eq!(i(0.3), 20.0); // d3-interpolate/test/round-test.js:9:3
    assert_eq!(i(0.4), 23.0); // d3-interpolate/test/round-test.js:10:3
    assert_eq!(i(0.5), 26.0); // d3-interpolate/test/round-test.js:11:3
    assert_eq!(i(0.6), 29.0); // d3-interpolate/test/round-test.js:12:3
    assert_eq!(i(0.7), 32.0); // d3-interpolate/test/round-test.js:13:3
    assert_eq!(i(0.8), 36.0); // d3-interpolate/test/round-test.js:14:3
    assert_eq!(i(0.9), 39.0); // d3-interpolate/test/round-test.js:15:3
    assert_eq!(i(1.0), 42.0); // d3-interpolate/test/round-test.js:16:3
}

// d3-interpolate/test/round-test.js: interpolateRound(a, b) does not pre-round a and b
#[test]
fn interpolateround_a_b_does_not_pre_round_a_and_b() {
    let i = interpolate::round(2.6, 3.6);
    assert_eq!(i(0.6), 3.0); // d3-interpolate/test/round-test.js:21:3
}

// d3-interpolate/test/round-test.js: interpolateRound(a, b) gives exact ends for t=0 and t=1
#[test]
fn interpolateround_a_b_gives_exact_ends_for_t_0_and_t_1() {
    let a = 2e+42;
    let b = 335.0;
    assert_eq!(interpolate::round(a, b)(1.0), b); // d3-interpolate/test/round-test.js:26:3
    assert_eq!(interpolate::round(a, b)(0.0), a); // d3-interpolate/test/round-test.js:27:3
}

// d3-interpolate/test/rgb-test.js: interpolateRgb(a, b) converts a and b to RGB colors
#[test]
fn interpolatergb_a_b_converts_a_and_b_to_rgb_colors() {
    assert_eq!(
        interpolate::rgb(
            "steelblue".parse::<Rgb>().expect("a CSS color"),
            "brown".parse::<Rgb>().expect("a CSS color")
        )(0.0)
        .to_string(),
        "steelblue".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/rgb-test.js:6:3
    assert_eq!(
        interpolate::rgb(
            "steelblue".parse::<Rgb>().expect("a CSS color"),
            Rgb::from("brown".parse::<Hsl>().expect("a CSS color"))
        )(1.0)
        .to_string(),
        "brown".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/rgb-test.js:7:3
    assert_eq!(
        interpolate::rgb(
            "steelblue".parse::<Rgb>().expect("a CSS color"),
            "brown".parse::<Rgb>().expect("a CSS color")
        )(1.0)
        .to_string(),
        "brown".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/rgb-test.js:8:3
}

// d3-interpolate/test/rgb-test.js: interpolateRgb(a, b) interpolates in RGB and returns an RGB string
#[test]
fn interpolatergb_a_b_interpolates_in_rgb_and_returns_an_rgb_string() {
    assert_eq!(
        interpolate::rgb(
            "steelblue".parse::<Rgb>().expect("a CSS color"),
            "#f00".parse::<Rgb>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgb(107, 104, 144)"
    ); // d3-interpolate/test/rgb-test.js:12:3
    assert_eq!(
        interpolate::rgb(
            "rgba(70, 130, 180, 1)".parse::<Rgb>().expect("a CSS color"),
            "rgba(255, 0, 0, 0.2)".parse::<Rgb>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgba(107, 104, 144, 0.84)"
    ); // d3-interpolate/test/rgb-test.js:13:3
}

// d3-interpolate/test/rgb-test.js: interpolateRgb(a, b) uses b’s channel value when a’s channel value is undefined
#[test]
fn interpolatergb_a_b_uses_b_s_channel_value_when_a_s_channel_value_is_undefined() {
    assert_eq!(
        interpolate::rgb(Rgb::new(f64::NAN, 20.0, 40.0), Rgb::new(60.0, 80.0, 100.0))(0.5)
            .to_string(),
        Rgb::new(60.0, 50.0, 70.0).to_string()
    ); // d3-interpolate/test/rgb-test.js:18:3
    assert_eq!(
        interpolate::rgb(Rgb::new(20.0, f64::NAN, 40.0), Rgb::new(60.0, 80.0, 100.0))(0.5)
            .to_string(),
        Rgb::new(40.0, 80.0, 70.0).to_string()
    ); // d3-interpolate/test/rgb-test.js:19:3
    assert_eq!(
        interpolate::rgb(Rgb::new(20.0, 40.0, f64::NAN), Rgb::new(60.0, 80.0, 100.0))(0.5)
            .to_string(),
        Rgb::new(40.0, 60.0, 100.0).to_string()
    ); // d3-interpolate/test/rgb-test.js:20:3
}

// d3-interpolate/test/rgb-test.js: interpolateRgb(a, b) uses a’s channel value when b’s channel value is undefined
#[test]
fn interpolatergb_a_b_uses_a_s_channel_value_when_b_s_channel_value_is_undefined() {
    assert_eq!(
        interpolate::rgb(Rgb::new(60.0, 80.0, 100.0), Rgb::new(f64::NAN, 20.0, 40.0))(0.5)
            .to_string(),
        Rgb::new(60.0, 50.0, 70.0).to_string()
    ); // d3-interpolate/test/rgb-test.js:25:3
    assert_eq!(
        interpolate::rgb(Rgb::new(60.0, 80.0, 100.0), Rgb::new(20.0, f64::NAN, 40.0))(0.5)
            .to_string(),
        Rgb::new(40.0, 80.0, 70.0).to_string()
    ); // d3-interpolate/test/rgb-test.js:26:3
    assert_eq!(
        interpolate::rgb(Rgb::new(60.0, 80.0, 100.0), Rgb::new(20.0, 40.0, f64::NAN))(0.5)
            .to_string(),
        Rgb::new(40.0, 60.0, 100.0).to_string()
    ); // d3-interpolate/test/rgb-test.js:27:3
}

// d3-interpolate/test/rgb-test.js: interpolateRgb.gamma(3)(a, b) returns the expected values
#[test]
fn interpolatergb_gamma_3_a_b_returns_the_expected_values() {
    assert_eq!(
        interpolate::rgb_gamma(
            3.0,
            "steelblue".parse::<Rgb>().expect("a CSS color"),
            "#f00".parse::<Rgb>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgb(153, 121, 167)"
    ); // d3-interpolate/test/rgb-test.js:31:3
}

// d3-interpolate/test/rgb-test.js: interpolateRgb.gamma(3)(a, b) uses linear interpolation for opacity
#[test]
fn interpolatergb_gamma_3_a_b_uses_linear_interpolation_for_opacity() {
    assert_eq!(
        interpolate::rgb_gamma(
            3.0,
            "transparent".parse::<Rgb>().expect("a CSS color"),
            "#f00".parse::<Rgb>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgba(255, 0, 0, 0.2)"
    ); // d3-interpolate/test/rgb-test.js:35:3
}

// d3-interpolate/test/rgb-test.js: interpolateRgb(a, b) is equivalent to interpolateRgb.gamma(1)(a, b)
#[test]
fn interpolatergb_a_b_is_equivalent_to_interpolatergb_gamma_1_a_b() {
    let i0 = interpolate::rgb_gamma(
        1.0,
        "purple".parse::<Rgb>().expect("a CSS color"),
        "orange".parse::<Rgb>().expect("a CSS color"),
    );
    let i1 = interpolate::rgb(
        "purple".parse::<Rgb>().expect("a CSS color"),
        "orange".parse::<Rgb>().expect("a CSS color"),
    );
    assert_eq!(i1(0.0).to_string(), i0(0.0).to_string()); // d3-interpolate/test/rgb-test.js:45:3
    assert_eq!(i1(0.2).to_string(), i0(0.2).to_string()); // d3-interpolate/test/rgb-test.js:46:3
    assert_eq!(i1(0.4).to_string(), i0(0.4).to_string()); // d3-interpolate/test/rgb-test.js:47:3
    assert_eq!(i1(0.6).to_string(), i0(0.6).to_string()); // d3-interpolate/test/rgb-test.js:48:3
    assert_eq!(i1(0.8).to_string(), i0(0.8).to_string()); // d3-interpolate/test/rgb-test.js:49:3
    assert_eq!(i1(1.0).to_string(), i0(1.0).to_string()); // d3-interpolate/test/rgb-test.js:50:3
}

// d3-interpolate/test/hsl-test.js: interpolateHsl(a, b) converts a and b to HSL colors
#[test]
fn interpolatehsl_a_b_converts_a_and_b_to_hsl_colors() {
    assert_eq!(
        interpolate::hsl(
            "steelblue".parse::<Hsl>().expect("a CSS color"),
            "brown".parse::<Hsl>().expect("a CSS color")
        )(0.0)
        .to_string(),
        "steelblue".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/hsl-test.js:6:3
    assert_eq!(
        interpolate::hsl(
            "steelblue".parse::<Hsl>().expect("a CSS color"),
            "brown".parse::<Hsl>().expect("a CSS color")
        )(1.0)
        .to_string(),
        "brown".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/hsl-test.js:7:3
    assert_eq!(
        interpolate::hsl(
            "steelblue".parse::<Hsl>().expect("a CSS color"),
            Hsl::from("brown".parse::<Rgb>().expect("a CSS color"))
        )(1.0)
        .to_string(),
        "brown".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/hsl-test.js:8:3
}

// d3-interpolate/test/hsl-test.js: interpolateHsl(a, b) interpolates in HSL and returns an RGB string
#[test]
fn interpolatehsl_a_b_interpolates_in_hsl_and_returns_an_rgb_string() {
    assert_eq!(
        interpolate::hsl(
            "steelblue".parse::<Hsl>().expect("a CSS color"),
            "#f00".parse::<Hsl>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgb(56, 61, 195)"
    ); // d3-interpolate/test/hsl-test.js:12:3
    assert_eq!(
        interpolate::hsl(
            "rgba(70, 130, 180, 1)".parse::<Hsl>().expect("a CSS color"),
            "rgba(255, 0, 0, 0.2)".parse::<Hsl>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgba(56, 61, 195, 0.84)"
    ); // d3-interpolate/test/hsl-test.js:13:3
}

// d3-interpolate/test/hsl-test.js: interpolateHsl(a, b) uses the shortest path when interpolating hue
#[test]
fn interpolatehsl_a_b_uses_the_shortest_path_when_interpolating_hue() {
    let i = interpolate::hsl(
        "hsl(10,50%,50%)".parse::<Hsl>().expect("a CSS color"),
        "hsl(350,50%,50%)".parse::<Hsl>().expect("a CSS color"),
    );
    assert_eq!(i(0.0).to_string(), "rgb(191, 85, 64)"); // d3-interpolate/test/hsl-test.js:18:3
    assert_eq!(i(0.2).to_string(), "rgb(191, 77, 64)"); // d3-interpolate/test/hsl-test.js:19:3
    assert_eq!(i(0.4).to_string(), "rgb(191, 68, 64)"); // d3-interpolate/test/hsl-test.js:20:3
    assert_eq!(i(0.6).to_string(), "rgb(191, 64, 68)"); // d3-interpolate/test/hsl-test.js:21:3
    assert_eq!(i(0.8).to_string(), "rgb(191, 64, 77)"); // d3-interpolate/test/hsl-test.js:22:3
    assert_eq!(i(1.0).to_string(), "rgb(191, 64, 85)"); // d3-interpolate/test/hsl-test.js:23:3
}

// d3-interpolate/test/hsl-test.js: interpolateHsl(a, b) uses a’s hue when b’s hue is undefined
#[test]
fn interpolatehsl_a_b_uses_a_s_hue_when_b_s_hue_is_undefined() {
    assert_eq!(
        interpolate::hsl(
            "#f60".parse::<Hsl>().expect("a CSS color"),
            "#000".parse::<Hsl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(128, 51, 0)"
    ); // d3-interpolate/test/hsl-test.js:27:3
    assert_eq!(
        interpolate::hsl(
            "#6f0".parse::<Hsl>().expect("a CSS color"),
            "#fff".parse::<Hsl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(179, 255, 128)"
    ); // d3-interpolate/test/hsl-test.js:28:3
}

// d3-interpolate/test/hsl-test.js: interpolateHsl(a, b) uses b’s hue when a’s hue is undefined
#[test]
fn interpolatehsl_a_b_uses_b_s_hue_when_a_s_hue_is_undefined() {
    assert_eq!(
        interpolate::hsl(
            "#000".parse::<Hsl>().expect("a CSS color"),
            "#f60".parse::<Hsl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(128, 51, 0)"
    ); // d3-interpolate/test/hsl-test.js:32:3
    assert_eq!(
        interpolate::hsl(
            "#fff".parse::<Hsl>().expect("a CSS color"),
            "#6f0".parse::<Hsl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(179, 255, 128)"
    ); // d3-interpolate/test/hsl-test.js:33:3
}

// d3-interpolate/test/hsl-test.js: interpolateHsl(a, b) uses a’s saturation when b’s saturation is undefined
#[test]
fn interpolatehsl_a_b_uses_a_s_saturation_when_b_s_saturation_is_undefined() {
    assert_eq!(
        interpolate::hsl(
            "#ccc".parse::<Hsl>().expect("a CSS color"),
            "#000".parse::<Hsl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(102, 102, 102)"
    ); // d3-interpolate/test/hsl-test.js:37:3
    assert_eq!(
        interpolate::hsl(
            "#f00".parse::<Hsl>().expect("a CSS color"),
            "#000".parse::<Hsl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(128, 0, 0)"
    ); // d3-interpolate/test/hsl-test.js:38:3
}

// d3-interpolate/test/hsl-test.js: interpolateHsl(a, b) uses b’s saturation when a’s saturation is undefined
#[test]
fn interpolatehsl_a_b_uses_b_s_saturation_when_a_s_saturation_is_undefined() {
    assert_eq!(
        interpolate::hsl(
            "#000".parse::<Hsl>().expect("a CSS color"),
            "#ccc".parse::<Hsl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(102, 102, 102)"
    ); // d3-interpolate/test/hsl-test.js:42:3
    assert_eq!(
        interpolate::hsl(
            "#000".parse::<Hsl>().expect("a CSS color"),
            "#f00".parse::<Hsl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(128, 0, 0)"
    ); // d3-interpolate/test/hsl-test.js:43:3
}

// d3-interpolate/test/hslLong-test.js: interpolateHslLong(a, b) converts a and b to HSL colors
#[test]
fn interpolatehsllong_a_b_converts_a_and_b_to_hsl_colors() {
    assert_eq!(
        interpolate::hsl_long(
            "steelblue".parse::<Hsl>().expect("a CSS color"),
            "brown".parse::<Hsl>().expect("a CSS color")
        )(0.0)
        .to_string(),
        "steelblue".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/hslLong-test.js:6:3
    assert_eq!(
        interpolate::hsl_long(
            "steelblue".parse::<Hsl>().expect("a CSS color"),
            "brown".parse::<Hsl>().expect("a CSS color")
        )(1.0)
        .to_string(),
        "brown".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/hslLong-test.js:7:3
    assert_eq!(
        interpolate::hsl_long(
            "steelblue".parse::<Hsl>().expect("a CSS color"),
            Hsl::from("brown".parse::<Rgb>().expect("a CSS color"))
        )(1.0)
        .to_string(),
        "brown".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/hslLong-test.js:8:3
}

// d3-interpolate/test/hslLong-test.js: interpolateHslLong(a, b) interpolates in HSL and returns an RGB string
#[test]
fn interpolatehsllong_a_b_interpolates_in_hsl_and_returns_an_rgb_string() {
    assert_eq!(
        interpolate::hsl_long(
            "steelblue".parse::<Hsl>().expect("a CSS color"),
            "#f00".parse::<Hsl>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgb(56, 195, 162)"
    ); // d3-interpolate/test/hslLong-test.js:12:3
    assert_eq!(
        interpolate::hsl_long(
            "rgba(70, 130, 180, 1)".parse::<Hsl>().expect("a CSS color"),
            "rgba(255, 0, 0, 0.2)".parse::<Hsl>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgba(56, 195, 162, 0.84)"
    ); // d3-interpolate/test/hslLong-test.js:13:3
}

// d3-interpolate/test/hslLong-test.js: interpolateHslLong(a, b) does not use the shortest path when interpolating hue
#[test]
fn interpolatehsllong_a_b_does_not_use_the_shortest_path_when_interpolating_hue() {
    let i = interpolate::hsl_long(
        "hsl(10,50%,50%)".parse::<Hsl>().expect("a CSS color"),
        "hsl(350,50%,50%)".parse::<Hsl>().expect("a CSS color"),
    );
    assert_eq!(i(0.0).to_string(), "rgb(191, 85, 64)"); // d3-interpolate/test/hslLong-test.js:18:3
    assert_eq!(i(0.2).to_string(), "rgb(153, 191, 64)"); // d3-interpolate/test/hslLong-test.js:19:3
    assert_eq!(i(0.4).to_string(), "rgb(64, 191, 119)"); // d3-interpolate/test/hslLong-test.js:20:3
    assert_eq!(i(0.6).to_string(), "rgb(64, 119, 191)"); // d3-interpolate/test/hslLong-test.js:21:3
    assert_eq!(i(0.8).to_string(), "rgb(153, 64, 191)"); // d3-interpolate/test/hslLong-test.js:22:3
    assert_eq!(i(1.0).to_string(), "rgb(191, 64, 85)"); // d3-interpolate/test/hslLong-test.js:23:3
}

// d3-interpolate/test/hslLong-test.js: interpolateHslLong(a, b) uses a’s hue when b’s hue is undefined
#[test]
fn interpolatehsllong_a_b_uses_a_s_hue_when_b_s_hue_is_undefined() {
    assert_eq!(
        interpolate::hsl_long(
            "#f60".parse::<Hsl>().expect("a CSS color"),
            "#000".parse::<Hsl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(128, 51, 0)"
    ); // d3-interpolate/test/hslLong-test.js:27:3
    assert_eq!(
        interpolate::hsl_long(
            "#6f0".parse::<Hsl>().expect("a CSS color"),
            "#fff".parse::<Hsl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(179, 255, 128)"
    ); // d3-interpolate/test/hslLong-test.js:28:3
}

// d3-interpolate/test/hslLong-test.js: interpolateHslLong(a, b) uses b’s hue when a’s hue is undefined
#[test]
fn interpolatehsllong_a_b_uses_b_s_hue_when_a_s_hue_is_undefined() {
    assert_eq!(
        interpolate::hsl_long(
            "#000".parse::<Hsl>().expect("a CSS color"),
            "#f60".parse::<Hsl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(128, 51, 0)"
    ); // d3-interpolate/test/hslLong-test.js:32:3
    assert_eq!(
        interpolate::hsl_long(
            "#fff".parse::<Hsl>().expect("a CSS color"),
            "#6f0".parse::<Hsl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(179, 255, 128)"
    ); // d3-interpolate/test/hslLong-test.js:33:3
}

// d3-interpolate/test/hslLong-test.js: interpolateHslLong(a, b) uses a’s saturation when b’s saturation is undefined
#[test]
fn interpolatehsllong_a_b_uses_a_s_saturation_when_b_s_saturation_is_undefined() {
    assert_eq!(
        interpolate::hsl_long(
            "#ccc".parse::<Hsl>().expect("a CSS color"),
            "#000".parse::<Hsl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(102, 102, 102)"
    ); // d3-interpolate/test/hslLong-test.js:37:3
    assert_eq!(
        interpolate::hsl_long(
            "#f00".parse::<Hsl>().expect("a CSS color"),
            "#000".parse::<Hsl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(128, 0, 0)"
    ); // d3-interpolate/test/hslLong-test.js:38:3
}

// d3-interpolate/test/hslLong-test.js: interpolateHslLong(a, b) uses b’s saturation when a’s saturation is undefined
#[test]
fn interpolatehsllong_a_b_uses_b_s_saturation_when_a_s_saturation_is_undefined() {
    assert_eq!(
        interpolate::hsl_long(
            "#000".parse::<Hsl>().expect("a CSS color"),
            "#ccc".parse::<Hsl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(102, 102, 102)"
    ); // d3-interpolate/test/hslLong-test.js:42:3
    assert_eq!(
        interpolate::hsl_long(
            "#000".parse::<Hsl>().expect("a CSS color"),
            "#f00".parse::<Hsl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(128, 0, 0)"
    ); // d3-interpolate/test/hslLong-test.js:43:3
}

// d3-interpolate/test/lab-test.js: interpolateLab(a, b) converts a and b to Lab colors
#[test]
fn interpolatelab_a_b_converts_a_and_b_to_lab_colors() {
    assert_eq!(
        interpolate::lab(
            "steelblue".parse::<Lab>().expect("a CSS color"),
            "brown".parse::<Lab>().expect("a CSS color")
        )(0.0)
        .to_string(),
        "steelblue".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/lab-test.js:6:3
    assert_eq!(
        interpolate::lab(
            "steelblue".parse::<Lab>().expect("a CSS color"),
            Lab::from(Rgb::from("brown".parse::<Hsl>().expect("a CSS color")))
        )(1.0)
        .to_string(),
        "brown".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/lab-test.js:7:3
    assert_eq!(
        interpolate::lab(
            "steelblue".parse::<Lab>().expect("a CSS color"),
            Lab::from("brown".parse::<Rgb>().expect("a CSS color"))
        )(1.0)
        .to_string(),
        "brown".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/lab-test.js:8:3
}

// d3-interpolate/test/lab-test.js: interpolateLab(a, b) interpolates in Lab and returns an RGB string
#[test]
fn interpolatelab_a_b_interpolates_in_lab_and_returns_an_rgb_string() {
    assert_eq!(
        interpolate::lab(
            "steelblue".parse::<Lab>().expect("a CSS color"),
            "#f00".parse::<Lab>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgb(134, 120, 146)"
    ); // d3-interpolate/test/lab-test.js:12:3
    assert_eq!(
        interpolate::lab(
            "rgba(70, 130, 180, 1)".parse::<Lab>().expect("a CSS color"),
            "rgba(255, 0, 0, 0.2)".parse::<Lab>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgba(134, 120, 146, 0.84)"
    ); // d3-interpolate/test/lab-test.js:13:3
}

// d3-interpolate/test/lab-test.js: interpolateLab(a, b) uses b’s channel value when a’s channel value is undefined
#[test]
fn interpolatelab_a_b_uses_b_s_channel_value_when_a_s_channel_value_is_undefined() {
    assert_eq!(
        interpolate::lab(Lab::new(f64::NAN, 20.0, 40.0), Lab::new(60.0, 80.0, 100.0))(0.5)
            .to_string(),
        Lab::new(60.0, 50.0, 70.0).to_string()
    ); // d3-interpolate/test/lab-test.js:18:3
    assert_eq!(
        interpolate::lab(Lab::new(20.0, f64::NAN, 40.0), Lab::new(60.0, 80.0, 100.0))(0.5)
            .to_string(),
        Lab::new(40.0, 80.0, 70.0).to_string()
    ); // d3-interpolate/test/lab-test.js:19:3
    assert_eq!(
        interpolate::lab(Lab::new(20.0, 40.0, f64::NAN), Lab::new(60.0, 80.0, 100.0))(0.5)
            .to_string(),
        Lab::new(40.0, 60.0, 100.0).to_string()
    ); // d3-interpolate/test/lab-test.js:20:3
}

// d3-interpolate/test/lab-test.js: interpolateLab(a, b) uses a’s channel value when b’s channel value is undefined
#[test]
fn interpolatelab_a_b_uses_a_s_channel_value_when_b_s_channel_value_is_undefined() {
    assert_eq!(
        interpolate::lab(Lab::new(60.0, 80.0, 100.0), Lab::new(f64::NAN, 20.0, 40.0))(0.5)
            .to_string(),
        Lab::new(60.0, 50.0, 70.0).to_string()
    ); // d3-interpolate/test/lab-test.js:25:3
    assert_eq!(
        interpolate::lab(Lab::new(60.0, 80.0, 100.0), Lab::new(20.0, f64::NAN, 40.0))(0.5)
            .to_string(),
        Lab::new(40.0, 80.0, 70.0).to_string()
    ); // d3-interpolate/test/lab-test.js:26:3
    assert_eq!(
        interpolate::lab(Lab::new(60.0, 80.0, 100.0), Lab::new(20.0, 40.0, f64::NAN))(0.5)
            .to_string(),
        Lab::new(40.0, 60.0, 100.0).to_string()
    ); // d3-interpolate/test/lab-test.js:27:3
}

// d3-interpolate/test/hcl-test.js: interpolateHcl(a, b) converts a and b to HCL colors
#[test]
fn interpolatehcl_a_b_converts_a_and_b_to_hcl_colors() {
    assert_eq!(
        interpolate::hcl(
            "steelblue".parse::<Hcl>().expect("a CSS color"),
            "brown".parse::<Hcl>().expect("a CSS color")
        )(0.0)
        .to_string(),
        "steelblue".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/hcl-test.js:6:3
    assert_eq!(
        interpolate::hcl(
            "steelblue".parse::<Hcl>().expect("a CSS color"),
            "brown".parse::<Hcl>().expect("a CSS color")
        )(1.0)
        .to_string(),
        "brown".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/hcl-test.js:7:3
    assert_eq!(
        interpolate::hcl(
            "steelblue".parse::<Hcl>().expect("a CSS color"),
            Hcl::from("brown".parse::<Rgb>().expect("a CSS color"))
        )(1.0)
        .to_string(),
        "brown".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/hcl-test.js:8:3
}

// d3-interpolate/test/hcl-test.js: interpolateHcl(a, b) interpolates in HCL and returns an RGB string
#[test]
fn interpolatehcl_a_b_interpolates_in_hcl_and_returns_an_rgb_string() {
    assert_eq!(
        interpolate::hcl(
            "steelblue".parse::<Hcl>().expect("a CSS color"),
            "#f00".parse::<Hcl>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgb(106, 121, 206)"
    ); // d3-interpolate/test/hcl-test.js:12:3
    assert_eq!(
        interpolate::hcl(
            "rgba(70, 130, 180, 1)".parse::<Hcl>().expect("a CSS color"),
            "rgba(255, 0, 0, 0.2)".parse::<Hcl>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgba(106, 121, 206, 0.84)"
    ); // d3-interpolate/test/hcl-test.js:13:3
}

// d3-interpolate/test/hcl-test.js: interpolateHcl(a, b) uses the shortest path when interpolating hue difference greater than 180°
#[test]
fn interpolatehcl_a_b_uses_the_shortest_path_when_interpolating_hue_difference_greater_than_180() {
    let i = interpolate::hcl(Hcl::new(10.0, 50.0, 50.0), Hcl::new(350.0, 50.0, 50.0));
    assert_eq!(i(0.0).to_string(), "rgb(194, 78, 107)"); // d3-interpolate/test/hcl-test.js:18:3
    assert_eq!(i(0.2).to_string(), "rgb(194, 78, 113)"); // d3-interpolate/test/hcl-test.js:19:3
    assert_eq!(i(0.4).to_string(), "rgb(193, 78, 118)"); // d3-interpolate/test/hcl-test.js:20:3
    assert_eq!(i(0.6).to_string(), "rgb(192, 78, 124)"); // d3-interpolate/test/hcl-test.js:21:3
    assert_eq!(i(0.8).to_string(), "rgb(191, 78, 130)"); // d3-interpolate/test/hcl-test.js:22:3
    assert_eq!(i(1.0).to_string(), "rgb(189, 79, 136)"); // d3-interpolate/test/hcl-test.js:23:3
}

// d3-interpolate/test/hcl-test.js: interpolateHcl(a, b) uses the shortest path when interpolating hue difference greater than 360°
#[test]
fn interpolatehcl_a_b_uses_the_shortest_path_when_interpolating_hue_difference_greater_than_360() {
    let i = interpolate::hcl(Hcl::new(10.0, 50.0, 50.0), Hcl::new(380.0, 50.0, 50.0));
    assert_eq!(i(0.0).to_string(), "rgb(194, 78, 107)"); // d3-interpolate/test/hcl-test.js:28:3
    assert_eq!(i(0.2).to_string(), "rgb(194, 78, 104)"); // d3-interpolate/test/hcl-test.js:29:3
    assert_eq!(i(0.4).to_string(), "rgb(194, 79, 101)"); // d3-interpolate/test/hcl-test.js:30:3
    assert_eq!(i(0.6).to_string(), "rgb(194, 79, 98)"); // d3-interpolate/test/hcl-test.js:31:3
    assert_eq!(i(0.8).to_string(), "rgb(194, 80, 96)"); // d3-interpolate/test/hcl-test.js:32:3
    assert_eq!(i(1.0).to_string(), "rgb(194, 80, 93)"); // d3-interpolate/test/hcl-test.js:33:3
}

// d3-interpolate/test/hcl-test.js: interpolateHcl(a, b) uses the shortest path when interpolating hue difference greater than 540°
#[test]
fn interpolatehcl_a_b_uses_the_shortest_path_when_interpolating_hue_difference_greater_than_540() {
    let i = interpolate::hcl(Hcl::new(10.0, 50.0, 50.0), Hcl::new(710.0, 50.0, 50.0));
    assert_eq!(i(0.0).to_string(), "rgb(194, 78, 107)"); // d3-interpolate/test/hcl-test.js:38:3
    assert_eq!(i(0.2).to_string(), "rgb(194, 78, 113)"); // d3-interpolate/test/hcl-test.js:39:3
    assert_eq!(i(0.4).to_string(), "rgb(193, 78, 118)"); // d3-interpolate/test/hcl-test.js:40:3
    assert_eq!(i(0.6).to_string(), "rgb(192, 78, 124)"); // d3-interpolate/test/hcl-test.js:41:3
    assert_eq!(i(0.8).to_string(), "rgb(191, 78, 130)"); // d3-interpolate/test/hcl-test.js:42:3
    assert_eq!(i(1.0).to_string(), "rgb(189, 79, 136)"); // d3-interpolate/test/hcl-test.js:43:3
}

// d3-interpolate/test/hcl-test.js: interpolateHcl(a, b) uses the shortest path when interpolating hue difference greater than 720°
#[test]
fn interpolatehcl_a_b_uses_the_shortest_path_when_interpolating_hue_difference_greater_than_720() {
    let i = interpolate::hcl(Hcl::new(10.0, 50.0, 50.0), Hcl::new(740.0, 50.0, 50.0));
    assert_eq!(i(0.0).to_string(), "rgb(194, 78, 107)"); // d3-interpolate/test/hcl-test.js:48:3
    assert_eq!(i(0.2).to_string(), "rgb(194, 78, 104)"); // d3-interpolate/test/hcl-test.js:49:3
    assert_eq!(i(0.4).to_string(), "rgb(194, 79, 101)"); // d3-interpolate/test/hcl-test.js:50:3
    assert_eq!(i(0.6).to_string(), "rgb(194, 79, 98)"); // d3-interpolate/test/hcl-test.js:51:3
    assert_eq!(i(0.8).to_string(), "rgb(194, 80, 96)"); // d3-interpolate/test/hcl-test.js:52:3
    assert_eq!(i(1.0).to_string(), "rgb(194, 80, 93)"); // d3-interpolate/test/hcl-test.js:53:3
}

// d3-interpolate/test/hcl-test.js: interpolateHcl(a, b) uses a’s hue when b’s hue is undefined
#[test]
fn interpolatehcl_a_b_uses_a_s_hue_when_b_s_hue_is_undefined() {
    assert_eq!(
        interpolate::hcl(
            "#f60".parse::<Hcl>().expect("a CSS color"),
            Hcl::new(f64::NAN, f64::NAN, 0.0)
        )(0.5)
        .to_string(),
        "rgb(155, 0, 0)"
    ); // d3-interpolate/test/hcl-test.js:57:3
    assert_eq!(
        interpolate::hcl(
            "#6f0".parse::<Hcl>().expect("a CSS color"),
            Hcl::new(f64::NAN, f64::NAN, 0.0)
        )(0.5)
        .to_string(),
        "rgb(0, 129, 0)"
    ); // d3-interpolate/test/hcl-test.js:58:3
}

// d3-interpolate/test/hcl-test.js: interpolateHcl(a, b) uses b’s hue when a’s hue is undefined
#[test]
fn interpolatehcl_a_b_uses_b_s_hue_when_a_s_hue_is_undefined() {
    assert_eq!(
        interpolate::hcl(
            Hcl::new(f64::NAN, f64::NAN, 0.0),
            "#f60".parse::<Hcl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(155, 0, 0)"
    ); // d3-interpolate/test/hcl-test.js:62:3
    assert_eq!(
        interpolate::hcl(
            Hcl::new(f64::NAN, f64::NAN, 0.0),
            "#6f0".parse::<Hcl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(0, 129, 0)"
    ); // d3-interpolate/test/hcl-test.js:63:3
}

// d3-interpolate/test/hcl-test.js: interpolateHcl(a, b) uses a’s chroma when b’s chroma is undefined
#[test]
fn interpolatehcl_a_b_uses_a_s_chroma_when_b_s_chroma_is_undefined() {
    assert_eq!(
        interpolate::hcl(
            "#ccc".parse::<Hcl>().expect("a CSS color"),
            Hcl::new(f64::NAN, f64::NAN, 0.0)
        )(0.5)
        .to_string(),
        "rgb(97, 97, 97)"
    ); // d3-interpolate/test/hcl-test.js:67:3
    assert_eq!(
        interpolate::hcl(
            "#f00".parse::<Hcl>().expect("a CSS color"),
            Hcl::new(f64::NAN, f64::NAN, 0.0)
        )(0.5)
        .to_string(),
        "rgb(166, 0, 0)"
    ); // d3-interpolate/test/hcl-test.js:68:3
}

// d3-interpolate/test/hcl-test.js: interpolateHcl(a, b) uses b’s chroma when a’s chroma is undefined
#[test]
fn interpolatehcl_a_b_uses_b_s_chroma_when_a_s_chroma_is_undefined() {
    assert_eq!(
        interpolate::hcl(
            Hcl::new(f64::NAN, f64::NAN, 0.0),
            "#ccc".parse::<Hcl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(97, 97, 97)"
    ); // d3-interpolate/test/hcl-test.js:72:3
    assert_eq!(
        interpolate::hcl(
            Hcl::new(f64::NAN, f64::NAN, 0.0),
            "#f00".parse::<Hcl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(166, 0, 0)"
    ); // d3-interpolate/test/hcl-test.js:73:3
}

// d3-interpolate/test/hclLong-test.js: interpolateHclLong(a, b) converts a and b to HCL colors
#[test]
fn interpolatehcllong_a_b_converts_a_and_b_to_hcl_colors() {
    assert_eq!(
        interpolate::hcl_long(
            "steelblue".parse::<Hcl>().expect("a CSS color"),
            "brown".parse::<Hcl>().expect("a CSS color")
        )(0.0)
        .to_string(),
        "steelblue".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/hclLong-test.js:6:3
    assert_eq!(
        interpolate::hcl_long(
            "steelblue".parse::<Hcl>().expect("a CSS color"),
            "brown".parse::<Hcl>().expect("a CSS color")
        )(1.0)
        .to_string(),
        "brown".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/hclLong-test.js:7:3
    assert_eq!(
        interpolate::hcl_long(
            "steelblue".parse::<Hcl>().expect("a CSS color"),
            Hcl::from("brown".parse::<Rgb>().expect("a CSS color"))
        )(1.0)
        .to_string(),
        "brown".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/hclLong-test.js:8:3
}

// d3-interpolate/test/hclLong-test.js: interpolateHclLong(a, b) interpolates in HCL and returns an RGB string
#[test]
fn interpolatehcllong_a_b_interpolates_in_hcl_and_returns_an_rgb_string() {
    assert_eq!(
        interpolate::hcl_long(
            "steelblue".parse::<Hcl>().expect("a CSS color"),
            "#f00".parse::<Hcl>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgb(0, 144, 169)"
    ); // d3-interpolate/test/hclLong-test.js:12:3
    assert_eq!(
        interpolate::hcl_long(
            "rgba(70, 130, 180, 1)".parse::<Hcl>().expect("a CSS color"),
            "rgba(255, 0, 0, 0.2)".parse::<Hcl>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgba(0, 144, 169, 0.84)"
    ); // d3-interpolate/test/hclLong-test.js:13:3
}

// d3-interpolate/test/hclLong-test.js: interpolateHclLong(a, b) does not use the shortest path when interpolating hue
#[test]
fn interpolatehcllong_a_b_does_not_use_the_shortest_path_when_interpolating_hue() {
    let i = interpolate::hcl_long(Hcl::new(10.0, 50.0, 50.0), Hcl::new(350.0, 50.0, 50.0));
    assert_eq!(i(0.0).to_string(), "rgb(194, 78, 107)"); // d3-interpolate/test/hclLong-test.js:18:3
    assert_eq!(i(0.2).to_string(), "rgb(151, 111, 28)"); // d3-interpolate/test/hclLong-test.js:19:3
    assert_eq!(i(0.4).to_string(), "rgb(35, 136, 68)"); // d3-interpolate/test/hclLong-test.js:20:3
    assert_eq!(i(0.6).to_string(), "rgb(0, 138, 165)"); // d3-interpolate/test/hclLong-test.js:21:3
    assert_eq!(i(0.8).to_string(), "rgb(91, 116, 203)"); // d3-interpolate/test/hclLong-test.js:22:3
    assert_eq!(i(1.0).to_string(), "rgb(189, 79, 136)"); // d3-interpolate/test/hclLong-test.js:23:3
}

// d3-interpolate/test/hclLong-test.js: interpolateHclLong(a, b) uses a’s hue when b’s hue is undefined
#[test]
fn interpolatehcllong_a_b_uses_a_s_hue_when_b_s_hue_is_undefined() {
    assert_eq!(
        interpolate::hcl_long(
            "#f60".parse::<Hcl>().expect("a CSS color"),
            Hcl::new(f64::NAN, f64::NAN, 0.0)
        )(0.5)
        .to_string(),
        "rgb(155, 0, 0)"
    ); // d3-interpolate/test/hclLong-test.js:27:3
    assert_eq!(
        interpolate::hcl_long(
            "#6f0".parse::<Hcl>().expect("a CSS color"),
            Hcl::new(f64::NAN, f64::NAN, 0.0)
        )(0.5)
        .to_string(),
        "rgb(0, 129, 0)"
    ); // d3-interpolate/test/hclLong-test.js:28:3
}

// d3-interpolate/test/hclLong-test.js: interpolateHclLong(a, b) uses b’s hue when a’s hue is undefined
#[test]
fn interpolatehcllong_a_b_uses_b_s_hue_when_a_s_hue_is_undefined() {
    assert_eq!(
        interpolate::hcl_long(
            Hcl::new(f64::NAN, f64::NAN, 0.0),
            "#f60".parse::<Hcl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(155, 0, 0)"
    ); // d3-interpolate/test/hclLong-test.js:32:3
    assert_eq!(
        interpolate::hcl_long(
            Hcl::new(f64::NAN, f64::NAN, 0.0),
            "#6f0".parse::<Hcl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(0, 129, 0)"
    ); // d3-interpolate/test/hclLong-test.js:33:3
}

// d3-interpolate/test/hclLong-test.js: interpolateHclLong(a, b) uses a’s chroma when b’s chroma is undefined
#[test]
fn interpolatehcllong_a_b_uses_a_s_chroma_when_b_s_chroma_is_undefined() {
    assert_eq!(
        interpolate::hcl_long(
            "#ccc".parse::<Hcl>().expect("a CSS color"),
            Hcl::new(f64::NAN, f64::NAN, 0.0)
        )(0.5)
        .to_string(),
        "rgb(97, 97, 97)"
    ); // d3-interpolate/test/hclLong-test.js:37:3
    assert_eq!(
        interpolate::hcl_long(
            "#f00".parse::<Hcl>().expect("a CSS color"),
            Hcl::new(f64::NAN, f64::NAN, 0.0)
        )(0.5)
        .to_string(),
        "rgb(166, 0, 0)"
    ); // d3-interpolate/test/hclLong-test.js:38:3
}

// d3-interpolate/test/hclLong-test.js: interpolateHclLong(a, b) uses b’s chroma when a’s chroma is undefined
#[test]
fn interpolatehcllong_a_b_uses_b_s_chroma_when_a_s_chroma_is_undefined() {
    assert_eq!(
        interpolate::hcl_long(
            Hcl::new(f64::NAN, f64::NAN, 0.0),
            "#ccc".parse::<Hcl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(97, 97, 97)"
    ); // d3-interpolate/test/hclLong-test.js:42:3
    assert_eq!(
        interpolate::hcl_long(
            Hcl::new(f64::NAN, f64::NAN, 0.0),
            "#f00".parse::<Hcl>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(166, 0, 0)"
    ); // d3-interpolate/test/hclLong-test.js:43:3
}

// d3-interpolate/test/cubehelix-test.js: interpolateCubehelix(a, b) converts a and b to Cubehelix colors
#[test]
fn interpolatecubehelix_a_b_converts_a_and_b_to_cubehelix_colors() {
    assert_eq!(
        interpolate::cubehelix(
            1.0,
            "steelblue".parse::<Cubehelix>().expect("a CSS color"),
            "brown".parse::<Cubehelix>().expect("a CSS color")
        )(0.0)
        .to_string(),
        "steelblue".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/cubehelix-test.js:6:3
    assert_eq!(
        interpolate::cubehelix(
            1.0,
            "steelblue".parse::<Cubehelix>().expect("a CSS color"),
            Cubehelix::from(Rgb::from("brown".parse::<Hcl>().expect("a CSS color")))
        )(1.0)
        .to_string(),
        "brown".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/cubehelix-test.js:7:3
    assert_eq!(
        interpolate::cubehelix(
            1.0,
            "steelblue".parse::<Cubehelix>().expect("a CSS color"),
            Cubehelix::from("brown".parse::<Rgb>().expect("a CSS color"))
        )(1.0)
        .to_string(),
        "brown".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/cubehelix-test.js:8:3
}

// d3-interpolate/test/cubehelix-test.js: interpolateCubehelix(a, b) interpolates in Cubehelix and returns an RGB string
#[test]
fn interpolatecubehelix_a_b_interpolates_in_cubehelix_and_returns_an_rgb_string() {
    assert_eq!(
        interpolate::cubehelix(
            1.0,
            "steelblue".parse::<Cubehelix>().expect("a CSS color"),
            "#f00".parse::<Cubehelix>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgb(88, 100, 218)"
    ); // d3-interpolate/test/cubehelix-test.js:12:3
    assert_eq!(
        interpolate::cubehelix(
            1.0,
            "rgba(70, 130, 180, 1)"
                .parse::<Cubehelix>()
                .expect("a CSS color"),
            "rgba(255, 0, 0, 0.2)"
                .parse::<Cubehelix>()
                .expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgba(88, 100, 218, 0.84)"
    ); // d3-interpolate/test/cubehelix-test.js:13:3
}

// d3-interpolate/test/cubehelix-test.js: interpolateCubehelix.gamma(3)(a, b) returns the expected values
#[test]
fn interpolatecubehelix_gamma_3_a_b_returns_the_expected_values() {
    assert_eq!(
        interpolate::cubehelix(
            3.0,
            "steelblue".parse::<Cubehelix>().expect("a CSS color"),
            "#f00".parse::<Cubehelix>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgb(96, 107, 228)"
    ); // d3-interpolate/test/cubehelix-test.js:17:3
}

// d3-interpolate/test/cubehelix-test.js: interpolateCubehelix(a, b) is equivalent to interpolateCubehelix.gamma(1)(a, b)
#[test]
fn interpolatecubehelix_a_b_is_equivalent_to_interpolatecubehelix_gamma_1_a_b() {
    let i0 = interpolate::cubehelix(
        1.0,
        "purple".parse::<Cubehelix>().expect("a CSS color"),
        "orange".parse::<Cubehelix>().expect("a CSS color"),
    );
    let i1 = interpolate::cubehelix(
        1.0,
        "purple".parse::<Cubehelix>().expect("a CSS color"),
        "orange".parse::<Cubehelix>().expect("a CSS color"),
    );
    assert_eq!(i1(0.0).to_string(), i0(0.0).to_string()); // d3-interpolate/test/cubehelix-test.js:27:3
    assert_eq!(i1(0.2).to_string(), i0(0.2).to_string()); // d3-interpolate/test/cubehelix-test.js:28:3
    assert_eq!(i1(0.4).to_string(), i0(0.4).to_string()); // d3-interpolate/test/cubehelix-test.js:29:3
    assert_eq!(i1(0.6).to_string(), i0(0.6).to_string()); // d3-interpolate/test/cubehelix-test.js:30:3
    assert_eq!(i1(0.8).to_string(), i0(0.8).to_string()); // d3-interpolate/test/cubehelix-test.js:31:3
    assert_eq!(i1(1.0).to_string(), i0(1.0).to_string()); // d3-interpolate/test/cubehelix-test.js:32:3
}

// d3-interpolate/test/cubehelix-test.js: interpolateCubehelix(a, b) uses the shortest path when interpolating hue difference greater than 180°
#[test]
fn interpolatecubehelix_a_b_uses_the_shortest_path_when_interpolating_hue_difference_greater_than_180()
 {
    let i = interpolate::cubehelix(
        1.0,
        "purple".parse::<Cubehelix>().expect("a CSS color"),
        "orange".parse::<Cubehelix>().expect("a CSS color"),
    );
    assert_eq!(i(0.0).to_string(), "rgb(128, 0, 128)"); // d3-interpolate/test/cubehelix-test.js:37:3
    assert_eq!(i(0.2).to_string(), "rgb(208, 1, 127)"); // d3-interpolate/test/cubehelix-test.js:38:3
    assert_eq!(i(0.4).to_string(), "rgb(255, 17, 93)"); // d3-interpolate/test/cubehelix-test.js:39:3
    assert_eq!(i(0.6).to_string(), "rgb(255, 52, 43)"); // d3-interpolate/test/cubehelix-test.js:40:3
    assert_eq!(i(0.8).to_string(), "rgb(255, 105, 5)"); // d3-interpolate/test/cubehelix-test.js:41:3
    assert_eq!(i(1.0).to_string(), "rgb(255, 165, 0)"); // d3-interpolate/test/cubehelix-test.js:42:3
}

// d3-interpolate/test/cubehelix-test.js: interpolateCubehelix(a, b) uses a’s hue when b’s hue is undefined
#[test]
fn interpolatecubehelix_a_b_uses_a_s_hue_when_b_s_hue_is_undefined() {
    assert_eq!(
        interpolate::cubehelix(
            1.0,
            "#f60".parse::<Cubehelix>().expect("a CSS color"),
            Cubehelix::new(f64::NAN, f64::NAN, 0.0)
        )(0.5)
        .to_string(),
        "rgb(162, 41, 0)"
    ); // d3-interpolate/test/cubehelix-test.js:46:3
    assert_eq!(
        interpolate::cubehelix(
            1.0,
            "#6f0".parse::<Cubehelix>().expect("a CSS color"),
            Cubehelix::new(f64::NAN, f64::NAN, 0.0)
        )(0.5)
        .to_string(),
        "rgb(3, 173, 0)"
    ); // d3-interpolate/test/cubehelix-test.js:47:3
}

// d3-interpolate/test/cubehelix-test.js: interpolateCubehelix(a, b) uses b’s hue when a’s hue is undefined
#[test]
fn interpolatecubehelix_a_b_uses_b_s_hue_when_a_s_hue_is_undefined() {
    assert_eq!(
        interpolate::cubehelix(
            1.0,
            Cubehelix::new(f64::NAN, f64::NAN, 0.0),
            "#f60".parse::<Cubehelix>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(162, 41, 0)"
    ); // d3-interpolate/test/cubehelix-test.js:51:3
    assert_eq!(
        interpolate::cubehelix(
            1.0,
            Cubehelix::new(f64::NAN, f64::NAN, 0.0),
            "#6f0".parse::<Cubehelix>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(3, 173, 0)"
    ); // d3-interpolate/test/cubehelix-test.js:52:3
}

// d3-interpolate/test/cubehelix-test.js: interpolateCubehelix(a, b) uses a’s chroma when b’s chroma is undefined
#[test]
fn interpolatecubehelix_a_b_uses_a_s_chroma_when_b_s_chroma_is_undefined() {
    assert_eq!(
        interpolate::cubehelix(
            1.0,
            "#ccc".parse::<Cubehelix>().expect("a CSS color"),
            Cubehelix::new(f64::NAN, f64::NAN, 0.0)
        )(0.5)
        .to_string(),
        "rgb(102, 102, 102)"
    ); // d3-interpolate/test/cubehelix-test.js:56:3
    assert_eq!(
        interpolate::cubehelix(
            1.0,
            "#f00".parse::<Cubehelix>().expect("a CSS color"),
            Cubehelix::new(f64::NAN, f64::NAN, 0.0)
        )(0.5)
        .to_string(),
        "rgb(147, 0, 0)"
    ); // d3-interpolate/test/cubehelix-test.js:57:3
}

// d3-interpolate/test/cubehelix-test.js: interpolateCubehelix(a, b) uses b’s chroma when a’s chroma is undefined
#[test]
fn interpolatecubehelix_a_b_uses_b_s_chroma_when_a_s_chroma_is_undefined() {
    assert_eq!(
        interpolate::cubehelix(
            1.0,
            Cubehelix::new(f64::NAN, f64::NAN, 0.0),
            "#ccc".parse::<Cubehelix>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(102, 102, 102)"
    ); // d3-interpolate/test/cubehelix-test.js:61:3
    assert_eq!(
        interpolate::cubehelix(
            1.0,
            Cubehelix::new(f64::NAN, f64::NAN, 0.0),
            "#f00".parse::<Cubehelix>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(147, 0, 0)"
    ); // d3-interpolate/test/cubehelix-test.js:62:3
}

// d3-interpolate/test/cubehelixLong-test.js: interpolateCubehelixLong(a, b) converts a and b to Cubehelix colors
#[test]
fn interpolatecubehelixlong_a_b_converts_a_and_b_to_cubehelix_colors() {
    assert_eq!(
        interpolate::cubehelix_long(
            1.0,
            "steelblue".parse::<Cubehelix>().expect("a CSS color"),
            "brown".parse::<Cubehelix>().expect("a CSS color")
        )(0.0)
        .to_string(),
        "steelblue".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/cubehelixLong-test.js:6:3
    assert_eq!(
        interpolate::cubehelix_long(
            1.0,
            "steelblue".parse::<Cubehelix>().expect("a CSS color"),
            Cubehelix::from(Rgb::from("brown".parse::<Hcl>().expect("a CSS color")))
        )(1.0)
        .to_string(),
        "brown".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/cubehelixLong-test.js:7:3
    assert_eq!(
        interpolate::cubehelix_long(
            1.0,
            "steelblue".parse::<Cubehelix>().expect("a CSS color"),
            Cubehelix::from("brown".parse::<Rgb>().expect("a CSS color"))
        )(1.0)
        .to_string(),
        "brown".parse::<Rgb>().expect("a CSS color").to_string()
    ); // d3-interpolate/test/cubehelixLong-test.js:8:3
}

// d3-interpolate/test/cubehelixLong-test.js: interpolateCubehelixLong(a, b) interpolates in Cubehelix and returns an RGB string
#[test]
fn interpolatecubehelixlong_a_b_interpolates_in_cubehelix_and_returns_an_rgb_string() {
    assert_eq!(
        interpolate::cubehelix_long(
            1.0,
            "steelblue".parse::<Cubehelix>().expect("a CSS color"),
            "#f00".parse::<Cubehelix>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgb(88, 100, 218)"
    ); // d3-interpolate/test/cubehelixLong-test.js:12:3
    assert_eq!(
        interpolate::cubehelix_long(
            1.0,
            "rgba(70, 130, 180, 1)"
                .parse::<Cubehelix>()
                .expect("a CSS color"),
            "rgba(255, 0, 0, 0.2)"
                .parse::<Cubehelix>()
                .expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgba(88, 100, 218, 0.84)"
    ); // d3-interpolate/test/cubehelixLong-test.js:13:3
}

// d3-interpolate/test/cubehelixLong-test.js: interpolateCubehelixLong.gamma(3)(a, b) returns the expected values
#[test]
fn interpolatecubehelixlong_gamma_3_a_b_returns_the_expected_values() {
    assert_eq!(
        interpolate::cubehelix_long(
            3.0,
            "steelblue".parse::<Cubehelix>().expect("a CSS color"),
            "#f00".parse::<Cubehelix>().expect("a CSS color")
        )(0.2)
        .to_string(),
        "rgb(96, 107, 228)"
    ); // d3-interpolate/test/cubehelixLong-test.js:17:3
}

// d3-interpolate/test/cubehelixLong-test.js: interpolateCubehelixLong(a, b) is equivalent to interpolateCubehelixLong.gamma(1)(a, b)
#[test]
fn interpolatecubehelixlong_a_b_is_equivalent_to_interpolatecubehelixlong_gamma_1_a_b() {
    let i0 = interpolate::cubehelix_long(
        1.0,
        "purple".parse::<Cubehelix>().expect("a CSS color"),
        "orange".parse::<Cubehelix>().expect("a CSS color"),
    );
    let i1 = interpolate::cubehelix_long(
        1.0,
        "purple".parse::<Cubehelix>().expect("a CSS color"),
        "orange".parse::<Cubehelix>().expect("a CSS color"),
    );
    assert_eq!(i1(0.0).to_string(), i0(0.0).to_string()); // d3-interpolate/test/cubehelixLong-test.js:27:3
    assert_eq!(i1(0.2).to_string(), i0(0.2).to_string()); // d3-interpolate/test/cubehelixLong-test.js:28:3
    assert_eq!(i1(0.4).to_string(), i0(0.4).to_string()); // d3-interpolate/test/cubehelixLong-test.js:29:3
    assert_eq!(i1(0.6).to_string(), i0(0.6).to_string()); // d3-interpolate/test/cubehelixLong-test.js:30:3
    assert_eq!(i1(0.8).to_string(), i0(0.8).to_string()); // d3-interpolate/test/cubehelixLong-test.js:31:3
    assert_eq!(i1(1.0).to_string(), i0(1.0).to_string()); // d3-interpolate/test/cubehelixLong-test.js:32:3
}

// d3-interpolate/test/cubehelixLong-test.js: interpolateCubehelixLong(a, b) uses the longest path when interpolating hue difference greater than 180°
#[test]
fn interpolatecubehelixlong_a_b_uses_the_longest_path_when_interpolating_hue_difference_greater_than_180()
 {
    let i = interpolate::cubehelix_long(
        1.0,
        "purple".parse::<Cubehelix>().expect("a CSS color"),
        "orange".parse::<Cubehelix>().expect("a CSS color"),
    );
    assert_eq!(i(0.0).to_string(), "rgb(128, 0, 128)"); // d3-interpolate/test/cubehelixLong-test.js:36:3
    assert_eq!(i(0.2).to_string(), "rgb(63, 54, 234)"); // d3-interpolate/test/cubehelixLong-test.js:37:3
    assert_eq!(i(0.4).to_string(), "rgb(0, 151, 217)"); // d3-interpolate/test/cubehelixLong-test.js:38:3
    assert_eq!(i(0.6).to_string(), "rgb(0, 223, 83)"); // d3-interpolate/test/cubehelixLong-test.js:39:3
    assert_eq!(i(0.8).to_string(), "rgb(79, 219, 0)"); // d3-interpolate/test/cubehelixLong-test.js:40:3
    assert_eq!(i(1.0).to_string(), "rgb(255, 165, 0)"); // d3-interpolate/test/cubehelixLong-test.js:41:3
}

// d3-interpolate/test/cubehelixLong-test.js: interpolateCubehelixLong(a, b) uses a’s hue when b’s hue is undefined
#[test]
fn interpolatecubehelixlong_a_b_uses_a_s_hue_when_b_s_hue_is_undefined() {
    assert_eq!(
        interpolate::cubehelix_long(
            1.0,
            "#f60".parse::<Cubehelix>().expect("a CSS color"),
            Cubehelix::from(Rgb::from(Hcl::new(f64::NAN, f64::NAN, 0.0)))
        )(0.5)
        .to_string(),
        "rgb(162, 41, 0)"
    ); // d3-interpolate/test/cubehelixLong-test.js:45:3
    assert_eq!(
        interpolate::cubehelix_long(
            1.0,
            "#6f0".parse::<Cubehelix>().expect("a CSS color"),
            Cubehelix::from(Rgb::from(Hcl::new(f64::NAN, f64::NAN, 0.0)))
        )(0.5)
        .to_string(),
        "rgb(3, 173, 0)"
    ); // d3-interpolate/test/cubehelixLong-test.js:46:3
}

// d3-interpolate/test/cubehelixLong-test.js: interpolateCubehelixLong(a, b) uses b’s hue when a’s hue is undefined
#[test]
fn interpolatecubehelixlong_a_b_uses_b_s_hue_when_a_s_hue_is_undefined() {
    assert_eq!(
        interpolate::cubehelix_long(
            1.0,
            Cubehelix::from(Rgb::from(Hcl::new(f64::NAN, f64::NAN, 0.0))),
            "#f60".parse::<Cubehelix>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(162, 41, 0)"
    ); // d3-interpolate/test/cubehelixLong-test.js:50:3
    assert_eq!(
        interpolate::cubehelix_long(
            1.0,
            Cubehelix::from(Rgb::from(Hcl::new(f64::NAN, f64::NAN, 0.0))),
            "#6f0".parse::<Cubehelix>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(3, 173, 0)"
    ); // d3-interpolate/test/cubehelixLong-test.js:51:3
}

// d3-interpolate/test/cubehelixLong-test.js: interpolateCubehelixLong(a, b) uses a’s chroma when b’s chroma is undefined
#[test]
fn interpolatecubehelixlong_a_b_uses_a_s_chroma_when_b_s_chroma_is_undefined() {
    assert_eq!(
        interpolate::cubehelix_long(
            1.0,
            "#ccc".parse::<Cubehelix>().expect("a CSS color"),
            Cubehelix::from(Rgb::from(Hcl::new(f64::NAN, f64::NAN, 0.0)))
        )(0.5)
        .to_string(),
        "rgb(102, 102, 102)"
    ); // d3-interpolate/test/cubehelixLong-test.js:55:3
    assert_eq!(
        interpolate::cubehelix_long(
            1.0,
            "#f00".parse::<Cubehelix>().expect("a CSS color"),
            Cubehelix::from(Rgb::from(Hcl::new(f64::NAN, f64::NAN, 0.0)))
        )(0.5)
        .to_string(),
        "rgb(147, 0, 0)"
    ); // d3-interpolate/test/cubehelixLong-test.js:56:3
}

// d3-interpolate/test/cubehelixLong-test.js: interpolateCubehelixLong(a, b) uses b’s chroma when a’s chroma is undefined
#[test]
fn interpolatecubehelixlong_a_b_uses_b_s_chroma_when_a_s_chroma_is_undefined() {
    assert_eq!(
        interpolate::cubehelix_long(
            1.0,
            Cubehelix::from(Rgb::from(Hcl::new(f64::NAN, f64::NAN, 0.0))),
            "#ccc".parse::<Cubehelix>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(102, 102, 102)"
    ); // d3-interpolate/test/cubehelixLong-test.js:60:3
    assert_eq!(
        interpolate::cubehelix_long(
            1.0,
            Cubehelix::from(Rgb::from(Hcl::new(f64::NAN, f64::NAN, 0.0))),
            "#f00".parse::<Cubehelix>().expect("a CSS color")
        )(0.5)
        .to_string(),
        "rgb(147, 0, 0)"
    ); // d3-interpolate/test/cubehelixLong-test.js:61:3
}

// d3-interpolate/test/hue-test.js: interpolateHue(a, b) returns a if b is NaN
#[test]
fn interpolatehue_a_b_returns_a_if_b_is_nan() {
    let i = interpolate::hue(10.0, f64::NAN);
    assert_eq!(i(0.0), 10.0); // d3-interpolate/test/hue-test.js:16:3
    assert_eq!(i(0.5), 10.0); // d3-interpolate/test/hue-test.js:17:3
    assert_eq!(i(1.0), 10.0); // d3-interpolate/test/hue-test.js:18:3
}

// d3-interpolate/test/hue-test.js: interpolateHue(a, b) returns b if a is NaN
#[test]
fn interpolatehue_a_b_returns_b_if_a_is_nan() {
    let i = interpolate::hue(f64::NAN, 20.0);
    assert_eq!(i(0.0), 20.0); // d3-interpolate/test/hue-test.js:23:3
    assert_eq!(i(0.5), 20.0); // d3-interpolate/test/hue-test.js:24:3
    assert_eq!(i(1.0), 20.0); // d3-interpolate/test/hue-test.js:25:3
}

// d3-interpolate/test/hue-test.js: interpolateHue(a, b) returns NaN if both a and b are NaN
#[test]
fn interpolatehue_a_b_returns_nan_if_both_a_and_b_are_nan() {
    let i = interpolate::hue(f64::NAN, f64::NAN);
    assert!((i(0.0)).is_nan()); // d3-interpolate/test/hue-test.js:30:3
    assert!((i(0.5)).is_nan()); // d3-interpolate/test/hue-test.js:31:3
    assert!((i(1.0)).is_nan()); // d3-interpolate/test/hue-test.js:32:3
}

// d3-interpolate/test/hue-test.js: interpolateHue(a, b) uses the shortest path
#[test]
fn interpolatehue_a_b_uses_the_shortest_path() {
    let i = interpolate::hue(10.0, 350.0);
    assert_eq!(i(0.0), 10.0); // d3-interpolate/test/hue-test.js:37:3
    assert_eq!(i(0.2), 6.0); // d3-interpolate/test/hue-test.js:38:3
    assert_eq!(i(0.4), 2.0); // d3-interpolate/test/hue-test.js:39:3
    assert_eq!(i(0.6), 358.0); // d3-interpolate/test/hue-test.js:40:3
    assert_eq!(i(0.8), 354.0); // d3-interpolate/test/hue-test.js:41:3
    assert_eq!(i(1.0), 350.0); // d3-interpolate/test/hue-test.js:42:3
}

// d3-interpolate/test/quantize-test.js: quantize(interpolate, n) returns n uniformly-spaced samples from the specified interpolator
#[test]
fn quantize_interpolate_n_returns_n_uniformly_spaced_samples_from_the_specified_interpolator() {
    assert_eq!(
        interpolate::quantize(interpolate::number(0.0, 1.0), 5),
        vec![
            (0.0 / 4.0),
            (1.0 / 4.0),
            (2.0 / 4.0),
            (3.0 / 4.0),
            (4.0 / 4.0)
        ]
    ); // d3-interpolate/test/quantize-test.js:5:3
    assert_eq!(
        interpolate::quantize(
            interpolate::rgb(
                "steelblue".parse::<Rgb>().expect("a CSS color"),
                "brown".parse::<Rgb>().expect("a CSS color")
            ),
            5
        )
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>(),
        vec![
            "rgb(70, 130, 180)",
            "rgb(94, 108, 146)",
            "rgb(118, 86, 111)",
            "rgb(141, 64, 77)",
            "rgb(165, 42, 42)"
        ]
    ); // d3-interpolate/test/quantize-test.js:12:3
}

// d3-interpolate/test/piecewise-test.js: piecewise(interpolate, values)(t) returns the expected values
#[test]
fn piecewise_interpolate_values_t_returns_the_expected_values() {
    let i = interpolate::piecewise(&[0.0, 2.0, 10.0], |a: &f64, b: &f64| {
        interpolate::number(*a, *b)
    });
    assert_eq!(i(-1.0), -4.0); // d3-interpolate/test/piecewise-test.js:6:3
    assert_eq!(i(0.0), 0.0); // d3-interpolate/test/piecewise-test.js:7:3
    assert_eq!(i(0.19), 0.76); // d3-interpolate/test/piecewise-test.js:8:3
    assert_eq!(i(0.21), 0.84); // d3-interpolate/test/piecewise-test.js:9:3
    assert_eq!(i(0.5), 2.0); // d3-interpolate/test/piecewise-test.js:10:3
    assert_eq!(i(0.75), 6.0); // d3-interpolate/test/piecewise-test.js:11:3
    assert_eq!(i(1.0), 10.0); // d3-interpolate/test/piecewise-test.js:12:3
}

// d3-interpolate/test/piecewise-test.js: piecewise(values) uses the default interpolator
#[test]
fn piecewise_values_uses_the_default_interpolator() {
    let i = interpolate::piecewise(&[0.0, 2.0, 10.0], |a: &f64, b: &f64| {
        interpolate::number(*a, *b)
    });
    assert_eq!(i(-1.0), -4.0); // d3-interpolate/test/piecewise-test.js:17:3
    assert_eq!(i(0.0), 0.0); // d3-interpolate/test/piecewise-test.js:18:3
    assert_eq!(i(0.19), 0.76); // d3-interpolate/test/piecewise-test.js:19:3
    assert_eq!(i(0.21), 0.84); // d3-interpolate/test/piecewise-test.js:20:3
    assert_eq!(i(0.5), 2.0); // d3-interpolate/test/piecewise-test.js:21:3
    assert_eq!(i(0.75), 6.0); // d3-interpolate/test/piecewise-test.js:22:3
    assert_eq!(i(1.0), 10.0); // d3-interpolate/test/piecewise-test.js:23:3
}
