//! Phase 31g: a real gesture frame of the `Plot` widget composed by
//! iced's software renderer, headlessly. The widget is driven on
//! tiny-skia through a zoom burst (a wheel line every 33 ms) and a drag (5
//! px a frame) over `trace` and `scatter100k` at factors 1 and 2, every
//! frame composed as a window composes it, paced at 60 Hz with the pool's
//! results landing between frames. One frame mid-burst and one mid-drag
//! are kept: their images (and the scaled ones among them), the handles
//! the renderer had not drawn the frame before, and what composing the
//! frame costs, the first time (which converts the new handles) and warm
//! (the median of later compositions of the same frame). The same images
//! are then replayed through a widget that only draws them, as captured
//! and as each candidate would draw them: (a) every image with nearest
//! filtering, (b) each plot area's images composed into one image on the
//! CPU first (row copies for images at 1:1, nearest sampling for scaled
//! ones), so iced draws one image per area.

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use iced::advanced::graphics;
use iced::advanced::image::{self, FilterMethod, Handle};
use iced::advanced::layout;
use iced::advanced::renderer::{self as iced_renderer, Headless as _};
use iced::advanced::widget::{Meta, Tree, Widget};
use iced::advanced::{Layout, Renderer as _, shell};
use iced::{Event, Length, Rectangle, mouse, window};
use iced_runtime::user_interface::{self, UserInterface};
use rustd3_figure::kurbo::Size;
use rustd3_figure::{Cache, Held, Target};
use rustd3_iced::Plot;
use rustd3_iced::plot::Render;

use super::{Row, failed};
use crate::scenario::WINDOW;
use crate::stats::percentile;
use crate::workload::Workload;

/// The gestures a frame is kept from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Gesture {
    /// The frame after the 8th wheel line of a burst.
    Zoom,
    /// The 30th frame of a drag.
    Drag,
}

/// One image a frame draws: its handle, filter, bounds and clip (logical
/// px), and whether every pixel of it is opaque.
#[derive(Debug, Clone)]
struct Drawn {
    handle: Handle,
    filter: FilterMethod,
    bounds: Rectangle,
    clip: Rectangle,
    opaque: bool,
}

/// A kept frame: its images, the plot areas' device rects, the handles new
/// to the renderer (count and bytes) and its composition, first and warm,
/// ms.
struct Kept {
    images: Vec<Drawn>,
    areas: Vec<[i32; 4]>,
    new: (usize, u64),
    first: f64,
    warm: f64,
}

/// What composing one replayed frame costs, ms: the first composition and
/// the warm median; and its largest channel difference from the frame as
/// captured.
struct Composed {
    first: f64,
    warm: f64,
    max: u8,
}

/// A kept frame replayed: no image, as captured, (a) every image nearest
/// and (b) one image per plot area, composed on the CPU in `cpu` ms.
struct Replayed {
    empty: Composed,
    captured: Composed,
    nearest: Composed,
    area: Composed,
    cpu: f64,
}

/// A widget that draws a list of images, in one layer over its bounds.
struct Shown {
    images: Vec<Drawn>,
}

impl Meta for Shown {}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for Shown {
    fn size(&self) -> iced::Size<Length> {
        iced::Size::new(Length::Fill, Length::Fill)
    }

    fn layout(&mut self, tree: &mut Tree, _renderer: &iced::Renderer, limits: &layout::Limits) {
        tree.size = layout::atomic(limits, Length::Fill, Length::Fill);
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &iced_renderer::Style,
        layout: Layout,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        renderer.with_layer(layout.bounds(), |renderer| {
            for drawn in &self.images {
                let mut image = image::Image::new(drawn.handle.clone());
                image.filter_method = drawn.filter;
                image::Renderer::draw_image(renderer, image, drawn.bounds, drawn.clip);
            }
        });
    }
}

