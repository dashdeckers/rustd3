//! The color schemes: the categorical ones' lengths, first and last
//! entries against d3, seaborn 0.13.2 and Okabe and Ito; every
//! sequential, diverging and cyclical scheme against d3-scale-chromatic
//! 3.1.0 (`tests/fixtures/schemes.json`).

use rustd3_core::color::Rgb;
use rustd3_core::scheme;
use serde_json::Value;

/// The fixture, read at compile time so the test needs no working directory.
const SCHEMES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/schemes.json"
));

/// A scheme function.
type Scheme = fn(f64) -> Rgb;

/// Every scheme function, under the fixture's name for it.
const FUNCTIONS: [(&str, Scheme); 38] = [
    ("blues", scheme::blues),
    ("br_bg", scheme::br_bg),
    ("bu_gn", scheme::bu_gn),
    ("bu_pu", scheme::bu_pu),
    ("cividis", scheme::cividis),
    ("cool", scheme::cool),
    ("cubehelix_default", scheme::cubehelix_default),
    ("gn_bu", scheme::gn_bu),
    ("greens", scheme::greens),
    ("greys", scheme::greys),
    ("inferno", scheme::inferno),
    ("magma", scheme::magma),
    ("or_rd", scheme::or_rd),
    ("oranges", scheme::oranges),
    ("pi_yg", scheme::pi_yg),
    ("plasma", scheme::plasma),
    ("pr_gn", scheme::pr_gn),
    ("pu_bu", scheme::pu_bu),
    ("pu_bu_gn", scheme::pu_bu_gn),
    ("pu_or", scheme::pu_or),
    ("pu_rd", scheme::pu_rd),
    ("purples", scheme::purples),
    ("rainbow", scheme::rainbow),
    ("rd_bu", scheme::rd_bu),
    ("rd_gy", scheme::rd_gy),
    ("rd_pu", scheme::rd_pu),
    ("rd_yl_bu", scheme::rd_yl_bu),
    ("rd_yl_gn", scheme::rd_yl_gn),
    ("reds", scheme::reds),
    ("sinebow", scheme::sinebow),
    ("spectral", scheme::spectral),
    ("turbo", scheme::turbo),
    ("viridis", scheme::viridis),
    ("warm", scheme::warm),
    ("yl_gn", scheme::yl_gn),
    ("yl_gn_bu", scheme::yl_gn_bu),
    ("yl_or_br", scheme::yl_or_br),
    ("yl_or_rd", scheme::yl_or_rd),
];

/// `colors` as `#rrggbb` strings.
fn hex(colors: &[Rgb]) -> Vec<String> {
    colors.iter().map(|c| c.format_hex()).collect()
}

/// The first and last entries of `colors`, as hex.
fn ends(colors: &[Rgb]) -> (String, String) {
    let all = hex(colors);
    (all[0].clone(), all[all.len() - 1].clone())
}

#[test]
fn each_scheme_has_its_length_and_ends() {
    let pair = |a: &str, b: &str| (a.to_owned(), b.to_owned());
    assert_eq!(ends(&scheme::CATEGORY10), pair("#1f77b4", "#17becf"));
    assert_eq!(ends(&scheme::OBSERVABLE10), pair("#4269d0", "#9498a0"));
    assert_eq!(ends(&scheme::DEEP), pair("#4c72b0", "#64b5cd"));
    assert_eq!(ends(&scheme::COLORBLIND), pair("#0173b2", "#56b4e9"));
    assert_eq!(ends(&scheme::OKABE_ITO), pair("#e69f00", "#000000"));
    assert_eq!(scheme::OKABE_ITO.len(), 8);
}

#[test]
fn deep_starts_blue_orange_green_then_red() {
    assert_eq!(hex(&scheme::DEEP[0..3]), ["#4c72b0", "#dd8452", "#55a868"]);
    assert_eq!(scheme::DEEP[3].format_hex(), "#c44e52");
}

#[test]
fn category10_is_d3s_and_every_entry_is_opaque() {
    assert_eq!(
        hex(&scheme::CATEGORY10),
        [
            "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd", "#8c564b", "#e377c2", "#7f7f7f",
            "#bcbd22", "#17becf"
        ]
    );
    let all = [
        &scheme::CATEGORY10[..],
        &scheme::OBSERVABLE10,
        &scheme::DEEP,
        &scheme::COLORBLIND,
        &scheme::OKABE_ITO,
    ];
    assert!(all.iter().flat_map(|s| s.iter()).all(|c| c.opacity == 1.0));
}

