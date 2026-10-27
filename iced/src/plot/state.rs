//! [`State`]: what the widget keeps in iced's widget tree between frames,
//! and the refresh that renders the cells whose content changed.

use std::sync::Arc;

use iced_core::shell::Waker;
use iced_core::time::Instant;
use iced_core::{Point, Rectangle, Size, image};
use rustd3_figure::list::{self, Bounds, Outlines};
use rustd3_figure::{Cache, Figure, Held, Target, Theme, hit, kurbo, raster};

use crate::plot::job::{Done, Job, Stamp};
use crate::plot::overlay::{self, Overlay};
use crate::plot::probe::{self, Probe, Sample, Stage};
use crate::plot::slot::{self, Band, Slot};
use crate::plot::worker::Worker;
use crate::plot::{Render, handle};

/// The widget's tree state: the figure's cache, the rasterizer, and the
/// slots of the partition's cells. It borrows nothing of the figure.
pub(crate) struct State {
    /// Laid-out labels, thinned lines and `Fit::Visible` memory.
    cache: Cache,
    /// vello_cpu with its own pool, for cells rendered inline.
    raster: raster::Renderer,
    /// Glyph outlines the list's command bounds are measured with.
    outlines: Outlines,
    /// The widget's logical size from the last layout.
    size: Size,
    /// The window's factor from `Opened` or `Rescaled`.
    window: Option<f32>,
    /// The factor the slots were planned at; `None` before the first
    /// refresh.
    factor: Option<f32>,
    /// Whether the figure was rebuilt or the widget resized since the
    /// last refresh.
    stale: bool,
    /// The partition's cells, row-major, and what each shows.
    slots: Vec<Slot>,
    /// Bumped whenever the content the slots hold changes.
    generation: u64,
    /// The generation that built the slots: results rendered for an older
    /// one belong to another partition.
    floor: u64,
    /// `RedrawRequested` events handled.
    frame: u64,
    /// Plot areas in device px, relative to the widget's device origin.
    areas: Vec<[i32; 4]>,
    /// The points the marks draw, list px, for hover.
    hits: hit::Index,
    /// The crosshair and highlight where the pointer is.
    overlay: Overlay,
    /// The render thread, started with the first job set it gets.
    worker: Option<Worker>,
}

/// How a `RedrawRequested` renders and reports what it did.
pub(crate) struct Pipeline<'a> {
    /// Where changed cells render.
    pub(crate) render: Render,
    /// Where stage timings and frame reports go, if anywhere.
    pub(crate) probe: Option<&'a Probe>,
    /// Wakes the UI when the worker has results; the worker keeps a clone.
    pub(crate) waker: &'a Waker,
}

impl Default for State {
    fn default() -> Self {
        State {
            cache: cache(),
            raster: raster::Renderer::new(raster::Settings::parallel()),
            outlines: Outlines::default(),
            size: Size::ZERO,
            window: None,
            factor: None,
            stale: true,
            slots: Vec::new(),
            generation: 0,
            floor: 0,
            frame: 0,
            areas: Vec::new(),
            hits: hit::Index::default(),
            overlay: Overlay::default(),
            worker: None,
        }
    }
}

impl State {
    /// A new view built a new figure, which may or may not differ.
    pub(crate) fn rebuilt(&mut self) {
        self.stale = true;
    }

    /// The layout's logical size; a new one marks the list for rebuilding.
    pub(crate) fn resize(&mut self, size: Size) {
        if size != self.size {
            self.size = size;
            self.stale = true;
        }
    }

    /// The window's factor, from `Opened` or `Rescaled`.
    pub(crate) fn rescale(&mut self, factor: f32) {
        self.window = Some(factor);
    }

    /// The window's factor, when a window event gave one.
    pub(crate) fn window_factor(&self) -> Option<f32> {
        self.window
    }

