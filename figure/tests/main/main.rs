//! rustd3-figure's tests: the drawing list, its checks and its outputs.

mod list;
mod lower;
mod raster;
mod sink;
#[cfg(feature = "tiny-skia")]
mod skia;
mod svg;
