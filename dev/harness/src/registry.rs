//! The tables `translate.mjs` generates: suites, their tests and their
//! assertions.

/// The translated test files of one suite: one generated module.
#[derive(Debug)]
pub struct Suite {
    /// The suite's name, which is its generated module's.
    pub name: &'static str,
    /// Its d3 tests, in file and source order.
    pub tests: &'static [Test],
    /// Every assertion of those tests, translated or skipped.
    pub asserts: &'static [Assertion],
}

/// One d3 test (`it(…)`), translated into a function.
#[derive(Debug)]
pub struct Test {
    /// `<file>: <d3's test name>`.
    pub name: &'static str,
    /// Runs the test's checks; it may panic.
    pub run: fn(),
}

/// One assertion of a d3 test.
#[derive(Debug)]
pub struct Assertion {
    /// `<package>/test/<file>:<line>:<column>`, relative to the vendored d3 root.
    pub id: &'static str,
    /// The name of the [`Test`] it belongs to.
    pub test: &'static str,
    /// Why the translator skipped it, if it did.
    pub skip: Option<&'static str>,
}

/// An id's test file, line and column:
/// `d3-ease/test/linear-test.js:6:3` is (`d3-ease/test/linear-test.js`, 6, 3).
/// An id without them, which the translator never writes, is all file, at 0:0.
pub fn location(id: &str) -> (&str, u32, u32) {
    let mut parts = id.rsplitn(3, ':');
    let column = parts.next().and_then(|c| c.parse().ok());
    let line = parts.next().and_then(|l| l.parse().ok());
    match (parts.next(), line, column) {
        (Some(file), Some(line), Some(column)) => (file, line, column),
        _ => (id, 0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_id_splits_into_file_line_and_column() {
        assert_eq!(
            location("d3-ease/test/linear-test.js:16:3"),
            ("d3-ease/test/linear-test.js", 16, 3)
        );
    }

    #[test]
    fn an_id_without_a_location_is_all_file() {
        assert_eq!(location("d3-ease/test/x.js"), ("d3-ease/test/x.js", 0, 0));
        assert_eq!(location("a:b:c"), ("a:b:c", 0, 0));
    }
}
