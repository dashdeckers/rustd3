//! [`Record`]: the raw JSONL lines of one run, one record per line, every
//! time in ms since the process's first line.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// One line of a run's raw file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Record {
    /// The run's header: what ran where (the file's first line).
    Run(Run),
    /// A `RedrawRequested` pass: the probe's `Frame` joined with the
    /// driver's timing of the child's update and draw.
    Frame {
        /// The widget's frame counter.
        frame: u64,
        /// The `RedrawRequested` instant.
        at_ms: f64,
        /// The newest generation planned.
        generation: u64,
        /// Slots with an image.
        images: u32,
        /// Slots without one.
        missing: u32,
        /// Slots showing outdated content.
        previews: u32,
        /// Bytes of handles drawn for the first time.
        uploaded: u64,
        /// The child's update for this `RedrawRequested`, inline renders
        /// included.
        update_ms: f64,
        /// The child's draw after it, if this pass was drawn.
        draw_ms: Option<f64>,
    },
    /// One injected event.
    Input {
        /// Its kind (`move`, `press`, `wheel_lines`, `append`, …).
        event: String,
        /// When the replay scheduled it.
        scheduled_ms: f64,
        /// The `RedrawRequested` it was injected in.
        injected_ms: f64,
        /// The frame (widget counter) whose `Frame` first shows its effect.
        frame: u64,
        /// The child's update for it (0 for cues the app applies).
        update_ms: f64,
    },
    /// A UI-thread stage of a refresh.
    Stage {
        /// The frame.
        frame: u64,
        /// `resolve` or `list`.
        stage: String,
        /// Its duration.
        ms: f64,
    },
    /// One rendered cell or tile.
    Raster {
        /// The frame it was reported in.
        frame: u64,
        /// The generation it rendered.
        generation: u64,
        /// The cell's index, for a cell outside the plot areas.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cell: Option<u32>,
        /// The plot area, column and row of a tile.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tile: Option<[i32; 3]>,
        /// Device px.
        pixels: u32,
        /// Work items.
        work: u64,
        /// Planning to start.
        queued_ms: f64,
        /// The render.
        ms: f64,
        /// On the UI thread.
        inline: bool,
    },
    /// Process counters at a moment.
    Sys {
        /// When.
        at_ms: f64,
        /// Why: `start`, `first_input`, `window_end`, `sample`, `done`.
        mark: String,
        /// utime + stime of `/proc/self/stat`, in clock ticks.
        cpu_ticks: Option<u64>,
        /// `VmRSS`, kB.
        rss_kb: Option<u64>,
        /// Whole-GPU memory used, MiB (`nvidia-smi`), when sampled.
        gpu_mb: Option<f64>,
    },
    /// A headless readback: the harness's own cost, not the widget's.
    Harness {
        /// When the frame began.
        at_ms: f64,
        /// The screenshot's duration.
        ms: f64,
    },
    /// The run's end.
    End {
        /// When `Done` was published.
        at_ms: f64,
        /// False when the settle timed out before every slot was sharp.
        settled: bool,
    },
}

/// What ran, where.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Run {
    /// The scenario's name.
    pub scenario: String,
    /// The workload's name.
    pub workload: String,
    /// The scale factor asked for.
    pub factor: f32,
    /// The run, from 1.
    pub run: u32,
    /// The renderer that ran (`wgpu`, `tiny-skia`).
    pub backend: String,
    /// Headless or windowed.
    pub headless: bool,
    /// The display's refresh rate (headless: the virtual one).
    pub refresh_hz: f64,
    /// The plot's logical size asked for, which the replay was generated
    /// for.
    pub window: [f32; 2],
    /// The window's device size obtained (windowed: from its events).
    pub device: [f32; 2],
    /// The plot widget's logical size the driver placed the replay's
    /// positions in (before the scenario's own first resize).
    pub widget: Option<[f32; 2]>,
    /// The window's logical size its events reported before the
    /// scenario's own first resize (windowed only).
    pub reported: Option<[f32; 2]>,
    /// The window's own factor, from its events (windowed only).
    pub window_factor: Option<f32>,
    /// The scenario's duration, ms.
    pub duration_ms: f64,
    /// The data size (1 = full).
    pub data_scale: f64,
    /// CPU clock ticks per second (`getconf CLK_TCK`).
    pub clk_tck: u64,
}

/// Why a raw file cannot be read or written.
#[derive(Debug)]
pub enum Error {
    /// The file.
    Io {
        /// Its path.
        path: String,
        /// What failed.
        source: io::Error,
    },
    /// A line that is not a record.
    Line {
        /// Its path.
        path: String,
        /// 1-based.
        line: usize,
        /// What serde said.
        source: serde_json::Error,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io { path, source } => write!(f, "{path}: {source}"),
            Error::Line { path, line, source } => write!(f, "{path}:{line}: {source}"),
        }
    }
}

impl std::error::Error for Error {}

/// Writes `records` to `path` as JSON Lines.
pub fn write(path: &Path, records: &[Record]) -> Result<(), Error> {
    let mut text = String::new();
    for record in records {
        text.push_str(&serde_json::to_string(record).expect("invariant: records serialize"));
        text.push('\n');
    }
    fs::write(path, text).map_err(|source| Error::Io {
        path: path.display().to_string(),
        source,
    })
}

/// Reads a raw file.
pub fn read(path: &Path) -> Result<Vec<Record>, Error> {
    let text = fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.display().to_string(),
        source,
    })?;
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            serde_json::from_str(line).map_err(|source| Error::Line {
                path: path.display().to_string(),
                line: index + 1,
                source,
            })
        })
        .collect()
}
