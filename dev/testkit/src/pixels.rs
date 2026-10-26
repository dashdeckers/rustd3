//! Pixel statistics for comparing two renders of the same size.
//!
//! The tile score is the worst 32 × 32 px tile's mean absolute channel
//! difference over 255: the maximum over tiles, not the mean, so a
//! localized defect (a missing glyph, a shifted marker) does not average
//! away over a large image.

mod error;

pub use error::Error;

/// Side of the square tiles the tile score is taken over, anchored at (0, 0).
const TILE: u32 = 32;

/// A borrowed RGBA8 image, row-major, 4 bytes per pixel.
#[derive(Debug, Clone, Copy)]
pub struct Image<'a> {
    width: u32,
    height: u32,
    rgba: &'a [u8],
}

/// How two images of the same size differ, channel by channel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Diff {
    max: u8,
    over: u64,
    channels: u64,
    tile: f64,
    worst: (u32, u32),
}

impl<'a> Image<'a> {
    /// The image, if `rgba` holds exactly `width × height × 4` bytes.
    pub fn new(width: u32, height: u32, rgba: &'a [u8]) -> Result<Self, Error> {
        let expected =
            usize::try_from(u64::from(width) * u64::from(height) * 4).unwrap_or(usize::MAX);
        if rgba.len() != expected {
            return Err(Error::Length {
                width,
                height,
                expected,
                actual: rgba.len(),
            });
        }
        Ok(Self {
            width,
            height,
            rgba,
        })
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }
}

impl Diff {
    /// Compares `a` and `b` channel by channel, as given (callers pass the
    /// same alpha convention on both sides); `threshold` sets what
    /// [`Diff::over`] counts.
    pub fn between(a: &Image<'_>, b: &Image<'_>, threshold: u8) -> Result<Diff, Error> {
        if (a.width, a.height) != (b.width, b.height) {
            return Err(Error::Size {
                a: (a.width, a.height),
                b: (b.width, b.height),
            });
        }
        let (width, tile) = (u64::from(a.width), u64::from(TILE));
        let columns = width.div_ceil(tile);
        let mut sums = vec![0_u64; index(columns * u64::from(a.height).div_ceil(tile))];
        let (mut max, mut over) = (0_u8, 0_u64);
        let pixels = a.rgba.chunks_exact(4).zip(b.rgba.chunks_exact(4));
        for (i, (p, q)) in (0_u64..).zip(pixels) {
            let sum = &mut sums[index(i / width / tile * columns + i % width / tile)];
            for (c, d) in p.iter().zip(q) {
                let difference = c.abs_diff(*d);
                max = max.max(difference);
                over += u64::from(difference > threshold);
                *sum += u64::from(difference);
            }
        }
        let (mut score, mut worst) = (0.0, (0, 0));
        for (i, sum) in (0_u64..).zip(&sums) {
            let origin = (
                coordinate(i % columns * tile),
                coordinate(i / columns * tile),
            );
            let w = (a.width - origin.0).min(TILE);
            let h = (a.height - origin.1).min(TILE);
            let mean = *sum as f64 / f64::from(w * h * 4) / 255.0;
            if mean > score {
                (score, worst) = (mean, origin);
            }
        }
        let channels = u64::from(a.width) * u64::from(a.height) * 4;
        Ok(Diff {
            max,
            over,
            channels,
            tile: score,
            worst,
        })
    }

    /// Largest absolute channel difference, 0–255.
    pub fn max(&self) -> u8 {
        self.max
    }

    /// Channels whose difference exceeds the threshold.
    pub fn over(&self) -> u64 {
        self.over
    }

    /// `over` as a share of all channels; 0 for an empty image.
    pub fn share_over(&self) -> f64 {
        if self.channels == 0 {
            return 0.0;
        }
        self.over as f64 / self.channels as f64
    }

    /// Worst 32 px tile: mean absolute channel difference / 255.
    pub fn tile_score(&self) -> f64 {
        self.tile
    }

    /// Top-left pixel of the worst tile, for failure messages.
    pub fn worst_tile(&self) -> (u32, u32) {
        self.worst
    }
}

/// A tile or channel index as `usize`.
fn index(i: u64) -> usize {
    usize::try_from(i).expect("invariant: an index into a buffer in memory fits in usize")
}

