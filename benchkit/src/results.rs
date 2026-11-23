//! [`Results`]: a level's `results.json`, the summary a submission is read
//! by, and the level's `report.md`, written from it. `results.json` is
//! versioned by its `format` (the schema is in `benchkit/README.md`).
//! Format 2 adds the figure profile's figures, and format 1 reads as a
//! level without them.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::display::Display;
use crate::experiment::{self, Experiment};
use crate::reference::Reference;
use crate::{figure, system};

/// The format `results.json` is written in.
pub const FORMAT: u32 = 2;

/// A level's results.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Results {
    /// [`FORMAT`] when written.
    pub format: u32,
    /// The rustd3 that ran.
    pub rustd3: Version,
    /// `quick`, `default` or `full`.
    pub level: String,
    /// When it started, UTC (`YYYYmmddTHHMMSSZ`).
    pub stamp: String,
    /// Whether every profile has run. False marks the archive of an
    /// interrupted level.
    pub complete: bool,
    /// The machine.
    pub machine: Machine,
    /// The display, and how the level knows it.
    pub display: Display,
    /// The plotting library alone (`--figure-only`, or a build without
    /// the widget): the CPU reference and the figure profile, no display.
    #[serde(default)]
    pub figure_only: bool,
    /// The experiment flags the level ran with, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experiment: Option<Experiment>,
    /// The CPU reference's timings.
    #[serde(default)]
    pub cpu_reference: Vec<Reference>,
    /// The profiles, in order, those skipped and those not yet run
    /// included.
    pub profiles: Vec<Profile>,
}

/// The rustd3 that ran.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Version {
    /// The workspace's version.
    pub version: String,
    /// `git rev-parse HEAD` of the checkout, else "unknown".
    pub commit: String,
    /// The pinned iced revision.
    pub iced: String,
}

/// The machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Machine {
    /// `linux`, `windows`, `macos`, …
    pub os: String,
    /// The release's name (`/etc/os-release`, `sw_vers`, `ver`).
    pub release: String,
    /// The kernel (`uname -r`).
    pub kernel: String,
    /// `x86_64`, `aarch64`, …
    pub arch: String,
    /// The CPU's model.
    pub cpu: String,
    /// The number of logical cores.
    pub cores: usize,
    /// Memory, MiB.
    pub ram_mb: u64,
    /// The GPU, where a tool names it.
    pub gpu: String,
    /// `XDG_SESSION_TYPE`.
    pub session: String,
    /// `XDG_CURRENT_DESKTOP`.
    pub desktop: String,
    /// Whether `WAYLAND_DISPLAY` is set.
    pub wayland: bool,
    /// Whether `DISPLAY` is set.
    pub x11: bool,
}

/// One profile of the level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    /// `figure`, `native`, `nogpu`, `nogpu-x11`, `4cores`, `2cores`.
    pub name: String,
    /// What it runs, in a line.
    pub what: String,
    /// Its plan.
    pub plan: String,
    /// The renderer asked for (`auto`: iced's choice).
    pub backend: String,
    /// The renderers its runs ran on.
    pub ran: Vec<String>,
    /// `desktop` or `weak`.
    pub budgets: String,
    /// How its cores were limited.
    pub limit: Option<String>,
    /// Whether its windows opened on X11 through XWayland.
    pub xwayland: bool,
    /// Whether it ran headless.
    pub headless: bool,
    /// Whether it ran.
    pub status: Status,
    /// Why it was skipped, or what failed.
    pub reason: Option<String>,
    /// Its outcome line.
    pub outcome: Option<String>,
    /// Its budgets' verdicts.
    pub checks: Counts,
    /// Its cells (the widget's profiles).
    pub cells: Vec<Cell>,
    /// Its figures (the figure profile).
    #[serde(default)]
    pub figures: Vec<Figure>,
}

