//! `rustd3-bench suite`: the whole windowed benchmark in one command, on
//! Linux, Windows and macOS alike: the smoke plan (or with `--focus` one
//! run of the focus plan's first cell), whose windows give the display's
//! refresh rate (their idle frame interval) and scale factor; then the
//! default plan (or the focus plan) and D22.4's weak plan three times,
//! without a GPU and on 4 and 2 cores, each in its own directory of
//! `bench-results/<UTC stamp>/` with its console in `run-<plan>.log`. A
//! core count is enforced by pinning the bench where the system offers it
//! (`taskset` on Linux, `start /AFFINITY` on Windows) and otherwise by the
//! widget's render pool (`--threads`), and the report says which. In a
//! Linux Wayland session with XWayland, the no-GPU profile runs a second
//! time on X11 (`weak-nogpu-x11`, report-only, D22.8).

use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::record::{self, Record};
use crate::stats::percentile;
use crate::system;

/// What `suite` runs.
#[derive(Debug, Clone, PartialEq)]
pub struct Suite {
    /// `--focus`: the focus plan in place of smoke and default.
    pub focus: bool,
    /// `--smoke-only`: stop after smoke.
    pub smoke_only: bool,
    /// `--refresh-hz`: the display's rate, instead of measuring it.
    pub refresh_hz: Option<f64>,
    /// `--native-factor`: the factor the first windows ask for (1 by
    /// default); the factor they report is used after them.
    pub native: Option<f32>,
    /// `--out`: where the stamped directory goes.
    pub out: PathBuf,
}

/// How a weak profile's cores are limited.
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

/// The refresh rate the first windows run at when none is given; their
/// idle frames give the real one.
const PROVISIONAL_HZ: f64 = 60.0;

/// Why `weak-nogpu-x11` does not run here.
pub const NO_XWAYLAND: &str = "it runs only in a Linux Wayland session with XWayland \
                               (WAYLAND_DISPLAY and DISPLAY both set)";

