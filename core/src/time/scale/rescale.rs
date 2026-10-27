//! [`Time`]'s gestures: Phase 10's [`Rescale`] in `i128` nanoseconds
//! from the domain's start, where only differences pass through `f64`.

use crate::interact::{self, Rescale};
use crate::time::{self, Time};

/// d3's `rescaleX` and `rescaleY` through `invert`, so a clamping scale
/// cannot zoom out, as in d3. `pan_px` and `zoom_px` do not clamp; they
/// return `None` when an input is not finite, the factor is not positive,
/// the new span is under 1 ns or reversed, or an end leaves jiff's range.
///
/// ```
/// use rustd3_core::interact::Rescale;
/// use rustd3_core::jiff::Timestamp;
/// use rustd3_core::time;
///
/// let t0: Timestamp = "2026-10-25T12:00:00Z".parse()?;
/// let t1: Timestamp = "2026-10-25T12:00:00.000001Z".parse()?;
/// let x = time::utc().domain([t0, t1]).range([0.0, 1000.0]);
/// let zoomed = x.zoom_px(500.0, 2.0).map(|x| x.get_domain());
/// let quarter = |s: &str| s.parse::<Timestamp>();
/// assert_eq!(zoomed, Some([quarter("2026-10-25T12:00:00.00000025Z")?, quarter("2026-10-25T12:00:00.00000075Z")?]));
/// assert!(x.zoom_px(500.0, 2000.0).is_none());
/// # Ok::<(), rustd3_core::jiff::Error>(())
/// ```
impl Rescale for Time {
    fn rescale_x(&self, t: &interact::Transform) -> Self {
        let domain = self.range.map(|r| self.invert(t.invert_x(r)));
        self.clone().domain(domain)
    }

    fn rescale_y(&self, t: &interact::Transform) -> Self {
        let domain = self.range.map(|r| self.invert(t.invert_y(r)));
        self.clone().domain(domain)
    }

    /// Both ends move by `−round(dx / (r1 − r0) · span)` ns.
    fn pan_px(&self, dx: f64) -> Option<Self> {
        if !dx.is_finite() {
            return None;
        }
        let [r0, r1] = self.range;
        let shift = self.offset(-dx / (r1 - r0));
        self.moved([0, self.span()].map(|end| end.checked_add(shift)))
    }

    /// The anchor's offset `a` (unclamped) stays; each end moves to `a +
    /// round((end − a) / factor)` ns.
    fn zoom_px(&self, anchor: f64, factor: f64) -> Option<Self> {
        if !(anchor.is_finite() && factor.is_finite() && factor > 0.0) {
            return None;
        }
        let a = self.offset(self.position(anchor));
        let toward = |end: i128| {
            // A float to integer cast saturates.
            let moved = (end.checked_sub(a)? as f64 / factor).round() as i128;
            a.checked_add(moved)
        };
        self.moved([0, self.span()].map(toward))
    }
}

impl Time {
    /// The scale with its domain at `ends`, offsets from the old start, if
    /// both are instants jiff holds and they span at least 1 ns in the old
    /// domain's direction.
    fn moved(&self, ends: [Option<i128>; 2]) -> Option<Time> {
        let [from, to] = ends;
        let (from, to) = (from?, to?);
        let span = to.checked_sub(from)?;
        if span == 0 || span.signum() != self.span().signum() {
            return None;
        }
        let start = self.start();
        let domain = [
            time::at(start.checked_add(from)?)?,
            time::at(start.checked_add(to)?)?,
        ];
        Some(self.clone().domain(domain))
    }
}