/// One figure of the figure profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Figure {
    /// `trace`, `scatter100k`, `line10m`, `preset-pie`, …
    pub name: String,
    /// What it draws.
    pub what: String,
    /// Its size, logical px.
    pub size: [f64; 2],
    /// The process's peak resident memory while its stages ran, MiB
    /// (Linux); `None` elsewhere.
    pub rss_peak_mb: Option<f64>,
    /// Why it has no timings: a stage's error, a panic, or its process
    /// ended or was killed.
    pub failed: Option<String>,
    /// Why it did not run: it needs more memory than the machine has.
    #[serde(default)]
    pub skipped: Option<String>,
    /// Its stages, in the order they ran.
    pub stages: Vec<Timing>,
}

/// One stage of a figure over its repetitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timing {
    /// `build`, `resolve`, `list`, `svg`, `pdf`, `png-1x`, `png-2x`, `gif`.
    pub stage: String,
    /// The first repetition's time, cold, in ms.
    pub cold_ms: f64,
    /// The median of the repetitions after it, ms.
    pub median_ms: f64,
    /// Their least, ms.
    pub min_ms: f64,
    /// Their 90th percentile, ms.
    pub p90_ms: f64,
    /// The repetitions after the cold one.
    pub repetitions: u32,
    /// The output's size in bytes, for the output stages.
    pub bytes: Option<u64>,
}

/// Whether a profile ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Every run wrote its records.
    Done,
    /// Some runs, or the profile's process, failed (`reason` says).
    Failed,
    /// The machine cannot run it (`reason` says).
    Skipped,
    /// Not run yet: the level stopped before it.
    Pending,
}

/// Budgets met, missed and inconclusive.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counts {
    /// Met.
    pub met: usize,
    /// Missed.
    pub missed: usize,
    /// Inconclusive.
    pub inconclusive: usize,
}

/// One cell of a profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    /// The scenario.
    pub scenario: String,
    /// The workload.
    pub workload: String,
    /// The scale factor.
    pub factor: f32,
    /// The number of runs planned.
    pub runs: u32,
    /// The number of runs that wrote their records.
    pub read: u32,
    /// Whether the cell applies. False shows as "n/a", where the workload
    /// has no such cell.
    pub applies: bool,
    /// Each per-run statistic's median across runs (the names of
    /// `summary.csv`).
    pub metrics: BTreeMap<String, f64>,
    /// The budgets the cell is held to.
    pub budgets: Vec<Check>,
}

/// One budget of a cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Check {
    /// `frame p99`, `pan latency p95`, `zoom latency p95` or
    /// `preview share`, with `pan: ` or `zoom: ` for `4k`'s halves.
    pub budget: String,
    /// Its limit (ms, or a share).
    pub limit: f64,
    /// The number of runs read.
    pub n: usize,
    /// The per-run statistic's mean.
    pub mean: Option<f64>,
    /// Its 95% interval, from two runs on.
    pub ci: Option<[f64; 2]>,
    /// `met`, `missed` or `inconclusive`.
    pub verdict: String,
}

/// A metric `compare` and the level's report show per cell: name,
/// heading, digits, and whether it is a share.
pub type Key = (&'static str, &'static str, usize, bool);

/// The metrics `compare` and the level's report show per cell of a
/// profile on a display.
pub const KEY: [Key; 7] = [
    ("frame_p50_ms", "frame p50 ms", 1, false),
    ("frame_p99_ms", "frame p99 ms", 1, false),
    ("latency_p95_ms", "latency p95 ms", 0, false),
    ("preview_share", "preview share", 0, true),
    ("drops", "drops", 0, false),
    ("ui_frame_p99_ms", "UI frame p99 ms", 1, false),
    ("cpu_s_per_s", "CPU s/s", 2, false),
];

/// The same for the software profile, whose frames are composed offscreen:
/// the frame's cost and its parts, then latency.
pub const SOFTWARE_KEY: [Key; 8] = [
    ("cost_p50_ms", "cost p50 ms", 1, false),
    ("cost_p99_ms", "cost p99 ms", 1, false),
    ("ui_p99_ms", "UI p99 ms", 1, false),
    ("compose_p99_ms", "composition p99 ms", 1, false),
    ("wait_mean_ms", "pool wait mean ms", 1, false),
    ("latency_p95_ms", "latency p95 ms", 0, false),
    ("preview_share", "preview share", 0, true),
    ("cpu_s_per_s", "CPU s/s", 2, false),
];

