//! d3's tests of d3-color, as assertions on rustd3-core; each names d3's file, line and column.

use rustd3_core::color::{Hcl, Hsl, Lab, Rgb};

use super::support::{hcl_equal, hsl_equal, lab_equal, rgb_approx_equal, rgb_equal};

// d3-color/test/color-test.js: color(format) parses CSS color names (e.g., "rebeccapurple")
#[test]
fn color_format_parses_css_color_names_e_g_rebeccapurple() {
    assert!(rgb_approx_equal(
        &"moccasin".parse::<Rgb>().expect("a CSS color"),
        255.0,
        228.0,
        181.0,
        1.0
    )); // d3-color/test/color-test.js:6:3
    assert!(rgb_approx_equal(
        &"aliceblue".parse::<Rgb>().expect("a CSS color"),
        240.0,
        248.0,
        255.0,
        1.0
    )); // d3-color/test/color-test.js:7:3
    assert!(rgb_approx_equal(
        &"yellow".parse::<Rgb>().expect("a CSS color"),
        255.0,
        255.0,
        0.0,
        1.0
    )); // d3-color/test/color-test.js:8:3
    assert!(rgb_approx_equal(
        &"moccasin".parse::<Rgb>().expect("a CSS color"),
        255.0,
        228.0,
        181.0,
        1.0
    )); // d3-color/test/color-test.js:9:3
    assert!(rgb_approx_equal(
        &"aliceblue".parse::<Rgb>().expect("a CSS color"),
        240.0,
        248.0,
        255.0,
        1.0
    )); // d3-color/test/color-test.js:10:3
    assert!(rgb_approx_equal(
        &"yellow".parse::<Rgb>().expect("a CSS color"),
        255.0,
        255.0,
        0.0,
        1.0
    )); // d3-color/test/color-test.js:11:3
    assert!(rgb_approx_equal(
        &"rebeccapurple".parse::<Rgb>().expect("a CSS color"),
        102.0,
        51.0,
        153.0,
        1.0
    )); // d3-color/test/color-test.js:12:3
    assert!(rgb_approx_equal(
        &"transparent".parse::<Rgb>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        f64::NAN,
        0.0
    )); // d3-color/test/color-test.js:13:3
}

// d3-color/test/color-test.js: color(format) parses 6-digit hexadecimal (e.g., "#abcdef")
#[test]
fn color_format_parses_6_digit_hexadecimal_e_g_abcdef() {
    assert!(rgb_approx_equal(
        &"#abcdef".parse::<Rgb>().expect("a CSS color"),
        171.0,
        205.0,
        239.0,
        1.0
    )); // d3-color/test/color-test.js:17:3
}

// d3-color/test/color-test.js: color(format) parses 3-digit hexadecimal (e.g., "#abc")
#[test]
fn color_format_parses_3_digit_hexadecimal_e_g_abc() {
    assert!(rgb_approx_equal(
        &"#abc".parse::<Rgb>().expect("a CSS color"),
        170.0,
        187.0,
        204.0,
        1.0
    )); // d3-color/test/color-test.js:21:3
}

