//! The capability traits for core's continuous scales with an `f64` range,
//! piecewise ones included: gestures through `Rescale`, fitting through
//! `Pad`, and the bridge to indexed recordings.

use std::any::Any;

use rustd3_core::interact::Rescale;
use rustd3_core::scale::{self, Continuous, Pad};
use rustd3_core::series::{Points, m4};

use crate::scale::{Indexed, Invert, Kind, Position, Scale, SeriesExtent, stops};
use crate::thin;

/// A piecewise scale's inner stops keep their share of the way between its
/// end stops when the range or the fitted domain changes.
impl<T: scale::Transform + 'static, const N: usize> Scale for Continuous<T, f64, N> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn boxed(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }

    fn with_range(&self, range: [f64; 2]) -> Box<dyn Scale> {
        Box::new(ranged(self, range))
    }

    fn pan_px(&self, range: [f64; 2], dx: f64) -> Option<Box<dyn Scale>> {
        let panned = Rescale::pan_px(&ranged(self, range), dx)?;
        Some(Box::new(panned))
    }

    fn zoom_px(&self, range: [f64; 2], anchor: f64, factor: f64) -> Option<Box<dyn Scale>> {
        let zoomed = Rescale::zoom_px(&ranged(self, range), anchor, factor)?;
        Some(Box::new(zoomed))
    }
}

impl<T: scale::Transform + 'static, const N: usize> Kind for Continuous<T, f64, N> {
    type Domain = f64;
    type Output = f64;
    type Span = f64;

    fn admits(&self, v: &f64) -> bool {
        Continuous::admits(self, *v)
    }

    fn merge(extent: &mut Option<[f64; 2]>, &v: &f64) {
        let [low, high] = extent.get_or_insert([v, v]);
        *low = low.min(v);
        *high = high.max(v);
    }

    fn fitted(&self, extent: Option<&[f64; 2]>, sticky: &[f64], margin: f64) -> Self {
        let ends = self.pad(extent.copied(), margin, sticky);
        self.clone().domain(stops(&self.get_domain(), ends))
    }

    fn domain(&self) -> Option<[f64; 2]> {
        let d = self.get_domain();
        Some([d[0], d[N - 1]])
    }
}

impl<T: scale::Transform + 'static, const N: usize> Position for Continuous<T, f64, N> {
    fn position(&self, v: &f64) -> Option<f64> {
        Some(self.apply(*v))
    }

    fn indexed(&self) -> Option<&dyn Indexed<f64>> {
        Some(self)
    }
}

impl<T: scale::Transform + 'static, const N: usize> Invert for Continuous<T, f64, N> {
    fn invert(&self, px: f64) -> Option<f64> {
        Some(Continuous::invert(self, px)).filter(|v| v.is_finite())
    }
}

impl<T: scale::Transform + 'static, const N: usize> Indexed<f64> for Continuous<T, f64, N> {
    fn invert(&self, px: f64) -> f64 {
        Continuous::invert(self, px)
    }

    fn series_extent(
        &self,
        points: &Points<'_, f64, f64>,
        x: Option<[f64; 2]>,
        valid: &dyn Fn(f64) -> bool,
    ) -> Option<SeriesExtent<f64>> {
        let series = points.series()?;
        let recording = series.recording();
        let samples = match x {
            None => 0..series.len(),
            Some([a, b]) => {
                let (low, high) = if b < a { (b, a) } else { (a, b) };
                recording.locate(low)..recording.locate(high.next_up())
            }
        };
        let last = samples.end.checked_sub(1)?;
        let x = [recording.x(samples.start)?, recording.x(last)?];
        let y = series
            .extremes(samples, valid)
            .map(|e| [e.min().1, e.max().1]);
        Some(SeriesExtent { x, y })
    }

    fn thin(
        &self,
        points: &Points<'_, f64, f64>,
        edges: &[f64],
        valid: &dyn Fn(f64) -> bool,
        frame: &mut thin::Frame,
    ) -> Option<m4::Lines> {
        let series = points.series()?;
        let reversed: Vec<f64>;
        let edges = if edges.first() > edges.last() {
            reversed = edges.iter().rev().copied().collect();
            &reversed
        } else {
            edges
        };
        let grid = thin::grid(edges.iter().map(|e| e.to_bits()));
        frame.lines(series, edges, grid, valid)
    }
}

