//! [`Overlay`]: the crosshair and the nearest-point highlight, drawn as
//! quads over the cells and clipped to the hovered panel's plot area.

use iced_core::border;
use iced_core::renderer::{self, Quad};
use iced_core::{Color, Point, Rectangle, Size};
use rustd3_core::color::Rgb;

/// The highlight ring's radius, logical px.
const RING: f32 = 4.0;

/// The highlight ring's border width, logical px.
const RING_WIDTH: f32 = 1.5;

/// The crosshair's share of the ink's opacity.
const CROSSHAIR_ALPHA: f32 = 0.4;

/// What the pointer over a widget shows, in device px from the widget's
/// device origin; empty when it is over no plot area.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Overlay {
    /// The cursor, when the crosshair shows.
    cursor: Option<Point>,
    /// Plot area (device rect) of the hovered panel.
    panel: Option<[i32; 4]>,
    /// Nearest point, device px, within 12 logical px of the cursor.
    point: Option<Point>,
}

/// Which parts of the overlay a widget shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Shown {
    /// The crosshair through the cursor.
    pub(crate) crosshair: bool,
    /// The ring around the nearest point.
    pub(crate) highlight: bool,
}

impl Overlay {
    /// The overlay over the plot area `panel` (device rect) of the hovered
    /// panel: the crosshair through `cursor` and the ring around `point`
    /// (device px from the widget's device origin), each when `shown` asks
    /// for it; empty over no plot area. Parts `shown` leaves out stay
    /// empty, so they never count as a change.
    pub(crate) fn new(
        cursor: Point,
        panel: Option<[i32; 4]>,
        point: Option<Point>,
        shown: Shown,
    ) -> Overlay {
        let Some(panel) = panel else {
            return Overlay::default();
        };
        Overlay {
            cursor: shown.crosshair.then_some(cursor),
            panel: Some(panel),
            point: point.filter(|_| shown.highlight),
        }
    }

    /// Draws the crosshair, two quads one device px wide in `ink` at 40%,
    /// and the ring, a 1.5 px border of radius 4 in `ink`, in a layer
    /// clipped to the hovered plot area, for a widget whose device origin
    /// is `origin`.
    pub(crate) fn draw<Renderer>(
        &self,
        renderer: &mut Renderer,
        origin: [i32; 2],
        factor: f32,
        ink: Rgb,
    ) where
        Renderer: renderer::Renderer,
    {
        let Some([x0, y0, x1, y1]) = self.panel else {
            return;
        };
        if self.cursor.is_none() && self.point.is_none() {
            return;
        }
        let logical = |x: f32, y: f32| {
            Point::new(
                (origin[0] as f32 + x) / factor,
                (origin[1] as f32 + y) / factor,
            )
        };
        let area = Rectangle::new(
            logical(x0 as f32, y0 as f32),
            Size::new((x1 - x0) as f32 / factor, (y1 - y0) as f32 / factor),
        );
        let ink = color(ink);
        renderer.with_layer(area, |renderer| {
            if let Some(at) = self.cursor {
                let line = Color {
                    a: ink.a * CROSSHAIR_ALPHA,
                    ..ink
                };
                let pixel = 1.0 / factor;
                let column = Rectangle::new(
                    logical(at.x.floor(), y0 as f32),
                    Size::new(pixel, area.height),
                );
                let row = Rectangle::new(
                    logical(x0 as f32, at.y.floor()),
                    Size::new(area.width, pixel),
                );
                for bounds in [column, row] {
                    renderer.fill_quad(
                        Quad {
                            bounds,
                            ..Quad::default()
                        },
                        line,
                    );
                }
            }
            if let Some(at) = self.point {
                let centre = logical(at.x, at.y);
                let ring = Quad {
                    bounds: Rectangle::new(
                        Point::new(centre.x - RING, centre.y - RING),
                        Size::new(2.0 * RING, 2.0 * RING),
                    ),
                    border: border::rounded(RING).width(RING_WIDTH).color(ink),
                    snap: false,
                    ..Quad::default()
                };
                renderer.fill_quad(ring, Color::TRANSPARENT);
            }
        });
    }
}

/// A figure color as iced's, channels clamped to 8 bits.
pub(crate) fn color(c: Rgb) -> Color {
    let c = c.clamp();
    let [r, g, b] = [c.r, c.g, c.b].map(|v| v as u8);
    Color::from_rgba8(r, g, b, c.opacity as f32)
}