// d3-color/test/color-test.js: color(format) does not parse 7-digit hexadecimal (e.g., "#abcdef3")
#[test]
fn color_format_does_not_parse_7_digit_hexadecimal_e_g_abcdef3() {
    assert!("#abcdef3".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:25:3
}

// d3-color/test/color-test.js: color(format) parses 8-digit hexadecimal (e.g., "#abcdef33")
#[test]
fn color_format_parses_8_digit_hexadecimal_e_g_abcdef33() {
    assert!(rgb_approx_equal(
        &"#abcdef33".parse::<Rgb>().expect("a CSS color"),
        171.0,
        205.0,
        239.0,
        0.2
    )); // d3-color/test/color-test.js:29:3
}

// d3-color/test/color-test.js: color(format) parses 4-digit hexadecimal (e.g., "#abc3")
#[test]
fn color_format_parses_4_digit_hexadecimal_e_g_abc3() {
    assert!(rgb_approx_equal(
        &"#abc3".parse::<Rgb>().expect("a CSS color"),
        170.0,
        187.0,
        204.0,
        0.2
    )); // d3-color/test/color-test.js:33:3
}

// d3-color/test/color-test.js: color(format) parses RGB integer format (e.g., "rgb(12,34,56)")
#[test]
fn color_format_parses_rgb_integer_format_e_g_rgb_12_34_56() {
    assert!(rgb_approx_equal(
        &"rgb(12,34,56)".parse::<Rgb>().expect("a CSS color"),
        12.0,
        34.0,
        56.0,
        1.0
    )); // d3-color/test/color-test.js:37:3
}

// d3-color/test/color-test.js: color(format) parses RGBA integer format (e.g., "rgba(12,34,56,0.4)")
#[test]
fn color_format_parses_rgba_integer_format_e_g_rgba_12_34_56_0_4() {
    assert!(rgb_approx_equal(
        &"rgba(12,34,56,0.4)".parse::<Rgb>().expect("a CSS color"),
        12.0,
        34.0,
        56.0,
        0.4
    )); // d3-color/test/color-test.js:41:3
}

// d3-color/test/color-test.js: color(format) parses RGB percentage format (e.g., "rgb(12%,34%,56%)")
#[test]
fn color_format_parses_rgb_percentage_format_e_g_rgb_12_34_56() {
    assert!(rgb_approx_equal(
        &"rgb(12%,34%,56%)".parse::<Rgb>().expect("a CSS color"),
        31.0,
        87.0,
        143.0,
        1.0
    )); // d3-color/test/color-test.js:45:3
    assert!(rgb_equal(
        &"rgb(100%,100%,100%)".parse::<Rgb>().expect("a CSS color"),
        255.0,
        255.0,
        255.0,
        1.0
    )); // d3-color/test/color-test.js:46:3
}

// d3-color/test/color-test.js: color(format) parses RGBA percentage format (e.g., "rgba(12%,34%,56%,0.4)")
#[test]
fn color_format_parses_rgba_percentage_format_e_g_rgba_12_34_56_0_4() {
    assert!(rgb_approx_equal(
        &"rgba(12%,34%,56%,0.4)".parse::<Rgb>().expect("a CSS color"),
        31.0,
        87.0,
        143.0,
        0.4
    )); // d3-color/test/color-test.js:50:3
    assert!(rgb_equal(
        &"rgba(100%,100%,100%,0.4)"
            .parse::<Rgb>()
            .expect("a CSS color"),
        255.0,
        255.0,
        255.0,
        0.4
    )); // d3-color/test/color-test.js:51:3
}

// d3-color/test/color-test.js: color(format) parses HSL format (e.g., "hsl(60,100%,20%)")
#[test]
fn color_format_parses_hsl_format_e_g_hsl_60_100_20() {
    assert!(hsl_equal(
        &"hsl(60,100%,20%)".parse::<Hsl>().expect("a CSS color"),
        60.0,
        1.0,
        0.2,
        1.0
    )); // d3-color/test/color-test.js:55:3
}

// d3-color/test/color-test.js: color(format) parses HSLA format (e.g., "hsla(60,100%,20%,0.4)")
#[test]
fn color_format_parses_hsla_format_e_g_hsla_60_100_20_0_4() {
    assert!(hsl_equal(
        &"hsla(60,100%,20%,0.4)".parse::<Hsl>().expect("a CSS color"),
        60.0,
        1.0,
        0.2,
        0.4
    )); // d3-color/test/color-test.js:59:3
}

// d3-color/test/color-test.js: color(format) ignores leading and trailing whitespace
#[test]
fn color_format_ignores_leading_and_trailing_whitespace() {
    assert!(rgb_approx_equal(
        &" aliceblue\t\n".parse::<Rgb>().expect("a CSS color"),
        240.0,
        248.0,
        255.0,
        1.0
    )); // d3-color/test/color-test.js:63:3
    assert!(rgb_approx_equal(
        &" #abc\t\n".parse::<Rgb>().expect("a CSS color"),
        170.0,
        187.0,
        204.0,
        1.0
    )); // d3-color/test/color-test.js:64:3
    assert!(rgb_approx_equal(
        &" #aabbcc\t\n".parse::<Rgb>().expect("a CSS color"),
        170.0,
        187.0,
        204.0,
        1.0
    )); // d3-color/test/color-test.js:65:3
    assert!(rgb_approx_equal(
        &" rgb(120,30,50)\t\n".parse::<Rgb>().expect("a CSS color"),
        120.0,
        30.0,
        50.0,
        1.0
    )); // d3-color/test/color-test.js:66:3
    assert!(hsl_equal(
        &" hsl(120,30%,50%)\t\n".parse::<Hsl>().expect("a CSS color"),
        120.0,
        0.3,
        0.5,
        1.0
    )); // d3-color/test/color-test.js:67:3
}

// d3-color/test/color-test.js: color(format) ignores whitespace between numbers
#[test]
fn color_format_ignores_whitespace_between_numbers() {
    assert!(rgb_approx_equal(
        &" rgb( 120 , 30 , 50 ) "
            .parse::<Rgb>()
            .expect("a CSS color"),
        120.0,
        30.0,
        50.0,
        1.0
    )); // d3-color/test/color-test.js:71:3
    assert!(hsl_equal(
        &" hsl( 120 , 30% , 50% ) "
            .parse::<Hsl>()
            .expect("a CSS color"),
        120.0,
        0.3,
        0.5,
        1.0
    )); // d3-color/test/color-test.js:72:3
    assert!(rgb_approx_equal(
        &" rgba( 12 , 34 , 56 , 0.4 ) "
            .parse::<Rgb>()
            .expect("a CSS color"),
        12.0,
        34.0,
        56.0,
        0.4
    )); // d3-color/test/color-test.js:73:3
    assert!(rgb_approx_equal(
        &" rgba( 12% , 34% , 56% , 0.4 ) "
            .parse::<Rgb>()
            .expect("a CSS color"),
        31.0,
        87.0,
        143.0,
        0.4
    )); // d3-color/test/color-test.js:74:3
    assert!(hsl_equal(
        &" hsla( 60 , 100% , 20% , 0.4 ) "
            .parse::<Hsl>()
            .expect("a CSS color"),
        60.0,
        1.0,
        0.2,
        0.4
    )); // d3-color/test/color-test.js:75:3
}

// d3-color/test/color-test.js: color(format) allows number signs
#[test]
fn color_format_allows_number_signs() {
    assert!(rgb_approx_equal(
        &"rgb(+120,+30,+50)".parse::<Rgb>().expect("a CSS color"),
        120.0,
        30.0,
        50.0,
        1.0
    )); // d3-color/test/color-test.js:79:3
    assert!(hsl_equal(
        &"hsl(+120,+30%,+50%)".parse::<Hsl>().expect("a CSS color"),
        120.0,
        0.3,
        0.5,
        1.0
    )); // d3-color/test/color-test.js:80:3
    assert!(rgb_approx_equal(
        &"rgb(-120,-30,-50)".parse::<Rgb>().expect("a CSS color"),
        -120.0,
        -30.0,
        -50.0,
        1.0
    )); // d3-color/test/color-test.js:81:3
    assert!(hsl_equal(
        &"hsl(-120,-30%,-50%)".parse::<Hsl>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        -0.5,
        1.0
    )); // d3-color/test/color-test.js:82:3
    assert!(rgb_approx_equal(
        &"rgba(12,34,56,+0.4)".parse::<Rgb>().expect("a CSS color"),
        12.0,
        34.0,
        56.0,
        0.4
    )); // d3-color/test/color-test.js:83:3
    assert!(rgb_approx_equal(
        &"rgba(12,34,56,-0.4)".parse::<Rgb>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        f64::NAN,
        -0.4
    )); // d3-color/test/color-test.js:84:3
    assert!(rgb_approx_equal(
        &"rgba(12%,34%,56%,+0.4)"
            .parse::<Rgb>()
            .expect("a CSS color"),
        31.0,
        87.0,
        143.0,
        0.4
    )); // d3-color/test/color-test.js:85:3
    assert!(rgb_approx_equal(
        &"rgba(12%,34%,56%,-0.4)"
            .parse::<Rgb>()
            .expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        f64::NAN,
        -0.4
    )); // d3-color/test/color-test.js:86:3
    assert!(hsl_equal(
        &"hsla(60,100%,20%,+0.4)"
            .parse::<Hsl>()
            .expect("a CSS color"),
        60.0,
        1.0,
        0.2,
        0.4
    )); // d3-color/test/color-test.js:87:3
    assert!(hsl_equal(
        &"hsla(60,100%,20%,-0.4)"
            .parse::<Hsl>()
            .expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        f64::NAN,
        -0.4
    )); // d3-color/test/color-test.js:88:3
}

// d3-color/test/color-test.js: color(format) allows decimals for non-integer values
#[test]
fn color_format_allows_decimals_for_non_integer_values() {
    assert!(rgb_approx_equal(
        &"rgb(20.0%,30.4%,51.2%)"
            .parse::<Rgb>()
            .expect("a CSS color"),
        51.0,
        78.0,
        131.0,
        1.0
    )); // d3-color/test/color-test.js:92:3
    assert!(hsl_equal(
        &"hsl(20.0,30.4%,51.2%)".parse::<Hsl>().expect("a CSS color"),
        20.0,
        0.304,
        0.512,
        1.0
    )); // d3-color/test/color-test.js:93:3
}

// d3-color/test/color-test.js: color(format) allows leading decimal for hue, opacity and percentages
#[test]
fn color_format_allows_leading_decimal_for_hue_opacity_and_percentages() {
    assert!(hsl_equal(
        &"hsl(.9,.3%,.5%)".parse::<Hsl>().expect("a CSS color"),
        0.9,
        0.003,
        0.005,
        1.0
    )); // d3-color/test/color-test.js:97:3
    assert!(hsl_equal(
        &"hsla(.9,.3%,.5%,.5)".parse::<Hsl>().expect("a CSS color"),
        0.9,
        0.003,
        0.005,
        0.5
    )); // d3-color/test/color-test.js:98:3
    assert!(rgb_approx_equal(
        &"rgb(.1%,.2%,.3%)".parse::<Rgb>().expect("a CSS color"),
        0.0,
        1.0,
        1.0,
        1.0
    )); // d3-color/test/color-test.js:99:3
    assert!(rgb_approx_equal(
        &"rgba(120,30,50,.5)".parse::<Rgb>().expect("a CSS color"),
        120.0,
        30.0,
        50.0,
        0.5
    )); // d3-color/test/color-test.js:100:3
}

// d3-color/test/color-test.js: color(format) allows exponential format for hue, opacity and percentages
#[test]
fn color_format_allows_exponential_format_for_hue_opacity_and_percentages() {
    assert!(hsl_equal(
        &"hsl(1e1,2e1%,3e1%)".parse::<Hsl>().expect("a CSS color"),
        10.0,
        0.2,
        0.3,
        1.0
    )); // d3-color/test/color-test.js:104:3
    assert!(hsl_equal(
        &"hsla(9e-1,3e-1%,5e-1%,5e-1)"
            .parse::<Hsl>()
            .expect("a CSS color"),
        0.9,
        0.003,
        0.005,
        0.5
    )); // d3-color/test/color-test.js:105:3
    assert!(rgb_approx_equal(
        &"rgb(1e-1%,2e-1%,3e-1%)"
            .parse::<Rgb>()
            .expect("a CSS color"),
        0.0,
        1.0,
        1.0,
        1.0
    )); // d3-color/test/color-test.js:106:3
    assert!(rgb_approx_equal(
        &"rgba(120,30,50,1e-1)".parse::<Rgb>().expect("a CSS color"),
        120.0,
        30.0,
        50.0,
        0.1
    )); // d3-color/test/color-test.js:107:3
}

// d3-color/test/color-test.js: color(format) does not allow decimals for integer values
#[test]
fn color_format_does_not_allow_decimals_for_integer_values() {
    assert!("rgb(120.5,30,50)".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:111:3
}

// d3-color/test/color-test.js: color(format) does not allow empty decimals
#[test]
fn color_format_does_not_allow_empty_decimals() {
    assert!("rgb(120.,30,50)".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:115:3
    assert!("rgb(120.%,30%,50%)".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:116:3
    assert!("rgba(120,30,50,1.)".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:117:3
    assert!("rgba(12%,30%,50%,1.)".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:118:3
    assert!("hsla(60,100%,20%,1.)".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:119:3
}

// d3-color/test/color-test.js: color(format) does not allow made-up names
#[test]
fn color_format_does_not_allow_made_up_names() {
    assert!("bostock".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:123:3
}

// d3-color/test/color-test.js: color(format) allows achromatic colors
#[test]
fn color_format_allows_achromatic_colors() {
    assert!(rgb_approx_equal(
        &"rgba(0,0,0,0)".parse::<Rgb>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        f64::NAN,
        0.0
    )); // d3-color/test/color-test.js:127:3
    assert!(rgb_approx_equal(
        &"#0000".parse::<Rgb>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        f64::NAN,
        0.0
    )); // d3-color/test/color-test.js:128:3
    assert!(rgb_approx_equal(
        &"#00000000".parse::<Rgb>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        f64::NAN,
        0.0
    )); // d3-color/test/color-test.js:129:3
}

// d3-color/test/color-test.js: color(format) does not allow whitespace before open paren or percent sign
#[test]
fn color_format_does_not_allow_whitespace_before_open_paren_or_percent_sign() {
    assert!("rgb (120,30,50)".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:133:3
    assert!("rgb (12%,30%,50%)".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:134:3
    assert!("hsl (120,30%,50%)".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:135:3
    assert!("hsl(120,30 %,50%)".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:136:3
    assert!("rgba (120,30,50,1)".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:137:3
    assert!("rgba (12%,30%,50%,1)".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:138:3
    assert!("hsla (120,30%,50%,1)".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:139:3
}

// d3-color/test/color-test.js: color(format) is case-insensitive
#[test]
fn color_format_is_case_insensitive() {
    assert!(rgb_approx_equal(
        &"aLiCeBlUE".parse::<Rgb>().expect("a CSS color"),
        240.0,
        248.0,
        255.0,
        1.0
    )); // d3-color/test/color-test.js:143:3
    assert!(rgb_approx_equal(
        &"transPARENT".parse::<Rgb>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        f64::NAN,
        0.0
    )); // d3-color/test/color-test.js:144:3
    assert!(rgb_approx_equal(
        &" #aBc\t\n".parse::<Rgb>().expect("a CSS color"),
        170.0,
        187.0,
        204.0,
        1.0
    )); // d3-color/test/color-test.js:145:3
    assert!(rgb_approx_equal(
        &" #aaBBCC\t\n".parse::<Rgb>().expect("a CSS color"),
        170.0,
        187.0,
        204.0,
        1.0
    )); // d3-color/test/color-test.js:146:3
    assert!(rgb_approx_equal(
        &" rGB(120,30,50)\t\n".parse::<Rgb>().expect("a CSS color"),
        120.0,
        30.0,
        50.0,
        1.0
    )); // d3-color/test/color-test.js:147:3
    assert!(hsl_equal(
        &" HSl(120,30%,50%)\t\n".parse::<Hsl>().expect("a CSS color"),
        120.0,
        0.3,
        0.5,
        1.0
    )); // d3-color/test/color-test.js:148:3
}

// d3-color/test/color-test.js: color(format) returns undefined RGB channel values for unknown formats
#[test]
fn color_format_returns_undefined_rgb_channel_values_for_unknown_formats() {
    assert!("invalid".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:152:3
    assert!("hasOwnProperty".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:153:3
    assert!("__proto__".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:154:3
    assert!("#ab".parse::<Rgb>().is_err()); // d3-color/test/color-test.js:155:3
}

// d3-color/test/color-test.js: color(format).hex() returns a hexadecimal string
#[test]
fn color_format_hex_returns_a_hexadecimal_string() {
    assert_eq!(
        "rgba(12%,34%,56%,0.4)"
            .parse::<Rgb>()
            .expect("a CSS color")
            .format_hex(),
        "#1f578f"
    ); // d3-color/test/color-test.js:159:3
}

// d3-color/test/rgb-test.js: rgb(…) exposes r, g and b channel values and opacity
#[test]
fn rgb_exposes_r_g_and_b_channel_values_and_opacity() {
    assert!(rgb_approx_equal(
        &"#abc".parse::<Rgb>().expect("a CSS color"),
        170.0,
        187.0,
        204.0,
        1.0
    )); // d3-color/test/rgb-test.js:12:3
    assert!(rgb_approx_equal(
        &"rgba(170, 187, 204, 0.4)"
            .parse::<Rgb>()
            .expect("a CSS color"),
        170.0,
        187.0,
        204.0,
        0.4
    )); // d3-color/test/rgb-test.js:13:3
}

// d3-color/test/rgb-test.js: rgb.toString() formats as rgb(…) or rgba(…)
#[test]
fn rgb_tostring_formats_as_rgb_or_rgba() {
    assert_eq!(
        "#abcdef".parse::<Rgb>().expect("a CSS color").to_string(),
        "rgb(171, 205, 239)"
    ); // d3-color/test/rgb-test.js:17:3
    assert_eq!(
        "moccasin".parse::<Rgb>().expect("a CSS color").to_string(),
        "rgb(255, 228, 181)"
    ); // d3-color/test/rgb-test.js:18:3
    assert_eq!(
        "hsl(60, 100%, 20%)"
            .parse::<Rgb>()
            .expect("a CSS color")
            .to_string(),
        "rgb(102, 102, 0)"
    ); // d3-color/test/rgb-test.js:19:3
    assert_eq!(
        "rgb(12, 34, 56)"
            .parse::<Rgb>()
            .expect("a CSS color")
            .to_string(),
        "rgb(12, 34, 56)"
    ); // d3-color/test/rgb-test.js:20:3
    assert_eq!(Rgb::new(12.0, 34.0, 56.0).to_string(), "rgb(12, 34, 56)"); // d3-color/test/rgb-test.js:21:3
    assert_eq!(
        Rgb::from(Hsl {
            h: 60.0,
            s: 1.0,
            l: 0.2,
            opacity: 1.0
        })
        .to_string(),
        "rgb(102, 102, 0)"
    ); // d3-color/test/rgb-test.js:22:3
    assert_eq!(
        "rgba(12, 34, 56, 0.4)"
            .parse::<Rgb>()
            .expect("a CSS color")
            .to_string(),
        "rgba(12, 34, 56, 0.4)"
    ); // d3-color/test/rgb-test.js:23:3
    assert_eq!(
        "rgba(12%, 34%, 56%, 0.4)"
            .parse::<Rgb>()
            .expect("a CSS color")
            .to_string(),
        "rgba(31, 87, 143, 0.4)"
    ); // d3-color/test/rgb-test.js:24:3
    assert_eq!(
        "hsla(60, 100%, 20%, 0.4)"
            .parse::<Rgb>()
            .expect("a CSS color")
            .to_string(),
        "rgba(102, 102, 0, 0.4)"
    ); // d3-color/test/rgb-test.js:25:3
}

// d3-color/test/rgb-test.js: rgb.formatRgb() formats as rgb(…) or rgba(…)
#[test]
fn rgb_formatrgb_formats_as_rgb_or_rgba() {
    assert_eq!(
        "#abcdef".parse::<Rgb>().expect("a CSS color").to_string(),
        "rgb(171, 205, 239)"
    ); // d3-color/test/rgb-test.js:29:3
    assert_eq!(
        "hsl(60, 100%, 20%)"
            .parse::<Rgb>()
            .expect("a CSS color")
            .to_string(),
        "rgb(102, 102, 0)"
    ); // d3-color/test/rgb-test.js:30:3
    assert_eq!(
        "rgba(12%, 34%, 56%, 0.4)"
            .parse::<Rgb>()
            .expect("a CSS color")
            .to_string(),
        "rgba(31, 87, 143, 0.4)"
    ); // d3-color/test/rgb-test.js:31:3
    assert_eq!(
        "hsla(60, 100%, 20%, 0.4)"
            .parse::<Rgb>()
            .expect("a CSS color")
            .to_string(),
        "rgba(102, 102, 0, 0.4)"
    ); // d3-color/test/rgb-test.js:32:3
}

// d3-color/test/rgb-test.js: rgb.formatHsl() formats as hsl(…) or hsla(…)
#[test]
fn rgb_formathsl_formats_as_hsl_or_hsla() {
    assert_eq!(
        Hsl::from("#abcdef".parse::<Rgb>().expect("a CSS color")).format_hsl(),
        "hsl(210, 68%, 80.3921568627451%)"
    ); // d3-color/test/rgb-test.js:36:3
    assert_eq!(
        Hsl::from("hsl(60, 100%, 20%)".parse::<Rgb>().expect("a CSS color")).format_hsl(),
        "hsl(60, 100%, 20%)"
    ); // d3-color/test/rgb-test.js:37:3
    assert_eq!(
        Hsl::from(
            "rgba(12%, 34%, 56%, 0.4)"
                .parse::<Rgb>()
                .expect("a CSS color")
        )
        .format_hsl(),
        "hsla(210, 64.70588235294117%, 34%, 0.4)"
    ); // d3-color/test/rgb-test.js:38:3
    assert_eq!(
        Hsl::from(
            "hsla(60, 100%, 20%, 0.4)"
                .parse::<Rgb>()
                .expect("a CSS color")
        )
        .format_hsl(),
        "hsla(60, 100%, 20%, 0.4)"
    ); // d3-color/test/rgb-test.js:39:3
}

// d3-color/test/rgb-test.js: rgb.formatHex() formats as #rrggbb
#[test]
fn rgb_formathex_formats_as_rrggbb() {
    assert_eq!(
        "#abcdef".parse::<Rgb>().expect("a CSS color").format_hex(),
        "#abcdef"
    ); // d3-color/test/rgb-test.js:43:3
    assert_eq!(
        "hsl(60, 100%, 20%)"
            .parse::<Rgb>()
            .expect("a CSS color")
            .format_hex(),
        "#666600"
    ); // d3-color/test/rgb-test.js:44:3
    assert_eq!(
        "rgba(12%, 34%, 56%, 0.4)"
            .parse::<Rgb>()
            .expect("a CSS color")
            .format_hex(),
        "#1f578f"
    ); // d3-color/test/rgb-test.js:45:3
    assert_eq!(
        "hsla(60, 100%, 20%, 0.4)"
            .parse::<Rgb>()
            .expect("a CSS color")
            .format_hex(),
        "#666600"
    ); // d3-color/test/rgb-test.js:46:3
}

// d3-color/test/rgb-test.js: rgb.formatHex8() formats as #rrggbbaa
#[test]
fn rgb_formathex8_formats_as_rrggbbaa() {
    assert_eq!(
        "#abcdef".parse::<Rgb>().expect("a CSS color").format_hex8(),
        "#abcdefff"
    ); // d3-color/test/rgb-test.js:50:3
    assert_eq!(
        "hsl(60, 100%, 20%)"
            .parse::<Rgb>()
            .expect("a CSS color")
            .format_hex8(),
        "#666600ff"
    ); // d3-color/test/rgb-test.js:51:3
    assert_eq!(
        "rgba(12%, 34%, 56%, 0.4)"
            .parse::<Rgb>()
            .expect("a CSS color")
            .format_hex8(),
        "#1f578f66"
    ); // d3-color/test/rgb-test.js:52:3
    assert_eq!(
        "hsla(60, 100%, 20%, 0.4)"
            .parse::<Rgb>()
            .expect("a CSS color")
            .format_hex8(),
        "#66660066"
    ); // d3-color/test/rgb-test.js:53:3
}

// d3-color/test/rgb-test.js: rgb.toString() reflects r, g and b channel values and opacity
#[test]
fn rgb_tostring_reflects_r_g_and_b_channel_values_and_opacity() {
    let mut c = "#abc".parse::<Rgb>().expect("a CSS color");
    c.r += 1.0;
    c.g += 1.0;
    c.b += 1.0;
    c.opacity = 0.5;
    assert_eq!(c.to_string(), "rgba(171, 188, 205, 0.5)"); // d3-color/test/rgb-test.js:64:3
}

// d3-color/test/rgb-test.js: rgb.toString() treats undefined channel values as 0
#[test]
fn rgb_tostring_treats_undefined_channel_values_as_0() {
    assert_eq!(Rgb::new(f64::NAN, 12.0, 34.0).to_string(), "rgb(0, 12, 34)"); // d3-color/test/rgb-test.js:69:3
}

// d3-color/test/rgb-test.js: rgb.toString() treats undefined opacity as 1
#[test]
fn rgb_tostring_treats_undefined_opacity_as_1() {
    let mut c = "#abc".parse::<Rgb>().expect("a CSS color");
    c.r += 1.0;
    c.g += 1.0;
    c.b += 1.0;
    c.opacity = f64::NAN;
    assert_eq!(c.to_string(), "rgb(171, 188, 205)"); // d3-color/test/rgb-test.js:75:3
}

// d3-color/test/rgb-test.js: rgb.toString() clamps r, g, b and opacity channel values
#[test]
fn rgb_tostring_clamps_r_g_b_and_opacity_channel_values() {
    assert_eq!(Rgb::new(-1.0, 2.0, 3.0).to_string(), "rgb(0, 2, 3)"); // d3-color/test/rgb-test.js:79:3
    assert_eq!(Rgb::new(2.0, -1.0, 3.0).to_string(), "rgb(2, 0, 3)"); // d3-color/test/rgb-test.js:80:3
    assert_eq!(Rgb::new(2.0, 3.0, -1.0).to_string(), "rgb(2, 3, 0)"); // d3-color/test/rgb-test.js:81:3
    assert_eq!(
        Rgb::new(2.0, 3.0, -1.0).with_opacity(-0.2).to_string(),
        "rgba(2, 3, 0, 0)"
    ); // d3-color/test/rgb-test.js:82:3
    assert_eq!(
        Rgb::new(2.0, 3.0, -1.0).with_opacity(1.2).to_string(),
        "rgb(2, 3, 0)"
    ); // d3-color/test/rgb-test.js:83:3
}

// d3-color/test/rgb-test.js: rgb.toString() rounds r, g and b channel values
#[test]
fn rgb_tostring_rounds_r_g_and_b_channel_values() {
    assert_eq!(Rgb::new(0.5, 2.0, 3.0).to_string(), "rgb(1, 2, 3)"); // d3-color/test/rgb-test.js:87:3
    assert_eq!(Rgb::new(2.0, 0.5, 3.0).to_string(), "rgb(2, 1, 3)"); // d3-color/test/rgb-test.js:88:3
    assert_eq!(Rgb::new(2.0, 3.0, 0.5).to_string(), "rgb(2, 3, 1)"); // d3-color/test/rgb-test.js:89:3
}

// d3-color/test/rgb-test.js: rgb(r, g, b) does not round channel values
#[test]
fn rgb_r_g_b_does_not_round_channel_values() {
    assert!(rgb_equal(&Rgb::new(1.2, 2.6, 42.9), 1.2, 2.6, 42.9, 1.0)); // d3-color/test/rgb-test.js:93:3
}

// d3-color/test/rgb-test.js: rgb(r, g, b) does not clamp channel values
#[test]
fn rgb_r_g_b_does_not_clamp_channel_values() {
    assert!(rgb_approx_equal(
        &Rgb::new(-10.0, -20.0, -30.0),
        -10.0,
        -20.0,
        -30.0,
        1.0
    )); // d3-color/test/rgb-test.js:97:3
    assert!(rgb_approx_equal(
        &Rgb::new(300.0, 400.0, 500.0),
        300.0,
        400.0,
        500.0,
        1.0
    )); // d3-color/test/rgb-test.js:98:3
}

// d3-color/test/rgb-test.js: rgb(r, g, b).clamp() rounds and clamps channel values
#[test]
fn rgb_r_g_b_clamp_rounds_and_clamps_channel_values() {
    assert!(rgb_approx_equal(
        &Rgb::new(-10.0, -20.0, -30.0).clamp(),
        0.0,
        0.0,
        0.0,
        1.0
    )); // d3-color/test/rgb-test.js:102:3
    assert!(rgb_approx_equal(
        &Rgb::new(10.5, 20.5, 30.5).clamp(),
        11.0,
        21.0,
        31.0,
        1.0
    )); // d3-color/test/rgb-test.js:103:3
    assert!(rgb_approx_equal(
        &Rgb::new(300.0, 400.0, 500.0).clamp(),
        255.0,
        255.0,
        255.0,
        1.0
    )); // d3-color/test/rgb-test.js:104:3
    assert_eq!(
        Rgb::new(10.5, 20.5, 30.5)
            .with_opacity(-1.0)
            .clamp()
            .opacity,
        0.0
    ); // d3-color/test/rgb-test.js:105:3
    assert_eq!(
        Rgb::new(10.5, 20.5, 30.5).with_opacity(0.5).clamp().opacity,
        0.5
    ); // d3-color/test/rgb-test.js:106:3
    assert_eq!(
        Rgb::new(10.5, 20.5, 30.5).with_opacity(2.0).clamp().opacity,
        1.0
    ); // d3-color/test/rgb-test.js:107:3
    assert_eq!(
        Rgb::new(10.5, 20.5, 30.5)
            .with_opacity(f64::NAN)
            .clamp()
            .opacity,
        1.0
    ); // d3-color/test/rgb-test.js:108:3
}

// d3-color/test/rgb-test.js: rgb(r, g, b, opacity) does not clamp opacity
#[test]
fn rgb_r_g_b_opacity_does_not_clamp_opacity() {
    assert!(rgb_approx_equal(
        &Rgb::new(-10.0, -20.0, -30.0).with_opacity(-0.2),
        -10.0,
        -20.0,
        -30.0,
        -0.2
    )); // d3-color/test/rgb-test.js:112:3
    assert!(rgb_approx_equal(
        &Rgb::new(300.0, 400.0, 500.0).with_opacity(1.2),
        300.0,
        400.0,
        500.0,
        1.2
    )); // d3-color/test/rgb-test.js:113:3
}

// d3-color/test/rgb-test.js: rgb(format) parses the specified format and converts to RGB
#[test]
fn rgb_format_parses_the_specified_format_and_converts_to_rgb() {
    assert!(rgb_approx_equal(
        &"#abcdef".parse::<Rgb>().expect("a CSS color"),
        171.0,
        205.0,
        239.0,
        1.0
    )); // d3-color/test/rgb-test.js:139:3
    assert!(rgb_approx_equal(
        &"#abc".parse::<Rgb>().expect("a CSS color"),
        170.0,
        187.0,
        204.0,
        1.0
    )); // d3-color/test/rgb-test.js:140:3
    assert!(rgb_approx_equal(
        &"rgb(12, 34, 56)".parse::<Rgb>().expect("a CSS color"),
        12.0,
        34.0,
        56.0,
        1.0
    )); // d3-color/test/rgb-test.js:141:3
    assert!(rgb_approx_equal(
        &"rgb(12%, 34%, 56%)".parse::<Rgb>().expect("a CSS color"),
        31.0,
        87.0,
        143.0,
        1.0
    )); // d3-color/test/rgb-test.js:142:3
    assert!(rgb_approx_equal(
        &"hsl(60,100%,20%)".parse::<Rgb>().expect("a CSS color"),
        102.0,
        102.0,
        0.0,
        1.0
    )); // d3-color/test/rgb-test.js:143:3
    assert!(rgb_approx_equal(
        &"aliceblue".parse::<Rgb>().expect("a CSS color"),
        240.0,
        248.0,
        255.0,
        1.0
    )); // d3-color/test/rgb-test.js:144:3
    assert!(rgb_approx_equal(
        &"hsla(60,100%,20%,0.4)".parse::<Rgb>().expect("a CSS color"),
        102.0,
        102.0,
        0.0,
        0.4
    )); // d3-color/test/rgb-test.js:145:3
}

// d3-color/test/rgb-test.js: rgb(format) ignores all channels if the alpha is <= 0
#[test]
fn rgb_format_ignores_all_channels_if_the_alpha_is_0() {
    assert!(rgb_approx_equal(
        &"rgba(12,34,45,0)".parse::<Rgb>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        f64::NAN,
        0.0
    )); // d3-color/test/rgb-test.js:149:3
    assert!(rgb_approx_equal(
        &"rgba(12,34,45,-0.1)".parse::<Rgb>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        f64::NAN,
        -0.1
    )); // d3-color/test/rgb-test.js:150:3
}

// d3-color/test/rgb-test.js: rgb(rgb) copies an RGB color
#[test]
fn rgb_rgb_copies_an_rgb_color() {
    let mut c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Rgb>()
        .expect("a CSS color");
    let c2 = c1;
    assert!(rgb_approx_equal(&c1, 70.0, 130.0, 180.0, 0.4)); // d3-color/test/rgb-test.js:160:3
    c1.r = {
        c1.g = {
            c1.b = {
                c1.opacity = 0.0;
                c1.opacity
            };
            c1.b
        };
        c1.g
    };
    assert!(rgb_approx_equal(&c1, 0.0, 0.0, 0.0, 0.0)); // d3-color/test/rgb-test.js:162:3
    assert!(rgb_approx_equal(&c2, 70.0, 130.0, 180.0, 0.4)); // d3-color/test/rgb-test.js:163:3
}

// d3-color/test/rgb-test.js: rgb(hsl) converts from HSL
#[test]
fn rgb_hsl_converts_from_hsl() {
    assert!(rgb_approx_equal(
        &Rgb::from(Hsl {
            h: 0.0,
            s: 1.0,
            l: 0.5,
            opacity: 1.0
        }),
        255.0,
        0.0,
        0.0,
        1.0
    )); // d3-color/test/rgb-test.js:167:3
    assert!(rgb_approx_equal(
        &Rgb::from(Hsl {
            h: 0.0,
            s: 1.0,
            l: 0.5,
            opacity: 0.4
        }),
        255.0,
        0.0,
        0.0,
        0.4
    )); // d3-color/test/rgb-test.js:168:3
}

// d3-color/test/rgb-test.js: rgb.displayable() returns true if the color is within the RGB gamut and opacity is in [0,1]
#[test]
fn rgb_displayable_returns_true_if_the_color_is_within_the_rgb_gamut_and_opacity_is_in_0_1() {
    assert!("white".parse::<Rgb>().expect("a CSS color").displayable()); // d3-color/test/rgb-test.js:180:3
    assert!("red".parse::<Rgb>().expect("a CSS color").displayable()); // d3-color/test/rgb-test.js:181:3
    assert!("black".parse::<Rgb>().expect("a CSS color").displayable()); // d3-color/test/rgb-test.js:182:3
    assert!(!Rgb::new(-1.0, 0.0, 0.0).displayable()); // d3-color/test/rgb-test.js:184:3
    assert!(!Rgb::new(0.0, -1.0, 0.0).displayable()); // d3-color/test/rgb-test.js:185:3
    assert!(!Rgb::new(0.0, 0.0, -1.0).displayable()); // d3-color/test/rgb-test.js:186:3
    assert!(!Rgb::new(256.0, 0.0, 0.0).displayable()); // d3-color/test/rgb-test.js:187:3
    assert!(!Rgb::new(0.0, 256.0, 0.0).displayable()); // d3-color/test/rgb-test.js:188:3
    assert!(!Rgb::new(0.0, 0.0, 256.0).displayable()); // d3-color/test/rgb-test.js:189:3
    assert!(Rgb::new(0.0, 0.0, 255.0).with_opacity(0.0).displayable()); // d3-color/test/rgb-test.js:190:3
    assert!(!Rgb::new(0.0, 0.0, 255.0).with_opacity(1.2).displayable()); // d3-color/test/rgb-test.js:191:3
    assert!(!Rgb::new(0.0, 0.0, 255.0).with_opacity(-0.2).displayable()); // d3-color/test/rgb-test.js:192:3
}

// d3-color/test/rgb-test.js: rgb.brighter(k) returns a brighter color if k > 0
#[test]
fn rgb_brighter_k_returns_a_brighter_color_if_k_0() {
    let c = "rgba(165, 42, 42, 0.4)"
        .parse::<Rgb>()
        .expect("a CSS color");
    assert!(rgb_approx_equal(&c.brighter(0.5), 197.0, 50.0, 50.0, 0.4)); // d3-color/test/rgb-test.js:197:3
    assert!(rgb_approx_equal(&c.brighter(1.0), 236.0, 60.0, 60.0, 0.4)); // d3-color/test/rgb-test.js:198:3
    assert!(rgb_approx_equal(&c.brighter(2.0), 337.0, 86.0, 86.0, 0.4)); // d3-color/test/rgb-test.js:199:3
}

// d3-color/test/rgb-test.js: rgb.brighter(k) returns a copy
#[test]
fn rgb_brighter_k_returns_a_copy() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Rgb>()
        .expect("a CSS color");
    let c2 = c1.brighter(1.0);
    assert!(rgb_approx_equal(&c1, 70.0, 130.0, 180.0, 0.4)); // d3-color/test/rgb-test.js:205:3
    assert!(rgb_approx_equal(&c2, 100.0, 186.0, 257.0, 0.4)); // d3-color/test/rgb-test.js:206:3
}

// d3-color/test/rgb-test.js: rgb.brighter() is equivalent to rgb.brighter(1)
#[test]
fn rgb_brighter_is_equivalent_to_rgb_brighter_1() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Rgb>()
        .expect("a CSS color");
    let c2 = c1.brighter(1.0);
    let c3 = c1.brighter(1.0);
    assert!(rgb_approx_equal(&c2, c3.r, c3.g, c3.b, 0.4)); // d3-color/test/rgb-test.js:213:3
}

// d3-color/test/rgb-test.js: rgb.brighter(k) is equivalent to rgb.darker(-k)
#[test]
fn rgb_brighter_k_is_equivalent_to_rgb_darker_k() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Rgb>()
        .expect("a CSS color");
    let c2 = c1.brighter(1.5);
    let c3 = c1.darker(-1.5);
    assert!(rgb_approx_equal(&c2, c3.r, c3.g, c3.b, 0.4)); // d3-color/test/rgb-test.js:220:3
}

// d3-color/test/rgb-test.js: rgb("black").brighter() still returns black
#[test]
fn rgb_black_brighter_still_returns_black() {
    let c1 = "black".parse::<Rgb>().expect("a CSS color");
    let c2 = c1.brighter(1.0);
    assert!(rgb_approx_equal(&c1, 0.0, 0.0, 0.0, 1.0)); // d3-color/test/rgb-test.js:226:3
    assert!(rgb_approx_equal(&c2, 0.0, 0.0, 0.0, 1.0)); // d3-color/test/rgb-test.js:227:3
}

// d3-color/test/rgb-test.js: rgb.darker(k) returns a darker color if k > 0
#[test]
fn rgb_darker_k_returns_a_darker_color_if_k_0() {
    let c = "rgba(165, 42, 42, 0.4)"
        .parse::<Rgb>()
        .expect("a CSS color");
    assert!(rgb_approx_equal(&c.darker(0.5), 138.0, 35.0, 35.0, 0.4)); // d3-color/test/rgb-test.js:232:3
    assert!(rgb_approx_equal(&c.darker(1.0), 115.0, 29.0, 29.0, 0.4)); // d3-color/test/rgb-test.js:233:3
    assert!(rgb_approx_equal(&c.darker(2.0), 81.0, 21.0, 21.0, 0.4)); // d3-color/test/rgb-test.js:234:3
}

// d3-color/test/rgb-test.js: rgb.darker(k) returns a copy
#[test]
fn rgb_darker_k_returns_a_copy() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Rgb>()
        .expect("a CSS color");
    let c2 = c1.darker(1.0);
    assert!(rgb_approx_equal(&c1, 70.0, 130.0, 180.0, 0.4)); // d3-color/test/rgb-test.js:240:3
    assert!(rgb_approx_equal(&c2, 49.0, 91.0, 126.0, 0.4)); // d3-color/test/rgb-test.js:241:3
}

// d3-color/test/rgb-test.js: rgb.darker() is equivalent to rgb.darker(1)
#[test]
fn rgb_darker_is_equivalent_to_rgb_darker_1() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Rgb>()
        .expect("a CSS color");
    let c2 = c1.darker(1.0);
    let c3 = c1.darker(1.0);
    assert!(rgb_approx_equal(&c2, c3.r, c3.g, c3.b, 0.4)); // d3-color/test/rgb-test.js:248:3
}

// d3-color/test/rgb-test.js: rgb.darker(k) is equivalent to rgb.brighter(-k)
#[test]
fn rgb_darker_k_is_equivalent_to_rgb_brighter_k() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Rgb>()
        .expect("a CSS color");
    let c2 = c1.darker(1.5);
    let c3 = c1.brighter(-1.5);
    assert!(rgb_approx_equal(&c2, c3.r, c3.g, c3.b, 0.4)); // d3-color/test/rgb-test.js:255:3
}

// d3-color/test/rgb-test.js: rgb.rgb() returns this
#[test]
fn rgb_rgb_returns_this() {
    let c = Rgb::new(70.0, 130.0, 180.0);
    assert_eq!(c.to_string(), c.to_string()); // d3-color/test/rgb-test.js:260:3
}

// d3-color/test/rgb-test.js: rgb.copy(…) returns a new rgb with the specified channel values
#[test]
fn rgb_copy_returns_a_new_rgb_with_the_specified_channel_values() {
    let c = Rgb::new(70.0, 130.0, 180.0);
    assert_eq!(c.to_string(), "rgb(70, 130, 180)"); // d3-color/test/rgb-test.js:266:3
    assert_eq!(
        Rgb { opacity: 0.2, ..c }.to_string(),
        "rgba(70, 130, 180, 0.2)"
    ); // d3-color/test/rgb-test.js:267:3
    assert_eq!(Rgb { r: 20.0, ..c }.to_string(), "rgb(20, 130, 180)"); // d3-color/test/rgb-test.js:268:3
    assert_eq!(
        Rgb {
            r: 20.0,
            g: 40.0,
            ..c
        }
        .to_string(),
        "rgb(20, 40, 180)"
    ); // d3-color/test/rgb-test.js:269:3
}

// d3-color/test/hsl-test.js: hsl(…) exposes h, s, and l channel values and opacity
#[test]
fn hsl_exposes_h_s_and_l_channel_values_and_opacity() {
    assert!(hsl_equal(
        &"#abc".parse::<Hsl>().expect("a CSS color"),
        210.0,
        0.25,
        0.7333333,
        1.0
    )); // d3-color/test/hsl-test.js:12:3
    assert!(hsl_equal(
        &"hsla(60, 100%, 20%, 0.4)"
            .parse::<Hsl>()
            .expect("a CSS color"),
        60.0,
        1.0,
        0.2,
        0.4
    )); // d3-color/test/hsl-test.js:13:3
}

// d3-color/test/hsl-test.js: hsl.toString() converts to RGB and formats as rgb(…) or rgba(…)
#[test]
fn hsl_tostring_converts_to_rgb_and_formats_as_rgb_or_rgba() {
    assert_eq!(
        "#abcdef".parse::<Hsl>().expect("a CSS color").to_string(),
        "rgb(171, 205, 239)"
    ); // d3-color/test/hsl-test.js:17:3
    assert_eq!(
        "moccasin".parse::<Hsl>().expect("a CSS color").to_string(),
        "rgb(255, 228, 181)"
    ); // d3-color/test/hsl-test.js:18:3
    assert_eq!(
        "hsl(60, 100%, 20%)"
            .parse::<Hsl>()
            .expect("a CSS color")
            .to_string(),
        "rgb(102, 102, 0)"
    ); // d3-color/test/hsl-test.js:19:3
    assert_eq!(
        "hsla(60, 100%, 20%, 0.4)"
            .parse::<Hsl>()
            .expect("a CSS color")
            .to_string(),
        "rgba(102, 102, 0, 0.4)"
    ); // d3-color/test/hsl-test.js:20:3
    assert_eq!(
        "rgb(12, 34, 56)"
            .parse::<Hsl>()
            .expect("a CSS color")
            .to_string(),
        "rgb(12, 34, 56)"
    ); // d3-color/test/hsl-test.js:21:3
    assert_eq!(
        Hsl::from(Rgb::new(12.0, 34.0, 56.0)).to_string(),
        "rgb(12, 34, 56)"
    ); // d3-color/test/hsl-test.js:22:3
    assert_eq!(
        Hsl {
            h: 60.0,
            s: 1.0,
            l: 0.2,
            opacity: 1.0
        }
        .to_string(),
        "rgb(102, 102, 0)"
    ); // d3-color/test/hsl-test.js:23:3
    assert_eq!(
        Hsl {
            h: 60.0,
            s: 1.0,
            l: 0.2,
            opacity: 0.4
        }
        .to_string(),
        "rgba(102, 102, 0, 0.4)"
    ); // d3-color/test/hsl-test.js:24:3
}

// d3-color/test/hsl-test.js: hsl.formatRgb() formats as rgb(…) or rgba(…)
#[test]
fn hsl_formatrgb_formats_as_rgb_or_rgba() {
    assert_eq!(
        "#abcdef".parse::<Hsl>().expect("a CSS color").to_string(),
        "rgb(171, 205, 239)"
    ); // d3-color/test/hsl-test.js:28:3
    assert_eq!(
        "hsl(60, 100%, 20%)"
            .parse::<Hsl>()
            .expect("a CSS color")
            .to_string(),
        "rgb(102, 102, 0)"
    ); // d3-color/test/hsl-test.js:29:3
    assert_eq!(
        "rgba(12%, 34%, 56%, 0.4)"
            .parse::<Hsl>()
            .expect("a CSS color")
            .to_string(),
        "rgba(31, 87, 143, 0.4)"
    ); // d3-color/test/hsl-test.js:30:3
    assert_eq!(
        "hsla(60, 100%, 20%, 0.4)"
            .parse::<Hsl>()
            .expect("a CSS color")
            .to_string(),
        "rgba(102, 102, 0, 0.4)"
    ); // d3-color/test/hsl-test.js:31:3
}

// d3-color/test/hsl-test.js: hsl.formatHsl() formats as hsl(…) or hsla(…)
#[test]
fn hsl_formathsl_formats_as_hsl_or_hsla() {
    assert_eq!(
        "#abcdef".parse::<Hsl>().expect("a CSS color").format_hsl(),
        "hsl(210, 68%, 80.3921568627451%)"
    ); // d3-color/test/hsl-test.js:35:3
    assert_eq!(
        "hsl(60, 100%, 20%)"
            .parse::<Hsl>()
            .expect("a CSS color")
            .format_hsl(),
        "hsl(60, 100%, 20%)"
    ); // d3-color/test/hsl-test.js:36:3
    assert_eq!(
        "rgba(12%, 34%, 56%, 0.4)"
            .parse::<Hsl>()
            .expect("a CSS color")
            .format_hsl(),
        "hsla(210, 64.70588235294117%, 34%, 0.4)"
    ); // d3-color/test/hsl-test.js:37:3
    assert_eq!(
        "hsla(60, 100%, 20%, 0.4)"
            .parse::<Hsl>()
            .expect("a CSS color")
            .format_hsl(),
        "hsla(60, 100%, 20%, 0.4)"
    ); // d3-color/test/hsl-test.js:38:3
}

// d3-color/test/hsl-test.js: hsl.formatHsl() clamps to the expected range
#[test]
fn hsl_formathsl_clamps_to_the_expected_range() {
    assert_eq!(
        Hsl {
            h: 180.0,
            s: -100.0,
            l: -50.0,
            opacity: 1.0
        }
        .format_hsl(),
        "hsl(180, 0%, 0%)"
    ); // d3-color/test/hsl-test.js:42:3
    assert_eq!(
        Hsl {
            h: 180.0,
            s: 150.0,
            l: 200.0,
            opacity: 1.0
        }
        .format_hsl(),
        "hsl(180, 100%, 100%)"
    ); // d3-color/test/hsl-test.js:43:3
    assert_eq!(
        Hsl {
            h: -90.0,
            s: 50.0,
            l: 50.0,
            opacity: 1.0
        }
        .format_hsl(),
        "hsl(270, 100%, 100%)"
    ); // d3-color/test/hsl-test.js:44:3
    assert_eq!(
        Hsl {
            h: 420.0,
            s: 50.0,
            l: 50.0,
            opacity: 1.0
        }
        .format_hsl(),
        "hsl(60, 100%, 100%)"
    ); // d3-color/test/hsl-test.js:45:3
}

// d3-color/test/hsl-test.js: hsl.formatHex() formats as #rrggbb
#[test]
fn hsl_formathex_formats_as_rrggbb() {
    assert_eq!(
        Rgb::from("#abcdef".parse::<Hsl>().expect("a CSS color")).format_hex(),
        "#abcdef"
    ); // d3-color/test/hsl-test.js:49:3
    assert_eq!(
        Rgb::from("hsl(60, 100%, 20%)".parse::<Hsl>().expect("a CSS color")).format_hex(),
        "#666600"
    ); // d3-color/test/hsl-test.js:50:3
    assert_eq!(
        Rgb::from(
            "rgba(12%, 34%, 56%, 0.4)"
                .parse::<Hsl>()
                .expect("a CSS color")
        )
        .format_hex(),
        "#1f578f"
    ); // d3-color/test/hsl-test.js:51:3
    assert_eq!(
        Rgb::from(
            "hsla(60, 100%, 20%, 0.4)"
                .parse::<Hsl>()
                .expect("a CSS color")
        )
        .format_hex(),
        "#666600"
    ); // d3-color/test/hsl-test.js:52:3
}

// d3-color/test/hsl-test.js: hsl.toString() reflects h, s and l channel values and opacity
#[test]
fn hsl_tostring_reflects_h_s_and_l_channel_values_and_opacity() {
    let mut c = "#abc".parse::<Hsl>().expect("a CSS color");
    c.h += 10.0;
    c.s += 0.01;
    c.l -= 0.01;
    c.opacity = 0.4;
    assert_eq!(c.to_string(), "rgba(166, 178, 203, 0.4)"); // d3-color/test/hsl-test.js:58:3
}

// d3-color/test/hsl-test.js: hsl.toString() treats undefined channel values as 0
#[test]
fn hsl_tostring_treats_undefined_channel_values_as_0() {
    assert_eq!(
        "#000".parse::<Hsl>().expect("a CSS color").to_string(),
        "rgb(0, 0, 0)"
    ); // d3-color/test/hsl-test.js:63:3
    assert_eq!(
        "#ccc".parse::<Hsl>().expect("a CSS color").to_string(),
        "rgb(204, 204, 204)"
    ); // d3-color/test/hsl-test.js:64:3
    assert_eq!(
        "#fff".parse::<Hsl>().expect("a CSS color").to_string(),
        "rgb(255, 255, 255)"
    ); // d3-color/test/hsl-test.js:65:3
    assert_eq!(
        Hsl {
            h: f64::NAN,
            s: 0.5,
            l: 0.4,
            opacity: 1.0
        }
        .to_string(),
        "rgb(102, 102, 102)"
    ); // d3-color/test/hsl-test.js:66:3
    assert_eq!(
        Hsl {
            h: 120.0,
            s: f64::NAN,
            l: 0.4,
            opacity: 1.0
        }
        .to_string(),
        "rgb(102, 102, 102)"
    ); // d3-color/test/hsl-test.js:67:3
    assert_eq!(
        Hsl {
            h: f64::NAN,
            s: f64::NAN,
            l: 0.4,
            opacity: 1.0
        }
        .to_string(),
        "rgb(102, 102, 102)"
    ); // d3-color/test/hsl-test.js:68:3
    assert_eq!(
        Hsl {
            h: 120.0,
            s: 0.5,
            l: f64::NAN,
            opacity: 1.0
        }
        .to_string(),
        "rgb(0, 0, 0)"
    ); // d3-color/test/hsl-test.js:69:3
}

// d3-color/test/hsl-test.js: hsl.toString() treats undefined opacity as 1
#[test]
fn hsl_tostring_treats_undefined_opacity_as_1() {
    let mut c = "#abc".parse::<Hsl>().expect("a CSS color");
    c.opacity = f64::NAN;
    assert_eq!(c.to_string(), "rgb(170, 187, 204)"); // d3-color/test/hsl-test.js:75:3
}

// d3-color/test/hsl-test.js: hsl(h, s, l) does not wrap hue to [0,360)
#[test]
fn hsl_h_s_l_does_not_wrap_hue_to_0_360() {
    assert!(hsl_equal(
        &Hsl {
            h: -10.0,
            s: 0.4,
            l: 0.5,
            opacity: 1.0
        },
        -10.0,
        0.4,
        0.5,
        1.0
    )); // d3-color/test/hsl-test.js:79:3
    assert!(hsl_equal(
        &Hsl {
            h: 0.0,
            s: 0.4,
            l: 0.5,
            opacity: 1.0
        },
        0.0,
        0.4,
        0.5,
        1.0
    )); // d3-color/test/hsl-test.js:80:3
    assert!(hsl_equal(
        &Hsl {
            h: 360.0,
            s: 0.4,
            l: 0.5,
            opacity: 1.0
        },
        360.0,
        0.4,
        0.5,
        1.0
    )); // d3-color/test/hsl-test.js:81:3
    assert!(hsl_equal(
        &Hsl {
            h: 370.0,
            s: 0.4,
            l: 0.5,
            opacity: 1.0
        },
        370.0,
        0.4,
        0.5,
        1.0
    )); // d3-color/test/hsl-test.js:82:3
}

// d3-color/test/hsl-test.js: hsl(h, s, l) does not clamp s and l channel values to [0,1]
#[test]
fn hsl_h_s_l_does_not_clamp_s_and_l_channel_values_to_0_1() {
    assert!(hsl_equal(
        &Hsl {
            h: 120.0,
            s: -0.1,
            l: 0.5,
            opacity: 1.0
        },
        120.0,
        -0.1,
        0.5,
        1.0
    )); // d3-color/test/hsl-test.js:86:3
    assert!(hsl_equal(
        &Hsl {
            h: 120.0,
            s: 1.1,
            l: 0.5,
            opacity: 1.0
        },
        120.0,
        1.1,
        0.5,
        1.0
    )); // d3-color/test/hsl-test.js:87:3
    assert!(hsl_equal(
        &Hsl {
            h: 120.0,
            s: 0.2,
            l: -0.1,
            opacity: 1.0
        },
        120.0,
        0.2,
        -0.1,
        1.0
    )); // d3-color/test/hsl-test.js:88:3
    assert!(hsl_equal(
        &Hsl {
            h: 120.0,
            s: 0.2,
            l: 1.1,
            opacity: 1.0
        },
        120.0,
        0.2,
        1.1,
        1.0
    )); // d3-color/test/hsl-test.js:89:3
}

// d3-color/test/hsl-test.js: hsl(h, s, l).clamp() clamps channel values
#[test]
fn hsl_h_s_l_clamp_clamps_channel_values() {
    assert!(hsl_equal(
        &Hsl {
            h: 120.0,
            s: -0.1,
            l: -0.2,
            opacity: 1.0
        }
        .clamp(),
        120.0,
        0.0,
        0.0,
        1.0
    )); // d3-color/test/hsl-test.js:93:3
    assert!(hsl_equal(
        &Hsl {
            h: 120.0,
            s: 1.1,
            l: 1.2,
            opacity: 1.0
        }
        .clamp(),
        120.0,
        1.0,
        1.0,
        1.0
    )); // d3-color/test/hsl-test.js:94:3
    assert!(hsl_equal(
        &Hsl {
            h: 120.0,
            s: 2.1,
            l: 2.2,
            opacity: 1.0
        }
        .clamp(),
        120.0,
        1.0,
        1.0,
        1.0
    )); // d3-color/test/hsl-test.js:95:3
    assert!(hsl_equal(
        &Hsl {
            h: 420.0,
            s: -0.1,
            l: -0.2,
            opacity: 1.0
        }
        .clamp(),
        60.0,
        0.0,
        0.0,
        1.0
    )); // d3-color/test/hsl-test.js:96:3
    assert!(hsl_equal(
        &Hsl {
            h: -420.0,
            s: -0.1,
            l: -0.2,
            opacity: 1.0
        }
        .clamp(),
        300.0,
        0.0,
        0.0,
        1.0
    )); // d3-color/test/hsl-test.js:97:3
    assert_eq!(
        Hsl {
            h: -420.0,
            s: -0.1,
            l: -0.2,
            opacity: f64::NAN
        }
        .clamp()
        .opacity,
        1.0
    ); // d3-color/test/hsl-test.js:98:3
    assert_eq!(
        Hsl {
            h: -420.0,
            s: -0.1,
            l: -0.2,
            opacity: 0.5
        }
        .clamp()
        .opacity,
        0.5
    ); // d3-color/test/hsl-test.js:99:3
    assert_eq!(
        Hsl {
            h: -420.0,
            s: -0.1,
            l: -0.2,
            opacity: -1.0
        }
        .clamp()
        .opacity,
        0.0
    ); // d3-color/test/hsl-test.js:100:3
    assert_eq!(
        Hsl {
            h: -420.0,
            s: -0.1,
            l: -0.2,
            opacity: 2.0
        }
        .clamp()
        .opacity,
        1.0
    ); // d3-color/test/hsl-test.js:101:3
}

// d3-color/test/hsl-test.js: hsl(h, s, l, opacity) does not clamp opacity to [0,1]
#[test]
fn hsl_h_s_l_opacity_does_not_clamp_opacity_to_0_1() {
    assert!(hsl_equal(
        &Hsl {
            h: 120.0,
            s: 0.1,
            l: 0.5,
            opacity: -0.2
        },
        120.0,
        0.1,
        0.5,
        -0.2
    )); // d3-color/test/hsl-test.js:105:3
    assert!(hsl_equal(
        &Hsl {
            h: 120.0,
            s: 0.9,
            l: 0.5,
            opacity: 1.2
        },
        120.0,
        0.9,
        0.5,
        1.2
    )); // d3-color/test/hsl-test.js:106:3
}

// d3-color/test/hsl-test.js: hsl(h, s, l) preserves explicit hue, even for grays
#[test]
fn hsl_h_s_l_preserves_explicit_hue_even_for_grays() {
    assert!(hsl_equal(
        &Hsl {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            opacity: 1.0
        },
        0.0,
        0.0,
        0.0,
        1.0
    )); // d3-color/test/hsl-test.js:131:3
    assert!(hsl_equal(
        &Hsl {
            h: 42.0,
            s: 0.0,
            l: 0.5,
            opacity: 1.0
        },
        42.0,
        0.0,
        0.5,
        1.0
    )); // d3-color/test/hsl-test.js:132:3
    assert!(hsl_equal(
        &Hsl {
            h: 118.0,
            s: 0.0,
            l: 1.0,
            opacity: 1.0
        },
        118.0,
        0.0,
        1.0,
        1.0
    )); // d3-color/test/hsl-test.js:133:3
}

// d3-color/test/hsl-test.js: hsl(h, s, l) preserves explicit saturation, even for white or black
#[test]
fn hsl_h_s_l_preserves_explicit_saturation_even_for_white_or_black() {
    assert!(hsl_equal(
        &Hsl {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            opacity: 1.0
        },
        0.0,
        0.0,
        0.0,
        1.0
    )); // d3-color/test/hsl-test.js:137:3
    assert!(hsl_equal(
        &Hsl {
            h: 0.0,
            s: 0.18,
            l: 0.0,
            opacity: 1.0
        },
        0.0,
        0.18,
        0.0,
        1.0
    )); // d3-color/test/hsl-test.js:138:3
    assert!(hsl_equal(
        &Hsl {
            h: 0.0,
            s: 0.42,
            l: 1.0,
            opacity: 1.0
        },
        0.0,
        0.42,
        1.0,
        1.0
    )); // d3-color/test/hsl-test.js:139:3
    assert!(hsl_equal(
        &Hsl {
            h: 0.0,
            s: 1.0,
            l: 1.0,
            opacity: 1.0
        },
        0.0,
        1.0,
        1.0,
        1.0
    )); // d3-color/test/hsl-test.js:140:3
}

// d3-color/test/hsl-test.js: hsl(format) parses the specified format and converts to HSL
#[test]
fn hsl_format_parses_the_specified_format_and_converts_to_hsl() {
    assert!(hsl_equal(
        &"#abcdef".parse::<Hsl>().expect("a CSS color"),
        210.0,
        0.68,
        0.8039215,
        1.0
    )); // d3-color/test/hsl-test.js:144:3
    assert!(hsl_equal(
        &"#abc".parse::<Hsl>().expect("a CSS color"),
        210.0,
        0.25,
        0.733333333,
        1.0
    )); // d3-color/test/hsl-test.js:145:3
    assert!(hsl_equal(
        &"rgb(12, 34, 56)".parse::<Hsl>().expect("a CSS color"),
        210.0,
        0.647058,
        0.1333333,
        1.0
    )); // d3-color/test/hsl-test.js:146:3
    assert!(hsl_equal(
        &"rgb(12%, 34%, 56%)".parse::<Hsl>().expect("a CSS color"),
        210.0,
        0.647058,
        0.34,
        1.0
    )); // d3-color/test/hsl-test.js:147:3
    assert!(hsl_equal(
        &"hsl(60,100%,20%)".parse::<Hsl>().expect("a CSS color"),
        60.0,
        1.0,
        0.2,
        1.0
    )); // d3-color/test/hsl-test.js:148:3
    assert!(hsl_equal(
        &"hsla(60,100%,20%,0.4)".parse::<Hsl>().expect("a CSS color"),
        60.0,
        1.0,
        0.2,
        0.4
    )); // d3-color/test/hsl-test.js:149:3
    assert!(hsl_equal(
        &"aliceblue".parse::<Hsl>().expect("a CSS color"),
        208.0,
        1.0,
        0.9705882,
        1.0
    )); // d3-color/test/hsl-test.js:150:3
    assert!(hsl_equal(
        &"transparent".parse::<Hsl>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        f64::NAN,
        0.0
    )); // d3-color/test/hsl-test.js:151:3
}

// d3-color/test/hsl-test.js: hsl(format) ignores the hue if the saturation is <= 0
#[test]
fn hsl_format_ignores_the_hue_if_the_saturation_is_0() {
    assert!(hsl_equal(
        &"hsl(120,0%,20%)".parse::<Hsl>().expect("a CSS color"),
        f64::NAN,
        0.0,
        0.2,
        1.0
    )); // d3-color/test/hsl-test.js:155:3
    assert!(hsl_equal(
        &"hsl(120,-10%,20%)".parse::<Hsl>().expect("a CSS color"),
        f64::NAN,
        -0.1,
        0.2,
        1.0
    )); // d3-color/test/hsl-test.js:156:3
}

// d3-color/test/hsl-test.js: hsl(format) ignores the hue and saturation if the lightness is <= 0 or >= 1
#[test]
fn hsl_format_ignores_the_hue_and_saturation_if_the_lightness_is_0_or_1() {
    assert!(hsl_equal(
        &"hsl(120,20%,-10%)".parse::<Hsl>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        -0.1,
        1.0
    )); // d3-color/test/hsl-test.js:160:3
    assert!(hsl_equal(
        &"hsl(120,20%,0%)".parse::<Hsl>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        0.0,
        1.0
    )); // d3-color/test/hsl-test.js:161:3
    assert!(hsl_equal(
        &"hsl(120,20%,100%)".parse::<Hsl>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        1.0,
        1.0
    )); // d3-color/test/hsl-test.js:162:3
    assert!(hsl_equal(
        &"hsl(120,20%,120%)".parse::<Hsl>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        1.2,
        1.0
    )); // d3-color/test/hsl-test.js:163:3
}

// d3-color/test/hsl-test.js: hsl(format) ignores all channels if the alpha is <= 0
#[test]
fn hsl_format_ignores_all_channels_if_the_alpha_is_0() {
    assert!(hsl_equal(
        &"hsla(120,20%,10%,0)".parse::<Hsl>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        f64::NAN,
        0.0
    )); // d3-color/test/hsl-test.js:167:3
    assert!(hsl_equal(
        &"hsla(120,20%,10%,-0.1)"
            .parse::<Hsl>()
            .expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        f64::NAN,
        -0.1
    )); // d3-color/test/hsl-test.js:168:3
}

// d3-color/test/hsl-test.js: hsl(format) does not lose precision when parsing HSL formats
#[test]
fn hsl_format_does_not_lose_precision_when_parsing_hsl_formats() {
    assert!(hsl_equal(
        &"hsl(325,50%,40%)".parse::<Hsl>().expect("a CSS color"),
        325.0,
        0.5,
        0.4,
        1.0
    )); // d3-color/test/hsl-test.js:172:3
}

// d3-color/test/hsl-test.js: hsl(hsl) copies an HSL color
#[test]
fn hsl_hsl_copies_an_hsl_color() {
    let mut c1 = "hsla(120,30%,50%,0.4)".parse::<Hsl>().expect("a CSS color");
    let c2 = c1;
    assert!(hsl_equal(&c1, 120.0, 0.3, 0.5, 0.4)); // d3-color/test/hsl-test.js:182:3
    c1.h = {
        c1.s = {
            c1.l = {
                c1.opacity = 0.0;
                c1.opacity
            };
            c1.l
        };
        c1.s
    };
    assert!(hsl_equal(&c1, 0.0, 0.0, 0.0, 0.0)); // d3-color/test/hsl-test.js:184:3
    assert!(hsl_equal(&c2, 120.0, 0.3, 0.5, 0.4)); // d3-color/test/hsl-test.js:185:3
}

// d3-color/test/hsl-test.js: hsl(rgb) converts from RGB
#[test]
fn hsl_rgb_converts_from_rgb() {
    assert!(hsl_equal(
        &Hsl::from(Rgb::new(255.0, 0.0, 0.0).with_opacity(0.4)),
        0.0,
        1.0,
        0.5,
        0.4
    )); // d3-color/test/hsl-test.js:189:3
}

// d3-color/test/hsl-test.js: hsl(color) returns undefined hue and zero saturation for grays (but not white and black)
#[test]
fn hsl_color_returns_undefined_hue_and_zero_saturation_for_grays_but_not_white_and_black() {
    assert!(hsl_equal(
        &"gray".parse::<Hsl>().expect("a CSS color"),
        f64::NAN,
        0.0,
        0.5019608,
        1.0
    )); // d3-color/test/hsl-test.js:193:3
    assert!(hsl_equal(
        &"#ccc".parse::<Hsl>().expect("a CSS color"),
        f64::NAN,
        0.0,
        0.8,
        1.0
    )); // d3-color/test/hsl-test.js:194:3
    assert!(hsl_equal(
        &Hsl::from("gray".parse::<Rgb>().expect("a CSS color")),
        f64::NAN,
        0.0,
        0.5019608,
        1.0
    )); // d3-color/test/hsl-test.js:195:3
}

// d3-color/test/hsl-test.js: hsl(color) returns undefined hue and saturation for black and white
#[test]
fn hsl_color_returns_undefined_hue_and_saturation_for_black_and_white() {
    assert!(hsl_equal(
        &"black".parse::<Hsl>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        0.0,
        1.0
    )); // d3-color/test/hsl-test.js:199:3
    assert!(hsl_equal(
        &"#000".parse::<Hsl>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        0.0,
        1.0
    )); // d3-color/test/hsl-test.js:200:3
    assert!(hsl_equal(
        &"white".parse::<Hsl>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        1.0,
        1.0
    )); // d3-color/test/hsl-test.js:201:3
    assert!(hsl_equal(
        &"#fff".parse::<Hsl>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        1.0,
        1.0
    )); // d3-color/test/hsl-test.js:202:3
    assert!(hsl_equal(
        &Hsl::from("#fff".parse::<Rgb>().expect("a CSS color")),
        f64::NAN,
        f64::NAN,
        1.0,
        1.0
    )); // d3-color/test/hsl-test.js:203:3
}

// d3-color/test/hsl-test.js: hsl.displayable() returns true if the color is within the RGB gamut and the opacity is in [0,1]
#[test]
fn hsl_displayable_returns_true_if_the_color_is_within_the_rgb_gamut_and_the_opacity_is_in_0_1() {
    assert!("white".parse::<Hsl>().expect("a CSS color").displayable()); // d3-color/test/hsl-test.js:215:3
    assert!("red".parse::<Hsl>().expect("a CSS color").displayable()); // d3-color/test/hsl-test.js:216:3
    assert!("black".parse::<Hsl>().expect("a CSS color").displayable()); // d3-color/test/hsl-test.js:217:3
    assert!(
        Hsl {
            h: f64::NAN,
            s: f64::NAN,
            l: 1.0,
            opacity: 1.0
        }
        .displayable()
    ); // d3-color/test/hsl-test.js:219:3
    assert!(
        !Hsl {
            h: f64::NAN,
            s: f64::NAN,
            l: 1.5,
            opacity: 1.0
        }
        .displayable()
    ); // d3-color/test/hsl-test.js:220:3
    assert!(
        !Hsl {
            h: 120.0,
            s: -0.5,
            l: 0.0,
            opacity: 1.0
        }
        .displayable()
    ); // d3-color/test/hsl-test.js:221:3
    assert!(
        !Hsl {
            h: 120.0,
            s: 1.5,
            l: 0.0,
            opacity: 1.0
        }
        .displayable()
    ); // d3-color/test/hsl-test.js:222:3
    assert!(
        Hsl {
            h: 0.0,
            s: 1.0,
            l: 1.0,
            opacity: 0.0
        }
        .displayable()
    ); // d3-color/test/hsl-test.js:223:3
    assert!(
        Hsl {
            h: 0.0,
            s: 1.0,
            l: 1.0,
            opacity: 1.0
        }
        .displayable()
    ); // d3-color/test/hsl-test.js:224:3
    assert!(
        !Hsl {
            h: 0.0,
            s: 1.0,
            l: 1.0,
            opacity: -0.2
        }
        .displayable()
    ); // d3-color/test/hsl-test.js:225:3
    assert!(
        !Hsl {
            h: 0.0,
            s: 1.0,
            l: 1.0,
            opacity: 1.2
        }
        .displayable()
    ); // d3-color/test/hsl-test.js:226:3
}

// d3-color/test/hsl-test.js: hsl.brighter(k) returns a brighter color if k > 0
#[test]
fn hsl_brighter_k_returns_a_brighter_color_if_k_0() {
    let c = "rgba(165, 42, 42, 0.4)"
        .parse::<Hsl>()
        .expect("a CSS color");
    assert!(hsl_equal(&c.brighter(0.5), 0.0, 0.5942028, 0.4851222, 0.4)); // d3-color/test/hsl-test.js:231:3
    assert!(hsl_equal(&c.brighter(1.0), 0.0, 0.5942028, 0.5798319, 0.4)); // d3-color/test/hsl-test.js:232:3
    assert!(hsl_equal(&c.brighter(2.0), 0.0, 0.5942028, 0.8283313, 0.4)); // d3-color/test/hsl-test.js:233:3
}

// d3-color/test/hsl-test.js: hsl.brighter(k) returns a copy
#[test]
fn hsl_brighter_k_returns_a_copy() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Hsl>()
        .expect("a CSS color");
    let c2 = c1.brighter(1.0);
    assert!(hsl_equal(&c1, 207.272727, 0.44, 0.4901961, 0.4)); // d3-color/test/hsl-test.js:239:3
    assert!(hsl_equal(&c2, 207.272727, 0.44, 0.7002801, 0.4)); // d3-color/test/hsl-test.js:240:3
}

// d3-color/test/hsl-test.js: hsl.brighter() is equivalent to hsl.brighter(1)
#[test]
fn hsl_brighter_is_equivalent_to_hsl_brighter_1() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Hsl>()
        .expect("a CSS color");
    let c2 = c1.brighter(1.0);
    let c3 = c1.brighter(1.0);
    assert!(hsl_equal(&c2, c3.h, c3.s, c3.l, 0.4)); // d3-color/test/hsl-test.js:247:3
}

// d3-color/test/hsl-test.js: hsl.brighter(k) is equivalent to hsl.darker(-k)
#[test]
fn hsl_brighter_k_is_equivalent_to_hsl_darker_k() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Hsl>()
        .expect("a CSS color");
    let c2 = c1.brighter(1.5);
    let c3 = c1.darker(-1.5);
    assert!(hsl_equal(&c2, c3.h, c3.s, c3.l, 0.4)); // d3-color/test/hsl-test.js:254:3
}

// d3-color/test/hsl-test.js: hsl("black").brighter() still returns black
#[test]
fn hsl_black_brighter_still_returns_black() {
    let c1 = "black".parse::<Hsl>().expect("a CSS color");
    let c2 = c1.brighter(1.0);
    assert!(hsl_equal(&c1, f64::NAN, f64::NAN, 0.0, 1.0)); // d3-color/test/hsl-test.js:260:3
    assert!(hsl_equal(&c2, f64::NAN, f64::NAN, 0.0, 1.0)); // d3-color/test/hsl-test.js:261:3
}

// d3-color/test/hsl-test.js: hsl.darker(k) returns a darker color if k > 0
#[test]
fn hsl_darker_k_returns_a_darker_color_if_k_0() {
    let c = "rgba(165, 42, 42, 0.4)"
        .parse::<Hsl>()
        .expect("a CSS color");
    assert!(hsl_equal(&c.darker(0.5), 0.0, 0.5942029, 0.3395855, 0.4)); // d3-color/test/hsl-test.js:266:3
    assert!(hsl_equal(&c.darker(1.0), 0.0, 0.5942029, 0.2841176, 0.4)); // d3-color/test/hsl-test.js:267:3
    assert!(hsl_equal(&c.darker(2.0), 0.0, 0.5942029, 0.1988823, 0.4)); // d3-color/test/hsl-test.js:268:3
}

// d3-color/test/hsl-test.js: hsl.darker(k) returns a copy
#[test]
fn hsl_darker_k_returns_a_copy() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Hsl>()
        .expect("a CSS color");
    let c2 = c1.darker(1.0);
    assert!(hsl_equal(&c1, 207.272727, 0.44, 0.4901961, 0.4)); // d3-color/test/hsl-test.js:274:3
    assert!(hsl_equal(&c2, 207.272727, 0.44, 0.3431373, 0.4)); // d3-color/test/hsl-test.js:275:3
}

// d3-color/test/hsl-test.js: hsl.darker() is equivalent to hsl.darker(1)
#[test]
fn hsl_darker_is_equivalent_to_hsl_darker_1() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Hsl>()
        .expect("a CSS color");
    let c2 = c1.darker(1.0);
    let c3 = c1.darker(1.0);
    assert!(hsl_equal(&c2, c3.h, c3.s, c3.l, 0.4)); // d3-color/test/hsl-test.js:282:3
}

// d3-color/test/hsl-test.js: hsl.darker(k) is equivalent to hsl.brighter(-k)
#[test]
fn hsl_darker_k_is_equivalent_to_hsl_brighter_k() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Hsl>()
        .expect("a CSS color");
    let c2 = c1.darker(1.5);
    let c3 = c1.brighter(-1.5);
    assert!(hsl_equal(&c2, c3.h, c3.s, c3.l, 0.4)); // d3-color/test/hsl-test.js:289:3
}

// d3-color/test/hsl-test.js: hsl.rgb() converts to RGB
#[test]
fn hsl_rgb_converts_to_rgb() {
    let c = Hsl {
        h: 120.0,
        s: 0.3,
        l: 0.5,
        opacity: 0.4,
    };
    assert!(rgb_approx_equal(&Rgb::from(c), 89.0, 166.0, 89.0, 0.4)); // d3-color/test/hsl-test.js:294:3
}

// d3-color/test/hsl-test.js: hsl.copy(…) returns a new hsl with the specified channel values
#[test]
fn hsl_copy_returns_a_new_hsl_with_the_specified_channel_values() {
    let c = Hsl {
        h: 120.0,
        s: 0.3,
        l: 0.5,
        opacity: 0.4,
    };
    assert_eq!(c.format_hsl(), "hsla(120, 30%, 50%, 0.4)"); // d3-color/test/hsl-test.js:300:3
    assert_eq!(Hsl { opacity: 1.0, ..c }.format_hsl(), "hsl(120, 30%, 50%)"); // d3-color/test/hsl-test.js:301:3
    assert_eq!(Hsl { h: 20.0, ..c }.format_hsl(), "hsla(20, 30%, 50%, 0.4)"); // d3-color/test/hsl-test.js:302:3
    assert_eq!(
        Hsl {
            h: 20.0,
            s: 0.4,
            ..c
        }
        .format_hsl(),
        "hsla(20, 40%, 50%, 0.4)"
    ); // d3-color/test/hsl-test.js:303:3
}

// d3-color/test/lab-test.js: lab(…) exposes l, a and b channel values and opacity
#[test]
fn lab_exposes_l_a_and_b_channel_values_and_opacity() {
    assert!(lab_equal(
        &"rgba(170, 187, 204, 0.4)"
            .parse::<Lab>()
            .expect("a CSS color"),
        74.96879980931759,
        -3.398998724348956,
        -10.696507207853333,
        0.4
    )); // d3-color/test/lab-test.js:12:3
}

// d3-color/test/lab-test.js: lab.toString() converts to RGB and formats as rgb(…) or rgba(…)
#[test]
fn lab_tostring_converts_to_rgb_and_formats_as_rgb_or_rgba() {
    assert_eq!(
        "#abcdef".parse::<Lab>().expect("a CSS color").to_string(),
        "rgb(171, 205, 239)"
    ); // d3-color/test/lab-test.js:16:3
    assert_eq!(
        "moccasin".parse::<Lab>().expect("a CSS color").to_string(),
        "rgb(255, 228, 181)"
    ); // d3-color/test/lab-test.js:17:3
    assert_eq!(
        "hsl(60, 100%, 20%)"
            .parse::<Lab>()
            .expect("a CSS color")
            .to_string(),
        "rgb(102, 102, 0)"
    ); // d3-color/test/lab-test.js:18:3
    assert_eq!(
        "hsla(60, 100%, 20%, 0.4)"
            .parse::<Lab>()
            .expect("a CSS color")
            .to_string(),
        "rgba(102, 102, 0, 0.4)"
    ); // d3-color/test/lab-test.js:19:3
    assert_eq!(
        "rgb(12, 34, 56)"
            .parse::<Lab>()
            .expect("a CSS color")
            .to_string(),
        "rgb(12, 34, 56)"
    ); // d3-color/test/lab-test.js:20:3
    assert_eq!(
        Lab::from(Rgb::new(12.0, 34.0, 56.0)).to_string(),
        "rgb(12, 34, 56)"
    ); // d3-color/test/lab-test.js:21:3
    assert_eq!(
        Lab::from(Rgb::from(Hsl {
            h: 60.0,
            s: 1.0,
            l: 0.2,
            opacity: 1.0
        }))
        .to_string(),
        "rgb(102, 102, 0)"
    ); // d3-color/test/lab-test.js:22:3
    assert_eq!(
        Lab::from(Rgb::from(Hsl {
            h: 60.0,
            s: 1.0,
            l: 0.2,
            opacity: 0.4
        }))
        .to_string(),
        "rgba(102, 102, 0, 0.4)"
    ); // d3-color/test/lab-test.js:23:3
}

// d3-color/test/lab-test.js: lab.toString() reflects l, a and b channel values and opacity
#[test]
fn lab_tostring_reflects_l_a_and_b_channel_values_and_opacity() {
    let mut c = "#abc".parse::<Lab>().expect("a CSS color");
    c.l += 10.0;
    c.a -= 10.0;
    c.b += 10.0;
    c.opacity = 0.4;
    assert_eq!(c.to_string(), "rgba(184, 220, 213, 0.4)"); // d3-color/test/lab-test.js:29:3
}

// d3-color/test/lab-test.js: lab.toString() treats undefined channel values as 0
#[test]
fn lab_tostring_treats_undefined_channel_values_as_0() {
    assert_eq!(Lab::new(f64::NAN, 0.0, 0.0).to_string(), "rgb(0, 0, 0)"); // d3-color/test/lab-test.js:34:3
    assert_eq!(
        Lab::new(50.0, f64::NAN, 0.0).to_string(),
        "rgb(119, 119, 119)"
    ); // d3-color/test/lab-test.js:35:3
    assert_eq!(
        Lab::new(50.0, 0.0, f64::NAN).to_string(),
        "rgb(119, 119, 119)"
    ); // d3-color/test/lab-test.js:36:3
    assert_eq!(
        Lab::new(50.0, f64::NAN, f64::NAN).to_string(),
        "rgb(119, 119, 119)"
    ); // d3-color/test/lab-test.js:37:3
}

// d3-color/test/lab-test.js: lab.toString() treats undefined opacity as 1
#[test]
fn lab_tostring_treats_undefined_opacity_as_1() {
    let mut c = "#abc".parse::<Lab>().expect("a CSS color");
    c.opacity = f64::NAN;
    assert_eq!(c.to_string(), "rgb(170, 187, 204)"); // d3-color/test/lab-test.js:43:3
}

// d3-color/test/lab-test.js: lab(l, a, b) does not clamp l channel value
#[test]
fn lab_l_a_b_does_not_clamp_l_channel_value() {
    assert!(lab_equal(&Lab::new(-10.0, 1.0, 2.0), -10.0, 1.0, 2.0, 1.0)); // d3-color/test/lab-test.js:47:3
    assert!(lab_equal(&Lab::new(0.0, 1.0, 2.0), 0.0, 1.0, 2.0, 1.0)); // d3-color/test/lab-test.js:48:3
    assert!(lab_equal(&Lab::new(100.0, 1.0, 2.0), 100.0, 1.0, 2.0, 1.0)); // d3-color/test/lab-test.js:49:3
    assert!(lab_equal(&Lab::new(110.0, 1.0, 2.0), 110.0, 1.0, 2.0, 1.0)); // d3-color/test/lab-test.js:50:3
}

// d3-color/test/lab-test.js: lab(l, a, b, opacity) does not clamp opacity to [0,1]
#[test]
fn lab_l_a_b_opacity_does_not_clamp_opacity_to_0_1() {
    assert!(lab_equal(
        &Lab {
            opacity: -0.2,
            ..Lab::new(50.0, 10.0, 20.0)
        },
        50.0,
        10.0,
        20.0,
        -0.2
    )); // d3-color/test/lab-test.js:54:3
    assert!(lab_equal(
        &Lab {
            opacity: 1.2,
            ..Lab::new(50.0, 10.0, 20.0)
        },
        50.0,
        10.0,
        20.0,
        1.2
    )); // d3-color/test/lab-test.js:55:3
}

// d3-color/test/lab-test.js: lab(format) parses the specified format and converts to Lab
#[test]
fn lab_format_parses_the_specified_format_and_converts_to_lab() {
    assert!(lab_equal(
        &"#abcdef".parse::<Lab>().expect("a CSS color"),
        80.77135418262527,
        -5.957098328496224,
        -20.785782794739237,
        1.0
    )); // d3-color/test/lab-test.js:79:3
    assert!(lab_equal(
        &"#abc".parse::<Lab>().expect("a CSS color"),
        74.96879980931759,
        -3.398998724348956,
        -10.696507207853333,
        1.0
    )); // d3-color/test/lab-test.js:80:3
    assert!(lab_equal(
        &"rgb(12, 34, 56)".parse::<Lab>().expect("a CSS color"),
        12.404844123471648,
        -2.159950219712034,
        -17.168132391132946,
        1.0
    )); // d3-color/test/lab-test.js:81:3
    assert!(lab_equal(
        &"rgb(12%, 34%, 56%)".parse::<Lab>().expect("a CSS color"),
        35.48300043476593,
        -2.507637675606522,
        -36.95112983195855,
        1.0
    )); // d3-color/test/lab-test.js:82:3
    assert!(lab_equal(
        &"rgba(12%, 34%, 56%, 0.4)"
            .parse::<Lab>()
            .expect("a CSS color"),
        35.48300043476593,
        -2.507637675606522,
        -36.95112983195855,
        0.4
    )); // d3-color/test/lab-test.js:83:3
    assert!(lab_equal(
        &"hsl(60,100%,20%)".parse::<Lab>().expect("a CSS color"),
        41.97125732118659,
        -8.03835128380484,
        47.65411917854332,
        1.0
    )); // d3-color/test/lab-test.js:84:3
    assert!(lab_equal(
        &"hsla(60,100%,20%,0.4)".parse::<Lab>().expect("a CSS color"),
        41.97125732118659,
        -8.03835128380484,
        47.65411917854332,
        0.4
    )); // d3-color/test/lab-test.js:85:3
    assert!(lab_equal(
        &"aliceblue".parse::<Lab>().expect("a CSS color"),
        97.12294991108756,
        -1.773836604137824,
        -4.332680308569969,
        1.0
    )); // d3-color/test/lab-test.js:86:3
}

// d3-color/test/lab-test.js: lab(lab) copies a Lab color
#[test]
fn lab_lab_copies_a_lab_color() {
    let mut c1 = Lab {
        opacity: 0.4,
        ..Lab::new(50.0, 4.0, -5.0)
    };
    let c2 = c1;
    assert!(lab_equal(&c1, 50.0, 4.0, -5.0, 0.4)); // d3-color/test/lab-test.js:96:3
    c1.l = {
        c1.a = {
            c1.b = {
                c1.opacity = 0.0;
                c1.opacity
            };
            c1.b
        };
        c1.a
    };
    assert!(lab_equal(&c1, 0.0, 0.0, 0.0, 0.0)); // d3-color/test/lab-test.js:98:3
    assert!(lab_equal(&c2, 50.0, 4.0, -5.0, 0.4)); // d3-color/test/lab-test.js:99:3
}

// d3-color/test/lab-test.js: lab(hcl(lab)) doesn’t lose a and b channels if luminance is zero
#[test]
fn lab_hcl_lab_doesn_t_lose_a_and_b_channels_if_luminance_is_zero() {
    assert!(lab_equal(
        &Lab::from(Hcl::from(Lab::new(0.0, 10.0, 0.0))),
        0.0,
        10.0,
        0.0,
        1.0
    )); // d3-color/test/lab-test.js:103:3
}

// d3-color/test/lab-test.js: lab(rgb) converts from RGB
#[test]
fn lab_rgb_converts_from_rgb() {
    assert!(lab_equal(
        &Lab::from(Rgb::new(255.0, 0.0, 0.0).with_opacity(0.4)),
        54.29173376861782,
        80.8124553179771,
        69.88504032350531,
        0.4
    )); // d3-color/test/lab-test.js:107:3
}

// d3-color/test/lab-test.js: lab.brighter(k) returns a brighter color if k > 0
#[test]
fn lab_brighter_k_returns_a_brighter_color_if_k_0() {
    let c = "rgba(165, 42, 42, 0.4)"
        .parse::<Lab>()
        .expect("a CSS color");
    assert!(lab_equal(
        &c.brighter(0.5),
        47.149667346714935,
        50.388769337115,
        31.834059255569358,
        0.4
    )); // d3-color/test/lab-test.js:120:3
    assert!(lab_equal(
        &c.brighter(1.0),
        56.149667346714935,
        50.388769337115,
        31.834059255569358,
        0.4
    )); // d3-color/test/lab-test.js:121:3
    assert!(lab_equal(
        &c.brighter(2.0),
        74.14966734671493,
        50.388769337115,
        31.834059255569358,
        0.4
    )); // d3-color/test/lab-test.js:122:3
}

// d3-color/test/lab-test.js: lab.brighter(k) returns a copy
#[test]
fn lab_brighter_k_returns_a_copy() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Lab>()
        .expect("a CSS color");
    let c2 = c1.brighter(1.0);
    assert!(lab_equal(
        &c1,
        51.98624890550498,
        -8.362792037014344,
        -32.832699449697685,
        0.4
    )); // d3-color/test/lab-test.js:128:3
    assert!(lab_equal(
        &c2,
        69.98624890550498,
        -8.362792037014344,
        -32.832699449697685,
        0.4
    )); // d3-color/test/lab-test.js:129:3
}

// d3-color/test/lab-test.js: lab.brighter() is equivalent to lab.brighter(1)
#[test]
fn lab_brighter_is_equivalent_to_lab_brighter_1() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Lab>()
        .expect("a CSS color");
    let c2 = c1.brighter(1.0);
    let c3 = c1.brighter(1.0);
    assert!(lab_equal(&c2, c3.l, c3.a, c3.b, 0.4)); // d3-color/test/lab-test.js:136:3
}

// d3-color/test/lab-test.js: lab.brighter(k) is equivalent to lab.darker(-k)
#[test]
fn lab_brighter_k_is_equivalent_to_lab_darker_k() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Lab>()
        .expect("a CSS color");
    let c2 = c1.brighter(1.5);
    let c3 = c1.darker(-1.5);
    assert!(lab_equal(&c2, c3.l, c3.a, c3.b, 0.4)); // d3-color/test/lab-test.js:143:3
}

// d3-color/test/lab-test.js: lab.darker(k) returns a darker color if k > 0
#[test]
fn lab_darker_k_returns_a_darker_color_if_k_0() {
    let c = "rgba(165, 42, 42, 0.4)"
        .parse::<Lab>()
        .expect("a CSS color");
    assert!(lab_equal(
        &c.darker(0.5),
        29.149667346714935,
        50.388769337115,
        31.834059255569358,
        0.4
    )); // d3-color/test/lab-test.js:148:3
    assert!(lab_equal(
        &c.darker(1.0),
        20.149667346714935,
        50.388769337115,
        31.834059255569358,
        0.4
    )); // d3-color/test/lab-test.js:149:3
    assert!(lab_equal(
        &c.darker(2.0),
        2.149667346714935,
        50.388769337115,
        31.834059255569358,
        0.4
    )); // d3-color/test/lab-test.js:150:3
}

// d3-color/test/lab-test.js: lab.darker(k) returns a copy
#[test]
fn lab_darker_k_returns_a_copy() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Lab>()
        .expect("a CSS color");
    let c2 = c1.darker(1.0);
    assert!(lab_equal(
        &c1,
        51.98624890550498,
        -8.362792037014344,
        -32.832699449697685,
        0.4
    )); // d3-color/test/lab-test.js:156:3
    assert!(lab_equal(
        &c2,
        33.98624890550498,
        -8.362792037014344,
        -32.832699449697685,
        0.4
    )); // d3-color/test/lab-test.js:157:3
}

// d3-color/test/lab-test.js: lab.darker() is equivalent to lab.darker(1)
#[test]
fn lab_darker_is_equivalent_to_lab_darker_1() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Lab>()
        .expect("a CSS color");
    let c2 = c1.darker(1.0);
    let c3 = c1.darker(1.0);
    assert!(lab_equal(&c2, c3.l, c3.a, c3.b, 0.4)); // d3-color/test/lab-test.js:164:3
}

// d3-color/test/lab-test.js: lab.darker(k) is equivalent to lab.brighter(-k)
#[test]
fn lab_darker_k_is_equivalent_to_lab_brighter_k() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Lab>()
        .expect("a CSS color");
    let c2 = c1.darker(1.5);
    let c3 = c1.brighter(-1.5);
    assert!(lab_equal(&c2, c3.l, c3.a, c3.b, 0.4)); // d3-color/test/lab-test.js:171:3
}

