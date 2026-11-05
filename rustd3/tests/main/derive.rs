//! `#[derive(Interpolate)]` through the facade's path, with the trait not
//! imported: `crate = "::rustd3"` names it `::rustd3::interpolate::Interpolate`.

#[derive(Debug, Clone, PartialEq, rustd3::interpolate::Interpolate)]
#[interpolate(crate = "::rustd3")]
struct Bar {
    y: f64,
    height: f64,
}

#[test]
fn the_derive_names_the_trait_through_the_facade() {
    let a = Bar {
        y: 0.0,
        height: 10.0,
    };
    let b = Bar {
        y: 40.0,
        height: 30.0,
    };
    let quarter = Bar {
        y: 10.0,
        height: 15.0,
    };
    assert_eq!(
        rustd3::interpolate::Interpolate::lerp(&a, &b, 0.25),
        quarter
    );
}
