//! The widget's part of a level (D22.9), built with the `widget`
//! feature: the display measurement before the CPU reference, then, after
//! the figure profile, the [`Profile`]s that drive the `Plot` widget, each a
//! `run` of a plan in its own directory of the level's with its console in
//! `run-<profile>.log`: the software path offscreen (D22.12, headless on
//! any machine), the native backend (iced's own choice), the software
//! renderer, the software renderer on X11 in a Linux Wayland session
//! (D22.8), and at the default and full levels the weak plan on 4 and on 2
//! cores (D22.4); at `--full`, every `also` measurement last. A
//! core count is enforced by pinning the bench where the system offers it
//! (`taskset` on Linux, `start /AFFINITY` on Windows) and otherwise by the
//! widget's render pool (`--threads`), and the report says which. Without
//! a display the profiles run headless, a profile the machine cannot run
//! is skipped with its reason, and a profile whose runs fail is recorded
//! and the level goes on.

use std::error::Error;
use std::ffi::OsStr;
use std::fmt::Write as _;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use iced::advanced::renderer::{self, Headless as _};

use crate::also;
use crate::cli::widget::Backend;
use crate::display::{self, Display};
use crate::level::{Level, Outline, Request, Widget};
use crate::plan::{self, Plan, Selection};
use crate::report;
use crate::results;
use crate::rule;
use crate::scenario::Scenario;
use crate::system;
use crate::workload::Workload;

/// The widget's part of a level, which `Request::run` runs.
#[derive(Debug)]
pub struct Part {
    /// The level.
    level: Level,
    /// The bench, which runs each profile in a process of its own.
    exe: PathBuf,
    /// `--refresh-hz`.
    refresh_hz: Option<f64>,
    /// `--native-factor`.
    native: Option<f32>,
    /// Why no window can open here, known without trying.
    unreachable: Option<String>,
    /// The machine before the display is measured.
    guess: Machine,
    /// The display, once measured.
    display: Option<Display>,
    /// The profiles this machine runs, each with why it is skipped, if it
    /// is: known once the display is measured.
    profiles: Vec<(Profile, Option<String>)>,
    /// `--frames DIR` and `--every N`, for the software profile.
    frames: Option<(PathBuf, u32)>,
}

/// One `run` of a level: a plan (and a selection of its cells) on a
/// backend, held to a profile of budgets, perhaps limited in cores or
/// opened on X11 through XWayland.
#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    /// Its directory and log name: `software`, `native`, `nogpu`,
    /// `nogpu-x11`, `4cores`, `2cores`.
    pub name: &'static str,
    /// The plan.
    pub plan: Plan,
    /// The cells of the plan it runs.
    pub selection: Selection,
    /// The renderer asked for.
    pub backend: Backend,
    /// The budgets its report holds the cells to.
    pub budgets: rule::Profile,
    /// The cores it is limited to.
    pub cores: Option<usize>,
    /// Its windows open on X11 through XWayland (D22.8).
    pub xwayland: bool,
    /// It composes its frames offscreen, headless whatever the display, at
    /// 60 Hz and a native factor of 2 so that its cells keep 1600 × 1000
    /// logical px at both factors (D22.12).
    pub offscreen: bool,
}

/// The software profile's frame rate: no display paces it (D22.12).
const OFFSCREEN_HZ: f64 = 60.0;

/// The native factor the software profile runs at: its factor-1 and
/// factor-2 cells both keep 1600 × 1000 logical px.
const OFFSCREEN_NATIVE: f32 = 2.0;

/// What a level knows of the machine when it decides which profiles run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Machine {
    /// Logical cores.
    pub cores: usize,
    /// A Linux Wayland session with XWayland.
    pub xwayland: bool,
    /// No window opens: every profile runs headless.
    pub headless: bool,
    /// iced's own choice is the software renderer (no GPU adapter).
    pub software: bool,
}

/// How a profile's cores are limited.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Limit {
    /// `taskset -c 0-(n-1)` (Linux, when present).
    Taskset(usize),
    /// `cmd /C start "" /B /WAIT /AFFINITY <mask>` (Windows).
    Affinity(usize),
    /// `--threads n-1` on the widget's pool, no pinning (macOS, or Linux
    /// without `taskset`).
    Threads(usize),
}

