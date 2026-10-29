//! Hand-parsed arguments: [`Command`] and the [`Options`] `run` and
//! `child` share.

use std::fmt;
use std::path::PathBuf;

use rustd3_iced::plot::{Caching, Render};

use crate::level::{Level, Request};
use crate::plan::{Cell, Plan, Selection};
use crate::rule;
use crate::scenario::Scenario;
use crate::workload::Workload;

/// What the process does.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// A level, the one entry point (D22.9): the display measurement, then
    /// the level's profiles, each a `run`.
    Level(Request),
    /// Run a plan: replays, one child process per cell and run, report.
    Run(Options),
    /// One scenario × workload × factor × run (internal).
    Child(Child),
    /// One figure of the figure profile, timed in its own process
    /// (internal).
    FigureChild {
        /// The level whose plan holds it.
        level: Level,
        /// Its name.
        subject: String,
    },
    /// Write the generated replays into a directory.
    Replay {
        /// The directory.
        out: PathBuf,
    },
    /// Record real input in the `trace` window as a replay file.
    Record {
        /// The file.
        out: PathBuf,
    },
    /// Rebuild and print the report of a level's directory or archive (or
    /// of one run's directory).
    Report {
        /// The directory or archive.
        path: PathBuf,
    },
    /// Two results side by side.
    Compare {
        /// A and B: directories or archives.
        paths: [PathBuf; 2],
    },
    /// Print scenarios, workloads and plans.
    List(Options),
    /// Measure the raster variants on the plan's workloads (CPU only).
    Raster(Options),
    /// Print the items earlier phases asked to be measured.
    Also,
    /// D22.8's plain-iced reproduction: moving images, continuous redraws.
    ReproSoftware {
        /// `--out FILE`; `None` is `bench-results/repro-<UTC stamp>.txt`.
        out: Option<PathBuf>,
        /// `--single`: one window-sized image in place of the grid of 40.
        single: bool,
    },
    /// Print the user's procedure and the flags.
    Help,
}

/// The flags of `run` (and `list`).
#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    /// `--plan`; `None` is `default` windowed and `smoke` headless.
    pub plan: Option<Plan>,
    /// `--scenario`, `--workload`, `--factor`, `--runs`.
    pub selection: Selection,
    /// `--refresh-hz`: required windowed, 60 headless.
    pub refresh_hz: Option<f64>,
    /// `--native-factor`: the display's scale factor; required windowed,
    /// 2 headless.
    pub native: Option<f32>,
    /// `--backend`.
    pub backend: Backend,
    /// `--budgets`: the budgets the report holds the cells to; `None` is
    /// the plan's (`weak` for the weak plan, else the desktop's).
    pub budgets: Option<rule::Profile>,
    /// False with `--no-also`: the report's "Also measured" items are not
    /// measured after the runs.
    pub also: bool,
    /// `--present-mode`, passed to iced as `ICED_PRESENT_MODE`.
    pub present_mode: Option<PresentMode>,
    /// `--headless`.
    pub headless: bool,
    /// `--readback`.
    pub readback: Readback,
    /// `--render`.
    pub render: Render,
    /// `--caching`.
    pub caching: Caching,
    /// `--threads`: the widget's render pool; `None` is its default (the
    /// available parallelism less one, 1 to 8).
    pub threads: Option<usize>,
    /// `--replay FILE`: replaces every selected scenario's input.
    pub replay: Option<PathBuf>,
    /// `--data-scale`: the plan's data size times this.
    pub data_scale: Option<f64>,
    /// `--out`: where the run's directory goes.
    pub out: PathBuf,
    /// `--cpu-limit TEXT`: how the cores the run may use were limited
    /// (pinned, or the render pool set), as `suite` says; recorded only.
    pub cpu_limit: Option<String>,
    /// `--xwayland`: the runs' windows open on X11 through XWayland in a
    /// Linux Wayland session (`WAYLAND_DISPLAY` removed from their
    /// environment); D22.8's report-only profile, which `suite` runs.
    pub xwayland: bool,
}

/// One run of one cell, as `run` spawns it.
#[derive(Debug, Clone, PartialEq)]
pub struct Child {
    /// The parent's options, resolved (refresh, factor, data scale).
    pub options: Options,
    /// The cell (its `runs` is 1).
    pub cell: Cell,
    /// Which run, from 1.
    pub run: u32,
    /// The run's directory.
    pub dir: PathBuf,
}

/// iced's renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Backend {
    /// The GPU renderer.
    #[default]
    Wgpu,
    /// The software renderer.
    TinySkia,
    /// Whichever iced chooses (`ICED_BACKEND` unset): wgpu, or tiny-skia
    /// on a machine without a GPU adapter, where an explicit `wgpu` opens
    /// no window at all.
    Auto,
}

