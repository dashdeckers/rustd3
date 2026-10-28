//! [`Probe`] and its [`Sample`]s: what each frame of a `Plot` cost and
//! what it showed, timed by a clock the app supplies.
//!
//! A `Plot` with a probe sends, per `RedrawRequested`, a [`Sample::Stage`]
//! for each UI-thread stage it ran, a [`Sample::Raster`] for each cell it
//! rendered inline, and one [`Sample::Frame`] describing the following
//! draw; cells rendered on the pool report their `Raster` sample when
//! their result is drained, on `Event::Waken`. Without a probe nothing is
//! sent and no clock is read.
//!
//! ```
//! use std::sync::mpsc;
//!
//! use iced_core::time::Instant;
//! use rustd3_iced::plot::Probe;
//!
//! /// The app's clock: library code reads none.
//! fn clock() -> Instant {
//!     Instant::now()
//! }
//!
//! let (tx, rx) = mpsc::channel();
//! let probe = Probe::new(clock, tx);
//! # let _ = (probe, rx);
//! ```

use std::sync::mpsc;

use iced_core::time::{Duration, Instant};

/// Where a `Plot` reports its frames: stage timings, raster times and
/// what each frame showed, timed by an app-supplied clock and sent to
/// `sink` as [`Sample`]s. The benchmark attributes frame time with it.
#[derive(Debug, Clone)]
pub struct Probe {
    clock: fn() -> Instant,
    sink: mpsc::Sender<Sample>,
}

/// One measurement of a `Plot`'s frame.
///
/// `frame` counts the `RedrawRequested` events the widget handled; a
/// sample sent between them carries the last one's count. `generation`
/// increments whenever the state the widget shows changes (a list whose
/// cell hashes differ from the stored ones).
#[derive(Debug, Clone, PartialEq)]
pub enum Sample {
    /// UI-thread work inside one `RedrawRequested`.
    Stage {
        /// The frame it ran in.
        frame: u64,
        /// What ran.
        stage: Stage,
        /// How long it took, by the probe's clock.
        duration: Duration,
    },
    /// One cell or tile rasterized (worker or inline), all its bands.
    Raster {
        /// The frame it was reported in: the one that rendered it inline,
        /// or the last before the drain that received a worker's result.
        frame: u64,
        /// The generation it was rendered for.
        generation: u64,
        /// The cell or tile.
        target: Target,
        /// Device pixels of the cell or tile.
        pixels: u32,
        /// The cell's `work` (Phase 17), for the cost model; a tile's
        /// share of its plot area's, by pixels.
        work: u64,
        /// Time between planning and the start of the render.
        queued: Duration,
        /// Time the render took.
        duration: Duration,
        /// Whether it ran on the UI thread.
        inline: bool,
    },
    /// What the `draw` after this `RedrawRequested` shows.
    Frame {
        /// The frame.
        frame: u64,
        /// That `RedrawRequested`'s instant (no clock call).
        at: Instant,
        /// The newest generation planned.
        generation: u64,
        /// Slots (the runs of gutter cells, the plot areas' visible tiles
        /// and their overlays) with an image of any generation; a tile
        /// without one of its own counts when its area's preview layer
        /// covers it.
        images: u32,
        /// Slots with no image at all (the theme background shows).
        missing: u32,
        /// Slots whose shown content hash differs from their current
        /// hash, gutters included: previews, an older render shown until
        /// the newest lands, and tiles the preview layer of an earlier
        /// grid stands in for. A slot whose content did not change is
        /// current whatever generation its image came from.
        previews: u32,
        /// Bytes of handles drawn for the first time.
        uploaded: u64,
    },
    /// Under `Caching::Verify`, a kept or translated slot whose image
    /// differed from a fresh render by more than 1/255; the fresh render
    /// replaced it.
    Mismatch {
        /// The frame it was reported in: the one that rendered it inline,
        /// or the last before the drain that received the render.
        frame: u64,
        /// The cell or tile.
        target: Target,
        /// The largest channel difference, of 255.
        max: u8,
    },
}

/// A UI-thread stage of a refresh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    /// Resolving the figure: layout, text and fitting.
    Resolve,
    /// Building the drawing list from the resolved figure.
    List,
}

/// What a raster sample rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    /// A cell outside every plot area, by its position in the partition's
    /// cells (row-major), stable while the partition's rect sequence is
    /// unchanged. Adjacent gutter cells of a row merged into one image
    /// report their first (leftmost) cell, with the pixels of all of
    /// them; what stays put over a plot area reports the cell holding its
    /// top left pixel.
    Cell(u32),
    /// A tile of a plot area: the area's position among the plot areas
    /// (panel order), and the tile's column and row on its grid, whose
    /// tile (0, 0) starts at the device position of the area's data
    /// anchor (its top-left corner until a gesture moves it).
    Tile {
        /// The plot area.
        area: u32,
        /// The column.
        i: i32,
        /// The row.
        j: i32,
    },
}

impl Probe {
    /// Samples timed by `clock`, sent to `sink`; a closed receiver drops
    /// them.
    pub fn new(clock: fn() -> Instant, sink: mpsc::Sender<Sample>) -> Self {
        Probe { clock, sink }
    }

    /// The probe's clock.
    pub(crate) fn clock(&self) -> fn() -> Instant {
        self.clock
    }

    /// Reads the probe's clock.
    pub(crate) fn now(&self) -> Instant {
        (self.clock)()
    }

    /// Sends `sample`, unless the receiver is gone.
    pub(crate) fn send(&self, sample: Sample) {
        let _ = self.sink.send(sample);
    }
}