    /// Handles one `RedrawRequested(now)`: counts the frame, refreshes
    /// `figure` at `factor` when it was rebuilt, the widget resized or the
    /// factor changed, and reports what the following draw shows to the
    /// pipeline's probe. Returns the number of cells whose content changed.
    pub(crate) fn redraw(
        &mut self,
        figure: &Figure<'_>,
        factor: f32,
        now: Instant,
        pipeline: &Pipeline<'_>,
    ) -> usize {
        self.frame += 1;
        let changed = self.refresh(figure, factor, pipeline);
        let report = self.report(now);
        if let Some(probe) = pipeline.probe {
            probe.send(report);
        }
        changed
    }

    /// Resolves `figure` once at the widget's size for the screen at
    /// `factor` and draws it, cuts the list into cells at every plot-area
    /// edge, and renders the cells whose content hash changed: inline,
    /// shown at once, or as a job set that replaces the worker's pending
    /// one. Returns how many changed.
    fn refresh(&mut self, figure: &Figure<'_>, factor: f32, pipeline: &Pipeline<'_>) -> usize {
        if !self.stale && self.factor == Some(factor) {
            return 0;
        }
        self.stale = false;
        self.factor = Some(factor);
        let k = f64::from(factor);
        let size = kurbo::Size::new(f64::from(self.size.width), f64::from(self.size.height));
        if !((size.width * k).round() >= 1.0 && (size.height * k).round() >= 1.0) {
            self.slots.clear();
            self.areas.clear();
            self.hits = hit::Index::default();
            return 0;
        }
        let probe = pipeline.probe;
        let target = Target::Screen { scale: k };
        let start = probe.map(Probe::now);
        self.cache.begin();
        let resolved = figure.resolve(size, &mut self.cache, &Held::default(), target);
        let middle = probe.map(Probe::now);
        let list = figure.draw_resolved(&mut self.cache, &resolved);
        if let (Some(probe), Some(start), Some(middle)) = (probe, start, middle) {
            let end = probe.now();
            for (stage, from, to) in [(Stage::Resolve, start, middle), (Stage::List, middle, end)] {
                probe.send(Sample::Stage {
                    frame: self.frame,
                    stage,
                    duration: to.saturating_duration_since(from),
                });
            }
        }
        let (xs, ys) = resolved.edges();
        let cells = list::cells(&list, &xs, &ys, k);
        let changed = self.plan(&cells.cells);
        if changed.is_empty() {
            return 0;
        }
        self.areas = resolved
            .plot_areas()
            .iter()
            .map(|area| [area.x0, area.y0, area.x1, area.y1].map(|edge| (edge * k).round() as i32))
            .collect();
        self.hits = figure.hits(&resolved, &mut self.cache);
        let bounds = Arc::new(Bounds::of(&list, &mut self.outlines));
        let list = Arc::new(list);
        let enqueued = probe.map(|probe| Stamp::now(probe.clock()));
        let generation = self.generation;
        let job = |index: usize, rect: [i32; 4]| {
            let cell = &cells.cells[index];
            Job {
                target: probe::Target::Cell(
                    u32::try_from(index).expect("invariant: fewer than 2^32 cells"),
                ),
                generation,
                hash: cell.hash,
                list: Arc::clone(&list),
                bounds: Arc::clone(&bounds),
                rect,
                bands: slot::bands(rect),
                scale: k,
                work: cell.work,
                enqueued,
            }
        };
        let inline = match pipeline.render {
            Render::Inline => true,
            Render::Worker => !self.start(pipeline.waker),
        };
        if inline {
            for &index in &changed {
                let done = job(index, self.slots[index].rect()).run(&mut self.raster);
                self.land(done, true, probe);
            }
            // Nothing left for the worker: its rest is out of date.
            if let Some(worker) = &self.worker
                && self.slots.iter().all(|slot| slot.pending().is_none())
            {
                worker.replace(Vec::new());
            }
        } else {
            // The new set replaces the pending one, whose unfinished jobs
            // the worker abandons: every slot still waiting renders again.
            let mut order: Vec<usize> = (0..self.slots.len())
                .filter(|&index| self.slots[index].pending().is_some())
                .collect();
            for &index in &order {
                self.slots[index].requeue(generation);
            }
            order.sort_by_key(|&index| !self.is_area(self.slots[index].rect()));
            let jobs = order
                .into_iter()
                .map(|index| job(index, self.slots[index].rect()))
                .collect();
            if let Some(worker) = &self.worker {
                worker.replace(jobs);
            }
        }
        changed.len()
    }

