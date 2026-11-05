//! [`Plot`]: a widget that shows a [`Figure`], zoomed and panned by the
//! wheel and the mouse.
//!
//! The widget shows the figure as images that vello_cpu renders from its
//! drawing list, with the same pixels as the figure's PNG. A crosshair and
//! a nearest-point highlight are drawn over them.
//!
//! The app's `view` builds the figure from the app's state and hands it to
//! the widget by value. The widget keeps its own state in iced's widget
//! tree between frames: the figure's cache, the rendered cells, and the
//! domains that gestures gave to handles no listener follows. In this
//! example the app owns the x domain. A wheel or a drag over the plot area
//! sends it the new window, and its view shows that window. The widget
//! keeps the y domain:
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
//! **Rendering.** The widget resolves and draws its figure at most once per
//! `RedrawRequested`, and only when the figure was rebuilt, the widget
//! resized, the scale factor changed or a gesture changed a domain. It
//! draws for the screen at the effective scale factor: `Plot::scale_factor`,
//! else the renderer's hint, else the window's factor, else 1.
//!
//! The drawing list splits into one layer per plot area and the chrome. A
//! plot area's layer holds the figure's background, the panel's background,
//! the grid lines and the marks, drawn with an overscan of two tiles beyond
//! the area. The chrome is everything else, such as the axes. Every
//! plot-area edge cuts the canvas into a partition, and the chrome is cut
//! into the partition's cells. The cells outside the plot areas are the
//! gutters.
//!
//! Each plot area, however the partition cuts it, is one grid of tiles of
//! 256 device px, with a ring of tiles around it. The grid is anchored at
//! the area's data anchor, which is its corner until a drag moves it. Each
//! tile renders over its whole square, and the area shows its part of the
//! chrome over the tiles. The widget renders only the cells, tiles and
//! overlays whose content hash changed, and every one when the partition's
//! rects change. The ring renders after the rest. Each is drawn as images
//! placed on whole device pixels. Adjacent gutter cells of a row whose
//! content depends on the same handles merge into one image, such as an
//! axis strip and the corners its labels reach into.
//!
//! A cell whose RGBA would reach 2 MiB is rendered and drawn as horizontal
//! bands below that size, so the GPU renderer uploads every image inside
//! the frame that first draws it. A tile always stays under that size.
//!
//! Hovering renders no cell. The crosshair and the highlight are quads in
//! a layer clipped to the hovered plot area, in the figure theme's ink.
//! [`Plot::on_hover`] reports the hovered point ([`Hover`]) from an index of
//! the drawn points that the widget keeps with each drawing list.
//!
//! [`Plot::revision`] re-renders every cell once, for a change the hashes
//! cannot see, such as a paint image's pixels replaced in place or a font
//! swapped. [`Plot::caching`] with [`Caching::Off`] re-renders every cell
//! whenever any cell changed, as a diagnostic for the benchmark. With
//! [`Caching::Verify`], every image kept or translated renders again after
//! the real renders. When the two differ by more than 1/255, the new image
//! replaces what shows and the difference is reported
//! (`probe::Sample::Mismatch`).
//!
//! **Scheduling.** Changed tiles, cells and overlays render in order:
//! visible tiles nearest the last gesture input first, then the gutters and
//! overlays, then the ring. While a gesture lasts, the gutters and overlays
//! go ahead of the tiles, so the axes keep step with the data. The ring
//! waits while a drag, a wheel burst or a brush lasts, since each zoom step
//! would render it all again, and renders once the gesture ends.
//!
//! A render runs inline when its predicted time, added to those before it,
//! still fits half the frame budget. The prediction comes from a cost model
//! that each widget fits to its own renders, with tiles fitted apart from
//! gutters and overlays, seeded by [`cost`]. The rest run on a pool of
//! threads the widget owns ([`Plot::threads`]), and the newest render wins
//! per tile. While a wheel burst or a brush lasts no tile renders inline,
//! so the UI thread only composes and renders the gutters and overlays the
//! model allows. During a drag, the entering column renders inline when it
//! fits. [`Plot::render`] forces either path. Plot areas of at least 10,000
//! markers splat them on screen (`raster::Renderer::splat`).
//!
//! Until a cell's new image arrives, the cell keeps its previous one as a
//! preview of outdated content. A cell that never had an image shows the
//! theme's background. When the partition's rects change (the layout
//! settling after a gesture, or a resize), the old plot areas' tiles join
//! the new areas' preview layers, mapped onto them. The old gutter images
//! stay drawn under the new merged gutter cells until those render, so no
//! frame shows the background where content was.
//!
//! On iced's software renderer (one that reports no scale), a plot area
//! that shows a preview layer is composed into one image on the CPU first
//! and drawn at once. Its scaled, overlapping tiles would each cost
//! tiny-skia a frame-sized clip mask and a filtered pass, and every new
//! tile a conversion. The previews are then sampled nearest.
//!
//! Results wake the runtime through its waker and land on `Event::Waken`,
//! newest generation first, so an older result never replaces a newer
//! image. Zoom steps coalesce. Each step's renders replace those still
//! waiting on the pool, so at most one generation waits. A tile that a
//! worker was rendering for an older step lands in the plot area's preview
//! layer, mapped to the current domains, until newer images cover it.
//! Dropping the widget's state joins the threads.
//!
//! [`Plot::probe`] reports what each frame cost and what it showed, timed
//! by a clock the app supplies. The only clock the library reads itself is
//! the monotonic one around each render, for the cost model. That clock
//! decides only where later renders run, never what they draw.
//!
//! **Gestures.** A wheel over a plot area zooms every handle its panel
//! zooms (`Panel::zoom_x`, `Panel::zoom_y`) around the cursor. A wheel over
//! an axis zooms only that axis's handle. A left-button drag pans, and so
//! do horizontal wheel deltas. Each handle has its own core `Zoom`. Pointer
//! input is queued and applied once per `RedrawRequested`. Each gesture
//! accumulates its transform from its start, so updates that arrive late
//! lose no distance. A step the scale rejects (non-finite, reversed, or a
//! span below 2⁻⁴⁰ of its center) leaves the gesture where it was.
//!
//! [`Plot::on_change`] turns a handle's changes into one message per frame,
//! and the app then owns that domain. During a gesture the widget continues
//! from what it last published. In a pass where it publishes, it skips
//! drawing, since the runtime runs the app's `update` and redraws at once.
//! Other handles' domains belong to the widget. It keeps them by handle
//! position and scale type, and drops one when the rebuilt figure puts
//! another scale type there.
//!
//! The layout holds still during a gesture (labels grow outward) and
//! settles once when it ends: for a drag at its release, and for a wheel
//! gesture at the first redraw 300 ms after its last step.
//!
//! A drag snaps each step to whole device pixels and applies the exact
//! offset once at its release. The grids of the plot areas it pans move by
//! as many pixels. Lines thin on columns anchored to the moved grid, and
//! dashes keep their phase, so a tile's image moves with the grid
//! unchanged. Only the tiles entering the ring render, a column per 256 px.
//! This holds while the area's paints, strokes, markers and glyphs look
//! the same apart from their positions, and while the scales leading its
//! axes moved by the same whole pixels at both its edges (`Scale::remap`).
//! A figure rebuilt in another color renders every tile, and so does a
//! `Fit::Visible` y that refits as x pans. The gutter cells showing the
//! panned handle render whole.
//!
//! Any other change of a plot area's view, such as a zoom, begins its grid
//! anew. While the new tiles' sharp renders are pending, the old tiles,
//! ring included, are drawn scaled by one affine map per axis, soft until
//! the new ones land. Gutters keep their last image. Only the newest step's
//! renders wait on the pool. A tile a thread was rendering when the next
//! step came still lands, scaled the same way over the older ones. A burst
//! of wheel steps therefore shows sharp but slightly stale tiles as each
//! finishes, rather than the tiles from before the burst until it ends.
//!
//! A double click returns the handles the panel zooms that a gesture
//! touched to their fitted domains. A published handle receives its fitted
//! domain as a message. The cursor shows a grab hand over a zoomable plot
//! area, a grabbing hand during a drag, a resize arrow over a zoomable axis
//! and a crosshair over other plot areas.
//!
//! **Brushes.** A drag over the plot area of a panel with a brush
//! (`Panel::brush`) brushes instead of panning, through core's `BrushX` or
//! `BrushY` over that plot area. [`Plot::on_brush`] publishes the selection
//! in the handle's domain, and a brush that `.sets(target)` sets the
//! target's domain as a zoom does. Between gestures each brush takes its
//! selection from the domain it selects, so a zoom of the target moves the
//! band. During a gesture it never does, so updates the app applies late
//! lose nothing. Brushing changes no cell: only the overlay redraws, unless
//! a target's domain changed. A double click clears the clicked panel's
//! brush and resets its target along with the handles it zooms.
//!
//! **Live data.** The widget keeps no recording state, because the content
//! hashes already track what changed. An append bumps the recording's
//! revision, which the thinning memo keys on, so the next frame thins the
//! view from the index again. The line's part hashes anew and its plot-area
//! cell renders. Nothing else renders while the scales stay. With an x
//! handle fitted to `Fit::Window(span)`, the domain follows the newest data
//! in whole device columns, so the x axes' ticks and labels re-render too.
//! A gesture holds the window until a double click lets it follow again.
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
// The widget, re-exported at the crate root only.
pub(crate) mod widget;
mod worker;