impl Suite {
    /// Runs it with the bench at `exe`.
    pub fn run(&self, exe: &Path) -> Result<(), Box<dyn Error>> {
        let x11 = xwayland();
        let (weak, more) = if x11 {
            (
                "the weak profiles (about 28 min: no GPU, no GPU on X11 through XWayland \
                 (report-only), 4 cores, 2 cores)",
                7,
            )
        } else {
            (
                "the weak profiles (about 21 min: no GPU, 4 cores, 2 cores)",
                0,
            )
        };
        let plans = if self.focus {
            format!(
                "one run of the focus plan's first cell (about 2 min; it measures the display), \
                 the focus plan (about 17 min, report-only), then {weak}; about {} min in all",
                40 + more
            )
        } else if self.smoke_only {
            "the smoke plan (about 6 min; its numbers do not count)".to_owned()
        } else {
            format!(
                "the smoke plan (about 6 min; it measures the display), the default plan (about \
                 47 min, the verdict's run), then {weak}; about {} min in all",
                76 + more
            )
        };
        println!("rustd3-bench suite: {plans}.");
        println!("Keep hands off the mouse and keyboard while windows open and close.");
        let _power = Power::performance();
        let out = self.out.join(system::stamp());
        fs::create_dir_all(&out)?;
        let native = self.native.unwrap_or(1.0);
        let hz = self.refresh_hz.unwrap_or(PROVISIONAL_HZ);
        let hz_flag = hz.to_string();
        let native_flag = native.to_string();
        let display = ["--refresh-hz", &hz_flag, "--native-factor", &native_flag];
        let first = if self.focus { "probe" } else { "smoke" };
        let status = if self.focus {
            plan(
                exe,
                &out,
                "probe",
                None,
                &[
                    &[
                        "--plan",
                        "focus",
                        "--scenario",
                        "zoom-lines",
                        "--workload",
                        "trace",
                    ][..],
                    &["--factor", &native_flag, "--runs", "1"][..],
                    &display[..],
                ]
                .concat(),
            )?
        } else {
            plan(
                exe,
                &out,
                "smoke",
                None,
                &[
                    &["--plan", "smoke", "--factor", &native_flag][..],
                    &display[..],
                ]
                .concat(),
            )?
        };
        if !status.success() {
            return Err(format!(
                "the {first} plan failed (see {}); nothing else ran",
                out.join(format!("run-{first}.log")).display()
            )
            .into());
        }
        let raw = out.join(first).join("raw");
        let measured = idle_hz(&raw);
        let hz = match (self.refresh_hz, measured) {
            (Some(hz), _) => hz,
            (None, Some(measured)) => measured,
            (None, None) => {
                return Err(format!(
                    "the {first} windows gave no refresh rate (no vsync?); rerun with \
                     --refresh-hz, e.g. --refresh-hz 144"
                )
                .into());
            }
        };
        let observed = window_factor(&raw);
        let factor = observed.unwrap_or(native);
        let mut notes = format!(
            "# rustd3-bench suite: refresh rate {hz} Hz ({}), native factor {factor} (asked {native}, \
             the {first} windows reported {}); plans after {first} at {hz} Hz and factor {factor}\n",
            match (self.refresh_hz, measured) {
                (Some(_), _) => "given".to_owned(),
                (None, Some(m)) => format!("measured from the {first} windows' idle frames: {m}"),
                (None, None) => "unknown".to_owned(),
            },
            observed.map_or_else(|| "none".to_owned(), |f| f.to_string())
        );
        if (factor - native).abs() > 1e-3 {
            println!(
                "The windows' scale factor is {factor}, not {native}: the plans after {first} use {factor}."
            );
        }
        println!("Refresh rate {hz} Hz, scale factor {factor}.");
        let hz_flag = hz.to_string();
        let factor_flag = factor.to_string();
        let display = ["--refresh-hz", &hz_flag, "--native-factor", &factor_flag];
        let mut failed = Vec::new();
        if !self.smoke_only {
            let main = if self.focus { "focus" } else { "default" };
            if !plan(
                exe,
                &out,
                main,
                None,
                &[&["--plan", main], &display[..]].concat(),
            )?
            .success()
            {
                failed.push(main.to_owned());
            }
            let nogpu = [&["--plan", "weak", "--backend", "tiny-skia"], &display[..]].concat();
            if !plan(exe, &out, "weak-nogpu", None, &nogpu)?.success() {
                failed.push("weak-nogpu".to_owned());
            }
            if x11 {
                notes.push_str(
                    "# weak-nogpu-x11: weak-nogpu again with WAYLAND_DISPLAY removed, on X11 \
                     through XWayland; report-only (D22.8)\n",
                );
                let nogpu = [&nogpu[..], &["--xwayland"]].concat();
                if !plan(exe, &out, "weak-nogpu-x11", None, &nogpu)?.success() {
                    failed.push("weak-nogpu-x11".to_owned());
                }
            } else {
                println!("Skipping weak-nogpu-x11: {NO_XWAYLAND}.");
                notes.push_str(&format!("# weak-nogpu-x11: skipped, {NO_XWAYLAND}\n"));
            }
            let cores = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
            for n in [4, 2] {
                let name = format!("weak-{n}cores");
                if cores < n {
                    println!("Skipping {name}: this machine has {cores} cores.");
                    notes.push_str(&format!("# {name}: skipped, {cores} cores\n"));
                    continue;
                }
                let limit = Limit::available(n);
                notes.push_str(&format!("# {name}: {}\n", limit.describe()));
                let weak = [&["--plan", "weak"], &display[..]].concat();
                if !plan(exe, &out, &name, Some(&limit), &weak)?.success() {
                    failed.push(name);
                }
            }
        }
        let diagnostics =
            fs::read_to_string(out.join(first).join("system.txt")).unwrap_or_default();
        fs::write(out.join("system.txt"), format!("{notes}\n{diagnostics}"))?;
        println!("\nResults: {}", out.display());
        for name in [
            "smoke",
            "probe",
            "default",
            "focus",
            "weak-nogpu",
            "weak-nogpu-x11",
            "weak-4cores",
            "weak-2cores",
        ] {
            let Ok(report) = fs::read_to_string(out.join(name).join("report.md")) else {
                continue;
            };
            println!("== {name}: rule outcome");
            for line in report
                .lines()
                .skip_while(|line| !line.starts_with("## Rule outcome"))
                .skip(1)
                .filter(|line| !line.is_empty())
            {
                println!("{line}");
            }
            let resized = report.matches("resized by the window manager").count();
            if resized > 0 {
                println!("({resized} runs had a window of another size: see Flags in report.md)");
            }
        }
        let stamp = out
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        println!(
            "To send: tar czf bench.tgz -C {} {stamp}",
            self.out.display()
        );
        if failed.is_empty() {
            Ok(())
        } else {
            Err(format!("plans failed: {}", failed.join(", ")).into())
        }
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

/// Runs `exe run ARGS --out OUT/NAME --cpu-limit …` (under `limit`, if
/// any), its console both printed and written to `OUT/run-NAME.log`, and
/// moves the stamped directory the bench writes into `OUT/NAME` up into
/// it.
fn plan(
    exe: &Path,
    out: &Path,
    name: &str,
    limit: Option<&Limit>,
    args: &[&str],
) -> Result<ExitStatus, Box<dyn Error>> {
    let dir = out.join(name);
    let mut args: Vec<String> = std::iter::once("run")
        .chain(args.iter().copied())
        .map(str::to_owned)
        .collect();
    args.extend(["--out".to_owned(), dir.display().to_string()]);
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
        out.join(format!("run-{name}.log")),
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
            .map(|pipe| echo(pipe, Arc::clone(&log), false)),
        child
            .stderr
            .take()
            .map(|pipe| echo(pipe, Arc::clone(&log), true)),
    ];
    let status = child.wait()?;
    for echo in echoes.into_iter().flatten() {
        let _ = echo.join();
    }
    // The bench writes into a stamped directory under --out.
    if let Some(stamped) = fs::read_dir(&dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.is_dir())
    {
        for entry in fs::read_dir(&stamped)? {
            let entry = entry?;
            fs::rename(entry.path(), dir.join(entry.file_name()))?;
        }
        fs::remove_dir(&stamped)?;
    }
    Ok(status)
}

/// Copies `pipe` line by line to the console (standard error when
/// `error`) and to `log`, on its own thread.
fn echo(
    pipe: impl io::Read + Send + 'static,
    log: Arc<Mutex<fs::File>>,
    error: bool,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for line in BufReader::new(pipe).lines().map_while(Result::ok) {
            if error {
                eprintln!("{line}");
            } else {
                println!("{line}");
            }
            if let Ok(mut log) = log.lock() {
                let _ = writeln!(log, "{line}");
            }
        }
    })
}

