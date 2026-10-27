//! [`Slot`]: one cell of the widget's partition, the image it shows, the
//! generation that image was rendered for and the render it waits for.

use std::ops::Range;

use iced_core::image::{self, FilterMethod};
use iced_core::renderer::Quad;
use iced_core::{Color, Rectangle};
use rustd3_figure::kurbo::Rect;

use crate::plot::{BAND_BYTES, NUDGE};

/// One cell of the partition: its device rect and current content hash,
/// what it shows and the render it waits for.
pub(crate) struct Slot {
    /// Device px [x0, y0, x1, y1], relative to the widget's device origin.
    rect: [i32; 4],
    /// Hash of the cell's current content.
    hash: u64,
    /// The image on screen, the generation it was rendered for and the
    /// content hash it shows.
    shown: Option<Shown>,
    /// The generation a render is pending for, if any.
    pending: Option<u64>,
}

/// A slot's image: its bands, the generation they were rendered for and
/// the content hash they show.
struct Shown {
    bands: Vec<Band>,
    generation: u64,
    hash: u64,
    /// Not yet reported as drawn for the first time.
    fresh: bool,
}

/// Whole device rows of a cell, rendered as one image under `BAND_BYTES`.
pub(crate) struct Band {
    /// Device rows relative to the cell.
    rows: Range<i32>,
    handle: image::Handle,
}

impl Band {
    /// Rows `rows` of a cell, shown by `handle`.
    pub(crate) fn new(rows: Range<i32>, handle: image::Handle) -> Band {
        Band { rows, handle }
    }
}

impl Slot {
    /// A slot of `rect` holding content `hash`, with no image yet.
    pub(crate) fn new(rect: [i32; 4], hash: u64) -> Slot {
        Slot {
            rect,
            hash,
            shown: None,
            pending: None,
        }
    }

    /// The slot's device rect.
    pub(crate) fn rect(&self) -> [i32; 4] {
        self.rect
    }

    /// The slot's current content hash.
    pub(crate) fn hash(&self) -> u64 {
        self.hash
    }

    /// The generation a render is pending for.
    pub(crate) fn pending(&self) -> Option<u64> {
        self.pending
    }

    /// The cell's content changed to `hash` at `generation`, whose render
    /// is now pending.
    pub(crate) fn change(&mut self, hash: u64, generation: u64) {
        self.hash = hash;
        self.pending = Some(generation);
    }

    /// A pending render moves to `generation`: the job set it was in was
    /// replaced by one that renders it again.
    pub(crate) fn requeue(&mut self, generation: u64) {
        if self.pending.is_some() {
            self.pending = Some(generation);
        }
    }

    /// A render of content `hash` for `generation` finished; `bands` is
    /// `None` when it failed. The pending render clears when it is this
    /// one; the bands are shown iff `generation` is newer than the image
    /// on screen, or there is none. True when they are.
    pub(crate) fn land(&mut self, generation: u64, hash: u64, bands: Option<Vec<Band>>) -> bool {
        if self.pending == Some(generation) {
            self.pending = None;
        }
        let Some(bands) = bands else {
            return false;
        };
        if self
            .shown
            .as_ref()
            .is_some_and(|shown| generation <= shown.generation)
        {
            return false;
        }
        self.shown = Some(Shown {
            bands,
            generation,
            hash,
            fresh: true,
        });
        true
    }

    /// No image at all: the theme background shows.
    pub(crate) fn is_missing(&self) -> bool {
        self.shown.is_none()
    }

    /// An image of content other than the current: an older render shown
    /// until the newest lands.
    pub(crate) fn is_preview(&self) -> bool {
        self.shown
            .as_ref()
            .is_some_and(|shown| shown.hash != self.hash)
    }

    /// RGBA bytes of an image shown since the last call, which the next
    /// draw uploads; 0 when there is none.
    pub(crate) fn take_fresh(&mut self) -> u64 {
        let width = u64::try_from(self.rect[2] - self.rect[0]).unwrap_or(0);
        match &mut self.shown {
            Some(shown) if shown.fresh => {
                shown.fresh = false;
                shown
                    .bands
                    .iter()
                    .map(|band| u64::try_from(band.rows.len()).unwrap_or(0) * width * 4)
                    .sum()
            }
            Some(_) | None => 0,
        }
    }