/// A wgpu present mode, as `ICED_PRESENT_MODE` names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentMode {
    /// `AutoVsync`.
    Vsync,
    /// `AutoNoVsync`.
    NoVsync,
    /// `Immediate`.
    Immediate,
    /// `Fifo`.
    Fifo,
    /// `FifoRelaxed`.
    FifoRelaxed,
    /// `Mailbox`.
    Mailbox,
}

/// When the headless loop reads pixels back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Readback {
    /// Every frame: headless renderers rasterize only when read back.
    #[default]
    Every,
    /// The last frame only.
    Last,
}

/// Why the arguments are rejected.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A flag no command takes.
    Unknown(String),
    /// A flag without its value.
    Missing(String),
    /// A value the flag does not take.
    Value {
        /// The flag.
        flag: String,
        /// The rejected value.
        value: String,
    },
    /// An argument that is not a flag where a flag goes.
    Stray(String),
    /// A windowed run without the display's refresh rate or factor.
    Windowed(&'static str),
    /// A flag of the old `suite` or `run.sh`, which the levels replace.
    Gone(String),
}

/// What `--help` prints.
pub const HELP: &str = "\
rustd3-bench: the benchmark of rustd3's Plot widget.

  cargo rustd3-bench [--quick | --default | --full] [--figure-only]
                     [--out DIR] [--refresh-hz HZ] [--native-factor F]

