//! The commands of `bench_gestures` and `bench_software`, which need the
//! widget's build: [`Command`] and the [`Options`] a plan's run and its
//! children share.

use std::path::PathBuf;

use rustd3_iced::plot::{Caching, Render};

use crate::cli::{Error, factor};
use crate::plan::{Cell, Plan, Selection};
use crate::scenario::Scenario;
use crate::workload::Workload;
use crate::{Example, rule};

/// A developer's command of the widget's build.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Run a plan (the example's flags alone): replays, one child process
    /// per cell and run, report.
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
    /// Print scenarios, workloads and plans.
    List(Options),
    /// Measure the raster variants on the plan's workloads (CPU only).
    Raster(Options),
    /// Print the measurements beside the scenarios.
    Also,
    /// A plain-iced reproduction: moving images, continuous redraws.
    ReproSoftware {
        /// `--out FILE`; `None` is `bench-results/repro-<UTC stamp>.txt`.
        out: Option<PathBuf>,
        /// `--single`: one window-sized image in place of the grid of 40.
        single: bool,
    },
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
    /// environment); a report-only profile, which `suite` runs.
    pub xwayland: bool,
    /// `--frames DIR` (headless): every `every`-th composed frame written as
    /// a PNG under `DIR/<cell>__r<run>/`, to look at.
    pub frames: Option<PathBuf>,
    /// `--every N`: with `--frames`, every N-th frame; 1 by default.
    pub every: u32,
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

