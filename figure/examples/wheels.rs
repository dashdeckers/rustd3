//! Writes the wheel figure, `wheels.svg` and `wheels.png` (the legend
//! below the grid) and `wheels_inset.svg` and `wheels_inset.png` (the
//! legend in FL's scatter), at 960 × 760 with the PNGs at 2×, into the
//! directory given on the command line, `target/wheels` by default.
//!
//! `--theme <style>:<context>` draws it in one of seaborn's presets
//! (styles `darkgrid`, `whitegrid`, `dark`, `white`, `ticks`; contexts
//! `paper`, `notebook`, `talk`, `poster`), adds `_<style>_<context>` to the
//! file names and prints what `fig.check` finds at that size: talk and
//! poster are too big for 960 × 760. `--hatch` hatches the histograms'
//! bars at 45° and adds `_hatched` to the file names.

use std::path::PathBuf;

use benchkit::gallery::wheels;
use rustd3_figure::Theme;
use rustd3_figure::kurbo::Size;
use rustd3_figure::theme::{Context, Style};

/// Device px per logical px of the PNGs.
const SCALE: f64 = 2.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dir = None;
    let mut preset = None;
    let mut hatch = false;
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--hatch" {
            hatch = true;
        } else if arg == "--theme" {
            let spec = args.next().ok_or("--theme takes <style>:<context>")?;
            let spec = spec.to_str().ok_or("--theme takes <style>:<context>")?;
            preset = Some(theme(spec)?);
        } else {
            dir = Some(PathBuf::from(arg));
        }
    }
    let dir = dir
        .unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../target/wheels")));
    std::fs::create_dir_all(&dir)?;
    let samples = wheels::simulate();
    for (inset, name) in [(false, "wheels"), (true, "wheels_inset")] {
        let (mut fig, name) = if hatch {
            (wheels::hatched(&samples, inset), format!("{name}_hatched"))
        } else {
            (wheels::figure(&samples, inset), name.to_string())
        };
        let name = match &preset {
            Some((theme, suffix)) => {
                fig.theme(theme.clone());
                for issue in fig.check(Size::new(wheels::SIZE.width, wheels::SIZE.height)) {
                    eprintln!("{name}{suffix}: {issue:?}");
                }
                format!("{name}{suffix}")
            }
            None => name,
        };
        for warning in fig.save_svg(dir.join(format!("{name}.svg")), wheels::SIZE)? {
            eprintln!("{name}.svg: {warning:?}");
        }
        fig.save_png(dir.join(format!("{name}.png")), wheels::SIZE, SCALE)?;
    }
    Ok(())
}

/// The seaborn preset `spec` names (`<style>:<context>`), and the suffix
/// its files get.
fn theme(spec: &str) -> Result<(Theme, String), String> {
    let usage = || format!("--theme takes <style>:<context>, not `{spec}`");
    let (style, context) = spec.split_once(':').ok_or_else(usage)?;
    let s = match style {
        "darkgrid" => Style::DarkGrid,
        "whitegrid" => Style::WhiteGrid,
        "dark" => Style::Dark,
        "white" => Style::White,
        "ticks" => Style::Ticks,
        _ => return Err(usage()),
    };
    let c = match context {
        "paper" => Context::Paper,
        "notebook" => Context::Notebook,
        "talk" => Context::Talk,
        "poster" => Context::Poster,
        _ => return Err(usage()),
    };
    Ok((Theme::seaborn(s, c), format!("_{style}_{context}")))
}