impl Version {
    /// This build's version, from the working directory's checkout.
    pub fn here() -> Version {
        Version {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            commit: system::git_rev(),
            iced: system::ICED_REV.to_owned(),
        }
    }
}

impl Machine {
    /// This machine's description.
    pub fn here() -> Machine {
        Machine {
            os: std::env::consts::OS.to_owned(),
            release: system::release(),
            kernel: system::kernel(),
            arch: std::env::consts::ARCH.to_owned(),
            cpu: system::cpu(),
            cores: std::thread::available_parallelism().map_or(1, std::num::NonZero::get),
            ram_mb: system::ram_mb(),
            gpu: system::gpu(),
            session: system::env("XDG_SESSION_TYPE"),
            desktop: system::env("XDG_CURRENT_DESKTOP"),
            wayland: system::is_set("WAYLAND_DISPLAY"),
            x11: system::is_set("DISPLAY"),
        }
    }
}

impl Profile {
    /// The profile `name` before it runs: pending, or skipped for
    /// `reason`.
    pub fn planned(name: &str, what: String, reason: Option<String>) -> Profile {
        Profile {
            name: name.to_owned(),
            what,
            plan: String::new(),
            backend: String::new(),
            ran: Vec::new(),
            budgets: String::new(),
            limit: None,
            xwayland: false,
            headless: false,
            status: if reason.is_some() {
                Status::Skipped
            } else {
                Status::Pending
            },
            reason,
            outcome: None,
            checks: Counts::default(),
            cells: Vec::new(),
            figures: Vec::new(),
        }
    }
}

impl Profile {
    /// The metrics its cells are shown by: the software profile's
    /// (its plan `software`), or those of a profile on a display.
    pub fn key(&self) -> &'static [Key] {
        if self.plan == "software" {
            &SOFTWARE_KEY
        } else {
            &KEY
        }
    }
}

impl Cell {
    /// `scenario × workload × factor`.
    pub fn name(&self) -> String {
        format!("{} × {} × {}", self.scenario, self.workload, self.factor)
    }
}

impl Results {
    /// The level's `report.md`: the machine, the display, flags, then
    /// every profile's budgets met and missed with its key metrics, and
    /// at the default and full levels the rule's outcome.
    pub fn markdown(&self) -> String {
        let mut md = String::new();
        if self.format == 0 {
            let _ = writeln!(md, "# rustd3 benchmark, {} ({})\n", self.stamp, self.level);
        } else if self.figure_only {
            let _ = writeln!(md, "# bench_figure --{}, {}\n", self.level, self.stamp);
        } else {
            let _ = writeln!(md, "# bench_all --{}, {}\n", self.level, self.stamp);
        }
        let commit = &self.rustd3.commit[..12.min(self.rustd3.commit.len())];
        let iced = &self.rustd3.iced[..7.min(self.rustd3.iced.len())];
        let _ = if self.figure_only {
            writeln!(
                md,
                "rustd3 {} at {commit}: the plotting library alone, the CPU reference and the \
                 figure profile, no window. `figure/report.md` holds the figures' full timings.\n",
                self.rustd3.version
            )
        } else {
            writeln!(
                md,
                "rustd3 {} at {commit} (iced {iced}). Each profile's directory holds its full \
                 report (`<profile>/report.md`: the readings, attribution, memory and flags) and \
                 its raw records.\n",
                self.rustd3.version
            )
        };
        if self.level == "full" {
            let _ = writeln!(
                md,
                "`also.md` holds every `also` measurement (the items beside the scenarios).\n"
            );
        }
        if !self.complete {
            let pending: Vec<&str> = self
                .profiles
                .iter()
                .filter(|p| p.status == Status::Pending)
                .map(|p| p.name.as_str())
                .collect();
            let _ = writeln!(
                md,
                "**Incomplete:** the level stopped before {}; this archive holds what \
                 finished.\n",
                if pending.is_empty() {
                    "its end".to_owned()
                } else {
                    pending.join(", ")
                }
            );
        }
        self.machine_table(&mut md);
        self.reference(&mut md);
        self.flags(&mut md);
        if let Some(experiment) = &self.experiment {
            let _ = writeln!(
                md,
                "## Experiment\n\nThis level ran with experiment flags, to answer one \
                 question: its profiles and cells are not the level's usual ones.\n"
            );
            for line in experiment.describe() {
                let _ = writeln!(md, "- {line}");
            }
            let _ = writeln!(md);
        }
        let _ = writeln!(md, "## Profiles\n");
        for profile in &self.profiles {
            self.profile(&mut md, profile);
        }
        self.sweeps(&mut md);
        if self.level != "quick" {
            let _ = writeln!(md, "## Rule outcome\n");
            let rule = |p: &&Profile| p.outcome.is_some() && p.name != figure::NAME;
            for profile in self.profiles.iter().filter(rule) {
                let _ = writeln!(
                    md,
                    "- {}: **{}**",
                    profile.name,
                    profile.outcome.as_deref().unwrap_or_default()
                );
            }
            let _ = writeln!(
                md,
                "\nThe native profile's line is the rule's outcome; the weak profiles' are the \
                 weak profile's.\n"
            );
        }
        md
    }

