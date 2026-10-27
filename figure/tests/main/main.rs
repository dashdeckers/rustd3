//! rustd3-figure's tests: the drawing list, its checks and its outputs.

mod list;
mod lower;
mod raster;
#[cfg(feature = "raster")]
mod resvg;
mod sink;
#[cfg(feature = "tiny-skia")]
mod skia;
mod svg;
