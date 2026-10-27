//! [`Plot`]: a [`Figure`] shown as images that vello_cpu renders from its
//! drawing list, the pixels the figure's PNG has.
//!
//! The widget resolves and draws its figure once per `RedrawRequested`,
//! for the screen at the effective scale factor (`Plot::scale_factor`,
//! else the renderer's hint, else the window's factor, else 1), and only
//! when the figure was rebuilt, the widget resized or the factor changed.
//! It cuts the list into the cells of the canvas's partition by every
//! plot-area edge, renders every cell, and draws each as images placed on
//! whole device pixels. A cell whose RGBA would reach [`BAND_BYTES`] is
//! rendered and drawn as horizontal bands below that size, so the GPU
//! renderer uploads every image inside the frame that first draws it.

mod cell;
#[cfg(test)]
mod fixture;
mod handle;
mod scale_factor;
mod state;

use std::marker::PhantomData;

use iced_core::image;
use iced_core::layout::{self, Layout};
use iced_core::mouse;
use iced_core::renderer;
use iced_core::widget::{Meta, Tree, tree};
use iced_core::{Element, Event, Length, Rectangle, Shell, Size, Widget, window};
use rustd3_figure::Figure;

use state::State;

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
        _layout: Layout,
        _cursor: mouse::Cursor,
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
            Event::Window(window::Event::RedrawRequested(_)) => {
                let factor = scale_factor::resolve(
                    self.scale_factor,
                    renderer.scale(),
                    state.window_factor(),
                );
                state.refresh(&self.figure, factor);
            }
            Event::Window(_)
            | Event::Mouse(_)
            | Event::Keyboard(_)
            | Event::Touch(_)
            | Event::InputMethod(_)
            | Event::Clipboard(_)
            | Event::Waken => {}
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
            .draw(renderer, layout.bounds());
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
