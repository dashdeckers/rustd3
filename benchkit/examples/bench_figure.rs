//! The benchmark of the plotting library alone: the CPU reference and the
//! figure profile, headless, no window (`--help` for the levels).
//!
//! ```text
//! cargo run --release --example bench_figure -- --quick
//! ```

use std::process::ExitCode;

fn main() -> ExitCode {
    benchkit::run(benchkit::Example::Figure)
}
