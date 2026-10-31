//! [`Plot`]: a [`Figure`] shown as images that vello_cpu renders from its
//! drawing list, the pixels the figure's PNG has, with a crosshair and a
//! nearest-point highlight drawn over them, zoomed and panned by the
//! wheel and the mouse.
//!
//! `view` builds the figure from the app's state and hands it to the
//! widget by value; the widget keeps its own state (the figure's cache,
//! the rendered cells, the domains gestures gave the handles nobody
//! listens to) in iced's widget tree between frames. Here the app owns
//! the x domain: a wheel or a drag over the plot area sends it the new
//! window, and its view shows that window, while y stays the widget's:
//!
//! ```no_run
//! use iced::Element;
//! use rustd3_core::scale;
//! use rustd3_figure::mark::line;
//! use rustd3_figure::{Figure, axis};
//! use rustd3_iced::Plot;
//!
//! struct App {
//!     data: Vec<(f64, f64)>,
//!     /// The x window the user chose, once there is one.
//!     window: Option<[f64; 2]>,
//! }
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     Window([f64; 2]),
//! }
//!
//! impl App {
//!     fn update(&mut self, message: Message) {
//!         match message {
//!             Message::Window(window) => self.window = Some(window),
//!         }
//!     }
//!
//!     fn view(&self) -> Element<'_, Message> {
//!         let mut fig = Figure::new();
//!         let x = match self.window {
//!             Some(window) => fig.scale(scale::linear().domain(window)),
//!             None => fig.fit(scale::linear()),
//!         };
//!         let y = fig.fit(scale::linear());
//!         fig.panel(0, 0)
//!             .mark(line(self.data.iter().copied(), x, y))
//!             .axis(axis::left(y))
//!             .axis(axis::bottom(x))
//!             .zoom_x(x)
//!             .zoom_y(y);
//!         Plot::new(fig).on_change(x, Message::Window).into()
//!     }
//! }
//! ```
//!
//! The widget resolves and draws its figure once per `RedrawRequested`,
//! for the screen at the effective scale factor (`Plot::scale_factor`,
//! else the renderer's hint, else the window's factor, else 1), and only
//! when the figure was rebuilt, the widget resized, the factor changed or
//! a gesture changed a domain. It splits the list into each plot area's
//! layer (the figure's background, the panel's background, grid lines
//! and marks, drawn with an overscan of two tiles beyond the area) and the
//! chrome, and cuts the chrome into the cells of the canvas's partition
//! by every plot-area edge; each plot area, however the partition cuts
//! it, is one grid of tiles of 256 device px anchored at its data anchor
//! (its corner until a drag moves it), with a ring of tiles around it,
//! each rendered over its whole square, and shows its part of the chrome
//! over them. It renders only the cells,
//! tiles and overlays whose content hash changed (every one when the
//! partition's rects change), the ring after the rest, and draws each as
//! images placed on whole device pixels. Adjacent gutter cells of a row
//! whose content depends on the same handles (an axis strip and the
//! corners its labels reach into) merge into one image.
//! A cell whose RGBA would reach [`BAND_BYTES`] is rendered and drawn as
//! horizontal bands below that size, so the GPU renderer uploads every
//! image inside the frame that first draws it; a tile always stays under
//! it. Hovering renders no cell:
//! the crosshair and the highlight are quads in a layer clipped to the
//! hovered plot area, in the figure theme's ink, and [`Plot::on_hover`]
//! reports the hovered point ([`Hover`]) from an index of the drawn points
//! the widget keeps with each drawing list.
//!
//! [`Plot::revision`] re-renders every cell once for a change the hashes
//! cannot see (a paint image's pixels replaced in place, a font swapped),
//! and [`Plot::caching`] with [`Caching::Off`] re-renders every cell
//! whenever any cell changed, a diagnostic for the benchmark; with
//! [`Caching::Verify`] every image kept or translated renders again after
//! the real renders, replaces what shows when they differ by more than
//! 1/255 and is reported (`probe::Sample::Mismatch`).
//!
//! Changed tiles, cells and overlays render in order (visible tiles
//! nearest the last gesture input first, then the gutters and overlays,
//! then the ring, which waits while a drag, a wheel burst or a brush
//! lasts, since each zoom step would render it all again, and renders
//! once it ends; while a gesture lasts the gutters and overlays go ahead
//! of the tiles, so the axes keep step with the data): inline each one
//! whose predicted time still fits half
//! the frame budget with those before it, by a cost model each widget fits
//! to its own renders, tiles apart from gutters and overlays (seeded by
//! [`cost`]), and the rest on a pool of
//! threads the widget owns ([`Plot::threads`]), newest wins per tile;
//! while a wheel burst or a brush lasts no tile renders inline, so the UI
//! thread only composes and renders the gutters and overlays the model
//! allows, while a drag's entering column renders inline when it fits.
//! [`Plot::render`] forces either path. Plot areas of at least 10,000
//! markers splat them on screen (`raster::Renderer::splat`). On iced's
//! software renderer (one that reports no scale), a plot area that shows
//! a preview layer is composed into one image on the CPU first and drawn
//! at once: its scaled, overlapping tiles would each cost tiny-skia a
//! frame-sized clip mask and a filtered pass, and every new tile a
//! conversion; the previews are then sampled nearest.
//! Until a cell's new image arrives it keeps its previous one, a preview
//! of outdated content, and a cell that never had one shows the theme's
//! background. When the partition's rects change (the layout settling
//! after a gesture, a resize), the old plot areas' tiles join the new
//! areas' preview layers, mapped onto them, and the old gutter images stay
//! drawn under the new runs until those render: no frame shows the
//! background where content was. Results wake the runtime through its waker and land on
//! `Event::Waken`, newest generation first: an older result never
//! replaces a newer image. Zoom steps coalesce: each step's renders
//! replace those still waiting on the pool, so at most one generation
//! waits, and a tile a worker was rendering for an older step lands in
//! the plot area's preview layer, mapped to the current domains, until
//! newer images cover it. Dropping the widget's state joins the threads.
//! [`Plot::probe`] reports what each frame cost and what it showed, timed
//! by a clock the app supplies. The only clock the library reads itself is
//! the monotonic one around each render, for the cost model: what
//! it renders never depends on it, only where later renders run.
//!
//! **Gestures.** A wheel over a plot area zooms every handle its panel
//! zooms (`Panel::zoom_x`, `Panel::zoom_y`) around the cursor, a wheel
//! over an axis zooms that axis's handle only, and a left-button drag
//! pans; horizontal wheel deltas pan too. Each handle has its own core
//! `Zoom`. Pointer input is queued and applied once per
//! `RedrawRequested`, and each gesture accumulates its transform from its
//! start, so updates that arrive late lose no distance; a step the scale
//! rejects (non-finite, reversed, or a span below 2⁻⁴⁰ of its centre)
//! leaves the gesture where it was. [`Plot::on_change`] turns a handle's
//! changes into one message per frame and the app owns that domain;
//! during a gesture the widget continues from what it last published, and
//! when it publishes it skips drawing in that pass, since the runtime runs
//! the app's `update` and redraws at once. Other handles' domains are the
//! widget's, kept by handle position and scale type, and dropped when the
//! rebuilt figure puts another scale type there. The layout holds still
//! during a gesture (labels grow outward) and settles once when it ends:
//! a drag at its release, a wheel gesture at the first redraw 300 ms
//! after its last step. A drag snaps each step to whole device pixels and
//! applies the exact offset once at its release: the grids of the plot
//! areas it pans move by as many pixels, lines thin on columns anchored
//! to the moved grid and dashes keep their phase, so a tile's image moves
//! with it unchanged and only the tiles entering the ring render, a column
//! per 256 px, while the plot area looks the same apart from where things
//! are (its paints, strokes, markers and glyphs: a figure rebuilt in
//! another color renders every tile) and the scales leading its axes moved
//! by the same whole pixels at both its edges (`Scale::remap`: a
//! `Fit::Visible` y that refits as x pans renders every tile); the gutter
//! cells showing the panned handle render whole. Any other change of a
//! plot area's view (a zoom) begins its grid
//! anew: while the new tiles' sharp renders are pending, the old tiles,
//! ring included, are drawn scaled by one affine map per axis, soft until
//! the new ones land; gutters keep their last image. Only the newest
//! step's renders wait on the pool, and a tile a thread was rendering
//! when the next step came still lands, scaled the same way over the
//! older ones, so a burst of wheel steps shows sharp but slightly stale
//! tiles as each finishes rather than the tiles from before the burst
//! until it ends. A double click
//! returns the handles the panel zooms that a gesture touched to their
//! fitted domains (a published one receives it as a message); the cursor
//! shows a grab hand over a zoomable plot area, a grabbing hand during a
//! drag, a resize arrow over a zoomable axis and a crosshair over other
//! plot areas.
//!
//! **Brushes.** A drag over the plot area of a panel with a brush
//! (`Panel::brush`) brushes instead of panning, through core's `BrushX`
//! or `BrushY` over that plot area; [`Plot::on_brush`] publishes the
//! selection in the handle's domain, and a brush that `.sets(target)`
//! sets the target's domain as a zoom does. Between gestures each brush
//! takes its selection from the domain it selects, so a zoom of the target
//! moves the band, and during one it never does, so updates the app
//! applies late lose nothing. Brushing changes no cell: only the overlay
//! redraws, unless a target's domain changed. A double click clears the
//! clicked panel's brush and resets its target with the handles it zooms.

