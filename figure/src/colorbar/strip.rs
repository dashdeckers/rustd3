//! [`Strip`]: a colorbar's colors along its bar over one resolve, and the
//! image that draws them.

use std::sync::Arc;

use peniko::{Blob, ImageAlphaType, ImageData, ImageFormat};
use rustd3_core::color::Rgb;

use crate::colorbar::{Ramp, Samples};
use crate::scale::{Kind, Paint};
use crate::text;

/// Image rows per px of bar for bins of unequal widths: their edges fall
/// within an eighth of a px of where the scale puts them.
const ROWS_PER_PX: f64 = 4.0;

/// The longest image, rows: a bar of 4,096 px at 4 rows per px, or of
/// 16,384 device px.
const MAX_ROWS: f64 = 16_384.0;

/// How far two bins' widths may differ, as a share of the bar, and still
/// be drawn as equal rows.
const EQUAL: f64 = 1e-9;

/// The end color of a strip without colors: transparent.
const NONE: Rgb = Rgb {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    opacity: 0.0,
};

/// A colorbar's colors over one resolve, from the low end of its bar:
/// equal rows, or bins and where each ends along the bar (shares of its
/// length, ascending from 0 to 1) when their widths differ; whether they
/// are a binned scale's; and the colors of its end triangles.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Strip {
    colors: Vec<Rgb>,
    bounds: Option<Vec<f64>>,
    binned: bool,
    ends: [Rgb; 2],
}

impl Strip {
    /// The strip of `scale`: its samples, binned ones placed by the scale's
    /// tick positions of their edges (bins whose edges have no position or
    /// do not ascend are drawn as equal rows); its end triangles in the
    /// colors it clamps to, those of its domain's ends, or of its first and
    /// last bins.
    pub(crate) fn of<S: Paint<Domain = f64> + Ramp>(scale: &S) -> Strip {
        let (colors, bounds, binned) = match scale.samples() {
            Samples::Even(colors) => (colors, None, false),
            Samples::Bins { colors, edges } => {
                let inner: Option<Vec<f64>> = edges
                    .iter()
                    .map(|e| scale.position(e, 0.0).filter(|p| p.is_finite()))
                    .collect();
                let bounds = inner
                    .map(|inner| {
                        let mut bounds = vec![0.0];
                        bounds.extend(inner.iter().map(|p| p.clamp(0.0, 1.0)));
                        bounds.push(1.0);
                        bounds
                    })
                    .filter(|b| b.len() == colors.len() + 1 && b.windows(2).all(|w| w[0] <= w[1]))
                    .filter(|b| !equal(b));
                (colors, bounds, true)
            }
        };
        let first = colors.first().copied().unwrap_or(NONE);
        let last = colors.last().copied().unwrap_or(NONE);
        let ends = match Kind::ends(scale) {
            Some([low, high]) => [
                scale.paint(&low).unwrap_or(first),
                scale.paint(&high).unwrap_or(last),
            ],
            None => [first, last],
        };
        Strip {
            colors,
            bounds,
            binned,
            ends,
        }
    }

    /// Whether the colors are a binned scale's.
    pub(crate) fn binned(&self) -> bool {
        self.binned
    }

    /// The colors of the end triangles, [low end, high end].
    pub(crate) fn ends(&self) -> [Rgb; 2] {
        self.ends
    }

    /// The strip's image for a bar `length` px long, nearest-sampled over
    /// it. On a raster target of `scale` device px per px it has one row
    /// per device px of the bar, each the color under its centre, so every
    /// pixel shows the sample (d3's `table[⌊256 t⌋]` for a table scheme)
    /// whatever way a renderer breaks ties at row edges. For vector output
    /// it has one row per color when the rows are equal, else 4 rows per px
    /// (at least one per color), each the color of the bin under its
    /// centre. A vertical bar's first row is its high end, at the top; a
    /// horizontal bar's first column its low end, at the left. `None`
    /// without colors.
    pub(crate) fn image(
        &self,
        length: f64,
        scale: Option<f64>,
        vertical: bool,
    ) -> Option<ImageData> {
        let n = self.colors.len();
        if n == 0 {
            return None;
        }
        let rows: Vec<Rgb> = match (scale, &self.bounds) {
            (None, None) => self.colors.clone(),
            (Some(scale), _) => self.rows((length * scale).round().clamp(1.0, MAX_ROWS)),
            (None, Some(_)) => {
                let count = (length * ROWS_PER_PX)
                    .ceil()
                    .clamp(n as f64, MAX_ROWS.max(n as f64));
                self.rows(count)
            }
        };
        let side = |n: usize| {
            u32::try_from(n).expect("invariant: a strip has at most 16,384 rows or one per color")
        };
        let (width, height) = if vertical {
            (1, side(rows.len()))
        } else {
            (side(rows.len()), 1)
        };
        let ordered: Box<dyn Iterator<Item = &Rgb>> = if vertical {
            Box::new(rows.iter().rev())
        } else {
            Box::new(rows.iter())
        };
        let rgba: Vec<u8> = ordered.flat_map(|c| text::channels(*c)).collect();
        Some(ImageData {
            data: Blob::new(Arc::new(rgba)),
            format: ImageFormat::Rgba8,
            alpha_type: ImageAlphaType::Alpha,
            width,
            height,
        })
    }

