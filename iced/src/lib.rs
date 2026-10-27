//! iced integration at two levels: at the core level, [`IcedPath`] draws
//! rustd3's shapes into an iced canvas and [`pointer::Tracker`] turns iced
//! events into pointer input for zoom, brush and drag; at the figure
//! level, behind the `figure` feature, the `Plot` widget shows rendered
//! images of a figure with interaction feedback drawn on top.
//!
//! ```
//! use rustd3_core::shape;
//!
//! // Core level: the app draws d3's shapes into its own canvas geometry.
//! let trace = rustd3_iced::path(|p| {
//!     shape::line().draw([(0.0, 120.0), (160.0, 40.0), (320.0, 120.0)], p)
//! });
//! assert_eq!(trace.raw().iter().count(), 4); // begin, two lines, end
//! ```

mod path;
#[cfg(feature = "figure")]
pub mod plot;
pub mod pointer;

pub use path::{IcedPath, path};
#[cfg(feature = "figure")]
pub use plot::Plot;
