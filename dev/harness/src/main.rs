//! The grading harness: d3's own tests, translated offline into Rust
//! assertions by `translate/translate.mjs`, run against rustd3-core, with
//! a ratchet of passing assertions and a floor per test file in
//! `passing.txt`.
//!
//! ```text
//! rustd3-harness                         report per suite and file, the first 20 failures
//! rustd3-harness --failing [id prefix]   the report with every failure (under the prefix)
//! rustd3-harness --check                 fail on any regression against passing.txt
//! rustd3-harness --bless [--allow-drop] [--bless-floors]
//! ```
//!
//! Exit codes: 0 ok, 1 a regression or an internal error, 2 usage.

#[rustfmt::skip]
mod generated;
mod prelude;
mod ratchet;
mod record;
mod registry;
mod run;
mod stub;

use std::path::Path;
use std::process::ExitCode;

use ratchet::{Ratchet, Regression};
use run::{Outcome, Run};

const PASSING: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/passing.txt");
const USAGE: &str = "usage: rustd3-harness [--failing [id prefix]] | --check | \
                     --bless [--allow-drop] [--bless-floors]";

/// What the command line asks for.
enum Mode {
    /// The report; every failure whose id starts with the prefix, if given.
    Report { failing: Option<String> },
    /// The ratchet's verdict.
    Check,
    /// Rewrite `passing.txt`.
    Bless { allow_drop: bool, floors: bool },
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(mode) = Mode::parse(&args) else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let run = match Run::suites(generated::SUITES) {
        Ok(run) => run,
        Err(faults) => {
            for fault in faults {
                eprintln!("internal error: {fault}");
            }
            return ExitCode::FAILURE;
        }
    };
    let path = Path::new(PASSING);
    let result = match mode {
        Mode::Report { failing } => {
            run.report(failing.as_deref());
            Ok(ExitCode::SUCCESS)
        }
        Mode::Check => check(&run, path),
        Mode::Bless { allow_drop, floors } => bless(&run, path, allow_drop, floors),
    };
    result.unwrap_or_else(|e| {
        eprintln!("{}: {e}", path.display());
        ExitCode::FAILURE
    })
}

impl Mode {
    fn parse(args: &[String]) -> Option<Mode> {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        match args[..] {
            [] => Some(Mode::Report { failing: None }),
            ["--failing"] => Some(Mode::Report {
                failing: Some(String::new()),
            }),
            ["--failing", prefix] if !prefix.starts_with("--") => Some(Mode::Report {
                failing: Some(prefix.to_owned()),
            }),
            ["--check"] => Some(Mode::Check),
            ["--bless", ref flags @ ..] => {
                let allow_drop = flags.contains(&"--allow-drop");
                let floors = flags.contains(&"--bless-floors");
                let known = usize::from(allow_drop) + usize::from(floors);
                (known == flags.len()).then_some(Mode::Bless { allow_drop, floors })
            }
            _ => None,
        }
    }
}

/// Prints each file's counts and every regression; 1 if there is one.
fn check(run: &Run, path: &Path) -> std::io::Result<ExitCode> {
    let verdict = Ratchet::load(path)?.check(run.outcomes());
    for line in run.file_lines() {
        println!("{line}");
    }
    for regression in &verdict.regressions {
        println!("regression: {regression}{}", detail(run, regression));
    }
    if !verdict.new.is_empty() {
        println!(
            "{} newly passing: read the diff, then --bless",
            verdict.new.len()
        );
        for id in &verdict.new {
            println!("  {id}");
        }
    }
    if verdict.regressions.is_empty() {
        println!("ratchet holds");
        Ok(ExitCode::SUCCESS)
    } else {
        println!("{} regressions", verdict.regressions.len());
        Ok(ExitCode::FAILURE)
    }
}

/// Rewrites `passing.txt`, refusing to drop a listed id without `allow_drop`.
fn bless(run: &Run, path: &Path, allow_drop: bool, floors: bool) -> std::io::Result<ExitCode> {
    let ratchet = match Ratchet::load(path) {
        Ok(ratchet) => ratchet,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ratchet::default(),
        Err(e) => return Err(e),
    };
    let verdict = ratchet.check(run.outcomes());
    let dropped: Vec<_> = verdict
        .regressions
        .iter()
        .filter(|r| r.drops_an_id())
        .collect();
    if !dropped.is_empty() && !allow_drop {
        println!(
            "refusing to drop {} listed ids without --allow-drop:",
            dropped.len()
        );
        for regression in dropped {
            println!("  {regression}{}", detail(run, regression));
        }
        return Ok(ExitCode::FAILURE);
    }
    let change = Ratchet::bless(path, run.outcomes(), floors)?;
    for id in &change.added {
        println!("added {id}");
    }
    for id in &change.removed {
        println!("removed {id}");
    }
    for (file, floor) in &change.floors {
        println!("floor {file} {floor}");
    }
    println!(
        "{}: {} added, {} removed, {} floors set",
        path.display(),
        change.added.len(),
        change.removed.len(),
        change.floors.len()
    );
    Ok(ExitCode::SUCCESS)
}

/// The values of a failing id, or the panic that kept an id from being reached.
fn detail(run: &Run, regression: &Regression) -> String {
    match regression {
        Regression::Fails(id) => match run.outcomes().get(id.as_str()) {
            Some(Outcome::Fail { actual, expected }) => {
                format!(": actual {actual:?}, expected {expected:?}")
            }
            Some(Outcome::Pass | Outcome::Skip(_) | Outcome::NotReached) | None => String::new(),
        },
        Regression::NotReached(id) => run.panic_of(id).map_or_else(String::new, |message| {
            format!(" (its test panicked: {message})")
        }),
        Regression::Skipped(..) | Regression::Missing(_) | Regression::Floor { .. } => {
            String::new()
        }
    }
}
