//! [`Level`]: the benchmark's one entry point (D22.9),
//! `rustd3-bench [--quick | --default | --full]`, the same on Linux,
//! Windows and macOS. A level prints what it will run and an estimate,
//! takes the CPU reference, times the plotting library alone (the
//! `figure` profile, D22.10), and around them runs the widget's part
//! ([`Widget`]: the display measurement first, the profiles that drive
//! the `Plot` widget after), unless `--figure-only` or a build without the
//! `widget` feature leaves it out. Every step lands in
//! `bench-results/<UTC stamp>-<level>/`, packed into one archive after
//! each, and nothing stops a level.

#[cfg(feature = "widget")]
pub mod widget;

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::archive;
use crate::display::Display;
use crate::figure;
use crate::privacy::{self, Identity};
use crate::reference;
use crate::results::{self, Results};
use crate::system;

/// How much a level runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// About 16 minutes: what an issue report needs; report-only.
    Quick,
    /// About 95 minutes: the software profile, the registered verdict run
    /// and the weak profiles.
    Default,
    /// Hours: every cell on the native backend, the weak profiles and
    /// every `also` measurement.
    Full,
}

/// What the command line asks of a level.
#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    /// `--quick`, `--default` (the default) or `--full`.
    pub level: Level,
    /// `--out`: where the stamped directory goes.
    pub out: PathBuf,
    /// `--refresh-hz`: the display's rate, instead of measuring it.
    pub refresh_hz: Option<f64>,
    /// `--native-factor`: the display's factor, instead of the window's.
    pub native: Option<f32>,
    /// `--figure-only`: the plotting library alone, the CPU reference and
    /// the figure profile, no window (D22.10 (2)).
    pub figure_only: bool,
    /// `--dry-run` (hidden): print the plan and the estimate, run nothing.
    pub dry_run: bool,
    /// `--frames DIR`: the software profile writes its composed frames as
    /// PNGs under DIR (D22.12).
    pub frames: Option<PathBuf>,
    /// `--every N`: with `--frames`, every N-th frame.
    pub every: u32,
}

/// The widget's part of a level, in a build with the `widget` feature
/// (`widget::Part`): the display measurement before the CPU reference,
/// then, after the figure profile, the profiles that drive the `Plot`
/// widget, and at `--full` the `also` measurements.
pub trait Widget {
    /// What it will run and the level's estimate with it.
    fn outline(&self) -> Outline;
    /// Measures the display into `dir`; its profiles are known from then.
    fn display(&mut self, dir: &Path) -> Result<Display, Box<dyn Error>>;
    /// Its profiles before they run: pending, or skipped with why.
    fn profiles(&self) -> Vec<results::Profile>;
    /// Runs its profile `index` into `dir` (one skipped comes back as it
    /// was planned), noting how it ran in `notes` (`system.txt`).
    fn run(
        &mut self,
        index: usize,
        dir: &Path,
        notes: &mut String,
    ) -> Result<results::Profile, Box<dyn Error>>;
    /// What runs after its profiles, into `dir`.
    fn finish(&mut self, dir: &Path, notes: &mut String) -> Result<(), Box<dyn Error>>;
}

/// What the widget's part will run, before anything runs.
#[derive(Debug, Clone, PartialEq)]
pub struct Outline {
    /// Its lines before the CPU reference's: the display measurement.
    pub before: Vec<String>,
    /// Its lines after the figure profile's: its profiles, `also`.
    pub after: Vec<String>,
    /// The whole level's estimated wall time with it, s.
    pub estimate_s: f64,
    /// How its estimate is made, for the estimate's line.
    pub note: String,
}

/// The CPU reference's allowance in the estimate, s (about 5 here, on
/// two cores at 2.1 GHz).
const REFERENCE_S: f64 = 15.0;

/// Why a level without the widget's part has no display.
const FIGURE_ONLY: &str = "the plotting library alone (--figure-only): no display measurement \
                           and no window";

