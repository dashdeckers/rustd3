//! [`Time`]'s padding: the continuous scales' [`Pad`] in `i128` nanoseconds, with
//! matplotlib's date rule for flat extents.

use jiff::Timestamp;

use crate::scale::Pad;
use crate::time::{self, Time};

/// One hour, in nanoseconds.
const HOUR: i128 = 3_600 * 1_000_000_000;
/// One day, in nanoseconds.
const DAY: i128 = 24 * HOUR;
/// No data: matplotlib's date axis without data, 1970-01-01 to 1970-01-02
/// UTC, in nanoseconds from the epoch.
const EMPTY: [i128; 2] = [0, DAY];
/// How far a flat extent widens per side: an hour, so one sample's axis
/// reads in hours and minutes (matplotlib's `DateLocator.nonsingular`
/// widens by two years).
const WIDEN: i128 = HOUR;
/// How far inside the extent a sticky value still stops the margin,
/// relative to the span, as in the continuous scales' `Pad`.
const STICKY_TOLERANCE: f64 = 1e-5;

/// matplotlib's autoscaling of a date axis: no data gives 1970-01-01 to
/// 1970-01-02 UTC; equal ends widen by an hour per side (matplotlib
/// widens anything under 10⁻⁶ days, which would flatten every recording
/// shorter than 0.1 s, and by two years, which hides a sensor's one
/// sample in a decade); each side then gets `margin` of the span, rounded
/// to the nanosecond, except where a sticky instant stops it. The result
/// keeps the domain's orientation and saturates at jiff's range.
///
/// ```
/// use rustd3_core::jiff::Timestamp;
/// use rustd3_core::scale::Pad;
/// use rustd3_core::time;
///
/// let at = |s: &str| s.parse::<Timestamp>();
/// let ten_days = [at("2026-01-01T00:00:00Z")?, at("2026-01-11T00:00:00Z")?];
/// let padded = time::utc().pad(Some(ten_days), 0.05, &[]);
/// assert_eq!(padded, [at("2025-12-31T12:00:00Z")?, at("2026-01-11T12:00:00Z")?]);
/// let noon = at("2026-10-25T12:00:00Z")?;
/// let flat = time::utc().pad(Some([noon, noon]), 0.05, &[]);
/// assert_eq!(flat, [at("2026-10-25T10:54:00Z")?, at("2026-10-25T13:06:00Z")?]);
/// # Ok::<(), rustd3_core::jiff::Error>(())
/// ```
impl Pad for Time {
    type Domain = Timestamp;

    fn pad(
        &self,
        extent: Option<[Timestamp; 2]>,
        margin: f64,
        sticky: &[Timestamp],
    ) -> [Timestamp; 2] {
        let [low, high] = fit(extent, margin, sticky).map(time::saturating);
        if self.span() < 0 {
            [high, low]
        } else {
            [low, high]
        }
    }
}

impl crate::sealed::Pad for Time {}

/// matplotlib's `handle_single_axis` for dates, ascending, in nanoseconds
/// from the epoch.
fn fit(extent: Option<[Timestamp; 2]>, margin: f64, sticky: &[Timestamp]) -> [i128; 2] {
    let Some(extent) = extent else {
        return EMPTY;
    };
    let [x0, x1] = ascending(extent.map(Timestamp::as_nanosecond));
    let [x0, x1] = if x0 == x1 {
        [x0 - WIDEN, x1 + WIDEN]
    } else {
        [x0, x1]
    };
    let span = x1 - x0;
    // Float to integer casts saturate, and NaN casts to 0.
    let tolerance = (STICKY_TOLERANCE * span as f64).round() as i128;
    let delta = (margin * span as f64).round() as i128;
    let sticky = sticky.iter().map(|s| s.as_nanosecond());
    // matplotlib's searchsorted: the largest sticky value below x0 + tol and
    // the smallest at or above x1 − tol.
    let low = sticky.clone().filter(|&s| s < x0 + tolerance);
    let high = sticky.filter(|&s| s >= x1 - tolerance);
    let [x0, x1] = ascending([x0.saturating_sub(delta), x1.saturating_add(delta)]);
    [low.fold(x0, i128::max), high.fold(x1, i128::min)]
}

/// The pair in ascending order.
fn ascending([a, b]: [i128; 2]) -> [i128; 2] {
    if b < a { [b, a] } else { [a, b] }
}