From a checkout of rustd3 (the cargo alias builds the release bench and
runs it; dev/bench/run.sh does the same). It runs on any machine and
platform: it prints what it will run and an estimate of the time, measures
the display (one window, a few seconds), takes three single-threaded CPU
timings, then runs the level's profiles in windows that open and close by
themselves; keep hands off the mouse and keyboard.
  --quick     about 10 minutes: what a performance issue needs. The cells
              hover, pan-x and zoom-lines on trace and scatter100k and
              drag-resize on trace, 3 runs each, on the native backend; pan-x
              and zoom-lines on trace on the software renderer (and again on
              X11 in a Linux Wayland session). Report-only.
  --default   about 75 minutes, the default: the registered verdict run
              (D22.1's rule outcome) and the weak profiles (no GPU, 4 cores,
              2 cores; D22.4).
  --full      hours: every scenario x workload x factor on the native
              backend, the weak profiles and every `also` measurement.
  --figure-only
              the plotting library alone: the CPU reference and the figure
              profile at the level, no display measurement and no window.
  --out DIR   where results go (default bench-results).
  --refresh-hz HZ, --native-factor F
              the display's refresh rate and scale factor, for displays that
              cannot be measured (by default the display measurement gives
              both; an unpaced display, such as a VM's, runs at 60 Hz).
Without a display it runs headless; without a GPU the native backend is
the software renderer; with too few cores a profile is skipped; a run that
fails is recorded and the level goes on. It writes
bench-results/<UTC stamp>-<level>/ and packs it, after every profile, into
bench-results/rustd3-bench-<level>-<UTC stamp>.tar.gz, printed last: the one
file to attach to an issue (PRIVACY.md inside lists what it holds; no host
or user name, no home path).

Advanced (developers):
  run     [--plan smoke|display|quick|default|weak|focus|full] [--scenario A,B]
          [--workload A,B] [--factor 1,1.25] [--runs N] [--refresh-hz HZ]
          [--native-factor F] [--backend wgpu|tiny-skia|auto]
          [--budgets desktop|weak]
          [--present-mode vsync|no_vsync|immediate|fifo|fifo_relaxed|mailbox]
          [--headless] [--readback every|last] [--render auto|inline|worker]
          [--caching on|off|verify] [--threads N] [--replay FILE]
          [--data-scale F] [--cpu-limit TEXT] [--xwayland] [--no-also]
          [--out DIR]
          one plan into DIR/<UTC stamp>/ (default bench-out): replays, a
          process per cell and run, the CSVs and report.md, then the \"Also
          measured\" items unless --no-also. Windowed, --refresh-hz and
          --native-factor are required; headless, they default to 60 and 2,
          and --headless without --plan is --plan smoke. --scenario,
          --workload and --factor restrict any plan (smoke runs at the
          listed factors instead of 2). `weak` is D22.4's plan, held to the
          weak budgets (as --budgets weak holds any plan); `focus` is D22.5's
          report-only plan. --backend auto leaves the choice to iced (wgpu,
          or tiny-skia without a GPU adapter). --threads sets the widget's
          render pool (1 to 64; default the available parallelism less one,
          1 to 8); --caching verify renders every kept image again and
          reports what differs. --xwayland (Linux, a Wayland session with
          XWayland) removes WAYLAND_DISPLAY from the runs, whose windows then
          open on X11 (D22.8). --cpu-limit records how the cores were limited.
  list    [run's flags]   scenarios, workloads, plans and a plan's cells
  raster  [--plan P] [--workload A,B] [--data-scale F] [--out DIR]
          the raster variants (envelope fills, marker splatting) against
          the cells as drawn, CPU only, into DIR/raster.csv
  also                    the items earlier phases asked to be measured
                          (the report's \"Also measured\"; release)
  replay  --out DIR       write the generated replays
  record  --out FILE      record your own input in the trace window as a
                          replay (relative to the first event; Escape stops)
  report  PATH            rebuild and print the report of a level's directory
                          or archive (each profile's from its raw records, in
                          a temporary directory for an archive), or of one
                          run's directory; a directory from before the levels
                          is read as it is
  compare A B             two results (directories or archives, from before
                          the levels too) side by side: the machines, then
                          per profile and cell frame p50 and p99, latency
                          p95, preview share, drops, UI frame p99 and CPU s/s
                          of A and B and B/A
  repro-software [--single] [--out FILE]
          D22.8's plain-iced reproduction for upstream, no rustd3: 40
          images (--single: one window-sized image, so that drawing is
          cheap and presentation shows) moving 1 px a frame in a
          1600 x 1000 window, redrawn every frame; the backend from
          ICED_BACKEND; prints and writes
          (default bench-results/repro-<UTC stamp>.txt) the frame
          intervals and CPU use; env -u WAYLAND_DISPLAY for X11.
          dev/bench/repro-software.sh runs both backends on Wayland and
          on X11
  child   (internal: one scenario x workload x factor x run)
  figure-child (internal: one figure of the figure profile)
";

impl Command {
    /// Parses the arguments after the program name: a level when they
    /// start with a flag or are empty, else a command.
    pub fn parse(args: &[String]) -> Result<Command, Error> {
        if args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "help")
        {
            return Ok(Command::Help);
        }
        let (command, rest) = match args.first().map(String::as_str) {
            Some(first) if !first.starts_with("--") => (first, &args[1..]),
            Some(_) | None => return Request::parse(args).map(Command::Level),
        };
        match command {
            "run" => Ok(Command::Run(Options::parse(rest, &[])?)),
            "list" => Ok(Command::List(Options::parse(rest, &[])?)),
            "raster" => Ok(Command::Raster(Options::parse(rest, &[])?)),
            "also" => match rest {
                [] => Ok(Command::Also),
                [first, ..] => Err(Error::Stray(first.clone())),
            },
            "child" => Child::parse(rest).map(Command::Child),
            "figure-child" => match rest {
                [l, level, s, subject] if l == "--level" && s == "--subject" => {
                    Ok(Command::FigureChild {
                        level: Level::parse(level).ok_or_else(|| Error::Value {
                            flag: l.clone(),
                            value: level.clone(),
                        })?,
                        subject: subject.clone(),
                    })
                }
                _ => Err(Error::Missing(
                    "figure-child --level L --subject NAME".to_owned(),
                )),
            },
            "replay" => Ok(Command::Replay {
                out: single_out(rest)?,
            }),
            "record" => Ok(Command::Record {
                out: single_out(rest)?,
            }),
            "repro-software" => {
                let single = rest.iter().any(|a| a == "--single");
                let rest: Vec<String> = rest.iter().filter(|a| *a != "--single").cloned().collect();
                Ok(Command::ReproSoftware {
                    out: if rest.is_empty() {
                        None
                    } else {
                        Some(single_out(&rest)?)
                    },
                    single,
                })
            }
            "report" => match rest {
                [path] if !path.starts_with("--") => Ok(Command::Report {
                    path: PathBuf::from(path),
                }),
                [first, ..] => Err(Error::Stray(first.clone())),
                [] => Err(Error::Missing("report PATH".to_owned())),
            },
            "compare" => match rest {
                [a, b] if !a.starts_with("--") && !b.starts_with("--") => Ok(Command::Compare {
                    paths: [PathBuf::from(a), PathBuf::from(b)],
                }),
                [_, _, third, ..] => Err(Error::Stray(third.clone())),
                [first, ..] if first.starts_with("--") => Err(Error::Unknown(first.clone())),
                [_, second] => Err(Error::Unknown(second.clone())),
                [_] | [] => Err(Error::Missing("compare A B".to_owned())),
            },
            other => Err(Error::Stray(other.to_owned())),
        }
    }
}

impl Options {
    /// Parses `run`'s flags, and `extra` flags with values into
    /// `(flag, value)` pairs the caller reads.
    fn parse(args: &[String], extra: &[&str]) -> Result<Options, Error> {
        Ok(Options::parse_with(args, extra)?.0)
    }

    /// As [`Options::parse`], returning the `extra` pairs too.
    fn parse_with(
        args: &[String],
        extra: &[&str],
    ) -> Result<(Options, Vec<(String, String)>), Error> {
        let mut options = Options::default();
        let mut pairs = Vec::new();
        let mut args = args.iter();
        while let Some(flag) = args.next() {
            if !flag.starts_with("--") {
                return Err(Error::Stray(flag.clone()));
            }
            if flag == "--headless" {
                options.headless = true;
                continue;
            }
            if flag == "--xwayland" {
                options.xwayland = true;
                continue;
            }
            if flag == "--no-also" {
                options.also = false;
                continue;
            }
            let value = args.next().ok_or_else(|| Error::Missing(flag.clone()))?;
            let bad = || Error::Value {
                flag: flag.clone(),
                value: value.clone(),
            };
            match flag.as_str() {
                "--plan" => options.plan = Some(Plan::parse(value).ok_or_else(bad)?),
                "--scenario" => {
                    options.selection.scenarios =
                        Some(list(value, Scenario::parse).ok_or_else(bad)?);
                }
                "--workload" => {
                    options.selection.workloads =
                        Some(list(value, Workload::parse).ok_or_else(bad)?);
                }
                "--factor" => {
                    options.selection.factors = Some(list(value, factor).ok_or_else(bad)?);
                }
                "--runs" => {
                    let runs = value.parse::<u32>().ok().filter(|&n| n > 0);
                    options.selection.runs = Some(runs.ok_or_else(bad)?);
                }
                "--refresh-hz" => {
                    let hz = value
                        .parse::<f64>()
                        .ok()
                        .filter(|hz| hz.is_finite() && *hz > 0.0);
                    options.refresh_hz = Some(hz.ok_or_else(bad)?);
                }
                "--native-factor" => options.native = Some(factor(value).ok_or_else(bad)?),
                "--backend" => {
                    options.backend = match value.as_str() {
                        "wgpu" => Backend::Wgpu,
                        "tiny-skia" => Backend::TinySkia,
                        "auto" => Backend::Auto,
                        _ => return Err(bad()),
                    };
                }
                "--budgets" => options.budgets = Some(rule::Profile::parse(value).ok_or_else(bad)?),
                "--present-mode" => {
                    options.present_mode = Some(PresentMode::parse(value).ok_or_else(bad)?);
                }
                "--readback" => {
                    options.readback = match value.as_str() {
                        "every" => Readback::Every,
                        "last" => Readback::Last,
                        _ => return Err(bad()),
                    };
                }
                "--render" => {
                    options.render = match value.as_str() {
                        "auto" => Render::Auto,
                        "inline" => Render::Inline,
                        "worker" => Render::Worker,
                        _ => return Err(bad()),
                    };
                }
                "--caching" => {
                    options.caching = match value.as_str() {
                        "on" => Caching::On,
                        "off" => Caching::Off,
                        "verify" => Caching::Verify,
                        _ => return Err(bad()),
                    };
                }
                "--threads" => {
                    let threads = value.parse::<usize>().ok();
                    let threads = threads.filter(|n| (1..=64).contains(n));
                    options.threads = Some(threads.ok_or_else(bad)?);
                }
                "--replay" => options.replay = Some(PathBuf::from(value)),
                "--data-scale" => {
                    let scale = value
                        .parse::<f64>()
                        .ok()
                        .filter(|s| s.is_finite() && *s > 0.0);
                    options.data_scale = Some(scale.ok_or_else(bad)?);
                }
                "--out" => options.out = PathBuf::from(value),
                "--cpu-limit" => options.cpu_limit = Some(value.clone()),
                other if extra.contains(&other) => pairs.push((other.to_owned(), value.clone())),
                other => return Err(Error::Unknown(other.to_owned())),
            }
        }
        Ok((options, pairs))
    }

    /// The plan: `--plan`, else `smoke` headless and `default` windowed.
    pub fn plan(&self) -> Plan {
        self.plan.unwrap_or(if self.headless {
            Plan::Smoke
        } else {
            Plan::Default
        })
    }

    /// The display's refresh rate: `--refresh-hz`, 60 headless; a windowed
    /// run without it is an error.
    pub fn refresh_hz(&self) -> Result<f64, Error> {
        match (self.refresh_hz, self.headless) {
            (Some(hz), _) => Ok(hz),
            (None, true) => Ok(60.0),
            (None, false) => Err(Error::Windowed("--refresh-hz")),
        }
    }

    /// The display's scale factor: `--native-factor`, 2 headless; a
    /// windowed run without it is an error.
    pub fn native(&self) -> Result<f32, Error> {
        match (self.native, self.headless) {
            (Some(factor), _) => Ok(factor),
            (None, true) => Ok(2.0),
            (None, false) => Err(Error::Windowed("--native-factor")),
        }
    }

    /// The data size: the plan's times `--data-scale`.
    pub fn data_scale(&self) -> f64 {
        self.plan().data_scale() * self.data_scale.unwrap_or(1.0)
    }

    /// The flags that reproduce these options for a child, with the
    /// refresh rate, factor and data scale resolved.
    pub fn to_args(&self, refresh_hz: f64, native: f32) -> Vec<String> {
        let mut args = vec![
            "--plan".to_owned(),
            self.plan().name().to_owned(),
            "--refresh-hz".to_owned(),
            refresh_hz.to_string(),
            "--native-factor".to_owned(),
            native.to_string(),
            "--backend".to_owned(),
            self.backend.name().to_owned(),
            "--readback".to_owned(),
            match self.readback {
                Readback::Every => "every",
                Readback::Last => "last",
            }
            .to_owned(),
            "--render".to_owned(),
            render_name(self.render).to_owned(),
            "--caching".to_owned(),
            caching_name(self.caching).to_owned(),
            "--out".to_owned(),
            self.out.display().to_string(),
        ];
        if let Some(scale) = self.data_scale {
            args.extend(["--data-scale".to_owned(), scale.to_string()]);
        }
        if let Some(budgets) = self.budgets {
            args.extend(["--budgets".to_owned(), budgets.name().to_owned()]);
        }
        if let Some(threads) = self.threads {
            args.extend(["--threads".to_owned(), threads.to_string()]);
        }
        if let Some(mode) = self.present_mode {
            args.extend(["--present-mode".to_owned(), mode.name().to_owned()]);
        }
        if self.headless {
            args.push("--headless".to_owned());
        }
        args
    }
}

impl Default for Options {
    fn default() -> Options {
        Options {
            plan: None,
            selection: Selection::default(),
            refresh_hz: None,
            native: None,
            backend: Backend::default(),
            budgets: None,
            also: true,
            present_mode: None,
            headless: false,
            readback: Readback::default(),
            render: Render::default(),
            caching: Caching::default(),
            threads: None,
            replay: None,
            data_scale: None,
            out: PathBuf::from("bench-out"),
            cpu_limit: None,
            xwayland: false,
        }
    }
}

impl Request {
    /// Parses a level's flags: at most one of `--quick`, `--default` and
    /// `--full` (`--default` when none), `--figure-only`, `--out DIR`,
    /// `--refresh-hz HZ`, `--native-factor F` and the hidden `--dry-run`.
    fn parse(args: &[String]) -> Result<Request, Error> {
        let mut request = Request {
            level: Level::Default,
            out: PathBuf::from("bench-results"),
            refresh_hz: None,
            native: None,
            figure_only: false,
            dry_run: false,
        };
        let mut named: Option<&str> = None;
        let mut args = args.iter();
        while let Some(flag) = args.next() {
            let level = match flag.as_str() {
                "--quick" => Some(Level::Quick),
                "--default" => Some(Level::Default),
                "--full" => Some(Level::Full),
                _ => None,
            };
            if let Some(level) = level {
                if let Some(first) = named.filter(|first| first != flag) {
                    return Err(Error::Value {
                        flag: flag.clone(),
                        value: format!("with {first}"),
                    });
                }
                named = Some(flag);
                request.level = level;
                continue;
            }
            match flag.as_str() {
                "--dry-run" => request.dry_run = true,
                "--figure-only" => request.figure_only = true,
                "--refresh-hz" | "--native-factor" | "--out" => {
                    let value = args.next().ok_or_else(|| Error::Missing(flag.clone()))?;
                    let bad = || Error::Value {
                        flag: flag.clone(),
                        value: value.clone(),
                    };
                    match flag.as_str() {
                        "--refresh-hz" => {
                            let hz = value
                                .parse::<f64>()
                                .ok()
                                .filter(|hz| hz.is_finite() && *hz > 0.0);
                            request.refresh_hz = Some(hz.ok_or_else(bad)?);
                        }
                        "--native-factor" => request.native = Some(factor(value).ok_or_else(bad)?),
                        _ => request.out = PathBuf::from(value),
                    }
                }
                "--smoke-only" | "--focus" => return Err(Error::Gone(flag.clone())),
                other if other.starts_with("--") => return Err(Error::Unknown(other.to_owned())),
                other => return Err(Error::Stray(other.to_owned())),
            }
        }
        Ok(request)
    }
}

impl Child {
    /// Parses `child`'s flags: `run`'s and `--cell KEY --run K --dir DIR`.
    fn parse(args: &[String]) -> Result<Child, Error> {
        let (options, pairs) = Options::parse_with(args, &["--cell", "--run", "--dir"])?;
        let get = |name: &str| {
            pairs
                .iter()
                .find(|(flag, _)| flag == name)
                .map(|(_, value)| value.clone())
                .ok_or_else(|| Error::Missing(name.to_owned()))
        };
        let key = get("--cell")?;
        let cell = cell(&key).ok_or(Error::Value {
            flag: "--cell".to_owned(),
            value: key,
        })?;
        let run = get("--run")?;
        let run = run.parse().map_err(|_| Error::Value {
            flag: "--run".to_owned(),
            value: run.clone(),
        })?;
        Ok(Child {
            options,
            cell,
            run,
            dir: PathBuf::from(get("--dir")?),
        })
    }

    /// The flags that start this child.
    pub fn to_args(&self, refresh_hz: f64, native: f32) -> Vec<String> {
        let mut args = vec!["child".to_owned()];
        args.extend(self.options.to_args(refresh_hz, native));
        if let Some(replay) = &self.options.replay {
            args.extend(["--replay".to_owned(), replay.display().to_string()]);
        }
        args.extend([
            "--cell".to_owned(),
            self.cell.key(),
            "--run".to_owned(),
            self.run.to_string(),
            "--dir".to_owned(),
            self.dir.display().to_string(),
        ]);
        args
    }
}

impl Backend {
    /// The name `ICED_BACKEND` takes.
    pub fn name(self) -> &'static str {
        match self {
            Backend::Wgpu => "wgpu",
            Backend::TinySkia => "tiny-skia",
            Backend::Auto => "auto",
        }
    }

    /// What `ICED_BACKEND` is set to; `None` (unset) for `auto`.
    pub fn iced(self) -> Option<&'static str> {
        match self {
            Backend::Wgpu | Backend::TinySkia => Some(self.name()),
            Backend::Auto => None,
        }
    }
}

