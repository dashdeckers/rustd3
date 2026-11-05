//! A benchmark's report, rebuilt from the raw records of a level's
//! directory or archive, or of one run's directory.
//!
//! ```text
//! cargo run --release --example bench_report -- bench-results/bench-quick-<stamp>.tar.gz
//! ```

use std::process::ExitCode;

fn main() -> ExitCode {
    benchkit::run(benchkit::Example::Report)
}
