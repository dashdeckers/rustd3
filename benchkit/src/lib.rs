//! benchkit: the machinery of rustd3's benchmark examples, which are thin
//! `main`s over [`run`].
//!
//! `bench_all [--quick | --default | --full]` runs a level. A level takes a
//! CPU reference and times the plotting library alone (the figure profile,
//! `bench_figure`). With the `widget` feature (the default), it also drives
//! the real `Plot` widget through a wrapper that replays time-stamped
//! input, offscreen (`bench_software`) and in windows (`bench_gestures`),
//! with one cold process per run. It writes raw records, CSV summaries and
//! a report that evaluates the rule registered before any run. Without the
//! feature, benchkit builds no iced, winit or wgpu and offers only the
//! library's part. Each example's `--help` prints what it runs.
//!
//! [`gallery`] holds the example figures the benchmark draws and the
//! examples show: the trace, the wheel figure and the chart presets.

// Without the `widget` feature its modules are compiled out, and some
// items that only serve them go unused. The default build, which has
// them, still checks for dead code.
#![cfg_attr(
    not(feature = "widget"),
    allow(
        dead_code,
        reason = "the widget's modules use it, and the default build checks it"
    )
)]

#[cfg(feature = "widget")]
mod also;
#[cfg(feature = "widget")]
mod app;
mod archive;
mod cli;
#[cfg(feature = "widget")]
mod commands;
mod compare;
mod display;
#[cfg(feature = "widget")]
mod driver;
mod experiment;
mod figure;
pub mod gallery;
#[cfg(feature = "widget")]
mod headless;
mod level;
mod load;
#[cfg(feature = "widget")]
mod metrics;
#[cfg(feature = "widget")]
mod plan;
mod privacy;
#[cfg(feature = "widget")]
mod raster;
mod record;
#[cfg(feature = "widget")]
mod recorder;
mod reference;
#[cfg(feature = "widget")]
mod replay;
#[cfg(feature = "widget")]
mod report;
#[cfg(feature = "widget")]
mod repro;
mod results;
#[cfg(feature = "widget")]
mod rule;
#[cfg(feature = "widget")]
mod scenario;
#[cfg(feature = "widget")]
mod sibling;
mod stats;
mod system;
mod workload;

use std::process::ExitCode;

use crate::cli::Command;

/// The benchmark examples, each a `main` that calls [`run`] with its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Example {
    /// `bench_figure`: the CPU reference and the figure profile, the
    /// plotting library alone, headless.
    Figure,
    /// `bench_gestures`: a plan of the `Plot` widget's cells in windows
    /// (the native backend, the software renderer, X11 through XWayland).
    #[cfg(feature = "widget")]
    Gestures,
    /// `bench_software`: the software profile, the widget composed
    /// offscreen on the software renderer, every frame timed by part.
    #[cfg(feature = "widget")]
    Software,
    /// `bench_all`: a level, which runs the other examples as child
    /// processes.
    #[cfg(feature = "widget")]
    All,
    /// `bench_compare`: two results side by side.
    Compare,
    /// `bench_report`: a level's report rebuilt from its raw records.
    Report,
}

impl Example {
    /// The example's name, which is also its binary's file stem.
    pub fn name(self) -> &'static str {
        match self {
            Example::Figure => "bench_figure",
            #[cfg(feature = "widget")]
            Example::Gestures => "bench_gestures",
            #[cfg(feature = "widget")]
            Example::Software => "bench_software",
            #[cfg(feature = "widget")]
            Example::All => "bench_all",
            Example::Compare => "bench_compare",
            Example::Report => "bench_report",
        }
    }
}

/// Runs `example` with the process's arguments and returns the exit code
/// for its `main`.
pub fn run(example: Example) -> ExitCode {
    // Cold start counts from here.
    #[cfg(feature = "widget")]
    let epoch = std::time::Instant::now();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = match Command::parse(example, &args) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("{}: {error}", example.name());
            return ExitCode::from(2);
        }
    };
    let result = match command {
        #[cfg(feature = "widget")]
        Command::Level(request) if example == Example::All => commands::level(&request),
        Command::Level(request) => std::env::current_exe()
            .map_err(Into::into)
            .and_then(|exe| request.run(&exe, None)),
        Command::Help => {
            print!("{}", cli::help(example));
            Ok(())
        }
        Command::FigureChild { level, subject } => figure::child(level, &subject),
        #[cfg(feature = "widget")]
        Command::Report { path } => commands::report(&path),
        #[cfg(not(feature = "widget"))]
        Command::Report { path } => load::results(&path).map(|results| {
            print!("{}", results.markdown());
        }),
        Command::Compare { paths: [a, b] } => load::results(&a).and_then(|ra| {
            let rb = load::results(&b)?;
            let names = [a.display().to_string(), b.display().to_string()];
            print!("{}", compare::markdown(&ra, &rb, [&names[0], &names[1]]));
            Ok(())
        }),
        #[cfg(feature = "widget")]
        Command::Widget(command) => commands::execute(*command, example, &args, epoch),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}: {error}", example.name());
            ExitCode::FAILURE
        }
    }
}
