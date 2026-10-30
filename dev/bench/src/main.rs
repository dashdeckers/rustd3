//! rustd3-bench: the benchmark of rustd3. Its one entry point is a level
//! (`rustd3-bench [--quick | --default | --full]`), which takes a CPU
//! reference and times the plotting library alone (the figure profile)
//! and, with the `widget` feature (the default), drives the real
//! `Plot` widget through a wrapper that replays time-stamped input, in a
//! window or headlessly, one cold process per run, and writes raw records,
//! CSV summaries and a report that evaluates the rule registered before
//! any run. Without the feature it builds no iced, winit or wgpu
//! and runs the library's part alone. `--help` prints what the build runs.

// Without the widget, its modules are out and some of what the rest
// offers them goes unused; the default build, which has them, checks
// for dead code.
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
mod stats;
mod system;
mod workload;

use std::process::ExitCode;

use crate::cli::Command;

fn main() -> ExitCode {
    // Cold start counts from here.
    #[cfg(feature = "widget")]
    let epoch = std::time::Instant::now();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = match Command::parse(&args) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("rustd3-bench: {error}");
            return ExitCode::from(2);
        }
    };
    let result = match command {
        #[cfg(feature = "widget")]
        Command::Level(request) => commands::level(&request),
        #[cfg(not(feature = "widget"))]
        Command::Level(request) => std::env::current_exe()
            .map_err(Into::into)
            .and_then(|exe| request.run(&exe, None)),
        Command::Help => {
            print!("{}", cli::HELP);
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
        Command::Widget(command) => commands::execute(*command, &args, epoch),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rustd3-bench: {error}");
            ExitCode::FAILURE
        }
    }
}
