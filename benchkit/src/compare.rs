//! `bench_compare A B`: two results side by side, each a level's directory
//! or archive (or a suite's directory from before the levels). It shows
//! the machines, then per profile and cell the key metrics of A and B and
//! their ratio B/A (per figure and stage for the figure profile). Cells
//! and profiles that only one side has are marked.

use std::fmt::Write as _;

use crate::figure;
use crate::results::{self, Profile, Results, Status};

/// A row of the machines' table: its heading and what it shows of a
/// side's results.
type Row = (&'static str, fn(&Results) -> String);

/// The comparison of `a` and `b`, named `names`, as Markdown.
pub fn markdown(a: &Results, b: &Results, names: [&str; 2]) -> String {
    let mut md = format!(
        "# bench_compare\n\nA: `{}`\nB: `{}`\n\n",
        names[0], names[1]
    );
    machines(&mut md, a, b);
    reference(&mut md, a, b);
    let mut profiles: Vec<&str> = a.profiles.iter().map(|p| p.name.as_str()).collect();
    for profile in &b.profiles {
        if !profiles.contains(&profile.name.as_str()) {
            profiles.push(&profile.name);
        }
    }
    for name in profiles {
        let _ = writeln!(md, "## {name}\n");
        match (ran(a, name), ran(b, name)) {
            (Some(pa), Some(pb)) if name == figure::NAME => figures(&mut md, pa, pb),
            (Some(pa), Some(pb)) => cells(&mut md, pa, pb),
            (Some(_), None) => {
                let _ = writeln!(md, "Only in A{}.\n", why(b, name));
            }
            (None, Some(_)) => {
                let _ = writeln!(md, "Only in B{}.\n", why(a, name));
            }
            (None, None) => {
                let _ = writeln!(md, "In neither: {}; {}.\n", why(a, name), why(b, name));
            }
        }
    }
    md
}

/// The machines side by side.
fn machines(md: &mut String, a: &Results, b: &Results) {
    let rows: [Row; 10] = [
        ("Level", |r| {
            let only = if r.figure_only { ", figure only" } else { "" };
            format!("{}{only} ({})", r.level, r.stamp)
        }),
        ("rustd3", |r| {
            let commit = &r.rustd3.commit[..12.min(r.rustd3.commit.len())];
            format!("{} at {commit}", r.rustd3.version)
        }),
        ("System", |r| {
            let m = &r.machine;
            format!("{} ({}), kernel {}, {}", m.release, m.os, m.kernel, m.arch)
        }),
        ("CPU", |r| {
            format!("{}, {} cores", r.machine.cpu, r.machine.cores)
        }),
        ("Memory", |r| {
            format!("{:.1} GiB", r.machine.ram_mb as f64 / 1024.0)
        }),
        ("GPU", |r| r.machine.gpu.clone()),
        ("Session", |r| {
            format!("{}, desktop {}", r.machine.session, r.machine.desktop)
        }),
        ("Display", |r| {
            if r.figure_only {
                return "not measured (figure only)".to_owned();
            }
            let d = &r.display;
            let mode = d.headless.as_ref().map_or("windowed", |_| "headless");
            format!(
                "{mode}, {} Hz ({}), factor {} ({})",
                d.refresh_hz,
                d.refresh.name(),
                d.native_factor,
                d.factor.name()
            )
        }),
        ("Native backend", |r| {
            r.display
                .backend
                .clone()
                .unwrap_or_else(|| "unknown".to_owned())
        }),
        ("Experiment", |r| {
            r.experiment
                .as_ref()
                .map_or_else(|| "none".to_owned(), |e| e.describe().join("; "))
        }),
    ];
    let _ = writeln!(md, "| | A | B |\n| --- | --- | --- |");
    for (name, row) in rows {
        let _ = writeln!(md, "| {name} | {} | {} |", row(a), row(b));
    }
    let _ = writeln!(md);
}

/// The profile `name` of `results`, if it ran.
fn ran<'a>(results: &'a Results, name: &str) -> Option<&'a Profile> {
    results
        .profiles
        .iter()
        .find(|p| p.name == name && matches!(p.status, Status::Done | Status::Failed))
}

/// The CPU reference's timings of both and B/A.
fn reference(md: &mut String, a: &Results, b: &Results) {
    let mut names: Vec<&str> = a.cpu_reference.iter().map(|r| r.name.as_str()).collect();
    for timing in &b.cpu_reference {
        if !names.contains(&timing.name.as_str()) {
            names.push(&timing.name);
        }
    }
    if names.is_empty() {
        return;
    }
    let _ = writeln!(
        md,
        "## CPU reference\n\n| Timing | A ms | B ms | B/A |\n| --- | --- | --- | --- |"
    );
    for name in names {
        let ms = |r: &Results| {
            r.cpu_reference
                .iter()
                .find(|t| t.name == name)
                .and_then(|t| t.ms)
        };
        let (va, vb) = (ms(a), ms(b));
        let ratio = match (va, vb) {
            (Some(va), Some(vb)) if va > 0.0 => format!("{:.2}", vb / va),
            (Some(_) | None, Some(_) | None) => "—".to_owned(),
        };
        let _ = writeln!(
            md,
            "| {name} | {} | {} | {ratio} |",
            results::value(va, 2, false),
            results::value(vb, 2, false)
        );
    }
    let _ = writeln!(md);
}