// d3-color/test/lab-test.js: lab.rgb() converts to RGB
#[test]
fn lab_rgb_converts_to_rgb() {
    let c = Lab {
        opacity: 0.4,
        ..Lab::new(50.0, 4.0, -5.0)
    };
    assert!(rgb_approx_equal(&Rgb::from(c), 123.0, 117.0, 128.0, 0.4)); // d3-color/test/lab-test.js:176:3
}

// d3-color/test/hcl-test.js: hcl(…) exposes h, c, and l channel values
#[test]
fn hcl_exposes_h_c_and_l_channel_values() {
    assert!(hcl_equal(
        &"#abc".parse::<Hcl>().expect("a CSS color"),
        252.37145234745182,
        11.223567114593477,
        74.96879980931759,
        1.0
    )); // d3-color/test/hcl-test.js:12:3
}

// d3-color/test/hcl-test.js: hcl(…) returns defined hue and undefined chroma for black and white
#[test]
fn hcl_returns_defined_hue_and_undefined_chroma_for_black_and_white() {
    assert!(hcl_equal(
        &"black".parse::<Hcl>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        0.0,
        1.0
    )); // d3-color/test/hcl-test.js:16:3
    assert!(hcl_equal(
        &"#000".parse::<Hcl>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        0.0,
        1.0
    )); // d3-color/test/hcl-test.js:17:3
    assert!(hcl_equal(
        &Hcl::from("#000".parse::<Lab>().expect("a CSS color")),
        f64::NAN,
        f64::NAN,
        0.0,
        1.0
    )); // d3-color/test/hcl-test.js:18:3
    assert!(hcl_equal(
        &"white".parse::<Hcl>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        100.0,
        1.0
    )); // d3-color/test/hcl-test.js:19:3
    assert!(hcl_equal(
        &"#fff".parse::<Hcl>().expect("a CSS color"),
        f64::NAN,
        f64::NAN,
        100.0,
        1.0
    )); // d3-color/test/hcl-test.js:20:3
    assert!(hcl_equal(
        &Hcl::from("#fff".parse::<Lab>().expect("a CSS color")),
        f64::NAN,
        f64::NAN,
        100.0,
        1.0
    )); // d3-color/test/hcl-test.js:21:3
}

