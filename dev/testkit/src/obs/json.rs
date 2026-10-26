//! Fixture values read from JSON.

use super::Obs;

/// A JSON value with no [`Obs`] form.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    /// An object; fixture tests navigate objects with serde_json and
    /// convert the leaves. Carries the object's JSON text.
    #[error("a JSON object has no Obs form: {0}")]
    Object(String),
}

impl TryFrom<&serde_json::Value> for Obs {
    type Error = Error;

    fn try_from(value: &serde_json::Value) -> Result<Self, Error> {
        match value {
            serde_json::Value::Null => Ok(Obs::Null),
            serde_json::Value::Bool(b) => Ok(Obs::Bool(*b)),
            serde_json::Value::Number(n) => Ok(Obs::Num(n.as_f64().expect(
                "invariant: without serde_json's arbitrary_precision every number has an f64",
            ))),
            serde_json::Value::String(s) => Ok(Obs::Str(s.clone())),
            serde_json::Value::Array(items) => items
                .iter()
                .map(Obs::try_from)
                .collect::<Result<_, _>>()
                .map(Obs::List),
            serde_json::Value::Object(_) => Err(Error::Object(value.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalars_and_arrays_convert() {
        let value: serde_json::Value =
            serde_json::from_str(r#"[null, true, -1.5, "x", [2]]"#).unwrap();
        assert_eq!(
            Obs::try_from(&value),
            Ok(Obs::List(vec![
                Obs::Null,
                Obs::Bool(true),
                Obs::Num(-1.5),
                Obs::Str("x".to_owned()),
                Obs::List(vec![Obs::Num(2.0)]),
            ]))
        );
    }

    #[test]
    fn objects_are_rejected_with_their_text() {
        let value: serde_json::Value = serde_json::from_str(r#"{"a": 1}"#).unwrap();
        assert_eq!(
            Obs::try_from(&value),
            Err(Error::Object("{\"a\":1}".to_owned()))
        );
    }

    #[test]
    fn an_object_inside_an_array_rejects_the_array() {
        let value: serde_json::Value = serde_json::from_str(r#"[1, {"b": null}]"#).unwrap();
        assert_eq!(
            Obs::try_from(&value),
            Err(Error::Object("{\"b\":null}".to_owned()))
        );
    }
}
