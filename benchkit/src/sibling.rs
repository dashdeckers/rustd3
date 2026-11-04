//! [`Siblings`]: the examples `bench_all` runs as child processes, found
//! beside its own binary. Under `cargo run` (which sets `CARGO`) it first
//! builds them with the same profile, so that one `cargo run --release
//! --example bench_all` works from a fresh checkout and never runs a
//! stale sibling.

use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::Example;

/// The workspace's manifest, which builds the examples.
const MANIFEST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../Cargo.toml");

/// The binaries of the examples a level runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Siblings {
    /// `bench_figure`: the figure profile's children.
    pub figure: PathBuf,
    /// `bench_gestures`: the display measurement and the windowed
    /// profiles.
    pub gestures: PathBuf,
    /// `bench_software`: the software profile.
    pub software: PathBuf,
}

impl Siblings {
    /// The examples beside this process's binary, built first when cargo
    /// runs it.
    pub fn here() -> Result<Siblings, Box<dyn Error>> {
        let exe = env::current_exe()?;
        let dir = exe.parent().ok_or("the binary has no directory")?;
        let siblings = Siblings::beside(dir);
        if let Some(cargo) = env::var_os("CARGO") {
            build(Path::new(&cargo), dir)?;
        }
        for path in [&siblings.figure, &siblings.gestures, &siblings.software] {
            if !path.is_file() {
                return Err(format!(
                    "{} is not built: run `cargo build --release --examples` first, or start \
                     bench_all with `cargo run --release --example bench_all`",
                    path.display()
                )
                .into());
            }
        }
        Ok(siblings)
    }

    /// The examples' binaries in `dir`.
    fn beside(dir: &Path) -> Siblings {
        let path =
            |example: Example| dir.join(format!("{}{}", example.name(), env::consts::EXE_SUFFIX));
        Siblings {
            figure: path(Example::Figure),
            gestures: path(Example::Gestures),
            software: path(Example::Software),
        }
    }
}

/// Builds the sibling examples with `cargo` in the profile whose examples
/// directory is `dir` (`target/<profile>/examples`).
fn build(cargo: &Path, dir: &Path) -> Result<(), Box<dyn Error>> {
    let profile = dir
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .unwrap_or("release");
    // The whole workspace's features, as `cargo run --example bench_all`
    // from the root resolves them, so that the two builds share their
    // artifacts.
    let mut command = Command::new(cargo);
    command.args(["build", "--manifest-path", MANIFEST, "--workspace"]);
    command.args(profile_args(profile));
    for example in [Example::Figure, Example::Gestures, Example::Software] {
        command.args(["--example", example.name()]);
    }
    println!("Building bench_figure, bench_gestures and bench_software ({profile}).");
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("building the examples failed ({status})").into())
    }
}

/// Cargo's flags for the profile whose directory is `name`: `debug` is
/// `dev`'s.
fn profile_args(name: &str) -> Vec<String> {
    match name {
        "debug" => Vec::new(),
        "release" => vec!["--release".to_owned()],
        other => vec!["--profile".to_owned(), other.to_owned()],
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{Siblings, profile_args};

    #[test]
    fn siblings_sit_beside_the_binary_and_build_in_its_profile() {
        let siblings = Siblings::beside(Path::new("target/release/examples"));
        let suffix = std::env::consts::EXE_SUFFIX;
        assert_eq!(
            siblings.software,
            Path::new("target/release/examples").join(format!("bench_software{suffix}"))
        );
        assert!(profile_args("debug").is_empty());
        assert_eq!(profile_args("release"), ["--release"]);
        assert_eq!(profile_args("bench"), ["--profile", "bench"]);
    }
}
