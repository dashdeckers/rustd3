//! Hand-parsed arguments: each example's [`Command`], a level's
//! [`Request`], and with the `widget` feature the widget examples'
//! commands of `widget`.

#[cfg(feature = "widget")]
pub mod widget;

use std::fmt;
use std::path::PathBuf;

use crate::Example;
use crate::experiment::{self, Experiment};
use crate::level::{Level, Request};

/// What the process does.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// A level (`bench_all`, `bench_figure`): the CPU reference and the
    /// figure profile, and in `bench_all` the display measurement and the
    /// widget's profiles.
    Level(Box<Request>),
    /// One figure of the figure profile, timed in its own process
    /// (`bench_figure figure-child`, internal).
    FigureChild {
        /// The level whose plan holds it.
        level: Level,
        /// Its name.
        subject: String,
    },
    /// Rebuild and print the report of a level's directory or archive (or
    /// of one run's directory): `bench_report`.
    Report {
        /// The directory or archive.
        path: PathBuf,
    },
    /// Two results side by side: `bench_compare`.
    Compare {
        /// A and B: directories or archives.
        paths: [PathBuf; 2],
    },
    /// Print the example's procedure and flags.
    Help,
    /// A command of `bench_gestures` or `bench_software`.
    #[cfg(feature = "widget")]
    Widget(Box<widget::Command>),
}

/// Why the arguments are rejected.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A flag the example does not take.
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
    /// Experiment flags that cannot hold together.
    Experiment(experiment::Error),
}

/// What `bench_all --help` prints.
#[cfg(feature = "widget")]
const HELP_ALL: &str = "\
bench_all: rustd3's benchmark, the plotting library and its Plot widget.

  cargo run --release --example bench_all -- [--quick | --default | --full]
      [--figure-only] [--out DIR] [--refresh-hz HZ] [--native-factor F]
      [--frames DIR [--every N]]