// d3-color/test/hcl-test.js: hcl(…) returns undefined hue and zero chroma for gray
#[test]
fn hcl_returns_undefined_hue_and_zero_chroma_for_gray() {
    assert!(hcl_equal(
        &"gray".parse::<Hcl>().expect("a CSS color"),
        f64::NAN,
        0.0,
        53.585013,
        1.0
    )); // d3-color/test/hcl-test.js:25:3
    assert!(hcl_equal(
        &Hcl::from("gray".parse::<Lab>().expect("a CSS color")),
        f64::NAN,
        0.0,
        53.585013,
        1.0
    )); // d3-color/test/hcl-test.js:26:3
}

// d3-color/test/hcl-test.js: hcl.toString() converts to RGB and formats as hexadecimal
#[test]
fn hcl_tostring_converts_to_rgb_and_formats_as_hexadecimal() {
    assert_eq!(
        "#abcdef".parse::<Hcl>().expect("a CSS color").to_string(),
        "rgb(171, 205, 239)"
    ); // d3-color/test/hcl-test.js:30:3
    assert_eq!(
        "moccasin".parse::<Hcl>().expect("a CSS color").to_string(),
        "rgb(255, 228, 181)"
    ); // d3-color/test/hcl-test.js:31:3
    assert_eq!(
        "hsl(60, 100%, 20%)"
            .parse::<Hcl>()
            .expect("a CSS color")
            .to_string(),
        "rgb(102, 102, 0)"
    ); // d3-color/test/hcl-test.js:32:3
    assert_eq!(
        "rgb(12, 34, 56)"
            .parse::<Hcl>()
            .expect("a CSS color")
            .to_string(),
        "rgb(12, 34, 56)"
    ); // d3-color/test/hcl-test.js:33:3
    assert_eq!(
        Hcl::from(Rgb::new(12.0, 34.0, 56.0)).to_string(),
        "rgb(12, 34, 56)"
    ); // d3-color/test/hcl-test.js:34:3
    assert_eq!(
        Hcl::from(Rgb::from(Hsl {
            h: 60.0,
            s: 1.0,
            l: 0.2,
            opacity: 1.0
        }))
        .to_string(),
        "rgb(102, 102, 0)"
    ); // d3-color/test/hcl-test.js:35:3
}

