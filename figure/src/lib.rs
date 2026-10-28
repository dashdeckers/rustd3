//! The figure layer: a `Figure` of panels, marks, axes, legends and
//! colorbars, laid out from text measured with typst and lowered into one
//! styled drawing list, which every backend renders, as SVG always and as
//! PNG, GIF and PDF behind their features.
//!
//! The list's geometry is kurbo's and its paint is peniko's; both crates
//! are re-exported so that callers name the same versions.

pub use kurbo;
pub use peniko;

pub mod axis;
pub mod brush;
pub mod check;
pub mod colorbar;
mod figure;
pub mod hatch;
pub mod hit;
pub mod layout;
pub mod legend;
pub mod list;
pub mod lower;
pub mod mark;
mod panel;
pub mod path;
mod pixels;
pub mod raster;
pub mod scale;
pub mod stats;
pub mod svg;
mod target;
pub mod text;
pub mod theme;
pub mod thin;

pub use figure::{Cache, Error, Figure, Fits, Held, Resolved};
pub use panel::Panel;
pub use target::Target;
pub use theme::settings::Theme;

/// What a figure's [`Cache`] reports.
pub mod cache {
    pub use crate::figure::cache::Stats;
}