//! **Live data.** The widget keeps no recording state: the hashes are the
//! dependency tracker. An append bumps the recording's revision, which
//! the thinning memo keys on, so the next frame thins the view from the
//! index again; the line's part hashes anew and its plot-area cell
//! renders, and nothing else does while the scales stay. With an x handle
//! fitted to `Fit::Window(span)` the domain follows the newest data in
//! whole device columns, so the x axes' ticks and labels re-render too;
//! a gesture holds the window until a double click lets it follow again.
//! `drop_front` renders the same way:
//!
//! ```no_run
//! use iced::Element;
//! use rustd3_core::scale;
//! use rustd3_core::series::Recording;
//! use rustd3_figure::mark::line;
//! use rustd3_figure::scale::Fit;
//! use rustd3_figure::{Figure, axis};
//! use rustd3_iced::Plot;
//!
//! struct Live {
//!     /// A regular clock: appends take no x.
//!     rec: Recording<f64>,
//! }
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     Samples(Vec<f64>),
//! }
//!
//! impl Live {
//!     fn update(&mut self, message: Message) {
//!         match message {
//!             Message::Samples(y) => {
//!                 self.rec.extend(&[], [y]).expect("one channel, finite samples");
//!             }
//!         }
//!     }
//!
//!     fn view(&self) -> Element<'_, Message> {
//!         let mut fig = Figure::new();
//!         // The last ten seconds, moving in whole device columns.
//!         let t = fig.fit_with(scale::linear(), Fit::Window(10.0));
//!         let v = fig.fit(scale::linear());
//!         fig.panel(0, 0)
//!             .mark(line(self.rec.series(0).expect("channel 0"), t, v))
//!             .axis(axis::left(v))
//!             .axis(axis::bottom(t))
//!             .zoom_x(t);
//!         Plot::new(fig).into()
//!     }
//! }
//! ```

