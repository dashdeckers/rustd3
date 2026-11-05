//! rustd3's benchmark of the plotting library and its `Plot` widget, at one
//! level. It builds and runs `bench_figure`, `bench_software` and
//! `bench_gestures` as child processes, then packs one archive to attach to
//! an issue (`--help` for the levels and flags).
//!
//! ```text
//! cargo run --release --example bench_all -- --quick
//! ```

use std::process::ExitCode;

fn main() -> ExitCode {
    benchkit::run(benchkit::Example::All)
}