// d3-color/test/hcl-test.js: hcl.toString() reflects h, c and l channel values
#[test]
fn hcl_tostring_reflects_h_c_and_l_channel_values() {
    let mut c = "#abc".parse::<Hcl>().expect("a CSS color");
    c.h += 10.0;
    c.c += 1.0;
    c.l -= 1.0;
    assert_eq!(c.to_string(), "rgb(170, 183, 204)"); // d3-color/test/hcl-test.js:41:3
}

// d3-color/test/hcl-test.js: hcl.toString() treats undefined opacity as 1
#[test]
fn hcl_tostring_treats_undefined_opacity_as_1() {
    let mut c = "#abc".parse::<Hcl>().expect("a CSS color");
    c.opacity = f64::NAN;
    assert_eq!(c.to_string(), "rgb(170, 187, 204)"); // d3-color/test/hcl-test.js:47:3
}

// d3-color/test/hcl-test.js: hcl.toString() treats undefined channel values as 0
#[test]
fn hcl_tostring_treats_undefined_channel_values_as_0() {
    assert_eq!(
        "#000".parse::<Hcl>().expect("a CSS color").to_string(),
        "rgb(0, 0, 0)"
    ); // d3-color/test/hcl-test.js:52:3
    assert_eq!(
        "#ccc".parse::<Hcl>().expect("a CSS color").to_string(),
        "rgb(204, 204, 204)"
    ); // d3-color/test/hcl-test.js:53:3
    assert_eq!(
        "#fff".parse::<Hcl>().expect("a CSS color").to_string(),
        "rgb(255, 255, 255)"
    ); // d3-color/test/hcl-test.js:54:3
    assert_eq!(
        Hcl::new(f64::NAN, 20.0, 40.0).to_string(),
        "rgb(94, 94, 94)"
    ); // d3-color/test/hcl-test.js:55:3
    assert_eq!(
        Hcl::new(120.0, f64::NAN, 40.0).to_string(),
        "rgb(94, 94, 94)"
    ); // d3-color/test/hcl-test.js:56:3
    assert_eq!(Hcl::new(0.0, f64::NAN, 40.0).to_string(), "rgb(94, 94, 94)"); // d3-color/test/hcl-test.js:57:3
    assert_eq!(Hcl::new(120.0, 50.0, f64::NAN).to_string(), "rgb(0, 0, 0)"); // d3-color/test/hcl-test.js:58:3
    assert_eq!(Hcl::new(0.0, 50.0, f64::NAN).to_string(), "rgb(0, 0, 0)"); // d3-color/test/hcl-test.js:59:3
    assert_eq!(Hcl::new(120.0, 0.0, f64::NAN).to_string(), "rgb(0, 0, 0)"); // d3-color/test/hcl-test.js:60:3
}