    /// Starts the worker unless it runs, waking the UI through `waker`.
    /// False when the OS cannot start a thread: cells then render inline.
    fn start(&mut self, waker: &Waker) -> bool {
        if self.worker.is_none() {
            self.worker = Worker::start(waker.clone()).ok();
        }
        self.worker.is_some()
    }

    /// Whether a device rect lies inside a plot area.
    fn is_area(&self, [x0, y0, x1, y1]: [i32; 4]) -> bool {
        self.areas
            .iter()
            .any(|&[ax0, ay0, ax1, ay1]| ax0 <= x0 && ay0 <= y0 && x1 <= ax1 && y1 <= ay1)
    }

    /// Applies the worker's results in arrival order, each reported to
    /// `probe`. True when any was shown, so the widget redraws.
    pub(crate) fn drain(&mut self, probe: Option<&Probe>) -> bool {
        let Some(worker) = &self.worker else {
            return false;
        };
        let mut shown = false;
        for done in worker.drain() {
            shown |= self.land(done, false, probe);
        }
        shown
    }

    /// Plans a refresh whose partition cells are `cells`: rebuilds the
    /// slots when their rect sequence differs (every slot new and
    /// changed), else finds the slots whose content hash differs from
    /// their cell's. If any changed, bumps the generation, stores their
    /// new hashes and marks them pending. Returns their indices.
    fn plan(&mut self, cells: &[list::Cell]) -> Vec<usize> {
        let rects: Vec<[i32; 4]> = cells.iter().map(|cell| slot::device(cell.rect)).collect();
        let same = self.slots.len() == rects.len()
            && self.slots.iter().zip(&rects).all(|(s, r)| s.rect() == *r);
        let changed: Vec<usize> = if same {
            self.slots
                .iter()
                .zip(cells)
                .enumerate()
                .filter(|(_, (slot, cell))| slot.hash() != cell.hash)
                .map(|(index, _)| index)
                .collect()
        } else {
            self.slots = rects
                .iter()
                .zip(cells)
                .map(|(&rect, cell)| Slot::new(rect, cell.hash))
                .collect();
            self.floor = self.generation + 1;
            (0..cells.len()).collect()
        };
        if !changed.is_empty() {
            self.generation += 1;
            for &index in &changed {
                self.slots[index].change(cells[index].hash, self.generation);
            }
        }
        changed
    }

    /// Applies a rendered job, reported to `probe` as rendered `inline` or
    /// not: shown when it belongs to this partition and is newer than its
    /// slot's image. True when it is shown.
    fn land(&mut self, done: Done, inline: bool, probe: Option<&Probe>) -> bool {
        if let Some(probe) = probe {
            probe.send(Sample::Raster {
                frame: self.frame,
                generation: done.generation,
                target: done.target,
                pixels: u32::try_from(done.pixels()).unwrap_or(u32::MAX),
                work: done.work,
                queued: done.queued.unwrap_or_default(),
                duration: done.duration.unwrap_or_default(),
                inline,
            });
        }
        let probe::Target::Cell(index) = done.target;
        let slot = usize::try_from(index)
            .ok()
            .and_then(|index| self.slots.get_mut(index));
        let Some(slot) = slot.filter(|_| done.generation >= self.floor) else {
            return false;
        };
        let bands: Result<Vec<Band>, raster::Error> = done
            .rows
            .into_iter()
            .zip(done.bands)
            .map(|(rows, band)| band.map(|image| Band::new(rows, handle::from_image(image))))
            .collect();
        // A cell renders unless its list fails `list::check`.
        debug_assert!(
            bands.is_ok(),
            "a cell failed to render: {:?}",
            bands.as_ref().err()
        );
        slot.land(done.generation, done.hash, bands.ok())
    }

