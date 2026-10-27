//! The categorical schemes: lengths, first and last entries against d3,
//! seaborn 0.13.2 and Okabe and Ito.

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
