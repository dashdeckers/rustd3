//! [`Plot`], the widget: its builders and its `Widget` implementation.

use iced_core::layout::{self, Layout};
use iced_core::widget::{Meta, Tree, tree};
use iced_core::{
    Element,
    Event,
    Length,
    Rectangle,
    Shell,
    Size,
    Widget,
    image,
    mouse,
    renderer,
    window,
};
use rustd3_figure::scale::{Invert, Kind, ScaleRef};
use rustd3_figure::{Figure, hit};

use crate::plot::listener::Listener;
use crate::plot::overlay::Shown;
use crate::plot::state::{Pipeline, State};
use crate::plot::{Caching, Hover, Probe, Render, brush, scale_factor, worker};

/// The app's message for what the pointer hovers.
type Report<'a, Message> = Box<dyn Fn(Option<Hover>) -> Message + 'a>;

/// What turns a readout into a message.
type Read<'a, Message> = Box<dyn Fn(Option<hit::Readout>) -> Message + 'a>;

/// A figure shown as images rendered from its drawing list.
///
/// The widget takes the figure by value, as `view` builds it, and keeps
/// it for relayouts. Between frames it keeps the figure's cache (laid-out
/// labels, thinned lines) and the rendered cells in iced's widget tree.
pub struct Plot<'a, Message> {
    figure: Figure<'a>,
    width: Length,
    height: Length,
    scale_factor: Option<f32>,
    magnification: f32,
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
    on_readout: Option<Read<'a, Message>>,
}

impl<'a, Message> std::fmt::Debug for Plot<'a, Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plot").finish_non_exhaustive()
    }
}

impl<'a, Message> Plot<'a, Message> {
    /// A widget that shows `figure`, which it keeps for relayouts.
    #[must_use]
    pub fn new(figure: Figure<'a>) -> Self {
        Plot {
            figure,
            width: Length::Fill,
            height: Length::Fill,
            scale_factor: None,
            magnification: 1.0,
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
            on_readout: None,
        }
    }

    /// The widget's width, `Length::Fill` by default.
    #[must_use]
    pub fn width(self, width: impl Into<Length>) -> Self {
        Plot {
            width: width.into(),
            ..self
        }
    }

    /// The widget's height, `Length::Fill` by default.
    #[must_use]
    pub fn height(self, height: impl Into<Length>) -> Self {
        Plot {
            height: height.into(),
            ..self
        }
    }

    /// Device pixels per logical pixel, which wins over every fallback.
    ///
    /// In a window the widget learns the factor from the renderer and the
    /// window's events, so an app has no need to pass it. `iced_test`'s
    /// `Simulator` has no window. It snapshots at 2 and tells widgets no
    /// factor, so a test that snapshots a `Plot` passes the snapshot's
    /// factor here. Without it the widget draws for 1, and the snapshot
    /// shows its pixels scaled to twice their size: doubled on wgpu, and
    /// resampled with soft seams between cells on the software renderer.
    ///
    /// A factor that is not finite and positive is skipped for the next in
    /// the chain. One above 8, from here or from the window, is taken as 8.
    /// [`Plot::magnification`] multiplies the factor the chain gives, and
    /// the product above 8 is taken as 8. A widget wider or taller than
    /// 65,408 device px, the most the renderer draws, renders at the
    /// factor that fits it.
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

    /// The scale of a transformation the host draws the widget under, 1 by
    /// default. The widget renders at the scale factor times this, so its
    /// images cover the device pixels they are shown on.
    ///
    /// A widget cannot see the transformation it is drawn under: it
    /// renders before the draw, and only wgpu reports one. A host that
    /// draws its content inside `Renderer::with_transformation`, such as a
    /// pan-and-zoom surface, passes the scale here. The widget's layout
    /// and the cursor it is given stay in its own logical units.
    ///
    /// The host's translation has to be a whole number of device pixels.
    /// At any other the software renderer resamples the images, which
    /// softens them and can show seams between cells; wgpu rounds each image
    /// to a device pixel.
    ///
    /// A change of magnification is a change of factor, and renders every
    /// cell again while the old images stand in. A host that animates its
    /// zoom passes the magnification once the zoom settles.
    ///
    /// A magnification that is not finite and positive is taken as 1.
    #[must_use]
    pub fn magnification(self, magnification: f32) -> Self {
        Plot {
            magnification,
            ..self
        }
    }

