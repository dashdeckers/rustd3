//! The commands of `bench_gestures` and `bench_software`, what
//! `bench_all` runs with the widget's part, and `bench_report`'s rebuild
//! of each profile from its raw records.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use rustd3_iced::plot::cost;

use crate::app::{App, Config};
use crate::cli::widget::{self, Child, Options};
use crate::level::{self, Request};
use crate::plan::{self, Cell};
use crate::replay::Replay;
use crate::scenario::{Anchors, Scenario};
use crate::sibling::Siblings;
use crate::system::{self, Meta, Planned};
use crate::workload::Workload;
use crate::{Example, also, app, archive, headless, load, raster, recorder, report, repro, rule};

/// Runs a command of `example`. `args` are the process's arguments, and
/// `epoch` is its start.
pub fn execute(
    command: widget::Command,
    example: Example,
    args: &[String],
    epoch: Instant,
) -> Result<(), Box<dyn Error>> {
    match command {
        widget::Command::List(options) => list(&options),
        widget::Command::Raster(options) => raster::run(&options),
        widget::Command::Also => {
            print!("{}", also::measure());
            Ok(())
        }
        widget::Command::Replay { out } => write_replays(&out),
        widget::Command::Record { out } => recorder::record(out).map_err(Into::into),
        widget::Command::ReproSoftware { out, single } => repro_software(out, single),
        widget::Command::Run(options) => run(&options, example, args),
        widget::Command::Child(child) => run_child(child, epoch),
    }
}

/// Runs `request`'s level, with the widget's part unless `--figure-only`.
/// Each profile runs in the sibling example that runs it.
pub fn level(request: &Request) -> Result<(), Box<dyn Error>> {
    let siblings = Siblings::here()?;
    let mut part = level::widget::Part::new(request, &siblings);
    let widget: &mut dyn level::Widget = &mut part;
    request.run(&siblings.figure, (!request.figure_only).then_some(widget))
}

/// How long a run may take beyond its scenario before it is killed, in s.
/// It covers start-up, data generation, the driver's 2 s settle and its
/// 30 s wait for sharp frames, plus a margin.
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

/// `report PATH`: rebuilds a report and prints it. One run's directory
/// (`meta.json` at its top) is rebuilt in place. A level's directory is
/// rebuilt in place, or its archive in a temporary directory: first each
/// profile's report from its raw records, then `results.json` and
/// `report.md`. A directory from before the levels is read as it is.
pub fn report(path: &Path) -> Result<(), Box<dyn Error>> {
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
        .then(|| std::env::temp_dir().join(format!("benchkit-report-{}", std::process::id())));
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
/// standard figures, with `4k` at factor 2, into `out`.
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

/// Runs `repro-software` (with one image larger than the window if
/// `single`) and writes to `out`, or else to
/// `bench-results/repro-<UTC stamp>.txt`.
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
fn run(options: &Options, example: Example, args: &[String]) -> Result<(), Box<dyn Error>> {
    let plan = options.plan();
    let refresh_hz = options.refresh_hz()?;
    let native = options.native()?;
    let cells = plan.cells(native, &options.selection);
    if cells.is_empty() {
        return Err("the flags select no cell (see `bench_gestures list`)".into());
    }
    if options.xwayland && !level::widget::xwayland() {
        return Err(format!("--xwayland: {}", level::widget::NO_XWAYLAND).into());
    }
    if options.frames.is_some() && !options.headless {
        return Err(
            "--frames: only a headless run composes frames to write (add --headless)".into(),
        );
    }
    let stamp = system::stamp();
    let dir = options.out.join(&stamp);
    fs::create_dir_all(dir.join("replays"))?;
    fs::create_dir_all(dir.join("raw"))?;
    let replays = replays(options, (plan, native), &cells, &dir)?;
    let command = format!("{} {}", example.name(), args.join(" "));
    let meta = meta(options, &stamp, refresh_hz, native, &cells, command);
    fs::write(dir.join("meta.json"), serde_json::to_string_pretty(&meta)?)?;
    fs::write(dir.join("system.txt"), system::diagnostics())?;
    println!(
        "{}: plan {} into {} (about {:.0} min on real hardware)",
        example.name(),
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
            // On Windows wgpu draws through DirectX 12 unless the user chose
            // an API: Intel's Vulkan driver shows frames it has not finished
            // drawing. iced still falls back to the software renderer
            // where no adapter is found.
            if cfg!(windows) && std::env::var_os("WGPU_BACKEND").is_none() {
                command.env("WGPU_BACKEND", "dx12");
            }
            if let Some(mode) = options.present_mode {
                command.env("ICED_PRESENT_MODE", mode.name());
            }
            if options.xwayland {
                // winit prefers Wayland while this is set.
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
            "{}: measuring the items beside the scenarios (thinning, cell origins, the \
             dense scatter, the windowed resolve)",
            example.name()
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

/// Writes each cell's input into `dir/replays` and returns the file per
/// cell key. Each input is generated for the cell's window on a display of
/// factor `native` and for its workload's anchors. When the user gives a
/// `--replay` file, it serves every cell.
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
    command: String,
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
        render: widget::render_name(options.render).to_owned(),
        caching: widget::caching_name(options.caching).to_owned(),
        threads: options.threads,
        readback: match options.readback {
            widget::Readback::Every => "every",
            widget::Readback::Last => "last",
        }
        .to_owned(),
        rustc,
        iced_rev: system::ICED_REV.to_owned(),
        git_rev,
        plan: options.plan().name().to_owned(),
        budgets: options.budgets.map(|budgets| budgets.name().to_owned()),
        data_scale: options.data_scale(),
        command,
        pixels_per_ms: cost::PIXELS_PER_MS,
        items_per_ms: cost::ITEMS_PER_MS,
        calibration: None,
        gpu_baseline_mb: system::gpu_mb(),
        clk_tck: system::clk_tck(),
        cpu_limit: options.cpu_limit.clone(),
        xwayland: options.xwayland,
        frames: options.frames.as_ref().map(|_| options.every),
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
        let frames = options.frames.as_ref().map(|dir| headless::Frames {
            dir: dir.join(format!("{}__r{}", child.cell.key(), child.run)),
            every: options.every,
        });
        let settings = headless::Settings {
            backend: options.backend.iced(),
            factor: child.cell.factor,
            refresh_hz: options.refresh_hz()?,
            readback: options.readback,
            frames,
        };
        let backend = headless::run(&mut app, &settings)?;
        app.finish(&backend)?;
        Ok(())
    } else {
        app::windowed(config).map_err(Into::into)
    }
}
