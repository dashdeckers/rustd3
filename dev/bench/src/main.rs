//! rustd3-bench: the full-frame benchmark of rustd3's `Plot` widget. It
//! drives the real widget through a wrapper that replays time-stamped
//! input, in a window or headlessly, one cold process per run, and writes
//! raw records, CSV summaries and a report that evaluates the decision
//! rule registered before any run (`--help` prints the procedure).

mod also;
mod app;
mod cli;
mod driver;
mod headless;
mod metrics;
mod plan;
mod record;
mod recorder;
mod replay;
mod report;
mod rule;
mod scenario;
mod stats;
mod system;
mod workload;

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use rustd3_iced::plot::cost;

use crate::app::{App, Config};
use crate::cli::{Child, Command, Options};
use crate::plan::Cell;
use crate::replay::Replay;
use crate::scenario::Scenario;
use crate::system::{Meta, Planned};
use crate::workload::Workload;

fn main() -> ExitCode {
    // Cold start counts from here.
    let epoch = Instant::now();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = match Command::parse(&args) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("rustd3-bench: {error}");
            return ExitCode::from(2);
        }
    };
    let result = match command {
        Command::Help => {
            print!("{}", cli::HELP);
            Ok(())
        }
        Command::List(options) => list(&options),
        Command::Replay { out } => write_replays(&out),
        Command::Record { out } => recorder::record(out).map_err(Into::into),
        Command::Run(options) => run(&options, &args),
        Command::Child(child) => run_child(child, epoch),
        Command::Report { dir } => report::build(&dir).map(|summary| {
            println!("{summary}");
        }),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rustd3-bench: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Prints scenarios, workloads, plans and the cells `options` select.
fn list(options: &Options) -> Result<(), Box<dyn Error>> {
    println!("{}", rule::READINGS);
    println!("Scenarios (full duration):");
    for scenario in Scenario::ALL {
        println!(
            "  {:<12} {:>6.1} s",
            scenario.name(),
            scenario.duration_ms() / 1e3
        );
    }
    println!("Workloads (full size):");
    for workload in Workload::ALL {
        let missing: Vec<&str> = Scenario::ALL
            .into_iter()
            .filter(|s| !workload.supports(*s))
            .map(Scenario::name)
            .collect();
        let na = if missing.is_empty() {
            String::new()
        } else {
            format!("; n/a: {}", missing.join(", "))
        };
        println!("  {:<12} {}{na}", workload.name(), workload.describe(1.0));
    }
    let native = options.native.unwrap_or(2.0);
    println!("Plans (native factor {native}):");
    for plan in plan::Plan::ALL {
        let cells = plan.cells(native, &plan::Selection::default());
        let runs: u32 = cells.iter().filter(|c| c.applies()).map(|c| c.runs).sum();
        println!(
            "  {:<8} {} cells, {} run, {runs} runs, about {:.0} min",
            plan.name(),
            cells.len(),
            cells.iter().filter(|c| c.applies()).count(),
            plan.estimate_s(&cells) / 60.0
        );
    }
    let plan = options.plan();
    let cells = plan.cells(native, &options.selection);
    println!("Cells of {} with these flags:", plan.name());
    for cell in &cells {
        let runs = if cell.applies() {
            format!("{} runs", cell.runs)
        } else {
            "n/a".to_owned()
        };
        println!("  {cell}: {runs}");
    }
    Ok(())
}

/// Writes every scenario's generated input at full duration, `4k` at
/// factor 2, into `out`.
fn write_replays(out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(out)?;
    for scenario in Scenario::ALL {
        let replay = scenario.replay(scenario.window(2.0), scenario.duration_ms());
        let path = out.join(format!("{}.jsonl", scenario.name()));
        replay.write(&path)?;
        println!("{}: {} events", path.display(), replay.events.len());
    }
    Ok(())
}

/// Runs a plan: replays, `meta.json`, then one child process per cell
/// and run.
fn run(options: &Options, args: &[String]) -> Result<(), Box<dyn Error>> {
    let plan = options.plan();
    let refresh_hz = options.refresh_hz()?;
    let native = options.native()?;
    let cells = plan.cells(native, &options.selection);
    if cells.is_empty() {
        return Err("the flags select no cell (see `rustd3-bench list`)".into());
    }
    let stamp = system::stamp();
    let dir = options.out.join(&stamp);
    fs::create_dir_all(dir.join("replays"))?;
    fs::create_dir_all(dir.join("raw"))?;
    let replays = replays(options, plan, &cells, &dir)?;
    let meta = meta(options, &stamp, refresh_hz, native, &cells, args);
    fs::write(dir.join("meta.json"), serde_json::to_string_pretty(&meta)?)?;
    println!(
        "rustd3-bench: plan {} into {} (about {:.0} min on real hardware)",
        plan.name(),
        dir.display(),
        plan.estimate_s(&cells) / 60.0
    );
    let total: u32 = cells.iter().filter(|c| c.applies()).map(|c| c.runs).sum();
    let exe = std::env::current_exe()?;
    let mut done = 0;
    let mut failed = Vec::new();
    for cell in cells.iter().filter(|c| c.applies()) {
        for run in 1..=cell.runs {
            done += 1;
            let mut child_options = options.clone();
            child_options.replay = Some(replays[&cell.key()].clone());
            let child = Child {
                options: child_options,
                cell: Cell { runs: 1, ..*cell },
                run,
                dir: dir.clone(),
            };
            let mut command = std::process::Command::new(&exe);
            command
                .args(child.to_args(refresh_hz, native))
                .env("ICED_BACKEND", options.backend.name());
            if let Some(mode) = options.present_mode {
                command.env("ICED_PRESENT_MODE", mode.name());
            }
            let begin = Instant::now();
            let status = command.status()?;
            let seconds = begin.elapsed().as_secs_f64();
            println!("[{done}/{total}] {cell} run {run}: {status} in {seconds:.1} s");
            if !status.success() {
                failed.push(format!("{cell} run {run} ({status})"));
            }
        }
    }
    println!(
        "rustd3-bench: measuring the items earlier phases asked for (D15.3, D18.3, D19.3, D21.1)"
    );
    fs::write(dir.join("also.md"), also::measure())?;
    let summary = report::build(&dir)?;
    println!("{summary}");
    println!("report: {}", dir.join("report.md").display());
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!("{} runs failed: {}", failed.len(), failed.join("; ")).into())
    }
}

