//! The raster cost model the widget predicts each refresh with: two
//! constants fitted to the wheel figure at 2× (1920 × 1520 device px) on a
//! 2-core container, rendered single-threaded: 16–25 ms whole, 6.7 ms
//! without its 5,600 dots. The dots cost about 14 ms, about 400 points per
//! millisecond; the rest, 2,918,400 px and about 760 items, about 6.7 ms,
//! of which about 1.9 ms are items, which leaves about 6.1e5 px per
//! millisecond, rounded down to 5e5. Checked back, the model gives 21.7 ms
//! for the whole figure and 7.7 ms without the dots. They seed each
//! widget's own fit, which adds a fixed overhead per render and follows
//! the machine from its timed renders; public so the benchmark can
//! record them.

/// Device pixels rasterized per millisecond.
pub const PIXELS_PER_MS: f64 = 5.0e5;

/// Work items (a cell's `work`: one per fill, stroke or image, one per 16
/// path elements, one per marker point, one per glyph) per millisecond.
pub const ITEMS_PER_MS: f64 = 400.0;

/// Modelled single-threaded time of one cell, in milliseconds: `pixels /
/// PIXELS_PER_MS + work / ITEMS_PER_MS`.
pub fn model_ms(pixels: u64, work: u64) -> f64 {
    pixels as f64 / PIXELS_PER_MS + work as f64 / ITEMS_PER_MS
}

#[cfg(test)]
mod tests {
    use super::model_ms;

    /// Within a microsecond.
    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn cost_model_matches_the_spot_rows() {
        // 320 × 200 at 2×, 150 items: 0.512 + 0.375.
        assert!(close(model_ms(640 * 400, 150), 0.887));
        // The wheel figure at 2×: 5.84 + 15.9 ≈ 21.7 ms; without its 5,600
        // dots 5.84 + 1.9 ≈ 7.7 ms.
        assert!(close(model_ms(1920 * 1520, 6_360), 5.8368 + 15.9));
        assert!(close(model_ms(2_918_400, 760), 7.7368));
        // 200k dots at 1600 × 1000 logical at 2×: 12.8 + 500 ≈ 513 ms.
        assert!(close(model_ms(3200 * 2000, 200_000), 512.8));
        // Three cells of 300 × 200 px with 100 items each: 0.36 + 0.75.
        assert!(close(3.0 * model_ms(300 * 200, 100), 1.11));
    }
}
