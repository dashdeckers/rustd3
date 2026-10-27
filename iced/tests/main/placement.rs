//! Image placement on both backends at factors 1, 1.25, 1.5 and 2, which
//! pins the two constants the `Plot` widget is built on: `NUDGE`, the bias
//! that lands an image on exact device pixels, and `BAND_BYTES`, the
//! largest image wgpu uploads inside the frame that first draws it.

use std::time::Duration;

use iced::advanced::image::{self, Renderer as _};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Meta, Tree, Widget};
use iced::{Length, Rectangle, Size, Theme, mouse};
use rustd3_testkit::pixels;

use crate::headless::{self, Harness, Shot};

/// An RGBA image of fewer bytes uploads inside the frame on wgpu.
pub const BAND_BYTES: usize = 2 * 1024 * 1024;
/// Device px added to an image's origin before dividing by the factor.
pub const NUDGE: f32 = 0.25;

/// The factors every placement is checked at.
const FACTORS: [f32; 4] = [1.0, 1.25, 1.5, 2.0];
const RED: [u8; 4] = [255, 0, 0, 255];
const WHITE: [u8; 4] = [255; 4];
/// The checkerboard's two colors, mid-tones so that a color conversion
/// shows.
const CHECKS: [[u8; 4]; 2] = [[40, 120, 200, 255], [230, 180, 60, 255]];

/// RGBA pixels drawn at a device position.
#[derive(Clone)]
struct Placed {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    handle: image::Handle,
}

/// Draws its images with nearest filtering at bounds `(device + NUDGE) / k`
/// and `size / k`, each clipped to its own rect, as the `Plot` widget
/// places its cells.
struct Images {
    scale: f32,
    placed: Vec<Placed>,
}

impl Placed {
    /// `rgba`, `width` × `height`, at device (`x`, `y`) from the widget's
    /// device origin.
    fn new(x: u32, y: u32, width: u32, height: u32, rgba: Vec<u8>) -> Self {
        let handle = image::Handle::from_rgba(width, height, rgba.clone());
        Placed {
            x,
            y,
            width,
            height,
            rgba,
            handle,
        }
    }

    /// The image's bytes.
    fn bytes(&self) -> usize {
        self.rgba.len()
    }
}

impl Meta for Images {}

impl Widget<(), Theme, iced::Renderer> for Images {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(&mut self, tree: &mut Tree, _renderer: &iced::Renderer, limits: &layout::Limits) {
        tree.size = layout::atomic(limits, Length::Fill, Length::Fill);
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let k = self.scale;
        let origin = layout.bounds().position();
        let (left, top) = ((origin.x * k).round(), (origin.y * k).round());
        for placed in &self.placed {
            let bounds = Rectangle {
                x: (left + placed.x as f32 + NUDGE) / k,
                y: (top + placed.y as f32 + NUDGE) / k,
                width: placed.width as f32 / k,
                height: placed.height as f32 / k,
            };
            let image = image::Image::new(placed.handle.clone())
                .filter_method(image::FilterMethod::Nearest);
            renderer.draw_image(image, bounds, bounds);
        }
    }
}

/// A checkerboard of 1-px cells inside a 1-px red border.
fn checkerboard(width: u32, height: u32) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let border = x == 0 || y == 0 || x == width - 1 || y == height - 1;
            let pixel = if border {
                RED
            } else {
                CHECKS[((x + y) % 2) as usize]
            };
            rgba.extend_from_slice(&pixel);
        }
    }
    rgba
}

/// One color everywhere.
fn solid(width: u32, height: u32, color: [u8; 4]) -> Vec<u8> {
    color.repeat((width * height) as usize)
}

/// The `width` × `height` pixels of `shot` from (`x`, `y`).
fn crop(shot: &Shot, x: u32, y: u32, width: u32, height: u32) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for row in y..y + height {
        let start = ((row * shot.width + x) * 4) as usize;
        rgba.extend_from_slice(&shot.rgba[start..start + (width * 4) as usize]);
    }
    rgba
}

/// The largest channel difference between `placed` and where `shot`
/// shows it.
fn shown_diff(shot: &Shot, placed: &Placed) -> u8 {
    let shown = crop(shot, placed.x, placed.y, placed.width, placed.height);
    let a = pixels::Image::new(placed.width, placed.height, &shown)
        .expect("invariant: the crop holds the image's pixels");
    let b = pixels::Image::new(placed.width, placed.height, &placed.rgba)
        .expect("invariant: an image holds width × height pixels");
    pixels::Diff::between(&a, &b, 1)
        .expect("invariant: both sides have the image's size")
        .max()
}

/// The largest channel difference between `pixel` and `color`.
fn channel_diff(pixel: [u8; 4], color: [u8; 4]) -> u8 {
    pixel
        .iter()
        .zip(color)
        .map(|(&a, b)| a.abs_diff(b))
        .max()
        .unwrap_or(0)
}

/// The pixel bound of `backend`: 1/255 on tiny-skia, 2/255 on wgpu.
fn bound(backend: &str) -> u8 {
    if backend == "wgpu" { 2 } else { 1 }
}

/// Device columns of row `y` in `x0..x1` that show red.
fn red_columns(shot: &Shot, y: u32, x0: u32, x1: u32) -> Vec<u32> {
    (x0..x1)
        .filter(|&x| channel_diff(shot.pixel(x, y), RED) <= 2)
        .collect()
}

