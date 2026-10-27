//! [`Plot`]: a [`Figure`] shown as images that vello_cpu renders from its
//! drawing list, the pixels the figure's PNG has, with a crosshair and a
//! nearest-point highlight drawn over them.
//!
//! `view` builds the figure from the app's state and hands it to the
//! widget by value; the widget keeps its own state (the figure's cache,
//! the rendered cells) in iced's widget tree between frames:
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
//! }
//!
//! #[derive(Debug, Clone)]
//! enum Message {}
//!
//! /// The app's data as a figure: one line on fitted scales.
//! fn figure(data: &[(f64, f64)]) -> Figure<'static> {
//!     let mut fig = Figure::new();
//!     let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
//!     fig.panel(0, 0)
//!         .mark(line(data.iter().copied(), x, y))
//!         .axis(axis::left(y))
//!         .axis(axis::bottom(x));
//!     fig
//! }
//!
//! impl App {
//!     fn view(&self) -> Element<'_, Message> {
//!         Plot::new(figure(&self.data)).into()
//!     }
//! }
//! ```
//!
//! The widget resolves and draws its figure once per `RedrawRequested`,
//! for the screen at the effective scale factor (`Plot::scale_factor`,
//! else the renderer's hint, else the window's factor, else 1), and only
//! when the figure was rebuilt, the widget resized or the factor changed.
//! It cuts the list into the cells of the canvas's partition by every
//! plot-area edge, renders only the cells whose content hash changed
//! (every cell when the partition's rects change), and draws each as
//! images placed on whole device pixels. A cell whose RGBA would reach
//! [`BAND_BYTES`] is rendered and drawn as horizontal bands below that
//! size, so the GPU renderer uploads every image inside the frame that
//! first draws it. Hovering renders no cell: the crosshair and the
//! highlight are quads in a layer clipped to the hovered plot area, in the
//! figure theme's ink.

#[cfg(test)]
mod fixture;
mod handle;
mod job;
mod overlay;
pub mod probe;
mod render;
mod scale_factor;
mod slot;
mod state;
mod worker;

use std::marker::PhantomData;

use iced_core::image;
use iced_core::layout::{self, Layout};
use iced_core::mouse;
use iced_core::renderer;
use iced_core::widget::{Meta, Tree, tree};
use iced_core::{Element, Event, Length, Rectangle, Shell, Size, Widget, window};
use rustd3_figure::Figure;

use overlay::Shown;
pub use probe::Probe;
pub use render::Render;
use state::{Pipeline, State};

/// RGBA bytes at which a cell is split into bands: wgpu uploads images
/// under 2 MiB inside the frame (`wgpu/src/image/cache.rs:223-246`)
/// and larger ones through a worker drained only by `Renderer::tick`.
pub const BAND_BYTES: usize = 2 * 1024 * 1024;

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
    probe: Option<Probe>,
    _message: PhantomData<Message>,
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
            probe: None,
            _message: PhantomData,
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

    /// Where changed cells rasterize; defaults to [`Render::Inline`].
    pub fn render(self, render: Render) -> Self {
        Plot { render, ..self }
    }

    /// Reports per-stage durations, per-cell raster times and what each
    /// frame showed to `probe`, timed by its clock (see [`probe`]).
    /// Without a probe nothing is sent and no clock is read.
    pub fn probe(self, probe: Probe) -> Self {
        Plot {
            probe: Some(probe),
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
        match event {
            Event::Window(
                window::Event::Opened { scale_factor, .. } | window::Event::Rescaled(scale_factor),
            ) => {
                state.rescale(*scale_factor);
                shell.request_redraw();
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                let factor = scale_factor::resolve(
                    self.scale_factor,
                    renderer.scale(),
                    state.window_factor(),
                );
                let pipeline = Pipeline {
                    render: self.render,
                    probe: self.probe.as_ref(),
                    waker: shell.waker(),
                };
                // New cells may move the plot areas or the points under
                // the pointer.
                if state.redraw(&self.figure, factor, *now, &pipeline) > 0 {
                    state.hover(cursor.position(), layout.bounds(), self.shown());
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.hover(cursor.position(), layout.bounds(), self.shown()) {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::CursorLeft) => {
                if state.hover(None, layout.bounds(), self.shown()) {
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
        let over = cursor.position().is_some_and(|at| {
            tree.state
                .downcast_ref::<State>()
                .is_over_area(at, layout.bounds())
        });
        if over {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::None
        }
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
