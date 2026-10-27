//! The machine and the process: `/proc` counters, `nvidia-smi`, and
//! [`Meta`], what `meta.json` records about a run.

use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::record::Record;

/// The iced revision the workspace pins.
pub const ICED_REV: &str = "3cbdacc3b0841c5145c766c3a758657d36b41ffe";

/// What a run's `meta.json` holds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Meta {
    /// The run's directory name (UTC).
    pub stamp: String,
    /// `model name` of `/proc/cpuinfo`.
    pub cpu: String,
    /// Logical cores.
    pub cores: usize,
    /// `MemTotal`, MiB.
    pub ram_mb: u64,
    /// `uname -r`.
    pub kernel: String,
    /// `nvidia-smi`'s GPU name and driver, else "unknown".
    pub gpu: String,
    /// `XDG_SESSION_TYPE`.
    pub session: String,
    /// `XDG_CURRENT_DESKTOP`.
    pub desktop: String,
    /// The present mode asked for, or iced's default.
    pub present_mode: String,
    /// `wgpu` or `tiny-skia`.
    pub backend: String,
    /// Headless or windowed.
    pub headless: bool,
    /// Hz.
    pub refresh_hz: f64,
    /// The display's factor.
    pub native_factor: f32,
    /// The factors the plan ran.
    pub factors: Vec<f32>,
    /// `auto`, `inline` or `worker`.
    pub render: String,
    /// `on` or `off`.
    pub caching: String,
    /// `every` or `last`.
    pub readback: String,
    /// `rustc --version`.
    pub rustc: String,
    /// The pinned iced revision.
    pub iced_rev: String,
    /// `git rev-parse HEAD` of the workspace, else "unknown".
    pub git_rev: String,
    /// The plan's name.
    pub plan: String,
    /// The data size (1 = full).
    pub data_scale: f64,
    /// The command line.
    pub command: String,
    /// The widget's cost model: device px per ms.
    pub pixels_per_ms: f64,
    /// The widget's cost model: work items per ms.
    pub items_per_ms: f64,
    /// Median of measured / modelled raster time over every `Raster`
    /// sample, once the runs are in.
    pub calibration: Option<f64>,
    /// Whole-GPU memory in use before the first run, MiB.
    pub gpu_baseline_mb: Option<f64>,
    /// `getconf CLK_TCK`.
    pub clk_tck: u64,
    /// Every cell of the plan, "n/a" ones included.
    pub cells: Vec<Planned>,
}

/// A cell as `meta.json` lists it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Planned {
    /// The scenario's name.
    pub scenario: String,
    /// The workload's name.
    pub workload: String,
    /// The factor.
    pub factor: f32,
    /// Runs asked for.
    pub runs: u32,
    /// False for "n/a": the workload does not support the scenario.
    pub applies: bool,
}

