//! [`Job`]: one cell or tile to rasterize, with all its bands, from a list
//! shared with the other jobs of its refresh. [`Done`] is its result.

use std::ops::Range;
use std::sync::Arc;

use iced_core::time::{Duration, Instant};
use rustd3_figure::kurbo::Rect;
use rustd3_figure::list::{Bounds, List};
use rustd3_figure::raster;

use crate::plot::budget::Kind;
use crate::plot::preview::Basis;
use crate::plot::{probe, slot};

/// One cell or tile of one refresh to render, inline or on the pool.
pub(crate) struct Job {
    pub(super) target: probe::Target,
    /// Where the image goes.
    pub(super) spot: Spot,
    pub(super) generation: u64,
    /// The cell content hash this job renders.
    pub(super) hash: u64,
    pub(super) list: Arc<List>,
    pub(super) bounds: Arc<Bounds>,
    /// The rect in device px, relative to the widget's device origin.
    pub(super) rect: [i32; 4],
    /// Device rows of each band, relative to the cell.
    pub(super) bands: Vec<Range<i32>>,
    /// Device px per list px.
    pub(super) scale: f64,
    /// The cell's work items, echoed into its `Raster` sample.
    pub(super) work: u64,
    /// Whether its markers are splatted (`raster::Renderer::splat`). A
    /// plot area's tiles splat when the area has a markers command of at
    /// least [`SPLAT`](crate::plot::state::SPLAT) points.
    pub(super) splat: bool,
    /// Whether it checks the image its slot shows (`Caching::Verify`)
    /// instead of replacing it. A check's `generation` and `hash` are the
    /// image's.
    pub(super) check: bool,
    /// The probe's clock and its reading at planning time.
    pub(super) enqueued: Option<Stamp>,
    /// For a plot-area cell, the view of the handle leading each axis at
    /// planning time, for previews of its image.
    pub(super) view: [Option<Basis>; 2],
}

/// Where a job's image goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Spot {
    /// A run of gutter cells, by its index among the runs.
    Gutter(usize),
    /// Tile `(i, j)` of plot area `area`'s grid numbered `grid`, planned
    /// with the grid's origin at `at` (device px).
    Tile {
        area: usize,
        grid: u64,
        i: i32,
        j: i32,
        at: [i32; 2],
    },
    /// What stays put over plot area `area`'s tiles, over its job's rect.
    Overlay { area: usize },
}

/// A probe clock and one reading of it.
#[derive(Clone, Copy)]
pub(crate) struct Stamp {
    pub(super) clock: fn() -> Instant,
    pub(super) at: Instant,
}

/// A rendered job: one result per band, the time it took, and its timing
/// with a probe.
pub(crate) struct Done {
    pub(super) target: probe::Target,
    pub(super) spot: Spot,
    pub(super) generation: u64,
    pub(super) hash: u64,
    pub(super) rect: [i32; 4],
    /// Device rows of each band, relative to the cell.
    pub(super) rows: Vec<Range<i32>>,
    pub(super) bands: Vec<Result<raster::Image, raster::Error>>,
    pub(super) work: u64,
    pub(super) check: bool,
    pub(super) queued: Option<Duration>,
    pub(super) duration: Option<Duration>,
    /// The render's time on the monotonic clock, which the cost model
    /// fits whether or not a probe is attached.
    pub(super) elapsed: Duration,
    pub(super) view: [Option<Basis>; 2],
}

impl Spot {
    /// The kind of render the cost model fits a job for this spot with: a
    /// tile, or a cell for a gutter run and an overlay.
    pub(crate) fn kind(self) -> Kind {
        match self {
            Spot::Tile { .. } => Kind::Tile,
            Spot::Gutter(_) | Spot::Overlay { .. } => Kind::Cell,
        }
    }
}

impl Stamp {
    /// `clock` and its reading at this instant.
    pub(crate) fn now(clock: fn() -> Instant) -> Stamp {
        Stamp { clock, at: clock() }
    }
}

impl Job {
    /// Renders every band with `renderer`, splatting markers as the job
    /// says. Each band is the list area of its device rows divided by the
    /// scale, which renders exactly the rows' device size. The render is
    /// timed for the cost model, and with a stamp also by the stamp's
    /// clock.
    pub(crate) fn run(self, renderer: &mut raster::Renderer) -> Done {
        let start = self.enqueued.map(|stamp| Stamp::now(stamp.clock));
        renderer.splat(self.splat);
        let k = self.scale;
        let [x0, y0, x1, _] = self.rect.map(f64::from);
        let (bands, elapsed) = timed(|| {
            self.bands
                .iter()
                .map(|rows| {
                    let (top, bottom) = (y0 + f64::from(rows.start), y0 + f64::from(rows.end));
                    let area = Rect::new(x0 / k, top / k, x1 / k, bottom / k);
                    renderer.render(&self.list, &self.bounds, area, k)
                })
                .collect()
        });
        let end = start.map(|stamp| Stamp::now(stamp.clock));
        Done {
            target: self.target,
            spot: self.spot,
            generation: self.generation,
            hash: self.hash,
            rect: self.rect,
            rows: self.bands,
            bands,
            work: self.work,
            check: self.check,
            queued: self
                .enqueued
                .zip(start)
                .map(|(enqueued, start)| start.at.saturating_duration_since(enqueued.at)),
            duration: start
                .zip(end)
                .map(|(start, end)| end.at.saturating_duration_since(start.at)),
            elapsed,
            view: self.view,
        }
    }
}

/// `render`'s result and how long it took on the monotonic clock.
///
/// The `Plot` widget times its own renders for its cost model, and this is
/// the one place it reads a clock it was not given. What it renders never
/// depends on the reading. The reading decides only whether later jobs
/// render inline or on the pool, which the frame interval already decides.
#[expect(
    clippy::disallowed_methods,
    reason = "the Plot widget times its own renders for its cost model"
)]
fn timed<T>(render: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let result = render();
    (result, start.elapsed())
}

impl Done {
    /// Device pixels of the cell.
    pub(crate) fn pixels(&self) -> u64 {
        slot::pixels(self.rect)
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
            spot: super::Spot::Gutter(0),
            generation: 1,
            hash: 0,
            list: Arc::new(list),
            bounds: Arc::new(bounds),
            rect,
            bands: slot::bands(rect),
            scale: 2.0,
            work: 0,
            splat: false,
            check: false,
            enqueued: None,
            view: [None; 2],
        };
        let done = job.run(&mut raster);
        assert_eq!(done.rows, [0..545, 545..640]);
        assert_eq!(
            (done.pixels(), done.queued, done.duration),
            (614_400, None, None)
        );
        // Without a probe the render is still timed, for the cost model.
        assert!(done.elapsed > std::time::Duration::ZERO);
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