/// Why `nogpu-x11` does not run here.
pub const NO_XWAYLAND: &str = "it runs only in a Linux Wayland session with XWayland \
                               (WAYLAND_DISPLAY and DISPLAY both set)";

/// The display measurement's allowance in the estimate beyond its plan,
/// s: a window opening once.
const DISPLAY_MARGIN_S: f64 = 3.0;

/// The `also` measurements' allowance in the full level's estimate, s
/// (about 120 here).
const ALSO_S: f64 = 180.0;

impl Level {
    /// Its profiles on `machine`, each with the reason it is skipped, if
    /// it is.
    pub fn profiles(self, machine: Machine) -> Vec<(Profile, Option<String>)> {
        let (main, weak, software) = match self {
            Level::Quick => (
                Plan::Quick,
                Plan::Quick,
                Selection {
                    scenarios: Some(vec![Scenario::PanX, Scenario::ZoomLines]),
                    workloads: Some(vec![Workload::Trace]),
                    ..Selection::default()
                },
            ),
            Level::Default => (Plan::Default, Plan::Weak, Selection::default()),
            Level::Full => (Plan::Full, Plan::Weak, Selection::default()),
        };
        let offscreen = Profile {
            name: "software",
            plan: Plan::Software,
            selection: Selection {
                runs: match self {
                    Level::Quick => Some(1),
                    Level::Default => None,
                    Level::Full => Some(5),
                },
                ..Selection::default()
            },
            backend: Backend::TinySkia,
            budgets: rule::Profile::Weak,
            cores: None,
            xwayland: false,
            offscreen: true,
        };
        let native = Profile {
            name: "native",
            plan: main,
            selection: Selection::default(),
            backend: Backend::Auto,
            budgets: rule::Profile::Desktop,
            cores: None,
            xwayland: false,
            offscreen: false,
        };
        let nogpu = Profile {
            name: "nogpu",
            plan: weak,
            selection: software,
            backend: Backend::TinySkia,
            budgets: rule::Profile::Weak,
            cores: None,
            xwayland: false,
            offscreen: false,
        };
        let x11 = Profile {
            name: "nogpu-x11",
            xwayland: true,
            ..nogpu.clone()
        };
        let duplicate = machine.software.then(|| {
            "the native backend already is the software renderer (no GPU adapter), so its cells \
             ran in the native profile"
                .to_owned()
        });
        let no_x11 = if machine.headless {
            Some("no window opens here, and it measures X11's windows".to_owned())
        } else if machine.xwayland {
            None
        } else {
            Some(NO_XWAYLAND.to_owned())
        };
        let mut profiles = vec![
            (offscreen, None),
            (native, None),
            (nogpu, duplicate),
            (x11, no_x11),
        ];
        if self != Level::Quick {
            for (name, n) in [("4cores", 4), ("2cores", 2)] {
                let profile = Profile {
                    name,
                    plan: Plan::Weak,
                    selection: Selection::default(),
                    backend: Backend::Auto,
                    budgets: rule::Profile::Weak,
                    cores: Some(n),
                    xwayland: false,
                    offscreen: false,
                };
                let few = (machine.cores < n).then(|| match machine.cores {
                    1 => "this machine has 1 core".to_owned(),
                    cores => format!("this machine has {cores} cores"),
                });
                profiles.push((profile, few));
            }
        }
        profiles
    }

    /// The estimated wall time of the level with `profiles` on a display
    /// of factor `native`, s: the CPU reference and the figure profile,
    /// the display measurement, then each profile that runs, from its
    /// plan's durations and run counts plus the per-run overhead
    /// (`Plan::estimate_s`), and at `--full` the `also` measurements.
    pub fn estimate_s(self, profiles: &[(Profile, Option<String>)], native: f32) -> f64 {
        let display = Plan::Display.cells(native, &Selection::default());
        let display = Plan::Display.estimate_s(&display) + DISPLAY_MARGIN_S;
        let measured = if self == Level::Full { ALSO_S } else { 0.0 };
        self.library_s(system::ram_mb())
            + display
            + measured
            + profiles
                .iter()
                .filter(|(_, skip)| skip.is_none())
                .map(|(profile, _)| profile.estimate_s(native))
                .sum::<f64>()
    }
}

