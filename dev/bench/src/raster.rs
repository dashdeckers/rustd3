//! The `raster` command: the two raster variants, envelope fills and
//! marker splatting, on every plot-area cell of the plan's workloads at
//! factor 1 and 2 in the standard window and at 4K, each rendered with the
//! figure's raster path on one vello_cpu thread (as each thread of the
//! widget's pool renders): thinned lines stroked as drawn against envelope
//! fills ([`envelope`]), and markers filled point by point against
//! splatted coverage stamps ([`splat`]). Writes `raster.csv` and prints
//! the table with the rule each variant must meet to be built.

mod envelope;
mod splat;

use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::time::Instant;

use iced::Size;
use rustd3_figure::cache::Cache;
use rustd3_figure::kurbo::{self, Rect};
use rustd3_figure::list::{Bounds, Outlines};
use rustd3_figure::raster::{Image, Renderer, Settings, Simd};
use rustd3_figure::{Held, Target};

use crate::cli::widget::Options;
use crate::scenario::Scenario;
use crate::stats::percentile;
use crate::workload::Workload;

/// Renders per variant and cell; the median is reported.
const RUNS: usize = 7;

/// The least speedup a variant needs to be built.
pub const SPEEDUP: f64 = 1.5;

/// The largest channel difference a variant may make to be built, /255.
pub const MAX_DIFF: u8 = 32;

/// A window and scale factor the variants render at.
#[derive(Debug, Clone, Copy)]
struct Config {
    /// Its name in `raster.csv`'s factor column.
    name: &'static str,
    /// The window, logical px.
    size: Size,
    /// Device px per logical px.
    factor: f64,
}

/// One row of `raster.csv`: a variant's time summed over a workload's
/// plot-area cells, and against the cells as drawn its speedup, largest
/// channel difference and share of channels off by more than 16/255.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// The workload's name.
    pub workload: &'static str,
    /// `1`, `2` or `4k`.
    pub factor: &'static str,
    /// `stroke` or `markers` (as drawn), `envelope` or `splat`.
    pub variant: &'static str,
    /// Median render time summed over the cells, ms.
    pub ms: f64,
    /// The as-drawn time over this one; for variants only.
    pub speedup: Option<f64>,
    /// The largest channel difference from the as-drawn render, /255.
    pub max: Option<u8>,
    /// Channels off by more than 16/255, percent.
    pub over: Option<f64>,
}

/// One variant measured over a workload's cells.
#[derive(Debug, Default)]
struct Sum {
    drawn_ms: f64,
    ms: f64,
    max: u8,
    over: u64,
    channels: u64,
    cells: usize,
}

/// Runs the variants over the plan's workloads, writes `raster.csv` into
/// `options.out` and prints the table.
pub fn run(options: &Options) -> Result<(), Box<dyn Error>> {
    let plan = options.plan();
    let mut workloads: Vec<Workload> = plan
        .cells(2.0, &options.selection)
        .iter()
        .map(|c| c.workload)
        .collect();
    workloads.sort();
    workloads.dedup();
    if workloads.is_empty() {
        return Err("the flags select no workload (see `rustd3-bench list`)".into());
    }
    let configs = [
        Config {
            name: "1",
            size: Scenario::PanX.window(1.0),
            factor: 1.0,
        },
        Config {
            name: "2",
            size: Scenario::PanX.window(2.0),
            factor: 2.0,
        },
        Config {
            name: "4k",
            size: Scenario::FourK.window(2.0),
            factor: 2.0,
        },
    ];
    let scale = options.data_scale();
    let mut rows = Vec::new();
    for workload in workloads {
        let data = workload.data(scale);
        for config in configs {
            let begin = Instant::now();
            let measured = measure(workload, &data, config)?;
            println!(
                "rustd3-bench raster: {} × {} in {:.1} s",
                workload.name(),
                config.name,
                begin.elapsed().as_secs_f64()
            );
            rows.extend(measured);
        }
    }
    fs::create_dir_all(&options.out)?;
    let path = options.out.join("raster.csv");
    fs::write(&path, csv(&rows))?;
    print!("{}", table(&rows));
    println!("raster.csv: {}", path.display());
    Ok(())
}

