//! Writes the wheel figure, `wheels.svg` and `wheels.png` (the legend
//! below the grid) and `wheels_inset.svg` and `wheels_inset.png` (the
//! legend in FL's scatter), at 960 × 760 with the PNGs at 2×, into the
//! directory given on the command line, `target/wheels` by default.

use std::path::PathBuf;

use rustd3_examples::wheels;

/// Device px per logical px of the PNGs.
const SCALE: f64 = 2.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir: PathBuf = std::env::args_os().nth(1).map_or_else(
        || PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/wheels")),
        PathBuf::from,
    );
    std::fs::create_dir_all(&dir)?;
    let samples = wheels::simulate();
    for (inset, name) in [(false, "wheels"), (true, "wheels_inset")] {
        let fig = wheels::figure(&samples, inset);
        for warning in fig.save_svg(dir.join(format!("{name}.svg")), wheels::SIZE)? {
            eprintln!("{name}.svg: {warning:?}");
        }
        fig.save_png(dir.join(format!("{name}.png")), wheels::SIZE, SCALE)?;
    }
    Ok(())
}