/// The row: per workload, gesture and factor, the kept frame and the
/// candidates replayed.
pub(super) fn frames() -> Row {
    let item = "Phase 31g: a gesture frame composed by tiny-skia (headless, 1600 × 1000 logical, \
                Auto, the default pool)"
        .to_owned();
    let mut parts = Vec::new();
    for workload in [Workload::Trace, Workload::Scatter100k] {
        for gesture in [Gesture::Zoom, Gesture::Drag] {
            for k in [1.0_f32, 2.0] {
                let Some(kept) = keep(workload, gesture, k) else {
                    return failed(&item, "no headless tiny-skia renderer");
                };
                parts.push(describe(workload, gesture, k, &kept));
            }
        }
    }
    Row {
        item,
        value: parts.join("; "),
    }
}

/// One cell of the row.
fn describe(workload: Workload, gesture: Gesture, k: f32, kept: &Kept) -> String {
    let scaled = kept
        .images
        .iter()
        .filter(|d| d.filter == FilterMethod::Linear)
        .count();
    let pixels: f64 = kept
        .images
        .iter()
        .map(|d| {
            let clip = d
                .clip
                .intersection(&d.bounds)
                .unwrap_or(Rectangle::default());
            f64::from(clip.width * clip.height * k * k)
        })
        .sum();
    let gesture = match gesture {
        Gesture::Zoom => "mid-burst zoom",
        Gesture::Drag => "mid-drag pan",
    };
    let mut text = format!(
        "{} {gesture} at {k}: {} images ({scaled} scaled), {:.2} Mpx drawn, {} new handles \
         ({:.2} MB); composed first {:.1} ms, warm {:.1} ms",
        workload.name(),
        kept.images.len(),
        pixels / 1e6,
        kept.new.0,
        kept.new.1 as f64 / 1e6,
        kept.first,
        kept.warm
    );
    if let Some(r) = replay(kept, k) {
        text.push_str(&format!(
            "; replayed warm: no image {:.1}, as captured {:.1}, (a) nearest {:.1} (max {}/255), \
             (b) one image per area {:.1}, first {:.1} + CPU {:.1} (max {}/255)",
            r.empty.warm,
            r.captured.warm,
            r.nearest.warm,
            r.nearest.max,
            r.area.warm,
            r.area.first,
            r.cpu,
            r.area.max
        ));
    }
    text
}

/// Drives the widget through `gesture` over `workload` at `k` on a
/// headless tiny-skia renderer, every frame composed, and keeps the
/// gesture's frame; `None` without the renderer.
fn keep(workload: Workload, gesture: Gesture, k: f32) -> Option<Kept> {
    let renderer = iced::futures::executor::block_on(iced::Renderer::new(
        iced_renderer::Settings::default(),
        Some("tiny-skia"),
    ))?;
    if !matches!(renderer, iced::Renderer::Secondary(_)) {
        return None;
    }
    let data = workload.data(1.0);
    let size = Size::new(f64::from(WINDOW.width), f64::from(WINDOW.height));
    let target = Target::Screen {
        scale: f64::from(k),
    };
    let areas: Vec<[i32; 4]> = data
        .figure(false)
        .resolve(size, &mut Cache::default(), &Held::default(), target)
        .plot_areas()
        .iter()
        .map(|a| [a.x0, a.y0, a.x1, a.y1].map(|e| (e * f64::from(k)).round() as i32))
        .collect();
    let plot = Plot::<()>::new(data.figure(false))
        .scale_factor(k)
        .render(Render::Auto)
        .crosshair(false)
        .highlight(false);
    let bounds = iced::Size::new(WINDOW.width, WINDOW.height);
    let mut window = Window::new(renderer, k);
    let mut ui = UserInterface::build(
        plot,
        bounds,
        user_interface::Cache::default(),
        &mut window.renderer,
    );
    // The top panel, right of its middle, where the bench's gestures act.
    let mut at = iced::Point::new(0.5 * bounds.width + 300.0, 0.27 * bounds.height);
    // Idle until every tile and the ring have landed.
    let begin = Instant::now();
    while begin.elapsed() < Duration::from_secs(2) {
        window.frame(&mut ui, Vec::new(), at);
    }
    match gesture {
        Gesture::Zoom => {
            let mut lines = 0;
            let mut last = Instant::now() - Duration::from_millis(33);
            while lines < 8 {
                let mut events = vec![Event::Mouse(mouse::Event::CursorMoved { position: at })];
                if last.elapsed() >= Duration::from_millis(33) {
                    events.push(Event::Mouse(mouse::Event::WheelScrolled {
                        delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
                    }));
                    last = Instant::now();
                    lines += 1;
                }
                window.frame(&mut ui, events, at);
            }
        }
        Gesture::Drag => {
            let press = vec![
                Event::Mouse(mouse::Event::CursorMoved { position: at }),
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            ];
            window.frame(&mut ui, press, at);
            for _ in 0..29 {
                at.x -= 5.0;
                let moved = vec![Event::Mouse(mouse::Event::CursorMoved { position: at })];
                window.frame(&mut ui, moved, at);
            }
        }
    }
    window.keep(&mut ui, at, areas)
}

