//! [`Time`], d3's time scale with an explicit zone, computing in `i128`
//! nanoseconds from the domain's start.

mod labels;
mod pad;
mod rescale;
mod ticks;

use jiff::Timestamp;
use jiff::tz::TimeZone;

use crate::interpolate::Interpolate;
use crate::time;

/// d3's scaleTime and scaleUtc: instants mapped linearly onto a range of
/// numbers, with ticks and nice in the scale's zone. Builders consume and
/// return the value. Made by [`time::time`] and [`time::utc`].
///
/// Offsets from the domain's start are `i128` nanoseconds; only their
/// ratios pass through `f64`, so instants in 2026 keep their nanoseconds,
/// where an `f64` of epoch nanoseconds resolves 256 ns.
///
/// ```
/// use rustd3_core::jiff::Timestamp;
/// use rustd3_core::time;
///
/// let t0: Timestamp = "2026-10-25T12:00:00Z".parse()?;
/// let t1: Timestamp = "2026-10-25T12:00:00.000001Z".parse()?;
/// let x = time::utc().domain([t0, t1]).range([0.0, 1000.0]);
/// assert_eq!(x.apply("2026-10-25T12:00:00.000000001Z".parse()?), 1.0);
/// assert_eq!(x.invert(500.0), "2026-10-25T12:00:00.0000005Z".parse()?);
/// # Ok::<(), rustd3_core::jiff::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Time {
    domain: [Timestamp; 2],
    range: [f64; 2],
    zone: TimeZone,
    clamp: bool,
    /// Whether ticks use d3's utc intervals (`UnixDay` for days), as
    /// `scaleUtc`'s do.
    utc_ticks: bool,
}

impl Time {
    /// `domain` onto [0, 1], unclamped, with ticks in `zone`.
    pub(super) fn new(domain: [Timestamp; 2], zone: TimeZone, utc_ticks: bool) -> Time {
        Time {
            domain,
            range: [0.0, 1.0],
            zone,
            clamp: false,
            utc_ticks,
        }
    }

    /// d3's `domain(d)`: the instants the range's ends map from; either
    /// order, equal ends included.
    #[must_use]
    pub fn domain(self, domain: [Timestamp; 2]) -> Self {
        Time { domain, ..self }
    }

    /// d3's `domain()`.
    #[must_use]
    pub fn get_domain(&self) -> [Timestamp; 2] {
        self.domain
    }

    /// d3's `range(r)`: the values the domain's ends map to.
    #[must_use]
    pub fn range(self, range: [f64; 2]) -> Self {
        Time { range, ..self }
    }

    /// d3's `range()`.
    #[must_use]
    pub fn get_range(&self) -> [f64; 2] {
        self.range
    }

    /// d3's `clamp(b)`: whether `apply` and `invert` keep to the domain.
    #[must_use]
    pub fn clamp(self, clamp: bool) -> Self {
        Time { clamp, ..self }
    }

    /// The zone ticks and nice lay intervals out in.
    #[must_use]
    pub fn get_zone(&self) -> &TimeZone {
        &self.zone
    }

    /// The range value of `t` (d3's `scale(t)`). `t`'s offset from the
    /// domain's start, as a fraction of the domain's span, is blended
    /// between the range's ends as d3's `interpolateNumber` does. Equal
    /// domain ends give the middle of the range, as d3's `normalize` does.
    /// When clamping, the fraction is kept to [0, 1] first.
    #[must_use]
    pub fn apply(&self, t: Timestamp) -> f64 {
        let span = self.span();
        let fraction = if span == 0 {
            0.5
        } else {
            (t.as_nanosecond() - self.start()) as f64 / span as f64
        };
        let [r0, r1] = self.range;
        r0.lerp(&r1, self.kept(fraction))
    }

    /// d3's `invert(y)`: the instant that maps to `y`, rounded to the
    /// nanosecond. A flat range gives the domain's middle (d3's
    /// `normalize`), and clamping keeps the result within the domain. It
    /// saturates at jiff's first and last instants. A NaN `y`, where d3 has
    /// an Invalid Date, gives the domain's start.
    #[must_use]
    pub fn invert(&self, y: f64) -> Timestamp {
        let offset = self.offset(self.kept(self.position(y)));
        time::saturating(self.start().saturating_add(offset))
    }

    /// Where `y` lies from the range's start (0) to its end (1), unclamped;
    /// 0.5 for a flat range, as d3's `normalize`.
    fn position(&self, y: f64) -> f64 {
        let [r0, r1] = self.range;
        let extent = r1 - r0;
        if extent == 0.0 {
            0.5
        } else {
            (y - r0) / extent
        }
    }

    /// The domain's start in nanoseconds from the epoch.
    fn start(&self) -> i128 {
        self.domain[0].as_nanosecond()
    }

    /// The domain's end less its start, in nanoseconds; negative for a
    /// descending domain.
    fn span(&self) -> i128 {
        self.domain[1].as_nanosecond() - self.start()
    }

    /// The nanoseconds from the domain's start at `fraction` of its span,
    /// rounded; a NaN fraction gives 0 and an infinite one saturates.
    fn offset(&self, fraction: f64) -> i128 {
        // A float to integer cast saturates, and NaN casts to 0.
        (fraction * self.span() as f64).round() as i128
    }

    /// `fraction` kept to [0, 1] when clamping (NaN stays NaN).
    fn kept(&self, fraction: f64) -> f64 {
        if self.clamp {
            fraction.clamp(0.0, 1.0)
        } else {
            fraction
        }
    }
}
