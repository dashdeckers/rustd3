//! The run directory's CSVs and `report.md`, rebuilt from `meta.json` and
//! the raw files: the readings, the budget table, the metrics, attribution
//! of misses, memory, flags and the rule's outcome.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::metrics::{Half, Metrics};
use crate::plan::Plan;
use crate::record;
use crate::rule::{self, Budget, Check, Outcome, Profile};
use crate::scenario::{self, Scenario};
use crate::stats::{Interval, Verdict, percentile};
use crate::system::{Meta, Planned};

/// What the smoke plan is, said before its verdicts.
const SMOKE: &str = "one run per cell, a check of the windowed path only (windows open, the \
                     replayed input pans, zooms and hovers the plots, every run writes its \
                     records; Flags lists windows the window manager resized). Every budget \
                     reads \"inconclusive\" by design: a verdict needs a 95% interval, from the \
                     default plan's 5 runs per cell.";

/// The smoke plan's outcome line, in place of the rule's.
const SMOKE_OUTCOME: &str = "no verdict by design: smoke only checks the windowed path; \
                             verdicts need the default plan's 5 runs per cell";

/// One planned cell and its runs.
struct Cell<'a> {
    planned: &'a Planned,
    scenario: Scenario,
    runs: Vec<Metrics>,
    /// Raw files that are missing or unreadable.
    failed: u32,
    /// For a windowed `4k` cell, the smallest device size the compositor
    /// gave its windows when one fell short of 3840 × 2160: the cell is
    /// then "n/a: display too small" (D22.3).
    small: Option<[f32; 2]>,
}

/// Rebuilds the CSVs and `report.md` of the run in `dir`, updates
/// `meta.json`'s calibration, and returns the budget table and the rule's
/// outcome line.
pub fn build(dir: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut meta: Meta = serde_json::from_str(&fs::read_to_string(dir.join("meta.json"))?)?;
    let mut cells: Vec<Cell<'_>> = Vec::new();
    let planned = meta.cells.clone();
    for planned in &planned {
        let Some(scenario) = Scenario::parse(&planned.scenario) else {
            continue;
        };
        let mut cell = Cell {
            planned,
            scenario,
            runs: Vec::new(),
            failed: 0,
            small: None,
        };
        if planned.applies {
            for run in 1..=planned.runs {
                let name = format!(
                    "{}__{}__f{}__r{run}.jsonl",
                    planned.scenario, planned.workload, planned.factor
                );
                let parsed = record::read(&dir.join("raw").join(name))
                    .ok()
                    .and_then(|records| Metrics::of(&records));
                match parsed {
                    Some(metrics) => cell.runs.push(metrics),
                    None => cell.failed += 1,
                }
            }
        }
        if scenario == Scenario::FourK && !meta.headless {
            cell.small = cell
                .runs
                .iter()
                .map(|m| m.run.device)
                .filter(|[w, h]| {
                    *w < scenario::FOUR_K.width - 1.0 || *h < scenario::FOUR_K.height - 1.0
                })
                .reduce(|a, b| if a[0] * a[1] <= b[0] * b[1] { a } else { b });
        }
        cells.push(cell);
    }
    let calibration: Vec<f64> = cells
        .iter()
        .flat_map(|c| c.runs.iter().flat_map(Metrics::calibration))
        .collect();
    meta.calibration = percentile(&calibration, 50.0);
    fs::write(dir.join("meta.json"), serde_json::to_string_pretty(&meta)?)?;
    fs::write(dir.join("frames.csv"), frames_csv(&cells))?;
    fs::write(dir.join("inputs.csv"), inputs_csv(&cells))?;
    fs::write(dir.join("memory.csv"), memory_csv(&cells))?;
    let period = 1e3 / meta.refresh_hz;
    let profile = Profile::of(&meta.plan);
    let checks: Vec<(usize, Scenario, Check)> = cells
        .iter()
        .enumerate()
        .filter(|(_, c)| c.planned.applies && c.small.is_none())
        .flat_map(|(index, c)| {
            Budget::of(c.scenario).into_iter().map(move |budget| {
                let check = Check::evaluate(budget, &c.runs, period, profile);
                (index, c.scenario, check)
            })
        })
        .collect();
    fs::write(dir.join("summary.csv"), summary_csv(&cells, &checks))?;
    let outcome = Outcome::of(
        &checks
            .iter()
            .map(|(_, scenario, check)| (*scenario, check.clone()))
            .collect::<Vec<_>>(),
    );
    let also = fs::read_to_string(dir.join("also.md")).ok();
    let report = markdown(&meta, &cells, &checks, outcome, also.as_deref());
    fs::write(dir.join("report.md"), report)?;
    let headless = if meta.headless {
        " (report-only: headless)"
    } else {
        ""
    };
    let mut summary = String::new();
    let line = if smoke(&meta) {
        let _ = writeln!(summary, "Smoke plan: {SMOKE}\n");
        SMOKE_OUTCOME
    } else {
        outcome.line(profile)
    };
    budgets(&mut summary, &cells, &checks);
    Ok(format!("{summary}Rule outcome{headless}: {line}"))
}