    /// Each swept profile's cells side by side, a column per value.
    fn sweeps(&self, md: &mut String) {
        let mut bases: Vec<&str> = Vec::new();
        for profile in &self.profiles {
            if let (base, Some(_)) = experiment::split(&profile.name)
                && !bases.contains(&base)
            {
                bases.push(base);
            }
        }
        for base in bases {
            let variants: Vec<&Profile> = self
                .profiles
                .iter()
                .filter(|p| experiment::split(&p.name).0 == base)
                .filter(|p| matches!(p.status, Status::Done | Status::Failed))
                .collect();
            let Some(first) = variants.first() else {
                continue;
            };
            let _ = writeln!(
                md,
                "## Sweep: {base}\n\nEach cell's key metrics per value, side by side.\n"
            );
            let mut heading = String::from("| Cell | metric |");
            let mut rule = String::from("| --- | --- |");
            for variant in &variants {
                let value = experiment::split(&variant.name).1.unwrap_or_default();
                let _ = write!(heading, " {value} |");
                rule.push_str(" --- |");
            }
            let _ = writeln!(md, "{heading}\n{rule}");
            for cell in first.cells.iter().filter(|c| c.applies) {
                for (index, &(key, name, digits, share)) in first.key().iter().enumerate() {
                    let shown = if index == 0 {
                        cell.name()
                    } else {
                        String::new()
                    };
                    let mut row = format!("| {shown} | {name} |");
                    for variant in &variants {
                        let found = variant
                            .cells
                            .iter()
                            .find(|c| c.applies && c.name() == cell.name())
                            .and_then(|c| c.metrics.get(key).copied());
                        let _ = write!(row, " {} |", value(found, digits, share));
                    }
                    let _ = writeln!(md, "{row}");
                }
            }
            let _ = writeln!(md);
        }
    }

    /// The machine and the display.
    fn machine_table(&self, md: &mut String) {
        let m = &self.machine;
        let set = |set: bool| if set { "set" } else { "unset" };
        let _ = writeln!(
            md,
            "## Machine\n\n| | |\n| --- | --- |\n\
             | System | {} ({}), kernel {}, {} |\n\
             | CPU | {}, {} cores |\n\
             | Memory | {} |\n\
             | GPU | {} |\n\
             | Session | {}, desktop {}; WAYLAND_DISPLAY {}, DISPLAY {} |\n\
             | Display | {} |\n",
            m.release,
            m.os,
            m.kernel,
            m.arch,
            m.cpu,
            m.cores,
            memory(m.ram_mb),
            m.gpu,
            m.session,
            m.desktop,
            set(m.wayland),
            set(m.x11),
            if self.figure_only {
                "not measured (--figure-only)".to_owned()
            } else {
                self.display.describe()
            }
        );
    }

