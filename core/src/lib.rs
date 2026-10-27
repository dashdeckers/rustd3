//! The d3 port: scales, ticks, axes, shapes and `PathContext`, number and
//! time formats, locales, color and schemes, interpolation and easing,
//! array statistics, recordings of sampled data, keyed joins, and zoom,
//! brush, drag and animation as plain values and state machines,
//! renderer-agnostic and graded by d3's own tests.

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

/// jiff, which time is built on: figure, iced and applications name its
/// types (`Timestamp`, `tz::TimeZone`, `civil::Weekday`) through this path.
pub use jiff;