impl PresentMode {
    /// Every mode.
    const ALL: [PresentMode; 6] = [
        PresentMode::Vsync,
        PresentMode::NoVsync,
        PresentMode::Immediate,
        PresentMode::Fifo,
        PresentMode::FifoRelaxed,
        PresentMode::Mailbox,
    ];

    /// The name `ICED_PRESENT_MODE` takes.
    pub fn name(self) -> &'static str {
        match self {
            PresentMode::Vsync => "vsync",
            PresentMode::NoVsync => "no_vsync",
            PresentMode::Immediate => "immediate",
            PresentMode::Fifo => "fifo",
            PresentMode::FifoRelaxed => "fifo_relaxed",
            PresentMode::Mailbox => "mailbox",
        }
    }

    fn parse(name: &str) -> Option<PresentMode> {
        PresentMode::ALL.into_iter().find(|m| m.name() == name)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Unknown(flag) => write!(f, "unknown flag {flag} (see --help)"),
            Error::Missing(flag) => write!(f, "{flag} needs a value"),
            Error::Value { flag, value } => write!(f, "{flag} does not take {value:?}"),
            Error::Stray(arg) => write!(f, "unexpected argument {arg:?} (see --help)"),
            Error::Gone(flag) => write!(
                f,
                "{flag} is gone: the levels --quick, --default and --full replace the suite \
                 (dev/bench/README.md); the focus plan stays as `rustd3-bench run --plan focus`"
            ),
            Error::Windowed(flag) => write!(
                f,
                "a windowed run needs {flag}: the display's refresh rate (--refresh-hz, e.g. 144) \
                 and scale factor (--native-factor, e.g. 1.25) are not guessed; \
                 add --headless for the smoke run"
            ),
        }
    }
}