    /// The CPU reference's timings.
    fn reference(&self, md: &mut String) {
        if self.cpu_reference.is_empty() {
            return;
        }
        let _ = writeln!(
            md,
            "## CPU reference\n\nSingle-threaded, headless, at factor 1: medians of a few \
             repetitions, which put this machine on a scale with others.\n\n\
             | Timing | median ms | of |\n| --- | --- | --- |"
        );
        for timing in &self.cpu_reference {
            let ms = timing
                .ms
                .map_or_else(|| "not measured".to_owned(), |ms| format!("{ms:.2}"));
            let _ = writeln!(md, "| {} | {ms} | {} |", timing.what, timing.repetitions);
        }
        let _ = writeln!(md);
    }

    /// What the reader must know before the numbers: headless, an
    /// unpaced display, skipped and failed profiles.
    fn flags(&self, md: &mut String) {
        let mut flags = Vec::new();
        if let Some(why) = self.display.headless.as_ref().filter(|_| !self.figure_only) {
            flags.push(format!(
                "Headless ({why}): no window opened, so every number is report-only (headless \
                 frames are virtual, paced at {} Hz).",
                self.display.refresh_hz
            ));
        }
        if self.display.unpaced() {
            flags.push(format!(
                "The display paced no frames (a VM, Xvfb or a remote desktop): the level ran at \
                 {} Hz, and every frame budget assumes that rate.",
                self.display.refresh_hz
            ));
        }
        for profile in &self.profiles {
            match profile.status {
                Status::Skipped | Status::Failed => flags.push(format!(
                    "{} {}: {}.",
                    profile.name,
                    if profile.status == Status::Skipped {
                        "skipped"
                    } else {
                        "failed"
                    },
                    profile.reason.as_deref().unwrap_or("no reason recorded")
                )),
                Status::Done | Status::Pending => {}
            }
        }
        if flags.is_empty() {
            return;
        }
        let _ = writeln!(md, "## Flags\n");
        for flag in flags {
            let _ = writeln!(md, "- {flag}");
        }
        let _ = writeln!(md);
    }

    /// One profile's section.
    fn profile(&self, md: &mut String, profile: &Profile) {
        let _ = writeln!(md, "### {}\n\n{}.", profile.name, profile.what);
        match profile.status {
            Status::Skipped => {
                let _ = writeln!(
                    md,
                    "\nSkipped: {}.\n",
                    profile.reason.as_deref().unwrap_or_default()
                );
                return;
            }
            Status::Pending => {
                let _ = writeln!(md, "\nNot run: the level stopped before it.\n");
                return;
            }
            Status::Done | Status::Failed => {}
        }
        if profile.name == figure::NAME {
            let _ = writeln!(md, "\n{}", figure::summary(profile));
            return;
        }
        let c = profile.checks;
        let _ = writeln!(
            md,
            "Ran on {}{}{}. {} checks: {} met, {} missed, {} inconclusive. Outcome: **{}**\n",
            if profile.ran.is_empty() {
                "an unrecorded renderer".to_owned()
            } else {
                profile.ran.join(", ")
            },
            profile
                .limit
                .as_deref()
                .map_or_else(String::new, |limit| format!(", {limit}")),
            if profile.headless { ", headless" } else { "" },
            c.met + c.missed + c.inconclusive,
            c.met,
            c.missed,
            c.inconclusive,
            profile.outcome.as_deref().unwrap_or("none")
        );
        let mut heading = String::from("| Cell | runs |");
        let mut rule = String::from("| --- | --- |");
        for (_, name, _, _) in profile.key() {
            let _ = write!(heading, " {name} |");
            rule.push_str(" --- |");
        }
        let _ = writeln!(md, "{heading} budgets |\n{rule} --- |");
        let assumed =
            (self.display.unpaced() && !profile.headless).then_some(self.display.refresh_hz);
        for cell in &profile.cells {
            if !cell.applies {
                continue;
            }
            let mut row = format!("| {} | {}/{} |", cell.name(), cell.read, cell.runs);
            for &(key, _, digits, share) in profile.key() {
                let _ = write!(
                    row,
                    " {} |",
                    value(cell.metrics.get(key).copied(), digits, share)
                );
            }
            let budgets: Vec<String> = cell
                .budgets
                .iter()
                .map(|check| verdict(check, assumed))
                .collect();
            let budgets = if budgets.is_empty() {
                "—".to_owned()
            } else {
                budgets.join("; ")
            };
            let _ = writeln!(md, "{row} {budgets} |");
        }
        let _ = writeln!(md);
    }
}

