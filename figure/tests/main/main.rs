//! rustd3-figure's tests: the figure model, fitting and layout; marks and
//! thinning; the drawing list, its checks and its outputs; text.

mod figure;
mod fit;
mod layout;
mod list;
mod lower;
mod marks;
mod raster;
#[cfg(feature = "raster")]
mod resvg;
mod sink;
#[cfg(feature = "tiny-skia")]
mod skia;
mod svg;
mod text;
mod thin;