impl std::error::Error for Error {}

/// `render`'s flag value.
pub fn render_name(render: Render) -> &'static str {
    match render {
        Render::Auto => "auto",
        Render::Inline => "inline",
        Render::Worker => "worker",
    }
}

/// `caching`'s flag value.
pub fn caching_name(caching: Caching) -> &'static str {
    match caching {
        Caching::On => "on",
        Caching::Off => "off",
        Caching::Verify => "verify",
    }
}

/// `--out DIR` as a command's only flag.
fn single_out(args: &[String]) -> Result<PathBuf, Error> {
    match args {
        [flag, value] if flag == "--out" => Ok(PathBuf::from(value)),
        [flag] if flag == "--out" => Err(Error::Missing(flag.clone())),
        [first, ..] if first.starts_with("--") && first != "--out" => {
            Err(Error::Unknown(first.clone()))
        }
        [first, ..] => Err(Error::Stray(first.clone())),
        [] => Err(Error::Missing("--out".to_owned())),
    }
}

/// A comma-separated list, every item parsed by `item`.
fn list<T>(value: &str, item: impl Fn(&str) -> Option<T>) -> Option<Vec<T>> {
    value.split(',').map(|s| item(s.trim())).collect()
}

/// A scale factor: finite, from 0.5 to 4.
fn factor(value: &str) -> Option<f32> {
    value
        .parse::<f32>()
        .ok()
        .filter(|f| f.is_finite() && (0.5..=4.0).contains(f))
}