impl Profile {
    /// Its cells with the display's `native` factor (the software
    /// profile's own, offscreen).
    fn cells(&self, native: f32) -> Vec<crate::plan::Cell> {
        let native = if self.offscreen {
            OFFSCREEN_NATIVE
        } else {
            native
        };
        self.plan.cells(native, &self.selection)
    }

    /// Its estimated wall time, s.
    pub fn estimate_s(&self, native: f32) -> f64 {
        self.plan.estimate_s(&self.cells(native))
    }

    /// What it runs, in a line.
    pub fn describe(&self, native: f32) -> String {
        let cells = self.cells(native);
        let runs: u32 = cells.iter().filter(|c| c.applies()).map(|c| c.runs).sum();
        let which = match (&self.selection.scenarios, &self.selection.workloads) {
            (Some(scenarios), Some(workloads)) => format!(
                " ({} × {})",
                scenarios
                    .iter()
                    .map(|s| s.name())
                    .collect::<Vec<_>>()
                    .join(", "),
                workloads
                    .iter()
                    .map(|w| w.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            (Some(_) | None, Some(_) | None) => String::new(),
        };
        let backend = match (self.backend, self.offscreen) {
            (Backend::TinySkia, true) => {
                "the software renderer (tiny-skia) offscreen: headless, every frame composed into \
                 a pixmap at 60 Hz, 1600 × 1000 logical px at factors 1 and 2"
            }
            (Backend::Auto, _) => "the native backend (iced's choice)",
            (Backend::TinySkia, false) => "the software renderer (tiny-skia)",
            (Backend::Wgpu, _) => "wgpu",
        };
        let mut text = format!(
            "plan {}{which}: {} cells, {runs} runs on {backend}",
            self.plan.name(),
            cells.len()
        );
        if let Some(n) = self.cores {
            let _ = write!(text, ", limited to {n} cores");
        }
        if self.xwayland {
            let _ = write!(text, ", on X11 through XWayland");
        }
        let _ = write!(text, "; {} budgets", self.budgets.name());
        text
    }

    /// What `results.json` holds of it once it ran with `status`, from
    /// its directory under `dir`.
    fn summary(
        &self,
        display: &Display,
        dir: &Path,
        status: ExitStatus,
        limit: Option<&Limit>,
    ) -> results::Profile {
        let mut out = results::Profile {
            plan: self.plan.name().to_owned(),
            backend: self.backend.name().to_owned(),
            budgets: self.budgets.name().to_owned(),
            limit: limit.map(Limit::describe),
            xwayland: self.xwayland,
            headless: self.offscreen || display.headless.is_some(),
            status: results::Status::Done,
            ..results::Profile::planned(self.name, self.describe(display.native_factor), None)
        };
        let log = format!("run-{}.log", self.name);
        match report::build(&dir.join(self.name)) {
            Ok(built) => built.apply(&mut out),
            Err(error) => {
                out.status = results::Status::Failed;
                out.reason = Some(format!("it wrote no report ({error}; {log})"));
                return out;
            }
        }
        let applies = out.cells.iter().filter(|c| c.applies);
        let (runs, read) = applies.fold((0, 0), |(runs, read), c| (runs + c.runs, read + c.read));
        if read < runs {
            out.status = results::Status::Failed;
            out.reason = Some(format!("{} of {runs} runs failed ({log})", runs - read));
        } else if !status.success() {
            out.status = results::Status::Failed;
            out.reason = Some(format!("its process ended with {status} ({log})"));
        }
        out
    }

    /// `run`'s arguments for this profile on `display`, writing under
    /// `out`, and for the software profile its frames into `frames`.
    fn args(&self, display: &Display, out: &Path, frames: Option<&(PathBuf, u32)>) -> Vec<String> {
        let mut args: Vec<String> = [
            "run",
            "--plan",
            self.plan.name(),
            "--backend",
            self.backend.name(),
            "--budgets",
            self.budgets.name(),
            "--no-also",
        ]
        .map(str::to_owned)
        .into();
        let (hz, native) = if self.offscreen {
            (OFFSCREEN_HZ, OFFSCREEN_NATIVE)
        } else {
            (display.refresh_hz, display.native_factor)
        };
        args.extend([
            "--refresh-hz".to_owned(),
            hz.to_string(),
            "--native-factor".to_owned(),
            native.to_string(),
        ]);
        if let Some(scenarios) = &self.selection.scenarios {
            let names: Vec<&str> = scenarios.iter().map(|s| s.name()).collect();
            args.extend(["--scenario".to_owned(), names.join(",")]);
        }
        if let Some(workloads) = &self.selection.workloads {
            let names: Vec<&str> = workloads.iter().map(|w| w.name()).collect();
            args.extend(["--workload".to_owned(), names.join(",")]);
        }
        if let Some(runs) = self.selection.runs {
            args.extend(["--runs".to_owned(), runs.to_string()]);
        }
        if self.xwayland {
            args.push("--xwayland".to_owned());
        }
        if self.offscreen || display.headless.is_some() {
            args.push("--headless".to_owned());
        }
        if let Some((dir, every)) = frames.filter(|_| self.offscreen) {
            args.extend([
                "--frames".to_owned(),
                dir.display().to_string(),
                "--every".to_owned(),
                every.to_string(),
            ]);
        }
        args.extend(["--out".to_owned(), out.display().to_string()]);
        args
    }
}

impl Part {
    /// The widget's part of `request`'s level, its profiles run by the
    /// bench at `exe`.
    pub fn new(request: &Request, exe: &Path) -> Part {
        let unreachable = display::unreachable_here();
        Part {
            level: request.level,
            exe: exe.to_path_buf(),
            refresh_hz: request.refresh_hz,
            native: request.native,
            guess: Machine {
                cores: std::thread::available_parallelism().map_or(1, std::num::NonZero::get),
                xwayland: xwayland(),
                headless: unreachable.is_some(),
                software: false,
            },
            unreachable,
            display: None,
            profiles: Vec::new(),
            frames: request.frames.clone().map(|dir| (dir, request.every)),
        }
    }

    /// The display: headless without trying where no window can open,
    /// else from the display plan's one window (`display/`), headless if
    /// it opened none.
    fn measure(&self, dir: &Path) -> Result<Display, Box<dyn Error>> {
        let measured = match self.unreachable.clone() {
            Some(why) => Err(why),
            None => {
                let hz = self.refresh_hz.unwrap_or(display::ASSUMED_HZ).to_string();
                let factor = self.native.unwrap_or(1.0).to_string();
                let args: Vec<String> = [
                    "run",
                    "--plan",
                    "display",
                    "--backend",
                    "auto",
                    "--no-also",
                    "--refresh-hz",
                    &hz,
                    "--native-factor",
                    &factor,
                    "--out",
                    &dir.join("display").display().to_string(),
                ]
                .map(str::to_owned)
                .into();
                println!("== display: one window, a few seconds (run-display.log)");
                let status = spawn(&self.exe, dir, "display", None, args, false)?;
                display::read(&dir.join("display").join("raw")).ok_or_else(|| {
                    format!(
                        "no window opened: the display measurement wrote no frames ({status}; \
                         run-display.log)"
                    )
                })
            }
        };
        let mut display = Display::decide(self.refresh_hz, self.native, measured);
        if display.headless.is_some() {
            display.backend = headless_backend();
        }
        Ok(display)
    }
}

impl Widget for Part {
    fn outline(&self) -> Outline {
        let native = self.native.unwrap_or(1.0);
        let profiles = self.level.profiles(self.guess);
        let display = match &self.unreachable {
            Some(why) => format!("display    {why}: every profile runs headless"),
            None => "display    one window measures the refresh rate and scale factor, and which \
                     renderer iced chooses"
                .to_owned(),
        };
        let mut after = Vec::new();
        for (profile, skip) in &profiles {
            after.push(match skip {
                Some(why) => format!("{:<10} skipped: {why}", profile.name),
                None => {
                    let unless = if profile.name == "nogpu" {
                        " (skipped if iced's own choice is the software renderer)"
                    } else {
                        ""
                    };
                    format!(
                        "{:<10} {}, about {:.1} min{unless}",
                        profile.name,
                        profile.describe(native),
                        profile.estimate_s(native) / 60.0
                    )
                }
            });
        }
        if self.level == Level::Full {
            after.push(format!(
                "also       every `also` measurement, into also.md, about {:.0} min",
                ALSO_S / 60.0
            ));
        }
        Outline {
            before: vec![display],
            after,
            estimate_s: self.level.estimate_s(&profiles, native),
            note: format!(
                "each run's scenario plus {:.0} s to start and settle",
                plan::OVERHEAD_MS / 1e3
            ),
        }
    }

    fn display(&mut self, dir: &Path) -> Result<Display, Box<dyn Error>> {
        let display = self.measure(dir)?;
        println!("Display: {}.", display.describe());
        if display.unpaced() {
            println!(
                "The display paces no frames: the profiles run at {} Hz (--refresh-hz sets \
                 another rate).",
                display.refresh_hz
            );
        }
        let machine = Machine {
            headless: display.headless.is_some(),
            software: display.software(),
            ..self.guess
        };
        self.profiles = self.level.profiles(machine);
        self.display = Some(display.clone());
        Ok(display)
    }

    fn profiles(&self) -> Vec<results::Profile> {
        let Some(display) = &self.display else {
            return Vec::new();
        };
        self.profiles
            .iter()
            .map(|(p, skip)| {
                results::Profile::planned(p.name, p.describe(display.native_factor), skip.clone())
            })
            .collect()
    }

    fn run(
        &mut self,
        index: usize,
        dir: &Path,
        notes: &mut String,
    ) -> Result<results::Profile, Box<dyn Error>> {
        let display = self.display.as_ref().ok_or("no display measured")?;
        let (profile, skip) = self.profiles.get(index).ok_or("no such profile")?;
        let native = display.native_factor;
        if let Some(why) = skip {
            println!("Skipping {}: {why}.", profile.name);
            let _ = writeln!(notes, "# {}: skipped, {why}", profile.name);
            return Ok(results::Profile::planned(
                profile.name,
                profile.describe(native),
                Some(why.clone()),
            ));
        }
        let limit = profile.cores.map(Limit::available);
        if let Some(limit) = &limit {
            let _ = writeln!(notes, "# {}: {}", profile.name, limit.describe());
        }
        let args = profile.args(display, &dir.join(profile.name), self.frames.as_ref());
        println!("== {}: {}", profile.name, profile.describe(native));
        let status = spawn(&self.exe, dir, profile.name, limit.as_ref(), args, true)?;
        Ok(profile.summary(display, dir, status, limit.as_ref()))
    }

    fn finish(&mut self, dir: &Path, notes: &mut String) -> Result<(), Box<dyn Error>> {
        if self.level == Level::Full {
            println!("== also: the items earlier phases asked to be measured (also.md)");
            match std::panic::catch_unwind(also::measure) {
                Ok(md) => fs::write(dir.join("also.md"), md)?,
                Err(_) => {
                    let _ = writeln!(notes, "# also: a measurement panicked (run's console)");
                }
            }
        }
        Ok(())
    }
}

impl Limit {
    /// How this machine limits a profile to `n` cores.
    fn available(n: usize) -> Limit {
        if cfg!(target_os = "linux") && present("taskset", &["--version"]) {
            Limit::Taskset(n)
        } else if cfg!(windows) {
            Limit::Affinity(n)
        } else {
            Limit::Threads(n)
        }
    }

    /// What the report says the limit is.
    fn describe(&self) -> String {
        match *self {
            Limit::Taskset(n) => format!("pinned to {n} cores (taskset -c 0-{})", n - 1),
            Limit::Affinity(n) => format!("pinned to {n} cores (start /AFFINITY {})", mask(n)),
            Limit::Threads(n) => format!(
                "not pinned; the widget's render pool set to {} threads (--threads), as on {n} cores",
                n.saturating_sub(1).max(1)
            ),
        }
    }

    /// The command that runs `exe` with `args` under the limit.
    fn command(&self, exe: &Path, args: &[String]) -> Command {
        match *self {
            Limit::Taskset(n) => {
                let mut command = Command::new("taskset");
                command
                    .args(["-c", &format!("0-{}", n - 1)])
                    .arg(exe)
                    .args(args);
                command
            }
            Limit::Affinity(n) => {
                // The empty title start takes before its flags.
                let mut command = Command::new("cmd");
                command
                    .args(["/C", "start", "", "/B", "/WAIT", "/AFFINITY", &mask(n)])
                    .arg(exe)
                    .args(args);
                command
            }
            Limit::Threads(n) => {
                let mut command = Command::new(exe);
                command
                    .args(args)
                    .args(["--threads", &n.saturating_sub(1).max(1).to_string()]);
                command
            }
        }
    }
}

/// The affinity mask of the first `n` cores, in hex.
fn mask(n: usize) -> String {
    format!("{:X}", (1_u64 << n.min(63)) - 1)
}

/// Runs `exe ARGS` (`run … --out DIR/NAME`, under `limit` with its
/// `--cpu-limit`, if any), its console written to `DIR/run-NAME.log` and,
/// when `shown`, printed, and moves the stamped directory the bench
/// writes into `DIR/NAME` up into it.
fn spawn(
    exe: &Path,
    dir: &Path,
    name: &str,
    limit: Option<&Limit>,
    mut args: Vec<String>,
    shown: bool,
) -> Result<ExitStatus, Box<dyn Error>> {
    let out = dir.join(name);
    if let Some(limit) = limit {
        args.extend(["--cpu-limit".to_owned(), limit.describe()]);
    }
    let mut command = match limit {
        Some(limit) => limit.command(exe, &args),
        None => {
            let mut command = Command::new(exe);
            command.args(&args);
            command
        }
    };
    let log = Arc::new(Mutex::new(fs::File::create(
        dir.join(format!("run-{name}.log")),
    )?));
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let echoes = [
        child
            .stdout
            .take()
            .map(|pipe| echo(pipe, Arc::clone(&log), shown.then_some(false))),
        child
            .stderr
            .take()
            .map(|pipe| echo(pipe, Arc::clone(&log), shown.then_some(true))),
    ];
    let status = child.wait()?;
    for echo in echoes.into_iter().flatten() {
        let _ = echo.join();
    }
    // The bench writes into a stamped directory under --out.
    if let Ok(entries) = fs::read_dir(&out)
        && let Some(stamped) = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| path.is_dir())
    {
        for entry in fs::read_dir(&stamped)? {
            let entry = entry?;
            fs::rename(entry.path(), out.join(entry.file_name()))?;
        }
        fs::remove_dir(&stamped)?;
    }
    Ok(status)
}

/// Copies `pipe` line by line to `log` and, unless `console` is `None`,
/// to the console (standard error when `Some(true)`), on its own thread.
fn echo(
    pipe: impl io::Read + Send + 'static,
    log: Arc<Mutex<fs::File>>,
    console: Option<bool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for line in BufReader::new(pipe).lines().map_while(Result::ok) {
            match console {
                Some(true) => eprintln!("{line}"),
                Some(false) => println!("{line}"),
                None => {}
            }
            if let Ok(mut log) = log.lock() {
                let _ = writeln!(log, "{line}");
            }
        }
    })
}