mod area;
mod brush;
mod budget;
mod caching;
pub mod cost;
mod domains;
#[cfg(test)]
mod fixture;
mod gesture;
mod handle;
mod hover;
mod input;
mod job;
mod layer;
mod listener;
mod merge;
mod mosaic;
mod overlay;
mod preview;
pub mod probe;
mod queue;
mod region;
mod render;
mod scale_factor;
mod slot;
mod state;
mod style;
mod tile;
mod worker;

use iced_core::image;
use iced_core::layout::{self, Layout};
use iced_core::mouse;
use iced_core::renderer;
use iced_core::widget::{Meta, Tree, tree};
use iced_core::{Element, Event, Length, Rectangle, Shell, Size, Widget, window};
use rustd3_figure::Figure;
use rustd3_figure::scale::{Invert, Kind, ScaleRef};

pub use caching::Caching;
pub use hover::Hover;
use listener::Listener;
use overlay::Shown;
pub use probe::Probe;
pub use render::Render;
use state::{Pipeline, State};

/// RGBA bytes at which a cell is split into bands: wgpu uploads images
/// under 2 MiB inside the frame (`wgpu/src/image/cache.rs:223-246`)
/// and larger ones through a worker drained only by `Renderer::tick`.
pub const BAND_BYTES: usize = 2 * 1024 * 1024;