/// The cell a key names (`<scenario>__<workload>__f<factor>`), one run.
pub fn cell(key: &str) -> Option<Cell> {
    let mut parts = key.split("__");
    let scenario = Scenario::parse(parts.next()?)?;
    let workload = Workload::parse(parts.next()?)?;
    let factor = factor(parts.next()?.strip_prefix('f')?)?;
    parts.next().is_none().then_some(Cell {
        scenario,
        workload,
        factor,
        runs: 1,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rustd3_iced::plot::{Caching, Render};

    use super::{Backend, Command, Error, PresentMode, Readback};
    use crate::level::{Level, Request};
    use crate::plan::Plan;
    use crate::rule;
    use crate::scenario::Scenario;
    use crate::workload::Workload;

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn cli_parses_every_documented_flag() {
        let line = "run --plan quick --scenario pan-x,zoom-lines --workload trace,grid16 \
                    --factor 1,1.25 --runs 3 --refresh-hz 144 --native-factor 1.25 \
                    --backend tiny-skia --present-mode fifo_relaxed --headless --readback last \
                    --render worker --caching verify --threads 3 --replay in.jsonl \
                    --data-scale 0.5 --budgets weak --no-also --out o";
        let Ok(Command::Run(options)) = Command::parse(&args(line)) else {
            panic!("a run");
        };
        assert_eq!(options.plan, Some(Plan::Quick));
        assert_eq!(
            options.selection.scenarios,
            Some(vec![Scenario::PanX, Scenario::ZoomLines])
        );
        assert_eq!(
            options.selection.workloads,
            Some(vec![Workload::Trace, Workload::Grid16])
        );
        assert_eq!(options.selection.factors, Some(vec![1.0, 1.25]));
        assert_eq!(options.selection.runs, Some(3));
        assert_eq!(options.refresh_hz, Some(144.0));
        assert_eq!(options.native, Some(1.25));
        assert_eq!(options.backend, Backend::TinySkia);
        assert_eq!(options.present_mode, Some(PresentMode::FifoRelaxed));
        assert!(options.headless);
        assert_eq!(options.readback, Readback::Last);
        assert_eq!(options.render, Render::Worker);
        assert_eq!(options.caching, Caching::Verify);
        assert_eq!(options.threads, Some(3));
        assert_eq!(options.replay, Some(PathBuf::from("in.jsonl")));
        assert_eq!(options.data_scale, Some(0.5));
        assert_eq!(options.budgets, Some(rule::Profile::Weak));
        assert!(!options.also);
        assert_eq!(options.out, PathBuf::from("o"));
        // Headless without a plan is smoke; `auto` leaves the choice to iced.
        let Ok(Command::Run(options)) =
            Command::parse(&args("run --headless --workload trace --backend auto"))
        else {
            panic!("a run");
        };
        assert_eq!(options.plan(), Plan::Smoke);
        assert!(options.also);
        assert_eq!(
            (options.backend, options.backend.iced()),
            (Backend::Auto, None)
        );
        assert_eq!(Backend::TinySkia.iced(), Some("tiny-skia"));
        assert!(Command::parse(&args("run --budgets laptop")).is_err());
        let Ok(Command::Run(limited)) = Command::parse(&args("run --cpu-limit pinned --xwayland"))
        else {
            panic!("a run");
        };
        assert_eq!(limited.cpu_limit.as_deref(), Some("pinned"));
        assert!(limited.xwayland && !options.xwayland);
        for line in [
            "replay --out d",
            "record --out f.jsonl",
            "repro-software",
            "report d",
            "report r.tar.gz",
            "compare a.tar.gz b",
            "list",
            "raster --plan default --out target/bench-raster",
            "also",
            "figure-child --level quick --subject trace",
            "--help",
        ] {
            assert!(Command::parse(&args(line)).is_ok(), "{line}");
        }
        // A child round-trips through the flags `run` gives it.
        let line = "child --plan weak --headless --threads 2 --caching verify \
                    --cell pan-x__trace__f1.25 --run 2 --dir d";
        let Ok(Command::Child(child)) = Command::parse(&args(line)) else {
            panic!("a child");
        };
        assert_eq!(child.cell.key(), "pan-x__trace__f1.25");
        let Ok(Command::Child(again)) = Command::parse(&child.to_args(60.0, 2.0)) else {
            panic!("a child again");
        };
        assert_eq!(
            (again.cell, again.run, &again.dir),
            (child.cell, 2, &child.dir)
        );
        assert_eq!(again.options.refresh_hz, Some(60.0));
        assert!(again.options.headless);
        assert_eq!(
            (
                again.options.plan(),
                again.options.threads,
                again.options.caching
            ),
            (Plan::Weak, Some(2), Caching::Verify)
        );
    }

    #[test]
    fn bare_flags_and_no_command_are_a_level() {
        let level = |line: &str| match Command::parse(&args(line)) {
            Ok(Command::Level(request)) => request,
            other => panic!("{line}: {other:?}"),
        };
        let default = level("");
        assert_eq!(
            default,
            Request {
                level: Level::Default,
                out: PathBuf::from("bench-results"),
                refresh_hz: None,
                native: None,
                figure_only: false,
                dry_run: false,
            }
        );
        assert_eq!(level("--quick").level, Level::Quick);
        assert_eq!(level("--full --full").level, Level::Full);
        let given = level("--quick --out o --refresh-hz 144 --native-factor 1.25 --dry-run");
        assert_eq!(
            (given.out, given.refresh_hz, given.native, given.dry_run),
            (PathBuf::from("o"), Some(144.0), Some(1.25), true)
        );
        assert!(!given.figure_only);
        for line in [
            "--figure-only",
            "--full --figure-only",
            "--figure-only --quick",
        ] {
            assert!(level(line).figure_only, "{line}");
        }
        assert_eq!(level("--figure-only --quick").level, Level::Quick);
        assert_eq!(
            Command::parse(&args("--quick --full")),
            Err(Error::Value {
                flag: "--full".to_owned(),
                value: "with --quick".to_owned()
            })
        );
        assert_eq!(
            Command::parse(&args("--quick --plan default")),
            Err(Error::Unknown("--plan".to_owned()))
        );
        assert!(Command::parse(&args("--refresh-hz 0")).is_err());
        assert!(Command::parse(&args("--quick extra")).is_err());
        // The suite's flags point to the levels.
        for gone in ["--smoke-only", "--focus"] {
            let error = Command::parse(&args(gone)).unwrap_err();
            assert!(
                error.to_string().contains("--quick, --default and --full"),
                "{error}"
            );
        }
        assert!(Command::parse(&args("suite")).is_err(), "suite is gone");
    }

    #[test]
    fn rejects_unknown_flags() {
        assert_eq!(
            Command::parse(&args("run --pool 4")),
            Err(Error::Unknown("--pool".to_owned()))
        );
        for (flag, value) in [
            ("--threads", "0"),
            ("--threads", "65"),
            ("--caching", "always"),
        ] {
            assert_eq!(
                Command::parse(&args(&format!("run {flag} {value}"))),
                Err(Error::Value {
                    flag: flag.to_owned(),
                    value: value.to_owned()
                })
            );
        }
        assert!(Command::parse(&args("run --plan")).is_err());
        assert_eq!(
            Command::parse(&args("repro-software --out r.txt")),
            Ok(Command::ReproSoftware {
                out: Some(PathBuf::from("r.txt")),
                single: false,
            })
        );
        assert_eq!(
            Command::parse(&args("repro-software --single")),
            Ok(Command::ReproSoftware {
                out: None,
                single: true,
            })
        );
        assert!(Command::parse(&args("repro-software --backend wgpu")).is_err());
        assert_eq!(
            Command::parse(&args("compare a")),
            Err(Error::Missing("compare A B".to_owned()))
        );
        assert!(Command::parse(&args("compare a b c")).is_err());
        assert!(Command::parse(&args("compare a --runs")).is_err());
        assert!(Command::parse(&args("run --factor 0")).is_err());
        assert!(Command::parse(&args("run --workload wheel")).is_err());
        assert!(Command::parse(&args("run --workload wheels,label-churn")).is_ok());
        assert!(Command::parse(&args("frobnicate")).is_err());
        // A windowed run needs the display's refresh rate and factor.
        let Ok(Command::Run(options)) = Command::parse(&args("run --plan default")) else {
            panic!("a run");
        };
        assert_eq!(options.refresh_hz(), Err(Error::Windowed("--refresh-hz")));
        assert_eq!(options.native(), Err(Error::Windowed("--native-factor")));
    }
}
