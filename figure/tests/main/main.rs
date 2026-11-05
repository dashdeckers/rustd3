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
// The wheel figure: its example draws benchkit's gallery figure.
#[cfg(all(feature = "fonts", feature = "pdf", feature = "raster"))]
mod wheels;

// The example figures, so the snapshots and the thinning spot checks draw
// exactly what the examples draw. The trace comes from benchkit's gallery
// and the scatter is compiled from its example.
use benchkit::gallery::trace;
#[path = "../../examples/scatter.rs"]
#[allow(
    dead_code,
    reason = "the example's `main`, and SCALE outside the PNG agreement test"
)]
mod scatter;