impl Level {
    /// The level named `name` (`quick`, `default`, `full`).
    pub fn parse(name: &str) -> Option<Level> {
        [Level::Quick, Level::Default, Level::Full]
            .into_iter()
            .find(|level| level.name() == name)
    }

    /// Its flag's name.
    pub fn name(self) -> &'static str {
        match self {
            Level::Quick => "quick",
            Level::Default => "default",
            Level::Full => "full",
        }
    }

    /// What it is, in a sentence.
    pub fn describe(self) -> &'static str {
        match self {
            Level::Quick => {
                "what a performance issue needs, report-only: the figure profile, the software \
                 path offscreen, the native backend's cells, then the software renderer's"
            }
            Level::Default => {
                "the figure profile with its ladders, the software path offscreen, the \
                 registered verdict run (D22.1's rule outcome) and the weak profiles (D22.4)"
            }
            Level::Full => {
                "the figure profile with its ladders and GIFs, the software path offscreen, every \
                 scenario × workload × factor on the native backend, the weak profiles and \
                 every `also` measurement"
            }
        }
    }

    /// The estimated wall time of the CPU reference and the figure
    /// profile on a machine of `ram_mb` MiB, s.
    pub fn library_s(self, ram_mb: u64) -> f64 {
        REFERENCE_S + figure::Plan::of(self, ram_mb).estimate_s()
    }
}