/// Memory in GiB, or "unknown".
pub fn memory(ram_mb: u64) -> String {
    if ram_mb == 0 {
        "unknown".to_owned()
    } else {
        format!("{:.1} GiB", ram_mb as f64 / 1024.0)
    }
}

/// A metric's value, or "n/a".
pub fn value(value: Option<f64>, digits: usize, share: bool) -> String {
    match value {
        Some(v) if share => format!("{:.digits$}%", 100.0 * v),
        Some(v) => format!("{v:.digits$}"),
        None => "n/a".to_owned(),
    }
}

/// A budget's verdict in a line, with its statistic against its limit. A
/// frame budget notes the rate it assumes on an unpaced display.
fn verdict(check: &Check, assumed: Option<f64>) -> String {
    let share = check.budget.ends_with("preview share");
    let mean = value(check.mean, if share { 0 } else { 1 }, share);
    let limit = value(Some(check.limit), if share { 0 } else { 1 }, share);
    let mark = match check.verdict.as_str() {
        "missed" => "**missed**",
        other => other,
    };
    let note = match assumed {
        Some(hz) if check.budget.ends_with("frame p99") => format!(" (assumes {hz} Hz)"),
        Some(_) | None => String::new(),
    };
    format!("{} {mean} (≤ {limit}) {mark}{note}", check.budget)
}

#[cfg(test)]
pub mod tests {
    use std::collections::BTreeMap;

    use super::{Cell, Check, Counts, Figure, Machine, Profile, Results, Status, Timing, Version};
    use crate::display::{Display, Measured};

    /// A cell of `scenario` on `trace` at 1 with the key metrics scaled
    /// by `k`, holding a frame budget.
    pub fn cell(scenario: &str, k: f64) -> Cell {
        let metrics: BTreeMap<String, f64> = [
            ("frame_p50_ms", 16.7),
            ("frame_p99_ms", 33.4),
            ("latency_p95_ms", 120.0),
            ("preview_share", 0.05),
            ("drops", 12.0),
            ("ui_frame_p99_ms", 8.0),
            ("cpu_s_per_s", 1.5),
        ]
        .into_iter()
        .map(|(name, v)| (name.to_owned(), v * k))
        .collect();
        Cell {
            scenario: scenario.to_owned(),
            workload: "trace".to_owned(),
            factor: 1.0,
            runs: 3,
            read: 3,
            applies: true,
            metrics,
            budgets: vec![Check {
                budget: "frame p99".to_owned(),
                limit: 25.0,
                n: 3,
                mean: Some(33.4 * k),
                ci: Some([30.0 * k, 36.0 * k]),
                verdict: if k > 0.7 { "missed" } else { "met" }.to_owned(),
            }],
        }
    }

    /// The results of a small synthetic quick level whose metrics are
    /// scaled by `k`, on a machine of `cores` cores.
    pub fn results(k: f64, cores: usize, display: Display) -> Results {
        let profile = |name: &str, cells: Vec<Cell>| Profile {
            plan: "quick".to_owned(),
            backend: "auto".to_owned(),
            ran: vec!["wgpu".to_owned()],
            budgets: "desktop".to_owned(),
            status: Status::Done,
            outcome: Some("report-only, the quick level: …".to_owned()),
            checks: Counts {
                met: 1,
                missed: 1,
                inconclusive: 0,
            },
            cells,
            ..Profile::planned(name, format!("{name}'s cells"), None)
        };
        Results {
            format: super::FORMAT,
            rustd3: Version {
                version: "0.1.0".to_owned(),
                commit: "c9b2c9d0123456789".to_owned(),
                iced: "98bd48fe011efc0ec5819fee0fa7673a7169ff11".to_owned(),
            },
            level: "quick".to_owned(),
            stamp: "20261029T120000Z".to_owned(),
            complete: true,
            machine: Machine {
                os: "linux".to_owned(),
                release: "Pop!_OS 24.04 LTS".to_owned(),
                kernel: "6.18.0".to_owned(),
                arch: "x86_64".to_owned(),
                cpu: "AMD Ryzen 7 5800X".to_owned(),
                cores,
                ram_mb: 64_000,
                gpu: "NVIDIA GeForce RTX 3090, 595.104.02".to_owned(),
                session: "wayland".to_owned(),
                desktop: "COSMIC".to_owned(),
                wayland: true,
                x11: true,
            },
            display,
            figure_only: false,
            experiment: None,
            cpu_reference: vec![crate::reference::Reference {
                name: "trace-tile".to_owned(),
                what: "one trace tile".to_owned(),
                ms: Some(4.0 * k),
                repetitions: 7,
            }],
            profiles: vec![
                profile("native", vec![cell("pan-x", k), cell("zoom-lines", k)]),
                Profile::planned(
                    "nogpu-x11",
                    "on X11".to_owned(),
                    Some("it runs only in a Linux Wayland session".to_owned()),
                ),
            ],
        }
    }

