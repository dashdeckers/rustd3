//! The `Plot` widget driven by replayed gestures in windows: one plan of
//! scenarios (hover, pans, zooms, resizes) by workload by scale factor, a
//! cold process per run, on the native backend or the software renderer
//! (`--help` for the plans and flags).
//!
//! ```text
//! cargo run --release --example bench_gestures -- --plan quick --refresh-hz 60 --native-factor 1
//! ```

use std::process::ExitCode;

fn main() -> ExitCode {
    benchkit::run(benchkit::Example::Gestures)
}