/// A headless window: its renderer, the widget's bus and waker, the wakes
/// it counted and handled, and the handles the last composition drew.
struct Window {
    renderer: iced::Renderer,
    bus: shell::Bus<()>,
    waker: shell::Waker,
    wakes: Arc<AtomicUsize>,
    handled: usize,
    k: f32,
    drawn: HashSet<image::Id>,
}

impl Window {
    fn new(renderer: iced::Renderer, k: f32) -> Window {
        let wakes = Arc::new(AtomicUsize::new(0));
        let waker = shell::Waker::new({
            let wakes = Arc::clone(&wakes);
            move || {
                wakes.fetch_add(1, Ordering::SeqCst);
            }
        });
        Window {
            renderer,
            bus: shell::Bus::new(),
            waker,
            wakes,
            handled: 0,
            k,
            drawn: HashSet::new(),
        }
    }

    /// The window's device size.
    fn physical(&self) -> iced::Size<u32> {
        iced::Size::new(
            (WINDOW.width * self.k).round() as u32,
            (WINDOW.height * self.k).round() as u32,
        )
    }

    /// One frame: the pool's results if it woke the window, `events` and a
    /// `RedrawRequested`, the draw and its composition, then a wait to the
    /// next 60 Hz frame. The images the frame drew.
    fn frame(
        &mut self,
        ui: &mut UserInterface<'_, (), iced::Theme, iced::Renderer>,
        mut events: Vec<Event>,
        at: iced::Point,
    ) -> Vec<Drawn> {
        let begin = Instant::now();
        let wakes = self.wakes.load(Ordering::SeqCst);
        if wakes != self.handled {
            self.handled = wakes;
            events.insert(0, Event::Waken);
        }
        events.push(Event::Window(window::Event::RedrawRequested(begin)));
        self.renderer.tick();
        let _ = ui.update(
            &window::Headless,
            &self.waker,
            &events,
            mouse::Cursor::Available(at),
            &mut self.renderer,
            &mut self.bus,
        );
        let images = self.draw(ui, at);
        let _ = self
            .renderer
            .screenshot(self.physical(), self.k, iced::Color::WHITE);
        self.drawn = images.iter().map(|d| d.handle.id()).collect();
        let next = begin + Duration::from_micros(16_667);
        if let Some(wait) = next.checked_duration_since(Instant::now()) {
            std::thread::sleep(wait);
        }
        images
    }

    /// Draws `ui` and returns the raster images the renderer holds.
    fn draw(
        &mut self,
        ui: &mut UserInterface<'_, (), iced::Theme, iced::Renderer>,
        at: iced::Point,
    ) -> Vec<Drawn> {
        let style = iced_renderer::Style {
            text_color: iced::Color::BLACK,
        };
        ui.draw(
            &mut self.renderer,
            &iced::Theme::Light,
            &style,
            mouse::Cursor::Available(at),
        );
        images(&mut self.renderer)
    }

