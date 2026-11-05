//! The figure profile's tables: `figure/summary.csv`, `figure/report.md`,
//! its part of the level's `report.md`, and the line each figure prints.

use std::fmt::Write as _;

use crate::results::{self, Figure, Timing};

/// How the stages are timed, for the reports.
pub const STAGES: &str = "Each figure is built from data in memory (`build`), \
resolved for vector output with a fresh cache (`resolve`: fitting, layout, typst text) and drawn \
into its list (`list`); SVG and PDF are written from that list (`svg`, `pdf`: what `save_svg` \
and `save_pdf` do after drawing). `png-1x` and `png-2x` are `save_png`'s whole work end to end, \
since a PNG is drawn for a file target with its own resolve and list: a fresh cache, vello_cpu \
on its own pool, the encoder. `gif` is three frames through `gif::Writer`, end to end. Every \
output is written to memory, so no disk is timed. Each figure runs in a process of its own, \
so its first repetition is cold as a program's first figure is (fonts parsed, typst started, \
caches and pages first touched): it is reported apart, and the median, least and 90th \
percentile are of the repetitions after it.";

/// `figure/summary.csv`: one row per figure and stage.
pub fn csv(profile: &results::Profile) -> String {
    let mut out = String::from(
        "figure,stage,repetitions,cold_ms,median_ms,min_ms,p90_ms,bytes,rss_peak_mb\n",
    );
    for figure in &profile.figures {
        for t in &figure.stages {
            let _ = writeln!(
                out,
                "{},{},{},{:.3},{:.3},{:.3},{:.3},{},{}",
                figure.name,
                t.stage,
                t.repetitions,
                t.cold_ms,
                t.median_ms,
                t.min_ms,
                t.p90_ms,
                t.bytes.map_or_else(String::new, |b| b.to_string()),
                figure
                    .rss_peak_mb
                    .map_or_else(String::new, |mb| format!("{mb:.1}"))
            );
        }
    }
    out
}

/// `figure/report.md`: what the profile measures, then per figure every
/// stage's cold time, median, least, 90th percentile and bytes.
pub fn markdown(profile: &results::Profile) -> String {
    let mut md = format!(
        "# The figure profile\n\n{}.\n\n{STAGES} Peak RSS is the figure's process's \
         resident high-water mark while its stages ran, its data held (Linux); \"n/a\" \
         elsewhere.\n\n",
        profile.what
    );
    for figure in &profile.figures {
        let _ = writeln!(
            md,
            "## {}\n\n{}, {} × {} px. Peak RSS: {}.\n",
            figure.name,
            figure.what,
            figure.size[0],
            figure.size[1],
            rss(figure.rss_peak_mb)
        );
        if let Some(why) = &figure.failed {
            let _ = writeln!(md, "Failed: {why}\n");
            continue;
        }
        if let Some(why) = &figure.skipped {
            let _ = writeln!(md, "Skipped: {why}\n");
            continue;
        }
        let _ = writeln!(
            md,
            "| Stage | cold ms | median ms | min ms | p90 ms | of | bytes |\n\
             | --- | --- | --- | --- | --- | --- | --- |"
        );
        for t in &figure.stages {
            let _ = writeln!(
                md,
                "| {} | {} | {} | {} | {} | {} | {} |",
                t.stage,
                ms(t.cold_ms),
                ms(t.median_ms),
                ms(t.min_ms),
                ms(t.p90_ms),
                t.repetitions,
                t.bytes.map_or_else(|| "—".to_owned(), bytes)
            );
        }
        let _ = writeln!(md);
    }
    md
}

