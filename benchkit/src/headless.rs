//! The headless frame loop: the app's interface on iced's headless
//! renderer, virtual frames paced at the refresh rate, pixels read back as
//! the windowed runtime would present them.
//!
//! Per frame: `renderer.tick()` (on wgpu it drains the image upload
//! worker, which only winit's `Action::Tick` calls otherwise); `Waken` if
//! the waker fired; `RedrawRequested(now)`, and while that publishes
//! messages, apply them, rebuild and repeat with the same `now` (at most
//! three passes, as `winit/src/lib.rs:788-830`); draw; screenshot.
//!
//! On the software renderer this is the software profile's offscreen
//! frame: the screenshot is iced composing the frame into a
//! pixmap. Before the first frame the loop times the readback of an empty
//! frame, the screenshot's fixed cost; with `--frames` it writes every
//! n-th frame as a PNG on a thread of its own.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};
use std::{fs, thread};

use iced::advanced::renderer::{self, Headless as _};
use iced::advanced::{Renderer as _, shell};
use iced::{Color, Event, Size, mouse, window};
use iced_runtime::user_interface::{self, UserInterface};

use crate::app::{App, Effect, Message};
use crate::cli::widget::Readback;
use crate::record::Record;
use crate::stats::percentile;
use crate::system;

/// Why the headless loop cannot run.
#[derive(Debug)]
pub enum Error {
    /// iced found no renderer for the backend.
    Backend(String),
}

/// How the loop runs: the renderer, the frame's scale and rate, and what it
/// reads back and writes.
#[derive(Debug, Clone)]
pub struct Settings<'a> {
    /// The backend (`None`: iced's choice).
    pub backend: Option<&'a str>,
    /// The scale factor.
    pub factor: f32,
    /// The virtual frames' rate.
    pub refresh_hz: f64,
    /// When pixels are read back.
    pub readback: Readback,
    /// Where composed frames are written as PNGs, if anywhere.
    pub frames: Option<Frames>,
}

/// Composed frames written as PNGs (`--frames DIR --every N`).
#[derive(Debug, Clone, PartialEq)]
pub struct Frames {
    /// The run's directory of PNGs.
    pub dir: PathBuf,
    /// Every n-th frame, from the first.
    pub every: u32,
}

/// A frame on its way to its PNG.
struct Shot {
    path: PathBuf,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

/// Empty frames read back to time the screenshot's fixed cost.
const EMPTY_READBACKS: usize = 5;

/// Frames on a grid of `period`, as vsync paces them: a frame that misses
/// its slot waits for the next one.
#[derive(Debug)]
struct Pacer {
    start: Instant,
    period: Duration,
    next: u64,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Backend(backend) => write!(f, "no headless renderer for backend {backend}"),
        }
    }
}

impl std::error::Error for Error {}

impl Pacer {
    fn new(refresh_hz: f64) -> Pacer {
        Pacer {
            start: Instant::now(),
            period: Duration::from_secs_f64(1.0 / refresh_hz),
            next: 0,
        }
    }

    /// Sleeps to the next free slot and returns the instant.
    fn wait(&mut self) -> Instant {
        let now = Instant::now();
        let due = |k: u64| self.start + self.period.mul_f64(k as f64);
        let elapsed = now.saturating_duration_since(self.start).as_secs_f64();
        let slot = (elapsed / self.period.as_secs_f64()).ceil() as u64;
        self.next = self.next.max(slot);
        let at = due(self.next);
        if let Some(sleep) = at.checked_duration_since(now) {
            thread::sleep(sleep);
        }
        self.next += 1;
        Instant::now()
    }
}

