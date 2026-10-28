//! [`App`]: the application a child runs, in a window or headlessly. It
//! owns the workload's data, builds the figure in `view` and wraps the
//! `Plot` in a [`Driver`]; it applies the driver's cues (appends, resizes)
//! and on `Done` writes the run's raw file.

use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Instant;

use iced::{Element, Size, Task, window};
use rustd3_iced::Plot;
use rustd3_iced::plot::Probe;
use rustd3_iced::plot::probe::Sample;

use crate::cli::Child;
use crate::driver::Driver;
use crate::record::{self, Record};
use crate::replay::{Cue, Replay};
use crate::scenario::Scenario;
use crate::system::Sampler;
use crate::workload::Data;

/// Everything a child needs to build its app.
#[derive(Debug, Clone)]
pub struct Config {
    /// The cell, run and options.
    pub child: Child,
    /// The input to replay.
    pub replay: Replay,
    /// The process's first instant.
    pub epoch: Instant,
    /// The display's refresh rate.
    pub refresh_hz: f64,
    /// The display's scale factor.
    pub native: f32,
    /// The scenario's duration in this plan, ms.
    pub duration_ms: f64,
    /// The data size.
    pub data_scale: f64,
    /// CPU ticks per second.
    pub clk_tck: u64,
}

/// The child's application state.
pub struct App {
    config: Config,
    data: Data,
    records: mpsc::Sender<Record>,
    received: mpsc::Receiver<Record>,
    sink: mpsc::Sender<Sample>,
    samples: mpsc::Receiver<Sample>,
    /// The plot's logical size now.
    size: Size,
    /// The window, once its events name it.
    window: Option<window::Id>,
    /// The window's own factor, from its events.
    window_factor: Option<f32>,
    /// The window's logical size its events reported, until the
    /// scenario's own first resize.
    reported: Option<Size>,
    /// The plot's logical size the driver placed the replay in.
    widget: Option<Size>,
    /// The scenario resized the window: later sizes are its own.
    resized: bool,
    /// RSS and GPU memory every second during `soak`.
    sampler: Option<Sampler>,
}

/// What the windowed app receives.
#[derive(Debug, Clone)]
pub enum Message {
    /// From the driver.
    Cue(Cue),
    /// A window event.
    Window(window::Id, window::Event),
}

/// What applying a cue asks of the event loop.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Effect {
    /// Nothing.
    None,
    /// Resize the plot to this logical size.
    Resize(Size),
    /// The run is over.
    Exit,
}

impl App {
    /// The app of `config`, its data generated.
    pub fn new(config: Config) -> App {
        let data = config.child.cell.workload.data(config.data_scale);
        let (records, received) = mpsc::channel();
        let (sink, samples) = mpsc::channel();
        let size = config.child.cell.window();
        let sampler = (config.child.cell.scenario == Scenario::Soak)
            .then(|| Sampler::start(records.clone(), config.epoch));
        App {
            config,
            data,
            records,
            received,
            sink,
            samples,
            size,
            window: None,
            window_factor: None,
            reported: None,
            widget: None,
            resized: false,
            sampler,
        }
    }

