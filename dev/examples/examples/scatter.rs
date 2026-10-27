//! Writes `scatter.svg` and `scatter.png` into the directory given on the
//! command line: 5,600 points in three palette keys, three `Markers`
//! commands.

use std::path::PathBuf;

use rustd3_examples::scatter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir: PathBuf = std::env::args_os()
        .nth(1)
        .ok_or("usage: scatter <directory>")?
        .into();
    let fig = scatter::scatter(scatter::POINTS);
    for warning in fig.save_svg(dir.join("scatter.svg"), scatter::SIZE)? {
        eprintln!("scatter.svg: {warning:?}");
    }
    fig.save_png(dir.join("scatter.png"), scatter::SIZE, scatter::SCALE)?;
    Ok(())
}
