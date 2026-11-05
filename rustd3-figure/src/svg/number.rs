//! Numbers as the SVG writer and the list dump print them.

use std::fmt::Write;

use kurbo::{Affine, BezPath, PathEl};

/// A coordinate in px: two places, trailing zeros and point trimmed, `-0`
/// printed as `0`.
pub(crate) fn px(v: f64) -> String {
    fixed(v, 2)
}

/// A coefficient, scale, offset or opacity: four places, trimmed alike.
pub(crate) fn unit(v: f64) -> String {
    fixed(v, 4)
}

/// A scale that may be small, such as a glyph run's size / 1000: six
/// places, trimmed alike.
pub(crate) fn fine(v: f64) -> String {
    fixed(v, 6)
}

/// `matrix(a b c d e f)`, coefficients through [`unit`].
pub(crate) fn matrix(t: Affine) -> String {
    let [a, b, c, d, e, f] = t.as_coeffs().map(unit);
    format!("matrix({a} {b} {c} {d} {e} {f})")
}

/// Path data with every coordinate formatted by `number`: `M0,0L10,0Z`.
pub(crate) fn path(p: &BezPath, number: fn(f64) -> String) -> String {
    let mut d = String::new();
    for el in p.elements() {
        let (letter, points) = match *el {
            PathEl::MoveTo(a) => ('M', vec![a]),
            PathEl::LineTo(a) => ('L', vec![a]),
            PathEl::QuadTo(a, b) => ('Q', vec![a, b]),
            PathEl::CurveTo(a, b, c) => ('C', vec![a, b, c]),
            PathEl::ClosePath => ('Z', Vec::new()),
        };
        d.push(letter);
        let numbers: Vec<String> = points
            .iter()
            .flat_map(|p| [number(p.x), number(p.y)])
            .collect();
        // Writing to a String cannot fail.
        let _ = write!(d, "{}", numbers.join(","));
    }
    d
}

/// `v` to `places` decimals, trailing zeros and point trimmed, `-0` as `0`.
fn fixed(v: f64, places: usize) -> String {
    let mut s = format!("{v:.places$}");
    if s.contains('.') {
        let trimmed = s.trim_end_matches('0').trim_end_matches('.').len();
        s.truncate(trimmed);
    }
    if s == "-0" {
        s.remove(0);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_have_two_places_trimmed() {
        assert_eq!(px(1.0 / 3.0), "0.33");
        assert_eq!(px(2.5), "2.5");
        assert_eq!(px(-0.001), "0");
        assert_eq!(px(10.0), "10");
        assert_eq!(px(-12.345), "-12.35");
        assert_eq!(px(1e6), "1000000");
    }

    #[test]
    fn coefficients_have_four_places_trimmed() {
        assert_eq!(unit(std::f64::consts::FRAC_1_SQRT_2), "0.7071");
        assert_eq!(unit(-0.00001), "0");
        assert_eq!(
            matrix(Affine::translate((40.0, 20.0))),
            "matrix(1 0 0 1 40 20)"
        );
    }

    #[test]
    fn path_data_joins_coordinates_with_commas() {
        let mut p = BezPath::new();
        p.move_to((0.0, 0.0));
        p.line_to((10.0, -0.5));
        p.curve_to((1.0, 2.0), (3.0, 4.0), (5.0, 6.0));
        p.close_path();
        assert_eq!(path(&p, px), "M0,0L10,-0.5C1,2,3,4,5,6Z");
    }
}