/// Whether this process runs in a Linux Wayland session with XWayland,
/// where the levels run `nogpu-x11`.
pub fn xwayland() -> bool {
    xwayland_on(
        std::env::consts::OS,
        std::env::var_os("WAYLAND_DISPLAY").as_deref(),
        std::env::var_os("DISPLAY").as_deref(),
    )
}

/// Whether `nogpu-x11` runs on `os` with these `WAYLAND_DISPLAY` and
/// `DISPLAY`: on Linux with both set, empty counting as unset as winit
/// reads them.
fn xwayland_on(os: &str, wayland_display: Option<&OsStr>, display: Option<&OsStr>) -> bool {
    let set = |var: Option<&OsStr>| var.is_some_and(|v| !v.is_empty());
    os == "linux" && set(wayland_display) && set(display)
}

/// Whether `program` runs.
fn present(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// The renderer a headless run gets from iced's own choice here; `None`
/// when there is none.
fn headless_backend() -> Option<String> {
    let renderer = iced::futures::executor::block_on(iced::Renderer::new(
        renderer::Settings::default(),
        None,
    ))?;
    Some(renderer.name())
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::{Path, PathBuf};

    use super::{Level, Limit, Machine, mask, xwayland_on};
    use crate::cli::widget::{self, Backend};
    use crate::display::{Display, Measured};
    use crate::plan::Plan;
    use crate::rule;

    /// A desktop: 16 cores, X11, a GPU.
    const DESKTOP: Machine = Machine {
        cores: 16,
        xwayland: false,
        headless: false,
        software: false,
    };

    /// The names of the profiles that run, and those skipped.
    fn names(level: Level, machine: Machine) -> (Vec<&'static str>, Vec<&'static str>) {
        let profiles = level.profiles(machine);
        let run = profiles
            .iter()
            .filter(|(_, skip)| skip.is_none())
            .map(|(p, _)| p.name)
            .collect();
        let skipped = profiles
            .iter()
            .filter(|(_, skip)| skip.is_some())
            .map(|(p, _)| p.name)
            .collect();
        (run, skipped)
    }

    #[test]
    fn each_level_has_d22_9s_profiles() {
        let quick = Level::Quick.profiles(DESKTOP);
        let [
            (software, None),
            (native, None),
            (nogpu, None),
            (x11, Some(_)),
        ] = &quick[..]
        else {
            panic!("{quick:?}");
        };
        // D22.12: the software path offscreen, one run a cell at quick.
        assert_eq!(
            (software.plan, software.backend, software.budgets),
            (Plan::Software, Backend::TinySkia, rule::Profile::Weak)
        );
        assert!(software.offscreen);
        assert_eq!(software.cells(1.25).len(), 24);
        assert!(software.cells(1.0).iter().all(|c| c.runs == 1));
        assert_eq!(
            (native.plan, native.backend, native.budgets),
            (Plan::Quick, Backend::Auto, rule::Profile::Desktop)
        );
        assert_eq!(
            (nogpu.plan, nogpu.backend, nogpu.budgets),
            (Plan::Quick, Backend::TinySkia, rule::Profile::Weak)
        );
        assert_eq!(nogpu.cells(1.0).len(), 2, "pan-x and zoom-lines on trace");
        assert!(x11.xwayland);
        let default = Level::Default.profiles(DESKTOP);
        let plans: Vec<(&str, Plan)> = default.iter().map(|(p, _)| (p.name, p.plan)).collect();
        assert_eq!(
            plans,
            [
                ("software", Plan::Software),
                ("native", Plan::Default),
                ("nogpu", Plan::Weak),
                ("nogpu-x11", Plan::Weak),
                ("4cores", Plan::Weak),
                ("2cores", Plan::Weak),
            ]
        );
        assert_eq!(default[4].0.cores, Some(4));
        assert!(default[0].0.cells(1.0).iter().all(|c| c.runs == 3));
        assert_eq!(Level::Full.profiles(DESKTOP)[1].0.plan, Plan::Full);
        // About 10 minutes for quick, 75 for default (D22.9), each with the
        // figure profile's 1 and 4 (D22.10): here 8.5 and 71 without the
        // X11 profile, 10 and 77 with it; and the software profile's 6.2
        // and 18.6 (D22.12).
        let minutes =
            |level: Level, machine| level.estimate_s(&level.profiles(machine), 1.0) / 60.0;
        let wayland = Machine {
            xwayland: true,
            ..DESKTOP
        };
        for machine in [DESKTOP, wayland] {
            let quick = minutes(Level::Quick, machine);
            assert!((14.0..17.5).contains(&quick), "{quick}");
            let default = minutes(Level::Default, machine);
            assert!((88.0..99.0).contains(&default), "{default}");
        }
        assert!(minutes(Level::Full, DESKTOP) > 300.0, "hours");
    }

    #[test]
    fn a_machine_skips_what_it_cannot_run_and_says_why() {
        // A Wayland session with XWayland runs the X11 profile.
        let wayland = Machine {
            xwayland: true,
            ..DESKTOP
        };
        assert_eq!(
            names(Level::Quick, wayland),
            (vec!["software", "native", "nogpu", "nogpu-x11"], vec![])
        );
        // No GPU adapter: the native profile is the software one.
        let software = Machine {
            software: true,
            ..wayland
        };
        assert_eq!(
            names(Level::Quick, software),
            (vec!["software", "native", "nogpu-x11"], vec!["nogpu"])
        );
        // Two cores: the 4-core profile is skipped.
        let small = Machine {
            cores: 2,
            ..DESKTOP
        };
        assert_eq!(
            names(Level::Default, small),
            (
                vec!["software", "native", "nogpu", "2cores"],
                vec!["nogpu-x11", "4cores"]
            )
        );
        let one = Machine {
            cores: 1,
            ..DESKTOP
        };
        let profiles = Level::Default.profiles(one);
        assert_eq!(profiles[5].1.as_deref(), Some("this machine has 1 core"));
        // Headless: no X11 profile, even in a Wayland session.
        let headless = Machine {
            headless: true,
            ..wayland
        };
        assert_eq!(
            names(Level::Quick, headless),
            (vec!["software", "native", "nogpu"], vec!["nogpu-x11"])
        );
    }

    #[test]
    fn a_profile_runs_its_plan_with_the_display_it_was_given() {
        let display = Display::decide(
            None,
            None,
            Ok(Measured {
                hz: Some(144.0),
                factor: Some(1.25),
                backend: Some("wgpu".to_owned()),
            }),
        );
        let profiles = Level::Quick.profiles(DESKTOP);
        let args = profiles[2].0.args(&display, Path::new("o/nogpu"), None);
        assert_eq!(
            args.join(" "),
            "run --plan quick --backend tiny-skia --budgets weak --no-also --refresh-hz 144 \
             --native-factor 1.25 --scenario pan-x,zoom-lines --workload trace --out o/nogpu"
        );
        // The software profile ignores the display: headless at 60 Hz, and
        // only it writes frames.
        let frames = (PathBuf::from("f"), 10);
        let args = profiles[0]
            .0
            .args(&display, Path::new("o/software"), Some(&frames));
        assert_eq!(
            args.join(" "),
            "run --plan software --backend tiny-skia --budgets weak --no-also --refresh-hz 60 \
             --native-factor 2 --runs 1 --headless --frames f --every 10 --out o/software"
        );
        assert!(
            !profiles[1]
                .0
                .args(&display, Path::new("o/native"), Some(&frames))
                .contains(&"--frames".to_owned())
        );
        let headless = Display::decide(None, None, Err("no display".to_owned()));
        let args = profiles[1].0.args(&headless, Path::new("o/native"), None);
        assert_eq!(
            args.join(" "),
            "run --plan quick --backend auto --budgets desktop --no-also --refresh-hz 60 \
             --native-factor 1 --headless --out o/native"
        );
        // Each round-trips through run's parser.
        let Ok(crate::cli::Command::Widget(command)) = crate::cli::Command::parse(&args) else {
            panic!("a widget command");
        };
        let widget::Command::Run(options) = *command else {
            panic!("a run");
        };
        assert!(options.headless && !options.also);
        assert_eq!(options.budgets, Some(rule::Profile::Desktop));
    }

    #[test]
    fn each_platform_limits_the_cores_its_own_way() {
        assert_eq!(mask(4), "F");
        assert_eq!(mask(2), "3");
        assert_eq!(
            Limit::Taskset(4).describe(),
            "pinned to 4 cores (taskset -c 0-3)"
        );
        assert_eq!(
            Limit::Affinity(2).describe(),
            "pinned to 2 cores (start /AFFINITY 3)"
        );
        assert_eq!(
            Limit::Threads(4).describe(),
            "not pinned; the widget's render pool set to 3 threads (--threads), as on 4 cores"
        );
        let args = ["run".to_owned()];
        let pinned = Limit::Affinity(2).command(Path::new("bench"), &args);
        let shown: Vec<String> = pinned
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(pinned.get_program(), "cmd");
        assert_eq!(
            shown,
            [
                "/C",
                "start",
                "",
                "/B",
                "/WAIT",
                "/AFFINITY",
                "3",
                "bench",
                "run"
            ]
        );
        let pooled = Limit::Threads(2).command(Path::new("bench"), &args);
        let shown: Vec<String> = pooled
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(shown, ["run", "--threads", "1"]);
    }

    #[test]
    fn the_x11_profile_runs_only_in_a_linux_wayland_session_with_xwayland() {
        let wayland = Some(OsStr::new("wayland-1"));
        let x = Some(OsStr::new(":0"));
        assert!(xwayland_on("linux", wayland, x));
        assert!(!xwayland_on("linux", None, x), "an X11 session");
        assert!(
            !xwayland_on("linux", wayland, None),
            "Wayland without XWayland"
        );
        assert!(!xwayland_on("linux", None, None), "no display");
        assert!(
            !xwayland_on("linux", Some(OsStr::new("")), x),
            "an empty WAYLAND_DISPLAY is unset to winit"
        );
        assert!(!xwayland_on("linux", wayland, Some(OsStr::new(""))));
        for os in ["windows", "macos", "freebsd"] {
            assert!(!xwayland_on(os, wayland, x), "{os}");
        }
    }
}