/// A tile origin as a pixel coordinate.
fn coordinate(c: u64) -> u32 {
    u32::try_from(c).expect("invariant: a tile origin lies inside the image")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diff(width: u32, height: u32, a: &[u8], b: &[u8], threshold: u8) -> Diff {
        let a = Image::new(width, height, a).unwrap();
        let b = Image::new(width, height, b).unwrap();
        Diff::between(&a, &b, threshold).unwrap()
    }

    fn set(
        rgba: &mut [u8],
        width: u32,
        (x, y): (u32, u32),
        channels: std::ops::Range<usize>,
        value: u8,
    ) {
        let at = (y as usize * width as usize + x as usize) * 4;
        rgba[at + channels.start..at + channels.end].fill(value);
    }

    #[test]
    fn equal_images_do_not_differ() {
        let rgba = vec![7; 64 * 64 * 4];
        let d = diff(64, 64, &rgba, &rgba, 0);
        assert_eq!(
            (d.max(), d.over(), d.tile_score(), d.worst_tile()),
            (0, 0, 0.0, (0, 0))
        );
    }

    #[test]
    fn one_channel_off_by_255_scores_its_share_of_one_tile() {
        let a = vec![0; 64 * 64 * 4];
        let mut b = a.clone();
        set(&mut b, 64, (5, 5), 0..1, 255);
        let d = diff(64, 64, &a, &b, 0);
        assert_eq!((d.max(), d.over(), d.worst_tile()), (255, 1, (0, 0)));
        assert_eq!(d.share_over(), 1.0 / 16_384.0);
        assert!((d.tile_score() - 1.0 / 4_096.0).abs() < 1e-15);
        assert!((d.tile_score() - 0.000244).abs() < 1e-6);
    }

    #[test]
    fn a_whole_tile_off_by_5_scores_5_over_255() {
        let a = vec![100; 64 * 64 * 4];
        for (offset, expected) in [(5, 0.0196), (6, 0.0235)] {
            let mut b = a.clone();
            for y in 32..64 {
                for x in 0..32 {
                    set(&mut b, 64, (x, y), 0..4, 100 + offset);
                }
            }
            let d = diff(64, 64, &a, &b, 0);
            assert!((d.tile_score() - f64::from(offset) / 255.0).abs() < 1e-15);
            assert!((d.tile_score() - expected).abs() < 1e-4);
            assert_eq!(d.worst_tile(), (0, 32));
        }
    }

    #[test]
    fn edge_tiles_are_averaged_over_their_own_channels() {
        let a = vec![0; 33 * 33 * 4];
        let mut b = a.clone();
        set(&mut b, 33, (32, 32), 0..4, 255);
        let d = diff(33, 33, &a, &b, 0);
        assert_eq!((d.tile_score(), d.worst_tile()), (1.0, (32, 32)));
    }

    #[test]
    fn the_threshold_counts_only_larger_differences() {
        let a = vec![0; 4 * 4];
        let mut b = a.clone();
        b[..3].copy_from_slice(&[1, 2, 3]);
        assert_eq!(diff(2, 2, &a, &b, 1).over(), 2);
    }

    #[test]
    fn an_empty_image_scores_zero() {
        let d = diff(0, 0, &[], &[], 0);
        assert_eq!(
            (d.max(), d.over(), d.share_over(), d.tile_score()),
            (0, 0, 0.0, 0.0)
        );
    }

    #[test]
    fn a_short_buffer_is_rejected_with_its_length() {
        assert!(matches!(
            Image::new(2, 2, &[0; 15]),
            Err(Error::Length {
                expected: 16,
                actual: 15,
                ..
            })
        ));
    }

    #[test]
    fn images_of_different_sizes_do_not_compare() {
        let (a, b) = (vec![0; 2 * 4], vec![0; 2 * 4]);
        let a = Image::new(2, 1, &a).unwrap();
        let b = Image::new(1, 2, &b).unwrap();
        assert!(matches!(
            Diff::between(&a, &b, 0),
            Err(Error::Size {
                a: (2, 1),
                b: (1, 2)
            })
        ));
        assert_eq!((a.width(), a.height()), (2, 1));
    }
}