/// Runs `app` as `settings` say until its driver is done; returns the
/// renderer that ran.
pub fn run(app: &mut App, settings: &Settings<'_>) -> Result<String, Error> {
    let Settings {
        backend,
        factor,
        refresh_hz,
        readback,
        ..
    } = *settings;
    let renderer = iced::futures::executor::block_on(iced::Renderer::new(
        renderer::Settings::default(),
        backend,
    ));
    let mut renderer =
        renderer.ok_or_else(|| Error::Backend(backend.unwrap_or("auto").to_owned()))?;
    renderer.hint(renderer::Scale {
        window: factor,
        application: 1.0,
    });
    let name = renderer.name();
    let woken = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&woken);
    let waker = shell::Waker::new(move || flag.store(true, Ordering::SeqCst));
    let records = app.records();
    let epoch = app.epoch();
    if readback == Readback::Every {
        let empty: Vec<f64> = (0..EMPTY_READBACKS)
            .map(|_| {
                let begin = Instant::now();
                let _ = screenshot(&mut renderer, app.size(), factor);
                begin.elapsed().as_secs_f64() * 1e3
            })
            .collect();
        if let Some(ms) = percentile(&empty, 50.0) {
            let _ = records.send(Record::Readback { ms });
        }
    }
    let (shots, writer) = match &settings.frames {
        Some(frames) => {
            let (send, receive) = mpsc::sync_channel(4);
            (Some((send, frames)), Some(write(receive)))
        }
        None => (None, None),
    };
    let mut composed = 0_u64;
    let mut pacer = Pacer::new(refresh_hz);
    let mut bus = shell::Bus::new();
    let mut cache = user_interface::Cache::default();
    // A frame whose RedrawRequested published messages continues, after
    // they are applied, with the same instant and the next pass.
    let mut carry: Option<(Instant, u8)> = None;
    loop {
        let size = app.size();
        let mut ui = UserInterface::build(app.view(), size, cache, &mut renderer);
        let messages: Vec<Message> = loop {
            let (now, pass) = match carry.take() {
                Some(carried) => carried,
                None => {
                    let now = pacer.wait();
                    renderer.tick();
                    if woken.swap(false, Ordering::SeqCst) {
                        let _ = ui.update(
                            &window::Headless,
                            &waker,
                            &[Event::Waken],
                            mouse::Cursor::Unavailable,
                            &mut renderer,
                            &mut bus,
                        );
                    }
                    (now, 0)
                }
            };
            let _ = ui.update(
                &window::Headless,
                &waker,
                &[Event::Window(window::Event::RedrawRequested(now))],
                mouse::Cursor::Unavailable,
                &mut renderer,
                &mut bus,
            );
            let messages: Vec<Message> = bus.drain().map(|(message, _)| message).collect();
            if !messages.is_empty() {
                if pass < 2 {
                    carry = Some((now, pass + 1));
                }
                break messages;
            }
            draw(&mut ui, &mut renderer);
            if readback == Readback::Every {
                let begin = Instant::now();
                let rgba = screenshot(&mut renderer, size, factor);
                let _ = records.send(Record::Harness {
                    at_ms: system::since(epoch, now),
                    ms: begin.elapsed().as_secs_f64() * 1e3,
                });
                if let Some((send, frames)) = &shots
                    && composed.is_multiple_of(u64::from(frames.every))
                {
                    let [width, height] = physical(size, factor);
                    let _ = send.send(Shot {
                        path: frames.dir.join(format!("{composed:05}.png")),
                        width,
                        height,
                        rgba,
                    });
                }
                composed += 1;
            }
        };
        cache = ui.into_cache();
        let mut exit = false;
        for message in messages {
            let effect = match message {
                Message::Cue(cue) => app.apply(cue),
                Message::Window(..) => Effect::None,
            };
            match effect {
                Effect::None | Effect::Resize(_) => {}
                Effect::Exit => exit = true,
            }
        }
        if exit {
            if readback == Readback::Last {
                let size = app.size();
                let mut ui = UserInterface::build(app.view(), size, cache, &mut renderer);
                draw(&mut ui, &mut renderer);
                let rgba = screenshot(&mut renderer, size, factor);
                if let Some((send, frames)) = &shots {
                    let [width, height] = physical(size, factor);
                    let _ = send.send(Shot {
                        path: frames.dir.join("last.png"),
                        width,
                        height,
                        rgba,
                    });
                }
            }
            drop(shots);
            if let Some(writer) = writer {
                let _ = writer.join();
            }
            return Ok(name);
        }
    }
}

/// Draws the interface on the light theme.
fn draw(
    ui: &mut UserInterface<'_, Message, iced::Theme, iced::Renderer>,
    renderer: &mut iced::Renderer,
) {
    ui.draw(
        renderer,
        &iced::Theme::Light,
        &renderer::Style {
            text_color: Color::BLACK,
        },
        mouse::Cursor::Unavailable,
    );
}

/// Reads the frame back at its device size.
fn screenshot(renderer: &mut iced::Renderer, size: Size, factor: f32) -> Vec<u8> {
    let [width, height] = physical(size, factor);
    renderer.screenshot(Size::new(width, height), factor, Color::WHITE)
}

/// A logical size's device px at `factor`.
fn physical(size: Size, factor: f32) -> [u32; 2] {
    [
        (size.width * factor).round() as u32,
        (size.height * factor).round() as u32,
    ]
}

/// Writes each frame it receives as a PNG, on a thread of its own so the
/// encoding stays off the frame loop (it still takes a core); the first
/// failure is printed and the rest dropped.
fn write(shots: mpsc::Receiver<Shot>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut failed = false;
        for shot in shots {
            if failed {
                continue;
            }
            let image = rustd3_figure::raster::Image {
                width: shot.width,
                height: shot.height,
                rgba: shot.rgba,
            };
            let written = shot
                .path
                .parent()
                .map_or(Ok(()), fs::create_dir_all)
                .map_err(|e| e.to_string())
                .and_then(|()| {
                    rustd3_figure::raster::png::encode(&image).map_err(|e| e.to_string())
                })
                .and_then(|png| fs::write(&shot.path, png).map_err(|e| e.to_string()));
            if let Err(error) = written {
                eprintln!("benchkit: {}: {error}", shot.path.display());
                failed = true;
            }
        }
    })
}