    /// The crosshair in the hovered panel, on by default. It is two lines
    /// one device pixel wide through the pointer, in the figure theme's ink
    /// at 40%.
    #[must_use]
    pub fn crosshair(self, on: bool) -> Self {
        Plot {
            crosshair: on,
            ..self
        }
    }

    /// The ring around the nearest data point, on by default. It circles
    /// the point of the hovered panel's marks nearest the pointer within 12
    /// logical px, in the figure theme's ink.
    #[must_use]
    pub fn highlight(self, on: bool) -> Self {
        Plot {
            highlight: on,
            ..self
        }
    }

    /// Where changed cells rasterize, [`Render::Auto`] by default.
    #[must_use]
    pub fn render(self, render: Render) -> Self {
        Plot { render, ..self }
    }

    /// How the widget reuses rendered images, [`Caching::On`] by default.
    /// [`Caching::Off`] re-renders every cell whenever any changed, as a
    /// diagnostic for the benchmark. [`Caching::Verify`] renders what it
    /// keeps again and reports what differs.
    #[must_use]
    pub fn caching(self, caching: Caching) -> Self {
        Plot { caching, ..self }
    }

    /// A revision the app bumps when something the hashes cannot see
    /// changed, such as a paint image's pixels replaced in place or a font
    /// swapped. Every cell then re-renders at the next refresh. It is 0 by
    /// default, and the widget compares each value with the last one it
    /// rendered.
    ///
    /// ```no_run
    /// # mod rustd3 { pub use rustd3_figure as figure; }
    /// use rustd3::figure::Figure;
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

    /// The render pool's size, clamped to 1..=64. Changing it restarts the
    /// pool at its next jobs. By default it is the available parallelism
    /// less the UI thread, from 1 to 8, which respects the process's CPU
    /// affinity: one thread on a 2-core machine, 8 on 16 cores. Every
    /// render the widget makes, on the pool or inline on the UI thread, is
    /// single-threaded, so the pool is the widget's only parallelism
    /// whatever its size.
    ///
    /// ```no_run
    /// # mod rustd3 { pub use rustd3_figure as figure; }
    /// use rustd3::figure::Figure;
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

    /// Reports to `probe` the per-stage durations, per-cell raster times
    /// and what each frame showed, timed by the probe's clock (see
    /// [`probe`](crate::plot::probe)). Without a probe nothing is sent. The
    /// widget times its renders for its cost model either way.
    ///
    /// ```no_run
    /// # mod rustd3 { pub use rustd3_figure as figure; }
    /// use std::sync::mpsc;
    ///
    /// use iced_core::time::Instant;
    /// use rustd3::figure::Figure;
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

    /// Changes to `handle`'s domain become messages, and the app owns the
    /// domain from then on. Without this, the widget keeps it.
    ///
    /// A gesture publishes one message per handle per frame, holding the
    /// domain's end stops in the scale's own type. Until the gesture ends,
    /// it continues from the domain it last published, so updates the app
    /// applies late lose no distance. Only continuous handles change. A
    /// later `on_change` for the same handle replaces this one. A handle of
    /// another figure has no domain here, so the listener is not added.
    ///
    /// The message carries the domain in the scale's own type: a time
    /// handle publishes instants, and a message of numbers for it does not
    /// compile (E0631, expected `fn([Timestamp; 2]) -> _`).
    ///
    /// ```compile_fail,E0631
    /// # mod rustd3 { pub use rustd3_core::time; pub use rustd3_figure as figure; }
    /// use rustd3::time;
    /// use rustd3::figure::Figure;
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
    /// # mod rustd3 { pub use rustd3_core::{jiff, time}; pub use rustd3_figure as figure; }
    /// use rustd3::time;
    /// use rustd3::figure::Figure;
    /// use rustd3_iced::Plot;
    ///
    /// enum Msg {
    ///     Range([rustd3::jiff::Timestamp; 2]),
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
    /// messages.
    ///
    /// While a gesture changes a selection, and when it ends, the message
    /// is `f(Some([a, b]))`. `a` and `b` are the values `over`'s scale
    /// inverts the selection's ends to, in their order along the axis (left
    /// to right, top to bottom). The message is `f(None)` when the gesture
    /// ends empty (a click without a drag) or a double click clears the
    /// selection. One message goes out per brush per frame, with the latest
    /// selection. A later `on_brush` for the same handle replaces this one.
    /// A handle of another figure has no brush here, so the listener is not
    /// added.
    ///
    /// A drag over a brushed plot area brushes, and the wheel still zooms.
    /// As in d3-brush, a press outside the selection starts a new one, a
    /// press on it moves it, and a press on its edges resizes it. The live
    /// rectangle and the selection band are drawn over the panel's plot
    /// area, and only the overlay redraws. With `.sets(target)` the
    /// selection sets the target's domain as a zoom does. That domain is
    /// published by `on_change`, or else is the widget's own, which fitting
    /// then skips until a double click. Between gestures the band follows
    /// the target's domain, so a zoom of the target moves it, and a target
    /// showing all of `over`'s domain shows no band.
    ///
    /// Only handles with a typed inverse have brushes. A selection over a
    /// band handle has no domain values, and `brush::x` does not take one.
    ///
    /// ```no_run
    /// # mod rustd3 { pub use rustd3_core::{jiff, time}; pub use rustd3_figure as figure; }
    /// use rustd3::jiff::Timestamp;
    /// use rustd3::time;
    /// use rustd3::figure::{Figure, brush};
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

