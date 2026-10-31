//! d3's isoFormat and isoParse: [`iso_format`] and [`iso_parse`].

use jiff::Timestamp;
use jiff::tz::TimeZone;

use crate::time::Locale;

/// d3's `isoSpecifier` (`isoFormat.js:3`).
const ISO: &str = "%Y-%m-%dT%H:%M:%S.%LZ";

/// d3's `isoFormat`: `t` as `%Y-%m-%dT%H:%M:%S.%LZ` in UTC, to the
/// millisecond (floored).
///
/// ```
/// use rustd3_core::jiff::Timestamp;
/// use rustd3_core::time;
///
/// let t: Timestamp = "2026-03-08T07:59:30.123456Z".parse()?;
/// assert_eq!(time::iso_format(t), "2026-03-08T07:59:30.123Z");
/// # Ok::<(), rustd3_core::jiff::Error>(())
/// ```
#[must_use]
pub fn iso_format(t: Timestamp) -> String {
    Locale::NEUTRAL.format(ISO).format(t, &TimeZone::UTC)
}

/// d3's `isoParse`: an RFC 3339 instant, through jiff's parser (an offset
/// or `Z` is required); its error where d3 returns null.
///
/// ```
/// use rustd3_core::time;
///
/// let t = time::iso_parse("2026-03-07T23:59:30.123-08:00")?;
/// assert_eq!(time::iso_format(t), "2026-03-08T07:59:30.123Z");
/// assert!(time::iso_parse("1990-01-01T00:00:00.000X").is_err());
/// # Ok::<(), rustd3_core::jiff::Error>(())
/// ```
pub fn iso_parse(s: &str) -> Result<Timestamp, jiff::Error> {
    s.parse()
}