    /// Draws the image with nearest filtering at bounds that land on its
    /// device pixels, clipped to them, for a widget whose device origin is
    /// `origin`; a slot with no image is filled with `background`.
    pub(crate) fn draw<Renderer>(
        &self,
        renderer: &mut Renderer,
        origin: [i32; 2],
        factor: f32,
        background: Color,
    ) where
        Renderer: image::Renderer<Handle = image::Handle>,
    {
        let Some(shown) = &self.shown else {
            let [x0, y0, x1, y1] = self.rect;
            let quad = Quad {
                bounds: Rectangle {
                    x: (origin[0] + x0) as f32 / factor,
                    y: (origin[1] + y0) as f32 / factor,
                    width: (x1 - x0) as f32 / factor,
                    height: (y1 - y0) as f32 / factor,
                },
                ..Quad::default()
            };
            renderer.fill_quad(quad, background);
            return;
        };
        for band in &shown.bands {
            let bounds = self.bounds(&band.rows, origin, factor);
            let image = image::Image::new(band.handle.clone()).filter_method(FilterMethod::Nearest);
            renderer.draw_image(image, bounds, bounds);
        }
    }

    /// Each band's rows and image: width, height and straight RGBA.
    #[cfg(test)]
    pub(crate) fn images(&self) -> Vec<(Range<i32>, u32, u32, iced_core::Bytes)> {
        self.shown
            .iter()
            .flat_map(|shown| &shown.bands)
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

    /// The generation of the image on screen.
    #[cfg(test)]
    pub(crate) fn generation(&self) -> Option<u64> {
        self.shown.as_ref().map(|shown| shown.generation)
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

/// Row ranges of the bands a cell of `rect` is drawn as: one band when its
/// RGBA stays under `BAND_BYTES`, else bands of `BAND_BYTES / (width × 4)
/// − 1` rows (at least one), the last one shorter, each strictly under the
/// limit.
pub(crate) fn bands(rect: [i32; 4]) -> Vec<Range<i32>> {
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

/// A partition cell's rect, whose edges are whole device px.
pub(crate) fn device(rect: Rect) -> [i32; 4] {
    [rect.x0, rect.y0, rect.x1, rect.y1].map(|edge| edge as i32)
}

#[cfg(test)]
mod tests {
    use iced_core::Rectangle;
    use iced_core::image::Handle;
    use rustd3_figure::kurbo::Rect;

    use super::{Band, Slot, bands};
    use crate::plot::{BAND_BYTES, NUDGE};

    /// A slot of `rect` showing nothing, its content hashed 0.
    fn slot(rect: [i32; 4]) -> Slot {
        Slot::new(rect, 0)
    }

    /// One band of a 4 × 2 cell.
    fn image() -> Vec<Band> {
        vec![Band::new(0..2, Handle::from_rgba(4, 2, vec![0_u8; 32]))]
    }

    #[test]
    fn a_960_by_640_cell_becomes_bands_of_545_and_95_rows() {
        let rect = [0, 0, 960, 640];
        assert_eq!(960 * 640 * 4, 2_457_600);
        assert_eq!(BAND_BYTES / (960 * 4) - 1, 545);
        assert_eq!(bands(rect), [0..545, 545..640]);
        assert_eq!(545 * 960 * 4, 2_092_800);
    }

    #[test]
    fn bands_stay_under_the_limit() {
        let one = bands([0, 0, 300, 200]);
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
            let bands = bands(rect);
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
        let bounds = slot([125, 0, 250, 50]).bounds(&(0..50), [0, 0], 1.25);
        assert_eq!(bounds.x, 100.2);
        assert_eq!(bounds.width, 100.0);
    }

    #[test]
    fn a_widget_at_logical_x_10_3_draws_its_first_cell_at_device_15() {
        let k = 1.5_f32;
        let origin = (10.3_f32 * k).round() as i32;
        assert_eq!(origin, 15);
        let bounds = slot([0, 0, 30, 30]).bounds(&(0..30), [origin, 0], k);
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
        let at = slot([40, 30, 1000, 670]).bounds(&(545..640), [3, 4], 2.0);
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
    fn partition_rects_convert_to_device_edges() {
        assert_eq!(
            super::device(Rect::new(0.0, 125.0, 960.0, 640.0)),
            [0, 125, 960, 640]
        );
    }

    #[test]
    fn a_newer_result_is_shown_and_stays_a_preview_until_the_newest_lands() {
        // Shown g = 3, pending g = 5, the result for g = 4 arrives.
        let mut slot = slot([0, 0, 4, 2]);
        slot.change(30, 3);
        assert!(slot.land(3, 30, Some(image())));
        slot.change(40, 4);
        slot.change(50, 5);
        assert!(slot.land(4, 40, Some(image())), "4 > 3: promoted");
        assert_eq!((slot.generation(), slot.pending()), (Some(4), Some(5)));
        assert!(slot.is_preview(), "content 40 is not the current 50");
        assert!(slot.land(5, 50, Some(image())));
        assert_eq!((slot.generation(), slot.pending()), (Some(5), None));
        assert!(!slot.is_preview());
    }

    #[test]
    fn an_older_result_is_dropped() {
        let mut slot = slot([0, 0, 4, 2]);
        slot.change(10, 1);
        slot.change(20, 2);
        assert!(slot.land(2, 20, Some(image())));
        assert!(!slot.land(1, 10, Some(image())), "1 < 2: dropped");
        assert!(!slot.land(2, 20, Some(image())), "2 is shown already");
        assert_eq!(slot.generation(), Some(2));
        assert!(!slot.is_preview());
    }

    #[test]
    fn an_unchanged_slot_is_current_whatever_its_generation() {
        // Panel 0's slot keeps its g = 6 image while g = 7 changes panel 1.
        let mut slot = slot([0, 0, 4, 2]);
        slot.change(60, 6);
        assert!(slot.land(6, 60, Some(image())));
        assert_eq!(slot.generation(), Some(6));
        assert!(!slot.is_preview() && !slot.is_missing());
    }

    #[test]
    fn a_slot_without_an_image_is_missing_not_a_preview() {
        let mut slot = slot([0, 0, 4, 2]);
        slot.change(10, 1);
        assert_eq!(slot.pending(), Some(1));
        assert!(slot.is_missing());
        assert!(!slot.is_preview());
    }

    #[test]
    fn a_failed_render_keeps_the_image_and_clears_its_pending_generation() {
        let mut slot = slot([0, 0, 4, 2]);
        slot.change(10, 1);
        assert!(slot.land(1, 10, Some(image())));
        slot.change(20, 2);
        assert!(!slot.land(2, 20, None));
        assert_eq!((slot.generation(), slot.pending()), (Some(1), None));
        assert!(slot.is_preview());
    }

    #[test]
    fn a_requeued_render_waits_for_the_new_generation() {
        let mut slot = slot([0, 0, 4, 2]);
        slot.change(10, 1);
        slot.requeue(3);
        assert!(
            slot.land(1, 10, Some(image())),
            "the old set's result shows"
        );
        assert_eq!(slot.pending(), Some(3));
        assert!(slot.land(3, 10, Some(image())));
        assert_eq!(slot.pending(), None);
        // Nothing pending, nothing to requeue.
        slot.requeue(4);
        assert_eq!(slot.pending(), None);
    }

    #[test]
    fn fresh_bytes_are_reported_once() {
        let mut slot = slot([0, 0, 4, 2]);
        assert_eq!(slot.take_fresh(), 0);
        slot.change(10, 1);
        assert!(slot.land(1, 10, Some(image())));
        assert_eq!(slot.take_fresh(), 32);
        assert_eq!(slot.take_fresh(), 0);
    }
}