/// Whether the run is the smoke plan at one run per cell.
fn smoke(meta: &Meta) -> bool {
    meta.plan == Plan::Smoke.name() && meta.cells.iter().all(|cell| cell.runs <= 1)
}

/// Every frame pass of every run.
fn frames_csv(cells: &[Cell<'_>]) -> String {
    let mut out = String::from(
        "scenario,workload,factor,run,frame,at_ms,interval_ms,generation,stale,uploaded,ui_ms\n",
    );
    for cell in cells {
        for metrics in &cell.runs {
            let mut last: Option<f64> = None;
            for f in &metrics.frames {
                let interval = last.map_or(0.0, |at| f.at_ms - at);
                last = Some(f.at_ms);
                let _ = writeln!(
                    out,
                    "{},{},{},{},{},{:.3},{:.3},{},{},{},{:.3}",
                    cell.planned.scenario,
                    cell.planned.workload,
                    cell.planned.factor,
                    metrics.run.run,
                    f.frame,
                    f.at_ms,
                    interval,
                    f.generation,
                    f.stale,
                    f.uploaded,
                    f.ui_ms
                );
            }
        }
    }
    out
}

/// Every input with its latency.
fn inputs_csv(cells: &[Cell<'_>]) -> String {
    let mut out = String::from(
        "scenario,workload,factor,run,event,scheduled_ms,latency_ms,sharp,frame,sharp_frame,generation\n",
    );
    for cell in cells {
        for metrics in &cell.runs {
            for l in &metrics.latencies {
                let _ = writeln!(
                    out,
                    "{},{},{},{},{},{:.3},{:.3},{},{},{},{}",
                    cell.planned.scenario,
                    cell.planned.workload,
                    cell.planned.factor,
                    metrics.run.run,
                    l.event,
                    l.scheduled_ms,
                    l.ms,
                    l.sharp,
                    l.frames[0],
                    l.frames[1],
                    l.generation
                );
            }
        }
    }
    out
}

/// RSS and GPU samples of every run.
fn memory_csv(cells: &[Cell<'_>]) -> String {
    let mut out = String::from("scenario,workload,factor,run,at_s,rss_mb,gpu_mb\n");
    for cell in cells {
        for metrics in &cell.runs {
            let gpu: BTreeMap<u64, f64> = metrics
                .gpu()
                .into_iter()
                .map(|(at, mb)| ((at * 1e3) as u64, mb))
                .collect();
            for (at, rss) in metrics.rss() {
                let gpu = gpu
                    .get(&((at * 1e3) as u64))
                    .map_or_else(|| "na".to_owned(), |mb| format!("{mb:.0}"));
                let _ = writeln!(
                    out,
                    "{},{},{},{},{at:.3},{rss:.1},{gpu}",
                    cell.planned.scenario,
                    cell.planned.workload,
                    cell.planned.factor,
                    metrics.run.run
                );
            }
        }
    }
    out
}

/// Per cell: every metric's interval across runs, budgets and verdicts.
fn summary_csv(cells: &[Cell<'_>], checks: &[(usize, Scenario, Check)]) -> String {
    let mut out = String::from(
        "scenario,workload,factor,metric,n,median,mean,ci_low,ci_high,budget,verdict\n",
    );
    for (index, cell) in cells.iter().enumerate() {
        let prefix = format!(
            "{},{},{}",
            cell.planned.scenario, cell.planned.workload, cell.planned.factor
        );
        if !cell.planned.applies {
            let _ = writeln!(out, "{prefix},n/a,0,na,na,na,na,na,n/a");
            continue;
        }
        if cell.small.is_some() {
            let _ = writeln!(out, "{prefix},n/a: display too small,0,na,na,na,na,na,n/a");
            continue;
        }
        for (name, interval) in metric_rows(cell) {
            let check = checks
                .iter()
                .find(|(i, _, c)| *i == index && metric_of(c) == name);
            let (budget, verdict) = check.map_or(("na".to_owned(), "na"), |(_, _, c)| {
                (format!("{}", c.limit), c.verdict.name())
            });
            match interval {
                Some(i) => {
                    let (low, high) = i.ci.map_or(("na".to_owned(), "na".to_owned()), |[l, h]| {
                        (format!("{l:.4}"), format!("{h:.4}"))
                    });
                    let _ = writeln!(
                        out,
                        "{prefix},{name},{},{:.4},{:.4},{low},{high},{budget},{verdict}",
                        i.n, i.median, i.mean
                    );
                }
                None => {
                    let _ = writeln!(out, "{prefix},{name},0,na,na,na,na,{budget},{verdict}");
                }
            }
        }
    }
    out
}

/// The metric a check reads, as named in the summary.
fn metric_of(check: &Check) -> &'static str {
    match (check.budget, check.half) {
        (Budget::Frame, _) => "frame_p99_ms",
        (Budget::PanLatency, Half::Pan) => "latency_p95_pan_ms",
        (Budget::ZoomLatency, Half::Zoom) => "latency_p95_zoom_ms",
        (Budget::PanLatency | Budget::ZoomLatency, Half::Whole | Half::Pan | Half::Zoom) => {
            "latency_p95_ms"
        }
        (Budget::Previews, Half::Pan) => "preview_share_pan",
        (Budget::Previews, Half::Whole | Half::Zoom) => "preview_share",
    }
}

/// Every per-run statistic of a cell, across its runs.
fn metric_rows(cell: &Cell<'_>) -> Vec<(&'static str, Option<Interval>)> {
    let of = |f: &dyn Fn(&Metrics) -> Option<f64>| {
        let values: Vec<f64> = cell.runs.iter().filter_map(f).collect();
        Interval::of(&values)
    };
    let mut rows: Vec<(&'static str, Option<Interval>)> = vec![
        ("frame_p50_ms", of(&|m| m.interval(50.0))),
        ("frame_p95_ms", of(&|m| m.interval(95.0))),
        ("frame_p99_ms", of(&|m| m.interval(99.0))),
        ("drops", of(&|m| Some(f64::from(m.drops())))),
        ("latency_p50_ms", of(&|m| m.latency(Half::Whole, 50.0))),
        ("latency_p95_ms", of(&|m| m.latency(Half::Whole, 95.0))),
        ("preview_share", of(&|m| m.preview_share(Half::Whole))),
        ("ui_event_p99_ms", of(&|m| m.event_busy(99.0))),
        ("ui_frame_p99_ms", of(&|m| m.frame_busy(99.0))),
        ("raster_critical_ms", of(&|m| m.raster_critical())),
        ("inline_share", of(&|m| m.inline_share())),
        ("cpu_s_per_s", of(&|m| m.cpu_per_s())),
        ("upload_mb_per_s", of(&|m| m.upload_mb_per_s())),
        ("resolve_ms", of(&|m| m.stage("resolve"))),
        ("list_ms", of(&|m| m.stage("list"))),
        ("calibration", of(&|m| percentile(&m.calibration(), 50.0))),
    ];
    if cell.scenario == Scenario::FourK {
        rows.push(("latency_p95_pan_ms", of(&|m| m.latency(Half::Pan, 95.0))));
        rows.push(("latency_p95_zoom_ms", of(&|m| m.latency(Half::Zoom, 95.0))));
        rows.push(("preview_share_pan", of(&|m| m.preview_share(Half::Pan))));
    }
    if cell.scenario == Scenario::Cold {
        rows.push(("cold_ms", of(&Metrics::cold_ms)));
    }
    if cell.scenario == Scenario::Soak {
        rows.push((
            "rss_max_mb",
            of(&|m| m.rss().iter().map(|s| s.1).reduce(f64::max)),
        ));
        rows.push((
            "gpu_max_mb",
            of(&|m| m.gpu().iter().map(|s| s.1).reduce(f64::max)),
        ));
    }
    rows
}

/// The report.
fn markdown(
    meta: &Meta,
    cells: &[Cell<'_>],
    checks: &[(usize, Scenario, Check)],
    outcome: Outcome,
    also: Option<&str>,
) -> String {
    let mut md = String::new();
    let mode = if meta.headless {
        "headless (report-only: headless frame intervals are not display intervals)"
    } else {
        "windowed"
    };
    let _ = writeln!(md, "# rustd3-bench {}\n", meta.stamp);
    let threads = meta.threads.map_or_else(
        || "the widget's default (available parallelism − 1, 1 to 8)".to_owned(),
        |n| n.to_string(),
    );
    let _ = writeln!(
        md,
        "Plan `{}`, {mode}; backend {}, present mode {}; {} Hz (T = {:.2} ms), native factor {}; \
         render {}, caching {}, readback {}, render threads {threads}; data scale {}.\n",
        meta.plan,
        meta.backend,
        meta.present_mode,
        meta.refresh_hz,
        1e3 / meta.refresh_hz,
        meta.native_factor,
        meta.render,
        meta.caching,
        meta.readback,
        meta.data_scale
    );
    let _ = writeln!(
        md,
        "{} ({} cores, {} MiB); kernel {}; GPU {}; session {}, desktop {}; {}; iced {}; \
         workspace {}.\n\nCommand: `{}`\n",
        meta.cpu,
        meta.cores,
        meta.ram_mb,
        meta.kernel,
        meta.gpu,
        meta.session,
        meta.desktop,
        meta.rustc,
        &meta.iced_rev[..7.min(meta.iced_rev.len())],
        meta.git_rev,
        meta.command
    );
    let _ = writeln!(md, "## The registered rule\n\n{}", rule::READINGS);
    let profile = Profile::of(&meta.plan);
    if profile == Profile::Weak {
        let gpu = if meta.backend == "tiny-skia" {
            "no GPU (tiny-skia)".to_owned()
        } else {
            format!("{}, {}", meta.backend, meta.gpu)
        };
        let _ = writeln!(
            md,
            "## Weak profile: {gpu}, {} cores\n\n{}\nThe budget table below holds this \
             profile's cells to these budgets; its outcome closes the report.\n",
            meta.cores,
            rule::WEAK
        );
    }
    budgets(&mut md, cells, checks);
    metrics(&mut md, cells);
    attribution(&mut md, cells, checks);
    memory(&mut md, cells, meta);
    flags(&mut md, cells, meta);
    if let Some(also) = also {
        let _ = writeln!(md, "{also}");
    }
    let _ = writeln!(md, "## Rule outcome\n");
    if meta.headless {
        let _ = writeln!(
            md,
            "**report-only: headless.** Headless numbers never decide; the verdict comes from \
             the windowed default plan on the user's hardware.\n"
        );
    }
    let missed = checks
        .iter()
        .filter(|c| c.2.verdict == Verdict::Missed)
        .count();
    let inconclusive = checks
        .iter()
        .filter(|c| c.2.verdict == Verdict::Inconclusive)
        .count();
    let line = if smoke(meta) {
        let _ = writeln!(md, "**Smoke plan:** {SMOKE}\n");
        SMOKE_OUTCOME
    } else {
        outcome.line(profile)
    };
    let _ = writeln!(
        md,
        "{} checks: {} met, {missed} missed, {inconclusive} inconclusive.\n\n**{line}**",
        checks.len(),
        checks.len() - missed - inconclusive,
    );
    md
}

/// The budget table: a row per cell, "n/a" cells included.
fn budgets(md: &mut String, cells: &[Cell<'_>], checks: &[(usize, Scenario, Check)]) {
    let _ = writeln!(
        md,
        "## Budgets\n\nPer-run statistic: mean [95% CI] over n runs; verdict by reading 3. \
         \"—\": the budget does not apply to the scenario; \"n/a\": the workload has no such \
         cell; \"n/a: display too small\": a `4k` window the compositor gave below 3840 × 2160 \
         device px (D22.3).\n"
    );
    let _ = writeln!(
        md,
        "| Cell | n | frame p99 ms | pan latency p95 ms | zoom latency p95 ms | preview share |\n\
         | --- | --- | --- | --- | --- | --- |"
    );
    for (index, cell) in cells.iter().enumerate() {
        let name = cell_name(cell);
        if !cell.planned.applies {
            let _ = writeln!(md, "| {name} | n/a | n/a | n/a | n/a | n/a |");
            continue;
        }
        if cell.small.is_some() {
            let na = "n/a: display too small";
            let _ = writeln!(
                md,
                "| {name} | {} | {na} | {na} | {na} | {na} |",
                n_runs(cell)
            );
            continue;
        }
        let column = |budget: Budget| {
            let found: Vec<&Check> = checks
                .iter()
                .filter(|(i, _, c)| *i == index && c.budget == budget)
                .map(|(_, _, c)| c)
                .collect();
            if found.is_empty() {
                return "—".to_owned();
            }
            found
                .iter()
                .map(|c| {
                    let share = c.budget == Budget::Previews;
                    let half = match c.half {
                        Half::Whole => "",
                        Half::Pan => "pan: ",
                        Half::Zoom => "zoom: ",
                    };
                    format!(
                        "{half}{} (≤ {}) {}",
                        interval(c.interval, share),
                        limit(c.limit, share),
                        c.verdict.name()
                    )
                })
                .collect::<Vec<_>>()
                .join("; ")
        };
        let _ = writeln!(
            md,
            "| {name} | {} | {} | {} | {} | {} |",
            n_runs(cell),
            column(Budget::Frame),
            column(Budget::PanLatency),
            column(Budget::ZoomLatency),
            column(Budget::Previews)
        );
    }
    let _ = writeln!(md);
}

/// The metrics table: medians across runs.
fn metrics(md: &mut String, cells: &[Cell<'_>]) {
    let _ = writeln!(
        md,
        "## Metrics\n\nMedians across runs. Frame intervals p50/p95/p99 and drops (reading 1); \
         input-to-sharp latency p50/p95 (scan-out adds up to one period, not included); preview \
         share (reading 4); UI-thread busy p99 per injected event and per frame (update + draw); \
         raster critical path (median over latency windows); share of renders inline; process CPU \
         seconds per second; upload MB/s; resolve and list stage medians.\n"
    );
    let _ = writeln!(
        md,
        "| Cell | frame p50/p95/p99 ms | drops | latency p50/p95 ms | previews | UI p99 event/frame ms | raster ms | inline | CPU s/s | upload MB/s | resolve/list ms |\n\
         | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |"
    );
    for cell in cells {
        let name = cell_name(cell);
        if !cell.planned.applies {
            let _ = writeln!(
                md,
                "| {name} | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |"
            );
            continue;
        }
        if cell.small.is_some() {
            let rest = " — |".repeat(9);
            let _ = writeln!(md, "| {name} | n/a: display too small |{rest}");
            continue;
        }
        if cell.runs.is_empty() {
            let _ = writeln!(
                md,
                "| {name} | failed | failed | failed | failed | failed | failed | failed | failed | failed | failed |"
            );
            continue;
        }
        let med = |f: &dyn Fn(&Metrics) -> Option<f64>| {
            let values: Vec<f64> = cell.runs.iter().filter_map(f).collect();
            percentile(&values, 50.0)
        };
        let num = |v: Option<f64>, digits: usize| {
            v.map_or_else(|| "n/a".to_owned(), |v| format!("{v:.digits$}"))
        };
        let pct =
            |v: Option<f64>| v.map_or_else(|| "n/a".to_owned(), |v| format!("{:.0}%", 100.0 * v));
        let _ = writeln!(
            md,
            "| {name} | {}/{}/{} | {} | {}/{} | {} | {}/{} | {} | {} | {} | {} | {}/{} |",
            num(med(&|m| m.interval(50.0)), 1),
            num(med(&|m| m.interval(95.0)), 1),
            num(med(&|m| m.interval(99.0)), 1),
            num(med(&|m| Some(f64::from(m.drops()))), 0),
            num(med(&|m| m.latency(Half::Whole, 50.0)), 0),
            num(med(&|m| m.latency(Half::Whole, 95.0)), 0),
            pct(med(&|m| m.preview_share(Half::Whole))),
            num(med(&|m| m.event_busy(99.0)), 2),
            num(med(&|m| m.frame_busy(99.0)), 1),
            num(med(&Metrics::raster_critical), 1),
            pct(med(&Metrics::inline_share)),
            num(med(&Metrics::cpu_per_s), 2),
            num(med(&Metrics::upload_mb_per_s), 1),
            num(med(&|m| m.stage("resolve")), 1),
            num(med(&|m| m.stage("list")), 1),
        );
    }
    let _ = writeln!(md);
    let cold: Vec<f64> = cells
        .iter()
        .filter(|c| c.scenario == Scenario::Cold)
        .flat_map(|c| c.runs.iter().filter_map(Metrics::cold_ms))
        .collect();
    if let Some(cold) = percentile(&cold, 50.0) {
        let _ = writeln!(
            md,
            "Cold start (process start to the first frame with every slot current), median over \
             every `cold` run: {cold:.0} ms.\n"
        );
    }
}

/// Attribution of every missed budget, and of budgets a single run
/// exceeded (no verdict).
fn attribution(md: &mut String, cells: &[Cell<'_>], checks: &[(usize, Scenario, Check)]) {
    let _ = writeln!(md, "## Attribution\n");
    let attributed: Vec<&(usize, Scenario, Check)> =
        checks.iter().filter(|c| c.2.attributed()).collect();
    if attributed.is_empty() {
        let _ = writeln!(md, "No budget missed, and no single run over a budget.\n");
        return;
    }
    let _ = writeln!(
        md,
        "Missed budgets, and budgets a single run exceeded (\"over, n = 1\": no verdict): the \
         mean per offending latency window or frame over every run (reading 5); latency \
         windows of one gesture overlap, so their means are not additive.\n\n\
         | Cell | Budget | Verdict | windows | UI ms | raster ms (inline renders) | upload ms | bucket |\n\
         | --- | --- | --- | --- | --- | --- | --- | --- |"
    );
    for (index, _, check) in attributed {
        let s = check.shares;
        let [ui, raster, upload] = s.per_window();
        let verdict = if check.verdict == Verdict::Missed {
            "missed"
        } else {
            "over, n = 1"
        };
        let _ = writeln!(
            md,
            "| {} | {} | {verdict} | {} | {ui:.1} | {raster:.1} ({}) | {upload:.1} | {} |",
            cell_name(&cells[*index]),
            check.budget.name(),
            s.windows,
            s.inline,
            s.bucket()
        );
    }
    let _ = writeln!(md);
}

/// RSS and GPU memory of the soak runs.
fn memory(md: &mut String, cells: &[Cell<'_>], meta: &Meta) {
    let _ = writeln!(md, "## Memory\n");
    let soaks: Vec<(&Cell<'_>, &Metrics)> = cells
        .iter()
        .filter(|c| c.scenario == Scenario::Soak)
        .flat_map(|c| c.runs.iter().map(move |m| (c, m)))
        .collect();
    if soaks.is_empty() {
        let _ = writeln!(md, "No `soak` run.\n");
        return;
    }
    let _ = writeln!(
        md,
        "One run per configuration. GPU memory is whole-GPU `nvidia-smi` minus the pre-run \
         baseline ({}), approximate.\n\n\
         | Cell | duration s | RSS first/max/last MB | GPU max − baseline MiB |\n\
         | --- | --- | --- | --- |",
        meta.gpu_baseline_mb
            .map_or_else(|| "no nvidia-smi".to_owned(), |mb| format!("{mb:.0} MiB"))
    );
    for (cell, m) in soaks {
        let rss = m.rss();
        let first = rss.first().map_or(0.0, |s| s.1);
        let max = rss.iter().map(|s| s.1).fold(0.0, f64::max);
        let last = rss.last().map_or(0.0, |s| s.1);
        let gpu = match (
            m.gpu().iter().map(|s| s.1).reduce(f64::max),
            meta.gpu_baseline_mb,
        ) {
            (Some(max), Some(base)) => format!("{:.0}", max - base),
            (Some(_), None) | (None, _) => "n/a".to_owned(),
        };
        let _ = writeln!(
            md,
            "| {} | {:.0} | {first:.0}/{max:.0}/{last:.0} | {gpu} |",
            cell_name(cell),
            (m.end_ms - m.start_ms) / 1e3
        );
    }
    let _ = writeln!(md);
}

/// Runs that failed or did not settle, windows that differ from what was
/// asked (by more than 1 logical px), and the cost model's calibration.
fn flags(md: &mut String, cells: &[Cell<'_>], meta: &Meta) {
    let _ = writeln!(md, "## Flags\n");
    let mut any = false;
    for cell in cells {
        if let Some([w, h]) = cell.small {
            any = true;
            let _ = writeln!(
                md,
                "- {}: the compositor gave a {w:.0}×{h:.0} device px window, below {}×{}: n/a \
                 (display too small, D22.3)",
                cell_name(cell),
                scenario::FOUR_K.width,
                scenario::FOUR_K.height
            );
        }
        if cell.failed > 0 {
            any = true;
            let _ = writeln!(
                md,
                "- {}: {} run(s) failed (no raw file)",
                cell_name(cell),
                cell.failed
            );
        }
        for m in &cell.runs {
            if !m.settled {
                any = true;
                let _ = writeln!(
                    md,
                    "- {} run {}: not every slot was sharp 32 s after the last event",
                    cell_name(cell),
                    m.run.run
                );
            }
            let [w0, h0] = m.run.window;
            let resized = [m.run.widget, m.run.reported]
                .into_iter()
                .flatten()
                .find(|[w, h]| (w - w0).abs() > 1.0 || (h - h0).abs() > 1.0);
            if let Some([w, h]) = resized {
                any = true;
                let _ = writeln!(
                    md,
                    "- {} run {}: window resized by the window manager: {w:.0}×{h:.0} instead of \
                     {w0:.0}×{h0:.0}; positions were scaled",
                    cell_name(cell),
                    m.run.run
                );
            }
            if let Some(factor) = m.run.window_factor
                && (factor - meta.native_factor).abs() > 1e-3
            {
                any = true;
                let _ = writeln!(
                    md,
                    "- {} run {}: the window's factor is {factor}, not the native {}",
                    cell_name(cell),
                    m.run.run,
                    meta.native_factor
                );
            }
        }
    }
    if !any {
        let _ = writeln!(
            md,
            "- none: every run wrote its records, settled sharp and got the window it asked for"
        );
    }
    if meta.caching == "verify" {
        let maxima: Vec<u8> = cells
            .iter()
            .flat_map(|c| c.runs.iter().flat_map(|m| m.mismatches.iter().copied()))
            .collect();
        let runs: usize = cells.iter().map(|c| c.runs.len()).sum();
        match maxima.iter().max() {
            Some(max) => {
                let _ = writeln!(
                    md,
                    "- caching verify: {} kept images differed from a fresh render by more than \
                     1/255 over {runs} runs (max {max}/255) and were replaced",
                    maxima.len()
                );
            }
            None => {
                let _ = writeln!(
                    md,
                    "- caching verify: every kept image matched a fresh render within 1/255 over \
                     {runs} runs"
                );
            }
        }
    }
    let _ = writeln!(
        md,
        "- cost model: `PIXELS_PER_MS` = {}, `ITEMS_PER_MS` = {}; calibration (median of measured \
         / modelled raster time over every render) = {}",
        meta.pixels_per_ms,
        meta.items_per_ms,
        meta.calibration
            .map_or_else(|| "n/a (no render)".to_owned(), |c| format!("{c:.3}"))
    );
    let _ = writeln!(
        md,
        "- label-churn labels a UTC time axis around midnight by its own ticks (Phase 34's real \
         time formats, in place of Phase 22's stand-in labels)\n"
    );
}

/// `scenario × workload × factor`.
fn cell_name(cell: &Cell<'_>) -> String {
    format!(
        "{} × {} × {}",
        cell.planned.scenario, cell.planned.workload, cell.planned.factor
    )
}

/// Runs read over runs planned.
fn n_runs(cell: &Cell<'_>) -> String {
    if cell.failed == 0 {
        cell.runs.len().to_string()
    } else {
        format!("{}/{}", cell.runs.len(), cell.planned.runs)
    }
}

/// An interval as `mean [low, high]`, or `mean (n = 1)`.
fn interval(interval: Option<Interval>, share: bool) -> String {
    let f = |v: f64| {
        if share {
            format!("{:.1}%", 100.0 * v)
        } else {
            format!("{v:.1}")
        }
    };
    match interval {
        Some(Interval {
            mean,
            ci: Some([low, high]),
            ..
        }) => format!("{} [{}, {}]", f(mean), f(low), f(high)),
        Some(Interval {
            mean, n, ci: None, ..
        }) => format!("{} (n = {n})", f(mean)),
        None => "n/a".to_owned(),
    }
}

/// A budget's limit.
fn limit(limit: f64, share: bool) -> String {
    if share {
        format!("{:.0}%", 100.0 * limit)
    } else {
        format!("{limit:.1}")
    }
}
