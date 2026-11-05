//! `color::Rgb`, `Hsl`, `Lab`, `Hcl` and `Cubehelix` against d3-color
//! 3.1.0: every CSS form the parser accepts or rejects, the alpha rules,
//! conversions both ways, formats, clamping, and d3-interpolate 3.0.1's
//! `interpolateRgb` at gamma 1 alone and as a scale's range.

use rustd3_core::color::{Cubehelix, Hcl, Hsl, Lab, ParseColorError, Rgb};
use rustd3_core::interpolate::Interpolate;
use rustd3_core::scale;

/// `s` parsed as sRGB.
fn rgb(s: &str) -> Rgb {
    s.parse().expect("a CSS color")
}

/// `s` parsed as HSL.
fn hsl(s: &str) -> Hsl {
    s.parse().expect("a CSS color")
}

/// The channels and opacity of `c`, NaN written as `None`.
fn rgba(c: Rgb) -> [Option<f64>; 4] {
    [c.r, c.g, c.b, c.opacity].map(|v| (!v.is_nan()).then_some(v))
}

/// The components of `c`, NaN written as `None`.
fn hsla(c: Hsl) -> [Option<f64>; 4] {
    [c.h, c.s, c.l, c.opacity].map(|v| (!v.is_nan()).then_some(v))
}

/// Whether `a` is within 1e-6 of `b`, NaN matching NaN.
fn near(a: [Option<f64>; 4], b: [Option<f64>; 4]) -> bool {
    a.iter().zip(&b).all(|pair| match pair {
        (Some(x), Some(y)) => (x - y).abs() <= 1e-6,
        (None, None) => true,
        (Some(_) | None, _) => false,
    })
}

const NONE3: [Option<f64>; 3] = [None, None, None];

#[test]
fn hex_forms_double_short_digits_and_divide_alpha_by_255() {
    assert_eq!(rgb("#abcdef"), Rgb::new(171.0, 205.0, 239.0));
    assert_eq!(rgb("#abc"), Rgb::new(170.0, 187.0, 204.0));
    assert_eq!(
        rgb("#abc3"),
        Rgb::new(170.0, 187.0, 204.0).with_opacity(0.2)
    );
    assert_eq!(
        rgb("#abcdef33"),
        Rgb::new(171.0, 205.0, 239.0).with_opacity(0.2)
    );
    assert_eq!(
        rgba(rgb("#ff000080")),
        [Some(255.0), Some(0.0), Some(0.0), Some(0.5019607843137255)]
    );
    assert_eq!(rgb("#f008").opacity, 0.5333333333333333);
}

#[test]
fn a_hex_alpha_of_zero_drops_the_channels() {
    for s in ["#f000", "#0000", "#00000000"] {
        let [r, g, b, a] = rgba(rgb(s));
        assert_eq!(([r, g, b], a), (NONE3, Some(0.0)), "{s}");
    }
}

#[test]
fn rgb_takes_integers_or_percentages_of_255() {
    assert_eq!(rgb("rgb(12,34,56)"), Rgb::new(12.0, 34.0, 56.0));
    assert_eq!(rgb("rgb(100%, 0%, 50%)"), Rgb::new(255.0, 0.0, 127.5));
    assert_eq!(rgb("rgb(100%, 0%, 50%)").to_string(), "rgb(255, 0, 128)");
    let c = rgb("rgb(12%,34%,56%)");
    assert!(near(
        rgba(c),
        [Some(30.6), Some(86.7), Some(142.8), Some(1.0)]
    ));
    assert_eq!(rgb("rgb(.1%,.2%,.3%)").to_string(), "rgb(0, 1, 1)");
    assert_eq!(rgb("rgb(1e2%,0%,0%)"), Rgb::new(255.0, 0.0, 0.0));
    assert_eq!(rgb("rgb(-120,-30,-50)"), Rgb::new(-120.0, -30.0, -50.0));
    assert_eq!(rgb("rgb(+120,+30,+50)"), Rgb::new(120.0, 30.0, 50.0));
}

