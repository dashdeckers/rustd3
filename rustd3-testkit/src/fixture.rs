//! [`Fixture`]: a reference file's JSON, read by pointer into [`Obs`]
//! values, for crates whose tests do not depend on serde_json themselves.

use crate::obs::{self, Obs};

/// A fixture's JSON, parsed once; its values are read by JSON pointer.
#[derive(Debug, Clone, PartialEq)]
pub struct Fixture(serde_json::Value);

/// Why a fixture or one of its values could not be read.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    /// The text is not JSON; carries serde_json's message.
    #[error("a fixture that is not JSON: {0}")]
    Parse(String),
    /// No value at the pointer; carries the pointer.
    #[error("no value at {0} in the fixture")]
    Missing(String),
    /// The value at the pointer has no `Obs` form (an object).
    #[error(transparent)]
    Obs(#[from] obs::Error),
}

impl Fixture {
    /// The fixture whose JSON text is `json`.
    ///
    /// ```
    /// use rustd3_testkit::fixture::Fixture;
    /// use rustd3_testkit::obs::Obs;
    ///
    /// let fixture = Fixture::parse(r#"{"domains": {"count": [0, 90.3]}}"#)?;
    /// let count = fixture.get("/domains/count")?;
    /// assert_eq!(count, Obs::List(vec![Obs::Num(0.0), Obs::Num(90.3)]));
    /// # Ok::<(), rustd3_testkit::fixture::Error>(())
    /// ```
    pub fn parse(json: &str) -> Result<Fixture, Error> {
        serde_json::from_str(json)
            .map(Fixture)
            .map_err(|e| Error::Parse(e.to_string()))
    }

    /// The value at `pointer` (RFC 6901: `/domains/vib`, `/trends/0`), as
    /// an [`Obs`].
    pub fn get(&self, pointer: &str) -> Result<Obs, Error> {
        let value = self
            .0
            .pointer(pointer)
            .ok_or_else(|| Error::Missing(pointer.to_owned()))?;
        Ok(Obs::try_from(value)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_are_read_by_pointer_and_misses_name_it() {
        let fixture = Fixture::parse(r#"{"a": {"b": [1, 2]}, "c": {"d": 3}}"#).unwrap();
        assert_eq!(fixture.get("/a/b/1"), Ok(Obs::Num(2.0)));
        assert_eq!(fixture.get("/a/x"), Err(Error::Missing("/a/x".to_owned())));
        assert!(matches!(fixture.get("/c"), Err(Error::Obs(_))));
        assert!(matches!(Fixture::parse("{"), Err(Error::Parse(_))));
    }
}
