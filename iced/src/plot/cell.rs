//! [`Cell`] and [`Band`]: one cell of a figure's partition, its device
//! rect and hash, and the images it is drawn as.

use std::ops::Range;

use iced_core::Rectangle;
use iced_core::image::{self, FilterMethod};
use rustd3_figure::kurbo::Rect;
use rustd3_figure::list::{self, Bounds, List};
use rustd3_figure::raster;

use crate::plot::{BAND_BYTES, NUDGE, handle};

/// A cell of the canvas's partition by every plot-area edge, rendered.
pub(crate) struct Cell {
    /// Device px [x0, y0, x1, y1], relative to the widget's device origin.
    rect: [i32; 4],
    /// The rect and the hashes of the parts the cell shows.
    hash: u64,
    /// The cell's images, top to bottom.
    bands: Vec<Band>,
}

/// Whole device rows of a cell, rendered as one image under `BAND_BYTES`.
pub(crate) struct Band {
    /// Device rows relative to the cell.
    rows: Range<i32>,
    handle: image::Handle,
}

impl Cell {
    /// Partition cell `of` of `list` (whose command bounds are `bounds`)
    /// rendered band by band at `factor` device px per list px: each band
    /// is the list area of its device rows divided by the factor, which
    /// renders exactly the device rows' size. A band that fails to render
    /// shows nothing.
    pub(super) fn render(
        of: &list::Cell,
        list: &List,
        bounds: &Bounds,
        factor: f64,
        raster: &mut raster::Renderer,
    ) -> Cell {
        let rect = device(of.rect);
        let [x0, y0, x1, _] = rect.map(f64::from);
        let bands = Cell::bands(rect)
            .into_iter()
            .filter_map(|rows| {
                let (top, bottom) = (y0 + f64::from(rows.start), y0 + f64::from(rows.end));
                let area = Rect::new(x0 / factor, top / factor, x1 / factor, bottom / factor);
                let image = raster.render(list, bounds, area, factor).ok()?;
                Some(Band {
                    rows,
                    handle: handle::from_image(image),
                })
            })
            .collect();
        Cell {
            rect,
            hash: of.hash,
            bands,
        }
    }

    /// Whether `other` has this cell's device rect and hash: the same
    /// pixels.
    pub(super) fn shows(&self, other: &list::Cell) -> bool {
        self.rect == device(other.rect) && self.hash == other.hash
    }

    /// Draws every band with nearest filtering at bounds that land on its
    /// device pixels, clipped to them, for a widget whose device origin is
    /// `origin`.
    pub(super) fn draw<Renderer>(&self, renderer: &mut Renderer, origin: [i32; 2], factor: f32)
    where
        Renderer: image::Renderer<Handle = image::Handle>,
    {
        for band in &self.bands {
            let bounds = self.bounds(&band.rows, origin, factor);
            let image = image::Image::new(band.handle.clone()).filter_method(FilterMethod::Nearest);
            renderer.draw_image(image, bounds, bounds);
        }
    }

    /// The cell's device rect.
    #[cfg(test)]
    pub(super) fn rect(&self) -> [i32; 4] {
        self.rect
    }

    /// Each band's rows and image: width, height and straight RGBA.
    #[cfg(test)]
    pub(super) fn images(&self) -> Vec<(Range<i32>, u32, u32, iced_core::Bytes)> {
        self.bands
            .iter()
            .map(|band| match &band.handle {
                image::Handle::Rgba {
                    width,
                    height,
                    pixels,
                    ..
                } => (band.rows.clone(), *width, *height, pixels.clone()),
                image::Handle::Path(..) | image::Handle::Bytes(..) => {
                    panic!("bands are RGBA handles")
                }
            })
            .collect()
    }