impl Command {
    /// Parses the arguments of `example`, `bench_gestures` or
    /// `bench_software`: one of its commands by name, else its plan's
    /// flags, which run it.
    pub fn parse(example: Example, args: &[String]) -> Result<Command, Error> {
        let (name, rest) = match args.split_first() {
            Some((first, rest)) if !first.starts_with("--") => (first.as_str(), rest),
            Some(_) | None => ("", args),
        };
        let software = example == Example::Software;
        match (name, software) {
            ("", false) => {
                let options = Options::parse(rest, &[])?;
                if options.plan == Some(Plan::Software) {
                    return Err(Error::Value {
                        flag: "--plan".to_owned(),
                        value: "software (bench_software runs it)".to_owned(),
                    });
                }
                Ok(Command::Run(options))
            }
            ("", true) => Options::software(rest).map(Command::Run),
            ("child", false) => Child::parse(rest).map(Command::Child),
            ("child", true) => {
                let child = Child::parse(rest)?;
                Options::software_checked(child.options.clone())?;
                Ok(Command::Child(child))
            }
            ("list", false) => Options::parse(rest, &[]).map(Command::List),
            ("raster", true) => Options::parse(rest, &[]).map(Command::Raster),
            ("also", false) => match rest {
                [] => Ok(Command::Also),
                [first, ..] => Err(Error::Stray(first.clone())),
            },
            ("replay", false) => single_out(rest).map(|out| Command::Replay { out }),
            ("record", false) => single_out(rest).map(|out| Command::Record { out }),
            ("repro-software", false) => {
                let single = rest.iter().any(|a| a == "--single");
                let rest: Vec<String> = rest.iter().filter(|a| *a != "--single").cloned().collect();
                let out = if rest.is_empty() {
                    Ok(None)
                } else {
                    single_out(&rest).map(Some)
                };
                out.map(|out| Command::ReproSoftware { out, single })
            }
            (other, _) => Err(Error::Stray(other.to_owned())),
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
                "--frames" => options.frames = Some(PathBuf::from(value)),
                "--every" => {
                    let every = value.parse::<u32>().ok().filter(|&n| n > 0);
                    options.every = every.ok_or_else(bad)?;
                }
                other if extra.contains(&other) => pairs.push((other.to_owned(), value.clone())),
                other => return Err(Error::Unknown(other.to_owned())),
            }
        }
        Ok((options, pairs))
    }

    /// `bench_software`'s flags: `run`'s, its plan the software plan,
    /// headless on tiny-skia.
    fn software(args: &[String]) -> Result<Options, Error> {
        let mut options = Options::parse(args, &[])?;
        options.headless = true;
        if !args.iter().any(|a| a == "--backend") {
            options.backend = Backend::TinySkia;
        }
        options.plan = Some(options.plan.unwrap_or(Plan::Software));
        Options::software_checked(options)
    }

    /// `options` when they are the software profile's: its plan, headless,
    /// on tiny-skia, at its own rate and factor.
    fn software_checked(options: Options) -> Result<Options, Error> {
        let given = |flag: &str, value: String| {
            Err(Error::Value {
                flag: flag.to_owned(),
                value: format!("{value} (bench_gestures runs the windowed plans)"),
            })
        };
        if let Some(plan) = options.plan.filter(|plan| *plan != Plan::Software) {
            return given("--plan", plan.name().to_owned());
        }
        if options.backend != Backend::TinySkia {
            return given("--backend", options.backend.name().to_owned());
        }
        if !options.headless {
            return given("--headless", "missing".to_owned());
        }
        if options.xwayland {
            return given("--xwayland", "set".to_owned());
        }
        Ok(options)
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
        if let Some(frames) = &self.frames {
            args.extend([
                "--frames".to_owned(),
                frames.display().to_string(),
                "--every".to_owned(),
                self.every.to_string(),
            ]);
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
            frames: None,
            every: 1,
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

/// `render`'s flag value.
pub fn render_name(render: Render) -> &'static str {
    match render {
        Render::Auto => "auto",
        Render::Inline => "inline",
        Render::Worker => "worker",
        _ => "other",
    }
}

/// `caching`'s flag value.
pub fn caching_name(caching: Caching) -> &'static str {
    match caching {
        Caching::On => "on",
        Caching::Off => "off",
        Caching::Verify => "verify",
        _ => "other",
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

    use super::{Backend, Command, PresentMode, Readback};
    use crate::cli::{self, Error};
    use crate::plan::Plan;
    use crate::scenario::Scenario;
    use crate::workload::Workload;
    use crate::{Example, rule};

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_owned).collect()
    }

    /// `line` parsed as `example` parses it, the widget's command
    /// unwrapped.
    fn parse_as(example: Example, line: &str) -> Result<Command, Error> {
        let parsed = cli::Command::parse(example, &args(line))?;
        let cli::Command::Widget(command) = parsed else {
            panic!("{line}: {parsed:?}");
        };
        Ok(*command)
    }

    /// `line` parsed as `bench_gestures` parses it.
    fn parse(line: &str) -> Result<Command, Error> {
        parse_as(Example::Gestures, line)
    }

    #[test]
    fn cli_parses_every_documented_flag() {
        let line = "--plan quick --scenario pan-x,zoom-lines --workload trace,grid16 \
                    --factor 1,1.25 --runs 3 --refresh-hz 144 --native-factor 1.25 \
                    --backend tiny-skia --present-mode fifo_relaxed --headless --readback last \
                    --render worker --caching verify --threads 3 --replay in.jsonl \
                    --data-scale 0.5 --budgets weak --no-also --out o";
        let Ok(Command::Run(options)) = parse(line) else {
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
        let Ok(Command::Run(options)) = parse("--headless --workload trace --backend auto") else {
            panic!("a run");
        };
        assert_eq!(options.plan(), Plan::Smoke);
        assert!(options.also);
        assert_eq!(
            (options.backend, options.backend.iced()),
            (Backend::Auto, None)
        );
        assert_eq!(Backend::TinySkia.iced(), Some("tiny-skia"));
        assert!(parse("--budgets laptop").is_err());
        let Ok(Command::Run(limited)) = parse("--cpu-limit pinned --xwayland") else {
            panic!("a run");
        };
        assert_eq!(limited.cpu_limit.as_deref(), Some("pinned"));
        assert!(limited.xwayland && !options.xwayland);
        for line in [
            "replay --out d",
            "record --out f.jsonl",
            "repro-software",
            "list",
            "also",
        ] {
            assert!(parse(line).is_ok(), "{line}");
        }
        for line in [
            "raster --plan default --out target/bench-raster",
            "child --headless --backend tiny-skia --cell hover__trace__f1 --run 1 --dir d",
        ] {
            assert!(parse_as(Example::Software, line).is_ok(), "{line}");
        }
        // Each example takes its own commands and plans.
        for line in ["raster", "--plan software", "report d", "figure-child"] {
            assert!(parse(line).is_err(), "{line}");
        }
        for line in [
            "list",
            "also",
            "--plan quick",
            "--backend wgpu",
            "--xwayland",
        ] {
            assert!(parse_as(Example::Software, line).is_err(), "{line}");
        }
        let Ok(Command::Run(software)) = parse_as(Example::Software, "--workload trace") else {
            panic!("a software run");
        };
        assert_eq!(
            (software.plan(), software.backend, software.headless),
            (Plan::Software, Backend::TinySkia, true)
        );
        // A child round-trips through the flags `run` gives it.
        let Ok(Command::Run(written)) = parse("--headless --frames f --every 5") else {
            panic!("a run");
        };
        assert_eq!(
            (written.frames, written.every),
            (Some(PathBuf::from("f")), 5)
        );
        assert!(parse("--every 0").is_err());
        let line = "child --plan weak --headless --threads 2 --caching verify --frames f \
                    --every 3 --cell pan-x__trace__f1.25 --run 2 --dir d";
        let Ok(Command::Child(child)) = parse(line) else {
            panic!("a child");
        };
        assert_eq!(child.cell.key(), "pan-x__trace__f1.25");
        let Ok(Command::Child(again)) = parse(&child.to_args(60.0, 2.0).join(" ")) else {
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
        assert_eq!(
            (again.options.frames, again.options.every),
            (Some(PathBuf::from("f")), 3)
        );
    }

    #[test]
    fn rejects_unknown_flags() {
        assert_eq!(parse("--pool 4"), Err(Error::Unknown("--pool".to_owned())));
        for (flag, value) in [
            ("--threads", "0"),
            ("--threads", "65"),
            ("--caching", "always"),
        ] {
            assert_eq!(
                parse(&format!("{flag} {value}")),
                Err(Error::Value {
                    flag: flag.to_owned(),
                    value: value.to_owned()
                })
            );
        }
        assert!(parse("--plan").is_err());
        assert_eq!(
            parse("repro-software --out r.txt"),
            Ok(Command::ReproSoftware {
                out: Some(PathBuf::from("r.txt")),
                single: false,
            })
        );
        assert_eq!(
            parse("repro-software --single"),
            Ok(Command::ReproSoftware {
                out: None,
                single: true,
            })
        );
        assert!(parse("repro-software --backend wgpu").is_err());
        assert!(parse("--factor 0").is_err());
        assert!(parse("--workload wheel").is_err());
        assert!(parse("--workload wheels,label-churn").is_ok());
        assert!(parse("frobnicate").is_err());
        // A windowed run needs the display's refresh rate and factor.
        let Ok(Command::Run(options)) = parse("--plan default") else {
            panic!("a run");
        };
        assert_eq!(options.refresh_hz(), Err(Error::Windowed("--refresh-hz")));
        assert_eq!(options.native(), Err(Error::Windowed("--native-factor")));
    }
}