// d3-color/test/hcl-test.js: hcl(yellow) is displayable
#[test]
fn hcl_yellow_is_displayable() {
    assert!(Rgb::from("yellow".parse::<Hcl>().expect("a CSS color")).displayable()); // d3-color/test/hcl-test.js:64:3
    assert_eq!(
        "yellow".parse::<Hcl>().expect("a CSS color").to_string(),
        "rgb(255, 255, 0)"
    ); // d3-color/test/hcl-test.js:65:3
}

// d3-color/test/hcl-test.js: hcl(h, c, l) does not wrap hue to [0,360)
#[test]
fn hcl_h_c_l_does_not_wrap_hue_to_0_360() {
    assert!(hcl_equal(
        &Hcl::new(-10.0, 40.0, 50.0),
        -10.0,
        40.0,
        50.0,
        1.0
    )); // d3-color/test/hcl-test.js:69:3
    assert!(hcl_equal(&Hcl::new(0.0, 40.0, 50.0), 0.0, 40.0, 50.0, 1.0)); // d3-color/test/hcl-test.js:70:3
    assert!(hcl_equal(
        &Hcl::new(360.0, 40.0, 50.0),
        360.0,
        40.0,
        50.0,
        1.0
    )); // d3-color/test/hcl-test.js:71:3
    assert!(hcl_equal(
        &Hcl::new(370.0, 40.0, 50.0),
        370.0,
        40.0,
        50.0,
        1.0
    )); // d3-color/test/hcl-test.js:72:3
}

