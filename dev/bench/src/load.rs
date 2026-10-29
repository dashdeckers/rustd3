//! Reading results back for `compare` and `report` (D22.9 (5)): a
//! level's directory or archive by its `results.json`, and the layout
//! before D22.9 (a suite's directory of plans, or one plan's) by each
//! plan's `summary.csv`, `meta.json` and `report.md`, read only.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use crate::archive;
use crate::display::{Display, Source};
use crate::results::{self, Cell, Check, Counts, Results, Status};
use crate::system::Meta;

/// The results at `path`: a directory or a `.tar.gz` archive of a level,
/// or of the layout before D22.9.
pub fn results(path: &Path) -> Result<Results, Box<dyn Error>> {
    let files = files(path)?;
    if let Some(json) = files.get("results.json") {
        return Ok(serde_json::from_slice(json)?);
    }
    old(&files, path).ok_or_else(|| {
        format!(
            "{}: neither a level's results (results.json) nor a run's or suite's directory \
             (summary.csv)",
            path.display()
        )
        .into()
    })
}

/// The files of `path` a reader needs, by their path relative to the
/// results directory: every file of an archive (its one top directory
/// taken off), or a directory's `results.json`, and its and its
/// subdirectories' `summary.csv`, `meta.json` and `report.md`.
pub fn files(path: &Path) -> Result<BTreeMap<String, Vec<u8>>, Box<dyn Error>> {
    let mut files = BTreeMap::new();
    if path.is_dir() {
        const WANTED: [&str; 4] = ["results.json", "summary.csv", "meta.json", "report.md"];
        let mut read = |relative: String, file: &Path| -> std::io::Result<()> {
            if file.is_file() {
                files.insert(relative, fs::read(file)?);
            }
            Ok(())
        };
        for name in WANTED {
            read(name.to_owned(), &path.join(name))?;
        }
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let sub = entry.file_name().to_string_lossy().into_owned();
                for name in WANTED {
                    read(format!("{sub}/{name}"), &entry.path().join(name))?;
                }
            }
        }
        return Ok(files);
    }
    let entries = archive::read(path)?;
    let top = |entry: &archive::Entry| entry.path.split_once('/').map(|(top, _)| top.to_owned());
    let common = entries.first().and_then(top);
    let strip = common.filter(|common| entries.iter().all(|e| top(e).as_ref() == Some(common)));
    for entry in entries {
        let relative = match &strip {
            Some(top) => entry.path[top.len() + 1..].to_owned(),
            None => entry.path,
        };
        files.insert(relative, entry.data);
    }
    Ok(files)
}