From a checkout of rustd3, on every platform. Under cargo it first builds
the examples it runs, each profile in processes of its own: bench_figure,
bench_software and bench_gestures. It runs on any machine and platform: it
prints what it will run and an estimate of the time, measures the display
(one window, a few seconds), takes three single-threaded CPU timings, times
figures from data to SVG, PDF and PNG through the library's own calls (the
figure profile, headless), drives the Plot widget on the software renderer
offscreen (the software profile, headless), then in the level's other
profiles in windows that open and close by themselves; keep hands off the
mouse and keyboard.
  --quick     about 16 minutes: what a performance issue needs. Six figures,
              7 times each; the software profile's 24 cells (hover, pan-x,
              zoom-lines, drag-resize on trace, scatter100k and grid16 at
              factors 1 and 2) once each; then the widget's cells hover,
              pan-x and zoom-lines on trace and scatter100k and drag-resize on
              trace, 3 runs each, on the native backend; pan-x and zoom-lines
              on trace on the software renderer (and again on X11 in a Linux
              Wayland session). Report-only.
  --default   about 95 minutes, the default: the figures with ladders of
              lines and scatters, the presets and the wheel figure; the
              software profile, 3 runs a cell; the registered verdict run
              (the rule's outcome) and the weak profiles (no GPU, 4 cores,
              2 cores).
  --full      hours: the figures to 10M markers and GIFs; the software
              profile, 5 runs a cell; every scenario x workload x factor on
              the native backend, the weak profiles and every `also`
              measurement.
  --figure-only
              the plotting library alone: the CPU reference and the figure
              profile at the level, no display measurement and no window, as
              bench_figure runs it.
  --out DIR   where results go (default bench-results).
  --refresh-hz HZ, --native-factor F
              the display's refresh rate and scale factor, for displays that
              cannot be measured (by default the display measurement gives
              both; an unpaced display, such as a VM's, runs at 60 Hz).
  --frames DIR [--every N]
              the software profile also writes its composed frames (every
              N-th) as PNGs under DIR/<cell>__r<run>/, to look at; the
              encoding takes a core, so those timings are not for comparing.

Experiments (one question on one machine, a command and an archive each;
recorded in results.json and the reports):
  --only P,...  only these profiles: figure, software, native, nogpu,
              nogpu-x11, 4cores, 2cores (with none that opens a window, no
              display is measured)
  --scenario A,B  --workload A,B  --factor F,G
              only these cells of the widget's profiles; a profile left
              without a cell is skipped and says why
  --runs N    N runs a cell
  --render auto|inline|worker  --threads N  --caching on|off|verify
              the Plot widget's settings in the widget's profiles
  --sweep KEY=V1,V2,...
              each selected widget profile once per value, as
              <profile>@KEY=V, into the one archive; KEY is render, threads
              or caching; the level's report sets the values side by side
              and bench_compare lines up profiles of the same name

Without a display it runs headless; without a GPU the native backend is
the software renderer; with too few cores a profile is skipped; a run that
fails is recorded and the level goes on. It writes
bench-results/<UTC stamp>-<level>/ and packs it, after every profile, into
bench-results/bench-<level>-<UTC stamp>.tar.gz, printed last: the one file
to attach to an issue (PRIVACY.md inside lists what it holds; no host or
user name, no home path). With --figure-only both names read
figure-<level> for <level>. bench_report rebuilds its report, bench_compare
sets two side by side.
";

/// What `bench_figure --help` prints.
const HELP_FIGURE: &str = "\
bench_figure: rustd3's benchmark of the plotting library alone.

  cargo run --release -p rustd3-figure --example bench_figure --
      [--quick | --default | --full] [--out DIR]

From a checkout of rustd3 (rustd3-figure's example; with -p rustd3-figure
cargo builds no iced, winit or wgpu for it). It runs on any machine and
platform, with no display and no GPU: it prints what it will run and an
estimate of the time, takes three single-threaded CPU timings, then times
figures from data to SVG, PDF and PNG through the library's own calls
(build, resolve, list, svg, pdf, png-1x, png-2x), each figure in a process
of its own.
  --quick     about a minute on two cores at 2.1 GHz: a 3 x 1M-sample trace,
              100k markers, 4 x 4 panels, typst math labels, a histogram and
              density of 1M values and a 1000 x 1000 heatmap, each stage 7
              times after a cold run.
  --default   about 4 minutes there, the default: also lines of 10k to 10M
              samples, scatters of 1k to 1M markers, the chart presets and the
              wheel figure, 10 times each.
  --full      also scatters of 10M markers, where memory allows, and GIFs,
              15 times each.
  --out DIR   where results go (default bench-results).
It writes bench-results/<UTC stamp>-figure-<level>/ and packs it into
bench-results/bench-figure-<level>-<UTC stamp>.tar.gz, printed last: the one
file to attach to an issue (PRIVACY.md inside lists what it holds; no host
or user name, no home path). bench_all runs the Plot widget's benchmark
too.

  figure-child --level L --subject NAME
              (internal: one figure of the figure profile)
";

/// What `bench_gestures --help` prints.
#[cfg(feature = "widget")]
const HELP_GESTURES: &str = "\
bench_gestures: the Plot widget driven by replayed gestures, in windows.

  cargo run --release --example bench_gestures -- [FLAGS]
      [--plan smoke|display|quick|default|weak|focus|full]
      [--scenario A,B] [--workload A,B] [--factor 1,1.25] [--runs N]
      [--refresh-hz HZ] [--native-factor F] [--backend wgpu|tiny-skia|auto]
      [--budgets desktop|weak]
      [--present-mode vsync|no_vsync|immediate|fifo|fifo_relaxed|mailbox]
      [--headless] [--readback every|last] [--render auto|inline|worker]
      [--caching on|off|verify] [--threads N] [--replay FILE]
      [--data-scale F] [--cpu-limit TEXT] [--xwayland] [--no-also]
      [--out DIR]

One plan into DIR/<UTC stamp>/ (default bench-out): replays, a process per
cell and run, the CSVs and report.md, then the \"Also measured\" items unless
--no-also. bench_all runs it for its native, nogpu and nogpu-x11 profiles
and the display measurement. Windowed, --refresh-hz and --native-factor
are required; headless, they default to 60 and 2, and --headless without
--plan is --plan smoke. --scenario, --workload and --factor restrict any
plan (smoke runs at the listed factors instead of 2); hover is a scenario
(--scenario hover). `weak` is the weaker machines' plan, held to the weak
budgets (as --budgets weak holds any plan); `focus` is report-only. The
software profile is bench_software. --backend auto leaves the choice to
iced (wgpu, or tiny-skia without a GPU adapter). --threads sets the
widget's render pool (1 to 64; default the available parallelism less one,
1 to 8); --caching verify renders every kept image again and reports what
differs. --xwayland (Linux, a Wayland session with XWayland) removes
WAYLAND_DISPLAY from the runs, whose windows then open on X11. --cpu-limit
records how the cores were limited.

  list    [FLAGS]       scenarios, workloads, plans and a plan's cells
  also                  the measurements beside the scenarios
                        (the report's \"Also measured\"; release)
  replay  --out DIR     write the generated replays
  record  --out FILE    record your own input in the trace window as a
                        replay (relative to the first event; Escape stops)
  repro-software [--single] [--out FILE]
          a plain-iced reproduction for upstream, no rustd3: 40
          images (--single: one window-sized image, so that drawing is
          cheap and presentation shows) moving 1 px a frame in a
          1600 x 1000 window, redrawn every frame; the backend from
          ICED_BACKEND; prints and writes
          (default bench-results/repro-<UTC stamp>.txt) the frame
          intervals and CPU use; env -u WAYLAND_DISPLAY for X11
  child   (internal: one scenario x workload x factor x run)
";

/// What `bench_software --help` prints.
#[cfg(feature = "widget")]
const HELP_SOFTWARE: &str = "\
bench_software: the Plot widget on the software renderer, offscreen.

  cargo run --release --example bench_software -- [--scenario A,B]
      [--workload A,B] [--factor 1,2] [--runs N] [--budgets desktop|weak]
      [--readback every|last] [--render auto|inline|worker]
      [--caching on|off|verify] [--threads N] [--replay FILE]
      [--data-scale F] [--cpu-limit TEXT] [--no-also]
      [--frames DIR [--every N]] [--out DIR]

The software profile: the software plan headless on tiny-skia at 60 Hz
and a native factor of 2, on any machine, every frame composed into a
pixmap and its cost timed by part (parts.csv), into DIR/<UTC stamp>/
(default bench-out), a process per cell and run. bench_all runs it as its
software profile. --frames writes the composed frames (every N-th) as PNGs
under DIR/<cell>__r<run>/.

  raster  [--plan P] [--workload A,B] [--data-scale F] [--out DIR]
          the raster variants (envelope fills, marker splatting) against
          the cells as drawn, CPU only, into DIR/raster.csv
  child   (internal: one scenario x workload x factor x run)
";

/// What `bench_report --help` prints.
const HELP_REPORT: &str = "\
bench_report: rebuild and print a benchmark's report.

  cargo run --release --example bench_report -- PATH

PATH is a level's directory or archive (each profile's report rebuilt from
its raw records, in a temporary directory for an archive), or one run's
directory; a directory from before the levels is read as it is.
";

