//! Color schemes: categorical ones as `const` arrays of [`Rgb`] (d3's,
//! seaborn's and Okabe and Ito's), and d3-scale-chromatic's sequential
//! ones as plain functions of `t` in [0, 1], named as d3's camel case
//! lowered to snake case (`interpolateViridis` is [`viridis`]).
//!
//! ```
//! use rustd3_core::{scale, scheme};
//!
//! let color = scale::ordinal(["cpu", "mem"], scheme::CATEGORY10);
//! assert_eq!(color.apply(&"cpu").map(ToString::to_string).as_deref(), Some("rgb(31, 119, 180)"));
//! assert_eq!(scheme::viridis(0.5).format_hex(), "#21918c");
//! ```

mod categorical;
mod inferno;
mod magma;
mod plasma;
mod viridis;

pub use categorical::{ACCENT, DARK2, PAIRED, PASTEL1, PASTEL2, SET1, SET2, SET3, TABLEAU10};
pub use inferno::inferno;
pub use magma::magma;
pub use plasma::plasma;
pub use viridis::viridis;

use crate::color::Rgb;

/// d3's `schemeCategory10`, which is matplotlib's `tab10`.
#[doc(alias = "schemeCategory10")]
pub const CATEGORY10: [Rgb; 10] = colors([
    0x1f77b4, 0xff7f0e, 0x2ca02c, 0xd62728, 0x9467bd, 0x8c564b, 0xe377c2, 0x7f7f7f, 0xbcbd22,
    0x17becf,
]);

/// d3's `schemeObservable10`.
#[doc(alias = "schemeObservable10")]
pub const OBSERVABLE10: [Rgb; 10] = colors([
    0x4269d0, 0xefb118, 0xff725c, 0x6cc5b0, 0x3ca951, 0xff8ab7, 0xa463f2, 0x97bbf5, 0x9c6b4e,
    0x9498a0,
]);

/// seaborn 0.13.2's `"deep"` palette.
pub const DEEP: [Rgb; 10] = colors([
    0x4c72b0, 0xdd8452, 0x55a868, 0xc44e52, 0x8172b3, 0x937860, 0xda8bc3, 0x8c8c8c, 0xccb974,
    0x64b5cd,
]);

/// seaborn 0.13.2's `"colorblind"` palette, the theme's default.
pub const COLORBLIND: [Rgb; 10] = colors([
    0x0173b2, 0xde8f05, 0x029e73, 0xd55e00, 0xcc78bc, 0xca9161, 0xfbafe4, 0x949494, 0xece133,
    0x56b4e9,
]);

/// Okabe and Ito's (2008) colors for color-blind readers, black last.
pub const OKABE_ITO: [Rgb; 8] = colors([
    0xe69f00, 0x56b4e9, 0x009e73, 0xf0e442, 0x0072b2, 0xd55e00, 0xcc79a7, 0x000000,
]);

/// d3's `ramp(range)`: the entry `floor(t·N)` of `table`, clamped to the
/// first and last; NaN casts to entry 0.
fn ramp<const N: usize>(table: &[Rgb; N], t: f64) -> Rgb {
    let i = (t * N as f64).floor().clamp(0.0, (N - 1) as f64);
    table[i as usize]
}

/// The opaque colors `0xrrggbb` of `hex`, in order.
const fn colors<const N: usize>(hex: [u32; N]) -> [Rgb; N] {
    let mut out = [Rgb::new(0.0, 0.0, 0.0); N];
    let mut i = 0;
    while i < N {
        out[i] = Rgb::hex(hex[i]);
        i += 1;
    }
    out
}
