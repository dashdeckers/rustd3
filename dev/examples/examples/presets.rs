//! Writes every chart preset of `helpers::chart` as SVG and as a PNG at 2×
//! into the directory given on the command line, `target/presets` by
//! default: lines, the symlog line of Part I's Helpers, a scatter, grouped,
//! stacked and diverging bars, a stacked area, a pie, a donut, a heatmap
//! and a keyed heatmap, each at the presets' 640 × 480. Prints what
//! `fig.check` finds for each (nothing, at that size).

use std::f64::consts::PI;
use std::path::{Path, PathBuf};

use rustd3_core::scale;
use rustd3_core::scheme;
use rustd3_core::series::{Clock, Recording};
use rustd3_figure::check::size;
use rustd3_figure::helpers::chart::{self, bar::Mode};
use rustd3_figure::mark::Grid;
use rustd3_figure::text;

/// Device px per logical px of the PNGs.
const SCALE: f64 = 2.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::args_os().nth(1).map_or_else(
        || PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/presets")),
        PathBuf::from,
    );
    std::fs::create_dir_all(&dir)?;
    let t: Vec<f64> = (0..=100).map(|i| f64::from(i) * 0.1).collect();
    let samples: Vec<f64> = (0..2000)
        .map(|i| 50.0 * (f64::from(i) * 0.01).sin().powi(3))
        .collect();
    let rec = Recording::new(
        Clock::Regular {
            start: 0.0,
            period: 0.01,
        },
        [&samples],
    )?;
    let week = [
        ("mon", vec![3.0, 2.5, 1.0]),
        ("tue", vec![4.0, 1.0, 2.0]),
        ("wed", vec![2.0, 3.5, 0.5]),
        ("thu", vec![5.0, 2.0, 1.5]),
    ];
    let regions = || vec!["north".to_string(), "south".into(), "east".into()];
    let hours: Vec<f64> = (0..24).map(f64::from).collect();
    let solar = hours
        .iter()
        .map(|h| (2.5 * (PI * (h - 6.0) / 12.0).sin()).max(0.0))
        .collect();
    let wind = hours.iter().map(|h| 1.5 + 0.5 * (h * 0.4).cos()).collect();
    let spending = [
        ("rent", 1200.0),
        ("food", 450.0),
        ("travel", 300.0),
        ("other", 150.0),
    ];
    let field = (0..60)
        .map(|i| {
            let (c, r) = (f64::from(i % 10), f64::from(i / 10));
            (c * 0.6).sin() * (r * 0.5).cos() + 0.1 * r
        })
        .collect();
    let names: Vec<String> = ["speed", "load", "temp", "vibration"]
        .map(String::from)
        .to_vec();
    let correlation = vec![
        1.0, 0.8, -0.2, 0.3, 0.8, 1.0, 0.1, 0.5, -0.2, 0.1, 1.0, -0.6, 0.3, 0.5, -0.6, 1.0,
    ];

    write(
        &dir,
        "line",
        chart::line(t.iter().map(|&t| (t, t.sin())))
            .series(t.iter().map(|&t| (t, 0.5 * (0.7 * t).cos())), "cos")
            .series(t.iter().map(|&t| (t, 0.1 * t - 0.5)), "trend")
            .title("line")
            .x_label("t [s]")
            .y_label("value")
            .figure(),
    )?;
    // Part I's sketch, written by the preset's own savers.
    let channel = rec.series(0).ok_or("channel 0")?;
    let rms = || {
        chart::line(channel)
            .y_scale(scale::symlog().constant(1.0))
            .minor(true)
            .title(text::typst("RMS [$m slash s^2$]"))
    };
    for issue in rms().figure().check(size(640.0, 480.0)) {
        eprintln!("line_symlog: {issue:?}");
    }
    rms().save_svg(dir.join("line_symlog.svg"))?;
    rms().save_png(dir.join("line_symlog.png"), SCALE)?;
    let points = (0..60).map(|i| {
        let k = f64::from(i);
        (
            k * 0.37 % 10.0,
            (k * 1.3).sin() * 1.5 + 0.03 * k + (k * 0.71).cos(),
        )
    });
    write(
        &dir,
        "scatter",
        chart::scatter(points)
            .title("scatter")
            .x_label("x")
            .y_label("y")
            .figure(),
    )?;
    for (mode, name) in [
        (Mode::Grouped, "bar_grouped"),
        (Mode::Stacked, "bar_stacked"),
    ] {
        let bars = chart::bar(week.clone(), regions())
            .mode(mode)
            .title(name)
            .y_label("orders");
        write(&dir, name, bars.figure())?;
    }
    write(
        &dir,
        "bar_diverging",
        chart::bar(
            [
                ("k0", vec![3.0, -1.0, 2.0]),
                ("k1", vec![-2.0, 4.0, 1.0]),
                ("k2", vec![1.5, -2.5, -1.0]),
            ],
            vec!["a".into(), "b".into(), "c".into()],
        )
        .mode(Mode::Diverging)
        .title("bar_diverging")
        .figure(),
    )?;
    write(
        &dir,
        "area",
        chart::area(vec![solar, wind, vec![1.0; 24]], hours)
            .title("area")
            .x_label("hour")
            .y_label("GW")
            .figure(),
    )?;
    write(&dir, "pie", chart::pie(spending).title("pie").figure())?;
    write(
        &dir,
        "donut",
        chart::pie(spending).inner(0.5).title("donut").figure(),
    )?;
    write(
        &dir,
        "heatmap",
        chart::heatmap(Grid::new(10, 6, field)?)
            .title("heatmap")
            .x_label("x")
            .y_label("y")
            .figure(),
    )?;
    write(
        &dir,
        "heatmap_keys",
        chart::heatmap(Grid::new(4, 4, correlation)?)
            .keys(names.clone(), names)
            .colors(scheme::magma)
            .title("correlation")
            .figure(),
    )?;
    Ok(())
}

/// Writes `fig` as `<name>.svg` and `<name>.png` at the presets' default
/// size into `dir`, printing what `fig.check` finds.
fn write(
    dir: &Path,
    name: &str,
    fig: rustd3_figure::Figure<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let at = size(640.0, 480.0);
    for issue in fig.check(at) {
        eprintln!("{name}: {issue:?}");
    }
    for warning in fig.save_svg(dir.join(format!("{name}.svg")), at)? {
        eprintln!("{name}.svg: {warning:?}");
    }
    fig.save_png(dir.join(format!("{name}.png")), at, SCALE)?;
    Ok(())
}