/// What `bench_compare --help` prints.
const HELP_COMPARE: &str = "\
bench_compare: two benchmark results side by side.

  cargo run --release --example bench_compare -- A B

A and B are directories or archives (from before the levels too): the
machines and experiments, then per profile (a sweep's by <profile>@KEY=V)
and cell frame p50 and p99, latency p95, preview share, drops, UI frame
p99 and CPU s/s (the software profile's frame cost by part), and per
figure and stage the medians, of A and B and B/A.
";

/// What `example`'s `--help` prints.
pub fn help(example: Example) -> &'static str {
    match example {
        Example::Figure => HELP_FIGURE,
        #[cfg(feature = "widget")]
        Example::Gestures => HELP_GESTURES,
        #[cfg(feature = "widget")]
        Example::Software => HELP_SOFTWARE,
        #[cfg(feature = "widget")]
        Example::All => HELP_ALL,
        Example::Compare => HELP_COMPARE,
        Example::Report => HELP_REPORT,
    }
}

impl Command {
    /// Parses `example`'s arguments after the program name.
    pub fn parse(example: Example, args: &[String]) -> Result<Command, Error> {
        if args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "help")
        {
            return Ok(Command::Help);
        }
        match example {
            Example::Figure => match args.split_first() {
                Some((first, rest)) if first == "figure-child" => figure_child(rest),
                Some(_) | None => Request::parse(args, false).map(|r| Command::Level(Box::new(r))),
            },
            #[cfg(feature = "widget")]
            Example::All => Request::parse(args, true).map(|r| Command::Level(Box::new(r))),
            #[cfg(feature = "widget")]
            Example::Gestures | Example::Software => {
                widget::Command::parse(example, args).map(|c| Command::Widget(Box::new(c)))
            }
            Example::Report => match args {
                [path] if !path.starts_with("--") => Ok(Command::Report {
                    path: PathBuf::from(path),
                }),
                [first, ..] => Err(Error::Stray(first.clone())),
                [] => Err(Error::Missing("PATH".to_owned())),
            },
            Example::Compare => match args {
                [a, b] if !a.starts_with("--") && !b.starts_with("--") => Ok(Command::Compare {
                    paths: [PathBuf::from(a), PathBuf::from(b)],
                }),
                [_, _, third, ..] => Err(Error::Stray(third.clone())),
                [first, ..] if first.starts_with("--") => Err(Error::Unknown(first.clone())),
                [_, second] => Err(Error::Unknown(second.clone())),
                [_] | [] => Err(Error::Missing("A B".to_owned())),
            },
        }
    }
}

