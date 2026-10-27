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

/// `f` stored as a scale or an animation stores it: `'static`, cloned and
/// sent across threads.
fn stored<R: Send + 'static>(
    f: impl Fn(f64) -> R + Clone + Send + Sync + 'static,
) -> impl Fn(f64) -> R {
    let copy = f.clone();
    std::thread::spawn(move || {
        copy(0.0);
    })
    .join()
    .expect("no panic");
    f
}

#[test]
fn every_interpolator_is_clone_send_sync_and_static() {
    let colors = [Rgb::hex(0xff0000), Rgb::hex(0x008000), Rgb::hex(0x0000ff)];
    let local = colors;
    assert_eq!(
        stored(interpolate::rgb_basis(&local))(0.5).to_string(),
        "rgb(43, 85, 43)"
    );
    assert_eq!(
        stored(interpolate::basis(&[0.0, 10.0, 0.0]))(0.5),
        6.666666666666667
    );
    let spread = stored(interpolate::piecewise(&local, |a, b| {
        interpolate::rgb(*a, *b)
    }));
    assert_eq!(spread(0.75).to_string(), "rgb(0, 64, 128)");
    assert_eq!(stored(interpolate::hue(350.0, 10.0))(0.5), 0.0);
    let [a, b] = [
        Cubehelix::new(300.0, 0.5, 0.0),
        Cubehelix::new(-240.0, 0.5, 1.0),
    ];
    assert_eq!(
        stored(interpolate::cubehelix_long(1.0, a, b))(0.5).to_string(),
        "rgb(160, 121, 73)"
    );
}

#[test]
fn rgb_basis_runs_d3s_spline_per_channel() {
    let [red, green, blue] = [Rgb::hex(0xff0000), Rgb::hex(0x008000), Rgb::hex(0x0000ff)];
    let spline = interpolate::rgb_basis(&[red, green, blue]);
    assert_eq!(spline(0.25).to_string(), "rgb(133, 59, 5)");
    // Segment 1 at 0: (v0 + 4·v1 + v2) / 6 per channel.
    assert_eq!(spline(0.5), Rgb::new(42.5, 128.0 * 4.0 / 6.0, 42.5));
    assert_eq!(spline(-1.0), red);
    let undefined = interpolate::rgb_basis(&[Rgb::new(f64::NAN, 0.0, 0.0), Rgb::hex(0xff0000)]);
    assert_eq!(undefined(0.0), Rgb::new(0.0, 0.0, 0.0));
}

#[test]
fn piecewise_extrapolates_its_end_segments() {
    let i = interpolate::piecewise(&[0.0, 2.0, 10.0], |a, b| interpolate::number(*a, *b));
    assert_eq!([i(-1.0), i(0.0), i(0.19), i(0.21)], [-4.0, 0.0, 0.76, 0.84]);
    assert_eq!([i(0.5), i(0.75), i(1.0), i(2.0)], [2.0, 6.0, 10.0, 26.0]);
}

#[test]
fn quantize_samples_both_ends() {
    let samples = interpolate::quantize(interpolate::number(0.0, 10.0), 4);
    assert_eq!(samples, [0.0, 3.333333333333333, 6.666666666666666, 10.0]);
    let colors = interpolate::quantize(interpolate::rgb(color("steelblue"), color("brown")), 3);
    assert_eq!(colors[1].to_string(), "rgb(118, 86, 111)");
    assert!(interpolate::quantize(|t| t, 1)[0].is_nan());
    assert!(interpolate::quantize(|t| t, 0).is_empty());
}

#[test]
fn round_halfway_between_20_and_10_is_15() {
    assert_eq!(interpolate::round(20.0, 10.0)(0.5), 15.0);
}

#[test]
fn timestamps_blend_in_nanoseconds_and_saturate() {
    use jiff::Timestamp;
    let a: Timestamp = "2026-10-27T00:00:00Z".parse().expect("a timestamp");
    let b: Timestamp = "2026-10-28T00:00:00Z".parse().expect("a timestamp");
    assert_eq!(a.lerp(&b, 0.25).to_string(), "2026-10-27T06:00:00Z");
    assert_eq!(b.lerp(&a, 0.5), a.lerp(&b, 0.5));
    let early: Timestamp = "2026-10-27T00:00:00.000000001Z"
        .parse()
        .expect("a timestamp");
    assert_eq!(a.lerp(&early, 0.5), early);
    assert_eq!(a.lerp(&b, -1e30), Timestamp::MIN);
    assert_eq!(a.lerp(&b, f64::NAN), a);
}
