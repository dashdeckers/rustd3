//! The `figure` profile: what a user of the plotting library
//! alone pays, at every level, headless and without iced, so it runs on
//! any machine. For each [`Subject`], each [`Stage`] from building the
//! figure to its files is timed through the library's public API with its
//! default settings, a cold repetition first and then the level's
//! repetitions. Each figure runs in a process of its own (`figure-child`):
//! in one process the resident high-water mark carries what earlier
//! figures left with the allocator, so a figure's peak would depend on
//! what ran before it; alone, its peak memory (Linux's
//! `/proc/self/status`, "n/a" elsewhere) is its own, its first repetition
//! is cold as a program's first figure is, and a crash fails that figure
//! only. There are no budgets: the profile is report-only, and the CPU
//! reference puts its numbers on a scale across machines. It writes
//! `figure/summary.csv`, `figure/report.md` and `run-figure.log` into the
//! level's directory and its entry in `results.json`.

mod report;
mod rss;
mod stage;
mod subject;

use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::level::Level;
use crate::results;
use crate::stats::percentile;
use crate::system;

pub use report::{csv, find, markdown, ms, summary};
pub use stage::Stage;
pub use subject::Subject;

/// The profile's name: its directory, its log and its entry in
/// `results.json`.
pub const NAME: &str = "figure";

/// What the profile runs at a level.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    /// The level, which its children are told.
    pub level: Level,
    /// The figures, in order.
    pub subjects: Vec<Subject>,
    /// The outputs timed after build, resolve and list.
    pub outputs: Vec<Stage>,
    /// Repetitions after the cold one.
    pub repetitions: u32,
    /// The machine's memory, MiB (0: unknown), against which a figure
    /// that needs more than [`MEMORY_SHARE`] of it is skipped.
    pub ram_mb: u64,
}

/// The share of the machine's memory a figure may need.
const MEMORY_SHARE: f64 = 0.6;

/// The figures of every level.
const MAIN: [Subject; 6] = [
    Subject::Trace,
    Subject::Scatter(100_000),
    Subject::Grid16,
    Subject::Labels,
    Subject::Stats,
    Subject::Heatmap,
];

/// How long a figure's process may run beyond ten times its estimate
/// before it is killed, s.
const CHILD_MARGIN_S: f64 = 120.0;

/// The ladders' rungs: lines, then scatters other than `scatter100k`.
const LINES: [usize; 4] = [10_000, 100_000, 1_000_000, 10_000_000];
const SCATTERS: [usize; 3] = [1_000, 10_000, 1_000_000];

impl Plan {
    /// The profile at `level` on a machine of `ram_mb` MiB (0: unknown):
    /// at `--quick` the six main figures, 7 repetitions after the cold
    /// one; at `--default` also the ladders (lines of 10k to 10M samples,
    /// scatters of 1k to 1M markers), the presets and the wheel figure, 10
    /// repetitions; at `--full` the scatters to 10M markers and a GIF of
    /// every figure but the ladders', 15 repetitions.
    pub fn of(level: Level, ram_mb: u64) -> Plan {
        let outputs = vec![Stage::Svg, Stage::Pdf, Stage::Png(1), Stage::Png(2)];
        let mut subjects = MAIN.to_vec();
        if level == Level::Quick {
            return Plan {
                level,
                subjects,
                outputs,
                repetitions: 7,
                ram_mb,
            };
        }
        subjects.extend(LINES.map(Subject::Line));
        subjects.extend(SCATTERS.map(Subject::Scatter));
        if level == Level::Full {
            subjects.push(Subject::Scatter(10_000_000));
        }
        subjects.extend(crate::gallery::presets::NAMES.map(Subject::Preset));
        subjects.push(Subject::Wheels);
        let (outputs, repetitions) = match level {
            Level::Full => ([outputs, vec![Stage::Gif]].concat(), 15),
            Level::Quick | Level::Default => (outputs, 10),
        };
        Plan {
            level,
            subjects,
            outputs,
            repetitions,
            ram_mb,
        }
    }

    /// Why `subject` is skipped on this machine: it needs more than
    /// [`MEMORY_SHARE`] of the memory.
    fn skip(&self, subject: Subject) -> Option<String> {
        let (need, ram) = (subject.memory_mb(), self.ram_mb as f64);
        (ram > 0.0 && need > MEMORY_SHARE * ram).then(|| {
            format!(
                "it needs about {:.1} GiB, more than {:.0}% of this machine's {:.1} GiB",
                need / 1024.0,
                100.0 * MEMORY_SHARE,
                ram / 1024.0
            )
        })
    }

