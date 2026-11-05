//! d3-color: [`Rgb`], [`Hsl`], [`Lab`], [`Hcl`] and [`Cubehelix`] as plain
//! values with d3's conversions (`From`), CSS parsing (`FromStr`, the 148
//! names) and formatting (`Display` and the `format_*` methods).
//!
//! ```
//! use rustd3_core::color::{Hcl, Hsl, Lab, Rgb};
//!
//! let hsl: Hsl = "hsl(120, 50%, 50%)".parse()?;
//! assert_eq!(Rgb::from(hsl).to_string(), "rgb(64, 191, 64)");
//! assert_eq!("rgba(70, 130, 180, 0.4)".parse::<Rgb>()?.format_hex(), "#4682b4");
//! assert!("#zzz".parse::<Rgb>().is_err());
//! assert_eq!(Hcl::from(Lab::gray(50.0)).c, 0.0);
//! # Ok::<(), rustd3_core::color::ParseColorError>(())
//! ```

mod cubehelix;
mod error;
mod hcl;
mod hsl;
mod lab;
mod named;
mod parse;
mod rgb;

pub use cubehelix::Cubehelix;
pub use error::ParseColorError;
pub use hcl::Hcl;
pub use hsl::Hsl;
pub use lab::Lab;
pub use rgb::Rgb;

use crate::js;

/// d3's `darker`: one step of `darker(1)` scales by this.
const DARKER: f64 = 0.7;

/// d3's `brighter`, 1/0.7: one step of `brighter(1)` scales by this.
const BRIGHTER: f64 = 1.0 / DARKER;

/// d3's `K`: one step of `brighter(1)` adds this to the lightness of a
/// [`Lab`] or [`Hcl`] color.
const K: f64 = 18.0;

/// d3's `clampa`: the opacity clamped to [0, 1], NaN as 1 and −0 as 0.
fn clamp_opacity(a: f64) -> f64 {
    if a.is_nan() {
        1.0
    } else {
        js::max(0.0, js::min(1.0, a))
    }
}
