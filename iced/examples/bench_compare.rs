//! Two benchmark results, directories or archives, side by side.
//!
//! ```text
//! cargo run --release --example bench_compare -- A.tar.gz B.tar.gz
//! ```

use std::process::ExitCode;

fn main() -> ExitCode {
    benchkit::run(benchkit::Example::Compare)
}
