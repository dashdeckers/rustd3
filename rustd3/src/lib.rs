//! d3's vocabulary for Rust (scales, ticks, axes, shapes, number and time
//! formats, color, zoom and brush), with figures behind the `figure`
//! feature: typst text and math, legends and colorbars, layout from
//! measured text, and SVG, PNG, PDF and GIF from one drawing list.
//!
//! This crate is the facade: it re-exports `rustd3-core`'s modules at its
//! root and, with `figure`, `rustd3-figure` as `rustd3::figure`.
//! The component crates' documentation shows examples under their own
//! names: `rustd3_core::x` is `rustd3::x` and `rustd3_figure::x` is
//! `rustd3::figure::x`. `rustd3_iced` is not re-exported: an iced app
//! depends on `rustd3-iced` and uses it directly.
//!
//! Three dependencies' types are part of the API, and each is re-exported
//! at the version rustd3 is built with: `rustd3::jiff` (instants, zones
//! and durations of the time API), and with `figure`,
//! `rustd3::figure::kurbo` (the drawing list's geometry) and
//! `rustd3::figure::peniko` (its paints). Name them through these paths;
//! a major version of any of them is a major version of rustd3.
//!
//! # The core level
//!
//! Scales and generators are plain values; generators draw into any
//! [`path::PathContext`], here an SVG path string:
//!
//! ```
//! use rustd3::{axis, path, scale, shape};
//!
//! let x = scale::linear().domain([0.0, 100.0]).range([0.0, 960.0]);
//! let y = scale::linear().domain([0.0, 1.0]).range([100.0, 0.0]);
//! assert_eq!(x.apply(25.0), 240.0);
//!
//! let points = [(0.0, 0.5), (50.0, 1.0), (100.0, 0.0)].map(|(a, b)| (x.apply(a), y.apply(b)));
//! assert_eq!(path::svg(|p| shape::line().draw(points, p)), "M0,50L480,0L960,100");
//!
//! let labels: Vec<String> = axis::bottom(&x).ticks(5).labels().iter().map(|l| l.get_text().to_owned()).collect();
//! assert_eq!(labels, ["0", "20", "40", "60", "80", "100"]);
//! assert_eq!(rustd3::fmt!(".1%").format(0.123), "12.3%");
//! ```
//!
//! # The figure level
//!
//! A `figure::Figure` of panels, marks and axes; layout measures its
//! text and a backend draws it:
//!
//! ```
//! # #[cfg(all(feature = "figure", feature = "fonts"))] {
//! use rustd3::figure::kurbo::Size;
//! use rustd3::figure::mark::line;
//! use rustd3::figure::{Figure, axis};
//! use rustd3::scale;
//!
//! let mut fig = Figure::new();
//! let x = fig.fit(scale::linear());
//! let y = fig.fit(scale::linear());
//! fig.panel(0, 0)
//!     .mark(line([(0.0, 1.0), (1.0, 3.0), (2.0, 2.0)], x, y))
//!     .axis(axis::bottom(x).label("x"))
//!     .axis(axis::left(y).label("$y^2$"));
//! let list = fig.draw(Size::new(480.0, 320.0));
//! assert!(!list.commands.is_empty());
//! # }
//! ```
//!
//! # Features
//!
//! - `fonts` (default): bundled DejaVu Sans and New Computer Modern Math.
//! - `figure`: figures, layout, typst text and the SVG writer.
//! - `raster`, `png`, `gif`: pixels through vello_cpu, PNG and GIF files.
//! - `pdf`: PDF through krilla, with selectable text.
//! - `tiny-skia`: the fallback rasterizer.
//! - `helpers`: one-call charts, statistical charts and subplots.
//! - `derive`: `#[derive(Interpolate)]`.
//!
//! Every feature but `fonts` and `derive` turns `figure` on. The guide,
//! `GUIDE.md` in the repository, explains the concepts and their types.

pub use rustd3_core::{
    anim,
    array,
    axis,
    color,
    ease,
    fmt,
    format,
    interact,
    interpolate,
    jiff,
    join,
    locale,
    path,
    scale,
    scheme,
    series,
    shape,
    ticks,
    time,
};
#[cfg(feature = "figure")]
pub use rustd3_figure as figure;