#[test]
fn colorbrewer_and_tableau_categorical_schemes_are_d3s() {
    let pair = |a: &str, b: &str| (a.to_owned(), b.to_owned());
    assert_eq!(ends(&scheme::ACCENT), pair("#7fc97f", "#666666"));
    assert_eq!(ends(&scheme::DARK2), pair("#1b9e77", "#666666"));
    assert_eq!(ends(&scheme::PAIRED), pair("#a6cee3", "#b15928"));
    assert_eq!(ends(&scheme::PASTEL1), pair("#fbb4ae", "#f2f2f2"));
    assert_eq!(ends(&scheme::PASTEL2), pair("#b3e2cd", "#cccccc"));
    assert_eq!(ends(&scheme::SET1), pair("#e41a1c", "#999999"));
    assert_eq!(ends(&scheme::SET2), pair("#66c2a5", "#b3b3b3"));
    assert_eq!(ends(&scheme::SET3), pair("#8dd3c7", "#ffed6f"));
    assert_eq!(ends(&scheme::TABLEAU10), pair("#4e79a7", "#bab0ab"));
    assert_eq!(
        [
            scheme::PAIRED.len(),
            scheme::SET3.len(),
            scheme::TABLEAU10.len()
        ],
        [12, 12, 10]
    );
}

#[test]
fn the_viridis_family_ramps_d3s_256_entry_tables() {
    assert_eq!(
        [0.0, 0.25, 0.5, 1.0].map(|t| scheme::viridis(t).format_hex()),
        ["#440154", "#3b528b", "#21918c", "#fde725"]
    );
    assert_eq!(scheme::magma(0.25).format_hex(), "#51127c");
    assert_eq!(scheme::inferno(0.5).format_hex(), "#bc3754");
    assert_eq!(scheme::plasma(0.5).format_hex(), "#cc4778");
    assert_eq!(scheme::viridis(-1.0), scheme::viridis(0.0));
    assert_eq!(scheme::viridis(2.0), scheme::viridis(255.0 / 256.0));
    // d3 returns undefined for NaN; the ramp reads entry 0.
    assert_eq!(scheme::viridis(f64::NAN), scheme::viridis(0.0));
}

#[test]
fn schemes_match_d3_at_11_samples() {
    let fixture: Value = serde_json::from_str(SCHEMES).expect("the fixture is JSON");
    let fixture = fixture.as_object().expect("an object of schemes");
    let names: Vec<&str> = fixture.keys().map(String::as_str).collect();
    assert_eq!(
        names,
        FUNCTIONS.map(|(name, _)| name),
        "one function per d3 scheme"
    );
    for (name, f) in FUNCTIONS {
        let expected = fixture[name].as_array().expect("11 samples");
        let ours: Vec<String> = (0..11)
            .map(|i| f(f64::from(i) / 10.0).to_string())
            .collect();
        assert_eq!(ours, *expected, "{name}");
    }
}

#[test]
fn spot_samples_of_each_kind_of_scheme() {
    assert_eq!(scheme::blues(0.5).to_string(), "rgb(109, 174, 213)");
    assert_eq!(scheme::rd_bu(0.25).to_string(), "rgb(228, 130, 104)");
    assert_eq!(scheme::turbo(0.5).to_string(), "rgb(149, 251, 81)");
    assert_eq!(scheme::cividis(0.5).to_string(), "rgb(127, 124, 117)");
    assert_eq!(scheme::sinebow(0.3).to_string(), "rgb(88, 252, 42)");
    assert_eq!(scheme::rainbow(0.3).to_string(), "rgb(255, 120, 71)");
    assert_eq!(scheme::warm(0.5).to_string(), "rgb(255, 94, 99)");
    assert_eq!(
        scheme::cubehelix_default(0.5).to_string(),
        "rgb(160, 121, 73)"
    );
}

#[test]
fn spline_and_formula_schemes_give_nan_channels_at_nan() {
    for f in [
        scheme::blues,
        scheme::rd_bu,
        scheme::turbo,
        scheme::sinebow,
        scheme::warm,
    ] {
        let c = f(f64::NAN);
        assert!(c.r.is_nan() && c.g.is_nan() && c.b.is_nan());
    }
    assert_eq!(scheme::blues(-1.0), scheme::blues(0.0));
    assert_eq!(scheme::turbo(2.0), scheme::turbo(1.0));
}