    /// The stages a subject runs: all, but no GIF of the ladders.
    fn stages(&self, subject: Subject) -> Vec<Stage> {
        let ladder = subject.rung().is_some() && !MAIN.contains(&subject);
        self.outputs
            .iter()
            .copied()
            .filter(|&stage| !(ladder && stage == Stage::Gif))
            .collect()
    }

    /// What it runs, in a line.
    pub fn describe(&self) -> String {
        let mut stages = vec!["build", "resolve", "list"];
        stages.extend(self.outputs.iter().map(|s| s.name()));
        format!(
            "{} figures ({}), each timed {} (a cold repetition and {} more), headless, no budgets",
            self.subjects.len(),
            names(&self.subjects),
            stages.join(", "),
            self.repetitions
        )
    }

    /// Its estimated wall time, s: each subject's repetitions on two
    /// cores at 2.1 GHz, where it was measured, those skipped aside.
    pub fn estimate_s(&self) -> f64 {
        self.subjects
            .iter()
            .filter(|&&subject| self.skip(subject).is_none())
            .map(|&subject| {
                let stages = self.stages(subject);
                subject.data_s() + f64::from(self.repetitions + 1) * subject.repetition_s(&stages)
            })
            .sum()
    }

    /// Times every subject, each in its own process of the bench at
    /// `exe` (`figure-child`), prints each as it ends, and writes
    /// `figure/summary.csv`, `figure/report.md` and `run-figure.log` into
    /// `dir`: the profile's entry in `results.json`.
    pub fn run(&self, exe: &Path, dir: &Path) -> Result<results::Profile, Box<dyn Error>> {
        let out = dir.join(NAME);
        fs::create_dir_all(&out)?;
        let mut log = String::new();
        let mut figures = Vec::new();
        let n = self.subjects.len();
        for (index, &subject) in self.subjects.iter().enumerate() {
            let begin = Instant::now();
            let figure = match self.skip(subject) {
                Some(why) => results::Figure {
                    skipped: Some(why),
                    ..blank(subject)
                },
                None => self.spawn(exe, subject),
            };
            let line = format!(
                "[{}/{n}] {}: {} ({:.1} s)",
                index + 1,
                figure.name,
                report::line(&figure),
                begin.elapsed().as_secs_f64()
            );
            println!("{line}");
            let _ = writeln!(log, "{line}");
            figures.push(figure);
        }
        let failed: Vec<String> = figures
            .iter()
            .filter(|f| f.failed.is_some())
            .map(|f| f.name.clone())
            .collect();
        let skipped = figures.iter().filter(|f| f.skipped.is_some()).count();
        let timed = figures.len() - failed.len() - skipped;
        let mut profile = results::Profile {
            plan: NAME.to_owned(),
            backend: "vello_cpu".to_owned(),
            ran: vec!["vello_cpu".to_owned()],
            budgets: "none".to_owned(),
            headless: true,
            status: results::Status::Done,
            outcome: Some(match skipped {
                0 => format!("report-only: {timed} figures timed, no budgets"),
                n => format!("report-only: {timed} figures timed, {n} skipped, no budgets"),
            }),
            figures,
            ..results::Profile::planned(NAME, self.describe(), None)
        };
        if !failed.is_empty() {
            profile.status = results::Status::Failed;
            profile.reason = Some(format!(
                "{} of {n} figures failed: {} (figure/report.md)",
                failed.len(),
                failed.join(", ")
            ));
        }
        fs::write(out.join("summary.csv"), csv(&profile))?;
        fs::write(out.join("report.md"), markdown(&profile))?;
        fs::write(dir.join(format!("run-{NAME}.log")), log)?;
        Ok(profile)
    }