    /// A figure profile of three figures, the first timed with its
    /// medians scaled by `k`, one failed, one skipped.
    pub fn figures(k: f64) -> Profile {
        let timing = |stage: &str, ms: f64, bytes: Option<u64>| Timing {
            stage: stage.to_owned(),
            cold_ms: 3.0 * ms,
            median_ms: ms * k,
            min_ms: 0.9 * ms,
            p90_ms: 1.2 * ms,
            repetitions: 3,
            bytes,
        };
        let figure = |name: &str| Figure {
            name: name.to_owned(),
            what: format!("{name}'s marks"),
            size: [1600.0, 1000.0],
            rss_peak_mb: None,
            failed: None,
            skipped: None,
            stages: Vec::new(),
        };
        Profile {
            plan: "figure".to_owned(),
            status: Status::Done,
            outcome: Some("report-only: 1 figures timed, 1 skipped, no budgets".to_owned()),
            figures: vec![
                Figure {
                    rss_peak_mb: Some(212.4 * k),
                    stages: vec![
                        timing("resolve", 41.3, None),
                        timing("svg", 5.5, Some(1_234_567)),
                    ],
                    ..figure("trace")
                },
                Figure {
                    failed: Some("pdf: too large".to_owned()),
                    ..figure("heatmap")
                },
                Figure {
                    skipped: Some("it needs about 13.8 GiB, more than 60%".to_owned()),
                    ..figure("scatter10m")
                },
            ],
            ..Profile::planned("figure", "3 figures".to_owned(), None)
        }
    }

    /// A display measured at `hz` (`None`: unpaced), factor 1, wgpu.
    pub fn display(hz: Option<f64>) -> Display {
        Display::decide(
            None,
            None,
            Ok(Measured {
                hz,
                factor: Some(1.0),
                backend: Some("wgpu".to_owned()),
            }),
        )
    }

    #[test]
    fn the_report_shows_the_machine_flags_and_every_profiles_budgets() {
        let md = results(1.0, 16, display(Some(60.0))).markdown();
        assert!(
            md.starts_with("# bench_all --quick, 20261029T120000Z"),
            "{md}"
        );
        assert!(md.contains("| CPU | AMD Ryzen 7 5800X, 16 cores |"), "{md}");
        assert!(md.contains("| one trace tile | 4.00 | 7 |"), "{md}");
        assert!(md.contains("60 Hz (measured from idle frames)"), "{md}");
        assert!(md.contains("- nogpu-x11 skipped: it runs only in a Linux Wayland session."));
        assert!(
            md.contains("| pan-x × trace × 1 | 3/3 | 16.7 | 33.4 | 120 | 5% | 12 | 8.0 | 1.50 |"),
            "{md}"
        );
        assert!(md.contains("frame p99 33.4 (≤ 25.0) **missed** |"), "{md}");
        assert!(
            !md.contains("Rule outcome"),
            "the quick level has no rule outcome"
        );
        // Unpaced: every frame budget says the rate it assumes.
        let unpaced = results(1.0, 2, display(None)).markdown();
        assert!(
            unpaced.contains("every frame budget assumes that rate"),
            "{unpaced}"
        );
        assert!(unpaced.contains("**missed** (assumes 60 Hz)"), "{unpaced}");
        // An interrupted level says what it holds.
        let mut cut = results(1.0, 2, display(None));
        cut.complete = false;
        cut.profiles
            .push(Profile::planned("nogpu", String::new(), None));
        assert!(
            cut.markdown()
                .contains("**Incomplete:** the level stopped before nogpu")
        );
        // results.json round-trips.
        let json = serde_json::to_string_pretty(&cut).unwrap();
        assert_eq!(serde_json::from_str::<Results>(&json).unwrap(), cut);
        assert!(json.contains("\"format\": 2"));
    }