pub use caching::Caching;
pub use hover::Hover;
pub use probe::channel::Probe;
pub use render::Render;
// The module docs name the widget and its figure.
#[cfg(doc)]
use rustd3_figure::Figure;

#[cfg(doc)]
use crate::Plot;

/// RGBA bytes at which a cell is split into bands: wgpu uploads images
/// under 2 MiB inside the frame (`wgpu/src/image/cache.rs:223-246`)
/// and larger ones through a worker drained only by `Renderer::tick`.
pub(crate) const BAND_BYTES: usize = 2 * 1024 * 1024;

/// Device px added to an image's origin before dividing by the factor.
///
/// tiny-skia truncates an image's position in image pixels, and wgpu adds
/// 0.001 and rounds its device position. With no nudge, tiny-skia
/// truncates a hair early from f32 round-off, which is under a hundredth of
/// a pixel up to 40,000 px. With half a pixel, wgpu rounds a pixel late. A
/// sixteenth lands on the intended pixel on both. It also stays under half
/// a device pixel when the renderer draws at up to seven times the factor
/// the widget assumes (a capture at 2 of a widget told 1), so both
/// renderers then place each image and its clip on the same device pixels.
/// A quarter, doubled to half a pixel, moved wgpu's images and tiny-skia's
/// clips a pixel later and left a one-pixel seam where cells meet.
pub(crate) const NUDGE: f32 = 0.0625;