    /// `subject` timed in a process of its own, `exe figure-child --level
    /// L --subject NAME`, which prints the figure as JSON: each figure's
    /// peak memory is its own and its first repetition truly cold, whatever
    /// ran before it. A child that fails or outlives its limit leaves the
    /// figure failed, with why.
    fn spawn(&self, exe: &Path, subject: Subject) -> results::Figure {
        let failed = |why: String| results::Figure {
            failed: Some(why),
            ..blank(subject)
        };
        let spawned = Command::new(exe)
            .args(["figure-child", "--level", self.level.name()])
            .args(["--subject", &subject.name()])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        let mut child = match spawned {
            Ok(child) => child,
            Err(error) => return failed(format!("could not start: {error}")),
        };
        let (out, err) = (
            system::drain(child.stdout.take()),
            system::drain(child.stderr.take()),
        );
        let estimate =
            f64::from(self.repetitions + 1) * subject.repetition_s(&self.stages(subject));
        let limit = Duration::from_secs_f64(CHILD_MARGIN_S + 10.0 * estimate);
        let status = system::wait(&mut child, limit);
        let (out, err) = (
            out.join().unwrap_or_default(),
            err.join().unwrap_or_default(),
        );
        let last = |text: &str| {
            text.lines()
                .rev()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("")
                .to_owned()
        };
        match status {
            Ok(Some(status)) if status.success() => serde_json::from_str(&last(&out))
                .unwrap_or_else(|error| failed(format!("an unreadable result: {error}"))),
            Ok(Some(status)) => failed(format!("its process ended with {status}: {}", last(&err))),
            Ok(None) => failed(format!("killed after {:.0} s", limit.as_secs_f64())),
            Err(error) => failed(format!("could not wait for it: {error}")),
        }
    }

    /// `subject` timed in this process: its data generated, the
    /// high-water mark reset, then a cold repetition and the plan's
    /// repetitions of its stages; a stage's error or a panic leaves it
    /// failed with the reason.
    fn time(&self, subject: Subject) -> results::Figure {
        let size = subject.size();
        let stages = self.stages(subject);
        let mut figure = blank(subject);
        let timed = panic::catch_unwind(AssertUnwindSafe(|| {
            let data = subject.data();
            rss::reset();
            (0..=self.repetitions)
                .map(|_| {
                    stage::repetition(&|| data.figure(), size, &stages).map_err(|e| e.to_string())
                })
                .collect::<Result<Vec<_>, String>>()
        }));
        match timed {
            Ok(Ok(repetitions)) => {
                figure.stages = summarize(&repetitions);
                figure.rss_peak_mb = rss::peak_mb();
            }
            Ok(Err(error)) => figure.failed = Some(error),
            Err(payload) => {
                let message = payload
                    .downcast_ref::<&str>()
                    .map(|s| (*s).to_owned())
                    .or_else(|| payload.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "a panic".to_owned());
                figure.failed = Some(format!("panicked: {message}"));
            }
        }
        figure
    }
}

/// `figure-child`: times the figure `name` of the plan at `level` in this
/// process and prints it as JSON, for [`Plan::run`].
pub fn child(level: Level, name: &str) -> Result<(), Box<dyn Error>> {
    let plan = Plan::of(level, 0);
    let subject = plan
        .subjects
        .iter()
        .copied()
        .find(|s| s.name() == name)
        .ok_or_else(|| format!("--{} times no figure {name}", level.name()))?;
    println!("{}", serde_json::to_string(&plan.time(subject))?);
    Ok(())
}

/// `subject`'s entry before it is timed.
fn blank(subject: Subject) -> results::Figure {
    let size = subject.size();
    results::Figure {
        name: subject.name(),
        what: subject.what(),
        size: [size.width, size.height],
        rss_peak_mb: None,
        failed: None,
        skipped: None,
        stages: Vec::new(),
    }
}

/// Each stage of `repetitions` (the first cold, every one with the same
/// stages in the same order): the cold time, the median, least and 90th
/// percentile of the rest, and the last output's bytes.
fn summarize(repetitions: &[Vec<stage::Measured>]) -> Vec<results::Timing> {
    let Some((cold, warm)) = repetitions.split_first() else {
        return Vec::new();
    };
    cold.iter()
        .enumerate()
        .map(|(i, first)| {
            let times: Vec<f64> = warm.iter().filter_map(|r| r.get(i)).map(|m| m.ms).collect();
            let at = |p: f64| percentile(&times, p).unwrap_or(first.ms);
            results::Timing {
                stage: first.stage.name().to_owned(),
                cold_ms: first.ms,
                median_ms: at(50.0),
                min_ms: at(0.0),
                p90_ms: at(90.0),
                repetitions: times.len() as u32,
                bytes: warm
                    .last()
                    .and_then(|r| r.get(i))
                    .map_or(first.bytes, |m| m.bytes),
            }
        })
        .collect()
}

