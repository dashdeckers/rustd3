//! The d3 port: scales, ticks, axes, shapes and `PathContext`, number and
//! time formats, locales, color and schemes, interpolation and easing,
//! array statistics, recordings of sampled data, keyed joins, and zoom,
//! brush, drag and animation as plain values and state machines,
//! renderer-agnostic and graded by d3's own tests. Applications usually
//! depend on the `rustd3` facade, which re-exports these modules: the
//! examples here use this crate's own name, and `rustd3_core::x` is
//! `rustd3::x` with the facade.
//!
//! ```
//! use rustd3_core::jiff::{Timestamp, tz::TimeZone};
//! use rustd3_core::{path, scale, shape, time};
//!
//! // Scales are plain values: build, call, drop.
//! let x = scale::linear().domain([0.0, 100.0]).range([0.0, 960.0]);
//! assert_eq!(x.apply(25.0), 240.0);
//! assert_eq!(x.invert(480.0), 50.0);
//!
//! // Generators draw into any `PathContext`, here SVG path data.
//! let y = scale::linear().domain([0.0, 1.0]).range([100.0, 0.0]);
//! let points = [(0.0, 0.5), (100.0, 0.0)].map(|(a, b)| (x.apply(a), y.apply(b)));
//! assert_eq!(path::svg(|p| shape::line().draw(points, p)), "M0,50L960,100");
//!
//! // Time is jiff's, and the zone is always a parameter.
//! let day: [Timestamp; 2] = ["2026-10-28T00:00:00Z".parse()?, "2026-10-29T00:00:00Z".parse()?];
//! let t = time::utc().domain(day).range([0.0, 960.0]);
//! let noon = t.invert(480.0);
//! assert_eq!(time::format("%H:%M").format(noon, &TimeZone::UTC), "12:00");
//! # Ok::<(), rustd3_core::jiff::Error>(())
//! ```
//!
//! The `derive` feature adds `#[derive(Interpolate)]`, re-exported beside
//! the trait as [`interpolate::Interpolate`].
//!
//! jiff's types (`Timestamp`, `tz::TimeZone`, `SignedDuration`) are part
//! of this crate's API and re-exported as [`jiff`]: a major version of
//! jiff is a major version of this crate.

// Builders and pure functions say so: a result dropped is a bug.
#![warn(clippy::must_use_candidate, clippy::return_self_not_must_use)]
#![warn(missing_debug_implementations)]

pub mod anim;
pub mod array;
pub mod axis;
pub mod color;
pub mod ease;
pub mod format;
pub mod interact;
pub mod interpolate;
pub mod join;
pub mod locale;
pub mod path;
pub mod scale;
pub mod scheme;
pub mod series;
pub mod shape;
pub mod ticks;
pub mod time;

mod js;
mod sealed;

/// jiff, which time is built on: figure, iced and applications name its
/// types (`Timestamp`, `tz::TimeZone`, `civil::Weekday`) through this path.
pub use jiff;
