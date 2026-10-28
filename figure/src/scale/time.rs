//! The capability traits for core's time scale: positions over
//! `Timestamp`, gestures through `Rescale` and fitting through `Pad`, all
//! in `i128` nanoseconds, and the bridge to `Recording<Timestamp>`.

use std::any::Any;

use rustd3_core::interact::Rescale;
use rustd3_core::jiff::{SignedDuration, Timestamp};
use rustd3_core::scale::Pad;
use rustd3_core::series::{Points, m4};
use rustd3_core::time::Time;

use crate::scale::{Indexed, Invert, Kind, Position, Scale, SeriesExtent, indexed, stuck, through};
use crate::thin;

/// Time scales pan and zoom in nanoseconds from the domain's start.
impl Scale for Time {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn boxed(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }

    fn with_range(&self, range: [f64; 2]) -> Box<dyn Scale> {
        Box::new(self.clone().range(range))
    }

    fn pan_px(&self, range: [f64; 2], dx: f64) -> Option<Box<dyn Scale>> {
        let panned = Rescale::pan_px(&self.clone().range(range), dx)?;
        Some(Box::new(panned))
    }

    fn zoom_px(&self, range: [f64; 2], anchor: f64, factor: f64) -> Option<Box<dyn Scale>> {
        let zoomed = Rescale::zoom_px(&self.clone().range(range), anchor, factor)?;
        Some(Box::new(zoomed))
    }

    /// Through the instants `from` places at `at`, to the nanosecond.
    fn remap(&self, from: &dyn Scale, range: [f64; 2], at: [f64; 2]) -> Option<[f64; 2]> {
        let from = from.as_any().downcast_ref::<Time>()?;
        through(&from.clone().range(range), &self.clone().range(range), at)
    }
}

/// Instants fit as numbers do, through core's `Pad for Time`: no data
/// gives 1970-01-01 to 1970-01-02 UTC, one instant widens by an hour per
/// side, and margins are whole nanoseconds.
impl Kind for Time {
    type Domain = Timestamp;
    type Output = f64;
    type Span = SignedDuration;

    /// Every instant.
    fn admits(&self, _t: &Timestamp) -> bool {
        true
    }

    fn merge(extent: &mut Option<[Timestamp; 2]>, &t: &Timestamp) {
        let [low, high] = extent.get_or_insert([t, t]);
        *low = (*low).min(t);
        *high = (*high).max(t);
    }

    fn fitted(&self, extent: Option<&[Timestamp; 2]>, sticky: &[Timestamp], margin: f64) -> Self {
        let ends = self.pad(extent.copied(), margin, sticky);
        self.clone().domain(ends)
    }

    /// Fitted without a margin, then floored and ceiled to the tick
    /// interval for `count` (core's `nice(count)`, in the scale's zone),
    /// stopped by `sticky` instants.
    fn niced(&self, extent: Option<&[Timestamp; 2]>, sticky: &[Timestamp], count: usize) -> Self {
        let nice = self.fitted(extent, sticky, 0.0).nice(count);
        let ends = stuck(nice.get_domain(), extent, sticky);
        self.clone().domain(ends)
    }

    fn domain(&self) -> Option<[Timestamp; 2]> {
        Some(self.get_domain())
    }
}

impl Position for Time {
    fn position(&self, t: &Timestamp) -> Option<f64> {
        Some(self.apply(*t))
    }

    fn indexed(&self) -> Option<&dyn Indexed<Timestamp>> {
        Some(self)
    }
}

impl Invert for Time {
    /// The instant at `px`, to the nanosecond; `None` for a position that
    /// is not finite.
    fn invert(&self, px: f64) -> Option<Timestamp> {
        px.is_finite().then(|| Time::invert(self, px))
    }
}

impl Indexed<Timestamp> for Time {
    fn invert(&self, px: f64) -> Timestamp {
        Time::invert(self, px)
    }

    fn series_extent(
        &self,
        points: &Points<'_, Timestamp, f64>,
        x: Option<[Timestamp; 2]>,
        valid: &dyn Fn(f64) -> bool,
    ) -> Option<SeriesExtent<Timestamp>> {
        let x = x.map(|[a, b]| {
            let (low, high) = if b < a { (b, a) } else { (a, b) };
            (low, high.checked_add(SignedDuration::from_nanos(1)).ok())
        });
        indexed::extent(points, x, valid)
    }

    fn thin(
        &self,
        points: &Points<'_, Timestamp, f64>,
        edges: &[Timestamp],
        valid: &dyn Fn(f64) -> bool,
        frame: &mut thin::Frame,
    ) -> Option<m4::Lines> {
        indexed::thin(points, None, edges, bits, valid, frame)
    }

