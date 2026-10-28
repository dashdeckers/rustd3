//! Hand-parsed arguments: [`Command`] and the [`Options`] `run` and
//! `child` share.

use std::fmt;
use std::path::PathBuf;

use rustd3_iced::plot::{Caching, Render};

use crate::plan::{Cell, Plan, Selection};
use crate::scenario::Scenario;
use crate::workload::Workload;

/// What the process does.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Run a plan: replays, one child process per cell and run, report.
    Run(Options),
    /// One scenario × workload × factor × run (internal).
    Child(Child),
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
    /// Rebuild the CSVs and `report.md` of a run's directory.
    Report {
        /// The run's directory.
        dir: PathBuf,
    },
    /// Print scenarios, workloads and plans.
    List(Options),
    /// Measure the raster variants on the plan's workloads (CPU only).
    Raster(Options),
    /// Print the items earlier phases asked to be measured.
    Also,
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
    /// `--replay FILE`: replaces every selected scenario's input.
    pub replay: Option<PathBuf>,
    /// `--data-scale`: the plan's data size times this.
    pub data_scale: Option<f64>,
    /// `--out`: where the run's directory goes.
    pub out: PathBuf,
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
}

/// What `--help` prints.
pub const HELP: &str = "\
rustd3-bench: the full-frame benchmark of rustd3's Plot widget.

The user's procedure (Pop!_OS, real hardware):
  1. Tell the agent the display's refresh rate and scale factor; read and
     confirm the operational readings (`rustd3-bench list` prints them).
  2. cargo build --release -p rustd3-bench
  3. Plug in power, `system76-power profile performance`, close other
     GPU-heavy apps; leave the desktop's compositor as it is (recorded).
  4. target/release/rustd3-bench run --plan default --refresh-hz <HZ>
       --native-factor <F> --out ~/bench-out
     Windows open and close for about 50 minutes; do not touch the mouse
     or keyboard.
  5. Optional: the same with --backend tiny-skia (report-only); `record`
     then `run --scenario zoom-pixels --replay FILE` with your own
     touchpad; --render inline to see the pure synchronous path.
  6. Send back ~/bench-out/<stamp>/ (tar czf bench.tgz -C ~/bench-out
     <stamp>); at least report.md, summary.csv, meta.json.

Commands:
  run     [--plan smoke|quick|default|full] [--scenario A,B] [--workload A,B]
          [--factor 1,1.25] [--runs N] [--refresh-hz HZ] [--native-factor F]
          [--backend wgpu|tiny-skia]
          [--present-mode vsync|no_vsync|immediate|fifo|fifo_relaxed|mailbox]
          [--headless] [--readback every|last] [--render auto|inline|worker]
          [--caching on|off] [--replay FILE] [--data-scale F] [--out DIR]
          `run` is the default command. Windowed, --refresh-hz and
          --native-factor are required; headless, they default to 60 and 2,
          and --headless without --plan is --plan smoke. --scenario,
          --workload and --factor restrict any plan (smoke runs at the
          listed factors instead of 2).
  child   (internal: one scenario × workload × factor × run)
  replay  --out DIR       write the generated replays
  record  --out FILE      record your own input in the trace window as a
                          replay (relative to the first event; Escape stops)
  report  DIR             rebuild the CSVs and report.md of a run
  list    [run's flags]   scenarios, workloads, plans and a plan's cells
  raster  [--plan P] [--workload A,B] [--data-scale F] [--out DIR]
          the raster variants (envelope fills, marker splatting) against
          the cells as drawn, CPU only, into DIR/raster.csv
  also                    the items earlier phases asked to be measured
                          (the report's \"Also measured\"; release)
";

impl Command {
    /// Parses the arguments after the program name.
    pub fn parse(args: &[String]) -> Result<Command, Error> {
        if args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "help")
        {
            return Ok(Command::Help);
        }
        let (command, rest) = match args.first().map(String::as_str) {
            Some(first) if !first.starts_with("--") => (first, &args[1..]),
            Some(_) | None => ("run", args),
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
            "replay" => Ok(Command::Replay {
                out: single_out(rest)?,
            }),
            "record" => Ok(Command::Record {
                out: single_out(rest)?,
            }),
            "report" => match rest {
                [dir] if !dir.starts_with("--") => Ok(Command::Report {
                    dir: PathBuf::from(dir),
                }),
                [first, ..] => Err(Error::Stray(first.clone())),
                [] => Err(Error::Missing("report DIR".to_owned())),
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
                        _ => return Err(bad()),
                    };
                }
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
                        _ => return Err(bad()),
                    };
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
            present_mode: None,
            headless: false,
            readback: Readback::default(),
            render: Render::default(),
            caching: Caching::default(),
            replay: None,
            data_scale: None,
            out: PathBuf::from("bench-out"),
        }
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
    use crate::plan::Plan;
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
                    --render worker --caching off --replay in.jsonl --data-scale 0.5 --out o";
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
        assert_eq!(options.caching, Caching::Off);
        assert_eq!(options.replay, Some(PathBuf::from("in.jsonl")));
        assert_eq!(options.data_scale, Some(0.5));
        assert_eq!(options.out, PathBuf::from("o"));
        // `run` is the default command; headless without a plan is smoke.
        let Ok(Command::Run(options)) = Command::parse(&args("--headless --workload trace")) else {
            panic!("a run");
        };
        assert_eq!(options.plan(), Plan::Smoke);
        for line in [
            "replay --out d",
            "record --out f.jsonl",
            "report d",
            "list",
            "raster --plan default --out target/bench-raster",
            "also",
            "--help",
        ] {
            assert!(Command::parse(&args(line)).is_ok(), "{line}");
        }
        // A child round-trips through the flags `run` gives it.
        let line = "child --plan smoke --headless --cell pan-x__trace__f1.25 --run 2 --dir d";
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
    }

    #[test]
    fn rejects_unknown_flags() {
        assert_eq!(
            Command::parse(&args("run --threads 4")),
            Err(Error::Unknown("--threads".to_owned()))
        );
        assert_eq!(
            Command::parse(&args("run --caching verify")),
            Err(Error::Value {
                flag: "--caching".to_owned(),
                value: "verify".to_owned()
            })
        );
        assert!(Command::parse(&args("run --plan")).is_err());
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