/// Samples RSS every second and GPU memory every five on its own thread,
/// as `sys` records, until dropped.
#[derive(Debug)]
pub struct Sampler {
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Sampler {
    /// Starts sampling into `records`, times since `epoch`.
    pub fn start(records: mpsc::Sender<Record>, epoch: Instant) -> Sampler {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let thread = thread::Builder::new()
            .name("bench-sampler".to_owned())
            .spawn(move || {
                let mut tick = 0_u64;
                while !flag.load(Ordering::Relaxed) {
                    let gpu = if tick.is_multiple_of(5) {
                        gpu_mb()
                    } else {
                        None
                    };
                    let _ = records.send(sys(epoch, "sample", gpu));
                    tick += 1;
                    for _ in 0..10 {
                        if flag.load(Ordering::Relaxed) {
                            return;
                        }
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            })
            .ok();
        Sampler { stop, thread }
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// A `sys` record now: CPU ticks, RSS and `gpu`.
pub fn sys(epoch: Instant, mark: &str, gpu_mb: Option<f64>) -> Record {
    Record::Sys {
        at_ms: since(epoch, Instant::now()),
        mark: mark.to_owned(),
        cpu_ticks: cpu_ticks(),
        rss_kb: rss_kb(),
        gpu_mb,
    }
}

/// The current UTC time as `YYYYmmddTHHMMSSZ`, a run directory's name.
pub fn stamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let (days, rest) = (secs / 86_400, secs % 86_400);
    let [year, month, day] = civil(days);
    let (h, m, s) = (rest / 3600, rest % 3600 / 60, rest % 60);
    format!("{year:04}{month:02}{day:02}T{h:02}{m:02}{s:02}Z")
}

/// The proleptic Gregorian date `days` after 1970-01-01 (Hinnant's
/// `civil_from_days`).
fn civil(days: u64) -> [u64; 3] {
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + u64::from(month <= 2);
    [year, month, day]
}

/// ms from `epoch` to `at`.
pub fn since(epoch: Instant, at: Instant) -> f64 {
    at.saturating_duration_since(epoch).as_secs_f64() * 1e3
}

/// utime + stime of this process, in clock ticks: fields 14 and 15 of
/// `/proc/self/stat`, counted after the command name's closing paren.
pub fn cpu_ticks() -> Option<u64> {
    let stat = fs::read_to_string("/proc/self/stat").ok()?;
    let rest = &stat[stat.rfind(')')? + 1..];
    let fields: Vec<&str> = rest.split_whitespace().collect();
    // `rest` starts at field 3 (state): utime is field 14, stime 15.
    let utime: u64 = fields.get(11)?.parse().ok()?;
    let stime: u64 = fields.get(12)?.parse().ok()?;
    Some(utime + stime)
}

/// `VmRSS` of this process, kB.
pub fn rss_kb() -> Option<u64> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    status
        .lines()
        .find_map(|line| line.strip_prefix("VmRSS:"))
        .and_then(|rest| rest.split_whitespace().next()?.parse().ok())
}

/// Whole-GPU memory in use, MiB, from `nvidia-smi`; `None` without one.
pub fn gpu_mb() -> Option<f64> {
    let out = output(
        "nvidia-smi",
        &["--query-gpu=memory.used", "--format=csv,noheader,nounits"],
    )?;
    out.lines().next()?.trim().parse().ok()
}

/// `getconf CLK_TCK`, 100 if unavailable.
pub fn clk_tck() -> u64 {
    output("getconf", &["CLK_TCK"])
        .and_then(|out| out.trim().parse().ok())
        .unwrap_or(100)
}

/// The machine's description for `meta.json`: CPU, cores, RAM, kernel,
/// GPU, session and desktop, compiler and git revision.
pub fn machine() -> [String; 7] {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let cpu = cpuinfo
        .lines()
        .find_map(|line| line.strip_prefix("model name"))
        .and_then(|rest| rest.split_once(':'))
        .map_or_else(|| "unknown".to_owned(), |(_, name)| name.trim().to_owned());
    let kernel =
        output("uname", &["-r"]).map_or_else(|| "unknown".to_owned(), |s| s.trim().to_owned());
    let gpu = output(
        "nvidia-smi",
        &["--query-gpu=name,driver_version", "--format=csv,noheader"],
    )
    .and_then(|s| s.lines().next().map(|line| line.trim().to_owned()))
    .unwrap_or_else(|| "unknown (no nvidia-smi)".to_owned());
    let env = |name: &str| std::env::var(name).unwrap_or_else(|_| "unset".to_owned());
    let rustc = output("rustc", &["--version"])
        .map_or_else(|| "unknown".to_owned(), |s| s.trim().to_owned());
    let git = output("git", &["rev-parse", "HEAD"])
        .map_or_else(|| "unknown".to_owned(), |s| s.trim().to_owned());
    [
        cpu,
        kernel,
        gpu,
        env("XDG_SESSION_TYPE"),
        env("XDG_CURRENT_DESKTOP"),
        rustc,
        git,
    ]
}

/// `MemTotal`, MiB.
pub fn ram_mb() -> u64 {
    fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|info| {
            info.lines()
                .find_map(|line| line.strip_prefix("MemTotal:"))
                .and_then(|rest| rest.split_whitespace().next()?.parse::<u64>().ok())
        })
        .map_or(0, |kb| kb / 1024)
}

/// A command's standard output when it succeeds.
fn output(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::{civil, cpu_ticks, rss_kb};

    #[test]
    fn days_since_1970_become_dates() {
        assert_eq!(civil(0), [1970, 1, 1]);
        assert_eq!(civil(20_753), [2026, 10, 27]);
        assert_eq!(civil(11_016), [2000, 2, 29]);
    }

    #[test]
    fn this_process_reports_its_cpu_time_and_rss() {
        let before = cpu_ticks().unwrap();
        let mut x = 0_u64;
        for i in 0..20_000_000_u64 {
            x = x.wrapping_mul(31).wrapping_add(i);
        }
        assert!(x != 1);
        assert!(cpu_ticks().unwrap() >= before);
        assert!(rss_kb().unwrap() > 0);
    }
}
