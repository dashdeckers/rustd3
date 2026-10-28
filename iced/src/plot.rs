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
//! a gesture changed a domain. It cuts the list into the cells of the
//! canvas's partition by every plot-area edge, renders only the cells
//! whose content hash changed (every cell when the partition's rects
//! change), and draws each as images placed on whole device pixels. A
//! cell whose RGBA would reach [`BAND_BYTES`] is rendered and drawn as
//! horizontal bands below that size, so the GPU renderer uploads every
//! image inside the frame that first draws it. Hovering renders no cell:
//! the crosshair and the highlight are quads in a layer clipped to the
//! hovered plot area, in the figure theme's ink, and [`Plot::on_hover`]
//! reports the hovered point ([`Hover`]) from an index of the drawn points
//! the widget keeps with each drawing list.
//!
//! [`Plot::revision`] re-renders every cell once for a change the hashes
//! cannot see (a paint image's pixels replaced in place, a font swapped),
//! and [`Plot::caching`] with [`Caching::Off`] re-renders every cell
//! whenever any cell changed, a diagnostic for the benchmark.
//!
//! Changed cells render inline when the [`cost`] model predicts they take
//! at most half the frame budget, and otherwise on one thread the widget
//! owns, with vello_cpu's own pool; [`Plot::render`] forces either path.
//! Until a cell's new image arrives it keeps its previous one, a preview
//! of outdated content, and a cell that never had one shows the theme's
//! background. Results wake the runtime through its waker and land on
//! `Event::Waken`, newest generation first: an older result never
//! replaces a newer image. Dropping the widget's state joins the thread.
//! [`Plot::probe`] reports what each frame cost and what it showed, timed
//! by a clock the app supplies: the library reads none.
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
//! after its last step. While a plot-area cell's sharp render is pending,
//! its previous image is drawn scaled by one affine map per axis, soft
//! until the new one lands; gutters keep their last image. A double click
//! returns the handles the panel zooms that a gesture touched to their
//! fitted domains (a published one receives it as a message); the cursor
//! shows a grab hand over a zoomable plot area, a grabbing hand during a
//! drag, a resize arrow over a zoomable axis and a crosshair over other
//! plot areas.

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
mod listener;
mod overlay;
mod preview;
pub mod probe;
mod region;
mod render;
mod scale_factor;
mod slot;
mod state;
mod worker;

use iced_core::image;
use iced_core::layout::{self, Layout};
use iced_core::mouse;
use iced_core::renderer;
use iced_core::widget::{Meta, Tree, tree};
use iced_core::{Element, Event, Length, Rectangle, Shell, Size, Widget, window};
use rustd3_figure::Figure;
use rustd3_figure::scale::{Kind, ScaleRef};

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
/// tiny-skia truncates an image's device position and wgpu adds 0.001
/// and rounds, and a quarter pixel lands on the intended pixel on both
/// where none truncates a hair early and a half rounds a pixel late.
pub const NUDGE: f32 = 0.25;

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
    probe: Option<Probe>,
    listeners: Vec<Listener<'a, Message>>,
    on_hover: Option<Report<'a, Message>>,
}

impl<'a, Message> Plot<'a, Message> {
    /// Shows `figure`; the widget keeps it for relayouts.
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
            probe: None,
            listeners: Vec::new(),
            on_hover: None,
        }
    }

    /// The widget's width; defaults to `Length::Fill`.
    pub fn width(self, width: impl Into<Length>) -> Self {
        Plot {
            width: width.into(),
            ..self
        }
    }

    /// The widget's height; defaults to `Length::Fill`.
    pub fn height(self, height: impl Into<Length>) -> Self {
        Plot {
            height: height.into(),
            ..self
        }
    }

    /// Device pixels per logical pixel; wins over every fallback. An app
    /// with its own scale factor on the software renderer, which reports
    /// none, passes it here.
    pub fn scale_factor(self, factor: f32) -> Self {
        Plot {
            scale_factor: Some(factor),
            ..self
        }
    }

    /// Crosshair in the hovered panel (default on): two lines one device
    /// pixel wide through the pointer, in the figure theme's ink at 40%.
    pub fn crosshair(self, on: bool) -> Self {
        Plot {
            crosshair: on,
            ..self
        }
    }

    /// Ring around the nearest data point (default on): the point of the
    /// hovered panel's marks nearest the pointer within 12 logical px,
    /// circled in the figure theme's ink.
    pub fn highlight(self, on: bool) -> Self {
        Plot {
            highlight: on,
            ..self
        }
    }

    /// Where changed cells rasterize; defaults to [`Render::Auto`].
    pub fn render(self, render: Render) -> Self {
        Plot { render, ..self }
    }

    /// How the widget reuses rendered images; defaults to
    /// [`Caching::On`]. [`Caching::Off`] re-renders every cell whenever
    /// any changed, a diagnostic for the benchmark.
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
    pub fn revision(self, revision: u64) -> Self {
        Plot { revision, ..self }
    }

    /// Reports per-stage durations, per-cell raster times and what each
    /// frame showed to `probe`, timed by its clock (see [`probe`]).
    /// Without a probe nothing is sent and no clock is read.
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
    /// for the same handle replaces this one.
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
    pub fn on_change<S>(
        mut self,
        handle: ScaleRef<S>,
        f: impl Fn([S::Domain; 2]) -> Message + 'a,
    ) -> Self
    where
        S: Kind + 'static,
    {
        let listener = Listener::new(handle, f);
        self.listeners.retain(|other| other.key() != listener.key());
        self.listeners.push(listener);
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
    /// messages.
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
        let factor =
            scale_factor::resolve(self.scale_factor, renderer.scale(), state.window_factor());
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
                };
                let (_, messages) =
                    state.redraw(&self.figure, factor, *now, &pipeline, &self.listeners);
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