/// The profile's part of the level's `report.md`: per figure each
/// stage's median, its peak RSS, and a failure's reason.
pub fn summary(profile: &results::Profile) -> String {
    let mut stages: Vec<&str> = Vec::new();
    for figure in &profile.figures {
        for t in &figure.stages {
            if !stages.contains(&t.stage.as_str()) {
                stages.push(&t.stage);
            }
        }
    }
    let repetitions = profile
        .figures
        .iter()
        .flat_map(|f| &f.stages)
        .map(|t| t.repetitions)
        .max()
        .unwrap_or(0);
    let mut md = format!(
        "Median ms of {repetitions} repetitions after a cold one; `figure/report.md` has the \
         cold times, the spread and the outputs' bytes.\n\n| Figure | px |"
    );
    let mut rule = String::from("| --- | --- |");
    for stage in &stages {
        let _ = write!(md, " {stage} |");
        rule.push_str(" --- |");
    }
    let _ = writeln!(md, " peak RSS |\n{rule} --- |");
    for figure in &profile.figures {
        let _ = write!(
            md,
            "| {} | {} × {} |",
            figure.name, figure.size[0], figure.size[1]
        );
        let missing = (figure.failed.as_ref().map(|why| format!("failed: {why}")))
            .or_else(|| figure.skipped.as_ref().map(|why| format!("skipped: {why}")));
        match missing {
            Some(why) => {
                let _ = write!(md, " {why} |");
                for _ in 1..stages.len() {
                    md.push_str(" |");
                }
            }
            None => {
                for stage in &stages {
                    let cell =
                        find(figure, stage).map_or_else(|| "—".to_owned(), |t| ms(t.median_ms));
                    let _ = write!(md, " {cell} |");
                }
            }
        }
        let _ = writeln!(md, " {} |", rss(figure.rss_peak_mb));
    }
    md.push('\n');
    md
}

/// One figure's line on the console: its medians, or why it failed.
pub fn line(figure: &Figure) -> String {
    if let Some(why) = &figure.failed {
        return format!("failed: {why}");
    }
    if let Some(why) = &figure.skipped {
        return format!("skipped: {why}");
    }
    let stages: Vec<String> = figure
        .stages
        .iter()
        .map(|t| format!("{} {}", t.stage, ms(t.median_ms)))
        .collect();
    format!(
        "{} ms; peak RSS {}",
        stages.join(", "),
        rss(figure.rss_peak_mb)
    )
}

/// The timing of `stage` in `figure`.
pub fn find<'a>(figure: &'a Figure, stage: &str) -> Option<&'a Timing> {
    figure.stages.iter().find(|t| t.stage == stage)
}

/// Milliseconds with two significant decimals below 10 and one below 100.
pub fn ms(v: f64) -> String {
    if v < 10.0 {
        format!("{v:.2}")
    } else if v < 100.0 {
        format!("{v:.1}")
    } else {
        format!("{v:.0}")
    }
}

/// A peak RSS in MiB, or "n/a".
fn rss(mb: Option<f64>) -> String {
    mb.map_or_else(|| "n/a".to_owned(), |mb| format!("{mb:.0} MiB"))
}

/// Bytes in kB or MB (10³, 10⁶).
fn bytes(n: u64) -> String {
    let n = n as f64;
    if n < 1e6 {
        format!("{:.1} kB", n / 1e3)
    } else {
        format!("{:.1} MB", n / 1e6)
    }
}

#[cfg(test)]
mod tests {
    use super::{csv, line, markdown, ms, summary};
    use crate::results::tests::figures;

    #[test]
    fn the_tables_show_every_stage_and_the_failures() {
        let p = figures(1.0);
        assert_eq!(
            csv(&p),
            "figure,stage,repetitions,cold_ms,median_ms,min_ms,p90_ms,bytes,rss_peak_mb\n\
             trace,resolve,3,123.900,41.300,37.170,49.560,,212.4\n\
             trace,svg,3,16.500,5.500,4.950,6.600,1234567,212.4\n"
        );
        let md = markdown(&p);
        assert!(
            md.contains("| svg | 16.5 | 5.50 | 4.95 | 6.60 | 3 | 1.2 MB |"),
            "{md}"
        );
        assert!(md.contains("Failed: pdf: too large"), "{md}");
        assert!(md.contains("Skipped: it needs about 13.8 GiB"), "{md}");
        assert!(md.contains("Peak RSS: n/a."), "{md}");
        let table = summary(&p);
        assert!(
            table.contains("| Figure | px | resolve | svg | peak RSS |"),
            "{table}"
        );
        assert!(
            table.contains("| trace | 1600 × 1000 | 41.3 | 5.50 | 212 MiB |"),
            "{table}"
        );
        assert!(
            table.contains("| heatmap | 1600 × 1000 | failed: pdf: too large | | n/a |"),
            "{table}"
        );
        assert_eq!(
            line(&p.figures[0]),
            "resolve 41.3, svg 5.50 ms; peak RSS 212 MiB"
        );
        assert_eq!([ms(0.123), ms(12.34), ms(1234.4)], ["0.12", "12.3", "1234"]);
    }
}
