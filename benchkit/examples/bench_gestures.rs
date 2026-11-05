//! The `Plot` widget driven by replayed gestures in windows. A plan crosses
//! scenarios (hover, pans, zooms, resizes) with workloads and scale
//! factors, and each run is a cold process on the native backend or the
//! software renderer (`--help` for the plans and flags).
//!
//! ```text
//! cargo run --release --example bench_gestures -- --plan quick --refresh-hz 60 --native-factor 1
//! ```

use std::process::ExitCode;

fn main() -> ExitCode {
    benchkit::run(benchkit::Example::Gestures)
}