/// `figure-child`'s flags: `--level L --subject NAME`.
fn figure_child(rest: &[String]) -> Result<Command, Error> {
    match rest {
        [l, level, s, subject] if l == "--level" && s == "--subject" => Ok(Command::FigureChild {
            level: Level::parse(level).ok_or_else(|| Error::Value {
                flag: l.clone(),
                value: level.clone(),
            })?,
            subject: subject.clone(),
        }),
        _ => Err(Error::Missing(
            "figure-child --level L --subject NAME".to_owned(),
        )),
    }
}

impl Request {
    /// Parses a level's flags: at most one of `--quick`, `--default` and
    /// `--full` (`--default` when none), `--out DIR`, the hidden
    /// `--dry-run`, and in `bench_all` (`widget`) `--figure-only`,
    /// `--refresh-hz HZ`, `--native-factor F`, `--frames DIR`, `--every N`
    /// and the experiment flags; without `widget` the level is the
    /// plotting library's alone.
    fn parse(args: &[String], widget: bool) -> Result<Request, Error> {
        let mut request = Request {
            level: Level::Default,
            out: PathBuf::from("bench-results"),
            refresh_hz: None,
            native: None,
            figure_only: !widget,
            dry_run: false,
            frames: None,
            every: 1,
            experiment: Experiment::default(),
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
                "--out" => {
                    let value = args.next().ok_or_else(|| Error::Missing(flag.clone()))?;
                    request.out = PathBuf::from(value);
                }
                #[cfg(feature = "widget")]
                "--figure-only" if widget => request.figure_only = true,
                #[cfg(feature = "widget")]
                "--refresh-hz" | "--native-factor" if widget => {
                    let value = args.next().ok_or_else(|| Error::Missing(flag.clone()))?;
                    let bad = || Error::Value {
                        flag: flag.clone(),
                        value: value.clone(),
                    };
                    if flag == "--refresh-hz" {
                        let hz = value
                            .parse::<f64>()
                            .ok()
                            .filter(|hz| hz.is_finite() && *hz > 0.0);
                        request.refresh_hz = Some(hz.ok_or_else(bad)?);
                    } else {
                        request.native = Some(factor(value).ok_or_else(bad)?);
                    }
                }
                #[cfg(feature = "widget")]
                "--frames" if widget => {
                    let value = args.next().ok_or_else(|| Error::Missing(flag.clone()))?;
                    request.frames = Some(PathBuf::from(value));
                }
                #[cfg(feature = "widget")]
                "--every" if widget => {
                    let value = args.next().ok_or_else(|| Error::Missing(flag.clone()))?;
                    let every = value.parse::<u32>().ok().filter(|&n| n > 0);
                    request.every = every.ok_or_else(|| Error::Value {
                        flag: flag.clone(),
                        value: value.clone(),
                    })?;
                }
                #[cfg(feature = "widget")]
                "--only" | "--scenario" | "--workload" | "--factor" | "--runs" | "--render"
                | "--threads" | "--caching" | "--sweep"
                    if widget =>
                {
                    let value = args.next().ok_or_else(|| Error::Missing(flag.clone()))?;
                    experiment_flag(&mut request.experiment, flag, value)?;
                }
                other if other.starts_with("--") => return Err(Error::Unknown(other.to_owned())),
                other => return Err(Error::Stray(other.to_owned())),
            }
        }
        if request.experiment.only.is_some() && request.figure_only && widget {
            return Err(Error::Value {
                flag: "--only".to_owned(),
                value: "with --figure-only".to_owned(),
            });
        }
        request.experiment = request.experiment.checked().map_err(Error::Experiment)?;
        Ok(request)
    }
}