/// The refresh rate the first windows' idle frames show: the median
/// interval between frames of the `hover` runs under `raw` (every run's
/// when there is none), in Hz, when at least half the intervals lie within
/// 5 % of it and it lies within 20–500 Hz. A display paces frames to its
/// vblanks (the user's hover frames: 96 % within 5 % of 16.67 ms); without
/// one, as under Xvfb, they spread (13–29 % here) and give no rate.
fn idle_hz(raw: &Path) -> Option<f64> {
    let files: Vec<PathBuf> = fs::read_dir(raw)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "jsonl"))
        .collect();
    let hover: Vec<&PathBuf> = files
        .iter()
        .filter(|path| {
            path.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("hover__"))
        })
        .collect();
    let chosen: Vec<&PathBuf> = if hover.is_empty() {
        files.iter().collect()
    } else {
        hover
    };
    let mut intervals = Vec::new();
    for path in chosen {
        let Ok(records) = record::read(path) else {
            continue;
        };
        let at: Vec<f64> = records
            .iter()
            .filter_map(|record| {
                if let Record::Frame { at_ms, .. } = record {
                    Some(*at_ms)
                } else {
                    None
                }
            })
            .collect();
        intervals.extend(at.windows(2).map(|w| w[1] - w[0]).filter(|&d| d > 0.0));
    }
    let period = percentile(&intervals, 50.0)?;
    let paced = intervals
        .iter()
        .filter(|&&d| (d - period).abs() <= 0.05 * period)
        .count();
    let hz = 1e3 / period;
    (2 * paced >= intervals.len() && (20.0..=500.0).contains(&hz))
        .then(|| (hz * 100.0).round() / 100.0)
}

