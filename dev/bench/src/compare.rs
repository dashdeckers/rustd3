//! `rustd3-bench compare A B` (D22.9 (5)): two results, each a level's
//! directory or archive (or the layout before D22.9), side by side: the
//! machines, then per profile and cell the key metrics of A and B and
//! their ratio B/A, with the cells and profiles only one side has marked.

use std::fmt::Write as _;

use crate::results::{self, Profile, Results, Status};

/// A row of the machines' table: its heading and what it shows of a
/// side's results.
type Row = (&'static str, fn(&Results) -> String);

/// The comparison of `a` and `b`, named `names`, as Markdown.
pub fn markdown(a: &Results, b: &Results, names: [&str; 2]) -> String {
    let mut md = format!(
        "# rustd3-bench compare\n\nA: `{}`\nB: `{}`\n\n",
        names[0], names[1]
    );
    machines(&mut md, a, b);
    let mut profiles: Vec<&str> = a.profiles.iter().map(|p| p.name.as_str()).collect();
    for profile in &b.profiles {
        if !profiles.contains(&profile.name.as_str()) {
            profiles.push(&profile.name);
        }
    }
    for name in profiles {
        let _ = writeln!(md, "## {name}\n");
        match (ran(a, name), ran(b, name)) {
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
    let rows: [Row; 9] = [
        ("Level", |r| format!("{} ({})", r.level, r.stamp)),
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
                for (index, (key, heading, digits, share)) in results::KEY.into_iter().enumerate() {
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
    use crate::results::tests::{cell, display, results};
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
}