#[test]
fn rgba_drops_the_channels_at_an_alpha_of_at_most_zero() {
    assert_eq!(
        rgb("rgba(12,34,56,0.4)"),
        Rgb::new(12.0, 34.0, 56.0).with_opacity(0.4)
    );
    assert_eq!(rgb("rgba(120,30,50,.5)").opacity, 0.5);
    assert_eq!(rgb("rgba(120,30,50,1e-1)").opacity, 0.1);
    for (s, a) in [
        ("rgba(10,20,30,0)", 0.0),
        ("rgba(12,34,56,-0.4)", -0.4),
        ("rgba(12%,34%,56%,-0.4)", -0.4),
        ("hsla(120,50%,50%,0)", 0.0),
    ] {
        let [r, g, b, opacity] = rgba(rgb(s));
        assert_eq!(([r, g, b], opacity), (NONE3, Some(a)), "{s}");
    }
}

#[test]
fn hsl_keeps_its_components_and_applies_d3s_rules() {
    assert_eq!(
        hsla(hsl("hsl(60,100%,20%)")),
        [Some(60.0), Some(1.0), Some(0.2), Some(1.0)]
    );
    assert_eq!(hsl("hsla(60,100%,20%,0.4)").opacity, 0.4);
    assert_eq!(
        hsla(hsl("hsl(.9,.3%,.5%)")),
        [Some(0.9), Some(0.003), Some(0.005), Some(1.0)]
    );
    assert_eq!(
        hsla(hsl("hsl(120,0%,20%)")),
        [None, Some(0.0), Some(0.2), Some(1.0)]
    );
    assert_eq!(
        hsla(hsl("hsl(120,-10%,20%)")),
        [None, Some(-0.1), Some(0.2), Some(1.0)]
    );
    for l in ["-10%", "0%", "100%", "120%"] {
        let [h, s, _, _] = hsla(hsl(&format!("hsl(120,20%,{l})")));
        assert_eq!([h, s], [None, None], "{l}");
    }
    assert_eq!(
        hsla(hsl("hsla(120,20%,10%,-0.1)")),
        [None, None, None, Some(-0.1)]
    );
}

#[test]
fn an_hsl_string_converts_to_rgb_unrounded() {
    assert_eq!(rgb("hsl(120, 50%, 50%)"), Rgb::new(63.75, 191.25, 63.75));
    assert_eq!(rgb("hsl(120, 50%, 50%)").to_string(), "rgb(64, 191, 64)");
}

#[test]
fn names_trim_and_case_follow_d3() {
    assert_eq!(rgb("  Steelblue ").to_string(), "rgb(70, 130, 180)");
    assert_eq!(rgb("ReBeccaPurple").to_string(), "rgb(102, 51, 153)");
    assert_eq!(rgb(" aliceblue\t\n"), Rgb::hex(0xf0f8ff));
    assert_eq!(rgb(" #aBc\t\n"), Rgb::hex(0xaabbcc));
    assert_eq!(rgb(" rGB( 120 , 30 , 50 ) "), Rgb::new(120.0, 30.0, 50.0));
    assert_eq!(
        hsla(hsl(" HSl( 120 , 30% , 50% ) ")),
        [Some(120.0), Some(0.3), Some(0.5), Some(1.0)]
    );
    // JavaScript's trim takes U+FEFF and not U+0085.
    assert_eq!(rgb("\u{feff}red"), Rgb::hex(0xff0000));
    assert!("\u{85}red".parse::<Rgb>().is_err());
}

#[test]
fn transparent_is_nan_channels_at_opacity_zero() {
    let [r, g, b, a] = rgba(rgb("transparent"));
    assert_eq!(([r, g, b], a), (NONE3, Some(0.0)));
    assert_eq!(rgb("transPARENT").to_string(), "rgba(0, 0, 0, 0)");
}

