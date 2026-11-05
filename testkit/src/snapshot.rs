//! Text snapshots compared byte for byte.
//!
//! References are text only (list dumps, SVG); raster checks compare the
//! workspace's own renderers at test time instead of stored images.

use std::path::{Path, PathBuf};
use std::{fs, io};

/// Text references in one directory.
#[derive(Debug, Clone)]
pub struct Store {
    dir: PathBuf,
    bless: bool,
}

impl Store {
    /// References under `dir` (callers pass
    /// `concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots")`).
    /// Reads `RUSTD3_BLESS` once: only the value `1` blesses.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        let bless = std::env::var_os("RUSTD3_BLESS").is_some_and(|v| v == "1");
        Self::at(dir.into(), bless)
    }

    /// Panics unless `dir/name` equals `actual` byte for byte. Only when
    /// blessing is a missing or different reference written, and the call
    /// passes. Otherwise the panic names the path, the first differing line
    /// (1-based, both versions) and both line counts, and writes `actual` to
    /// `dir/name.new` for inspection.
    ///
    /// `name` matches `[A-Za-z0-9_][A-Za-z0-9_.-]*` and may carry an
    /// extension (`trace.svg`); any other name panics.
    pub fn check(&self, name: &str, actual: &str) {
        assert!(
            valid(name),
            "snapshot name {name:?} is not [A-Za-z0-9_][A-Za-z0-9_.-]*"
        );
        let path = self.dir.join(name);
        let reference = match fs::read(&path) {
            Ok(bytes) => Some(bytes),
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Err(e) => panic!("cannot read snapshot {}: {e}", path.display()),
        };
        if reference.as_deref() == Some(actual.as_bytes()) {
            return;
        }
        if self.bless {
            write(&path, actual);
            return;
        }
        let new = self.dir.join(format!("{name}.new"));
        write(&new, actual);
        match reference {
            None => panic!(
                "missing snapshot {}; wrote {} (rerun with RUSTD3_BLESS=1 to accept it)",
                path.display(),
                new.display()
            ),
            Some(bytes) => panic!(
                "snapshot {} differs; wrote {}\n{}",
                path.display(),
                new.display(),
                first_difference(&String::from_utf8_lossy(&bytes), actual)
            ),
        }
    }

    fn at(dir: PathBuf, bless: bool) -> Self {
        Self { dir, bless }
    }
}

fn valid(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
        && bytes.all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
}

fn write(path: &Path, text: &str) {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
    }
    fs::write(path, text).unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
}

/// The first differing line, 1-based, in both versions, and both line
/// counts. Lines keep their terminators, so a missing final newline shows.
fn first_difference(reference: &str, actual: &str) -> String {
    let (r, a): (Vec<_>, Vec<_>) = (
        reference.split_inclusive('\n').collect(),
        actual.split_inclusive('\n').collect(),
    );
    let line = r.iter().zip(&a).take_while(|(x, y)| x == y).count();
    let shown = |lines: &[&str]| {
        lines
            .get(line)
            .map_or("<none>".to_owned(), |l| format!("{l:?}"))
    };
    format!(
        "first difference at line {}: reference {}, actual {}; the reference has {} lines, actual {}",
        line + 1,
        shown(&r),
        shown(&a),
        r.len(),
        a.len()
    )
}

#[cfg(test)]
mod tests {
    use std::panic;

    use super::*;

    /// A fresh directory under the system's temporary directory, named after the test.
    fn scratch(test: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("rustd3-testkit").join(test);
        match fs::remove_dir_all(&dir) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => panic!("cannot clear {}: {e}", dir.display()),
        }
        dir
    }

    fn message(result: std::thread::Result<()>) -> String {
        let payload = result.expect_err("check must panic");
        payload
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_default()
    }

    #[test]
    fn a_missing_reference_panics_and_writes_the_new_file() {
        let dir = scratch("a_missing_reference_panics_and_writes_the_new_file");
        let store = Store::at(dir.clone(), false);
        let msg = message(panic::catch_unwind(|| store.check("trace.svg", "<svg/>\n")));
        assert!(
            msg.contains(&dir.join("trace.svg").display().to_string()),
            "{msg}"
        );
        assert!(msg.contains("RUSTD3_BLESS=1"), "{msg}");
        assert_eq!(
            fs::read_to_string(dir.join("trace.svg.new")).unwrap(),
            "<svg/>\n"
        );
        assert!(!dir.join("trace.svg").exists());
    }

    #[test]
    fn blessing_writes_a_missing_reference_and_passes() {
        let dir = scratch("blessing_writes_a_missing_reference_and_passes");
        Store::at(dir.clone(), true).check("list.txt", "a\nb\n");
        assert_eq!(fs::read_to_string(dir.join("list.txt")).unwrap(), "a\nb\n");
        Store::at(dir, false).check("list.txt", "a\nb\n");
    }

    #[test]
    fn a_different_reference_names_the_first_differing_line() {
        let dir = scratch("a_different_reference_names_the_first_differing_line");
        Store::at(dir.clone(), true).check("dump", "one\ntwo\nthree\n");
        let store = Store::at(dir.clone(), false);
        let msg = message(panic::catch_unwind(|| {
            store.check("dump", "one\n2\nthree\nfour\n")
        }));
        assert!(
            msg.ends_with(
                "first difference at line 2: reference \"two\\n\", actual \"2\\n\"; \
                 the reference has 3 lines, actual 4"
            ),
            "{msg}"
        );
        assert_eq!(
            fs::read_to_string(dir.join("dump.new")).unwrap(),
            "one\n2\nthree\nfour\n"
        );
        assert_eq!(
            fs::read_to_string(dir.join("dump")).unwrap(),
            "one\ntwo\nthree\n"
        );
    }

    #[test]
    fn a_missing_final_newline_is_a_difference() {
        assert_eq!(
            first_difference("a\n", "a"),
            "first difference at line 1: reference \"a\\n\", actual \"a\"; \
             the reference has 1 lines, actual 1"
        );
        assert_eq!(
            first_difference("a\n", "a\nb"),
            "first difference at line 2: reference <none>, actual \"b\"; \
             the reference has 1 lines, actual 2"
        );
    }

    #[test]
    fn blessing_replaces_a_different_reference() {
        let dir = scratch("blessing_replaces_a_different_reference");
        Store::at(dir.clone(), true).check("v", "1\n");
        Store::at(dir.clone(), true).check("v", "2\n");
        assert_eq!(fs::read_to_string(dir.join("v")).unwrap(), "2\n");
    }

    #[test]
    fn names_outside_the_pattern_panic() {
        let store = Store::at(scratch("names_outside_the_pattern_panic"), true);
        for name in ["", ".hidden", "-x", "a/b", "../up", "a b"] {
            let msg = message(panic::catch_unwind(|| store.check(name, "")));
            assert!(
                msg.contains("is not [A-Za-z0-9_][A-Za-z0-9_.-]*"),
                "{name}: {msg}"
            );
        }
        store.check("_a.b-c", "");
    }

    #[test]
    fn the_environment_variable_blesses_only_when_it_is_1() {
        let expected = std::env::var_os("RUSTD3_BLESS").is_some_and(|v| v == "1");
        assert_eq!(
            Store::new(scratch("the_environment_variable")).bless,
            expected
        );
    }
}
