//! `interpolate`'s number and color interpolators and `Interpolate`
//! against d3-interpolate 3.0.1.

use rustd3_core::color::{Cubehelix, Hcl, Hsl, Lab, ParseColorError, Rgb};
use rustd3_core::interpolate::{self, Interpolate};

/// `s` parsed as a color of the space `C`.
fn color<C: std::str::FromStr<Err = ParseColorError>>(s: &str) -> C {
    s.parse().expect("a CSS color")
}

#[test]
fn number_blends_linearly() {
    assert_eq!(interpolate::number(10.0, 20.0)(0.25), 12.5);
    assert_eq!(interpolate::number(10.0, 42.0)(0.5), 26.0);
}

#[test]
fn number_is_exact_at_both_ends() {
    let i = interpolate::number(2e42, 335.0);
    assert_eq!(i(1.0), 335.0);
    assert_eq!(i(0.0), 2e42);
}

#[test]
fn round_rounds_the_blend_not_its_ends() {
    assert_eq!(interpolate::round(10.0, 42.0)(0.3), 20.0);
    assert_eq!(interpolate::round(2.6, 3.6)(0.6), 3.0);
}

#[test]
fn round_takes_ties_toward_positive_infinity() {
    assert_eq!(interpolate::round(-3.0, -2.0)(0.5), -2.0);
    assert_eq!(interpolate::round(2.0, 3.0)(0.5), 3.0);
}

#[test]
fn f32_blends_through_f64() {
    assert_eq!(1.0_f32.lerp(&3.0, 0.5), 2.0);
    assert_eq!(0.5_f64.lerp(&1.5, 1.0), 1.5);
}

#[test]
fn colors_blend_in_their_space_as_d3s_interpolators() {
    let [steel, brown]: [Rgb; 2] = [color("steelblue"), color("brown")];
    let at_half = |blend: &dyn Fn(f64) -> Rgb| blend(0.5).to_string();
    assert_eq!(
        at_half(&interpolate::rgb(steel, brown)),
        "rgb(118, 86, 111)"
    );
    assert_eq!(
        at_half(&interpolate::rgb_gamma(2.2, steel, brown)),
        "rgb(128, 98, 134)"
    );
    let lab = interpolate::lab(Lab::from(steel), Lab::from(brown));
    assert_eq!(at_half(&lab), "rgb(139, 93, 108)");
    let [a, b] = [Hcl::from(steel), Hcl::from(brown)];
    assert_eq!(at_half(&interpolate::hcl(a, b)), "rgb(150, 81, 154)");
    assert_eq!(at_half(&interpolate::hcl_long(a, b)), "rgb(35, 122, 57)");
    let [a, b] = [Hsl::from(steel), Hsl::from(brown)];
    assert_eq!(at_half(&interpolate::hsl(a, b)), "rgb(141, 55, 173)");
    assert_eq!(at_half(&interpolate::hsl_long(a, b)), "rgb(87, 173, 55)");
    let [a, b] = [Cubehelix::from(steel), Cubehelix::from(brown)];
    assert_eq!(
        at_half(&interpolate::cubehelix(1.0, a, b)),
        "rgb(143, 62, 172)"
    );
    let [purple, orange]: [Cubehelix; 2] = [color("purple"), color("orange")];
    assert_eq!(
        at_half(&interpolate::cubehelix(3.0, purple, orange)),
        "rgb(207, 4, 32)"
    );
}

#[test]
fn white_has_no_hue_so_hcl_takes_the_other_colors() {
    let blend = interpolate::hcl(color("white"), color("steelblue"));
    assert_eq!(blend(0.5).to_string(), "rgb(138, 194, 248)");
}

#[test]
fn hue_wraps_the_shorter_arc_into_0_to_360() {
    assert_eq!(interpolate::hue(350.0, 10.0)(0.5), 0.0);
    assert_eq!(interpolate::hue(10.0, 350.0)(0.2), 6.0);
    assert!(interpolate::hue(f64::NAN, f64::NAN)(0.5).is_nan());
}

#[test]
fn color_spaces_interpolate_into_their_own_type() {
    let [a, b]: [Hsl; 2] = [color("hsl(10,50%,50%)"), color("hsl(350,50%,50%)")];
    assert_eq!(a.lerp(&b, 0.5).h, 0.0);
    assert_eq!(Rgb::from(a.lerp(&b, 0.2)).to_string(), "rgb(191, 77, 64)");
    let [a, b]: [Lab; 2] = [color("steelblue"), color("brown")];
    assert_eq!(Rgb::from(a.lerp(&b, 0.5)).to_string(), "rgb(139, 93, 108)");
    let gray = Hcl::new(f64::NAN, f64::NAN, 0.0);
    let red: Hcl = color("#f00");
    assert_eq!(red.lerp(&gray, 0.5).to_string(), "rgb(166, 0, 0)");
    let [a, b]: [Cubehelix; 2] = [color("purple"), color("orange")];
    assert_eq!(a.lerp(&b, 0.2).to_string(), "rgb(208, 1, 127)");
}