/// Writes each cell's input into `dir/replays` (the user's `--replay`
/// file for every cell, when given) and returns the file per cell key.
fn replays(
    options: &Options,
    plan: plan::Plan,
    cells: &[Cell],
    dir: &Path,
) -> Result<BTreeMap<String, PathBuf>, Box<dyn Error>> {
    let mut files = BTreeMap::new();
    let given = match &options.replay {
        Some(path) => {
            let replay = Replay::read(path)?;
            let copy = dir.join("replays").join("given.jsonl");
            replay.write(&copy)?;
            Some(copy)
        }
        None => None,
    };
    for cell in cells.iter().filter(|c| c.applies()) {
        let path = match &given {
            Some(copy) => copy.clone(),
            None => {
                let size = cell.window();
                let name = format!(
                    "{}__{}x{}.jsonl",
                    cell.scenario.name(),
                    size.width,
                    size.height
                );
                let path = dir.join("replays").join(name);
                if !path.exists() {
                    let duration = plan.duration_ms(cell.scenario);
                    cell.scenario.replay(size, duration).write(&path)?;
                }
                path
            }
        };
        files.insert(cell.key(), path);
    }
    Ok(files)
}

/// What `meta.json` records before the runs.
fn meta(
    options: &Options,
    stamp: &str,
    refresh_hz: f64,
    native: f32,
    cells: &[Cell],
    args: &[String],
) -> Meta {
    let [cpu, kernel, gpu, session, desktop, rustc, git_rev] = system::machine();
    let mut factors: Vec<f32> = Vec::new();
    for cell in cells {
        if !factors.contains(&cell.factor) {
            factors.push(cell.factor);
        }
    }
    Meta {
        stamp: stamp.to_owned(),
        cpu,
        cores: std::thread::available_parallelism().map_or(1, std::num::NonZero::get),
        ram_mb: system::ram_mb(),
        kernel,
        gpu,
        session,
        desktop,
        present_mode: options.present_mode.map_or_else(
            || "AutoVsync (iced default)".to_owned(),
            |m| m.name().to_owned(),
        ),
        backend: options.backend.name().to_owned(),
        headless: options.headless,
        refresh_hz,
        native_factor: native,
        factors,
        render: cli::render_name(options.render).to_owned(),
        caching: cli::caching_name(options.caching).to_owned(),
        readback: match options.readback {
            cli::Readback::Every => "every",
            cli::Readback::Last => "last",
        }
        .to_owned(),
        rustc,
        iced_rev: system::ICED_REV.to_owned(),
        git_rev,
        plan: options.plan().name().to_owned(),
        data_scale: options.data_scale(),
        command: format!("rustd3-bench {}", args.join(" ")),
        pixels_per_ms: cost::PIXELS_PER_MS,
        items_per_ms: cost::ITEMS_PER_MS,
        calibration: None,
        gpu_baseline_mb: system::gpu_mb(),
        clk_tck: system::clk_tck(),
        cells: cells
            .iter()
            .map(|cell| Planned {
                scenario: cell.scenario.name().to_owned(),
                workload: cell.workload.name().to_owned(),
                factor: cell.factor,
                runs: cell.runs,
                applies: cell.applies(),
            })
            .collect(),
    }
}

/// One run of one cell: headless here, or in a window.
fn run_child(child: Child, epoch: Instant) -> Result<(), Box<dyn Error>> {
    let options = &child.options;
    let plan = options.plan();
    let scenario = child.cell.scenario;
    let duration_ms = plan.duration_ms(scenario);
    let replay = match &options.replay {
        Some(path) => Replay::read(path)?,
        None => scenario.replay(child.cell.window(), duration_ms),
    };
    let config = Config {
        refresh_hz: options.refresh_hz()?,
        native: options.native()?,
        duration_ms,
        data_scale: options.data_scale(),
        clk_tck: system::clk_tck(),
        replay,
        epoch,
        child: child.clone(),
    };
    if options.headless {
        let mut app = App::new(config);
        let backend = headless::run(
            &mut app,
            options.backend.name(),
            child.cell.factor,
            options.refresh_hz()?,
            options.readback,
        )?;
        app.finish(&backend)?;
        Ok(())
    } else {
        app::windowed(config).map_err(Into::into)
    }
}