#[test]
fn images_land_on_device_pixels_at_every_factor() {
    // The window, in device px; four 300 × 200 images at odd offsets.
    let (width, height) = (764, 506);
    let origins = [(125, 75), (438, 75), (125, 286), (438, 286)];
    for k in FACTORS {
        let placed: Vec<Placed> = origins
            .map(|(x, y)| Placed::new(x, y, 300, 200, checkerboard(300, 200)))
            .into();
        let images = Images {
            scale: k,
            placed: placed.clone(),
        };
        let size = Size::new(width as f32 / k, height as f32 / k);
        let Some(mut harness) = Harness::new(images, size, k) else {
            return;
        };
        let backend = harness.backend();
        let limit = bound(&backend);
        let shot = harness.frame(headless::epoch());
        assert_eq!((shot.width, shot.height), (width, height), "k = {k}");
        let mut worst = 0;
        for image in &placed {
            let (x, y) = (image.x, image.y);
            // The red border lies exactly on the image's first and last
            // device columns, with background just outside every edge.
            let found = red_columns(&shot, y + 100, x - 2, x + 302);
            assert_eq!(
                found,
                [x, x + 299],
                "{backend}, k = {k}: the image at device ({x}, {y}) shows red at {found:?}"
            );
            let outside = [
                (x - 1, y + 100),
                (x + 300, y + 100),
                (x + 150, y - 1),
                (x + 150, y + 200),
            ];
            for (px, py) in outside {
                let off = channel_diff(shot.pixel(px, py), WHITE);
                assert!(
                    off <= limit,
                    "{backend}, k = {k}: ({px}, {py}) is not background"
                );
            }
            worst = worst.max(shown_diff(&shot, image));
        }
        println!(
            "placement {backend} k = {k}: max {worst}/255 ({})",
            headless::features()
        );
        assert!(worst <= limit, "{backend}, k = {k}: max {worst}/255");
    }
}

#[test]
fn image_bounds_for_device_125_at_125_percent_are_100_2() {
    assert_eq!((125.0 + NUDGE) / 1.25, 100.2);
}

#[test]
fn a_sub_2mib_image_shows_in_its_first_frame() {
    let placed = Placed::new(0, 0, 724, 724, solid(724, 724, [30, 160, 90, 255]));
    assert_eq!(placed.bytes(), 2_096_704);
    assert!(placed.bytes() < BAND_BYTES);
    let images = Images {
        scale: 1.0,
        placed: vec![placed.clone()],
    };
    let Some(mut harness) = Harness::new(images, Size::new(724.0, 724.0), 1.0) else {
        return;
    };
    let backend = harness.backend();
    let shot = harness.frame(headless::epoch());
    let shown = shown_diff(&shot, &placed);
    assert!(
        shown <= bound(&backend),
        "{backend}: max {shown}/255 in the first frame"
    );
}

#[test]
fn a_larger_image_needs_a_tick() {
    // The section's 1000 × 600, and an image of exactly BAND_BYTES, which
    // Phase 18 bands too.
    for (width, height, bytes) in [(1000, 600, 2_400_000), (1024, 512, BAND_BYTES)] {
        let rgba = solid(width, height, [200, 40, 120, 255]);
        let placed = Placed::new(0, 0, width, height, rgba);
        assert_eq!(placed.bytes(), bytes);
        assert!(placed.bytes() >= BAND_BYTES);
        let images = Images {
            scale: 1.0,
            placed: vec![placed.clone()],
        };
        let size = Size::new(width as f32, height as f32);
        let Some(mut harness) = Harness::new(images, size, 1.0) else {
            return;
        };
        let backend = harness.backend();
        let limit = bound(&backend);
        let t = headless::epoch();
        let first = harness.frame(t);
        if backend == "tiny-skia" {
            assert!(shown_diff(&first, &placed) <= limit);
            continue;
        }
        assert!(
            first.rgba.chunks(4).all(|p| p == WHITE),
            "{backend}: {bytes} B drawn in its first frame"
        );
        // The worker uploads on a thread of its own: tick and draw until
        // it is done.
        let frames = (1..=200).find(|&frame| {
            std::thread::sleep(Duration::from_millis(10));
            harness.tick();
            let shot = harness.frame(t + Duration::from_millis(16 * frame));
            shown_diff(&shot, &placed) <= limit
        });
        let frames = frames.unwrap_or_else(|| panic!("{backend}: {bytes} B never showed"));
        println!("{backend}: {bytes} B shows after {frames} tick(s) and frame(s)");
    }
}

#[test]
fn every_device_offset_lands_exactly_at_every_factor() {
    // 4 × 4 images of distinct texels at device offsets 1, 8, …, 771 on
    // both axes: every residue the factors repeat with, at magnitudes up to
    // the window's. Biases of 0 (tiny-skia, a pixel early at some offsets
    // at 1.25 and 1.5) and 0.5 (wgpu, a pixel late at all) fail here.
    let texels: Vec<u8> = (0..4u8)
        .flat_map(|j| (0..4u8).flat_map(move |i| [50 + 60 * i, 50 + 60 * j, 0, 255]))
        .collect();
    let offsets: Vec<u32> = (0..=110).map(|i| 7 * i + 1).collect();
    let side = 7 * 110 + 1 + 4 + 4;
    for k in FACTORS {
        let placed: Vec<Placed> = offsets
            .iter()
            .map(|&d| Placed::new(d, d, 4, 4, texels.clone()))
            .collect();
        let images = Images {
            scale: k,
            placed: placed.clone(),
        };
        let size = Size::new(side as f32 / k, side as f32 / k);
        let Some(mut harness) = Harness::new(images, size, k) else {
            return;
        };
        let backend = harness.backend();
        let shot = harness.frame(headless::epoch());
        let off: Vec<u32> = placed
            .iter()
            .filter(|image| shown_diff(&shot, image) > bound(&backend))
            .map(|image| image.x)
            .collect();
        assert!(
            off.is_empty(),
            "{backend}, k = {k}: offsets {off:?} misplaced"
        );
    }
}