/// The app's message for what the pointer hovers.
type Report<'a, Message> = Box<dyn Fn(Option<Hover>) -> Message + 'a>;

/// Device px added to an image's origin before dividing by the factor:
/// tiny-skia truncates an image's position in image pixels and wgpu adds
/// 0.001 and rounds its device position, and a sixteenth of a pixel lands
/// on the intended pixel on both where none truncates a hair early (the
/// f32 round-off is under a hundredth of a pixel up to 40,000 px) and a
/// half rounds a pixel late. It stays under half a device pixel when the
/// renderer draws at up to seven times the factor the widget assumes (a
/// capture at 2 of a widget told 1), so both renderers then place each
/// image and its clip on the same device pixels: a quarter, doubled to
/// half a pixel, moved wgpu's images and tiny-skia's clips a pixel later
/// and left a one-pixel seam where cells meet.
pub const NUDGE: f32 = 0.0625;

/// A figure shown as images rendered from its drawing list.
///
/// The widget takes the figure by value, as `view` builds it, and keeps
/// it for relayouts; between frames it keeps the figure's cache (laid-out
/// labels, thinned lines) and the rendered cells in iced's widget tree.
pub struct Plot<'a, Message> {
    figure: Figure<'a>,
    width: Length,
    height: Length,
    scale_factor: Option<f32>,
    crosshair: bool,
    highlight: bool,
    render: Render,
    caching: Caching,
    revision: u64,
    threads: Option<usize>,
    probe: Option<Probe>,
    listeners: Vec<Listener<'a, Message>>,
    brushes: Vec<brush::Listener<'a, Message>>,
    on_hover: Option<Report<'a, Message>>,
}

impl<'a, Message> std::fmt::Debug for Plot<'a, Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plot").finish_non_exhaustive()
    }
}

impl<'a, Message> Plot<'a, Message> {
    /// Shows `figure`; the widget keeps it for relayouts.
    #[must_use]
    pub fn new(figure: Figure<'a>) -> Self {
        Plot {
            figure,
            width: Length::Fill,
            height: Length::Fill,
            scale_factor: None,
            crosshair: true,
            highlight: true,
            render: Render::default(),
            caching: Caching::default(),
            revision: 0,
            threads: None,
            probe: None,
            listeners: Vec::new(),
            brushes: Vec::new(),
            on_hover: None,
        }
    }

    /// The widget's width; defaults to `Length::Fill`.
    #[must_use]
    pub fn width(self, width: impl Into<Length>) -> Self {
        Plot {
            width: width.into(),
            ..self
        }
    }

    /// The widget's height; defaults to `Length::Fill`.
    #[must_use]
    pub fn height(self, height: impl Into<Length>) -> Self {
        Plot {
            height: height.into(),
            ..self
        }
    }