    fn thin_joint(
        &self,
        lower: &Points<'_, Timestamp, f64>,
        upper: &Points<'_, Timestamp, f64>,
        edges: &[Timestamp],
        valid: &dyn Fn(f64) -> bool,
        frame: &mut thin::Frame,
    ) -> Option<m4::Lines> {
        indexed::thin(lower, Some(upper), edges, bits, valid, frame)
    }
}

/// An instant's i128 nanoseconds as two words, low first: a column grid's
/// key.
fn bits(t: &Timestamp) -> [u64; 2] {
    let ns = t.as_nanosecond() as u128;
    [ns as u64, (ns >> 64) as u64]
}

#[cfg(test)]
mod tests {
    use rustd3_core::series::{Clock, Data, Recording};
    use rustd3_core::time;

    use super::*;

    /// `s` as an instant.
    fn at(s: &str) -> Timestamp {
        s.parse().expect("an instant")
    }

    #[test]
    fn time_scales_fit_through_core_in_nanoseconds() {
        let x = time::utc();
        let t = at("2026-10-25T12:00:00.000000001Z");
        let micro = t + SignedDuration::from_micros(1);
        let fitted = x.fitted(Some(&[t, micro]), &[], 0.05);
        let ns = SignedDuration::from_nanos(50);
        assert_eq!(Kind::domain(&fitted), Some([t - ns, micro + ns]));
        let empty = x.fitted(None, &[], 0.05).get_domain();
        assert_eq!(empty, [Timestamp::UNIX_EPOCH, at("1970-01-02T00:00:00Z")]);
        let mut extent = None;
        for v in [micro, t, micro] {
            <Time as Kind>::merge(&mut extent, &v);
        }
        assert_eq!(extent, Some([t, micro]));
    }

    #[test]
    fn time_gestures_delegate_to_core_on_the_panel_range() {
        let day = [at("2026-10-25T00:00:00Z"), at("2026-10-26T00:00:00Z")];
        let x = time::utc().domain(day);
        let panned = Scale::pan_px(&x, [0.0, 960.0], 96.0).expect("pans");
        let panned = panned.as_any().downcast_ref::<Time>().expect("its type");
        let tenth = SignedDuration::from_mins(144);
        assert_eq!(panned.get_domain(), [day[0] - tenth, day[1] - tenth]);
        let zoomed = Scale::zoom_px(&x, [0.0, 960.0], 480.0, 2.0).expect("zooms");
        let zoomed = zoomed.as_any().downcast_ref::<Time>().expect("its type");
        let six = SignedDuration::from_hours(6);
        assert_eq!(zoomed.get_domain(), [day[0] + six, day[1] - six]);
        let ranged = Scale::with_range(&x, [0.0, 24.0]);
        let ranged = ranged.as_any().downcast_ref::<Time>().expect("its type");
        assert_eq!(ranged.position(&at("2026-10-25T03:00:00Z")), Some(3.0));
        assert_eq!(
            Invert::invert(ranged, 3.0),
            Some(at("2026-10-25T03:00:00Z"))
        );
        assert_eq!(Invert::invert(ranged, f64::NAN), None);
        // Panned by 96 px, the instants at the edges moved by 96 px.
        let range = [0.0, 960.0];
        assert_eq!(
            Scale::pan_px(&x, range, 96.0)
                .expect("pans")
                .remap(&x, range, [0.0, 960.0]),
            Some([96.0, 1056.0])
        );
        let linear = rustd3_core::scale::linear();
        assert_eq!(Scale::remap(&x, &linear, range, [0.0, 960.0]), None);
    }

    #[test]
    fn indexed_extent_includes_both_ends_of_the_window() {
        let start = at("2026-10-25T12:00:00Z");
        let period = SignedDuration::from_millis(100);
        let y: Vec<f64> = (0..100).map(f64::from).collect();
        let recording = Recording::new(Clock::Regular { start, period }, [y]).expect("valid");
        let points = recording.series(0).expect("channel 0").into_points();
        let x = time::utc();
        let indexed = x.indexed().expect("time scales are indexed");
        let all = indexed.series_extent(&points, None, &|_| true);
        assert_eq!(all.and_then(|e| e.y), Some([0.0, 99.0]));
        let window = [start + period * 10, start + period * 20];
        let part = indexed
            .series_extent(&points, Some(window), &|_| true)
            .expect("samples in the window");
        assert_eq!((part.x, part.y), (window, Some([10.0, 20.0])));
        let reversed = indexed.series_extent(&points, Some([window[1], window[0]]), &|_| true);
        assert_eq!(reversed, Some(part));
        let last = [start, Timestamp::MAX];
        let to_max = indexed.series_extent(&points, Some(last), &|_| true);
        assert_eq!(to_max.and_then(|e| e.y), Some([0.0, 99.0]));
    }
}
