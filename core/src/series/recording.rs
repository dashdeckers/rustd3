//! Samples on one clock with any number of channels, appended atomically
//! and dropped from the front in whole chunks.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::series::channel::Channel;
use crate::series::chunks::Chunks;
use crate::series::{CHUNK, Clock, Columns, Coordinate, Error, Series};

/// The most samples a recording retains: the index's offsets are `u32`.
const MAX: u64 = u32::MAX as u64;

/// The next recording id.
static NEXT: AtomicU64 = AtomicU64::new(0);

/// Identifies a recording for caches: a fresh `AtomicU64` value per
/// recording, including clones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(u64);

/// Samples on one clock with any number of `f64` channels; NaN is a gap.
///
/// ```
/// use rustd3_core::series::{Clock, Recording};
///
/// let speed = vec![0.0; 150_000];
/// let torque = vec![1.0; 150_000];
/// // 10 Hz, two channels on one clock.
/// let mut rec = Recording::new(Clock::Regular { start: 0.0, period: 0.1 }, [&speed, &torque])?;
/// // Every channel or none; a regular clock takes no x values.
/// rec.extend(&[], [&speed[..50_000], &torque[..50_000]])?;
///
/// // With 200,000 samples, drop what lies before sample 150,000, in whole chunks:
/// assert_eq!(rec.drop_front(150_000), 131_072);
/// assert_eq!(rec.len(), 68_928);
/// assert_eq!(rec.x(0), Some(13107.2));
/// # Ok::<(), rustd3_core::series::Error>(())
/// ```
#[derive(Debug)]
pub struct Recording<X: Coordinate> {
    id: Id,
    revision: u64,
    offset: u64,
    len: usize,
    clock: Stored<X>,
    channels: Vec<Channel>,
}

/// A recording's clock as stored: explicit x in chunks aligned with the
/// channels' chunks.
#[derive(Debug, Clone)]
enum Stored<X: Coordinate> {
    Regular { start: X, period: X::Period },
    Explicit(Chunks<X>),
}