    /// What the pointer is over becomes messages.
    ///
    /// `f(Some(hover))` goes out when the drawn point nearest the pointer
    /// changes, within 12 logical px and in the panel whose plot area holds
    /// the pointer. `f(None)` goes out once when there is no such point any
    /// more or the pointer left the widget. Moving within one point sends
    /// nothing. Moves between frames are hit-tested against an index of
    /// the drawn points built with the drawing list, so hovering resolves
    /// and renders nothing. At most one message goes out per frame, and
    /// none during a gesture. The highlight ring shows the same point.
    /// Tooltips are the app's own widgets, built from these messages.
    ///
    /// The index is built only for a widget with the highlight or
    /// `on_hover`, and not while a gesture lasts. The refresh after the
    /// gesture builds it, and nothing is hit-tested meanwhile.
    ///
    /// ```no_run
    /// # mod rustd3 { pub use rustd3_core::scale; pub use rustd3_figure as figure; }
    /// use rustd3::scale;
    /// use rustd3::figure::mark::dots;
    /// use rustd3::figure::Figure;
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

    /// What every curve shows at the pointer's x becomes messages, for a
    /// readout of every line in the panel.
    ///
    /// `f(Some(readout))` goes out whenever the pointer moves inside a
    /// panel's plot area: the panel, the pointer in figure px (the
    /// widget's logical px from its top-left corner) and, per line in the
    /// panel that spans the pointer's x, its place, its data x and y there
    /// in the zoomed view, its legend label and its color
    /// ([`hit::Index::read`]). `f(None)` goes out once when the pointer
    /// leaves every plot area or the widget. As with
    /// [`on_hover`](Plot::on_hover), at most one message goes out per
    /// frame, none during a gesture, and nothing renders; `on_hover` keeps
    /// working beside it.
    ///
    /// ```no_run
    /// # mod rustd3 { pub use rustd3_core::scale; pub use rustd3_figure as figure; }
    /// use rustd3::figure::hit::Readout;
    /// use rustd3::figure::mark::line;
    /// use rustd3::figure::Figure;
    /// use rustd3::scale;
    /// use rustd3_iced::Plot;
    ///
    /// #[derive(Debug, Clone)]
    /// enum Message {
    ///     Read(Option<Readout>),
    /// }
    ///
    /// let mut fig = Figure::new();
    /// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    /// fig.panel(0, 0).mark(line([(0.0, 1.0), (1.0, 3.0)], x, y));
    /// let plot: Plot<'_, Message> = Plot::new(fig).on_readout(Message::Read);
    /// # let _ = plot;
    /// ```
    #[must_use]
    pub fn on_readout(self, f: impl Fn(Option<hit::Readout>) -> Message + 'a) -> Self {
        Plot {
            on_readout: Some(Box::new(f)),
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
            scale_factor::magnified(
                scale_factor::resolve(self.scale_factor, renderer.scale(), state.window_factor()),
                self.magnification,
            ),
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
                    hover: self.highlight || self.on_hover.is_some() || self.on_readout.is_some(),
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
                if let Some(reading) = state.read()
                    && let Some(read) = &self.on_readout
                {
                    shell.publish(read(reading));
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
