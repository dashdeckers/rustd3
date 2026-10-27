//! rustd3-iced's integration tests.

mod canvas;
mod headless;
mod path;
mod placement;
#[cfg(all(feature = "figure", feature = "fonts"))]
mod plot;
mod pointer;
#[cfg(all(feature = "figure", feature = "fonts"))]
mod reference;
