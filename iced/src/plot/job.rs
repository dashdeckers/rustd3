//! [`Job`]: one cell to rasterize, all its bands, from a list shared with
//! the other cells of its refresh; and [`Done`], its result.

use std::ops::Range;
use std::sync::Arc;

use iced_core::time::{Duration, Instant};
use rustd3_figure::kurbo::Rect;
use rustd3_figure::list::{Bounds, List};
use rustd3_figure::raster;

use crate::plot::probe;

/// One cell of one refresh to render, inline or on the worker.
pub(crate) struct Job {
    pub(super) target: probe::Target,
    pub(super) generation: u64,
    /// The cell content hash this job renders.
    pub(super) hash: u64,
    pub(super) list: Arc<List>,
    pub(super) bounds: Arc<Bounds>,
    /// Device px, relative to the widget's device origin.
    pub(super) rect: [i32; 4],
    /// Device rows of each band, relative to the cell.
    pub(super) bands: Vec<Range<i32>>,
    /// Device px per list px.
    pub(super) scale: f64,
    /// The cell's work items, echoed into its `Raster` sample.
    pub(super) work: u64,
    /// The probe's clock and its reading at planning time.
    pub(super) enqueued: Option<Stamp>,
}

/// A probe clock and one reading of it.
#[derive(Clone, Copy)]
pub(crate) struct Stamp {
    pub(super) clock: fn() -> Instant,
    pub(super) at: Instant,
}

/// A rendered job: one result per band, and its timing with a probe.
pub(crate) struct Done {
    pub(super) target: probe::Target,
    pub(super) generation: u64,
    pub(super) hash: u64,
    pub(super) rect: [i32; 4],
    /// Device rows of each band, relative to the cell.
    pub(super) rows: Vec<Range<i32>>,
    pub(super) bands: Vec<Result<raster::Image, raster::Error>>,
    pub(super) work: u64,
    pub(super) queued: Option<Duration>,
    pub(super) duration: Option<Duration>,
}

impl Stamp {
    /// `clock` read now.
    pub(crate) fn now(clock: fn() -> Instant) -> Stamp {
        Stamp { clock, at: clock() }
    }
}

impl Job {
    /// Renders every band with `renderer`: each band is the list area of
    /// its device rows divided by the scale, which renders exactly the
    /// rows' device size. With a stamp, the render is timed by its clock.
    pub(crate) fn run(self, renderer: &mut raster::Renderer) -> Done {
        let start = self.enqueued.map(|stamp| Stamp::now(stamp.clock));
        let k = self.scale;
        let [x0, y0, x1, _] = self.rect.map(f64::from);
        let bands = self
            .bands
            .iter()
            .map(|rows| {
                let (top, bottom) = (y0 + f64::from(rows.start), y0 + f64::from(rows.end));
                let area = Rect::new(x0 / k, top / k, x1 / k, bottom / k);
                renderer.render(&self.list, &self.bounds, area, k)
            })
            .collect();
        let end = start.map(|stamp| Stamp::now(stamp.clock));
        Done {
            target: self.target,
            generation: self.generation,
            hash: self.hash,
            rect: self.rect,
            rows: self.bands,
            bands,
            work: self.work,
            queued: self
                .enqueued
                .zip(start)
                .map(|(enqueued, start)| start.at.saturating_duration_since(enqueued.at)),
            duration: start
                .zip(end)
                .map(|(start, end)| end.at.saturating_duration_since(start.at)),
        }
    }
}

impl Done {
    /// Device pixels of the cell.
    pub(crate) fn pixels(&self) -> u64 {
        let [x0, y0, x1, y1] = self.rect;
        let side = |a: i32, b: i32| u64::try_from(b - a).unwrap_or(0);
        side(x0, x1) * side(y0, y1)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rustd3_figure::kurbo::{Rect, Size};
    use rustd3_figure::list::{Bounds, List, Outlines};
    use rustd3_figure::{Held, Target, raster};
    use rustd3_testkit::pixels;

    use super::Job;
    use crate::plot::{fixture, probe, slot, state};

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
        let rect = [0, 0, 960, 640];
        let job = Job {
            target: probe::Target::Cell(0),
            generation: 1,
            hash: 0,
            list: Arc::new(list),
            bounds: Arc::new(bounds),
            rect,
            bands: slot::bands(rect),
            scale: 2.0,
            work: 0,
            enqueued: None,
        };
        let done = job.run(&mut raster);
        assert_eq!(done.rows, [0..545, 545..640]);
        assert_eq!(
            (done.pixels(), done.queued, done.duration),
            (614_400, None, None)
        );
        let bands: Vec<raster::Image> = done.bands.into_iter().map(Result::unwrap).collect();
        assert_eq!(bands[0].rgba.len(), 2_092_800);
        let stitched: Vec<u8> = bands.iter().flat_map(|image| image.rgba.clone()).collect();
        let a = pixels::Image::new(960, 640, &whole.rgba).unwrap();
        let b = pixels::Image::new(960, 640, &stitched).unwrap();
        let diff = pixels::Diff::between(&a, &b, 1).unwrap();
        println!("bands against the whole cell: max {}/255", diff.max());
        assert!(diff.max() <= 1, "max {}/255", diff.max());
    }
}
