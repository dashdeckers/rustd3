//! The headless pixel harness the iced tests share: build a
//! `UserInterface`, update it with `RedrawRequested`, draw, and read the
//! pixels back with `Headless::screenshot`, ticking the renderer once per
//! virtual frame. The backend is `ICED_TEST_BACKEND`'s; with it set, iced
//! does not fall back to the other one.
//!
//! `iced_test`'s `matches_image` and `matches_hash` are not used: they
//! compare exactly and write a missing reference instead of failing.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use iced::advanced::Renderer as _;
use iced::advanced::graphics::color::GAMMA_CORRECTION;
use iced::advanced::renderer::{self, CRISP, Headless as _};
use iced_core::time::Instant;
use iced_core::{Color, Event, Size, event, mouse, shell, window};
use iced_runtime::user_interface::{self, UserInterface};
use rustd3_testkit::pixels;

/// A user interface on a headless renderer at a fixed size and scale.
pub struct Harness<'a, Message> {
    ui: UserInterface<'a, Message, iced::Theme, iced::Renderer>,
    renderer: iced::Renderer,
    /// Logical size.
    size: Size,
    scale: f32,
    cursor: mouse::Cursor,
    bus: shell::Bus<Message>,
    wakes: Arc<AtomicUsize>,
    redraw: window::RedrawRequest,
}

/// A screenshot: RGBA8 at physical size, row-major.
pub struct Shot {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl<'a, Message: 'a> Harness<'a, Message> {
    /// `root` laid out at `size` logical pixels and drawn at `scale` on
    /// `ICED_TEST_BACKEND`'s renderer. None when that is `wgpu`, it finds
    /// no device and `RUSTD3_ALLOW_NO_GPU=1`; panics otherwise.
    pub fn new(root: impl iced::Widget<Message> + 'a, size: Size, scale: f32) -> Option<Self> {
        let backend = std::env::var("ICED_TEST_BACKEND").ok();
        Self::with_backend(root, size, scale, backend.as_deref())
    }

    /// As [`Harness::new`] on the named backend (`tiny-skia` or `wgpu`), or
    /// iced's first that works for `None`.
    pub fn with_backend(
        root: impl iced::Widget<Message> + 'a,
        size: Size,
        scale: f32,
        backend: Option<&str>,
    ) -> Option<Self> {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            renderer::Settings::default(),
            backend,
        ));
        let Some(mut renderer) = renderer else {
            let allowed = std::env::var("RUSTD3_ALLOW_NO_GPU").is_ok_and(|v| v == "1");
            if backend == Some("wgpu") && allowed {
                eprintln!("skipped: wgpu found no device (RUSTD3_ALLOW_NO_GPU=1)");
                return None;
            }
            panic!("no headless renderer for backend {backend:?}");
        };
        // wgpu reports the factor from the hint; tiny-skia ignores hints.
        renderer.hint(renderer::Scale {
            window: scale,
            application: 1.0,
        });
        let ui = UserInterface::build(root, size, user_interface::Cache::default(), &mut renderer);
        Some(Harness {
            ui,
            renderer,
            size,
            scale,
            cursor: mouse::Cursor::Unavailable,
            bus: shell::Bus::new(),
            wakes: Arc::new(AtomicUsize::new(0)),
            redraw: window::RedrawRequest::Wait,
        })
    }

    /// Places the cursor at `at` (logical) for later events and frames;
    /// sends no event.
    pub fn point_at(&mut self, at: iced::Point) {
        self.cursor = mouse::Cursor::Available(at);
    }

    /// Updates the interface with `events`, in one update.
    pub fn events(&mut self, events: &[Event]) -> Vec<event::Status> {
        let waker = self.waker();
        let (state, statuses) = self.ui.update(
            &window::Headless,
            &waker,
            events,
            self.cursor,
            &mut self.renderer,
            &mut self.bus,
        );
        self.redraw = match state {
            user_interface::State::Updated { redraw_request, .. } => redraw_request,
            user_interface::State::Outdated => window::RedrawRequest::Wait,
        };
        statuses
    }

    /// `renderer.tick()`: on wgpu, drains the image upload worker.
    pub fn tick(&mut self) {
        self.renderer.tick();
    }

    /// One virtual frame: tick, update with `RedrawRequested(now)`, draw
    /// and take a screenshot on a white background.
    pub fn frame(&mut self, now: Instant) -> Shot {
        self.tick();
        let _ = self.events(&[Event::Window(window::Event::RedrawRequested(now))]);
        self.ui.draw(
            &mut self.renderer,
            &iced::Theme::Light,
            &renderer::Style {
                text_color: Color::BLACK,
            },
            self.cursor,
        );
        let (width, height) = (
            (self.size.width * self.scale).round() as u32,
            (self.size.height * self.scale).round() as u32,
        );
        let rgba = self
            .renderer
            .screenshot(Size::new(width, height), self.scale, Color::WHITE);
        Shot {
            width,
            height,
            rgba,
        }
    }

    /// The redraw request of the last update: `Wait` after one that left
    /// the interface outdated.
    pub fn redraw_requested(&self) -> window::RedrawRequest {
        self.redraw
    }

    /// A waker that counts its wakes, as the updates receive it.
    pub fn waker(&self) -> shell::Waker {
        let wakes = Arc::clone(&self.wakes);
        shell::Waker::new(move || {
            wakes.fetch_add(1, Ordering::SeqCst);
        })
    }

    /// How many times a waker of this harness was woken.
    pub fn wakes(&self) -> usize {
        self.wakes.load(Ordering::SeqCst)
    }

    /// The messages published since the last call.
    pub fn messages(&mut self) -> Vec<Message> {
        self.bus.drain().map(|(message, _)| message).collect()
    }

    /// The renderer's name: `tiny-skia` or `wgpu`.
    pub fn backend(&self) -> String {
        self.renderer.name()
    }
}

impl Shot {
    /// The screenshot as testkit's image.
    pub fn image(&self) -> pixels::Image<'_> {
        pixels::Image::new(self.width, self.height, &self.rgba)
            .expect("invariant: a screenshot holds width × height pixels")
    }

    /// The pixel at (`x`, `y`).
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let i = (y as usize * self.width as usize + x as usize) * 4;
        [
            self.rgba[i],
            self.rgba[i + 1],
            self.rgba[i + 2],
            self.rgba[i + 3],
        ]
    }
}

/// The state of iced's `crisp` and `web-colors` features in this build,
/// which every recorded pixel number states.
pub fn features() -> String {
    format!("crisp {CRISP}, web-colors {}", !GAMMA_CORRECTION)
}

/// One real instant tests offset their frames from.
#[expect(
    clippy::disallowed_methods,
    reason = "frames need one instant to build later ones from"
)]
pub fn epoch() -> Instant {
    Instant::now()
}