    /// `count` equal rows from the low end, each the color under its
    /// centre: of equal rows, the `⌊t n⌋`-th of `n`; of bins, the one whose
    /// bounds hold it.
    fn rows(&self, count: f64) -> Vec<Rgb> {
        let count = count as usize;
        let n = self.colors.len();
        let mut bin = 0;
        (0..count)
            .map(|i| {
                let centre = (i as f64 + 0.5) / count as f64;
                match &self.bounds {
                    None => self.colors[((centre * n as f64) as usize).min(n - 1)],
                    Some(bounds) => {
                        while bin + 1 < n && bounds[bin + 1] <= centre {
                            bin += 1;
                        }
                        self.colors[bin]
                    }
                }
            })
            .collect()
    }
}

/// Whether the bins between `bounds` have equal widths.
fn equal(bounds: &[f64]) -> bool {
    let n = (bounds.len() - 1) as f64;
    bounds
        .windows(2)
        .all(|w| ((w[1] - w[0]) - 1.0 / n).abs() <= EQUAL)
}

#[cfg(test)]
mod tests {
    use rustd3_core::scale;

    use super::*;

    fn rows(image: &ImageData) -> Vec<[u8; 4]> {
        image.data.data().as_chunks::<4>().0.to_vec()
    }

    fn colors(n: usize) -> Vec<Rgb> {
        (0..n)
            .map(|i| Rgb::new(i as f64 * 10.0, 0.0, 0.0))
            .collect()
    }

    #[test]
    fn equal_bins_give_one_row_each_with_the_high_end_on_top() {
        let q = scale::quantize([0.0, 100.0], colors(5));
        let strip = Strip::of(&q);
        assert_eq!(strip.bounds, None);
        let up = strip.image(300.0, None, true).expect("an image");
        assert_eq!((up.width, up.height), (1, 5));
        assert_eq!(rows(&up)[0], [40, 0, 0, 255]);
        let across = strip.image(300.0, None, false).expect("an image");
        assert_eq!((across.width, across.height), (5, 1));
        assert_eq!(rows(&across)[0], [0, 0, 0, 255]);
        let t = scale::threshold([0.0, 1.0, 50.0], colors(4));
        assert_eq!(Strip::of(&t).bounds, None);
    }

    #[test]
    fn unequal_bins_become_groups_of_rows_at_their_widths() {
        let sample = [3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0];
        let q = scale::quantile(sample, colors(4));
        let strip = Strip::of(&q);
        let bounds = strip.bounds.clone().expect("unequal bins");
        assert_eq!(bounds[1], 0.25);
        let image = strip.image(100.0, None, false).expect("an image");
        assert_eq!(image.width, 400);
        let rows = rows(&image);
        // Edge 7.25 at 0.25 of the bar: rows 0–99 are bin 0.
        assert_eq!((rows[99][0], rows[100][0]), (0, 10));
        // Edge 9 at 6/17: row 141's centre 141.5/400 = 0.35375 is past it.
        assert_eq!((rows[140][0], rows[141][0]), (10, 20));
        assert_eq!(rows[399][0], 30);
    }

    #[test]
    fn end_triangles_take_the_colors_the_scale_clamps_to() {
        let s = scale::sequential([0.0, 80.0], rustd3_core::scheme::viridis);
        let strip = Strip::of(&s);
        assert!(!strip.binned());
        assert_eq!(strip.ends().map(|c| c.format_hex()), ["#440154", "#fde725"]);
        let q = scale::quantize([0.0, 100.0], colors(5));
        let strip = Strip::of(&q);
        assert!(strip.binned());
        assert_eq!(strip.ends(), [colors(5)[0], colors(5)[4]]);
    }

    #[test]
    fn a_scale_without_colors_has_no_image() {
        let q = scale::quantize([0.0, 1.0], Vec::<Rgb>::new());
        assert_eq!(Strip::of(&q).image(100.0, Some(2.0), true), None);
    }

    #[test]
    fn a_raster_strip_has_a_row_per_device_px_showing_the_sample_under_it() {
        let s = scale::sequential([0.0, 80.0], rustd3_core::scheme::viridis);
        let strip = Strip::of(&s);
        let image = strip.image(150.0, Some(2.0), false).expect("an image");
        assert_eq!((image.width, image.height), (300, 1));
        let across = rows(&image);
        for (j, row) in across.iter().enumerate() {
            let t = (j as f64 + 0.5) / 300.0;
            assert_eq!(*row, text::channels(s.apply(t * 80.0)), "row {j}");
        }
        // Device px 37's centre is the edge between entries 31 and 32: d3's
        // floor takes 32.
        assert_eq!(
            across[37],
            text::channels(rustd3_core::scheme::viridis(32.0 / 256.0))
        );
        let sample = [3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0];
        let q = scale::quantile(sample, colors(4));
        let up = Strip::of(&q)
            .image(100.0, Some(1.0), true)
            .expect("an image");
        assert_eq!((up.width, up.height), (1, 100));
        // From the bottom, rows 0–24 are bin 0: the 75 rows above it.
        let up = rows(&up);
        assert_eq!((up[74][0], up[75][0]), (10, 0));
    }
}