/// The rows of one workload at one configuration.
fn measure(
    workload: Workload,
    data: &crate::workload::Data,
    config: Config,
) -> Result<Vec<Row>, Box<dyn Error>> {
    let k = config.factor;
    let size = kurbo::Size::new(f64::from(config.size.width), f64::from(config.size.height));
    let fig = data.figure(false);
    let target = Target::Screen { scale: k };
    let mut cache = Cache::default();
    let list = fig.draw_with(&mut cache, size, target, &Held::default());
    let areas: Vec<Rect> = fig
        .resolve(size, &mut cache, &Held::default(), target)
        .plot_areas()
        .to_vec();
    let bounds = Bounds::of(&list, &mut Outlines::default());
    let mut renderer = Renderer::new(Settings {
        threads: 0,
        simd: Simd::Detect,
    });
    let (mut fills, mut splats) = (Sum::default(), Sum::default());
    let mut stamps = splat::Stamps::default();
    let mut cold: Vec<f64> = Vec::new();
    for &area in &areas {
        let (drawn_ms, drawn) = timed(|| renderer.render(&list, &bounds, area, k))?;
        if let Some(filled) = envelope::envelope(&list, area, k) {
            let filled_bounds = Bounds::of(&filled, &mut Outlines::default());
            let (ms, image) = timed(|| renderer.render(&filled, &filled_bounds, area, k))?;
            fills.add(drawn_ms, ms, &drawn, &image);
        }
        if let Some(plan) = splat::Plan::new(&list, &bounds, area, k) {
            let begin = Instant::now();
            let _ = plan.render(&mut renderer, &mut stamps)?;
            cold.push(begin.elapsed().as_secs_f64() * 1e3);
            let (ms, image) = timed(|| plan.render(&mut renderer, &mut stamps))?;
            splats.add(drawn_ms, ms, &drawn, &image);
        }
    }
    if !cold.is_empty() {
        println!(
            "  splat's first render with no stamps made: {:.2} ms over {} cells",
            cold.iter().sum::<f64>(),
            cold.len()
        );
    }
    let mut rows = Vec::new();
    for (sum, drawn, variant) in [(fills, "stroke", "envelope"), (splats, "markers", "splat")] {
        if sum.cells == 0 {
            continue;
        }
        rows.push(Row {
            workload: workload.name(),
            factor: config.name,
            variant: drawn,
            ms: sum.drawn_ms,
            speedup: None,
            max: None,
            over: None,
        });
        rows.push(Row {
            workload: workload.name(),
            factor: config.name,
            variant,
            ms: sum.ms,
            speedup: Some(sum.drawn_ms / sum.ms),
            max: Some(sum.max),
            over: Some(100.0 * sum.over as f64 / sum.channels.max(1) as f64),
        });
    }
    Ok(rows)
}

impl Sum {
    /// Adds one cell: its as-drawn and variant times and their images.
    fn add(&mut self, drawn_ms: f64, ms: f64, drawn: &Image, image: &Image) {
        self.drawn_ms += drawn_ms;
        self.ms += ms;
        self.cells += 1;
        for (a, b) in drawn.rgba.iter().zip(&image.rgba) {
            let diff = a.abs_diff(*b);
            self.max = self.max.max(diff);
            self.over += u64::from(diff > 16);
        }
        self.channels += drawn.rgba.len() as u64;
    }
}

/// The median time of [`RUNS`] calls of `render`, ms, and its last image.
fn timed(
    mut render: impl FnMut() -> Result<Image, rustd3_figure::raster::Error>,
) -> Result<(f64, Image), rustd3_figure::raster::Error> {
    let mut times = Vec::with_capacity(RUNS);
    let mut last = None;
    for _ in 0..RUNS {
        let begin = Instant::now();
        let image = render()?;
        times.push(begin.elapsed().as_secs_f64() * 1e3);
        last = Some(image);
    }
    let image = last.expect("invariant: RUNS > 0");
    Ok((percentile(&times, 50.0).unwrap_or(0.0), image))
}

/// `raster.csv`: a header and one line per row.
fn csv(rows: &[Row]) -> String {
    let mut out = String::from("workload,factor,variant,ms,speedup,max_diff,pct_over_16\n");
    for row in rows {
        let _ = writeln!(
            out,
            "{},{},{},{:.3},{},{},{}",
            row.workload,
            row.factor,
            row.variant,
            row.ms,
            row.speedup.map_or(String::new(), |s| format!("{s:.3}")),
            row.max.map_or(String::new(), |m| m.to_string()),
            row.over.map_or(String::new(), |o| format!("{o:.4}")),
        );
    }
    out
}

/// The rows as a table, each variant against the rule.
fn table(rows: &[Row]) -> String {
    let mut out = format!(
        "| workload | factor | variant | ms | speedup | max diff | % > 16 | rule (≥ {SPEEDUP}×, ≤ \
         {MAX_DIFF}/255) |\n| --- | --- | --- | --- | --- | --- | --- | --- |\n"
    );
    for row in rows {
        let rule = match (row.speedup, row.max) {
            (Some(s), Some(m)) if s >= SPEEDUP && m <= MAX_DIFF => "meets",
            (Some(_), Some(_)) => "misses",
            (None, _) | (_, None) => "",
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} | {:.2} | {} | {} | {} | {rule} |",
            row.workload,
            row.factor,
            row.variant,
            row.ms,
            row.speedup.map_or(String::new(), |s| format!("{s:.2}")),
            row.max.map_or(String::new(), |m| m.to_string()),
            row.over.map_or(String::new(), |o| format!("{o:.3}")),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{Row, csv, table};

    #[test]
    fn raster_csv_has_the_sections_columns() {
        let rows = [
            Row {
                workload: "trace",
                factor: "2",
                variant: "stroke",
                ms: 12.0,
                speedup: None,
                max: None,
                over: None,
            },
            Row {
                workload: "trace",
                factor: "2",
                variant: "envelope",
                ms: 4.0,
                speedup: Some(3.0),
                max: Some(40),
                over: Some(0.5),
            },
        ];
        let text = csv(&rows);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(
            lines,
            [
                "workload,factor,variant,ms,speedup,max_diff,pct_over_16",
                "trace,2,stroke,12.000,,,",
                "trace,2,envelope,4.000,3.000,40,0.5000",
            ]
        );
        // 3× faster but 40/255 off: the rule is missed.
        assert!(table(&rows).contains("| misses |"));
    }
}
