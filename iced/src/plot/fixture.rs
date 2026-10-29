//! Figures the widget's unit tests show, and [`Recorder`], a renderer that
//! records what the widget draws.

use iced_core::{Background, Rectangle, Size, Transformation, image, renderer};
use rustd3_core::scale;
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::legend::{self, Slot};
use rustd3_figure::mark::{dots, line};
use rustd3_figure::{Figure, axis};

/// A 2 × 2 grid: a line, dots, a line and dots, each panel with left and
/// bottom axes, a figure title and a legend right of the grid.
pub(crate) fn grid() -> Figure<'static> {
    let mut fig = Figure::new();
    let color = fig.scale(scale::ordinal(["sine", "noise"], COLORBLIND));
    fig.title("A grid of four");
    fig.legend(Slot::Right, legend::legend(color).title("series"));
    for (i, (row, col)) in [(0, 0), (0, 1), (1, 0), (1, 1)].into_iter().enumerate() {
        let x = fig.fit(scale::linear());
        let y = fig.fit(scale::linear());
        let samples = sine(200, i);
        let panel = fig.panel(row, col);
        if i % 2 == 0 {
            panel.mark(line(samples, x, y).stroke(color, "sine"));
        } else {
            panel.mark(dots(samples, x, y).fill(color, "noise"));
        }
        panel.axis(axis::left(y)).axis(axis::bottom(x));
    }
    fig
}

/// Two panels side by side: a line of 10k samples and 2k dots, each with
/// left and bottom axes, under a title.
pub(crate) fn two_panels() -> Figure<'static> {
    let mut fig = Figure::new();
    fig.title("Line and dots");
    let (x0, y0) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    let (x1, y1) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    fig.panel(0, 0)
        .mark(line(sine(10_000, 0), x0, y0))
        .axis(axis::left(y0))
        .axis(axis::bottom(x0));
    fig.panel(0, 1)
        .mark(dots(sine(2_000, 1), x1, y1))
        .axis(axis::left(y1))
        .axis(axis::bottom(x1));
    fig
}

/// Two panels side by side on fixed [0, 10] scales: a line of 200
/// samples, and dots at `points`, each with left and bottom axes.
pub(crate) fn pair(points: Vec<(f64, f64)>) -> Figure<'static> {
    let mut fig = Figure::new();
    let (x0, y0) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    let x1 = fig.scale(scale::linear().domain([0.0, 10.0]));
    let y1 = fig.scale(scale::linear().domain([0.0, 10.0]));
    fig.panel(0, 0)
        .mark(line(sine(200, 0), x0, y0))
        .axis(axis::left(y0))
        .axis(axis::bottom(x0));
    fig.panel(0, 1)
        .mark(dots(points, x1, y1))
        .axis(axis::left(y1))
        .axis(axis::bottom(x1));
    fig
}

/// `n` samples of a sine with a little deterministic wobble, phase `seed`.
fn sine(n: usize, seed: usize) -> Vec<(f64, f64)> {
    (0..n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let wobble = ((i * 7919 + seed * 104_729) % 1000) as f64 / 5000.0;
            (t * 10.0, (t * 25.0 + seed as f64).sin() + wobble)
        })
        .collect()
}

/// A renderer that records what a widget draws, for the tests of how it
/// draws: by default one that reports no scale, as iced's software
/// renderer does; [`Recorder::gpu`] reports one, as the GPU renderer does.
#[derive(Debug, Default)]
pub(crate) struct Recorder {
    pub(crate) drawn: Vec<Drawn>,
    scale: Option<renderer::Scale>,
}

/// One call a [`Recorder`] received.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Drawn {
    /// A layer opened with these bounds.
    Layer(Rectangle),
    /// The last layer opened closed.
    Close,
    /// An image drawn at these bounds, with linear filtering or not.
    Image { bounds: Rectangle, linear: bool },
    /// A quad filled.
    Quad,
}

impl Recorder {
    /// A recorder that reports a scale of 1, as the GPU renderer does.
    pub(crate) fn gpu() -> Recorder {
        Recorder {
            drawn: Vec::new(),
            scale: Some(renderer::Scale {
                window: 1.0,
                application: 1.0,
            }),
        }
    }

    /// The bounds of each image drawn, with linear filtering or not.
    pub(crate) fn drawn_images(&self) -> Vec<(Rectangle, bool)> {
        self.drawn
            .iter()
            .filter_map(|drawn| match drawn {
                Drawn::Image { bounds, linear } => Some((*bounds, *linear)),
                Drawn::Layer(_) | Drawn::Close | Drawn::Quad => None,
            })
            .collect()
    }

    /// The bounds of each layer opened, in order.
    pub(crate) fn layers(&self) -> Vec<Rectangle> {
        self.drawn
            .iter()
            .filter_map(|drawn| match drawn {
                Drawn::Layer(bounds) => Some(*bounds),
                Drawn::Close | Drawn::Image { .. } | Drawn::Quad => None,
            })
            .collect()
    }

    /// How many images were drawn, and how many of them with linear
    /// filtering (the previews).
    pub(crate) fn images(&self) -> (usize, usize) {
        self.drawn
            .iter()
            .fold((0, 0), |(all, linear), drawn| match drawn {
                Drawn::Image { linear: on, .. } => (all + 1, linear + usize::from(*on)),
                Drawn::Layer(_) | Drawn::Close | Drawn::Quad => (all, linear),
            })
    }

    /// Whether every image was drawn inside the first layer opened.
    pub(crate) fn images_in_first_layer(&self) -> bool {
        let mut depth = 0_i32;
        let mut first = false;
        let mut inside = true;
        for drawn in &self.drawn {
            match drawn {
                Drawn::Layer(_) => {
                    depth += 1;
                    first = first || depth == 1;
                }
                Drawn::Close => depth -= 1,
                Drawn::Image { .. } => inside &= first && depth == 1,
                Drawn::Quad => {}
            }
        }
        inside
    }
}

impl iced_core::Renderer for Recorder {
    fn start_layer(&mut self, bounds: Rectangle) {
        self.drawn.push(Drawn::Layer(bounds));
    }

    fn end_layer(&mut self) {
        self.drawn.push(Drawn::Close);
    }

    fn start_transformation(&mut self, _transformation: Transformation) {}

    fn end_transformation(&mut self) {}

    fn fill_quad(&mut self, _quad: renderer::Quad, _background: impl Into<Background>) {
        self.drawn.push(Drawn::Quad);
    }

    fn allocate_image(
        &self,
        _handle: &image::Handle,
        callback: impl FnOnce(Result<image::Allocation, image::Error>) + Send + 'static,
    ) {
        callback(Err(image::Error::Unsupported));
    }

    fn hint(&mut self, _scale: renderer::Scale) {}

    fn scale(&self) -> Option<renderer::Scale> {
        self.scale
    }

    fn reset(&mut self, _new_bounds: Rectangle) {
        self.drawn.clear();
    }

    fn settings(&self) -> renderer::Settings {
        renderer::Settings::default()
    }
}

impl image::Renderer for Recorder {
    type Handle = image::Handle;

    fn load_image(&self, _handle: &Self::Handle) -> Result<image::Allocation, image::Error> {
        Err(image::Error::Unsupported)
    }

    fn measure_image(&self, _handle: &Self::Handle) -> Option<Size<u32>> {
        None
    }

    fn draw_image(
        &mut self,
        image: image::Image<Self::Handle>,
        bounds: Rectangle,
        _clip: Rectangle,
    ) {
        self.drawn.push(Drawn::Image {
            bounds,
            linear: image.filter_method == image::FilterMethod::Linear,
        });
    }
}