impl Id {
    fn fresh() -> Self {
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

/// A deep copy with a fresh [`Id`]; the revision and offset are kept.
impl<X: Coordinate> Clone for Recording<X> {
    fn clone(&self) -> Self {
        Self {
            id: Id::fresh(),
            revision: self.revision,
            offset: self.offset,
            len: self.len,
            clock: self.clock.clone(),
            channels: self.channels.clone(),
        }
    }
}

impl<X: Coordinate> Recording<X> {
    /// A recording of `channels` on `clock`, at revision 0. Every channel
    /// has as many samples as an explicit clock has x values (on a regular
    /// clock, as many as the first channel); zero channels are allowed.
    /// Nothing is stored unless everything is valid: the period and start
    /// of a regular clock, the lengths, explicit x valid and non-decreasing,
    /// and at most `u32::MAX` samples.
    pub fn new<C: AsRef<[f64]>>(
        clock: Clock<X>,
        channels: impl IntoIterator<Item = C>,
    ) -> Result<Self, Error<X>> {
        let (clock, x) = match clock {
            Clock::Regular { period, .. } if !X::is_valid_period(period) => {
                return Err(Error::Period(period));
            }
            Clock::Regular { start, .. } if !start.is_valid() => {
                return Err(Error::Invalid {
                    index: 0,
                    value: start,
                });
            }
            Clock::Regular { start, period } => (Stored::Regular { start, period }, Vec::new()),
            Clock::Explicit(x) => (Stored::Explicit(Chunks::default()), x),
        };
        let channels: Vec<C> = channels.into_iter().collect();
        let mut recording = Self {
            id: Id::fresh(),
            revision: 0,
            offset: 0,
            len: 0,
            clock,
            channels: channels.iter().map(|_| Channel::default()).collect(),
        };
        recording.append(&x, &channels)?;
        Ok(recording)
    }

    /// Appends to every channel at once, or to none. A regular clock takes
    /// `x = &[]`; an explicit one takes one x per new sample, valid and not
    /// below the last stored x. On an error the recording is unchanged.
    pub fn extend<C: AsRef<[f64]>>(
        &mut self,
        x: &[X],
        channels: impl IntoIterator<Item = C>,
    ) -> Result<(), Error<X>> {
        let channels: Vec<C> = channels.into_iter().collect();
        self.append(x, &channels)?;
        self.revision += 1;
        Ok(())
    }

    /// Drops the whole chunks that lie before sample `before`; returns
    /// how many samples went (a multiple of 65,536). Indices shift down.
    pub fn drop_front(&mut self, before: usize) -> usize {
        let chunks = before.min(self.len) / CHUNK;
        if chunks == 0 {
            return 0;
        }
        if let Stored::Explicit(x) = &mut self.clock {
            x.drop_front(chunks);
        }
        for channel in &mut self.channels {
            channel.drop_front(chunks);
        }
        let dropped = chunks * CHUNK;
        self.len -= dropped;
        self.offset += dropped as u64;
        self.revision += 1;
        dropped
    }

    /// A view of channel `channel`, or `None` if there is no such channel.
    pub fn series(&self, channel: usize) -> Option<Series<'_, X>> {
        (channel < self.channels.len()).then(|| Series::new(self, channel))
    }

    /// The number of samples, the same in every channel.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the recording holds no samples.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The number of channels.
    pub fn channels(&self) -> usize {
        self.channels.len()
    }

    /// The recording's identity for caches.
    pub fn id(&self) -> Id {
        self.id
    }

    /// Bumped by every successful `extend` and every `drop_front` that dropped.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Absolute number of the current first sample (samples ever dropped).
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// The x of sample `i`, if there is one.
    pub fn x(&self, i: usize) -> Option<X> {
        match &self.clock {
            Stored::Regular { start, period } => {
                (i < self.len).then(|| X::at(*start, *period, self.offset + i as u64))
            }
            Stored::Explicit(x) => x.get(i),
        }
    }

    /// The first sample index whose x is ≥ `value` (`partition_point`),
    /// in 0..=len.
    pub fn locate(&self, value: X) -> usize {
        match &self.clock {
            Stored::Regular { start, period } => {
                let below = |i: usize| X::at(*start, *period, self.offset + i as u64) < value;
                let estimate = X::index(*start, *period, value).saturating_sub(self.offset);
                let guess = usize::try_from(estimate).map_or(self.len, |i| i.min(self.len));
                gallop(guess, self.len, below)
            }
            Stored::Explicit(x) => x.partition_point(|x| *x < value),
        }
    }

    /// The first and last x, if there are samples.
    pub fn x_extent(&self) -> Option<[X; 2]> {
        Some([self.x(0)?, self.x(self.len.checked_sub(1)?)?])
    }

    /// Column boundaries in samples for data-space `edges` (≥ 2,
    /// non-decreasing): each edge maps to `locate(edge)`, so an edge below
    /// `x(0)` gives 0 and one above the last x gives `len`.
    pub fn columns(&self, edges: &[X]) -> Result<Columns, Error<X>> {
        if edges.len() < 2 {
            return Err(Error::Edges(edges.len()));
        }
        let mut previous = None;
        for (index, &value) in edges.iter().enumerate() {
            if !value.is_valid() || previous.is_some_and(|previous| previous > value) {
                return Err(Error::Edge { index, value });
            }
            previous = Some(value);
        }
        let samples = edges.iter().map(|&edge| self.locate(edge)).collect();
        Ok(Columns::new(self.id, self.revision, samples))
    }

    /// Channel `i`, which must exist.
    pub(crate) fn channel(&self, i: usize) -> &Channel {
        &self.channels[i]
    }

    /// Validates an append of `x` and `channels`, then stores it.
    fn append<C: AsRef<[f64]>>(&mut self, x: &[X], channels: &[C]) -> Result<(), Error<X>> {
        if channels.len() != self.channels.len() {
            return Err(Error::Channels {
                expected: self.channels.len(),
                found: channels.len(),
            });
        }
        let added = match &self.clock {
            Stored::Regular { .. } if !x.is_empty() => return Err(Error::Regular(x.len())),
            Stored::Regular { .. } => channels.first().map_or(0, |values| values.as_ref().len()),
            Stored::Explicit(_) => x.len(),
        };
        for (channel, values) in channels.iter().enumerate() {
            let found = values.as_ref().len();
            if found != added {
                return Err(Error::Length {
                    channel,
                    expected: added,
                    found,
                });
            }
        }
        let len = self.len as u64 + added as u64;
        if len > MAX {
            return Err(Error::Capacity { len, max: MAX });
        }
        self.check(x, added)?;
        if let Stored::Explicit(stored) = &mut self.clock {
            stored.extend(x);
        }
        for (channel, values) in self.channels.iter_mut().zip(channels) {
            channel.extend(values.as_ref());
        }
        self.len += added;
        Ok(())
    }

    /// Checks the x of `added` new samples: on a regular clock the last one
    /// (its x grows with the index), on an explicit clock each of `x`.
    fn check(&self, x: &[X], added: usize) -> Result<(), Error<X>> {
        let first = self.offset + self.len as u64;
        match &self.clock {
            Stored::Regular { .. } if added == 0 => Ok(()),
            Stored::Regular { start, period } => {
                let index = first + added as u64 - 1;
                let value = X::at(*start, *period, index);
                if value.is_valid() {
                    Ok(())
                } else {
                    Err(Error::Invalid { index, value })
                }
            }
            Stored::Explicit(stored) => {
                let mut previous = self.len.checked_sub(1).and_then(|i| stored.get(i));
                for (index, &next) in (first..).zip(x) {
                    if !next.is_valid() {
                        return Err(Error::Invalid { index, value: next });
                    }
                    if let Some(previous) = previous.filter(|previous| *previous > next) {
                        return Err(Error::Unordered {
                            index,
                            previous,
                            next,
                        });
                    }
                    previous = Some(next);
                }
                Ok(())
            }
        }
    }
}

/// The first index in `0..=len` where `below` fails, `below` holding on a
/// prefix: doubling steps outward from `guess` bracket it, then bisection
/// finds it, so a guess off by d costs O(log d) probes.
fn gallop(guess: usize, len: usize, below: impl Fn(usize) -> bool) -> usize {
    let (mut lo, mut hi) = (0, len);
    let mut step = 1;
    if guess < len && below(guess) {
        lo = guess + 1;
        while let Some(probe) = guess.checked_add(step).filter(|probe| *probe < len) {
            if !below(probe) {
                hi = probe;
                break;
            }
            lo = probe + 1;
            step *= 2;
        }
    } else {
        hi = guess;
        while let Some(probe) = guess.checked_sub(step) {
            if below(probe) {
                lo = probe + 1;
                break;
            }
            hi = probe;
            step *= 2;
        }
    }
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if below(mid) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use super::*;

    /// A 64-bit LCG, uniform in [0, 1).
    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> f64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (self.0 >> 11) as f64 / (1_u64 << 53) as f64
        }

