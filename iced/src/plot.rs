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
//! # mod rustd3 { pub use rustd3_core::scale; pub use rustd3_figure as figure; }
//! use iced::Element;
//! use rustd3::scale;
//! use rustd3::figure::mark::line;
//! use rustd3::figure::{Figure, axis};
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
//! A cell whose RGBA would reach 2 MiB is rendered and drawn as
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
//! # mod rustd3 { pub use rustd3_core::{scale, series}; pub use rustd3_figure as figure; }
//! use iced::Element;
//! use rustd3::scale;
//! use rustd3::series::Recording;
//! use rustd3::figure::mark::line;
//! use rustd3::figure::scale::Fit;
//! use rustd3::figure::{Figure, axis};
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
// The cost model's seed constants, which the benchmark records: public to
// the dev crates through the `__dev` feature only.
#[cfg(feature = "__dev")]
#[doc(hidden)]
pub mod cost;
#[cfg(not(feature = "__dev"))]
mod cost;
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
// The widget, re-exported at the crate root only.
pub(crate) mod widget;
mod worker;

// The module docs name the widget and its figure.
#[cfg(doc)]
use rustd3_figure::Figure;

#[cfg(doc)]
use crate::Plot;
pub use caching::Caching;
pub use hover::Hover;
pub use probe::channel::Probe;
pub use render::Render;

/// RGBA bytes at which a cell is split into bands: wgpu uploads images
/// under 2 MiB inside the frame (`wgpu/src/image/cache.rs:223-246`)
/// and larger ones through a worker drained only by `Renderer::tick`.
pub(crate) const BAND_BYTES: usize = 2 * 1024 * 1024;

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
pub(crate) const NUDGE: f32 = 0.0625;
