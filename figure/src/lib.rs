//! The figure layer: a `Figure` of panels, marks, axes, legends and
//! colorbars, laid out from text measured with typst and lowered into one
//! styled drawing list, which every backend renders, as SVG always and as
//! PNG, GIF and PDF behind their features.
//!
//! The list's geometry is kurbo's and its paint is peniko's; both crates
//! are re-exported so that callers name the same versions.

pub use kurbo;
pub use peniko;

pub mod list;
pub mod path;
pub mod raster;
pub mod svg;

mod pixels;