/// A copy of `scale` whose range runs from `range[0]` to `range[1]`,
/// through `with_range`, which keeps the interpolator, clamp and unknown
/// value (d3's `range` for the same type).
fn ranged<T: scale::Transform, const N: usize>(
    scale: &Continuous<T, f64, N>,
    range: [f64; 2],
) -> Continuous<T, f64, N> {
    scale.clone().with_range(stops(scale.get_range(), range))
}

#[cfg(test)]
mod tests {
    use rustd3_core::series::{Clock, Data, Recording};

    use super::*;

    /// 1M samples on a regular clock: a ramp with a wobble, a NaN every
    /// 1,000th sample and a spike every 99,991st.
    fn ramp() -> Recording<f64> {
        let y: Vec<f64> = (0..1_000_000)
            .map(|i| {
                let v = f64::from(i);
                if i % 1000 == 7 {
                    f64::NAN
                } else if i % 99_991 == 0 {
                    -v
                } else {
                    v * 1e-3 + (v * 0.37).sin()
                }
            })
            .collect();
        Recording::new(
            Clock::Regular {
                start: 0.0,
                period: 1.0,
            },
            [y],
        )
        .expect("a valid recording")
    }

    /// The extent by a scan of every sample with x in `[a, b]`.
    fn scan(
        recording: &Recording<f64>,
        [a, b]: [f64; 2],
        valid: impl Fn(f64) -> bool,
    ) -> SeriesExtent<f64> {
        let series = recording.series(0).expect("channel 0");
        let inside: Vec<(f64, f64)> = series.iter().filter(|(x, _)| (a..=b).contains(x)).collect();
        let ys = inside
            .iter()
            .map(|p| p.1)
            .filter(|y| !y.is_nan() && valid(*y));
        let y = ys.fold(None, |e: Option<[f64; 2]>, y| match e {
            None => Some([y, y]),
            Some([lo, hi]) => Some([lo.min(y), hi.max(y)]),
        });
        SeriesExtent {
            x: [inside[0].0, inside[inside.len() - 1].0],
            y,
        }
    }

    #[test]
    fn indexed_extent_reads_the_index() {
        let recording = ramp();
        let points = recording.series(0).expect("channel 0").into_points();
        let x = scale::linear();
        let indexed = x.indexed().expect("continuous scales are indexed");
        let all = indexed.series_extent(&points, None, &|_| true);
        assert_eq!(all, Some(scan(&recording, [0.0, 999_999.0], |_| true)));
        let range = [250_000.0, 500_000.0];
        let part = indexed.series_extent(&points, Some(range), &|_| true);
        assert_eq!(part, Some(scan(&recording, range, |_| true)));
        let positive = indexed.series_extent(&points, Some(range), &|y| y > 0.0);
        assert_eq!(positive, Some(scan(&recording, range, |y| y > 0.0)));
        let reversed = indexed.series_extent(&points, Some([500_000.0, 250_000.0]), &|_| true);
        assert_eq!(reversed, part);
    }

    #[test]
    fn indexed_extent_is_none_without_a_series_or_samples() {
        let x = scale::linear();
        let indexed = x.indexed().expect("continuous scales are indexed");
        let owned = [(0.0, 1.0), (1.0, 2.0)].into_points();
        assert_eq!(indexed.series_extent(&owned, None, &|_| true), None);
        let recording = ramp();
        let points = recording.series(0).expect("channel 0").into_points();
        assert_eq!(
            indexed.series_extent(&points, Some([-5.0, -1.0]), &|_| true),
            None
        );
    }

    #[test]
    fn indexed_thinning_is_core_m4_over_located_columns() {
        let recording = ramp();
        let series = recording.series(0).expect("channel 0");
        let points = series.into_points();
        let x = scale::linear();
        let indexed = x.indexed().expect("continuous scales are indexed");
        let mut frame = thin::Frame::new(thin::Memo::default());
        let edges: Vec<f64> = (0..=100).map(|i| f64::from(i) * 10_000.0).collect();
        let columns = recording.columns(&edges).expect("valid edges");
        let core = series.m4(&columns, |_| true).expect("fresh columns");
        let thinned = indexed.thin(&points, &edges, &|_| true, &mut frame);
        assert_eq!(thinned, Some(core.clone()));
        let backwards: Vec<f64> = edges.iter().rev().copied().collect();
        let thinned = indexed.thin(&points, &backwards, &|_| true, &mut frame);
        assert_eq!(thinned, Some(core));
        let disordered = [1.0, 3.0, 2.0];
        assert_eq!(
            indexed.thin(&points, &disordered, &|_| true, &mut frame),
            None
        );
        let owned = [(0.0, 1.0)].into_points();
        assert_eq!(indexed.thin(&owned, &edges, &|_| true, &mut frame), None);
    }

