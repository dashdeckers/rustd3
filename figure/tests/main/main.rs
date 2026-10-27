//! rustd3-figure's tests: the figure model, fitting and layout; marks and
//! thinning; drawing; the drawing list, its checks and its outputs; text.

mod draw;
mod figure;
mod fit;
mod layout;
mod legend;
mod list;
mod lower;
mod marks;
mod parts;
mod raster;
#[cfg(feature = "raster")]
mod resvg;
mod sink;
#[cfg(feature = "tiny-skia")]
mod skia;
mod support;
mod svg;
mod text;
mod thin;

// The example figures of rustd3-examples, compiled into these tests so
// that the snapshots and the thinning spot checks draw exactly what the
// examples draw.
#[path = "../../../dev/examples/src/scatter.rs"]
mod scatter;
#[path = "../../../dev/examples/src/trace.rs"]
mod trace;