/// Why `results` has no profile `name` to compare.
fn why(results: &Results, name: &str) -> String {
    match results.profiles.iter().find(|p| p.name == name) {
        None => String::new(),
        Some(p) => match (p.status, &p.reason) {
            (Status::Skipped, Some(reason)) => format!(" (skipped there: {reason})"),
            (Status::Pending, _) => " (not run there: the level stopped before it)".to_owned(),
            (Status::Skipped | Status::Done | Status::Failed, _) => String::new(),
        },
    }
}

/// Every cell of either profile: the key metrics of both and B/A.
fn cells(md: &mut String, a: &Profile, b: &Profile) {
    let _ = writeln!(
        md,
        "A ran on {}, B on {}.\n\n| Cell | Metric | A | B | B/A |\n| --- | --- | --- | --- | --- |",
        renderers(a),
        renderers(b)
    );
    let applies = |p: &Profile| -> Vec<String> {
        p.cells
            .iter()
            .filter(|c| c.applies)
            .map(results::Cell::name)
            .collect()
    };
    let mut names = applies(a);
    for name in applies(b) {
        if !names.contains(&name) {
            names.push(name);
        }
    }
    for name in names {
        let find = |p: &Profile| p.cells.iter().position(|c| c.applies && c.name() == name);
        match (find(a).map(|i| &a.cells[i]), find(b).map(|i| &b.cells[i])) {
            (Some(ca), Some(cb)) => {
                for (index, &(key, heading, digits, share)) in a.key().iter().enumerate() {
                    let (va, vb) = (ca.metrics.get(key).copied(), cb.metrics.get(key).copied());
                    let ratio = match (va, vb) {
                        (Some(va), Some(vb)) if va.abs() > 1e-12 => format!("{:.2}", vb / va),
                        (Some(_) | None, Some(_) | None) => "—".to_owned(),
                    };
                    let shown = if index == 0 { name.as_str() } else { "" };
                    let _ = writeln!(
                        md,
                        "| {shown} | {heading} | {} | {} | {ratio} |",
                        results::value(va, digits, share),
                        results::value(vb, digits, share)
                    );
                }
            }
            (Some(_), None) => {
                let _ = writeln!(md, "| {name} | only in A | | | |");
            }
            (None, Some(_)) => {
                let _ = writeln!(md, "| {name} | only in B | | | |");
            }
            (None, None) => {}
        }
    }
    let _ = writeln!(md);
}

/// Every figure of either figure profile: each stage's median of both and
/// B/A, then the peak RSS.
fn figures(md: &mut String, a: &Profile, b: &Profile) {
    let _ = writeln!(
        md,
        "Median ms after a cold repetition.\n\n| Figure | Stage | A | B | B/A |\n\
         | --- | --- | --- | --- | --- |"
    );
    let mut names: Vec<&str> = a.figures.iter().map(|f| f.name.as_str()).collect();
    for f in &b.figures {
        if !names.contains(&f.name.as_str()) {
            names.push(&f.name);
        }
    }
    for name in names {
        let (fa, fb) = (timed(a, name), timed(b, name));
        let (Some(fa), Some(fb)) = (fa, fb) else {
            let side = match (fa, fb) {
                (Some(_), _) => "A only",
                (None, Some(_)) => "B only",
                (None, None) => "neither",
            };
            let _ = writeln!(md, "| {name} | timed in {side} | | | |");
            continue;
        };
        let mut stages: Vec<&str> = fa.stages.iter().map(|t| t.stage.as_str()).collect();
        for t in &fb.stages {
            if !stages.contains(&t.stage.as_str()) {
                stages.push(&t.stage);
            }
        }
        let median = |f: &results::Figure, stage: &str| figure::find(f, stage).map(|t| t.median_ms);
        for (index, stage) in stages.iter().enumerate() {
            let shown = if index == 0 { name } else { "" };
            let (va, vb) = (median(fa, stage), median(fb, stage));
            let _ = writeln!(
                md,
                "| {shown} | {stage} | {} | {} | {} |",
                va.map_or_else(|| "n/a".to_owned(), figure::ms),
                vb.map_or_else(|| "n/a".to_owned(), figure::ms),
                ratio(va, vb)
            );
        }
        let (ra, rb) = (fa.rss_peak_mb, fb.rss_peak_mb);
        let _ = writeln!(
            md,
            "|  | peak RSS MiB | {} | {} | {} |",
            results::value(ra, 0, false),
            results::value(rb, 0, false),
            ratio(ra, rb)
        );
    }
    let _ = writeln!(md);
}