    /// Device pixels per logical pixel; wins over every fallback. In a
    /// window the widget learns the factor from the window's events (and
    /// wgpu's renderer), so an app passes it only when it scales itself on
    /// the software renderer, which reports none. `iced_test`'s
    /// `Simulator` has no window: it snapshots at 2 and tells widgets no
    /// factor, so a test that snapshots a `Plot` passes the snapshot's
    /// factor here; without it the widget draws for 1 and the snapshot
    /// shows each of its pixels doubled. A factor that is not finite and
    /// positive is skipped for the next in the chain; one above 8, from
    /// here or from the window, is taken as 8; and a widget wider or
    /// taller than 65,408 device px, the most the renderer draws, renders
    /// at the factor that fits it.
    ///
    /// ```text
    /// let plot = Plot::new(fig).scale_factor(2.0); // as iced_test snapshots
    /// let mut ui = Simulator::with_size(Settings::default(), (800.0, 500.0), plot);
    /// ui.snapshot(&Theme::Light)?.matches_image("plot.png")?;
    /// ```
    #[must_use]
    pub fn scale_factor(self, factor: f32) -> Self {
        Plot {
            scale_factor: Some(factor),
            ..self
        }
    }

    /// Crosshair in the hovered panel (default on): two lines one device
    /// pixel wide through the pointer, in the figure theme's ink at 40%.
    #[must_use]
    pub fn crosshair(self, on: bool) -> Self {
        Plot {
            crosshair: on,
            ..self
        }
    }

    /// Ring around the nearest data point (default on): the point of the
    /// hovered panel's marks nearest the pointer within 12 logical px,
    /// circled in the figure theme's ink.
    #[must_use]
    pub fn highlight(self, on: bool) -> Self {
        Plot {
            highlight: on,
            ..self
        }
    }

    /// Where changed cells rasterize; defaults to [`Render::Auto`].
    #[must_use]
    pub fn render(self, render: Render) -> Self {
        Plot { render, ..self }
    }

    /// How the widget reuses rendered images; defaults to
    /// [`Caching::On`]. [`Caching::Off`] re-renders every cell whenever
    /// any changed, a diagnostic for the benchmark; [`Caching::Verify`]
    /// renders what it keeps again and reports what differs.
    #[must_use]
    pub fn caching(self, caching: Caching) -> Self {
        Plot { caching, ..self }
    }

    /// Bump when something the hashes cannot see changed (a paint
    /// image's pixels replaced in place, a font swapped); every cell
    /// re-renders at the next refresh. 0 by default; the widget compares
    /// each value with the last one it rendered.
    ///
    /// ```no_run
    /// use rustd3_figure::Figure;
    /// use rustd3_iced::Plot;
    ///
    /// /// The app's state: a counter it bumps when it replaces an image's
    /// /// pixels in place.
    /// struct App {
    ///     replaced: u64,
    /// }
    ///
    /// impl App {
    ///     fn plot(&self) -> Plot<'_, ()> {
    ///         Plot::new(Figure::new()).revision(self.replaced)
    ///     }
    /// }
    /// # let _ = App { replaced: 0 }.plot();
    /// ```
    #[must_use]
    pub fn revision(self, revision: u64) -> Self {
        Plot { revision, ..self }
    }

    /// Render pool size, clamped to 1..=64; changing it restarts the pool
    /// (at its next jobs). By default the available parallelism less the
    /// UI thread, 1 to 8, which respects the process's CPU affinity: one
    /// thread on a 2-core machine, 8 on 16 cores. Every render the widget
    /// makes, on the pool or inline on the UI thread, is single-threaded,
    /// so the pool is the widget's only parallelism whatever its size.
    ///
    /// ```no_run
    /// use rustd3_figure::Figure;
    /// use rustd3_iced::Plot;
    ///
    /// // Leave the other cores to the app's own work.
    /// let plot = Plot::<()>::new(Figure::new()).threads(2);
    /// # let _ = plot;
    /// ```
    #[must_use]
    pub fn threads(self, threads: usize) -> Self {
        Plot {
            threads: Some(threads.clamp(1, worker::MAX)),
            ..self
        }
    }