    /// Row ranges of the bands a cell of this rect is drawn as: one band
    /// when its RGBA stays under `BAND_BYTES`, else bands of
    /// `BAND_BYTES / (width × 4) − 1` rows (at least one), the last one
    /// shorter, each strictly under the limit.
    fn bands(rect: [i32; 4]) -> Vec<Range<i32>> {
        let [x0, y0, x1, y1] = rect;
        let row = usize::try_from(x1 - x0).unwrap_or(0) * 4;
        let height = y1 - y0;
        if row * usize::try_from(height).unwrap_or(0) < BAND_BYTES {
            return std::iter::once(0..height).collect();
        }
        let step = (BAND_BYTES / row).saturating_sub(1).max(1);
        let rows = i32::try_from(step).expect("invariant: fewer rows than BAND_BYTES");
        (0..height)
            .step_by(step)
            .map(|start| start..(start + rows).min(height))
            .collect()
    }

    /// Logical bounds that land exactly on a band's device pixels: its
    /// device position plus `NUDGE`, and its device size, over the factor.
    fn bounds(&self, rows: &Range<i32>, origin: [i32; 2], factor: f32) -> Rectangle {
        let [x0, y0, x1, _] = self.rect;
        Rectangle {
            x: ((origin[0] + x0) as f32 + NUDGE) / factor,
            y: ((origin[1] + y0 + rows.start) as f32 + NUDGE) / factor,
            width: (x1 - x0) as f32 / factor,
            height: rows.len() as f32 / factor,
        }
    }
}

/// A partition cell's rect, whose edges are whole device px.
fn device(rect: Rect) -> [i32; 4] {
    [rect.x0, rect.y0, rect.x1, rect.y1].map(|edge| edge as i32)
}

#[cfg(test)]
mod tests {
    use iced_core::Rectangle;
    use rustd3_figure::kurbo::{Rect, Size};
    use rustd3_figure::list::{self, Bounds, List, Outlines};
    use rustd3_figure::{Held, Target, raster};
    use rustd3_testkit::pixels;

    use super::Cell;
    use crate::plot::{BAND_BYTES, NUDGE, fixture, state};

    /// An unrendered cell of `rect`.
    fn cell(rect: [i32; 4]) -> Cell {
        Cell {
            rect,
            hash: 0,
            bands: Vec::new(),
        }
    }

    #[test]
    fn a_960_by_640_cell_becomes_bands_of_545_and_95_rows() {
        let rect = [0, 0, 960, 640];
        assert_eq!(960 * 640 * 4, 2_457_600);
        assert_eq!(BAND_BYTES / (960 * 4) - 1, 545);
        assert_eq!(Cell::bands(rect), [0..545, 545..640]);
        assert_eq!(545 * 960 * 4, 2_092_800);
    }

    #[test]
    fn bands_stay_under_the_limit() {
        let one = Cell::bands([0, 0, 300, 200]);
        assert_eq!((one.len(), one[0].clone()), (1, 0..200));
        // Rows that divide the limit evenly, an image of exactly the limit,
        // and one row of a very wide cell.
        for rect in [
            [0, 0, 1024, 512],
            [0, 0, 1024, 513],
            [10, 20, 1034, 532],
            [0, 0, 1000, 600],
            [0, 0, 3840, 2160],
            [0, 0, 65_408, 40],
            [0, 0, 724, 724],
        ] {
            let width = (rect[2] - rect[0]) as usize;
            let bands = Cell::bands(rect);
            let mut next = 0;
            for rows in &bands {
                assert_eq!(rows.start, next, "{rect:?}: bands are contiguous");
                assert!(!rows.is_empty());
                assert!(rows.len() * width * 4 < BAND_BYTES, "{rect:?}: {rows:?}");
                next = rows.end;
            }
            assert_eq!(next, rect[3] - rect[1], "{rect:?}: bands cover the cell");
        }
    }

    #[test]
    fn a_cell_at_device_x_125_at_factor_1_25_is_drawn_at_logical_x_100_2() {
        let bounds = cell([125, 0, 250, 50]).bounds(&(0..50), [0, 0], 1.25);
        assert_eq!(bounds.x, 100.2);
        assert_eq!(bounds.width, 100.0);
    }