/// The layout before D22.9: a plan's directory (`summary.csv` at the
/// top), or a suite's (one per plan), as results whose profiles keep
/// their cells' medians and budgets; the suite's plan names become the
/// profiles' (`default` → `native`, `weak-nogpu` → `nogpu`, …).
fn old(files: &BTreeMap<String, Vec<u8>>, path: &Path) -> Option<Results> {
    let text = |name: &str| files.get(name).map(|bytes| String::from_utf8_lossy(bytes));
    let mut plans: Vec<String> = files
        .keys()
        .filter_map(|key| key.strip_suffix("/summary.csv"))
        .filter(|dir| !dir.contains('/'))
        .map(str::to_owned)
        .collect();
    if files.contains_key("summary.csv") {
        plans.insert(0, String::new());
    }
    // A suite's plans in a level's order of profiles.
    let order = |dir: &String| {
        [
            "",
            "default",
            "weak-nogpu",
            "weak-nogpu-x11",
            "weak-4cores",
            "weak-2cores",
        ]
        .iter()
        .position(|known| known == dir)
        .unwrap_or(usize::MAX)
    };
    plans.sort_by_key(order);
    if plans.is_empty() {
        return None;
    }
    let at = |dir: &str, name: &str| {
        if dir.is_empty() {
            name.to_owned()
        } else {
            format!("{dir}/{name}")
        }
    };
    let metas: Vec<Meta> = plans
        .iter()
        .filter_map(|dir| serde_json::from_str(&text(&at(dir, "meta.json"))?).ok())
        .collect();
    let report = plans
        .iter()
        .find_map(|dir| text(&at(dir, "report.md")))
        .unwrap_or_default();
    let meta = metas.first();
    let stamp = path
        .file_name()
        .map(|n| n.to_string_lossy().trim_end_matches(".tar.gz").to_owned())
        .unwrap_or_default();
    let profiles = plans
        .iter()
        .map(|dir| {
            let meta: Option<Meta> =
                serde_json::from_str(&text(&at(dir, "meta.json")).unwrap_or_default()).ok();
            let plan = meta
                .as_ref()
                .map_or_else(|| dir.clone(), |m| m.plan.clone());
            let name = match dir.as_str() {
                "" | "default" | "focus" | "full" | "quick" => "native",
                "weak-nogpu" => "nogpu",
                "weak-nogpu-x11" => "nogpu-x11",
                "weak-4cores" => "4cores",
                "weak-2cores" => "2cores",
                other => other,
            };
            let cells = summary(&text(&at(dir, "summary.csv")).unwrap_or_default());
            let verdicts = |verdict: &str| {
                cells
                    .iter()
                    .flat_map(|c| &c.budgets)
                    .filter(|b| b.verdict == verdict)
                    .count()
            };
            results::Profile {
                plan: plan.clone(),
                backend: meta
                    .as_ref()
                    .map_or_else(String::new, |m| m.backend.clone()),
                ran: meta.iter().map(|m| m.backend.clone()).collect(),
                budgets: if plan == "weak" { "weak" } else { "desktop" }.to_owned(),
                limit: meta.as_ref().and_then(|m| m.cpu_limit.clone()),
                xwayland: meta.as_ref().is_some_and(|m| m.xwayland),
                headless: meta.as_ref().is_some_and(|m| m.headless),
                status: Status::Done,
                outcome: outcome(&text(&at(dir, "report.md")).unwrap_or_default()),
                checks: Counts {
                    met: verdicts("met"),
                    missed: verdicts("missed"),
                    inconclusive: verdicts("inconclusive"),
                },
                cells,
                ..results::Profile::planned(
                    name,
                    format!("plan {plan} (`{dir}/`, the layout before D22.9)"),
                    None,
                )
            }
        })
        .collect();
    let unknown = || "unknown".to_owned();
    let machine = results::Machine {
        os: unknown(),
        release: unknown(),
        kernel: meta.map_or_else(unknown, |m| m.kernel.clone()),
        arch: unknown(),
        // Without meta.json, the report's line on the machine.
        cpu: meta.map_or_else(
            || {
                report
                    .lines()
                    .find(|line| line.contains(" cores, ") && line.contains("; kernel "))
                    .map_or_else(unknown, |line| {
                        line.split(" (").next().unwrap_or_default().to_owned()
                    })
            },
            |m| m.cpu.clone(),
        ),
        cores: meta.map_or_else(
            || number(&report, "(", " cores").unwrap_or(0.0) as usize,
            |m| m.cores,
        ),
        ram_mb: meta.map_or(0, |m| m.ram_mb),
        gpu: meta.map_or_else(
            || between(&report, "; GPU ", "; session").unwrap_or_else(unknown),
            |m| m.gpu.clone(),
        ),
        session: meta.map_or_else(
            || between(&report, "; session ", ", desktop").unwrap_or_else(unknown),
            |m| m.session.clone(),
        ),
        desktop: meta.map_or_else(
            || between(&report, ", desktop ", "; ").unwrap_or_else(unknown),
            |m| m.desktop.clone(),
        ),
        wayland: false,
        x11: false,
    };
    let display = Display {
        headless: meta
            .is_some_and(|m| m.headless)
            .then(|| "headless (the layout before D22.9)".to_owned()),
        refresh_hz: meta.map_or_else(
            || number(&report, "; ", " Hz (T = ").unwrap_or(60.0),
            |m| m.refresh_hz,
        ),
        refresh: Source::Given,
        native_factor: meta.map_or_else(
            || number(&report, "native factor ", ";").unwrap_or(1.0) as f32,
            |m| m.native_factor,
        ),
        factor: Source::Given,
        backend: meta.map(|m| m.backend.clone()),
    };
    Some(Results {
        format: 0,
        rustd3: results::Version {
            version: unknown(),
            commit: meta.map_or_else(
                || between(&report, "; workspace ", ".").unwrap_or_else(unknown),
                |m| m.git_rev.clone(),
            ),
            iced: meta.map_or_else(unknown, |m| m.iced_rev.clone()),
        },
        level: "the layout before D22.9".to_owned(),
        stamp,
        complete: true,
        machine,
        display,
        figure_only: false,
        experiment: None,
        cpu_reference: Vec::new(),
        profiles,
    })
}