// d3-color/test/hcl-test.js: hcl(h, c, l) does not clamp l channel value
#[test]
fn hcl_h_c_l_does_not_clamp_l_channel_value() {
    assert!(hcl_equal(
        &Hcl::new(120.0, 20.0, -10.0),
        120.0,
        20.0,
        -10.0,
        1.0
    )); // d3-color/test/hcl-test.js:76:3
    assert!(hcl_equal(
        &Hcl::new(120.0, 20.0, 0.0),
        120.0,
        20.0,
        0.0,
        1.0
    )); // d3-color/test/hcl-test.js:77:3
    assert!(hcl_equal(
        &Hcl::new(120.0, 20.0, 100.0),
        120.0,
        20.0,
        100.0,
        1.0
    )); // d3-color/test/hcl-test.js:78:3
    assert!(hcl_equal(
        &Hcl::new(120.0, 20.0, 110.0),
        120.0,
        20.0,
        110.0,
        1.0
    )); // d3-color/test/hcl-test.js:79:3
}

// d3-color/test/hcl-test.js: hcl(h, c, l, opacity) does not clamp opacity to [0,1]
#[test]
fn hcl_h_c_l_opacity_does_not_clamp_opacity_to_0_1() {
    assert!(hcl_equal(
        &Hcl {
            opacity: -0.2,
            ..Hcl::new(120.0, 20.0, 100.0)
        },
        120.0,
        20.0,
        100.0,
        -0.2
    )); // d3-color/test/hcl-test.js:83:3
    assert!(hcl_equal(
        &Hcl {
            opacity: 1.2,
            ..Hcl::new(120.0, 20.0, 110.0)
        },
        120.0,
        20.0,
        110.0,
        1.2
    )); // d3-color/test/hcl-test.js:84:3
}