    /// What the following draw shows; reports each newly shown image's
    /// bytes once.
    fn report(&mut self, at: Instant) -> Sample {
        let count = |test: fn(&Slot) -> bool| {
            u32::try_from(self.slots.iter().filter(|s| test(s)).count()).unwrap_or(u32::MAX)
        };
        let (missing, previews) = (count(Slot::is_missing), count(Slot::is_preview));
        let images = u32::try_from(self.slots.len()).unwrap_or(u32::MAX) - missing;
        Sample::Frame {
            frame: self.frame,
            at,
            generation: self.generation,
            images,
            missing,
            previews,
            uploaded: self.slots.iter_mut().map(Slot::take_fresh).sum(),
        }
    }

    /// Hovers at `at` (logical, in the window; `None` when the pointer left
    /// or something above covers it) over a widget laid out at `bounds`,
    /// showing what `shown` asks for. True when the overlay changed, so the
    /// widget redraws only then; no cell renders.
    pub(crate) fn hover(
        &mut self,
        at: Option<Point>,
        bounds: Rectangle,
        shown: overlay::Shown,
    ) -> bool {
        let Some(factor) = self.factor else {
            return false;
        };
        let [x0, y0] = origin(bounds, factor);
        let device = at.map(|p| Point::new(p.x * factor - x0 as f32, p.y * factor - y0 as f32));
        let next = Overlay::hover(device, &self.areas, &self.hits, factor, shown);
        let changed = next != self.overlay;
        self.overlay = next;
        changed
    }

    /// Draws every slot's image (the theme background where a slot has
    /// none), then the overlay in the theme's ink, for a widget laid out at
    /// `bounds`.
    pub(crate) fn draw<Renderer>(&self, renderer: &mut Renderer, bounds: Rectangle, theme: &Theme)
    where
        Renderer: image::Renderer<Handle = image::Handle>,
    {
        let Some(factor) = self.factor else {
            return;
        };
        let origin = origin(bounds, factor);
        let background = overlay::color(theme.get_background());
        for slot in &self.slots {
            slot.draw(renderer, origin, factor, background);
        }
        self.overlay.draw(renderer, origin, factor, theme.get_ink());
    }

    /// Whether `at` (logical, in the window) lies over a plot area of a
    /// widget laid out at `bounds`.
    pub(crate) fn is_over_area(&self, at: Point, bounds: Rectangle) -> bool {
        let Some(factor) = self.factor else {
            return false;
        };
        let [x0, y0] = origin(bounds, factor);
        let (x, y) = (at.x * factor - x0 as f32, at.y * factor - y0 as f32);
        self.areas.iter().any(|&[ax0, ay0, ax1, ay1]| {
            (ax0 as f32..ax1 as f32).contains(&x) && (ay0 as f32..ay1 as f32).contains(&y)
        })
    }
}

/// The widget's device origin: its layout position times the factor,
/// rounded, so that a widget at a fractional logical offset still puts
/// every cell on whole device pixels.
fn origin(bounds: Rectangle, factor: f32) -> [i32; 2] {
    [bounds.x, bounds.y].map(|v| (v * factor).round() as i32)
}

/// A figure cache over the bundled fonts.
#[cfg(feature = "fonts")]
pub(super) fn cache() -> Cache {
    Cache::default()
}

