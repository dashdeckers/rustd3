//! d3-scale-chromatic's diverging schemes: ColorBrewer's, a basis spline
//! over the 11-stop row.

use crate::color::Rgb;
use crate::interpolate::basis_at;
use crate::scheme::sequential::colorbrewer;
use crate::scheme::stops;

colorbrewer! {
    /// d3's `interpolateBrBG`: ColorBrewer's diverging BrBG.
    "interpolateBrBG" br_bg BR_BG;
    /// d3's `interpolatePRGn`: ColorBrewer's diverging PRGn.
    "interpolatePRGn" pr_gn PR_GN;
    /// d3's `interpolatePiYG`: ColorBrewer's diverging PiYG.
    "interpolatePiYG" pi_yg PI_YG;
    /// d3's `interpolatePuOr`: ColorBrewer's diverging PuOr.
    "interpolatePuOr" pu_or PU_OR;
    /// d3's `interpolateRdBu`: ColorBrewer's diverging RdBu.
    ///
    /// ```
    /// use rustd3_core::scheme;
    ///
    /// assert_eq!(scheme::rd_bu(0.25).to_string(), "rgb(228, 130, 104)");
    /// ```
    "interpolateRdBu" rd_bu RD_BU;
    /// d3's `interpolateRdGy`: ColorBrewer's diverging RdGy.
    "interpolateRdGy" rd_gy RD_GY;
    /// d3's `interpolateRdYlBu`: ColorBrewer's diverging RdYlBu.
    "interpolateRdYlBu" rd_yl_bu RD_YL_BU;
    /// d3's `interpolateRdYlGn`: ColorBrewer's diverging RdYlGn.
    "interpolateRdYlGn" rd_yl_gn RD_YL_GN;
    /// d3's `interpolateSpectral`: ColorBrewer's diverging Spectral.
    "interpolateSpectral" spectral SPECTRAL;
}