    /// The next frame, kept: drawn, its new handles counted against the
    /// last composition, composed once (converting them) and then 8 times
    /// more for the warm median.
    fn keep(
        &mut self,
        ui: &mut UserInterface<'_, (), iced::Theme, iced::Renderer>,
        at: iced::Point,
        areas: Vec<[i32; 4]>,
    ) -> Option<Kept> {
        let wakes = self.wakes.load(Ordering::SeqCst);
        let mut events = Vec::new();
        if wakes != self.handled {
            self.handled = wakes;
            events.push(Event::Waken);
        }
        events.push(Event::Window(
            window::Event::RedrawRequested(Instant::now()),
        ));
        self.renderer.tick();
        let _ = ui.update(
            &window::Headless,
            &self.waker,
            &events,
            mouse::Cursor::Available(at),
            &mut self.renderer,
            &mut self.bus,
        );
        let mut images = self.draw(ui, at);
        let new: Vec<&Drawn> = images
            .iter()
            .filter(|d| !self.drawn.contains(&d.handle.id()))
            .collect();
        let new = (
            new.len(),
            new.iter()
                .map(|d| match &d.handle {
                    Handle::Rgba { pixels, .. } => pixels.len() as u64,
                    Handle::Path(..) | Handle::Bytes(..) => 0,
                })
                .sum(),
        );
        let physical = self.physical();
        let begin = Instant::now();
        let _ = self
            .renderer
            .screenshot(physical, self.k, iced::Color::WHITE);
        let first = begin.elapsed().as_secs_f64() * 1e3;
        let mut times = Vec::new();
        for _ in 0..8 {
            let _ = self.draw(ui, at);
            let begin = Instant::now();
            let _ = self
                .renderer
                .screenshot(physical, self.k, iced::Color::WHITE);
            times.push(begin.elapsed().as_secs_f64() * 1e3);
        }
        for drawn in &mut images {
            drawn.opaque = match &drawn.handle {
                Handle::Rgba { pixels, .. } => pixels.chunks_exact(4).all(|p| p[3] == 255),
                Handle::Path(..) | Handle::Bytes(..) => false,
            };
        }
        Some(Kept {
            images,
            areas,
            new,
            first,
            warm: percentile(&times, 50.0)?,
        })
    }
}

/// The raster images the tiny-skia renderer holds, in draw order.
fn images(renderer: &mut iced::Renderer) -> Vec<Drawn> {
    let iced::Renderer::Secondary(renderer) = renderer else {
        return Vec::new();
    };
    renderer
        .layers()
        .iter()
        .flat_map(|layer| &layer.images)
        .filter_map(|image| match image {
            graphics::Image::Raster {
                image,
                bounds,
                clip_bounds,
            } => Some(Drawn {
                handle: image.handle.clone(),
                filter: image.filter_method,
                bounds: *bounds,
                clip: *clip_bounds,
                opaque: false,
            }),
            graphics::Image::Vector { .. } => None,
        })
        .collect()
}

/// `kept`'s frame replayed on a fresh tiny-skia renderer: no image, as
/// captured, (a) every image nearest, (b) one image per plot area composed
/// on the CPU; `None` without the renderer.
fn replay(kept: &Kept, k: f32) -> Option<Replayed> {
    let mut renderer = iced::futures::executor::block_on(iced::Renderer::new(
        iced_renderer::Settings::default(),
        Some("tiny-skia"),
    ))?;
    let physical = iced::Size::new(
        (WINDOW.width * k).round() as u32,
        (WINDOW.height * k).round() as u32,
    );
    let bounds = iced::Size::new(WINDOW.width, WINDOW.height);
    let mut compose = |images: Vec<Drawn>| -> (Composed, Vec<u8>) {
        let mut ui: UserInterface<'_, (), iced::Theme, iced::Renderer> = UserInterface::build(
            Shown { images }._boxed(),
            bounds,
            user_interface::Cache::default(),
            &mut renderer,
        );
        let style = iced_renderer::Style {
            text_color: iced::Color::BLACK,
        };
        let mut times = Vec::new();
        let mut pixels = Vec::new();
        for _ in 0..9 {
            ui.draw(
                &mut renderer,
                &iced::Theme::Light,
                &style,
                mouse::Cursor::Unavailable,
            );
            let begin = Instant::now();
            pixels = renderer.screenshot(physical, k, iced::Color::WHITE);
            times.push(begin.elapsed().as_secs_f64() * 1e3);
        }
        let first = times.remove(0);
        let warm = percentile(&times, 50.0).unwrap_or(f64::NAN);
        (
            Composed {
                first,
                warm,
                max: 0,
            },
            pixels,
        )
    };
    let (empty, _) = compose(Vec::new());
    let (captured, reference) = compose(kept.images.clone());
    let diff = |pixels: &[u8]| {
        pixels
            .iter()
            .zip(&reference)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap_or(0)
    };
    let nearest_images = kept
        .images
        .iter()
        .map(|d| Drawn {
            filter: FilterMethod::Nearest,
            ..d.clone()
        })
        .collect();
    let (mut nearest, pixels) = compose(nearest_images);
    nearest.max = diff(&pixels);
    let begin = Instant::now();
    let per_area = per_area(&kept.images, &kept.areas, k);
    let cpu = begin.elapsed().as_secs_f64() * 1e3;
    let (mut area, pixels) = compose(per_area);
    area.max = diff(&pixels);
    Some(Replayed {
        empty,
        captured,
        nearest,
        area,
        cpu,
    })
}