    /// Reports per-stage durations, per-cell raster times and what each
    /// frame showed to `probe`, timed by its clock (see [`probe`]).
    /// Without a probe nothing is sent; the widget times its renders for
    /// its cost model either way.
    ///
    /// ```no_run
    /// use std::sync::mpsc;
    ///
    /// use iced_core::time::Instant;
    /// use rustd3_figure::Figure;
    /// use rustd3_iced::Plot;
    /// use rustd3_iced::plot::Probe;
    /// use rustd3_iced::plot::probe::Sample;
    ///
    /// /// The app's clock.
    /// fn clock() -> Instant {
    ///     Instant::now()
    /// }
    ///
    /// let (tx, rx) = mpsc::channel();
    /// let plot = Plot::<()>::new(Figure::new()).probe(Probe::new(clock, tx));
    /// # let _ = plot;
    /// // After some frames: how many showed an outdated cell.
    /// let previews = rx
    ///     .try_iter()
    ///     .filter(|sample| matches!(sample, Sample::Frame { previews, .. } if *previews > 0))
    ///     .count();
    /// # let _ = previews;
    /// ```
    #[must_use]
    pub fn probe(self, probe: Probe) -> Self {
        Plot {
            probe: Some(probe),
            ..self
        }
    }

    /// Changes to `handle`'s domain become messages; the app owns the
    /// domain from then on. Without this, the widget keeps it.
    ///
    /// A gesture publishes one message per handle per frame, the domain's
    /// end stops in the scale's own type, and continues from the domain it
    /// last published until it ends, so updates the app applies late lose
    /// no distance. Only continuous handles change. A later `on_change`
    /// for the same handle replaces this one. A handle of another figure
    /// has no domain here: the listener is not added.
    ///
    /// The message carries the domain in the scale's own type: a time
    /// handle publishes instants, and a message of numbers for it does not
    /// compile (E0631, expected `fn([Timestamp; 2]) -> _`).
    ///
    /// ```compile_fail,E0631
    /// use rustd3_core::time;
    /// use rustd3_figure::Figure;
    /// use rustd3_iced::Plot;
    ///
    /// enum Msg {
    ///     Range([f64; 2]),
    /// }
    ///
    /// let mut fig = Figure::new();
    /// let x_time = fig.scale(time::utc());
    /// let plot: Plot<'_, Msg> = Plot::new(fig).on_change(x_time, Msg::Range); // E0631
    /// ```
    ///
    /// ```
    /// use rustd3_core::time;
    /// use rustd3_figure::Figure;
    /// use rustd3_iced::Plot;
    ///
    /// enum Msg {
    ///     Range([rustd3_core::jiff::Timestamp; 2]),
    /// }
    ///
    /// let mut fig = Figure::new();
    /// let x_time = fig.scale(time::utc());
    /// let plot: Plot<'_, Msg> = Plot::new(fig).on_change(x_time, Msg::Range);
    /// ```
    #[must_use]
    pub fn on_change<S>(
        mut self,
        handle: ScaleRef<S>,
        f: impl Fn([S::Domain; 2]) -> Message + 'a,
    ) -> Self
    where
        S: Kind + 'static,
    {
        if !self.figure.owns(handle) {
            return self;
        }
        let listener = Listener::new(handle, f);
        self.listeners.retain(|other| other.key() != listener.key());
        self.listeners.push(listener);
        self
    }