/// Without the `fonts` feature, a figure cache over the fonts
/// rustd3-figure has, which may be none: labels then draw no glyphs.
#[cfg(not(feature = "fonts"))]
pub(super) fn cache() -> Cache {
    let fonts = rustd3_figure::text::Fonts::with(Vec::<Vec<u8>>::new())
        .expect("invariant: no font files, no font error");
    Cache::new(std::sync::Arc::new(fonts))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, mpsc};

    use iced_core::shell::Waker;
    use iced_core::time::Instant;
    use iced_core::{Point, Rectangle, Size};
    use rustd3_figure::{Figure, Held, Target, kurbo};

    use super::{Pipeline, State};
    use crate::plot::overlay::{Overlay, Shown};
    use crate::plot::probe::{Probe, Sample, Stage};
    use crate::plot::{Render, fixture};

    /// The factors layout, snapping and placement are checked at.
    const FACTORS: [f32; 4] = [1.0, 1.25, 1.5, 2.0];

    #[expect(clippy::disallowed_methods, reason = "test clock")]
    fn clock() -> Instant {
        Instant::now()
    }

    /// A state laid out at `width` × `height` logical px.
    fn state(width: f32, height: f32) -> State {
        let mut state = State::default();
        state.resize(Size::new(width, height));
        state
    }

    /// One inline `RedrawRequested` without a probe; the number of cells
    /// whose content changed.
    fn refresh(state: &mut State, fig: &Figure<'_>, k: f32) -> usize {
        let pipeline = Pipeline {
            render: Render::Inline,
            probe: None,
            waker: &Waker::noop(),
        };
        state.redraw(fig, k, clock(), &pipeline)
    }

    /// One inline `RedrawRequested` reported to a fresh probe; its
    /// samples.
    fn probed(state: &mut State, fig: &Figure<'_>, k: f32) -> Vec<Sample> {
        let (tx, rx) = mpsc::channel();
        let probe = Probe::new(clock, tx);
        let pipeline = Pipeline {
            render: Render::Inline,
            probe: Some(&probe),
            waker: &Waker::noop(),
        };
        state.redraw(fig, k, clock(), &pipeline);
        rx.try_iter().collect()
    }

    /// A waker that counts its wakes.
    fn counting() -> (Waker, Arc<AtomicUsize>) {
        let wakes = Arc::new(AtomicUsize::new(0));
        let waker = Waker::new({
            let wakes = Arc::clone(&wakes);
            move || {
                wakes.fetch_add(1, Ordering::SeqCst);
            }
        });
        (waker, wakes)
    }

    /// Drains the worker's results into `state` after each wake, reported
    /// to `probe`, until no slot waits for a render; at most ten seconds
    /// per wake.
    fn settle(state: &mut State, wakes: &AtomicUsize, probe: Option<&Probe>) {
        let mut seen = 0;
        while state.slots.iter().any(|slot| slot.pending().is_some()) {
            let woke = (0..10_000).any(|_| {
                wakes.load(Ordering::SeqCst) > seen || {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    false
                }
            });
            assert!(woke, "no wake within ten seconds");
            seen = wakes.load(Ordering::SeqCst);
            state.drain(probe);
        }
    }

    /// The targets of the raster samples among `samples`.
    fn rasters(samples: &[Sample]) -> Vec<u32> {
        samples
            .iter()
            .filter_map(|sample| match sample {
                Sample::Raster {
                    target: crate::plot::probe::Target::Cell(cell),
                    ..
                } => Some(*cell),
                Sample::Stage { .. } | Sample::Frame { .. } => None,
            })
            .collect()
    }

    #[test]
    fn cells_partition_the_canvas() {
        let fig = fixture::grid();
        for k in FACTORS {
            let mut state = state(640.0, 480.0);
            let rendered = refresh(&mut state, &fig, k);
            let canvas = [0, 0, (640.0 * k).round() as i32, (480.0 * k).round() as i32];
            let rects: Vec<[i32; 4]> = state.slots.iter().map(|slot| slot.rect()).collect();
            assert_eq!(rendered, rects.len());
            // Four plot areas cut each axis into at most nine tracks.
            assert!(rects.len() > 4, "k = {k}: {} cells", rects.len());
            let area = |[x0, y0, x1, y1]: [i32; 4]| i64::from(x1 - x0) * i64::from(y1 - y0);
            for r in &rects {
                assert!(r[0] < r[2] && r[1] < r[3], "k = {k}: empty cell {r:?}");
                assert!(
                    r[0] >= canvas[0]
                        && r[1] >= canvas[1]
                        && r[2] <= canvas[2]
                        && r[3] <= canvas[3],
                    "k = {k}: {r:?} outside {canvas:?}"
                );
            }
            for (i, a) in rects.iter().enumerate() {
                for b in &rects[i + 1..] {
                    let overlap =
                        a[0].max(b[0]) < a[2].min(b[2]) && a[1].max(b[1]) < a[3].min(b[3]);
                    assert!(!overlap, "k = {k}: {a:?} and {b:?} overlap");
                }
            }
            // Disjoint and inside the canvas, they cover it when their
            // areas add up to its area.
            let covered: i64 = rects.iter().copied().map(area).sum();
            assert_eq!(covered, area(canvas), "k = {k}");
        }
    }

    #[test]
    fn rebuild_of_an_equal_figure_renders_nothing() {
        let mut state = state(640.0, 480.0);
        let first = probed(&mut state, &fixture::grid(), 1.25);
        assert_eq!(rasters(&first).len(), state.slots.len());
        let shown: Vec<_> = state.slots.iter().map(|slot| slot.images()).collect();
        // The view is rebuilt: a new figure, equal to the last.
        state.rebuilt();
        let again = probed(&mut state, &fixture::grid(), 1.25);
        assert!(rasters(&again).is_empty(), "{again:?}");
        let kept: Vec<_> = state.slots.iter().map(|slot| slot.images()).collect();
        assert_eq!(kept, shown);
        assert_eq!(state.generation, 1);
        // Nothing rebuilt, resized or rescaled: no refresh at all.
        assert_eq!(refresh(&mut state, &fixture::grid(), 1.25), 0);
        // A new size renders every cell again.
        state.resize(Size::new(600.0, 480.0));
        assert!(refresh(&mut state, &fixture::grid(), 1.25) > 0);
        assert_eq!(state.generation, 2);
    }

    #[test]
    fn unchanged_cells_are_skipped() {
        let dots: Vec<(f64, f64)> = (1..=8).map(|i| (f64::from(i), f64::from(9 - i))).collect();
        let mut moved = dots.clone();
        moved[3] = (4.0, 6.5);
        let mut state = state(480.0, 320.0);
        let first = probed(&mut state, &fixture::pair(dots), 2.0);
        assert_eq!(rasters(&first).len(), state.slots.len());
        // The dots panel's plot area is one cell.
        let area = state.areas[1];
        let cell = state
            .slots
            .iter()
            .position(|slot| slot.rect() == area)
            .expect("panel 1's plot area is a cell");
        state.rebuilt();
        let again = probed(&mut state, &fixture::pair(moved), 2.0);
        assert_eq!(rasters(&again), [u32::try_from(cell).unwrap()]);
        assert_eq!(state.generation, 2);
        let generations: Vec<_> = state.slots.iter().map(|slot| slot.generation()).collect();
        for (i, generation) in generations.into_iter().enumerate() {
            let expected = if i == cell { 2 } else { 1 };
            assert_eq!(generation, Some(expected), "slot {i}");
        }
    }

    #[test]
    fn probe_reports_every_stage() {
        let mut state = state(640.0, 480.0);
        let samples = probed(&mut state, &fixture::grid(), 2.0);
        let stages: Vec<Stage> = samples
            .iter()
            .filter_map(|sample| match sample {
                Sample::Stage {
                    frame: 1, stage, ..
                } => Some(*stage),
                Sample::Stage { .. } | Sample::Raster { .. } | Sample::Frame { .. } => None,
            })
            .collect();
        assert_eq!(stages, [Stage::Resolve, Stage::List]);
        let cells = rasters(&samples);
        let expected: Vec<u32> = (0..u32::try_from(state.slots.len()).unwrap()).collect();
        assert_eq!(cells, expected, "one raster sample per cell, in order");
        let (mut pixels, mut work, mut ms) = (0, 0, 0.0);
        for sample in &samples {
            if let Sample::Raster {
                frame,
                generation,
                pixels: p,
                work: w,
                duration,
                inline,
                ..
            } = sample
            {
                assert_eq!((*frame, *generation, *inline), (1, 1, true));
                pixels += u64::from(*p);
                work += *w;
                ms += duration.as_secs_f64() * 1e3;
            }
        }
        assert_eq!(pixels, 1280 * 960);
        let frames: Vec<_> = samples
            .iter()
            .filter_map(|sample| match sample {
                Sample::Frame {
                    frame,
                    generation,
                    images,
                    missing,
                    previews,
                    uploaded,
                    ..
                } => Some((*frame, *generation, *images, *missing, *previews, *uploaded)),
                Sample::Stage { .. } | Sample::Raster { .. } => None,
            })
            .collect();
        let slots = u32::try_from(state.slots.len()).unwrap();
        assert_eq!(frames, [(1, 1, slots, 0, 0, 1280 * 960 * 4)]);
        assert!(matches!(samples.last(), Some(Sample::Frame { .. })));
        println!(
            "inline: {} samples ({} rasters); {pixels} px and {work} items in {ms:.1} ms: {:.0} px/ms",
            samples.len(),
            cells.len(),
            pixels as f64 / ms
        );
        // A redraw that refreshes nothing still reports its frame.
        let idle = probed(&mut state, &fixture::grid(), 2.0);
        assert!(
            matches!(
                idle.as_slice(),
                [Sample::Frame {
                    frame: 2,
                    uploaded: 0,
                    ..
                }]
            ),
            "{idle:?}"
        );
        // The same refresh on the worker: the frame shows no image, and
        // every cell reports when its result is drained.
        let (tx, rx) = mpsc::channel();
        let probe = Probe::new(clock, tx);
        let (waker, wakes) = counting();
        let pipeline = Pipeline {
            render: Render::Worker,
            probe: Some(&probe),
            waker: &waker,
        };
        let mut state = self::state(640.0, 480.0);
        state.redraw(&fixture::grid(), 2.0, clock(), &pipeline);
        let planned: Vec<Sample> = rx.try_iter().collect();
        let stages = planned
            .iter()
            .filter(|sample| matches!(sample, Sample::Stage { .. }))
            .count();
        assert_eq!(stages, 2);
        assert!(rasters(&planned).is_empty());
        assert!(
            matches!(
                planned.last(),
                Some(Sample::Frame { frame: 1, generation: 1, images: 0, missing, previews: 0, uploaded: 0, .. })
                    if *missing == slots
            ),
            "{planned:?}"
        );
        settle(&mut state, &wakes, Some(&probe));
        let drained: Vec<Sample> = rx.try_iter().collect();
        let mut cells = rasters(&drained);
        cells.sort_unstable();
        assert_eq!(cells, expected, "one raster sample per cell");
        let (mut pixels, mut ms) = (0, 0.0);
        for sample in &drained {
            if let Sample::Raster {
                frame,
                inline,
                pixels: p,
                duration,
                ..
            } = sample
            {
                assert_eq!((*frame, *inline), (1, false));
                pixels += u64::from(*p);
                ms += duration.as_secs_f64() * 1e3;
            }
        }
        println!(
            "worker: {} samples after {} wakes; {pixels} px in {ms:.1} ms: {:.0} px/ms",
            drained.len(),
            wakes.load(Ordering::SeqCst),
            pixels as f64 / ms
        );
        state.redraw(&fixture::grid(), 2.0, clock(), &pipeline);
        let shown: Vec<Sample> = rx.try_iter().collect();
        assert!(
            matches!(
                shown.as_slice(),
                [Sample::Frame { frame: 2, generation: 1, images, missing: 0, previews: 0, uploaded, .. }]
                    if *images == slots && *uploaded == 1280 * 960 * 4
            ),
            "{shown:?}"
        );
    }

    #[test]
    fn every_slot_lands_after_a_replaced_set() {
        // A set replaced before it finished: the slots it would have
        // rendered render in the new set.
        let dots: Vec<(f64, f64)> = (1..=8).map(|i| (f64::from(i), f64::from(9 - i))).collect();
        let mut moved = dots.clone();
        moved[3] = (4.0, 6.5);
        let (waker, wakes) = counting();
        let pipeline = Pipeline {
            render: Render::Worker,
            probe: None,
            waker: &waker,
        };
        let mut state = state(480.0, 320.0);
        state.redraw(&fixture::pair(dots), 2.0, clock(), &pipeline);
        state.rebuilt();
        state.redraw(&fixture::pair(moved), 2.0, clock(), &pipeline);
        assert_eq!(state.generation, 2);
        settle(&mut state, &wakes, None);
        assert!(
            state
                .slots
                .iter()
                .all(|slot| !slot.is_missing() && !slot.is_preview())
        );
    }

    #[test]
    fn a_refresh_resolves_the_figure_once() {
        // One resolve of the grid over a warm cache: every label a hit.
        let fig = fixture::grid();
        let mut cache = super::cache();
        let (size, target) = (
            kurbo::Size::new(640.0, 480.0),
            Target::Screen { scale: 1.5 },
        );
        cache.begin();
        let _ = fig.resolve(size, &mut cache, &Held::default(), target);
        let warm = cache.stats().text.hits;
        cache.begin();
        let _ = fig.resolve(size, &mut cache, &Held::default(), target);
        let one = cache.stats().text.hits - warm;
        assert!(one > 0);
        let mut state = state(640.0, 480.0);
        refresh(&mut state, &fig, 1.5);
        let before = state.cache.stats().text.hits;
        state.rebuilt();
        assert_eq!(refresh(&mut state, &fixture::grid(), 1.5), 0);
        assert_eq!(state.cache.stats().text.hits - before, one);
    }

    #[test]
    fn cell_images_have_their_device_size() {
        let fig = fixture::grid();
        for k in [1.25, 1.5] {
            let mut state = state(640.0, 480.0);
            refresh(&mut state, &fig, k);
            for slot in &state.slots {
                let [x0, y0, x1, y1] = slot.rect();
                let images = slot.images();
                assert_eq!(images.len(), 1, "k = {k}: every cell is one band");
                let (rows, width, height, rgba) = &images[0];
                assert_eq!(*rows, 0..y1 - y0);
                assert_eq!(
                    (*width as i32, *height as i32),
                    (x1 - x0, y1 - y0),
                    "k = {k}"
                );
                assert_eq!(rgba.len(), (*width * *height * 4) as usize);
            }
        }
    }

    #[test]
    fn hover_renders_no_cell() {
        let fig = fixture::grid();
        let mut state = state(640.0, 480.0);
        assert!(refresh(&mut state, &fig, 1.25) > 0);
        let shown = Shown {
            crosshair: true,
            highlight: true,
        };
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(640.0, 480.0));
        let [x0, y0, x1, y1] = state.areas[0];
        let centre = Point::new((x0 + x1) as f32 / 2.5, (y0 + y1) as f32 / 2.5);
        assert!(state.hover(Some(centre), bounds, shown));
        assert!(!state.hover(Some(centre), bounds, shown), "the same place");
        // The redraw the hover asks for renders nothing, and neither does
        // the rebuild an app's message would cause.
        assert_eq!(refresh(&mut state, &fig, 1.25), 0);
        state.rebuilt();
        assert_eq!(refresh(&mut state, &fixture::grid(), 1.25), 0);
        assert!(state.hover(None, bounds, shown), "the pointer left");
        assert_eq!(refresh(&mut state, &fig, 1.25), 0);
    }

    #[test]
    fn hovering_outside_every_plot_area_shows_nothing() {
        let mut state = state(640.0, 480.0);
        refresh(&mut state, &fixture::grid(), 1.0);
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(640.0, 480.0));
        let shown = Shown {
            crosshair: true,
            highlight: true,
        };
        assert!(!state.hover(Some(Point::new(1.0, 1.0)), bounds, shown));
        assert_eq!(state.overlay, Overlay::default());
        // With nothing shown, moving inside a plot area changes no part.
        let none = Shown {
            crosshair: false,
            highlight: false,
        };
        let [x0, y0, ..] = state.areas[0];
        let inside = Point::new(x0 as f32 + 3.0, y0 as f32 + 3.0);
        assert!(state.hover(Some(inside), bounds, none), "the panel changed");
        let further = Point::new(x0 as f32 + 9.0, y0 as f32 + 9.0);
        assert!(!state.hover(Some(further), bounds, none));
    }

    #[test]
    fn a_widget_of_no_device_pixel_has_no_cells() {
        let mut state = state(0.0, 480.0);
        assert_eq!(refresh(&mut state, &fixture::grid(), 1.0), 0);
        assert!(state.slots.is_empty());
    }
}