// d3-color/test/hcl-test.js: hcl(format) parses the specified format and converts to HCL
#[test]
fn hcl_format_parses_the_specified_format_and_converts_to_hcl() {
    assert!(hcl_equal(
        &"#abcdef".parse::<Hcl>().expect("a CSS color"),
        254.0079700170605,
        21.62257586147983,
        80.77135418262527,
        1.0
    )); // d3-color/test/hcl-test.js:108:3
    assert!(hcl_equal(
        &"#abc".parse::<Hcl>().expect("a CSS color"),
        252.37145234745182,
        11.223567114593477,
        74.96879980931759,
        1.0
    )); // d3-color/test/hcl-test.js:109:3
    assert!(hcl_equal(
        &"rgb(12, 34, 56)".parse::<Hcl>().expect("a CSS color"),
        262.8292023352897,
        17.30347233219686,
        12.404844123471648,
        1.0
    )); // d3-color/test/hcl-test.js:110:3
    assert!(hcl_equal(
        &"rgb(12%, 34%, 56%)".parse::<Hcl>().expect("a CSS color"),
        266.117653326772,
        37.03612078188506,
        35.48300043476593,
        1.0
    )); // d3-color/test/hcl-test.js:111:3
    assert!(hcl_equal(
        &"rgba(12%, 34%, 56%, 0.4)"
            .parse::<Hcl>()
            .expect("a CSS color"),
        266.117653326772,
        37.03612078188506,
        35.48300043476593,
        0.4
    )); // d3-color/test/hcl-test.js:112:3
    assert!(hcl_equal(
        &"hsl(60,100%,20%)".parse::<Hcl>().expect("a CSS color"),
        99.57458688693686,
        48.327323183108916,
        41.97125732118659,
        1.0
    )); // d3-color/test/hcl-test.js:113:3
    assert!(hcl_equal(
        &"hsla(60,100%,20%,0.4)".parse::<Hcl>().expect("a CSS color"),
        99.57458688693686,
        48.327323183108916,
        41.97125732118659,
        0.4
    )); // d3-color/test/hcl-test.js:114:3
    assert!(hcl_equal(
        &"aliceblue".parse::<Hcl>().expect("a CSS color"),
        247.7353849904697,
        4.681732046417135,
        97.12294991108756,
        1.0
    )); // d3-color/test/hcl-test.js:115:3
}

// d3-color/test/hcl-test.js: hcl(hcl) copies an HCL color
#[test]
fn hcl_hcl_copies_an_hcl_color() {
    let mut c1 = Hcl {
        opacity: 0.4,
        ..Hcl::new(120.0, 30.0, 50.0)
    };
    let c2 = c1;
    assert!(hcl_equal(&c1, 120.0, 30.0, 50.0, 0.4)); // d3-color/test/hcl-test.js:125:3
    c1.h = {
        c1.c = {
            c1.l = {
                c1.opacity = 0.0;
                c1.opacity
            };
            c1.l
        };
        c1.c
    };
    assert!(hcl_equal(&c1, 0.0, 0.0, 0.0, 0.0)); // d3-color/test/hcl-test.js:127:3
    assert!(hcl_equal(&c2, 120.0, 30.0, 50.0, 0.4)); // d3-color/test/hcl-test.js:128:3
}

// d3-color/test/hcl-test.js: hcl(lab) returns h = NaN if a and b are zero
#[test]
fn hcl_lab_returns_h_nan_if_a_and_b_are_zero() {
    assert!(hcl_equal(
        &Hcl::from(Lab::new(0.0, 0.0, 0.0)),
        f64::NAN,
        f64::NAN,
        0.0,
        1.0
    )); // d3-color/test/hcl-test.js:132:3
    assert!(hcl_equal(
        &Hcl::from(Lab::new(50.0, 0.0, 0.0)),
        f64::NAN,
        0.0,
        50.0,
        1.0
    )); // d3-color/test/hcl-test.js:133:3
    assert!(hcl_equal(
        &Hcl::from(Lab::new(100.0, 0.0, 0.0)),
        f64::NAN,
        f64::NAN,
        100.0,
        1.0
    )); // d3-color/test/hcl-test.js:134:3
    assert!(hcl_equal(
        &Hcl::from(Lab::new(0.0, 10.0, 0.0)),
        0.0,
        10.0,
        0.0,
        1.0
    )); // d3-color/test/hcl-test.js:135:3
    assert!(hcl_equal(
        &Hcl::from(Lab::new(50.0, 10.0, 0.0)),
        0.0,
        10.0,
        50.0,
        1.0
    )); // d3-color/test/hcl-test.js:136:3
    assert!(hcl_equal(
        &Hcl::from(Lab::new(100.0, 10.0, 0.0)),
        0.0,
        10.0,
        100.0,
        1.0
    )); // d3-color/test/hcl-test.js:137:3
    assert!(hcl_equal(
        &Hcl::from(Lab::new(0.0, 0.0, 10.0)),
        90.0,
        10.0,
        0.0,
        1.0
    )); // d3-color/test/hcl-test.js:138:3
    assert!(hcl_equal(
        &Hcl::from(Lab::new(50.0, 0.0, 10.0)),
        90.0,
        10.0,
        50.0,
        1.0
    )); // d3-color/test/hcl-test.js:139:3
    assert!(hcl_equal(
        &Hcl::from(Lab::new(100.0, 0.0, 10.0)),
        90.0,
        10.0,
        100.0,
        1.0
    )); // d3-color/test/hcl-test.js:140:3
}

// d3-color/test/hcl-test.js: hcl(rgb) converts from RGB
#[test]
fn hcl_rgb_converts_from_rgb() {
    assert!(hcl_equal(
        &Hcl::from(Rgb::new(255.0, 0.0, 0.0).with_opacity(0.4)),
        40.85261277607024,
        106.83899941284552,
        54.29173376861782,
        0.4
    )); // d3-color/test/hcl-test.js:144:3
}

// d3-color/test/hcl-test.js: hcl.brighter(k) returns a brighter color if k > 0
#[test]
fn hcl_brighter_k_returns_a_brighter_color_if_k_0() {
    let c = "rgba(165, 42, 42, 0.4)"
        .parse::<Hcl>()
        .expect("a CSS color");
    assert!(hcl_equal(
        &c.brighter(0.5),
        32.28342524928155,
        59.60231039142763,
        47.149667346714935,
        0.4
    )); // d3-color/test/hcl-test.js:157:3
    assert!(hcl_equal(
        &c.brighter(1.0),
        32.28342524928155,
        59.60231039142763,
        56.149667346714935,
        0.4
    )); // d3-color/test/hcl-test.js:158:3
    assert!(hcl_equal(
        &c.brighter(2.0),
        32.28342524928155,
        59.60231039142763,
        74.14966734671493,
        0.4
    )); // d3-color/test/hcl-test.js:159:3
}

// d3-color/test/hcl-test.js: hcl.brighter(k) returns a copy
#[test]
fn hcl_brighter_k_returns_a_copy() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Hcl>()
        .expect("a CSS color");
    let c2 = c1.brighter(1.0);
    assert!(hcl_equal(
        &c1,
        255.71009124439382,
        33.88100417355615,
        51.98624890550498,
        0.4
    )); // d3-color/test/hcl-test.js:165:3
    assert!(hcl_equal(
        &c2,
        255.71009124439382,
        33.88100417355615,
        69.98624890550498,
        0.4
    )); // d3-color/test/hcl-test.js:166:3
}

// d3-color/test/hcl-test.js: hcl.brighter() is equivalent to hcl.brighter(1)
#[test]
fn hcl_brighter_is_equivalent_to_hcl_brighter_1() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Hcl>()
        .expect("a CSS color");
    let c2 = c1.brighter(1.0);
    let c3 = c1.brighter(1.0);
    assert!(hcl_equal(&c2, c3.h, c3.c, c3.l, 0.4)); // d3-color/test/hcl-test.js:173:3
}

// d3-color/test/hcl-test.js: hcl.brighter(k) is equivalent to hcl.darker(-k)
#[test]
fn hcl_brighter_k_is_equivalent_to_hcl_darker_k() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Hcl>()
        .expect("a CSS color");
    let c2 = c1.brighter(1.5);
    let c3 = c1.darker(-1.5);
    assert!(hcl_equal(&c2, c3.h, c3.c, c3.l, 0.4)); // d3-color/test/hcl-test.js:180:3
}

// d3-color/test/hcl-test.js: hcl.darker(k) returns a darker color if k > 0
#[test]
fn hcl_darker_k_returns_a_darker_color_if_k_0() {
    let c = "rgba(165, 42, 42, 0.4)"
        .parse::<Hcl>()
        .expect("a CSS color");
    assert!(hcl_equal(
        &c.darker(0.5),
        32.28342524928155,
        59.60231039142763,
        29.149667346714935,
        0.4
    )); // d3-color/test/hcl-test.js:185:3
    assert!(hcl_equal(
        &c.darker(1.0),
        32.28342524928155,
        59.60231039142763,
        20.149667346714935,
        0.4
    )); // d3-color/test/hcl-test.js:186:3
    assert!(hcl_equal(
        &c.darker(2.0),
        32.28342524928155,
        59.60231039142763,
        2.149667346714935,
        0.4
    )); // d3-color/test/hcl-test.js:187:3
}

// d3-color/test/hcl-test.js: hcl.darker(k) returns a copy
#[test]
fn hcl_darker_k_returns_a_copy() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Hcl>()
        .expect("a CSS color");
    let c2 = c1.darker(1.0);
    assert!(hcl_equal(
        &c1,
        255.71009124439382,
        33.88100417355615,
        51.98624890550498,
        0.4
    )); // d3-color/test/hcl-test.js:193:3
    assert!(hcl_equal(
        &c2,
        255.71009124439382,
        33.88100417355615,
        33.98624890550498,
        0.4
    )); // d3-color/test/hcl-test.js:194:3
}

// d3-color/test/hcl-test.js: hcl.darker() is equivalent to hcl.darker(1)
#[test]
fn hcl_darker_is_equivalent_to_hcl_darker_1() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Hcl>()
        .expect("a CSS color");
    let c2 = c1.darker(1.0);
    let c3 = c1.darker(1.0);
    assert!(hcl_equal(&c2, c3.h, c3.c, c3.l, 0.4)); // d3-color/test/hcl-test.js:201:3
}

// d3-color/test/hcl-test.js: hcl.darker(k) is equivalent to hcl.brighter(-k)
#[test]
fn hcl_darker_k_is_equivalent_to_hcl_brighter_k() {
    let c1 = "rgba(70, 130, 180, 0.4)"
        .parse::<Hcl>()
        .expect("a CSS color");
    let c2 = c1.darker(1.5);
    let c3 = c1.brighter(-1.5);
    assert!(hcl_equal(&c2, c3.h, c3.c, c3.l, 0.4)); // d3-color/test/hcl-test.js:208:3
}

// d3-color/test/hcl-test.js: hcl.rgb() converts to RGB
#[test]
fn hcl_rgb_converts_to_rgb() {
    let c = Hcl {
        opacity: 0.4,
        ..Hcl::new(120.0, 30.0, 50.0)
    };
    assert!(rgb_approx_equal(&Rgb::from(c), 105.0, 126.0, 73.0, 0.4)); // d3-color/test/hcl-test.js:213:3
}

// d3-color/test/gray-test.js: gray(l[, opacity]) is an alias for lab(l, 0, 0[, opacity])
#[test]
fn gray_l_opacity_is_an_alias_for_lab_l_0_0_opacity() {
    assert!(lab_equal(&Lab::gray(120.0), 120.0, 0.0, 0.0, 1.0)); // d3-color/test/gray-test.js:5:3
    assert!(lab_equal(
        &Lab {
            opacity: 0.5,
            ..Lab::gray(120.0)
        },
        120.0,
        0.0,
        0.0,
        0.5
    )); // d3-color/test/gray-test.js:6:3
}

// d3-color/test/lch-test.js: lch(color) is equivalent to hcl(color)
#[test]
fn lch_color_is_equivalent_to_hcl_color() {
    assert!(hcl_equal(
        &"#abc".parse::<Hcl>().expect("a CSS color"),
        252.37145234745182,
        11.223567114593477,
        74.96879980931759,
        1.0
    )); // d3-color/test/lch-test.js:5:3
    assert!(hcl_equal(
        &Hcl::from("#abc".parse::<Rgb>().expect("a CSS color")),
        252.37145234745182,
        11.223567114593477,
        74.96879980931759,
        1.0
    )); // d3-color/test/lch-test.js:6:3
}

// d3-color/test/lch-test.js: lch(l, c, h[, opacity]) is equivalent to hcl(h, c, l[, opacity])
#[test]
fn lch_l_c_h_opacity_is_equivalent_to_hcl_h_c_l_opacity() {
    assert!(hcl_equal(
        &Hcl::lch(74.0, 11.0, 252.0),
        252.0,
        11.0,
        74.0,
        1.0
    )); // d3-color/test/lch-test.js:10:3
    assert!(hcl_equal(
        &Hcl::lch(74.0, 11.0, 252.0),
        252.0,
        11.0,
        74.0,
        1.0
    )); // d3-color/test/lch-test.js:11:3
    assert!(hcl_equal(
        &Hcl {
            opacity: 0.5,
            ..Hcl::lch(74.0, 11.0, 252.0)
        },
        252.0,
        11.0,
        74.0,
        0.5
    )); // d3-color/test/lch-test.js:14:3
}
