//! The `Plot` widget on the software renderer offscreen, headless on any
//! machine: every frame composed into a pixmap and timed by part
//! (`--help` for the flags).
//!
//! ```text
//! cargo run --release --example bench_software -- --scenario zoom-lines --workload trace
//! ```

use std::process::ExitCode;

fn main() -> ExitCode {
    benchkit::run(benchkit::Example::Software)
}
