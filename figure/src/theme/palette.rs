//! Categorical palettes for [`Theme::palette`](crate::Theme::palette):
//! core's schemes re-exported, one definition each, and seaborn 0.13.2's
//! palettes core does not carry, the first ten colors of
//! `color_palette(name)`.

use rustd3_core::color::Rgb;

pub use rustd3_core::scheme::{CATEGORY10, COLORBLIND, DEEP, OBSERVABLE10, OKABE_ITO};

/// seaborn's `"muted"` palette.
pub const MUTED: [Rgb; 10] = colors([
    0x4878d0, 0xee854a, 0x6acc64, 0xd65f5f, 0x956cb4, 0x8c613c, 0xdc7ec0, 0x797979, 0xd5bb67,
    0x82c6e2,
]);

/// seaborn's `"pastel"` palette.
pub const PASTEL: [Rgb; 10] = colors([
    0xa1c9f4, 0xffb482, 0x8de5a1, 0xff9f9b, 0xd0bbff, 0xdebb9b, 0xfab0e4, 0xcfcfcf, 0xfffea3,
    0xb9f2f0,
]);

/// seaborn's `"bright"` palette.
pub const BRIGHT: [Rgb; 10] = colors([
    0x023eff, 0xff7c00, 0x1ac938, 0xe8000b, 0x8b2be2, 0x9f4800, 0xf14cc1, 0xa3a3a3, 0xffc400,
    0x00d7ff,
]);

/// seaborn's `"dark"` palette.
pub const DARK: [Rgb; 10] = colors([
    0x001c7f, 0xb1400d, 0x12711c, 0x8c0800, 0x591e71, 0x592f0d, 0xa23582, 0x3c3c3c, 0xb8850a,
    0x006374,
]);

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

#[cfg(test)]
mod tests {
    use super::*;

    /// The palette as seaborn's `as_hex()` writes it, without the `#`s.
    fn hex(palette: &[Rgb]) -> String {
        palette
            .iter()
            .map(|c| c.format_hex().trim_start_matches('#').to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn palettes_are_seaborns_first_ten_colors() {
        // seaborn 0.13.2: `sns.color_palette(name).as_hex()`.
        let rows = [
            (
                &DEEP,
                "4c72b0 dd8452 55a868 c44e52 8172b3 937860 da8bc3 8c8c8c ccb974 64b5cd",
            ),
            (
                &MUTED,
                "4878d0 ee854a 6acc64 d65f5f 956cb4 8c613c dc7ec0 797979 d5bb67 82c6e2",
            ),
            (
                &PASTEL,
                "a1c9f4 ffb482 8de5a1 ff9f9b d0bbff debb9b fab0e4 cfcfcf fffea3 b9f2f0",
            ),
            (
                &BRIGHT,
                "023eff ff7c00 1ac938 e8000b 8b2be2 9f4800 f14cc1 a3a3a3 ffc400 00d7ff",
            ),
            (
                &DARK,
                "001c7f b1400d 12711c 8c0800 591e71 592f0d a23582 3c3c3c b8850a 006374",
            ),
            (
                &COLORBLIND,
                "0173b2 de8f05 029e73 d55e00 cc78bc ca9161 fbafe4 949494 ece133 56b4e9",
            ),
        ];
        for (palette, expected) in rows {
            assert_eq!(hex(palette), expected);
        }
    }
}
