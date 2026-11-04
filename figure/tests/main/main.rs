//! rustd3-figure's tests: the figure model, fitting and layout; marks and
//! thinning; drawing; the drawing list, its checks and its outputs; text.

#[cfg(feature = "fonts")]
mod check;
mod colorbar;
mod draw;
mod figure;
mod fit;
#[cfg(feature = "fonts")]
mod foreign;
#[cfg(all(feature = "gif", feature = "fonts"))]
mod gif;
mod handles;
mod hatch;
#[cfg(feature = "helpers")]
mod helpers;
mod hit;
#[cfg(feature = "fonts")]
mod hostile;
mod layout;
mod legend;
mod list;
mod locale;
mod lower;
mod marks;
mod parts;
#[cfg(feature = "pdf")]
mod pdf;
#[cfg(feature = "pdf")]
mod poppler;
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
mod threads;
mod visible;

// The example figures of rustd3-examples, compiled into these tests so
// that the snapshots and the thinning spot checks draw exactly what the
// examples draw.
#[path = "../../../dev/examples/src/scatter.rs"]
#[cfg_attr(
    not(all(feature = "png", feature = "fonts")),
    expect(dead_code, reason = "only the PNG agreement test reads SCALE")
)]
mod scatter;
#[path = "../../../benchkit/src/gallery/trace.rs"]
mod trace;