    /// The driver around the plot of the workload's figure.
    pub fn view(&self) -> Element<'_, Message> {
        let cell = self.config.child.cell;
        let options = &self.config.child.options;
        let figure = self.data.figure(cell.scenario.needs_recordings());
        let plot = Plot::new(figure)
            .scale_factor(cell.factor)
            .render(options.render)
            .caching(options.caching)
            .probe(Probe::new(clock, self.sink.clone()));
        Driver::new(
            plot,
            &self.config.replay,
            cell.window(),
            self.records.clone(),
            Message::Cue,
            &self.samples,
            self.config.epoch,
        )
        .into()
    }

    /// Applies a cue from the driver.
    pub fn apply(&mut self, cue: Cue) -> Effect {
        match cue {
            Cue::Append(n) => {
                self.data.append(n as usize);
                Effect::None
            }
            Cue::Resize { width, height } => {
                self.size = Size::new(width, height);
                self.resized = true;
                Effect::Resize(self.size)
            }
            Cue::Done { widget } => {
                self.widget = Some(Size::new(widget[0], widget[1]));
                Effect::Exit
            }
        }
    }

    /// The plot's logical size now.
    pub fn size(&self) -> Size {
        self.size
    }

    /// The process's first instant, which record times count from.
    pub fn epoch(&self) -> Instant {
        self.config.epoch
    }

    /// Where records go (the headless loop adds its readbacks).
    pub fn records(&self) -> mpsc::Sender<Record> {
        self.records.clone()
    }

    /// Writes the run's raw file under `raw/`, headed by its `run` record;
    /// `backend` names the renderer that ran. Returns its path.
    pub fn finish(&mut self, backend: &str) -> Result<PathBuf, record::Error> {
        self.sampler = None;
        let cell = self.config.child.cell;
        let options = &self.config.child.options;
        let window = cell.window();
        // The window's factor times the app's (the cell's over the
        // window's) is the cell's: logical px times it are device px.
        let device = self.reported.unwrap_or(window) * cell.factor;
        let pair = |size: Size| [size.width, size.height];
        let mut records = vec![Record::Run(record::Run {
            scenario: cell.scenario.name().to_owned(),
            workload: cell.workload.name().to_owned(),
            factor: cell.factor,
            run: self.config.child.run,
            backend: backend.to_owned(),
            headless: options.headless,
            refresh_hz: self.config.refresh_hz,
            window: pair(window),
            device: pair(device),
            widget: self.widget.map(pair),
            reported: self.reported.map(pair),
            window_factor: self.window_factor,
            duration_ms: self.config.duration_ms,
            data_scale: self.config.data_scale,
            clk_tck: self.config.clk_tck,
        })];
        records.extend(self.received.try_iter());
        let dir = self.config.child.dir.join("raw");
        std::fs::create_dir_all(&dir).map_err(|source| record::Error::Io {
            path: dir.display().to_string(),
            source,
        })?;
        let path = dir.join(format!("{}__r{}.jsonl", cell.key(), self.config.child.run));
        record::write(&path, &records)?;
        Ok(path)
    }

    /// The windowed app's update: cues and the window's events.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Cue(cue) => match self.apply(cue) {
                Effect::None => Task::none(),
                // iced sizes windows in the app's logical px (the plot's):
                // it applies the app's factor and the window's itself.
                Effect::Resize(size) => match self.window {
                    Some(id) => window::resize(id, size),
                    None => Task::none(),
                },
                Effect::Exit => {
                    let backend = self.config.child.options.backend.name();
                    if let Err(error) = self.finish(backend) {
                        eprintln!("rustd3-bench child: {error}");
                    }
                    iced::exit()
                }
            },
            Message::Window(id, event) => {
                match event {
                    window::Event::Opened {
                        size, scale_factor, ..
                    } => {
                        self.window = Some(id);
                        self.window_factor = Some(scale_factor);
                        self.reported = Some(size);
                    }
                    window::Event::Resized(size) if !self.resized => self.reported = Some(size),
                    window::Event::Rescaled(factor) => self.window_factor = Some(factor),
                    window::Event::Closed
                    | window::Event::Resized(_)
                    | window::Event::Moved(_)
                    | window::Event::RedrawRequested(_)
                    | window::Event::CloseRequested
                    | window::Event::Focused
                    | window::Event::Unfocused
                    | window::Event::FileHovered(_)
                    | window::Event::FileDropped(_)
                    | window::Event::FilesHoveredLeft => {}
                }
                Task::none()
            }
        }
    }

    /// The application's scale factor: the cell's factor over the
    /// window's own, so the total is the cell's.
    pub fn scale(&self) -> f32 {
        self.config.child.cell.factor / self.window_factor.unwrap_or(self.config.native)
    }
}

/// Runs `config` in a window; the window factor is assumed to be the
/// display's native factor until the window reports its own.
///
/// iced takes the window's size in the app's logical px and multiplies it
/// by the app's factor (the cell's over the native one) into the window
/// system's, so the plot's own size opens a window of that size × factor ÷
/// native there, and the cell's factor in device px.
///
/// The window is not resizable, so tiling window managers float it at its
/// size, except in `resize`, whose cues resize it (a non-resizable
/// Wayland window keeps its first size).
pub fn windowed(config: Config) -> iced::Result {
    let cell = config.child.cell;
    let settings = window::Settings {
        size: cell.window(),
        resizable: cell.scenario == Scenario::Resize,
        ..window::Settings::default()
    };
    let boot = move || App::new(config.clone());
    iced::application(boot, App::update, App::view)
        .title("rustd3-bench")
        .window(settings)
        .scale_factor(App::scale)
        .subscription(|_| window::events().map(|(id, event)| Message::Window(id, event)))
        .run()
}

/// The probe's clock.
fn clock() -> Instant {
    Instant::now()
}
