//! rustd3's benchmark: a level of the plotting library and its `Plot`
//! widget, which builds and runs `bench_figure`, `bench_software` and
//! `bench_gestures` as child processes and packs one archive to attach to
//! an issue (`--help` for the levels and flags).
//!
//! ```text
//! cargo run --release --example bench_all -- --quick
//! ```

use std::process::ExitCode;

fn main() -> ExitCode {
    benchkit::run(benchkit::Example::All)
}