/// The factor most of the first windows reported (their raw files' first
/// lines).
fn window_factor(raw: &Path) -> Option<f32> {
    let mut counts: Vec<(f32, usize)> = Vec::new();
    for entry in fs::read_dir(raw).ok()?.filter_map(Result::ok) {
        let Ok(records) = record::read(&entry.path()) else {
            continue;
        };
        let Some(factor) = records.iter().find_map(|record| {
            if let Record::Run(run) = record {
                run.window_factor
            } else {
                None
            }
        }) else {
            continue;
        };
        match counts.iter_mut().find(|(f, _)| (*f - factor).abs() < 1e-3) {
            Some((_, n)) => *n += 1,
            None => counts.push((factor, 1)),
        }
    }
    counts.into_iter().max_by_key(|&(_, n)| n).map(|(f, _)| f)
}

/// Whether this process runs in a Linux Wayland session with XWayland,
/// where the suite runs `weak-nogpu-x11`.
pub fn xwayland() -> bool {
    xwayland_on(
        std::env::consts::OS,
        std::env::var_os("WAYLAND_DISPLAY").as_deref(),
        std::env::var_os("DISPLAY").as_deref(),
    )
}

/// Whether `weak-nogpu-x11` runs on `os` with these `WAYLAND_DISPLAY`
/// and `DISPLAY`: on Linux with both set, empty counting as unset as
/// winit reads them.
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

/// The `system76-power` profile while the suite runs: `performance`,
/// the one before restored when dropped; nothing where the tool is
/// absent.
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
    use std::ffi::OsStr;
    use std::fs;

    use super::{Limit, idle_hz, mask, xwayland_on};
    use crate::record::{self, Record};

    /// Frames every `period` ms from 0 for `n` frames.
    fn frames(period: f64, n: u32) -> Vec<Record> {
        (0..n)
            .map(|k| Record::Frame {
                frame: u64::from(k),
                at_ms: f64::from(k) * period,
                generation: 1,
                images: 1,
                missing: 0,
                previews: 0,
                uploaded: 0,
                update_ms: 0.1,
                draw_ms: Some(0.1),
            })
            .collect()
    }

    #[test]
    fn the_first_windows_idle_frames_give_the_refresh_rate() {
        // Hover runs' frames 6.94 ms apart (144 Hz) win over a pan's longer
        // ones; frames 1 ms apart (no vsync, Xvfb) give no rate.
        let dir = std::env::temp_dir().join(format!("rustd3-suite-{}", std::process::id()));
        let raw = dir.join("raw");
        fs::create_dir_all(&raw).unwrap();
        record::write(
            &raw.join("hover__trace__f1__r1.jsonl"),
            &frames(1e3 / 144.0, 300),
        )
        .unwrap();
        record::write(&raw.join("pan-x__trace__f1__r1.jsonl"), &frames(33.4, 300)).unwrap();
        assert_eq!(idle_hz(&raw), Some(144.0));
        fs::remove_file(raw.join("hover__trace__f1__r1.jsonl")).unwrap();
        assert_eq!(idle_hz(&raw), Some(29.94), "every run's, without hover");
        record::write(&raw.join("hover__trace__f1__r1.jsonl"), &frames(1.0, 300)).unwrap();
        assert_eq!(idle_hz(&raw), None, "1,000 Hz");
        // Frames 8, 13 and 17 ms apart in turn: paced by nothing.
        let mut jitter = frames(1.0, 300);
        let mut at = 0.0;
        for (k, record) in jitter.iter_mut().enumerate() {
            if let Record::Frame { at_ms, .. } = record {
                *at_ms = at;
            }
            at += [8.0, 13.0, 17.0][k % 3];
        }
        record::write(&raw.join("hover__trace__f1__r1.jsonl"), &jitter).unwrap();
        assert_eq!(idle_hz(&raw), None, "unpaced");
        fs::remove_dir_all(&dir).unwrap();
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
        let pinned = Limit::Affinity(2).command(std::path::Path::new("bench"), &args);
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
        let pooled = Limit::Threads(2).command(std::path::Path::new("bench"), &args);
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