/// Sets the experiment flag `flag` to `value`, each item checked.
#[cfg(feature = "widget")]
fn experiment_flag(experiment: &mut Experiment, flag: &str, value: &str) -> Result<(), Error> {
    use crate::experiment::Sweep;
    use crate::scenario::Scenario;
    use crate::workload::Workload;

    let bad = || Error::Value {
        flag: flag.to_owned(),
        value: value.to_owned(),
    };
    let names = |known: &dyn Fn(&str) -> bool| -> Result<Vec<String>, Error> {
        let items: Vec<String> = value.split(',').map(|s| s.trim().to_owned()).collect();
        if items.iter().all(|item| known(item)) {
            Ok(items)
        } else {
            Err(bad())
        }
    };
    match flag {
        "--only" => experiment.only = Some(names(&|_| true)?),
        "--scenario" => {
            experiment.scenarios = Some(names(&|s| Scenario::parse(s).is_some())?);
        }
        "--workload" => {
            experiment.workloads = Some(names(&|w| Workload::parse(w).is_some())?);
        }
        "--factor" => {
            let factors: Option<Vec<f32>> = value.split(',').map(|f| factor(f.trim())).collect();
            experiment.factors = Some(factors.ok_or_else(bad)?);
        }
        "--runs" => {
            let runs = value.parse::<u32>().ok().filter(|&n| n > 0);
            experiment.runs = Some(runs.ok_or_else(bad)?);
        }
        "--render" => {
            let known = experiment::RENDERS.contains(&value);
            experiment.render = Some(known.then(|| value.to_owned()).ok_or_else(bad)?);
        }
        "--caching" => {
            let known = experiment::CACHINGS.contains(&value);
            experiment.caching = Some(known.then(|| value.to_owned()).ok_or_else(bad)?);
        }
        "--threads" => {
            let threads = value.parse::<usize>().ok();
            let threads = threads.filter(|n| (1..=64).contains(n));
            experiment.threads = Some(threads.ok_or_else(bad)?);
        }
        _ => experiment.sweep = Some(Sweep::parse(value).map_err(Error::Experiment)?),
    }
    Ok(())
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Unknown(flag) => write!(f, "unknown flag {flag} (see --help)"),
            Error::Missing(flag) => write!(f, "{flag} needs a value"),
            Error::Value { flag, value } => write!(f, "{flag} does not take {value:?}"),
            Error::Stray(arg) => write!(f, "unexpected argument {arg:?} (see --help)"),
            Error::Experiment(error) => write!(f, "{error}"),
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

/// A scale factor: finite, from 0.5 to 4.
fn factor(value: &str) -> Option<f32> {
    value
        .parse::<f32>()
        .ok()
        .filter(|f| f.is_finite() && (0.5..=4.0).contains(f))
}

#[cfg(test)]
mod tests {
    use super::{Command, Error};
    use crate::Example;

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_owned).collect()
    }

    #[cfg(feature = "widget")]
    #[test]
    fn bench_all_takes_a_level() {
        use std::path::PathBuf;

        use crate::level::{Level, Request};

        let parse = |line: &str| Command::parse(Example::All, &args(line));
        let level = |line: &str| match parse(line) {
            Ok(Command::Level(request)) => *request,
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
                frames: None,
                every: 1,
                experiment: crate::experiment::Experiment::default(),
            }
        );
        assert_eq!(level("--quick").level, Level::Quick);
        assert_eq!(level("--full --full").level, Level::Full);
        let given = level(
            "--quick --out o --refresh-hz 144 --native-factor 1.25 --dry-run --frames f --every 4",
        );
        assert_eq!(
            (given.out, given.refresh_hz, given.native, given.dry_run),
            (PathBuf::from("o"), Some(144.0), Some(1.25), true)
        );
        assert_eq!((given.frames, given.every), (Some(PathBuf::from("f")), 4));
        assert!(parse("--quick --every 0").is_err());
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
            parse("--quick --full"),
            Err(Error::Value {
                flag: "--full".to_owned(),
                value: "with --quick".to_owned()
            })
        );
        assert_eq!(
            parse("--quick --plan default"),
            Err(Error::Unknown("--plan".to_owned()))
        );
        assert!(parse("--refresh-hz 0").is_err());
        assert!(parse("--quick extra").is_err());
        assert!(parse("run --plan quick").is_err());
        assert!(parse("compare a b").is_err());
        assert_eq!(parse("--help"), Ok(Command::Help));
    }

    #[cfg(feature = "widget")]
    #[test]
    fn experiment_flags_narrow_and_vary_a_level() {
        use crate::experiment::{self, Key};

        let parse = |line: &str| Command::parse(Example::All, &args(line));
        let level = |line: &str| match parse(line) {
            Ok(Command::Level(request)) => request.experiment,
            other => panic!("{line}: {other:?}"),
        };
        let e = level(
            "--quick --only software,figure --scenario zoom-lines,pan-x --workload trace \
             --factor 1,2 --runs 2 --render inline --caching off --sweep threads=1,2,8",
        );
        assert_eq!(
            e.only.as_deref(),
            Some(&["software".to_owned(), "figure".to_owned()][..])
        );
        assert_eq!(e.scenarios.unwrap(), ["zoom-lines", "pan-x"]);
        assert_eq!(e.workloads.unwrap(), ["trace"]);
        assert_eq!(e.factors.unwrap(), [1.0, 2.0]);
        assert_eq!(e.runs, Some(2));
        assert_eq!(
            (e.render.as_deref(), e.caching.as_deref()),
            (Some("inline"), Some("off"))
        );
        assert_eq!(e.sweep.unwrap().key, Key::Threads);
        assert_eq!(level("--threads 3").threads, Some(3));
        for bad in [
            "--only gpu",
            "--scenario zoom",
            "--workload scatter",
            "--factor 9",
            "--runs 0",
            "--render gpu",
            "--threads 0",
            "--caching always",
            "--sweep pacing=bench,none",
            "--sweep threads=2 --threads 2",
            "--only software --figure-only",
        ] {
            assert!(parse(bad).is_err(), "{bad}");
        }
        assert_eq!(
            parse("--sweep render=inline --render worker"),
            Err(Error::Experiment(experiment::Error::Twice(Key::Render)))
        );
    }

    #[test]
    fn bench_figure_runs_the_library_alone() {
        use crate::level::Level;

        let parse = |line: &str| Command::parse(Example::Figure, &args(line));
        let Ok(Command::Level(request)) = parse("--quick") else {
            panic!("a level");
        };
        assert!(request.figure_only, "implied");
        for line in [
            "run --plan quick",
            "list",
            "repro-software",
            "child",
            "compare a b",
        ] {
            assert!(matches!(parse(line), Err(Error::Stray(_))), "{line}");
        }
        for flag in ["--refresh-hz 60", "--figure-only", "--only figure"] {
            let name = flag.split(' ').next().unwrap_or_default().to_owned();
            assert_eq!(parse(flag), Err(Error::Unknown(name)), "{flag}");
        }
        assert_eq!(
            parse("figure-child --level quick --subject trace"),
            Ok(Command::FigureChild {
                level: Level::Quick,
                subject: "trace".to_owned()
            })
        );
        assert!(parse("figure-child --level fast --subject trace").is_err());
        assert!(super::help(Example::Figure).contains("figure-child"));
        assert!(!super::help(Example::Figure).contains("repro-software"));
    }

    #[test]
    fn bench_report_and_bench_compare_take_paths() {
        use std::path::PathBuf;

        let report = |line: &str| Command::parse(Example::Report, &args(line));
        assert_eq!(
            report("r.tar.gz"),
            Ok(Command::Report {
                path: PathBuf::from("r.tar.gz")
            })
        );
        assert!(report("").is_err() && report("a b").is_err() && report("--out o").is_err());
        let compare = |line: &str| Command::parse(Example::Compare, &args(line));
        assert!(compare("a.tar.gz b").is_ok());
        assert_eq!(compare("a"), Err(Error::Missing("A B".to_owned())));
        assert!(compare("a b c").is_err());
        assert!(compare("a --runs").is_err());
    }
}