    /// The selections of brushes over `over` (`Panel::brush`) become
    /// messages: `f(Some([a, b]))` with the values `over`'s scale inverts
    /// the selection's ends to, in their order along the axis (left to
    /// right, top to bottom), while a gesture changes it and when it ends;
    /// `f(None)` when it ends empty (a click without a drag) or a double
    /// click clears it. One message per brush per frame, the latest. A
    /// later `on_brush` for the same handle replaces this one. A handle of
    /// another figure has no brush here: the listener is not added.
    ///
    /// A drag over a brushed plot area brushes (the wheel still zooms):
    /// a press outside the selection starts a new one, on it moves it, on
    /// its edges resizes it, as d3-brush does; the live rectangle and the
    /// selection band are drawn over the panel's plot area, and only the
    /// overlay redraws. With `.sets(target)` the selection sets the
    /// target's domain as a zoom does (published by `on_change`, else the
    /// widget's own, which fitting then skips until a double click), and
    /// between gestures the band follows the target's domain: a zoom of the
    /// target moves it, and a target showing all of `over`'s domain shows
    /// none.
    ///
    /// Only handles with a typed inverse have brushes: a selection over a
    /// band handle has no domain values, and `brush::x` does not take one.
    ///
    /// ```no_run
    /// use rustd3_core::jiff::Timestamp;
    /// use rustd3_core::time;
    /// use rustd3_figure::{Figure, brush};
    /// use rustd3_iced::Plot;
    ///
    /// #[derive(Debug, Clone)]
    /// enum Message {
    ///     /// The window the focus panel shows.
    ///     Window([Timestamp; 2]),
    ///     /// What the overview selects, if anything.
    ///     Brushed(Option<[Timestamp; 2]>),
    /// }
    ///
    /// fn view(full: [Timestamp; 2], window: [Timestamp; 2]) -> Plot<'static, Message> {
    ///     let mut fig = Figure::new();
    ///     let focus = fig.scale(time::utc().domain(window));
    ///     let context = fig.scale(time::utc().domain(full));
    ///     fig.panel(0, 0).zoom_x(focus);
    ///     fig.panel(1, 0).brush(brush::x(context).sets(focus));
    ///     Plot::new(fig)
    ///         .on_change(focus, Message::Window)
    ///         .on_brush(context, Message::Brushed)
    /// }
    /// # let _ = view;
    /// ```
    #[must_use]
    pub fn on_brush<A: Invert>(
        mut self,
        over: ScaleRef<A>,
        f: impl Fn(Option<[<A as Kind>::Domain; 2]>) -> Message + 'a,
    ) -> Self {
        if !self.figure.owns(over) {
            return self;
        }
        let listener = brush::Listener::new(over, f);
        self.brushes.retain(|other| other.key() != listener.key());
        self.brushes.push(listener);
        self
    }

    /// What the pointer is over becomes messages: `f(Some(hover))` when
    /// the drawn point nearest the pointer (within 12 logical px, in the
    /// panel whose plot area holds it) changes, `f(None)` once when there
    /// is none any more or the pointer left the widget. Moving within one
    /// point sends nothing. Moves between frames are hit-tested against an
    /// index of the drawn points built with the drawing list, so hovering
    /// resolves and renders nothing; at most one message goes out per
    /// frame, and none during a gesture. The highlight ring shows the same
    /// point; tooltips are the app's own widgets, built from these
    /// messages. The index is built only for a widget with the highlight
    /// or `on_hover`, and not while a gesture lasts: the refresh after it
    /// builds it, and nothing is hit-tested meanwhile.
    ///
    /// ```no_run
    /// use rustd3_core::scale;
    /// use rustd3_figure::mark::dots;
    /// use rustd3_figure::Figure;
    /// use rustd3_iced::Plot;
    /// use rustd3_iced::plot::Hover;
    ///
    /// #[derive(Debug, Clone)]
    /// enum Message {
    ///     Hovered(Option<Hover>),
    /// }
    ///
    /// let mut fig = Figure::new();
    /// let x = fig.fit(scale::linear());
    /// let y = fig.fit(scale::linear());
    /// fig.panel(0, 0).mark(dots([(1.0, 2.0), (3.0, 4.0)], x, y));
    /// let plot: Plot<'_, Message> = Plot::new(fig).on_hover(Message::Hovered);
    /// # let _ = plot;
    /// ```
    #[must_use]
    pub fn on_hover(self, f: impl Fn(Option<Hover>) -> Message + 'a) -> Self {
        Plot {
            on_hover: Some(Box::new(f)),
            ..self
        }
    }

