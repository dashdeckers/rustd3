//! Worked examples of formats, time and color, and of the color scales,
//! with the values d3 prints for them.

use rustd3_core::color::{Hcl, Hsl, Rgb};
use rustd3_core::interpolate::{self, Interpolate};
use rustd3_core::{scale, scheme};

#[test]
fn examples_in_formats_time_and_color() {
    let steel: Rgb = "steelblue".parse().expect("a CSS color");
    let white = Rgb::new(255.0, 255.0, 255.0);

    // Colors are value types.
    let hsl = Hsl {
        h: 200.0,
        s: 0.6,
        l: 0.5,
        opacity: 1.0,
    };
    assert_eq!(Rgb::from(hsl).to_string(), "rgb(51, 153, 204)");
    assert_eq!(steel.brighter(1.0).to_string(), "rgb(100, 186, 255)");
    let error = "#zzz".parse::<Rgb>().expect_err("not a color");
    assert_eq!(error.input(), "#zzz");
    let gray = Rgb::hex(0x000000).lerp(&Rgb::hex(0xffffff), 0.5);
    assert_eq!(gray.to_string(), "rgb(128, 128, 128)");

    // d3's rules for undefined hue: white has none.
    let blend = interpolate::hcl(Hcl::from(white), Hcl::from(steel));
    assert_eq!(blend(0.5).to_string(), "rgb(138, 194, 248)");

    // Schemes are d3's own code.
    assert_eq!(scheme::viridis(0.5).format_hex(), "#21918c");

    // Scales: a color range, and a sequential scale over a scheme.
    let heat = scale::linear().domain([0.0, 1.0]).range([white, steel]);
    assert_eq!(heat.apply(0.5).to_string(), "rgb(163, 193, 218)");
    let seq = scale::sequential([0.0, 50.0], scheme::blues);
    assert_eq!(seq.apply(25.0).to_string(), "rgb(109, 174, 213)");
}
