//! rustd3-bench: the full-frame benchmark of rustd3's `Plot` widget. It
//! drives the real widget through a wrapper that replays time-stamped
//! input, in a window or headlessly, one cold process per run, and writes
//! raw records, CSV summaries and a report that evaluates the decision
//! rule registered before any run. Its one entry point is a level
//! (`rustd3-bench [--quick | --default | --full]`, D22.9); `--help`
//! prints it and the developers' commands.

mod also;
mod app;
mod archive;
mod cli;
mod compare;
mod display;
mod driver;
mod headless;
mod level;
mod load;
mod metrics;
mod plan;
mod privacy;
mod raster;
mod record;
mod recorder;
mod replay;
mod report;
mod repro;
mod results;
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
use std::time::{Duration, Instant};

use rustd3_iced::plot::cost;

use crate::app::{App, Config};
use crate::cli::{Child, Command, Options};
use crate::plan::Cell;
use crate::replay::Replay;
use crate::scenario::{Anchors, Scenario};
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
        Command::Level(request) => std::env::current_exe()
            .map_err(Into::into)
            .and_then(|exe| request.run(&exe)),
        Command::Help => {
            print!("{}", cli::HELP);
            Ok(())
        }
        Command::List(options) => list(&options),
        Command::Raster(options) => raster::run(&options),
        Command::Also => {
            print!("{}", also::measure());
            Ok(())
        }
        Command::Replay { out } => write_replays(&out),
        Command::Record { out } => recorder::record(out).map_err(Into::into),
        Command::ReproSoftware { out, single } => repro_software(out, single),
        Command::Run(options) => run(&options, &args),
        Command::Child(child) => run_child(child, epoch),
        Command::Report { path } => report(&path),
        Command::Compare { paths: [a, b] } => load::results(&a).and_then(|ra| {
            let rb = load::results(&b)?;
            let names = [a.display().to_string(), b.display().to_string()];
            print!("{}", compare::markdown(&ra, &rb, [&names[0], &names[1]]));
            Ok(())
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

/// How long a run may take beyond its scenario before it is killed, s:
/// start, data, the driver's 2 s settle and its 30 s wait for sharp
/// frames, and a margin.
const RUN_MARGIN_S: f64 = 120.0;

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

/// `report PATH`: one run's directory (`meta.json` at its top) rebuilt
/// in place; a level's directory rebuilt in place, or its archive in a
/// temporary directory, each profile's report from its raw records and
/// then `results.json` and `report.md`; a directory from before the
/// levels read as it is. Prints the report.
fn report(path: &Path) -> Result<(), Box<dyn Error>> {
    if path.join("meta.json").is_file() {
        println!("{}", report::build(path)?.summary);
        return Ok(());
    }
    let mut results = load::results(path)?;
    if results.format == 0 {
        print!("{}", results.markdown());
        return Ok(());
    }
    let temp = (!path.is_dir())
        .then(|| std::env::temp_dir().join(format!("rustd3-bench-report-{}", std::process::id())));
    let dir = match &temp {
        None => path.to_path_buf(),
        Some(temp) => {
            archive::extract(&archive::read(path)?, temp)?;
            fs::read_dir(temp)?
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .find(|path| path.is_dir())
                .ok_or("an archive without its directory")?
        }
    };
    for profile in &mut results.profiles {
        let at = dir.join(&profile.name);
        if at.join("meta.json").is_file() {
            report::build(&at)?.apply(profile);
        }
    }
    let markdown = results.markdown();
    match &temp {
        Some(temp) => fs::remove_dir_all(temp)?,
        None => {
            fs::write(
                dir.join("results.json"),
                serde_json::to_string_pretty(&results)?,
            )?;
            fs::write(dir.join("report.md"), &markdown)?;
        }
    }
    print!("{markdown}");
    Ok(())
}

/// Writes every scenario's generated input at full duration for the
/// standard figures, `4k` at factor 2, into `out`.
fn write_replays(out: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(out)?;
    for scenario in Scenario::ALL {
        let window = scenario.window(2.0);
        let replay = scenario.replay(window, Anchors::STANDARD, scenario.duration_ms());
        let path = out.join(format!("{}.jsonl", scenario.name()));
        replay.write(&path)?;
        println!("{}: {} events", path.display(), replay.events.len());
    }
    Ok(())
}

/// `repro-software` (with one window-sized image if `single`) into `out`,
/// else `bench-results/repro-<UTC stamp>.txt`.
fn repro_software(out: Option<PathBuf>, single: bool) -> Result<(), Box<dyn Error>> {
    let out = out.unwrap_or_else(|| {
        PathBuf::from("bench-results").join(format!("repro-{}.txt", system::stamp()))
    });
    if let Some(dir) = out.parent() {
        fs::create_dir_all(dir)?;
    }
    repro::run(out, system::clk_tck(), single)?;
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
    if options.xwayland && !level::xwayland() {
        return Err(format!("--xwayland: {}", level::NO_XWAYLAND).into());
    }
    let stamp = system::stamp();
    let dir = options.out.join(&stamp);
    fs::create_dir_all(dir.join("replays"))?;
    fs::create_dir_all(dir.join("raw"))?;
    let replays = replays(options, (plan, native), &cells, &dir)?;
    let meta = meta(options, &stamp, refresh_hz, native, &cells, args);
    fs::write(dir.join("meta.json"), serde_json::to_string_pretty(&meta)?)?;
    fs::write(dir.join("system.txt"), system::diagnostics())?;
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
            command.args(child.to_args(refresh_hz, native));
            match options.backend.iced() {
                Some(backend) => command.env("ICED_BACKEND", backend),
                None => command.env_remove("ICED_BACKEND"),
            };
            if let Some(mode) = options.present_mode {
                command.env("ICED_PRESENT_MODE", mode.name());
            }
            if options.xwayland {
                // winit prefers Wayland while this is set (D22.8).
                command.env_remove("WAYLAND_DISPLAY");
            }
            let begin = Instant::now();
            // A run that hangs (a window that never draws) ends as failed.
            let limit =
                Duration::from_secs_f64(plan.duration_ms(cell.scenario) / 1e3 + RUN_MARGIN_S);
            let status = system::wait(&mut command.spawn()?, limit)?;
            let seconds = begin.elapsed().as_secs_f64();
            let shown = status.map_or_else(
                || format!("killed after {seconds:.0} s"),
                |status| status.to_string(),
            );
            println!("[{done}/{total}] {cell} run {run}: {shown} in {seconds:.1} s");
            if !status.is_some_and(|status| status.success()) {
                failed.push(format!("{cell} run {run} ({shown})"));
            }
        }
    }
    if options.also {
        println!(
            "rustd3-bench: measuring the items earlier phases asked for (D15.3, D18.3, D19.3, \
             D21.1)"
        );
        fs::write(dir.join("also.md"), also::measure())?;
    }
    let built = report::build(&dir)?;
    println!("{}", built.summary);
    println!("report: {}", dir.join("report.md").display());
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!("{} runs failed: {}", failed.len(), failed.join("; ")).into())
    }
}