/// A plan's cells from its `summary.csv`
/// (`scenario,workload,factor,metric,n,median,mean,ci_low,ci_high,budget,verdict`).
fn summary(csv: &str) -> Vec<Cell> {
    let mut cells: Vec<Cell> = Vec::new();
    for line in csv.lines().skip(1) {
        let fields: Vec<&str> = line.split(',').collect();
        let [
            scenario,
            workload,
            factor,
            metric,
            n,
            median,
            mean,
            low,
            high,
            budget,
            verdict,
        ] = fields[..]
        else {
            continue;
        };
        let Ok(factor) = factor.parse::<f32>() else {
            continue;
        };
        let index = match cells
            .iter()
            .position(|c| c.scenario == scenario && c.workload == workload && c.factor == factor)
        {
            Some(index) => index,
            None => {
                cells.push(Cell {
                    scenario: scenario.to_owned(),
                    workload: workload.to_owned(),
                    factor,
                    runs: 0,
                    read: 0,
                    applies: true,
                    metrics: BTreeMap::new(),
                    budgets: Vec::new(),
                });
                cells.len() - 1
            }
        };
        let cell = &mut cells[index];
        if metric.starts_with("n/a") {
            cell.applies = false;
            continue;
        }
        let n: u32 = n.parse().unwrap_or(0);
        cell.runs = cell.runs.max(n);
        cell.read = cell.runs;
        if let Ok(median) = median.parse::<f64>() {
            cell.metrics.insert(metric.to_owned(), median);
        }
        let Ok(limit) = budget.parse::<f64>() else {
            continue;
        };
        let pan = scenario.starts_with("pan");
        let name = match metric {
            "frame_p99_ms" => "frame p99",
            "latency_p95_pan_ms" => "pan: pan latency p95",
            "latency_p95_zoom_ms" => "zoom: zoom latency p95",
            "latency_p95_ms" if pan => "pan latency p95",
            "latency_p95_ms" => "zoom latency p95",
            "preview_share_pan" => "pan: preview share",
            _ => "preview share",
        };
        cell.budgets.push(Check {
            budget: name.to_owned(),
            limit,
            n: n as usize,
            mean: mean.parse().ok(),
            ci: low.parse().ok().zip(high.parse().ok()).map(|(l, h)| [l, h]),
            verdict: verdict.to_owned(),
        });
    }
    cells
}

/// A report's outcome line: the bold line after `## Rule outcome`.
fn outcome(report: &str) -> Option<String> {
    report
        .lines()
        .skip_while(|line| !line.starts_with("## Rule outcome"))
        .filter_map(|line| line.strip_prefix("**")?.strip_suffix("**"))
        .last()
        .map(str::to_owned)
}

/// The text between `before` and `after` in `text`.
fn between(text: &str, before: &str, after: &str) -> Option<String> {
    let start = text.find(before)? + before.len();
    let end = text[start..].find(after)?;
    Some(text[start..start + end].trim().to_owned())
}

