//! Writes `trace.svg` and `trace.png` into the directory given on the
//! command line: three channels of a million samples each, thinned at draw
//! time.

use std::path::PathBuf;

use benchkit::gallery::trace;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir: PathBuf = std::env::args_os()
        .nth(1)
        .ok_or("usage: trace <directory>")?
        .into();
    let rec = trace::recording(trace::SAMPLES);
    let fig = trace::trace(&rec);
    for warning in fig.save_svg(dir.join("trace.svg"), trace::SIZE)? {
        eprintln!("trace.svg: {warning:?}");
    }
    fig.save_png(dir.join("trace.png"), trace::SIZE, trace::SCALE)?;
    Ok(())
}