/// `images` with each plot area's (those clipped inside one of `areas`,
/// device px at `k`) composed into one image of the area in their order,
/// drawn where the first of them was: images at 1:1 by rows, opaque rows
/// copied, scaled ones by nearest sampling, each pixel composited once
/// per image source-over in straight RGBA.
fn per_area(images: &[Drawn], areas: &[[i32; 4]], k: f32) -> Vec<Drawn> {
    let device =
        |r: Rectangle| [r.x, r.y, r.x + r.width, r.y + r.height].map(|e| (e * k).round() as i32);
    let inside = |d: &Drawn| {
        let [x0, y0, x1, y1] = device(d.clip);
        areas
            .iter()
            .position(|a| a[0] <= x0 && a[1] <= y0 && x1 <= a[2] && y1 <= a[3])
    };
    let mut out: Vec<Drawn> = Vec::new();
    let mut buffers: Vec<Option<(usize, Vec<u8>)>> = vec![None; areas.len()];
    for drawn in images {
        let Some(a) = inside(drawn) else {
            out.push(drawn.clone());
            continue;
        };
        let [ax0, ay0, ax1, ay1] = areas[a];
        let width = (ax1 - ax0) as usize;
        let (_, buffer) = buffers[a].get_or_insert_with(|| {
            out.push(drawn.clone());
            (out.len() - 1, vec![0; width * (ay1 - ay0) as usize * 4])
        });
        let Handle::Rgba {
            width: iw,
            height: ih,
            pixels,
            ..
        } = &drawn.handle
        else {
            continue;
        };
        let (iw, ih) = (*iw as usize, *ih as usize);
        let [bx0, by0, bx1, by1] = [
            drawn.bounds.x,
            drawn.bounds.y,
            drawn.bounds.x + drawn.bounds.width,
            drawn.bounds.y + drawn.bounds.height,
        ]
        .map(|e| e * k);
        let [cx0, cy0, cx1, cy1] = device(drawn.clip);
        let (x0, y0) = (
            cx0.max(ax0).max(bx0.floor() as i32),
            cy0.max(ay0).max(by0.floor() as i32),
        );
        let (x1, y1) = (
            cx1.min(ax1).min(bx1.ceil() as i32),
            cy1.min(ay1).min(by1.ceil() as i32),
        );
        if x0 >= x1 || y0 >= y1 {
            continue;
        }
        let (sx, sy) = (iw as f32 / (bx1 - bx0), ih as f32 / (by1 - by0));
        let whole = (sx - 1.0).abs() < 1e-3 && (sy - 1.0).abs() < 1e-3;
        for y in y0..y1 {
            let v = (((y as f32 + 0.5 - by0) * sy) as usize).min(ih - 1);
            let row = &mut buffer[((y - ay0) as usize * width) * 4..][..width * 4];
            if whole && drawn.opaque {
                let u0 = (x0 as f32 + 0.5 - bx0) as usize;
                let n = (x1 - x0) as usize;
                let from = &pixels[(v * iw + u0) * 4..][..n * 4];
                row[(x0 - ax0) as usize * 4..][..n * 4].copy_from_slice(from);
                continue;
            }
            for x in x0..x1 {
                let u = (((x as f32 + 0.5 - bx0) * sx) as usize).min(iw - 1);
                let s = &pixels[(v * iw + u) * 4..][..4];
                let d = &mut row[(x - ax0) as usize * 4..][..4];
                over(d, s);
            }
        }
    }
    for (a, slot) in buffers.into_iter().enumerate() {
        let Some((index, buffer)) = slot else {
            continue;
        };
        let [x0, y0, x1, y1] = areas[a];
        let rect = Rectangle {
            x: x0 as f32 / k,
            y: y0 as f32 / k,
            width: (x1 - x0) as f32 / k,
            height: (y1 - y0) as f32 / k,
        };
        out[index] = Drawn {
            handle: Handle::from_rgba((x1 - x0) as u32, (y1 - y0) as u32, buffer),
            filter: FilterMethod::Nearest,
            bounds: rect,
            clip: rect,
            opaque: false,
        };
    }
    out
}