/// The subjects' names, the ladders as ranges and the presets as a count:
/// "trace, …, lines of 10k to 10m samples, 11 presets, wheels".
fn names(subjects: &[Subject]) -> String {
    let presets = subjects
        .iter()
        .filter(|s| matches!(s, Subject::Preset(_)))
        .count();
    let mut out: Vec<String> = Vec::new();
    for &subject in subjects {
        let name = if matches!(subject, Subject::Preset(_)) {
            format!("{presets} presets")
        } else if let Some((ladder, n)) = subject.rung().filter(|_| !MAIN.contains(&subject)) {
            let sizes = || {
                subjects
                    .iter()
                    .filter_map(|s| s.rung())
                    .filter(|rung| rung.0 == ladder)
                    .map(|rung| rung.1)
            };
            let unit = if ladder == "lines" {
                "samples"
            } else {
                "markers"
            };
            format!(
                "{ladder} of {} to {} {unit}",
                subject::count(sizes().min().unwrap_or(n)),
                subject::count(sizes().max().unwrap_or(n))
            )
        } else {
            subject.name()
        };
        if !out.contains(&name) {
            out.push(name);
        }
    }
    out.join(", ")
}

#[cfg(test)]
mod tests {
    use super::{MAIN, Plan, Stage, Subject, names, summarize};
    use crate::level::Level;

    #[test]
    fn each_level_times_d22_10s_figures() {
        let quick = Plan::of(Level::Quick, 0);
        assert_eq!(quick.subjects, MAIN);
        assert_eq!(
            quick.outputs,
            [Stage::Svg, Stage::Pdf, Stage::Png(1), Stage::Png(2)]
        );
        let default = Plan::of(Level::Default, 0);
        assert!(default.repetitions > quick.repetitions);
        assert!(default.subjects.contains(&Subject::Line(10_000_000)));
        assert!(default.subjects.contains(&Subject::Scatter(1_000_000)));
        assert!(!default.subjects.contains(&Subject::Scatter(10_000_000)));
        assert!(default.subjects.contains(&Subject::Preset("pie")));
        assert!(default.subjects.contains(&Subject::Wheels));
        assert!(!default.outputs.contains(&Stage::Gif));
        let full = Plan::of(Level::Full, 0);
        assert!(full.subjects.contains(&Subject::Scatter(10_000_000)));
        assert!(full.stages(Subject::Trace).contains(&Stage::Gif));
        assert!(!full.stages(Subject::Line(10_000)).contains(&Stage::Gif));
        assert!(full.stages(Subject::Scatter(100_000)).contains(&Stage::Gif));
        // About 1-2 minutes at quick and 5 at default on two cores at 2.1
        // GHz, where they were measured.
        let minutes = |level| Plan::of(level, 8_000).estimate_s() / 60.0;
        let (short, long) = (minutes(Level::Quick), minutes(Level::Default));
        assert!((1.0..2.0).contains(&short), "{short}");
        assert!((3.0..6.0).contains(&long), "{long}");
        assert!(minutes(Level::Full) > long);
        // 10M markers need about 14 GiB: skipped with 8, timed with 32.
        let full = Plan::of(Level::Full, 8_000);
        let why = full.skip(Subject::Scatter(10_000_000)).unwrap();
        assert!(
            why.starts_with("it needs about 13.8 GiB, more than 60%"),
            "{why}"
        );
        assert_eq!(full.skip(Subject::Scatter(1_000_000)), None);
        let big = Plan::of(Level::Full, 32_000);
        assert_eq!(big.skip(Subject::Scatter(10_000_000)), None);
        let unknown = Plan::of(Level::Full, 0);
        assert_eq!(unknown.skip(Subject::Scatter(10_000_000)), None);
        assert_eq!(
            names(&default.subjects),
            "trace, scatter100k, grid16, labels, stats, heatmap, lines of 10k to 10m samples, \
             scatters of 1k to 1m markers, 11 presets, wheels"
        );
    }

    #[test]
    fn the_cold_repetition_is_apart_from_the_rest() {
        let measured = |stage, ms, bytes| super::stage::Measured { stage, ms, bytes };
        let repetitions: Vec<Vec<_>> = [90.0, 10.0, 14.0, 12.0, 11.0]
            .into_iter()
            .map(|ms| {
                vec![
                    measured(Stage::Resolve, ms, None),
                    measured(Stage::Svg, 2.0 * ms, Some(4_000)),
                ]
            })
            .collect();
        let timings = summarize(&repetitions);
        assert_eq!(timings.len(), 2);
        let resolve = &timings[0];
        assert_eq!(resolve.stage, "resolve");
        assert_eq!(resolve.cold_ms, 90.0);
        assert_eq!(resolve.repetitions, 4);
        assert_eq!((resolve.min_ms, resolve.median_ms), (10.0, 11.5));
        assert!((resolve.p90_ms - 13.4).abs() < 1e-9, "{}", resolve.p90_ms);
        assert_eq!(resolve.bytes, None);
        assert_eq!(timings[1].bytes, Some(4_000));
        assert!(summarize(&[]).is_empty());
    }
}