        /// A length from 1 to `max`, log-uniform.
        fn len(&mut self, max: usize) -> usize {
            ((max as f64).powf(self.next()) as usize).clamp(1, max)
        }
    }

    /// Values with NaN runs, ties and ±∞.
    fn values(n: usize, random: &mut Lcg) -> Vec<f64> {
        let mut gap = 0;
        (0..n)
            .map(|_| {
                let u = random.next();
                if gap > 0 || u < 1e-4 {
                    gap = if gap > 0 {
                        gap - 1
                    } else {
                        (u * 1e8) as usize % 2_000
                    };
                    return f64::NAN;
                }
                match (u * 1e9) as u64 % 5_003 {
                    0 => f64::INFINITY,
                    1 => f64::NEG_INFINITY,
                    _ => (u * 64.0).floor() - 32.0,
                }
            })
            .collect()
    }

    /// Every channel's index levels as bits and runs.
    fn layout(rec: &Recording<f64>) -> Vec<crate::series::channel::Layout> {
        rec.channels.iter().map(Channel::layout).collect()
    }

    /// Column edges in samples and M4's polylines of every channel over a
    /// few views of the recording's x extent.
    fn thinned(rec: &Recording<f64>) -> Vec<(Vec<usize>, Vec<Vec<usize>>)> {
        let Some([lo, hi]) = rec.x_extent() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (a, b, columns) in [(0.0, 1.0, 997), (0.25, 0.3, 50), (0.5, 0.500_01, 3)] {
            let (a, b) = (lo + (hi - lo) * a, lo + (hi - lo) * b);
            let edges: Vec<f64> = (0..=columns)
                .map(|c| a + (b - a) * c as f64 / columns as f64)
                .collect();
            let grid = rec.columns(&edges).unwrap();
            for channel in 0..rec.channels() {
                let lines = rec
                    .series(channel)
                    .unwrap()
                    .m4(&grid, |y| y > -20.0)
                    .unwrap();
                out.push((
                    grid.edges().to_vec(),
                    lines.iter().map(<[usize]>::to_vec).collect(),
                ));
            }
        }
        out
    }

    #[test]
    fn split_appends_equal_a_one_shot_build() {
        let mut random = Lcg(17);
        for pattern in 0..20 {
            let n = 150_000 + random.len(250_000);
            let channels = [values(n, &mut random), values(n, &mut random)];
            // Odd patterns on an explicit clock with repeated x, even ones on a regular clock.
            let explicit = pattern % 2 == 1;
            let x: Vec<f64> = (0..n).map(|i| (i / 3) as f64 * 0.5).collect();
            let x = |range: Range<usize>| if explicit { &x[range] } else { &x[..0] };
            let clock = |x: &[f64]| {
                if explicit {
                    Clock::Explicit(x.to_vec())
                } else {
                    Clock::Regular {
                        start: 2.5,
                        period: 0.125,
                    }
                }
            };
            let pick = |range: Range<usize>| channels.iter().map(move |c| &c[range.clone()]);
            let once = Recording::new(clock(x(0..n)), pick(0..n)).unwrap();
            let first = random.len(200_000).min(n);
            let mut split = Recording::new(clock(x(0..first)), pick(0..first)).unwrap();
            let mut at = first;
            while at < n {
                let len = random.len(200_000).min(n - at);
                split.extend(x(at..at + len), pick(at..at + len)).unwrap();
                at += len;
            }
            assert_eq!(layout(&split), layout(&once), "pattern {pattern}");
            assert_eq!(thinned(&split), thinned(&once), "pattern {pattern}");
        }
    }

    #[test]
    fn a_recording_after_drop_front_equals_a_fresh_explicit_one_of_the_retained_samples() {
        let mut random = Lcg(23);
        let n = 6 * CHUNK + 12_345;
        let channels = [values(n, &mut random), values(n, &mut random)];
        let mut original = Recording::new(
            Clock::Regular {
                start: 3.0,
                period: 0.25,
            },
            [&channels[0][..70_000], &channels[1][..70_000]],
        )
        .unwrap();
        original
            .extend(&[], [&channels[0][70_000..], &channels[1][70_000..]])
            .unwrap();
        for chunks in [1, 2, 5] {
            let mut dropped = original.clone();
            assert_eq!(dropped.drop_front(chunks * CHUNK + 7), chunks * CHUNK);
            let retained = chunks * CHUNK..n;
            let x: Vec<f64> = (0..dropped.len()).map(|i| dropped.x(i).unwrap()).collect();
            let mut fresh = Recording::new(
                Clock::Explicit(x),
                channels.iter().map(|c| &c[retained.clone()]),
            )
            .unwrap();
            assert_eq!(layout(&dropped), layout(&fresh), "{chunks} chunks");
            assert_eq!(thinned(&dropped), thinned(&fresh), "{chunks} chunks");
            let more = values(3_000, &mut random);
            let next: Vec<f64> = (0..3_000).map(|i| 3.0 + (n + i) as f64 * 0.25).collect();
            dropped.extend(&[], [&more, &more]).unwrap();
            fresh.extend(&next, [&more, &more]).unwrap();
            assert_eq!(
                layout(&dropped),
                layout(&fresh),
                "{chunks} chunks, then an append"
            );
            assert_eq!(
                thinned(&dropped),
                thinned(&fresh),
                "{chunks} chunks, then an append"
            );
        }
    }

    #[test]
    fn appends_past_u32_max_retained_samples_are_rejected_before_storing() {
        let mut rec = Recording::new(
            Clock::Regular {
                start: 0.0,
                period: 1.0,
            },
            [[0.0; 3]],
        )
        .unwrap();
        // Stands in for u32::MAX − 2 stored samples; validation fails before storage is read.
        rec.len = u32::MAX as usize - 2;
        let err = rec.extend(&[], [[1.0; 3]]).unwrap_err();
        assert_eq!(
            err,
            Error::Capacity {
                len: MAX + 1,
                max: MAX
            }
        );
        assert_eq!(
            err.to_string(),
            "a recording holds at most 4294967295 samples, got 4294967296"
        );
        assert_eq!(rec.revision(), 0);
    }

    #[test]
    fn gallop_finds_the_partition_point_from_any_guess() {
        let x = [1, 3, 3, 3, 5, 8, 8, 9];
        for value in 0..11 {
            let expected = x.partition_point(|v| *v < value);
            for guess in 0..=x.len() {
                assert_eq!(
                    gallop(guess, x.len(), |i| x[i] < value),
                    expected,
                    "{value} {guess}"
                );
            }
        }
        assert_eq!(gallop(0, 0, |_| true), 0);
    }
}
