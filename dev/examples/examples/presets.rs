//! Writes every chart preset of `helpers::chart` as SVG and as a PNG at 2×
//! into the directory given on the command line, `target/presets` by
//! default: lines, the symlog line of Part I's Helpers, a scatter, grouped,
//! stacked and diverging bars, a stacked area, a pie, a donut, a heatmap
//! and a keyed heatmap, each at the presets' 640 × 480. Prints what
//! `fig.check` finds for each (nothing, at that size).

use std::path::{Path, PathBuf};

use rustd3_examples::presets;

/// Device px per logical px of the PNGs.
const SCALE: f64 = 2.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::args_os().nth(1).map_or_else(
        || PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/presets")),
        PathBuf::from,
    );
    std::fs::create_dir_all(&dir)?;
    let data = presets::Data::new();
    for (name, fig) in presets::figures(&data) {
        if name != "line_symlog" {
            write(&dir, name, fig)?;
            continue;
        }
        // Part I's sketch, written by the preset's own savers.
        for issue in fig.check(presets::SIZE) {
            eprintln!("line_symlog: {issue:?}");
        }
        presets::rms(&data).save_svg(dir.join("line_symlog.svg"))?;
        presets::rms(&data).save_png(dir.join("line_symlog.png"), SCALE)?;
    }
    Ok(())
}

/// Writes `fig` as `<name>.svg` and `<name>.png` at the presets' default
/// size into `dir`, printing what `fig.check` finds.
fn write(
    dir: &Path,
    name: &str,
    fig: rustd3_figure::Figure<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let at = presets::SIZE;
    for issue in fig.check(at) {
        eprintln!("{name}: {issue:?}");
    }
    for warning in fig.save_svg(dir.join(format!("{name}.svg")), at)? {
        eprintln!("{name}.svg: {warning:?}");
    }
    fig.save_png(dir.join(format!("{name}.png")), at, SCALE)?;
    Ok(())
}
