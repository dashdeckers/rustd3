//! iced integration for rustd3, at two levels.
//!
//! At the core level, [`IcedPath`] draws rustd3's shapes into an iced
//! canvas, and [`pointer::Tracker`] turns iced events into pointer input for
//! zoom, brush and drag. At the figure level, behind the `figure` feature,
//! the `Plot` widget shows rendered images of a figure with interaction
//! feedback drawn on top.
//!
//! With the `fonts` feature, [`font`] holds the bundled faces as iced fonts,
//! so an app's own text is set in the faces its plots are drawn in.
//!
//! An app depends on the `rustd3` facade and on this crate. The examples
//! here use the facade's paths (`rustd3::shape`, `rustd3::figure::Figure`),
//! which are `rustd3_core::shape` and `rustd3_figure::Figure` in the
//! component crates.
//!
//! The crate depends on iced at git revision 98bd48f, and its signatures
//! name iced's types (`Event`, `Rectangle`, `mouse::Cursor`, `Point`,
//! `Length`, `Element`, `canvas::Path`). The app depends on iced at the
//! same revision and names them through `iced`.
//!
//! ```
//! # mod rustd3 { pub use rustd3_core::shape; }
//! use rustd3::shape;
//!
//! // Core level: the app draws d3's shapes into its own canvas geometry.
//! let trace = rustd3_iced::path(|p| {
//!     shape::line().draw([(0.0, 120.0), (160.0, 40.0), (320.0, 120.0)], p)
//! });
//! assert_eq!(trace.raw().iter().count(), 4); // begin, two lines, end
//! ```

// Builders and pure functions are must-use, since dropping their result is
// a bug.
#![warn(clippy::must_use_candidate, clippy::return_self_not_must_use)]
#![warn(missing_debug_implementations)]
// The package denies unsafe code so that its examples can allow one call.
#![forbid(unsafe_code)]

#[cfg(feature = "fonts")]
pub mod font;
mod path;
#[cfg(feature = "figure")]
pub mod plot;
pub mod pointer;

pub use path::{IcedPath, path};
#[cfg(feature = "figure")]
pub use plot::widget::Plot;