/// The number that ends right before the first `after` that follows
/// `before` in `text`.
fn number(text: &str, before: &str, after: &str) -> Option<f64> {
    let at = text.find(after)?;
    let start = text[..at].rfind(before)? + before.len();
    text[start..at].trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::results;
    use crate::archive;
    use crate::results::tests::{display, results as synthetic};

    #[test]
    fn a_levels_directory_and_its_archive_read_alike() {
        let root = std::env::temp_dir().join(format!("rustd3-load-{}", std::process::id()));
        let dir = root.join("20261029T120000Z-quick");
        fs::create_dir_all(&dir).unwrap();
        let written = synthetic(1.0, 2, display(Some(60.0)));
        fs::write(
            dir.join("results.json"),
            serde_json::to_string_pretty(&written).unwrap(),
        )
        .unwrap();
        let archive = root.join("rustd3-bench-quick-20261029T120000Z.tar.gz");
        archive::pack(&dir, &archive).unwrap();
        assert_eq!(results(&dir).unwrap(), written);
        assert_eq!(results(&archive).unwrap(), written);
        assert!(results(&root.join("missing")).is_err());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_layout_before_the_levels_reads_from_its_summaries() {
        let root = std::env::temp_dir().join(format!("rustd3-old-{}", std::process::id()));
        let dir = root.join("20261029T071522Z");
        for plan in ["default", "weak-nogpu", "smoke"] {
            fs::create_dir_all(dir.join(plan)).unwrap();
        }
        let csv = "scenario,workload,factor,metric,n,median,mean,ci_low,ci_high,budget,verdict\n\
                   pan-x,trace,1,frame_p50_ms,5,16.6653,16.6651,16.6643,16.6659,na,na\n\
                   pan-x,trace,1,frame_p99_ms,5,18.2764,18.2744,18.2610,18.2878,25.01,met\n\
                   pan-x,trace,1,latency_p95_ms,5,15.8,15.9,15.7,16.1,100,met\n\
                   live,scatter100k,1,n/a,0,na,na,na,na,na,n/a\n";
        fs::write(dir.join("default").join("summary.csv"), csv).unwrap();
        fs::write(dir.join("weak-nogpu").join("summary.csv"), csv).unwrap();
        fs::write(
            dir.join("default").join("report.md"),
            "# rustd3-bench 20261029T071522Z\n\nPlan `default`, windowed; backend wgpu, present \
             mode AutoVsync (iced default); 59.96 Hz (T = 16.68 ms), native factor 1; render \
             auto.\n\nAMD Ryzen 7 5800X 8-Core Processor (16 cores, 64215 MiB); kernel 6.17; GPU \
             NVIDIA GeForce RTX 3090, 595.104.02; session wayland, desktop COSMIC; rustc 1.97.0; \
             iced 3cbdacc; workspace 920ac26.\n\n## Rule outcome\n\n52 checks.\n\n**still \
             raster/upload bound after Phase 31 → GPU data layer recommended**\n",
        )
        .unwrap();
        let old = results(&dir).unwrap();
        let names: Vec<&str> = old.profiles.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["native", "nogpu"], "smoke has no summary");
        let native = &old.profiles[0];
        assert_eq!(
            native.outcome.as_deref(),
            Some("still raster/upload bound after Phase 31 → GPU data layer recommended")
        );
        let cell = &native.cells[0];
        assert_eq!((cell.name().as_str(), cell.runs), ("pan-x × trace × 1", 5));
        assert_eq!(cell.metrics["frame_p99_ms"], 18.2764);
        assert_eq!(cell.budgets[1].budget, "pan latency p95");
        assert!(!native.cells[1].applies);
        assert_eq!(native.checks.met, 2);
        assert_eq!(old.machine.cpu, "AMD Ryzen 7 5800X 8-Core Processor");
        assert_eq!(old.machine.cores, 16);
        assert_eq!(old.machine.gpu, "NVIDIA GeForce RTX 3090, 595.104.02");
        assert_eq!(
            (old.display.refresh_hz, old.display.native_factor),
            (59.96, 1.0)
        );
        assert_eq!(old.rustd3.commit, "920ac26");
        fs::remove_dir_all(&root).unwrap();
    }
}
