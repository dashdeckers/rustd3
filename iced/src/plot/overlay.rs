//! [`Overlay`]: the crosshair and the nearest-point highlight, drawn as
//! quads over the cells and clipped to the hovered panel's plot area.

use iced_core::border;
use iced_core::renderer::{self, Quad};
use iced_core::{Color, Point, Rectangle, Size};
use rustd3_core::color::Rgb;
use rustd3_figure::{hit, kurbo};

/// Logical px within which the nearest drawn point is highlighted.
const RADIUS: f64 = 12.0;

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
    /// The overlay for a pointer at `at` (device px; `None` when it left
    /// or something above the widget covers it): the panel among `areas`
    /// whose plot area holds it, the crosshair there, and the nearest point
    /// of that panel's marks in `hits` (list px) within 12 logical px, at
    /// `factor` device px per logical px. Parts `shown` leaves out stay
    /// empty, so they never count as a change.
    pub(crate) fn hover(
        at: Option<Point>,
        areas: &[[i32; 4]],
        hits: &hit::Index,
        factor: f32,
        shown: Shown,
    ) -> Overlay {
        let Some(at) = at else {
            return Overlay::default();
        };
        let inside = |[x0, y0, x1, y1]: &[i32; 4]| {
            (*x0 as f32..*x1 as f32).contains(&at.x) && (*y0 as f32..*y1 as f32).contains(&at.y)
        };
        let Some((index, &panel)) = areas.iter().enumerate().find(|(_, area)| inside(area)) else {
            return Overlay::default();
        };
        let k = f64::from(factor);
        let near = kurbo::Point::new(f64::from(at.x) / k, f64::from(at.y) / k);
        let point = hits
            .nearest(near, RADIUS)
            .filter(|hit| shown.highlight && hit.panel == index)
            .map(|hit| Point::new((hit.at.x * k) as f32, (hit.at.y * k) as f32));
        Overlay {
            cursor: shown.crosshair.then_some(at),
            panel: Some(panel),
            point,
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
fn color(c: Rgb) -> Color {
    let c = c.clamp();
    let [r, g, b] = [c.r, c.g, c.b].map(|v| v as u8);
    Color::from_rgba8(r, g, b, c.opacity as f32)
}