    #[test]
    fn a_widget_at_logical_x_10_3_draws_its_first_cell_at_device_15() {
        let k = 1.5_f32;
        let origin = (10.3_f32 * k).round() as i32;
        assert_eq!(origin, 15);
        let bounds = cell([0, 0, 30, 30]).bounds(&(0..30), [origin, 0], k);
        assert_eq!(bounds.x, (15.0 + NUDGE) / 1.5);
        assert!((bounds.x - 10.1667).abs() < 1e-4);
        // Back to device px: tiny-skia truncates, wgpu adds 0.001 and
        // rounds; both land on 15, where a half-pixel bias rounds to 16.
        assert_eq!((bounds.x * k) as i32, 15);
        assert_eq!((bounds.x * k + 0.001).round(), 15.0);
        assert_eq!(((15.0 + 0.5) / k * k + 0.001_f32).round(), 16.0);
    }

    #[test]
    fn band_bounds_start_at_their_rows() {
        let at = cell([40, 30, 1000, 670]).bounds(&(545..640), [3, 4], 2.0);
        assert_eq!(
            at,
            Rectangle {
                x: (43.0 + NUDGE) / 2.0,
                y: (579.0 + NUDGE) / 2.0,
                width: 480.0,
                height: 47.5,
            }
        );
    }

    #[test]
    fn an_area_of_204_8_by_100_renders_256_by_125_at_1_25() {
        let list = List {
            size: Size::new(204.8, 100.0),
            ..List::default()
        };
        let bounds = Bounds::of(&list, &mut Outlines::default());
        let area = Rect::new(0.0, 0.0, 204.8, 100.0);
        let image = raster::Renderer::new(raster::Settings::default())
            .render(&list, &bounds, area, 1.25)
            .unwrap();
        assert_eq!((image.width, image.height), (256, 125));
    }

    #[test]
    fn bands_match_the_whole_cell() {
        // The two-panel figure at 480 × 320 at 2×: one 960 × 640 cell.
        let fig = fixture::two_panels();
        let size = Size::new(480.0, 320.0);
        let list = fig.draw_with(
            &mut state::cache(),
            size,
            Target::Screen { scale: 2.0 },
            &Held::default(),
        );
        let bounds = Bounds::of(&list, &mut Outlines::default());
        let mut raster = raster::Renderer::new(raster::Settings::default());
        let whole = raster
            .render(
                &list,
                &bounds,
                Rect::from_origin_size((0.0, 0.0), size),
                2.0,
            )
            .unwrap();
        let of = list::Cell {
            rect: Rect::new(0.0, 0.0, 960.0, 640.0),
            parts: Vec::new(),
            hash: 0,
            work: 0,
        };
        let cell = Cell::render(&of, &list, &bounds, 2.0, &mut raster);
        let images = cell.images();
        let rows: Vec<_> = images.iter().map(|(rows, ..)| rows.clone()).collect();
        assert_eq!(rows, [0..545, 545..640]);
        assert_eq!(images[0].3.len(), 2_092_800);
        let stitched: Vec<u8> = images.iter().flat_map(|image| image.3.to_vec()).collect();
        let a = pixels::Image::new(960, 640, &whole.rgba).unwrap();
        let b = pixels::Image::new(960, 640, &stitched).unwrap();
        let diff = pixels::Diff::between(&a, &b, 1).unwrap();
        println!("bands against the whole cell: max {}/255", diff.max());
        assert!(diff.max() <= 1, "max {}/255", diff.max());
    }

    #[test]
    fn partition_rects_convert_to_device_edges() {
        assert_eq!(
            super::device(Rect::new(0.0, 125.0, 960.0, 640.0)),
            [0, 125, 960, 640]
        );
    }
}