    #[test]
    fn invert_round_trips_position() {
        let linear = scale::linear().domain([-3.0, 17.0]).range([0.0, 640.0]);
        let log = scale::log().domain([0.5, 2e4]).range([480.0, 0.0]);
        let values = [-3.0, -1.25, 0.0, 0.1, 2.5, 9.75, 17.0];
        for v in values {
            let back = Invert::invert(&linear, linear.position(&v).expect("placed"));
            let back = back.expect("finite");
            assert!(
                (back - v).abs() <= 1e-12 * v.abs().max(1.0),
                "linear {v}: {back}"
            );
        }
        for v in [0.5, 0.8, 1.0, 3.0, 47.0, 1234.5, 2e4] {
            let back = Invert::invert(&log, log.position(&v).expect("placed"));
            let back = back.expect("finite");
            assert!((back - v).abs() <= 1e-12 * v, "log {v}: {back}");
        }
    }

    #[test]
    fn invert_is_none_where_the_inverse_is_not_finite() {
        let log = scale::log().domain([1.0, 10.0]).range([0.0, 100.0]);
        assert_eq!(Invert::invert(&log, f64::INFINITY), None);
        assert_eq!(Invert::invert(&log, 1e6), None);
    }

    #[test]
    fn with_range_keeps_clamp_rounding_and_the_unknown_value() {
        let x = scale::linear()
            .domain([0.0, 10.0])
            .clamp(true)
            .unknown(-1.0)
            .range_round([0.0, 1.0]);
        let ranged = Scale::with_range(&x, [0.0, 100.0]);
        let ranged = ranged
            .as_any()
            .downcast_ref::<scale::Linear>()
            .expect("its type");
        assert_eq!(ranged.apply(20.0), 100.0);
        assert_eq!(ranged.apply(f64::NAN), -1.0);
        assert_eq!(ranged.apply(5.56), 56.0);
    }

    #[test]
    fn a_piecewise_range_keeps_the_shares_of_its_inner_stops() {
        let s = scale::piecewise([0.0, 10.0, 100.0], [0.0, 0.5, 1.0]);
        let ranged = Scale::with_range(&s, [0.0, 400.0]);
        let ranged = ranged
            .as_any()
            .downcast_ref::<Continuous<scale::transform::Identity, f64, 3>>()
            .expect("its type");
        assert_eq!(ranged.get_range(), &[0.0, 200.0, 400.0]);
        assert_eq!(ranged.position(&10.0), Some(200.0));
    }

    #[test]
    fn gestures_delegate_to_core_on_the_panel_range() {
        let x = scale::linear().domain([0.0, 100.0]);
        let panned = Scale::pan_px(&x, [0.0, 960.0], 96.0).expect("pans");
        let panned = panned
            .as_any()
            .downcast_ref::<scale::Linear>()
            .expect("its type");
        assert_eq!(panned.get_domain(), [-10.0, 90.0]);
        let zoomed = Scale::zoom_px(&x, [0.0, 960.0], 480.0, 2.0).expect("zooms");
        let zoomed = zoomed
            .as_any()
            .downcast_ref::<scale::Linear>()
            .expect("its type");
        assert_eq!(zoomed.get_domain(), [25.0, 75.0]);
        assert!(Scale::zoom_px(&x, [0.0, 960.0], 480.0, 1e13).is_none());
    }

    #[test]
    fn fitting_pads_through_core() {
        let y = scale::linear();
        let bars = y.fitted(Some(&[0.0, 4.0]), &[0.0], 0.05);
        assert_eq!(Kind::domain(&bars), Some([0.0, 4.2]));
        assert_eq!(Kind::domain(&y.fitted(None, &[], 0.05)), Some([0.0, 1.0]));
        let mut extent = None;
        for v in [3.0, -1.0, 2.0] {
            <scale::Linear as Kind>::merge(&mut extent, &v);
        }
        assert_eq!(extent, Some([-1.0, 3.0]));
    }
}