    /// The overlay parts this widget shows.
    fn shown(&self) -> Shown {
        Shown {
            crosshair: self.crosshair,
            highlight: self.highlight,
        }
    }
}

impl<Message> Meta for Plot<'_, Message> {}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Plot<'_, Message>
where
    Renderer: image::Renderer<Handle = image::Handle>,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn diff(&mut self, tree: &mut Tree) {
        // iced rebuilds the view, and so the figure, on every message.
        tree.state.downcast_mut::<State>().rebuilt();
    }

    fn layout(&mut self, tree: &mut Tree, _renderer: &Renderer, limits: &layout::Limits) {
        let size = layout::atomic(limits, self.width, self.height);
        tree.size = size;
        tree.state.downcast_mut::<State>().resize(size);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        if let Event::Window(
            window::Event::Opened { scale_factor, .. } | window::Event::Rescaled(scale_factor),
        ) = event
        {
            state.rescale(*scale_factor);
        }
        let factor = scale_factor::fit(
            scale_factor::resolve(self.scale_factor, renderer.scale(), state.window_factor()),
            layout.bounds().size(),
        );
        // Gesture input is queued for the next `RedrawRequested`, which the
        // tracker translates first (a double click comes from it).
        if let Some(pointer) = state.translate(event, layout.bounds(), cursor, factor) {
            let queued = state.queue(pointer);
            if queued.capture {
                shell.capture_event();
            }
            if queued.redraw {
                shell.request_redraw();
            }
        }
        if state.wants_redraw() {
            shell.request_redraw();
        }
        match event {
            Event::Window(window::Event::Opened { .. } | window::Event::Rescaled(_)) => {
                shell.request_redraw();
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                let pipeline = Pipeline {
                    render: self.render,
                    caching: self.caching,
                    revision: self.revision,
                    probe: self.probe.as_ref(),
                    waker: shell.waker(),
                    threads: self.threads.unwrap_or_else(worker::default_size),
                    hover: self.highlight || self.on_hover.is_some(),
                };
                let listeners = (self.listeners.as_slice(), self.brushes.as_slice());
                let (_, messages) = state.redraw(&self.figure, factor, *now, &pipeline, listeners);
                // New cells may move the plot areas or the points under
                // the pointer, and a gesture may have ended.
                state.hover(layout.bounds(), self.shown());
                for message in messages {
                    shell.publish(message);
                }
                if let Some(hover) = state.reported()
                    && let Some(report) = &self.on_hover
                {
                    shell.publish(report(hover));
                }
                if let Some(end) = state.wheel_end() {
                    shell.request_redraw_at(window::RedrawRequest::At(end));
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.moved(cursor.position(), layout.bounds(), self.shown()) {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::CursorLeft) => {
                if state.moved(None, layout.bounds(), self.shown()) {
                    shell.request_redraw();
                }
            }
            Event::Waken => {
                if state.drain(self.probe.as_ref()) {
                    shell.request_redraw();
                }
            }
            Event::Window(_)
            | Event::Mouse(_)
            | Event::Keyboard(_)
            | Event::Touch(_)
            | Event::InputMethod(_)
            | Event::Clipboard(_) => {}
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        tree.state
            .downcast_ref::<State>()
            .draw(renderer, layout.bounds(), self.figure.get_theme());
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        tree.state
            .downcast_ref::<State>()
            .interaction(cursor.position(), layout.bounds())
    }
}

impl<'a, Message, Theme, Renderer> From<Plot<'a, Message>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: image::Renderer<Handle = image::Handle> + 'a,
{
    fn from(plot: Plot<'a, Message>) -> Self {
        // `Element::new` is private at the pinned revision
        // (`core/src/widget/element.rs:22`); widgets box themselves.
        plot._boxed()
    }
}
