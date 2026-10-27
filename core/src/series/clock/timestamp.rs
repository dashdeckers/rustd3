//! Recordings of instants: [`Coordinate`] for jiff's [`Timestamp`], in
//! `i128` nanoseconds and never through `f64`.

use jiff::{SignedDuration, Timestamp};

use crate::series::Coordinate;
use crate::time;

/// Each sample is computed from the clock's start, without accumulation.
/// A regular clock's x that would leave jiff's range saturates at
/// [`Timestamp::MAX`], which is therefore not a valid x, as an `f64`
/// overflows to the infinity a recording rejects.
///
/// ```
/// use rustd3_core::jiff::{SignedDuration, Timestamp};
/// use rustd3_core::series::{Clock, Recording};
///
/// let start: Timestamp = "2026-10-25T00:00:00Z".parse()?;
/// let period = SignedDuration::from_millis(1);
/// let rec = Recording::new(Clock::Regular { start, period }, [vec![0.0; 1_000]])?;
/// assert_eq!(rec.x(999), Some("2026-10-25T00:00:00.999Z".parse()?));
/// assert_eq!(rec.locate("2026-10-25T00:00:00.0005Z".parse()?), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
impl Coordinate for Timestamp {
    type Period = SignedDuration;

    /// `start + index · period` in `i128` nanoseconds; saturates at jiff's
    /// first and last instants.
    fn at(start: Timestamp, period: SignedDuration, index: u64) -> Timestamp {
        let offset = i128::from(index).saturating_mul(period.as_nanos());
        time::saturating(start.as_nanosecond().saturating_add(offset))
    }

    /// `⌈(value − start) / period⌉` in `i128`, clamped to `0..=u64::MAX`:
    /// the exact first index whose `at` is ≥ `value`.
    fn index(start: Timestamp, period: SignedDuration, value: Timestamp) -> u64 {
        let period = period.as_nanos();
        if period <= 0 {
            return 0;
        }
        let after = value.as_nanosecond() - start.as_nanosecond();
        let index = after.div_euclid(period) + i128::from(after.rem_euclid(period) != 0);
        u64::try_from(index.max(0)).unwrap_or(u64::MAX)
    }

    /// Every instant but jiff's last, where `at` saturates.
    fn is_valid(self) -> bool {
        self < Timestamp::MAX
    }

    /// Positive: at least 1 ns.
    fn is_valid_period(period: SignedDuration) -> bool {
        period.is_positive()
    }
}