/// Writes each cell's input, for its window on a display of factor
/// `native` and its workload's anchors, into `dir/replays` (the user's
/// `--replay` file for every cell, when given) and returns the file per
/// cell key.
fn replays(
    options: &Options,
    (plan, native): (plan::Plan, f32),
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
                let size = cell.window(native);
                let name = format!(
                    "{}__{}__{}x{}.jsonl",
                    cell.scenario.name(),
                    cell.workload.name(),
                    size.width,
                    size.height
                );
                let path = dir.join("replays").join(name);
                if !path.exists() {
                    let duration = plan.duration_ms(cell.scenario);
                    let anchors = cell.workload.anchors();
                    cell.scenario.replay(size, anchors, duration).write(&path)?;
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
        threads: options.threads,
        readback: match options.readback {
            cli::Readback::Every => "every",
            cli::Readback::Last => "last",
        }
        .to_owned(),
        rustc,
        iced_rev: system::ICED_REV.to_owned(),
        git_rev,
        plan: options.plan().name().to_owned(),
        budgets: options.budgets.map(|budgets| budgets.name().to_owned()),
        data_scale: options.data_scale(),
        command: format!("rustd3-bench {}", args.join(" ")),
        pixels_per_ms: cost::PIXELS_PER_MS,
        items_per_ms: cost::ITEMS_PER_MS,
        calibration: None,
        gpu_baseline_mb: system::gpu_mb(),
        clk_tck: system::clk_tck(),
        cpu_limit: options.cpu_limit.clone(),
        xwayland: options.xwayland,
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
        None => {
            let window = child.cell.window(options.native()?);
            scenario.replay(window, child.cell.workload.anchors(), duration_ms)
        }
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
            options.backend.iced(),
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