    #[test]
    fn an_experiment_is_said_and_a_sweep_shows_its_values_side_by_side() {
        let mut r = results(1.0, 16, display(Some(60.0)));
        r.experiment = Some(crate::experiment::Experiment {
            only: Some(vec!["native".to_owned()]),
            sweep: Some(crate::experiment::Sweep::parse("threads=1,8").unwrap()),
            ..crate::experiment::Experiment::default()
        });
        let native = r.profiles.remove(0);
        for (k, threads) in [(1.0, "1"), (2.0, "8")] {
            let mut variant = native.clone();
            variant.name = format!("native@threads={threads}");
            variant.cells = vec![cell("pan-x", k)];
            r.profiles.push(variant);
        }
        let md = r.markdown();
        assert!(md.contains("## Experiment"), "{md}");
        assert!(md.contains("- only the profiles native"), "{md}");
        assert!(
            md.contains("- a sweep of threads over 1, 8, a profile per value"),
            "{md}"
        );
        assert!(md.contains("### native@threads=8"), "{md}");
        assert!(
            md.contains(
                "## Sweep: native\n\nEach cell's key metrics per value, side by side.\n\n\
                         | Cell | metric | threads=1 | threads=8 |"
            ),
            "{md}"
        );
        assert!(
            md.contains("| pan-x × trace × 1 | frame p50 ms | 16.7 | 33.4 |"),
            "{md}"
        );
        let json = serde_json::to_string_pretty(&r).unwrap();
        assert!(json.contains("\"sweep\""), "{json}");
        assert_eq!(serde_json::from_str::<Results>(&json).unwrap(), r);
    }

    #[test]
    fn the_figure_profile_shows_its_figures_and_no_rule_outcome() {
        let mut r = results(1.0, 16, display(Some(60.0)));
        r.level = "default".to_owned();
        r.profiles.insert(0, figures(1.0));
        let md = r.markdown();
        assert!(
            md.contains("### figure\n\n3 figures.\n\nMedian ms of 3 repetitions"),
            "{md}"
        );
        assert!(md.contains("| trace | 1600 × 1000 | 41.3 | 5.50 | 212 MiB |"));
        let outcome = md.split("## Rule outcome").nth(1).unwrap();
        assert!(outcome.contains("- native:") && !outcome.contains("figure"));
        // Format 1, from before the figure profile, reads without figures.
        let mut old = serde_json::to_value(results(1.0, 2, display(None))).unwrap();
        old["format"] = 1.into();
        for profile in old["profiles"].as_array_mut().unwrap() {
            profile.as_object_mut().unwrap().remove("figures");
        }
        let read: Results = serde_json::from_value(old).unwrap();
        assert_eq!(read.format, 1);
        assert!(read.profiles.iter().all(|p| p.figures.is_empty()));
    }

    #[test]
    fn a_figure_only_level_says_so_and_flags_no_display() {
        let mut r = results(1.0, 2, display(None));
        r.figure_only = true;
        r.display = Display::decide(None, None, Err("--figure-only".to_owned()));
        r.profiles = vec![figures(1.0)];
        let md = r.markdown();
        assert!(
            md.starts_with("# bench_figure --quick, 20261029T120000Z"),
            "{md}"
        );
        assert!(md.contains("the plotting library alone"), "{md}");
        assert!(
            md.contains("| Display | not measured (--figure-only) |"),
            "{md}"
        );
        assert!(!md.contains("Headless"), "{md}");
        assert!(md.contains("### figure"), "{md}");
    }
}
