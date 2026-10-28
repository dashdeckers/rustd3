//! The `label-churn` workload: `trace` on a time axis. The same three
//! channels run on a 10 kHz clock of instants whose span is centred on
//! midnight UTC, and x is a UTC time handle labelled by its own ticks in
//! the figure's locale (the neutral ISO ladder: seconds or sub-seconds,
//! minutes, and the date at the day boundary), so pans across midnight and
//! zooms bring label strings the text cache has not seen. Phase 22's
//! stand-in labelled a linear x `t+{v:.4} s` at 16 ticks.

use rustd3_core::jiff::{SignedDuration, Timestamp};
use rustd3_core::series::Clock;
use rustd3_core::time;
use rustd3_figure::Figure;
use rustd3_figure::scale::Fit;

use crate::workload::trace::{RATE, Trace};

/// The midnight the data's span is centred on: 2026-10-28T00:00:00Z.
const MIDNIGHT: Timestamp = Timestamp::constant(1_793_145_600, 0);

impl Trace<Timestamp> {
    /// `n` samples per channel at 10 kHz, half of them before
    /// [`MIDNIGHT`], seeds 1, 2 and 3.
    pub fn around_midnight(n: usize) -> Trace<Timestamp> {
        let period = SignedDuration::from_nanos((1e9 / RATE) as i64);
        let half = SignedDuration::from_nanos(period.as_nanos() as i64 * (n / 2) as i64);
        let start = MIDNIGHT
            .checked_sub(half)
            .expect("invariant: a day of samples before 2026 is within jiff's range");
        Trace::on(Clock::Regular { start, period }, n)
    }

    /// The figure: x a UTC time handle, on `Fit::Window` of 10 s when
    /// `windowed`.
    pub fn figure(&self, windowed: bool) -> Figure<'_> {
        let mut fig = Figure::new();
        let time = if windowed {
            fig.fit_with(time::utc(), Fit::Window(SignedDuration::from_secs(10)))
        } else {
            fig.fit(time::utc())
        };
        self.panels(&mut fig, time, "time (UTC)");
        fig
    }
}