/// The figure `name` of `profile`, if it was timed (neither failed nor
/// skipped).
fn timed<'a>(profile: &'a Profile, name: &str) -> Option<&'a results::Figure> {
    profile
        .figures
        .iter()
        .find(|f| f.name == name && f.failed.is_none() && f.skipped.is_none())
}

/// B/A, or "—" without both or with A zero.
fn ratio(a: Option<f64>, b: Option<f64>) -> String {
    match (a, b) {
        (Some(a), Some(b)) if a.abs() > 1e-12 => format!("{:.2}", b / a),
        (Some(_) | None, Some(_) | None) => "—".to_owned(),
    }
}

/// The renderers a profile ran on.
fn renderers(profile: &Profile) -> String {
    if profile.ran.is_empty() {
        "an unrecorded renderer".to_owned()
    } else {
        profile.ran.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::markdown;
    use crate::results::tests::{cell, display, figures, results};
    use crate::results::{Profile, Status};

    #[test]
    fn two_machines_side_by_side_with_their_ratios() {
        let a = results(1.0, 16, display(Some(60.0)));
        let mut b = results(2.0, 2, display(None));
        b.machine.cpu = "Intel Celeron N4020".to_owned();
        // B has one more cell and lacks one; its X11 profile ran.
        b.profiles[0].cells.remove(1);
        b.profiles[0].cells.push(cell("hover", 2.0));
        b.profiles[1] = Profile {
            status: Status::Done,
            cells: vec![cell("pan-x", 3.0)],
            ..b.profiles[0].clone()
        };
        b.profiles[1].name = "nogpu-x11".to_owned();
        let md = markdown(&a, &b, ["a.tar.gz", "b"]);
        assert!(md.contains("A: `a.tar.gz`\nB: `b`"), "{md}");
        assert!(
            md.contains("| CPU | AMD Ryzen 7 5800X, 16 cores | Intel Celeron N4020, 2 cores |"),
            "{md}"
        );
        assert!(
            md.contains("| pan-x × trace × 1 | frame p50 ms | 16.7 | 33.4 | 2.00 |"),
            "{md}"
        );
        assert!(
            md.contains("|  | preview share | 5% | 10% | 2.00 |"),
            "{md}"
        );
        assert!(md.contains("| Display | windowed, 60 Hz (measured), factor 1 (measured) |"));
        assert!(md.contains("| Experiment | none | none |"), "{md}");
        assert!(
            md.contains("| zoom-lines × trace × 1 | only in A | | | |"),
            "{md}"
        );
        assert!(
            md.contains("| hover × trace × 1 | only in B | | | |"),
            "{md}"
        );
        assert!(
            md.contains(
                "## nogpu-x11\n\nOnly in B (skipped there: it runs only in a Linux Wayland session)."
            ),
            "{md}"
        );
    }

    #[test]
    fn a_sweep_compared_with_itself_lines_up_each_value() {
        let mut a = results(1.0, 16, display(Some(60.0)));
        a.experiment = Some(crate::experiment::Experiment {
            sweep: Some(crate::experiment::Sweep::parse("threads=1,2").unwrap()),
            ..crate::experiment::Experiment::default()
        });
        let native = a.profiles.remove(0);
        a.profiles = ["1", "2"]
            .map(|v| Profile {
                name: format!("software@threads={v}"),
                plan: "software".to_owned(),
                ..native.clone()
            })
            .into();
        let md = markdown(&a, &a, ["a", "a"]);
        assert!(
            md.contains("| Experiment | a sweep of threads over 1, 2, a profile per value |"),
            "{md}"
        );
        assert!(md.contains("## software@threads=1\n"), "{md}");
        assert!(md.contains("## software@threads=2\n"), "{md}");
        assert!(!md.contains("Only in"), "{md}");
        // The software profile's cells show its frame cost by part.
        assert!(md.contains("| cost p99 ms |"), "{md}");
    }

    #[test]
    fn figure_profiles_side_by_side_per_figure_and_stage() {
        let mut a = results(1.0, 16, display(Some(60.0)));
        let mut b = results(2.0, 2, display(None));
        a.profiles.insert(0, figures(1.0));
        b.profiles.insert(0, figures(2.0));
        let md = markdown(&a, &b, ["a", "b"]);
        assert!(
            md.contains("## figure\n\nMedian ms after a cold repetition."),
            "{md}"
        );
        assert!(
            md.contains("| trace | resolve | 41.3 | 82.6 | 2.00 |"),
            "{md}"
        );
        assert!(md.contains("|  | svg | 5.50 | 11.0 | 2.00 |"), "{md}");
        assert!(
            md.contains("|  | peak RSS MiB | 212 | 425 | 2.00 |"),
            "{md}"
        );
        assert!(md.contains("| heatmap | timed in neither | | | |"), "{md}");
        assert!(
            md.contains("| scatter10m | timed in neither | | | |"),
            "{md}"
        );
    }
}
