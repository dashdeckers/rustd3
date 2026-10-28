//! rustd3-iced's integration tests.

#[cfg(all(feature = "figure", feature = "fonts"))]
mod brush;
mod canvas;
#[cfg(all(feature = "figure", feature = "fonts"))]
mod classify;
#[cfg(all(feature = "figure", feature = "fonts"))]
mod gestures;
mod headless;
#[cfg(all(feature = "figure", feature = "fonts"))]
mod hover;
mod path;
#[cfg(all(feature = "figure", feature = "fonts"))]
mod pipeline;
mod placement;
#[cfg(all(feature = "figure", feature = "fonts"))]
mod plot;
mod pointer;
#[cfg(all(feature = "figure", feature = "fonts"))]
mod reference;
#[cfg(all(feature = "figure", feature = "fonts"))]
mod tiles;