#[test]
fn malformed_colors_are_rejected_with_the_input() {
    for bad in [
        "#zzz",
        "#12345",
        "#abcdef3",
        "#ab",
        "#+abc",
        "rgb(1.5, 2, 3)",
        "rgb(10%, 2, 3)",
        "rgb(120.,30,50)",
        "rgba(120,30,50,1.)",
        "rgb (120,30,50)",
        "hsl(120,30 %,50%)",
        "rgb(1,2,3,4)",
        "rgba(1,2,3)",
        "hsl(120,30,50)",
        "bostock",
        "invalid",
        "",
    ] {
        let error = bad.parse::<Rgb>().expect_err(bad);
        assert_eq!(error.get_input(), bad);
        assert!(bad.parse::<Hsl>().is_err(), "{bad}");
    }
    let error: Result<Rgb, ParseColorError> = "#zzz".parse();
    assert_eq!(
        error.map_err(|e| e.to_string()),
        Err(r##"not a CSS color: "#zzz""##.to_owned())
    );
}

#[test]
fn hsl_200_60_50_is_rgb_51_153_204() {
    let c = Hsl {
        h: 200.0,
        s: 0.6,
        l: 0.5,
        opacity: 1.0,
    };
    assert_eq!(Rgb::from(c).to_string(), "rgb(51, 153, 204)");
}

#[test]
fn steelblue_brightens_and_darkens_by_steps_of_0_7() {
    let steel = rgb("steelblue");
    assert_eq!(steel.brighter(1.0).to_string(), "rgb(100, 186, 255)");
    assert_eq!(steel.darker(1.0).to_string(), "rgb(49, 91, 126)");
    let c = rgb("rgba(165, 42, 42, 0.4)");
    assert_eq!(c.brighter(2.0).clamp().r, 255.0);
    assert_eq!(c.darker(0.5).to_string(), "rgba(138, 35, 35, 0.4)");
    let h = hsl("rgba(70, 130, 180, 0.4)");
    assert!((h.brighter(1.0).l - 0.7002801).abs() < 1e-6);
    assert!((h.darker(1.0).l - 0.3431373).abs() < 1e-6);
}

#[test]
fn rgb_converts_to_hsl_with_d3s_undefined_hues() {
    let steel = Hsl::from(rgb("steelblue"));
    assert_eq!(
        hsla(steel),
        [
            Some(207.27272727272728),
            Some(0.44),
            Some(0.4901960784313726),
            Some(1.0)
        ]
    );
    assert_eq!(
        hsla(hsl("#808080")),
        [None, Some(0.0), Some(0.5019607843137255), Some(1.0)]
    );
    assert_eq!(hsla(hsl("black")), [None, None, Some(0.0), Some(1.0)]);
    assert_eq!(hsla(hsl("white")), [None, None, Some(1.0), Some(1.0)]);
    assert_eq!(hsla(hsl("transparent")), [None, None, None, Some(0.0)]);
    let out = Hsl::from(Rgb::new(-10.0, 0.0, 0.0));
    assert_eq!((out.h, out.s), (180.0, -1.0));
}

#[test]
fn hsl_converts_to_rgb_and_back() {
    let red = Rgb::from(Hsl {
        h: 0.0,
        s: 1.0,
        l: 0.5,
        opacity: 0.4,
    });
    assert_eq!(red, Rgb::new(255.0, 0.0, 0.0).with_opacity(0.4));
    let gray = Rgb::from(Hsl {
        h: f64::NAN,
        s: 0.5,
        l: 0.4,
        opacity: 1.0,
    });
    assert_eq!(gray.to_string(), "rgb(102, 102, 102)");
    let back = Hsl::from(Rgb::from(hsl("hsl(325,50%,40%)")));
    assert!(near(
        hsla(back),
        [Some(325.0), Some(0.5), Some(0.4), Some(1.0)]
    ));
    assert_eq!(hsl("#abcdef").to_string(), "rgb(171, 205, 239)");
}

#[test]
fn formats_round_and_clamp_channels_and_opacity() {
    let steel = rgb("steelblue");
    assert_eq!(steel.format_hex(), "#4682b4");
    assert_eq!(
        Rgb::new(255.0, 0.0, 0.0).with_opacity(0.5).format_hex8(),
        "#ff000080"
    );
    let wild = Rgb {
        r: 300.0,
        g: -5.0,
        b: 12.4,
        opacity: 1.0,
    };
    assert_eq!(wild.to_string(), "rgb(255, 0, 12)");
    assert_eq!(Rgb::new(f64::NAN, 12.0, 34.0).to_string(), "rgb(0, 12, 34)");
    assert_eq!(
        Rgb::new(2.0, 3.0, -1.0).with_opacity(-0.2).to_string(),
        "rgba(2, 3, 0, 0)"
    );
    assert_eq!(
        Rgb::new(2.0, 3.0, -1.0).with_opacity(1.2).to_string(),
        "rgb(2, 3, 0)"
    );
    assert_eq!(
        Rgb::new(1.0, 2.0, 3.0).with_opacity(f64::NAN).to_string(),
        "rgb(1, 2, 3)"
    );
    assert_eq!(Rgb::new(0.5, 2.0, 3.0).to_string(), "rgb(1, 2, 3)");
    assert_eq!(rgb("hsla(60, 100%, 20%, 0.4)").format_hex8(), "#66660066");
    assert_eq!(
        rgb("#abcdef").with_opacity(f64::NAN).format_hex8(),
        "#abcdefff"
    );
}

#[test]
fn hsl_formats_clamp_hue_saturation_and_lightness() {
    assert_eq!(
        Hsl::from(rgb("#abcdef")).format_hsl(),
        "hsl(210, 68%, 80.3921568627451%)"
    );
    assert_eq!(
        Hsl::from(rgb("rgba(12%, 34%, 56%, 0.4)")).format_hsl(),
        "hsla(210, 64.70588235294117%, 34%, 0.4)"
    );
    let format = |h, s, l| {
        Hsl {
            h,
            s,
            l,
            opacity: 1.0,
        }
        .format_hsl()
    };
    assert_eq!(format(180.0, -100.0, -50.0), "hsl(180, 0%, 0%)");
    assert_eq!(format(-90.0, 50.0, 50.0), "hsl(270, 100%, 100%)");
    assert_eq!(format(420.0, 50.0, 50.0), "hsl(60, 100%, 100%)");
    assert_eq!(format(-360.0, 0.5, 0.5), "hsl(0, 50%, 50%)");
    assert_eq!(format(f64::NAN, f64::NAN, f64::NAN), "hsl(0, 0%, 0%)");
}

#[test]
fn clamp_rounds_rgb_channels_and_wraps_hsl_hues() {
    assert_eq!(
        Rgb::new(10.5, 20.5, 30.5).clamp(),
        Rgb::new(11.0, 21.0, 31.0)
    );
    assert_eq!(
        Rgb::new(-10.0, 300.0, f64::NAN).clamp(),
        Rgb::new(0.0, 255.0, 0.0)
    );
    let opacity = |a: f64| Rgb::new(0.0, 0.0, 0.0).with_opacity(a).clamp().opacity;
    assert_eq!(
        [opacity(-1.0), opacity(0.5), opacity(2.0), opacity(f64::NAN)],
        [0.0, 0.5, 1.0, 1.0]
    );
    let c = Hsl {
        h: -420.0,
        s: -0.1,
        l: 1.2,
        opacity: f64::NAN,
    }
    .clamp();
    assert_eq!(hsla(c), [Some(300.0), Some(0.0), Some(1.0), Some(1.0)]);
}

#[test]
fn displayable_checks_the_gamut_and_opacity() {
    assert!(rgb("white").displayable() && rgb("black").displayable());
    assert!(!Rgb::new(f64::NAN, 0.0, 0.0).displayable());
    assert!(!Rgb::new(-1.0, 0.0, 0.0).displayable());
    assert!(!Rgb::new(0.0, 256.0, 0.0).displayable());
    assert!(Rgb::new(255.4, -0.5, 0.0).displayable());
    assert!(Rgb::new(0.0, 0.0, 255.0).with_opacity(0.0).displayable());
    assert!(!Rgb::new(0.0, 0.0, 255.0).with_opacity(1.2).displayable());
    let h = |h, s, l, opacity| Hsl { h, s, l, opacity }.displayable();
    assert!(h(f64::NAN, f64::NAN, 1.0, 1.0));
    assert!(!h(f64::NAN, f64::NAN, 1.5, 1.0));
    assert!(!h(120.0, -0.5, 0.0, 1.0));
    assert!(!h(0.0, 1.0, 1.0, -0.2));
}

#[test]
fn hex_takes_the_lowest_24_bits() {
    assert_eq!(Rgb::hex(0x4682b4), rgb("steelblue"));
    assert_eq!(Rgb::hex(0xff_4682b4), Rgb::hex(0x4682b4));
    const RED: Rgb = Rgb::hex(0xff0000);
    assert_eq!(RED.to_string(), "rgb(255, 0, 0)");
}

#[test]
fn black_to_white_is_gray_halfway() {
    let mid = Rgb::hex(0x000000).lerp(&Rgb::hex(0xffffff), 0.5);
    assert_eq!(mid.to_string(), "rgb(128, 128, 128)");
    assert_eq!(mid, Rgb::new(127.5, 127.5, 127.5));
}

#[test]
fn a_color_range_makes_a_linear_scale_of_colors() {
    let steel = rgb("steelblue");
    let heat = scale::linear().range([Rgb::hex(0xffffff), steel]);
    assert_eq!(heat.apply(0.5).to_string(), "rgb(163, 193, 218)");
    assert_eq!(heat.apply(0.0), Rgb::hex(0xffffff));
    assert_eq!(heat.apply(1.0), steel);
}

#[test]
fn an_undefined_channel_takes_the_other_colors() {
    let red = rgb("red");
    assert_eq!(
        rgb("transparent").lerp(&red, 0.5).to_string(),
        "rgba(255, 0, 0, 0.5)"
    );
    let to = Rgb::new(60.0, 80.0, 100.0);
    assert_eq!(
        Rgb::new(f64::NAN, 20.0, 40.0).lerp(&to, 0.5),
        Rgb::new(60.0, 50.0, 70.0)
    );
    assert_eq!(
        to.lerp(&Rgb::new(20.0, 40.0, f64::NAN), 0.5),
        Rgb::new(40.0, 60.0, 100.0)
    );
    let [r, g, b, a] = rgba(Rgb::new(f64::NAN, 1.0, 1.0).lerp(&Rgb::new(f64::NAN, 1.0, 3.0), 0.5));
    assert_eq!([r, g, b, a], [None, Some(1.0), Some(2.0), Some(1.0)]);
}

#[test]
fn rgb_interpolation_is_d3s_with_gamma_one() {
    let i = |t| rgb("steelblue").lerp(&rgb("#f00"), t).to_string();
    assert_eq!(i(0.2), "rgb(107, 104, 144)");
    let a = rgb("rgba(70, 130, 180, 1)");
    let b = rgb("rgba(255, 0, 0, 0.2)");
    assert_eq!(a.lerp(&b, 0.2).to_string(), "rgba(107, 104, 144, 0.84)");
}

/// The three channels of a color, NaN written as `None`, to compare with d3
/// within 1e-6.
fn channels(c: [f64; 3]) -> [Option<f64>; 4] {
    [c[0], c[1], c[2], 1.0].map(|v| (!v.is_nan()).then_some(v))
}

#[test]
fn rgb_converts_to_lab_with_d3s_d50_constants() {
    let c = Lab::from(Rgb::new(170.0, 187.0, 204.0));
    assert!(near(
        channels([c.l, c.a, c.b]),
        channels([74.96879980931759, -3.398998724348956, -10.696507207853333])
    ));
    let steel: Lab = "steelblue".parse().expect("a CSS color");
    assert!(near(
        channels([steel.l, steel.a, steel.b]),
        channels([51.98624890550498, -8.362792037014344, -32.832699449697714])
    ));
    let gray = Lab::from(rgb("gray"));
    assert_eq!((gray.a, gray.b), (0.0, 0.0));
    assert_eq!(Rgb::from(Lab::gray(50.0)).to_string(), "rgb(119, 119, 119)");
    assert_eq!(
        Rgb::from(steel.brighter(1.0)).to_string(),
        "rgb(121, 177, 230)"
    );
    assert_eq!(steel.darker(1.0).l, steel.l - 18.0);
    let hsl: Lab = Lab::from(hsl("hsl(60,100%,20%)"));
    assert!((hsl.l - 41.97125732118659).abs() < 1e-6);
}

#[test]
fn lab_parses_through_rgb_and_prints_as_rgb() {
    assert_eq!(
        "moccasin".parse::<Lab>().map(|c| c.to_string()),
        Ok("rgb(255, 228, 181)".to_owned())
    );
    assert_eq!(
        "hsla(60, 100%, 20%, 0.4)"
            .parse::<Lab>()
            .map(|c| c.to_string()),
        Ok("rgba(102, 102, 0, 0.4)".to_owned())
    );
    assert_eq!(
        "invalid"
            .parse::<Lab>()
            .map_err(|e| e.get_input().to_owned()),
        Err("invalid".to_owned())
    );
    let undefined = Lab {
        a: f64::NAN,
        b: f64::NAN,
        ..Lab::new(50.0, 0.0, 0.0)
    };
    assert_eq!(undefined.to_string(), "rgb(119, 119, 119)");
}

#[test]
fn hcl_is_lab_in_cylindrical_form_with_undefined_hues() {
    let steel: Hcl = "steelblue".parse().expect("a CSS color");
    assert!(near(
        channels([steel.h, steel.c, steel.l]),
        channels([255.71009124439382, 33.88100417355618, 51.98624890550498])
    ));
    let white: Hcl = "white".parse().expect("a CSS color");
    assert_eq!(
        channels([white.h, white.c, white.l]),
        [None, None, Some(100.0), Some(1.0)]
    );
    let black = Hcl::from(Lab::new(0.0, 0.0, 0.0));
    assert_eq!(
        channels([black.h, black.c, black.l]),
        [None, None, Some(0.0), Some(1.0)]
    );
    let gray = Hcl::from(Lab::gray(50.0));
    assert_eq!(
        channels([gray.h, gray.c, gray.l]),
        [None, Some(0.0), Some(50.0), Some(1.0)]
    );
    assert_eq!(Hcl::from(Lab::new(50.0, 0.0, 10.0)).h, 90.0);
    assert_eq!(
        Rgb::from(Hcl::new(200.0, 40.0, 60.0)).to_string(),
        "rgb(0, 163, 167)"
    );
    assert_eq!(Hcl::lch(74.0, 11.0, 252.0), Hcl::new(252.0, 11.0, 74.0));
    let dark = Lab::from(Hcl::from(Lab::new(0.0, 10.0, 0.0)));
    assert_eq!((dark.a, dark.b), (10.0, 0.0));
    assert_eq!(Lab::from(Hcl::new(f64::NAN, 30.0, 40.0)), Lab::gray(40.0));
    assert_eq!(steel.brighter(1.0).l, steel.l + 18.0);
    assert_eq!(
        Hcl::from(hsl("hsl(60,100%,20%)")).to_string(),
        "rgb(102, 102, 0)"
    );
}

#[test]
fn cubehelix_converts_with_d3s_constants() {
    let steel: Cubehelix = "steelblue".parse().expect("a CSS color");
    assert!(near(
        channels([steel.h, steel.s, steel.l]),
        channels([202.84837896488932, 0.6273147230777709, 0.460784355920337])
    ));
    assert_eq!(steel.to_string(), "rgb(70, 130, 180)");
    let white = Cubehelix::from(rgb("white"));
    assert!(white.h.is_nan() && white.s.is_nan());
    assert_eq!(
        Cubehelix::new(f64::NAN, f64::NAN, 0.4).to_string(),
        "rgb(102, 102, 102)"
    );
    assert!((steel.brighter(1.0).l - steel.l / 0.7).abs() < 1e-15);
    assert_eq!(steel.darker(2.0).l, steel.l * 0.7_f64.powf(2.0));
    assert_eq!(
        Cubehelix::from(hsl("steelblue")).to_string(),
        "rgb(70, 130, 180)"
    );
}
