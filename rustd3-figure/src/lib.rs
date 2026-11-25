//! The figure layer: a `Figure` of panels, marks, axes, legends and
//! colorbars. Its layout is computed from text measured with typst, and it
//! is drawn into one styled drawing list that every backend renders. SVG
//! is always available; PNG, GIF and PDF are behind their features.
//!
//! The list's geometry is kurbo's and its paint is peniko's; both crates
//! are re-exported ([`kurbo`], [`peniko`]) so that callers name the same
//! versions, and a major version of either is a major version of this
//! crate. So is [`typst`], which sets the text, for callers that use its
//! compiler or types themselves.
//!
//! Applications usually reach this crate as `rustd3::figure`, through the
//! `rustd3` facade's `figure` feature. The examples here use the component
//! crates' own names. With the facade as the only dependency,
//! `rustd3_figure::x` is `rustd3::figure::x` and `rustd3_core::x` is
//! `rustd3::x`. `rustd3_iced` (the `Plot` widget) is used directly, since
//! the facade does not re-export it.
//!
//! ```
//! # #[cfg(feature = "fonts")] {
//! use rustd3_core::scale;
//! use rustd3_figure::kurbo::Size;
//! use rustd3_figure::mark::line;
//! use rustd3_figure::{Figure, axis, lower, svg};
//!
//! let mut fig = Figure::new();
//! let x = fig.fit(scale::linear());
//! let y = fig.fit(scale::linear());
//! fig.panel(0, 0)
//!     .mark(line([(0.0, 1.0), (1.0, 3.0), (2.0, 2.0)], x, y))
//!     .axis(axis::bottom(x).label("x"))
//!     .axis(axis::left(y).label("y"));
//! // The drawing list every backend renders: here SVG, as `save_svg` writes it.
//! let list = fig.draw(Size::new(480.0, 320.0));
//! let lowered = lower::lower(&list, svg::CAPS)?;
//! let svg = svg::write(&lowered.list, &svg::Options::default())?;
//! assert!(svg.starts_with("<svg"));
//! # }
//! # Ok::<(), rustd3_figure::list::Error>(())
//! ```
//!
//! # Features
//!
//! - `fonts` (default): DejaVu Sans and New Computer Modern Math from
//!   rustd3-fonts, and the items that use them when the app supplies no
//!   fonts (`Figure::draw`, `Figure::save_svg`, `gif::Writer`). Without
//!   it, text is set in the fonts the app gives `text::Fonts::with`.
//! - `raster`: vello_cpu renders a drawing list into pixels
//!   (`raster::Renderer`).
//! - `png`: `raster` and the PNG encoder (`Figure::save_png`,
//!   `raster::png`).
//! - `gif`: `raster` and animated GIFs (`gif`).
//! - `pdf`: PDF through krilla, text selectable (`pdf`,
//!   `Figure::save_pdf`).
//! - `tiny-skia`: the fallback rasterizer (`raster::skia`).
//! - `helpers`: one-call charts, statistical charts and subplots
//!   (`helpers`).

// Builders and pure functions are `#[must_use]`: dropping their result is
// a bug.
#![warn(clippy::must_use_candidate, clippy::return_self_not_must_use)]
#![warn(missing_debug_implementations)]

pub use kurbo;
pub use peniko;
pub use typst;

pub mod axis;
pub mod brush;
pub mod cache;
pub mod check;
pub mod colorbar;
mod figure;
#[cfg(feature = "gif")]
pub mod gif;
pub mod hatch;
#[cfg(feature = "helpers")]
pub mod helpers;
pub mod hit;
pub mod layout;
pub mod legend;
pub mod list;
pub mod lower;
pub mod mark;
mod panel;
pub mod path;
#[cfg(feature = "pdf")]
pub mod pdf;
mod pixels;
pub mod raster;
pub mod scale;
mod sealed;
pub mod stats;
pub mod svg;
mod target;
pub mod text;
pub mod theme;
pub mod thin;

pub use figure::{Error, Figure, Fits, Held, Resolved};
pub use panel::Panel;
pub use target::Target;
pub use theme::settings::Theme;