impl Request {
    /// Runs the level with the bench at `exe`, and with `widget`'s part
    /// when given; without it, the plotting library alone: the CPU
    /// reference and the figure profile, no window.
    pub fn run(
        &self,
        exe: &Path,
        mut widget: Option<&mut dyn Widget>,
    ) -> Result<(), Box<dyn Error>> {
        let level = self.level;
        let ram_mb = system::ram_mb();
        let figures = figure::Plan::of(level, ram_mb);
        let outline = widget.as_ref().map(|widget| widget.outline());
        let name = match outline {
            Some(_) => level.name().to_owned(),
            None => format!("figure-{}", level.name()),
        };
        match outline {
            Some(_) => println!("rustd3-bench --{}: {}.", level.name(), level.describe()),
            None => println!(
                "rustd3-bench --{} --figure-only: the plotting library alone, headless.",
                level.name()
            ),
        }
        for line in outline.iter().flat_map(|o| &o.before) {
            println!("  {line}");
        }
        println!("  cpu        three single-threaded timings: the CPU reference");
        println!(
            "  {:<10} {}, about {:.1} min",
            figure::NAME,
            figures.describe(),
            figures.estimate_s() / 60.0
        );
        for line in outline.iter().flat_map(|o| &o.after) {
            println!("  {line}");
        }
        let (estimate, note) = match &outline {
            Some(o) => (o.estimate_s, format!("{}; ", o.note)),
            None => (level.library_s(ram_mb), String::new()),
        };
        let minutes = (estimate / 60.0).round().max(1.0);
        println!(
            "Estimate: about {minutes:.0} minute{} ({note}the figures as timed on two cores at \
             2.1 GHz; machines vary).",
            if minutes > 1.0 { "s" } else { "" }
        );
        if self.dry_run {
            println!("--dry-run: nothing runs.");
            return Ok(());
        }
        if widget.is_some() {
            println!("Keep hands off the mouse and keyboard while windows open and close.");
        }
        let _power = Power::performance();
        let stamp = system::stamp();
        let dir = self.out.join(format!("{stamp}-{name}"));
        let archive = self.out.join(format!("rustd3-bench-{name}-{stamp}.tar.gz"));
        fs::create_dir_all(&dir)?;
        let display = match widget.as_mut() {
            Some(widget) => widget.display(&dir)?,
            None => Display::decide(None, None, Err(FIGURE_ONLY.to_owned())),
        };
        let mut results = Results {
            format: results::FORMAT,
            rustd3: results::Version::here(),
            level: level.name().to_owned(),
            stamp,
            complete: false,
            machine: results::Machine::here(),
            display: display.clone(),
            cpu_reference: Vec::new(),
            figure_only: widget.is_none(),
            profiles: std::iter::once(results::Profile::planned(
                figure::NAME,
                figures.describe(),
                None,
            ))
            .chain(widget.iter().flat_map(|widget| widget.profiles()))
            .collect(),
        };
        let mut notes = match widget {
            Some(_) => format!(
                "# rustd3-bench --{}: {}\n",
                level.name(),
                display.describe()
            ),
            None => format!("# rustd3-bench --{} --figure-only\n", level.name()),
        };
        let diagnostics = system::diagnostics();
        let identity = Identity::here();
        let save = |results: &Results, notes: &str| {
            let system = format!("{notes}\n{diagnostics}");
            save(&dir, &archive, results, &system, &identity)
        };
        println!("== cpu: the CPU reference, single-threaded timings");
        let begin = std::time::Instant::now();
        results.cpu_reference = reference::measure();
        println!("  ({:.1} s)", begin.elapsed().as_secs_f64());
        for timing in &results.cpu_reference {
            let ms = timing
                .ms
                .map_or_else(|| "not measured".to_owned(), |ms| format!("{ms:.2} ms"));
            println!("  {:<17} {ms}", timing.name);
        }
        // An archive after every step: std cannot catch Ctrl-C portably,
        // so an interrupted level leaves the last whole one.
        save(&results, &notes)?;
        println!(
            "== {}: the plotting library alone, headless (figure/report.md)",
            figure::NAME
        );
        results.profiles[0] = figures.run(exe, &dir)?;
        save(&results, &notes)?;
        if let Some(widget) = widget.as_mut() {
            for index in 1..results.profiles.len() {
                results.profiles[index] = widget.run(index - 1, &dir, &mut notes)?;
                save(&results, &notes)?;
            }
            widget.finish(&dir, &mut notes)?;
        }
        results.complete = true;
        let (files, size) = save(&results, &notes)?;
        println!("\nResults: {}", dir.display());
        for profile in &results.profiles {
            match (&profile.outcome, &profile.reason) {
                (Some(outcome), None) => println!("  {}: {outcome}", profile.name),
                (Some(outcome), Some(why)) => println!("  {}: {outcome} ({why})", profile.name),
                (None, Some(why)) => println!("  {}: {why}", profile.name),
                (None, None) => {}
            }
        }
        let size = if size < 1_000_000 {
            format!("{:.0} kB", size as f64 / 1e3)
        } else {
            format!("{:.1} MB", size as f64 / 1e6)
        };
        println!(
            "The file to attach to an issue ({files} files, {size}; PRIVACY.md inside says what it \
             holds):"
        );
        println!("{}", archive.display());
        Ok(())
    }
}

/// Writes the level's own files into `dir` (`system.txt`, `results.json`,
/// `report.md`, `PRIVACY.md`), rewrites every text file under it through
/// `identity`'s scrub, and packs it into `archive`: the archive's files,
/// read back, and its bytes.
fn save(
    dir: &Path,
    archive: &Path,
    results: &Results,
    system: &str,
    identity: &Identity,
) -> Result<(usize, u64), Box<dyn Error>> {
    fs::write(dir.join("system.txt"), system)?;
    fs::write(
        dir.join("results.json"),
        serde_json::to_string_pretty(results)?,
    )?;
    fs::write(dir.join("report.md"), results.markdown())?;
    fs::write(dir.join("PRIVACY.md"), privacy::PRIVACY)?;
    identity.scrub_dir(dir)?;
    let size = archive::pack(dir, archive)?;
    Ok((archive::read(archive)?.len(), size))
}

/// The `system76-power` profile while a level runs: `performance`, the
/// one before restored when dropped; nothing where the tool is absent.
struct Power {
    previous: Option<String>,
}

