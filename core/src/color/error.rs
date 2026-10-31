//! [`ParseColorError`]: a string that is not a CSS color.

/// A string that is not a CSS color d3 accepts, where d3's `color(s)`
/// returns `null`; it carries the input as given.
///
/// ```
/// use rustd3_core::color::Rgb;
///
/// let error = "#zzz".parse::<Rgb>().unwrap_err();
/// assert_eq!(error.get_input(), "#zzz");
/// assert_eq!(error.to_string(), r##"not a CSS color: "#zzz""##);
/// ```
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("not a CSS color: {0:?}")]
pub struct ParseColorError(String);

impl ParseColorError {
    /// The error for `input`, as given.
    pub(super) fn new(input: &str) -> Self {
        ParseColorError(input.to_owned())
    }

    /// The string that did not parse, untrimmed.
    #[must_use]
    pub fn get_input(&self) -> &str {
        &self.0
    }
}
