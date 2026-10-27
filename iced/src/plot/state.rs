//! [`State`]: what the widget keeps in iced's widget tree between frames,
//! and the refresh that renders a figure's cells.

use iced_core::{Point, Rectangle, Size, image};
use rustd3_figure::list::{self, Bounds, Outlines};
use rustd3_figure::{Cache, Figure, Held, Target, kurbo, raster};

use crate::plot::cell::Cell;

/// The widget's tree state: the figure's cache, the rasterizer, and the
/// cells last rendered. It borrows nothing of the figure.
pub(crate) struct State {
    /// Laid-out labels, thinned lines and `Fit::Visible` memory.
    cache: Cache,
    /// vello_cpu with its own pool, built once.
    raster: raster::Renderer,
    /// Glyph outlines the list's command bounds are measured with.
    outlines: Outlines,
    /// The widget's logical size from the last layout.
    size: Size,
    /// The window's factor from `Opened` or `Rescaled`.
    window: Option<f32>,
    /// The factor the cells were rendered at; `None` before the first
    /// refresh.
    factor: Option<f32>,
    /// Whether the figure was rebuilt or the widget resized since the
    /// last refresh.
    stale: bool,
    /// The partition's cells, row-major.
    cells: Vec<Cell>,
    /// Plot areas in device px, relative to the widget's device origin.
    areas: Vec<[i32; 4]>,
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
            cells: Vec::new(),
            areas: Vec::new(),
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

    /// Rebuilds the list when the figure was rebuilt, the widget resized or
    /// the factor changed: resolves and draws `figure` at the widget's size
    /// for the screen at `factor`, cuts the list into cells at every
    /// plot-area edge and, unless every cell's device rect and hash is the
    /// stored one, renders every band of every cell and keeps the plot
    /// areas. Returns the number of cells rendered.
    pub(crate) fn refresh(&mut self, figure: &Figure<'_>, factor: f32) -> usize {
        if !self.stale && self.factor == Some(factor) {
            return 0;
        }
        self.stale = false;
        self.factor = Some(factor);
        let k = f64::from(factor);
        let size = kurbo::Size::new(f64::from(self.size.width), f64::from(self.size.height));
        if !((size.width * k).round() >= 1.0 && (size.height * k).round() >= 1.0) {
            self.cells.clear();
            self.areas.clear();
            return 0;
        }
        let target = Target::Screen { scale: k };
        let held = Held::default();
        let list = figure.draw_with(&mut self.cache, size, target, &held);
        let resolved = figure.resolve(size, &mut self.cache, &held, target);
        let (xs, ys) = resolved.edges();
        let cells = list::cells(&list, &xs, &ys, k);
        let unchanged = self.cells.len() == cells.cells.len()
            && self.cells.iter().zip(&cells.cells).all(|(a, b)| a.shows(b));
        if unchanged {
            return 0;
        }
        let bounds = Bounds::of(&list, &mut self.outlines);
        self.cells = cells
            .cells
            .iter()
            .map(|cell| Cell::render(cell, &list, &bounds, k, &mut self.raster))
            .collect();
        self.areas = resolved
            .plot_areas()
            .iter()
            .map(|area| [area.x0, area.y0, area.x1, area.y1].map(|edge| (edge * k).round() as i32))
            .collect();
        self.cells.len()
    }

    /// Draws every cell for a widget laid out at `bounds`.
    pub(crate) fn draw<Renderer>(&self, renderer: &mut Renderer, bounds: Rectangle)
    where
        Renderer: image::Renderer<Handle = image::Handle>,
    {
        let Some(factor) = self.factor else {
            return;
        };
        let origin = origin(bounds, factor);
        for cell in &self.cells {
            cell.draw(renderer, origin, factor);
        }
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
    use iced_core::Size;

    use super::State;
    use crate::plot::fixture;

    /// The factors layout, snapping and placement are checked at.
    const FACTORS: [f32; 4] = [1.0, 1.25, 1.5, 2.0];

    /// A state laid out at `width` × `height` logical px.
    fn state(width: f32, height: f32) -> State {
        let mut state = State::default();
        state.resize(Size::new(width, height));
        state
    }

    #[test]
    fn cells_partition_the_canvas() {
        let fig = fixture::grid();
        for k in FACTORS {
            let mut state = state(640.0, 480.0);
            let rendered = state.refresh(&fig, k);
            let canvas = [0, 0, (640.0 * k).round() as i32, (480.0 * k).round() as i32];
            let rects: Vec<[i32; 4]> = state.cells.iter().map(|cell| cell.rect()).collect();
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
    fn unchanged_figure_renders_no_cell() {
        let mut state = state(640.0, 480.0);
        let first = state.refresh(&fixture::grid(), 1.25);
        assert!(first > 0);
        let shown: Vec<_> = state.cells.iter().map(|cell| cell.images()).collect();
        // The view is rebuilt: a new figure, equal to the last.
        state.rebuilt();
        assert_eq!(state.refresh(&fixture::grid(), 1.25), 0);
        let kept: Vec<_> = state.cells.iter().map(|cell| cell.images()).collect();
        assert_eq!(kept, shown);
        // Nothing rebuilt, resized or rescaled: no refresh at all.
        assert_eq!(state.refresh(&fixture::grid(), 1.25), 0);
        // A new size renders every cell again.
        state.resize(Size::new(600.0, 480.0));
        assert!(state.refresh(&fixture::grid(), 1.25) > 0);
    }

    #[test]
    fn cell_images_have_their_device_size() {
        let fig = fixture::grid();
        for k in [1.25, 1.5] {
            let mut state = state(640.0, 480.0);
            state.refresh(&fig, k);
            for cell in &state.cells {
                let [x0, y0, x1, y1] = cell.rect();
                let images = cell.images();
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
    fn a_widget_of_no_device_pixel_has_no_cells() {
        let mut state = state(0.0, 480.0);
        assert_eq!(state.refresh(&fixture::grid(), 1.0), 0);
        assert!(state.cells.is_empty());
    }
}