impl Power {
    fn performance() -> Power {
        let Ok(out) = Command::new("system76-power").arg("profile").output() else {
            return Power { previous: None };
        };
        let previous = String::from_utf8_lossy(&out.stdout)
            .lines()
            .find_map(|line| line.strip_prefix("Power Profile:"))
            .map(|p| p.trim().to_lowercase());
        let set = Command::new("system76-power")
            .args(["profile", "performance"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if !set.is_ok_and(|s| s.success()) {
            println!("Could not set the performance profile; continuing.");
        }
        Power { previous }
    }
}

impl Drop for Power {
    fn drop(&mut self) {
        if let Some(previous) = &self.previous {
            let _ = Command::new("system76-power")
                .args(["profile", previous])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::save;
    use crate::archive;
    use crate::privacy::Identity;
    use crate::results::tests::{display, results};
    use crate::system;

    #[test]
    fn an_archive_holds_no_home_and_no_host_name() {
        // A results directory under a fake home, with what names paths in
        // a real one: a run's command line, its console, a panic.
        let root = std::env::temp_dir().join(format!("rustd3-privacy-{}", std::process::id()));
        let home = root.join("home").join("alice");
        let out = home.join("rustd3").join("bench-results");
        let dir = out.join("20261029T120000Z-quick");
        fs::create_dir_all(dir.join("native")).unwrap();
        let host = Identity::here()
            .host
            .filter(|host| host.len() >= 3)
            .unwrap_or_else(|| "alices-laptop".to_owned());
        let identity = Identity {
            home: Some(home.display().to_string()),
            host: Some(host.clone()),
            user: Some("alice".to_owned()),
        };
        fs::write(
            dir.join("native").join("meta.json"),
            format!(
                "{{\"command\": \"rustd3-bench run --out {}\"}}",
                dir.join("native").display()
            ),
        )
        .unwrap();
        fs::write(
            dir.join("run-native.log"),
            format!(
                "report: {}\nthread 'main' panicked at {}:1:2 on {host}\n",
                dir.join("native").join("report.md").display(),
                home.join(".cargo").join("x.rs").display()
            ),
        )
        .unwrap();
        let archive = out.join("rustd3-bench-quick-20261029T120000Z.tar.gz");
        let system = format!("# notes\n{}", system::diagnostics());
        let (files, _) = save(
            &dir,
            &archive,
            &results(1.0, 2, display(None)),
            &system,
            &identity,
        )
        .unwrap();
        let entries = archive::read(&archive).unwrap();
        assert_eq!(files, entries.len());
        let home = home.display().to_string();
        let tokens = |text: &str| {
            text.split(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_'))
                .any(|token| token == host || token == host.to_lowercase())
        };
        for entry in &entries {
            let text = String::from_utf8_lossy(&entry.data);
            assert!(
                !entry.path.contains(&home) && !text.contains(&home),
                "{}",
                entry.path
            );
            assert!(
                !tokens(&entry.path) && !tokens(&text),
                "{}: {text}",
                entry.path
            );
            assert!(!text.contains("/alice"), "{}", entry.path);
        }
        let read = |name: &str| {
            let entry = entries.iter().find(|e| e.path.ends_with(name)).unwrap();
            String::from_utf8_lossy(&entry.data).into_owned()
        };
        assert_eq!(
            read("run-native.log"),
            "report: native/report.md\nthread 'main' panicked at ~/.cargo/x.rs:1:2 on <host>\n"
        );
        assert_eq!(
            read("meta.json"),
            "{\"command\": \"rustd3-bench run --out native\"}"
        );
        assert!(read("system.txt").contains("$ uname -srm"));
        assert!(read("PRIVACY.md").starts_with("# What this archive holds"));
        assert!(read("results.json").contains("\"format\": 2"));
        fs::remove_dir_all(&root).unwrap();
    }
}
