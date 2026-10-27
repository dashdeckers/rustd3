//! One channel of a recording, borrowed.

use std::ops::Range;

use crate::series::channel::Channel;
use crate::series::{Columns, Coordinate, Error, Recording, m4};

/// One channel of a recording, borrowed; `Copy`, passed by value from `view`.
///
/// ```
/// use rustd3_core::series::{Clock, Recording};
///
/// let y = [0.0, 5.0, -1.0, 3.0, f64::NAN, f64::NAN, 2.0, 8.0, 1.0, 4.0];
/// let rec = Recording::new(Clock::Regular { start: 0.0, period: 1.0 }, [y])?;
/// let s = rec.series(0).expect("channel 0");
/// // Sample index per column edge, from data-space edges (the x scale's
/// // invert at pixel edges).
/// let columns = rec.columns(&[1.5, 5.0, 8.5])?;
/// assert_eq!(columns.edges(), [2, 5, 9]);
/// // Admit what the y scale can draw (log: y > 0).
/// let lines = s.m4(&columns, |y| y > 0.0)?;
/// // Sample indices of each NaN-free run.
/// assert_eq!(lines.iter().collect::<Vec<_>>(), [&[1, 2, 3][..], &[6, 7, 8, 9]]);
/// # Ok::<(), rustd3_core::series::Error>(())
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Series<'a, X: Coordinate> {
    recording: &'a Recording<X>,
    channel: usize,
}

impl<'a, X: Coordinate> Series<'a, X> {
    /// Channel `channel` of `recording`, which must exist.
    pub(crate) fn new(recording: &'a Recording<X>, channel: usize) -> Self {
        Self { recording, channel }
    }

    /// The recording the channel belongs to.
    pub fn recording(self) -> &'a Recording<X> {
        self.recording
    }

    /// The channel's number in its recording.
    pub fn channel(self) -> usize {
        self.channel
    }

    /// The number of samples.
    pub fn len(self) -> usize {
        self.recording.len()
    }

    /// Whether the channel holds no samples.
    pub fn is_empty(self) -> bool {
        self.recording.is_empty()
    }

    /// Sample `i`'s x and value, if there is one.
    pub fn get(self, i: usize) -> Option<(X, f64)> {
        Some((self.recording.x(i)?, self.stored().get(i)?))
    }

    /// Every sample's x and value, in order.
    pub fn iter(self) -> impl Iterator<Item = (X, f64)> + 'a {
        (0..self.len()).map_while(move |i| self.get(i))
    }

    /// NaN-ignoring min and max of the whole channel, O(log n).
    pub fn extent(self) -> Option<[f64; 2]> {
        let extremes = self.extremes(0..self.len(), |_| true)?;
        Some([extremes.min().1, extremes.max().1])
    }

    /// The earliest minimum and maximum of `samples` (clamped to the
    /// channel) among the values that are not NaN and pass `valid`, or
    /// `None` when none does.
    pub fn extremes(
        self,
        samples: Range<usize>,
        valid: impl Fn(f64) -> bool,
    ) -> Option<m4::Extremes> {
        self.stored().extremes(self.clamp(samples), &valid)
    }

    /// The NaN-free runs of `samples` (clamped to the channel), in order.
    pub fn runs(self, samples: Range<usize>) -> impl Iterator<Item = Range<usize>> + 'a {
        let samples = self.clamp(samples);
        let runs = self.stored().runs().overlapping(samples.clone());
        runs.iter()
            .map(move |run| run.start.max(samples.start)..run.end.min(samples.end))
    }

    /// M4 per column and NaN-free run: one polyline per run that overlaps
    /// the columns, holding each column's first, minimum, maximum and last
    /// sample of the run (minimum and maximum among the values that pass
    /// `valid`), plus the samples just outside the columns when the run
    /// holds them. Columns built for another recording or revision are
    /// [`Error::Stale`].
    pub fn m4(self, columns: &Columns, valid: impl Fn(f64) -> bool) -> Result<m4::Lines, Error<X>> {
        let recording = self.recording;
        if columns.recording() != recording.id() || columns.revision() != recording.revision() {
            return Err(Error::Stale {
                recording: columns.recording(),
                built: columns.revision(),
                current: recording.revision(),
            });
        }
        let stored = self.stored();
        let runs = stored.runs().overlapping(columns.samples());
        Ok(m4::Lines::thin(runs, columns.edges(), |samples| {
            stored.extremes(samples, &valid)
        }))
    }

    fn stored(self) -> &'a Channel {
        self.recording.channel(self.channel)
    }

    fn clamp(self, samples: Range<usize>) -> Range<usize> {
        let end = samples.end.min(self.len());
        samples.start.min(end)..end
    }
}
