//! d3-scale-chromatic's sequential schemes: ColorBrewer's (a basis spline
//! over the 9-stop row), turbo and cividis (d3's polynomials), and the
//! cubehelix ones.

use crate::color::{Cubehelix, Rgb};
use crate::interpolate::{self, basis_at};
use crate::js;
use crate::scheme::stops;

/// Defines `pub fn $name(t: f64) -> Rgb`, d3's `$alias`: a basis spline
/// over the ColorBrewer row `$stops`.
macro_rules! colorbrewer {
    ($($(#[doc = $doc:literal])* $alias:literal $name:ident $stops:ident;)*) => {$(
        $(#[doc = $doc])*
        #[doc(alias = $alias)]
        pub fn $name(t: f64) -> Rgb {
            basis_at(&stops::$stops, t)
        }
    )*};
}
pub(super) use colorbrewer;

colorbrewer! {
    /// d3's `interpolateBlues`: ColorBrewer's single-hue Blues.
    ///
    /// ```
    /// use rustd3_core::scheme;
    ///
    /// assert_eq!(scheme::blues(0.5).to_string(), "rgb(109, 174, 213)");
    /// ```
    "interpolateBlues" blues BLUES;
    /// d3's `interpolateGreens`: ColorBrewer's single-hue Greens.
    "interpolateGreens" greens GREENS;
    /// d3's `interpolateGreys`: ColorBrewer's single-hue Greys.
    "interpolateGreys" greys GREYS;
    /// d3's `interpolateOranges`: ColorBrewer's single-hue Oranges.
    "interpolateOranges" oranges ORANGES;
    /// d3's `interpolatePurples`: ColorBrewer's single-hue Purples.
    "interpolatePurples" purples PURPLES;
    /// d3's `interpolateReds`: ColorBrewer's single-hue Reds.
    "interpolateReds" reds REDS;
    /// d3's `interpolateBuGn`: ColorBrewer's multi-hue BuGn.
    "interpolateBuGn" bu_gn BU_GN;
    /// d3's `interpolateBuPu`: ColorBrewer's multi-hue BuPu.
    "interpolateBuPu" bu_pu BU_PU;
    /// d3's `interpolateGnBu`: ColorBrewer's multi-hue GnBu.
    "interpolateGnBu" gn_bu GN_BU;
    /// d3's `interpolateOrRd`: ColorBrewer's multi-hue OrRd.
    "interpolateOrRd" or_rd OR_RD;
    /// d3's `interpolatePuBuGn`: ColorBrewer's multi-hue PuBuGn.
    "interpolatePuBuGn" pu_bu_gn PU_BU_GN;
    /// d3's `interpolatePuBu`: ColorBrewer's multi-hue PuBu.
    "interpolatePuBu" pu_bu PU_BU;
    /// d3's `interpolatePuRd`: ColorBrewer's multi-hue PuRd.
    "interpolatePuRd" pu_rd PU_RD;
    /// d3's `interpolateRdPu`: ColorBrewer's multi-hue RdPu.
    "interpolateRdPu" rd_pu RD_PU;
    /// d3's `interpolateYlGnBu`: ColorBrewer's multi-hue YlGnBu.
    "interpolateYlGnBu" yl_gn_bu YL_GN_BU;
    /// d3's `interpolateYlGn`: ColorBrewer's multi-hue YlGn.
    "interpolateYlGn" yl_gn YL_GN;
    /// d3's `interpolateYlOrBr`: ColorBrewer's multi-hue YlOrBr.
    "interpolateYlOrBr" yl_or_br YL_OR_BR;
    /// d3's `interpolateYlOrRd`: ColorBrewer's multi-hue YlOrRd.
    "interpolateYlOrRd" yl_or_rd YL_OR_RD;
}

/// d3's `warm` ends: cubehelix(−100, 0.75, 0.35) to (80, 1.5, 0.8).
const WARM: [Cubehelix; 2] = [
    Cubehelix::new(-100.0, 0.75, 0.35),
    Cubehelix::new(80.0, 1.5, 0.8),
];

/// d3's `cool` ends: cubehelix(260, 0.75, 0.35) to (80, 1.5, 0.8).
const COOL: [Cubehelix; 2] = [
    Cubehelix::new(260.0, 0.75, 0.35),
    Cubehelix::new(80.0, 1.5, 0.8),
];

/// d3's `interpolateCubehelixDefault` ends: cubehelix(300, 0.5, 0) to
/// (−240, 0.5, 1).
const CUBEHELIX: [Cubehelix; 2] = [
    Cubehelix::new(300.0, 0.5, 0.0),
    Cubehelix::new(-240.0, 0.5, 1.0),
];

/// d3's `interpolateTurbo` (Mikhailov's turbo, d3's polynomial
/// approximation): `t` clamped to [0, 1], each channel a polynomial
/// rounded and clamped to 0..=255.
///
/// ```
/// use rustd3_core::scheme;
///
/// assert_eq!(scheme::turbo(0.5).to_string(), "rgb(149, 251, 81)");
/// ```
#[doc(alias = "interpolateTurbo")]
pub fn turbo(t: f64) -> Rgb {
    let t = unit(t);
    Rgb::new(
        channel(
            34.61 + t * (1172.33 - t * (10793.56 - t * (33300.12 - t * (38394.49 - t * 14825.05)))),
        ),
        channel(23.31 + t * (557.33 + t * (1225.33 - t * (3574.96 - t * (1073.77 + t * 707.56))))),
        channel(
            27.2 + t * (3211.1 - t * (15327.97 - t * (27814.0 - t * (22569.18 - t * 6838.66)))),
        ),
    )
}

/// d3's `interpolateCividis` (Nuñez, Anderton and Renslow, d3's
/// polynomial approximation): `t` clamped to [0, 1], each channel a
/// polynomial rounded and clamped to 0..=255.
///
/// ```
/// use rustd3_core::scheme;
///
/// assert_eq!(scheme::cividis(0.5).to_string(), "rgb(127, 124, 117)");
/// ```
#[doc(alias = "interpolateCividis")]
pub fn cividis(t: f64) -> Rgb {
    let t = unit(t);
    Rgb::new(
        channel(-4.54 - t * (35.34 - t * (2381.73 - t * (6402.7 - t * (7024.72 - t * 2710.57))))),
        channel(32.49 + t * (170.73 + t * (52.82 - t * (131.46 - t * (176.58 - t * 67.37))))),
        channel(81.24 + t * (442.36 - t * (2482.43 - t * (6167.24 - t * (6614.94 - t * 2475.67))))),
    )
}

/// d3's `interpolateWarm`: [`interpolate::cubehelix_long`] from
/// cubehelix(−100, 0.75, 0.35) to (80, 1.5, 0.8).
///
/// ```
/// use rustd3_core::scheme;
///
/// assert_eq!(scheme::warm(0.5).to_string(), "rgb(255, 94, 99)");
/// ```
#[doc(alias = "interpolateWarm")]
pub fn warm(t: f64) -> Rgb {
    interpolate::cubehelix_long(1.0, WARM[0], WARM[1])(t)
}

/// d3's `interpolateCool`: [`interpolate::cubehelix_long`] from
/// cubehelix(260, 0.75, 0.35) to (80, 1.5, 0.8).
#[doc(alias = "interpolateCool")]
pub fn cool(t: f64) -> Rgb {
    interpolate::cubehelix_long(1.0, COOL[0], COOL[1])(t)
}

/// d3's `interpolateCubehelixDefault`, Green's cubehelix:
/// [`interpolate::cubehelix_long`] from cubehelix(300, 0.5, 0) to
/// (−240, 0.5, 1).
///
/// ```
/// use rustd3_core::scheme;
///
/// assert_eq!(scheme::cubehelix_default(0.5).to_string(), "rgb(160, 121, 73)");
/// ```
#[doc(alias = "interpolateCubehelixDefault")]
pub fn cubehelix_default(t: f64) -> Rgb {
    interpolate::cubehelix_long(1.0, CUBEHELIX[0], CUBEHELIX[1])(t)
}

/// `t` clamped to [0, 1] as JavaScript's `Math.max(0, Math.min(1, t))`,
/// NaN kept.
fn unit(t: f64) -> f64 {
    js::max(0.0, js::min(1.0, t))
}

/// A polynomial's channel: rounded as JavaScript's `Math.round` and
/// clamped to 0..=255, NaN kept.
fn channel(v: f64) -> f64 {
    js::max(0.0, js::min(255.0, js::round(v)))
}
