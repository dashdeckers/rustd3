//! rustd3-core's spot checks: values d3 prints, and the port's own rules
//! where it departs from d3.

mod anim;
mod array;
mod axis;
mod band;
mod binned;
mod brush;
mod color;
#[cfg(feature = "derive")]
mod derive;
mod drag;
mod ease;
mod examples;
mod fixtures;
mod format;
mod fuzz;
mod interact;
mod interpolate;
mod join;
mod locale;
mod ordinal;
mod pad;
mod path;
mod point;
mod rules;
mod scale;
mod scheme;
mod series;
mod shape;
mod ticks;
mod time;
