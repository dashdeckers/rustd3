//! rustd3-figure's tests: the figure model, fitting and layout; marks and
//! thinning; drawing; the drawing list, its checks and its outputs; text.

#[cfg(feature = "fonts")]
mod check;
mod colorbar;
mod draw;
mod figure;
mod fit;
mod handles;
mod hit;
mod layout;
mod legend;
mod list;
mod locale;
mod lower;
mod marks;
mod parts;
mod raster;
#[cfg(feature = "raster")]
mod resvg;
mod rotate;
mod sink;
#[cfg(feature = "tiny-skia")]
mod skia;
mod support;
mod svg;
mod text;
mod theme;
mod thin;
mod visible;

// The example figures of rustd3-examples, compiled into these tests so
// that the snapshots and the thinning spot checks draw exactly what the
// examples draw.
#[path = "../../../dev/examples/src/scatter.rs"]
#[cfg_attr(
    not(feature = "raster"),
    expect(dead_code, reason = "only the raster agreement test reads SCALE")
)]
mod scatter;
#[path = "../../../dev/examples/src/trace.rs"]
mod trace;
