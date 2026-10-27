//! The color schemes: the categorical ones' lengths, first and last
//! entries against d3, seaborn 0.13.2 and Okabe and Ito; the viridis
//! family's ramps against d3-scale-chromatic 3.1.0.

use rustd3_core::color::Rgb;
use rustd3_core::scheme;

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