/// Straight-alpha source-over of pixel `s` onto `d`.
fn over(d: &mut [u8], s: &[u8]) {
    match s[3] {
        0 => {}
        255 => d.copy_from_slice(s),
        a => {
            let sa = u32::from(a);
            let da = u32::from(d[3]) * (255 - sa) / 255;
            let out = sa + da;
            for c in 0..3 {
                let v = (u32::from(s[c]) * sa + u32::from(d[c]) * da) / out.max(1);
                d[c] = v as u8;
            }
            d[3] = out as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use iced::Rectangle;
    use iced::advanced::image::{FilterMethod, Handle};

    use super::{Drawn, per_area};

    /// A `width` × `height` image of one opaque color.
    fn solid(width: u32, height: u32, rgb: [u8; 3]) -> Handle {
        let pixel = [rgb[0], rgb[1], rgb[2], 255];
        Handle::from_rgba(width, height, pixel.repeat((width * height) as usize))
    }

    /// The image at logical rect `(x, y, w, h)`, clipped there, nearest.
    fn at(handle: Handle, (x, y, w, h): (f32, f32, f32, f32), opaque: bool) -> Drawn {
        let rect = Rectangle {
            x,
            y,
            width: w,
            height: h,
        };
        Drawn {
            handle,
            filter: FilterMethod::Nearest,
            bounds: rect,
            clip: rect,
            opaque,
        }
    }

    #[test]
    fn an_areas_images_compose_into_one_and_the_rest_stay() {
        // At factor 2: a gutter strip outside the area, then two 2 × 2 px
        // tiles side by side in the 4 × 2 px area at (2, 2), then a
        // half-transparent 1 × 1 px overlay over the second tile, drawn
        // over 2 × 2 px (scaled).
        let gutter = at(solid(4, 2, [9, 9, 9]), (0.0, 0.0, 2.0, 1.0), true);
        let left = at(solid(2, 2, [255, 0, 0]), (1.0, 1.0, 1.0, 1.0), true);
        let right = at(solid(2, 2, [0, 0, 255]), (2.0, 1.0, 1.0, 1.0), true);
        let veil = Handle::from_rgba(1, 1, vec![255, 255, 255, 128]);
        let veil = at(veil, (2.0, 1.0, 1.0, 1.0), false);
        let drawn = per_area(&[gutter, left, right, veil], &[[2, 2, 6, 4]], 2.0);
        assert_eq!(drawn.len(), 2, "the gutter, then the area's one image");
        let Handle::Rgba {
            width,
            height,
            pixels,
            ..
        } = &drawn[1].handle
        else {
            panic!("an RGBA handle");
        };
        assert_eq!((*width, *height), (4, 2));
        assert_eq!(
            drawn[1].bounds,
            Rectangle {
                x: 1.0,
                y: 1.0,
                width: 2.0,
                height: 1.0
            }
        );
        let px = |x: usize, y: usize| &pixels[(y * 4 + x) * 4..][..4];
        assert_eq!(px(0, 0), [255, 0, 0, 255]);
        assert_eq!(px(1, 1), [255, 0, 0, 255]);
        // The veil over blue: (255·128 + 0·127) / 255.
        assert_eq!(px(2, 0), [128, 128, 255, 255]);
        assert_eq!(px(3, 1), [128, 128, 255, 255]);
    }
}
