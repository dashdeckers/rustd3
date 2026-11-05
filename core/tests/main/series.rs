//! Recordings: storage, clocks, appends and M4, the index's answers
//! checked against raw scans of the same samples.

mod agree;
mod exact;
mod timestamp;

use rustd3_core::series::{self, Clock, Coordinate, Data, Error, Points, Recording};

/// A seeded generator: a 64-bit LCG, uniform in [0, 1).
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 11) as f64 / (1_u64 << 53) as f64
    }
}

fn regular(start: f64, period: f64, channels: &[&[f64]]) -> Recording<f64> {
    Recording::new(Clock::Regular { start, period }, channels.iter().copied())
        .expect("a valid recording")
}

/// Every sample's x, in order.
fn xs<X: Coordinate>(rec: &Recording<X>) -> Vec<X> {
    (0..rec.len()).map(|i| rec.x(i).expect("i < len")).collect()
}

/// Column edges in samples and M4's polylines for data-space `edges`, from
/// channel 0's index.
fn thinned_by_index<X: Coordinate>(
    rec: &Recording<X>,
    edges: &[X],
    valid: impl Fn(f64) -> bool,
) -> (Vec<usize>, Vec<Vec<usize>>) {
    let columns = rec.columns(edges).expect("valid edges");
    let series = rec.series(0).expect("channel 0");
    let lines = series.m4(&columns, valid).expect("fresh columns");
    let lines = lines.iter().map(<[usize]>::to_vec).collect();
    (columns.get_edges().to_vec(), lines)
}

/// The raw-scan reference: column edges by `partition_point` over `x`, NaN
/// runs and per-column extremes by scanning `y`, and M4 by its
/// definition: first, last, least and greatest per column.
fn m4_scan<X: PartialOrd>(
    x: &[X],
    y: &[f64],
    edges: &[X],
    valid: impl Fn(f64) -> bool,
) -> (Vec<usize>, Vec<Vec<usize>>) {
    scan(x, &[y], edges, valid)
}

/// The raw-scan reference of a joint query: runs where no channel of `ys`
/// is NaN, and per column each channel's extremes.
fn scan<X: PartialOrd>(
    x: &[X],
    ys: &[&[f64]],
    edges: &[X],
    valid: impl Fn(f64) -> bool,
) -> (Vec<usize>, Vec<Vec<usize>>) {
    let edges: Vec<usize> = edges.iter().map(|e| x.partition_point(|t| t < e)).collect();
    let mut runs = Vec::new();
    let mut start = None;
    let gap = |i: usize| ys.iter().any(|y| y[i].is_nan());
    for i in 0..x.len() {
        match (gap(i), start) {
            (false, None) => start = Some(i),
            (true, Some(s)) => {
                runs.push((s, i));
                start = None;
            }
            (false, Some(_)) | (true, None) => {}
        }
    }
    runs.extend(start.map(|s| (s, x.len())));
    let (view_a, view_b) = (edges[0], edges[edges.len() - 1]);
    let mut lines = Vec::new();
    for (ra, rb) in runs {
        if ra >= view_b || rb <= view_a {
            continue;
        }
        let mut line = Vec::new();
        if ra < view_a {
            line.push(view_a - 1);
        }
        for column in edges.windows(2) {
            let (a, b) = (column[0].max(ra), column[1].min(rb));
            if a >= b {
                continue;
            }
            let mut picks = vec![a, b - 1];
            for y in ys {
                let (mut min, mut max) = (None::<usize>, None::<usize>);
                for i in (a..b).filter(|&i| valid(y[i])) {
                    if min.is_none_or(|m| y[i] < y[m]) {
                        min = Some(i);
                    }
                    if max.is_none_or(|m| y[i] > y[m]) {
                        max = Some(i);
                    }
                }
                picks.extend(min.into_iter().chain(max));
            }
            picks.sort_unstable();
            picks.dedup();
            line.extend(picks);
        }
        if rb > view_b {
            line.push(view_b);
        }
        lines.push(line);
    }
    (edges, lines)
}

#[test]
fn a_regular_clock_needs_a_finite_positive_period_and_a_valid_start() {
    for period in [0.0, -1.0, f64::INFINITY] {
        let err = Recording::new(Clock::Regular { start: 0.0, period }, [[1.0]]).unwrap_err();
        assert_eq!(err, Error::Period(period));
    }
    let err = Recording::new(
        Clock::Regular {
            start: 0.0,
            period: f64::NAN,
        },
        [[1.0]],
    )
    .unwrap_err();
    assert!(matches!(err, Error::Period(p) if p.is_nan()));
    let err = Recording::new(
        Clock::Regular {
            start: f64::NEG_INFINITY,
            period: 1.0,
        },
        [[1.0]],
    );
    assert_eq!(
        err.unwrap_err(),
        Error::Invalid {
            index: 0,
            value: f64::NEG_INFINITY
        }
    );
    assert_eq!(
        Error::<f64>::Period(0.0).to_string(),
        "a regular clock needs a finite positive period, got 0.0"
    );
}

#[test]
fn a_regular_clock_rejects_samples_past_the_largest_finite_x() {
    let err = Recording::new(
        Clock::Regular {
            start: 1e308,
            period: 1e308,
        },
        [[1.0, 2.0]],
    );
    assert_eq!(
        err.unwrap_err(),
        Error::Invalid {
            index: 1,
            value: f64::INFINITY
        }
    );
}

#[test]
fn explicit_x_must_be_finite_and_non_decreasing() {
    let err = Recording::new(Clock::Explicit(vec![0.0, 1.0, 0.5]), [[0.0; 3]]).unwrap_err();
    assert_eq!(
        err,
        Error::Unordered {
            index: 2,
            previous: 1.0,
            next: 0.5
        }
    );
    assert_eq!(
        err.to_string(),
        "x is not ordered at sample 2: 0.5 after 1.0"
    );
    let err = Recording::new(Clock::Explicit(vec![0.0, f64::INFINITY]), [[0.0; 2]]).unwrap_err();
    assert_eq!(
        err,
        Error::Invalid {
            index: 1,
            value: f64::INFINITY
        }
    );
    let err = Recording::new(Clock::Explicit(vec![f64::NAN]), [[0.0]]).unwrap_err();
    assert!(matches!(err, Error::Invalid { index: 0, value } if value.is_nan()));
    assert!(Recording::new(Clock::Explicit(vec![0.0, 0.0, 1.0]), [[0.0; 3]]).is_ok());
}

#[test]
fn every_channel_needs_as_many_samples_as_the_clock() {
    let err = Recording::new(
        Clock::Explicit(vec![0.0, 1.0, 2.0]),
        [&[1.0, 2.0, 3.0][..], &[1.0, 2.0]],
    );
    assert_eq!(
        err.unwrap_err(),
        Error::Length {
            channel: 1,
            expected: 3,
            found: 2
        }
    );
    let err = Recording::new(Clock::Explicit(vec![0.0, 1.0]), [&[1.0, 2.0, 3.0][..]]);
    assert_eq!(
        err.unwrap_err(),
        Error::Length {
            channel: 0,
            expected: 2,
            found: 3
        }
    );
    let err = Recording::new(
        Clock::Regular {
            start: 0.0,
            period: 1.0,
        },
        [&[1.0, 2.0][..], &[1.0]],
    );
    assert_eq!(
        err.unwrap_err(),
        Error::Length {
            channel: 1,
            expected: 2,
            found: 1
        }
    );
}

#[test]
fn appends_name_every_channel_and_regular_clocks_take_no_x() {
    let mut rec = regular(0.0, 1.0, &[&[1.0], &[2.0]]);
    assert_eq!(
        rec.extend(&[], [[3.0]]),
        Err(Error::Channels {
            expected: 2,
            found: 1
        })
    );
    assert_eq!(
        rec.extend(&[], [[3.0]; 3]),
        Err(Error::Channels {
            expected: 2,
            found: 3
        })
    );
    assert_eq!(rec.extend(&[1.0, 2.0], [[3.0]; 2]), Err(Error::Regular(2)));
    assert_eq!(
        rec.extend(&[], [&[3.0, 4.0][..], &[3.0]]),
        Err(Error::Length {
            channel: 1,
            expected: 2,
            found: 1
        })
    );
    assert_eq!((rec.len(), rec.get_revision()), (1, 0));
}

#[test]
fn zero_channels_are_allowed() {
    let rec = Recording::new(
        Clock::Explicit(vec![0.0, 2.0]),
        std::iter::empty::<&[f64]>(),
    )
    .unwrap();
    assert_eq!(
        (rec.len(), rec.channels(), rec.x_extent()),
        (2, 0, Some([0.0, 2.0]))
    );
    let rec = Recording::new(
        Clock::Regular {
            start: 0.0,
            period: 1.0,
        },
        std::iter::empty::<&[f64]>(),
    )
    .unwrap();
    assert!(rec.is_empty());
    assert_eq!(rec.x_extent(), None);
}

#[test]
fn a_failed_append_leaves_the_recording_as_it_was() {
    let mut rec = Recording::new(Clock::Explicit(vec![0.0, 1.0]), [[5.0, 6.0]]).unwrap();
    assert_eq!(
        rec.extend(&[2.0, 1.5], [[7.0, 8.0]]),
        Err(Error::Unordered {
            index: 3,
            previous: 2.0,
            next: 1.5
        })
    );
    assert_eq!(
        rec.extend(&[0.5], [[7.0]]),
        Err(Error::Unordered {
            index: 2,
            previous: 1.0,
            next: 0.5
        })
    );
    assert_eq!(
        (rec.len(), rec.get_revision(), xs(&rec)),
        (2, 0, vec![0.0, 1.0])
    );
    rec.extend(&[1.0, 3.0], [[7.0, 8.0]]).unwrap();
    assert_eq!(
        (rec.len(), rec.get_revision(), xs(&rec)),
        (4, 1, vec![0.0, 1.0, 1.0, 3.0])
    );
}

#[test]
fn clones_get_fresh_ids() {
    let rec = regular(0.0, 1.0, &[&[1.0, 2.0]]);
    let clone = rec.clone();
    let other = regular(0.0, 1.0, &[&[1.0, 2.0]]);
    assert_ne!(rec.get_id(), clone.get_id());
    assert_ne!(rec.get_id(), other.get_id());
    assert_ne!(clone.get_id(), other.get_id());
    assert_eq!(
        (clone.len(), clone.get_revision(), xs(&clone)),
        (2, 0, xs(&rec))
    );
}

#[test]
fn the_revision_counts_appends_and_drops_that_dropped() {
    let mut rec = regular(0.0, 1.0, &[&[0.0; 100]]);
    assert_eq!(rec.get_revision(), 0);
    rec.extend(&[], [vec![1.0; 200_000]]).unwrap();
    rec.extend(&[], [[0.0; 0]]).unwrap();
    assert_eq!(rec.get_revision(), 2);
    assert_eq!(rec.drop_front(65_535), 0);
    assert_eq!(rec.get_revision(), 2);
    assert_eq!(rec.drop_front(65_536), 65_536);
    assert_eq!(rec.get_revision(), 3);
}

#[test]
fn drop_front_drops_whole_chunks_and_keeps_the_regular_numbering() {
    let mut rec = regular(0.0, 0.1, &[&vec![0.0; 200_000], &vec![1.0; 200_000]]);
    let revision = rec.get_revision();
    assert_eq!(rec.drop_front(150_000), 131_072);
    assert_eq!(
        (rec.len(), rec.get_offset(), rec.get_revision()),
        (68_928, 131_072, revision + 1)
    );
    assert_eq!(rec.x(0), Some(13107.2));
    assert_eq!(rec.x(68_928), None);
    assert_eq!(rec.drop_front(usize::MAX), 65_536);
    assert_eq!((rec.len(), rec.get_offset()), (3_392, 196_608));
    assert_eq!(rec.drop_front(usize::MAX), 0);
}

#[test]
fn drop_front_on_an_explicit_clock_keeps_the_retained_x() {
    let x: Vec<f64> = (0..140_000).map(|i| f64::from(i) * 0.5).collect();
    let mut rec = Recording::new(Clock::Explicit(x.clone()), [&x]).unwrap();
    assert_eq!(rec.drop_front(140_000), 131_072);
    assert_eq!(xs(&rec), x[131_072..]);
    assert_eq!(rec.locate(x[131_072]), 0);
    assert_eq!(rec.locate(x[139_999] + 0.1), 8_928);
}

#[test]
fn regular_locate_equals_partition_point_over_the_materialized_x() {
    let mut random = Lcg(5);
    for (start, period) in [(0.1, 1e-4), (1e9, 1e-6)] {
        let mut rec = regular(start, period, &[&vec![0.0; 150_000]]);
        for dropped in [0, 65_536] {
            rec.drop_front(dropped);
            let x = xs(&rec);
            let (lo, hi) = (x[0], x[x.len() - 1]);
            let span = hi - lo;
            for k in 0..10_000 {
                let value = match k % 4 {
                    0 => x[(random.next() * x.len() as f64) as usize],
                    1 => lo - span * random.next(),
                    _ => lo - 0.1 * span + 1.2 * span * random.next(),
                };
                assert_eq!(
                    rec.locate(value),
                    x.partition_point(|t| *t < value),
                    "{start} {period} {value}"
                );
            }
            for value in [f64::NEG_INFINITY, f64::INFINITY, f64::NAN, -1e300, 1e300] {
                assert_eq!(
                    rec.locate(value),
                    x.partition_point(|t| *t < value),
                    "{value}"
                );
            }
        }
    }
    assert_eq!(regular(0.0, 1.0, &[&[0.0; 10]]).locate(-5.0), 0);
}

#[test]
fn explicit_locate_equals_partition_point_across_chunks_and_duplicates() {
    let x: Vec<f64> = (0..200_000).map(|i| f64::from(i / 3)).collect();
    let rec = Recording::new(Clock::Explicit(x.clone()), std::iter::empty::<&[f64]>()).unwrap();
    for value in [
        -1.0,
        0.0,
        0.5,
        21_845.0,
        21_845.5,
        43_690.0,
        66_666.0,
        66_667.0,
        f64::NAN,
    ] {
        assert_eq!(
            rec.locate(value),
            x.partition_point(|t| *t < value),
            "{value}"
        );
    }
}

/// The spot checks' channel y: two NaN at 4 and 5.
const Y: [f64; 10] = [0.0, 5.0, -1.0, 3.0, f64::NAN, f64::NAN, 2.0, 8.0, 1.0, 4.0];

/// The spot checks' channel a: no NaN.
const A: [f64; 10] = [0.0, 5.0, -1.0, 3.0, 1.0, 2.0, 2.0, 8.0, 1.0, 4.0];

/// Column edges in samples and M4's polylines for data-space `edges`.
fn thinned(
    values: &[f64],
    edges: &[f64],
    valid: impl Fn(f64) -> bool,
) -> (Vec<usize>, Vec<Vec<usize>>) {
    let rec = regular(0.0, 1.0, &[values]);
    let x = xs(&rec);
    let thinned = thinned_by_index(&rec, edges, &valid);
    assert_eq!(thinned, m4_scan(&x, values, edges, &valid), "the raw scan");
    thinned
}

#[test]
fn m4_keeps_first_extremes_and_last_per_column_and_run() {
    let all = |_| true;
    let (edges, lines) = thinned(&Y, &[0.0, 5.0, 10.0], all);
    assert_eq!(
        (edges, lines),
        (vec![0, 5, 10], vec![vec![0, 1, 2, 3], vec![6, 7, 8, 9]])
    );
    let (edges, lines) = thinned(&A, &[0.0, 5.0, 10.0], all);
    assert_eq!(
        (edges, lines),
        (vec![0, 5, 10], vec![vec![0, 1, 2, 4, 5, 7, 8, 9]])
    );
}

#[test]
fn m4_reaches_one_sample_past_the_view_inside_the_run() {
    let (edges, lines) = thinned(&Y, &[1.5, 5.0, 8.5], |_| true);
    assert_eq!(
        (edges, lines),
        (vec![2, 5, 9], vec![vec![1, 2, 3], vec![6, 7, 8, 9]])
    );
    let (edges, lines) = thinned(&A, &[3.2, 3.4], |_| true);
    assert_eq!((edges, lines), (vec![4, 4], vec![vec![3, 4]]));
    let (edges, lines) = thinned(&Y, &[4.2, 4.8], |_| true);
    assert_eq!((edges, lines), (vec![5, 5], vec![]));
    let (edges, lines) = thinned(&Y, &[3.5, 3.6], |_| true);
    assert_eq!((edges, lines), (vec![4, 4], vec![]));
}

#[test]
fn m4_filters_extremes_but_not_a_columns_first_and_last_samples() {
    let (_, lines) = thinned(&Y, &[0.0, 5.0, 10.0], |v| v > 0.0);
    assert_eq!(lines, [vec![0, 1, 3], vec![6, 7, 8, 9]]);
    let (_, lines) = thinned(&Y, &[0.0, 5.0, 10.0], |v| v > 100.0);
    assert_eq!(lines, [vec![0, 3], vec![6, 9]]);
}

#[test]
fn column_edges_are_clamped_by_locate() {
    let (edges, lines) = thinned(&A, &[-1.0, 20.0], |_| true);
    assert_eq!((edges, lines), (vec![0, 10], vec![vec![0, 2, 7, 9]]));
}

#[test]
fn a_column_of_one_sample_emits_that_sample() {
    let (edges, lines) = thinned(&A, &[0.0, 0.5, 1.5, 10.0], |_| true);
    assert_eq!(
        (edges, lines),
        (vec![0, 1, 2, 10], vec![vec![0, 1, 2, 7, 9]])
    );
}

#[test]
fn extent_and_extremes_ignore_nan() {
    let rec = regular(0.0, 1.0, &[&Y]);
    let s = rec.series(0).expect("channel 0");
    assert_eq!(s.extent(), Some([-1.0, 8.0]));
    let found = s.extremes(0..10, |_| true).expect("values");
    assert_eq!((found.get_min(), found.get_max()), ((2, -1.0), (7, 8.0)));
    assert_eq!(s.extremes(4..6, |_| true), None);
    assert_eq!(s.extremes(3..5, |v| v < 0.0), None);
    let found = s.extremes(8..100, |_| true).expect("clamped");
    assert_eq!((found.get_min(), found.get_max()), ((8, 1.0), (9, 4.0)));
    assert_eq!(s.extremes(20..30, |_| true), None);
    let nan = regular(0.0, 1.0, &[&[f64::NAN; 300]]);
    assert_eq!(nan.series(0).expect("channel 0").extent(), None);
}

#[test]
fn ties_pick_the_earliest_sample_in_raw_scans_and_nodes() {
    let mut values = vec![5.0; 1000];
    for i in [130, 70, 600] {
        values[i] = -1.0;
    }
    for i in [900, 500, 64] {
        values[i] = 9.0;
    }
    let rec = regular(0.0, 1.0, &[&values]);
    let s = rec.series(0).expect("channel 0");
    let picks = |samples| {
        let found = s.extremes(samples, |_| true).expect("values");
        (found.get_min(), found.get_max())
    };
    assert_eq!(picks(0..1000), ((70, -1.0), (64, 9.0)));
    assert_eq!(picks(71..1000), ((130, -1.0), (500, 9.0)));
    assert_eq!(picks(131..899), ((600, -1.0), (500, 9.0)));
    assert_eq!(picks(601..1000), ((601, 5.0), (900, 9.0)));
    assert_eq!(picks(1..64), ((1, 5.0), (1, 5.0)));
    let zeros = regular(0.0, 1.0, &[&[0.0, -0.0, 0.0]]);
    let found = zeros.series(0).expect("channel 0").extremes(0..3, |_| true);
    assert_eq!(found.map(|f| (f.get_min().0, f.get_max().0)), Some((0, 0)));
}

#[test]
fn infinite_samples_are_values_not_gaps() {
    let values = [1.0, f64::INFINITY, f64::NEG_INFINITY, 2.0];
    let rec = regular(0.0, 1.0, &[&values]);
    let s = rec.series(0).expect("channel 0");
    assert_eq!(s.extent(), Some([f64::NEG_INFINITY, f64::INFINITY]));
    assert!(s.runs(0..4).eq(std::iter::once(0..4)));
    let (_, lines) = thinned(&values, &[0.0, 4.0], |v| v.is_finite());
    assert_eq!(lines, [vec![0, 3]]);
}

#[test]
fn runs_are_the_nan_free_ranges_of_the_samples_asked_for() {
    let rec = regular(0.0, 1.0, &[&Y]);
    let s = rec.series(0).expect("channel 0");
    assert_eq!(s.runs(0..10).collect::<Vec<_>>(), [0..4, 6..10]);
    assert_eq!(s.runs(2..7).collect::<Vec<_>>(), [2..4, 6..7]);
    assert_eq!(s.runs(4..6).count(), 0);
    assert!(s.runs(7..100).eq(std::iter::once(7..10)));
}

#[test]
fn a_series_reads_its_channel_on_the_recordings_clock() {
    let rec = Recording::new(
        Clock::Explicit(vec![0.5, 1.5, 4.0]),
        [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
    )
    .expect("valid recording");
    let s = rec.series(1).expect("channel 1");
    assert!(rec.series(2).is_none());
    assert_eq!((s.get_channel(), s.len(), s.is_empty()), (1, 3, false));
    assert_eq!(s.get_recording().get_id(), rec.get_id());
    assert_eq!((s.get(2), s.get(3)), (Some((4.0, 6.0)), None));
    assert_eq!(
        s.iter().collect::<Vec<_>>(),
        [(0.5, 4.0), (1.5, 5.0), (4.0, 6.0)]
    );
}

#[test]
fn column_edges_must_number_two_and_be_valid_and_ordered() {
    let rec = regular(0.0, 1.0, &[&A]);
    assert_eq!(rec.columns(&[]), Err(Error::Edges(0)));
    assert_eq!(rec.columns(&[1.0]), Err(Error::Edges(1)));
    assert_eq!(
        rec.columns(&[2.0, 1.0]),
        Err(Error::Edge {
            index: 1,
            value: 1.0
        })
    );
    assert_eq!(
        rec.columns(&[0.0, f64::INFINITY]),
        Err(Error::Edge {
            index: 1,
            value: f64::INFINITY
        })
    );
    assert!(
        matches!(rec.columns(&[f64::NAN, 1.0]), Err(Error::Edge { index: 0, value }) if value.is_nan())
    );
    let columns = rec.columns(&[1.0, 1.0, 5.0]).expect("valid edges");
    assert_eq!((columns.len(), columns.samples()), (2, 1..5));
}

#[test]
fn columns_from_another_revision_or_recording_are_stale() {
    let mut rec = regular(0.0, 1.0, &[&A]);
    let columns = rec.columns(&[0.0, 10.0]).expect("valid edges");
    let other = regular(0.0, 1.0, &[&A]);
    let err = other.series(0).expect("channel 0").m4(&columns, |_| true);
    assert_eq!(
        err,
        Err(Error::Stale {
            recording: rec.get_id(),
            built: 0,
            current: 0
        })
    );
    rec.extend(&[], [[1.0]]).expect("an append");
    let err = rec.series(0).expect("channel 0").m4(&columns, |_| true);
    assert_eq!(
        err,
        Err(Error::Stale {
            recording: rec.get_id(),
            built: 0,
            current: 1
        })
    );
    let columns = rec.columns(&[0.0, 10.0]).expect("valid edges");
    assert!(
        rec.series(0)
            .expect("channel 0")
            .m4(&columns, |_| true)
            .is_ok()
    );
}

/// The joint spot checks' second channel b: one NaN, at 3.
const B: [f64; 10] = [2.0, 2.0, 2.0, f64::NAN, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0];

#[test]
fn joint_m4_splits_where_either_channel_has_a_gap() {
    let rec = regular(0.0, 1.0, &[&A, &B]);
    let (a, b) = (rec.series(0).expect("a"), rec.series(1).expect("b"));
    let columns = rec.columns(&[0.0, 5.0, 10.0]).expect("valid edges");
    let lines = a.m4_joint(b, &columns, |_| true).expect("one recording");
    let lines: Vec<Vec<usize>> = lines.iter().map(<[usize]>::to_vec).collect();
    assert_eq!(lines, [vec![0, 1, 2], vec![4, 5, 7, 8, 9]]);
    let x = xs(&rec);
    assert_eq!(
        scan(&x, &[&A, &B], &[0.0, 5.0, 10.0], |_| true).1,
        lines,
        "the raw scan"
    );
    let reversed = b.m4_joint(a, &columns, |_| true).expect("one recording");
    assert_eq!(
        reversed.iter().map(<[usize]>::to_vec).collect::<Vec<_>>(),
        lines
    );
}

#[test]
fn joint_m4_agrees_with_a_raw_scan_of_both_channels() {
    let mut lcg = Lcg(34);
    for case in 0..40 {
        let n = 50 + (lcg.next() * 3000.0) as usize;
        let mut channel = |gaps: f64| -> Vec<f64> {
            (0..n)
                .map(|_| {
                    let v = lcg.next() * 2.0 - 1.0;
                    if lcg.next() < gaps { f64::NAN } else { v }
                })
                .collect()
        };
        let (lower, upper) = (channel(0.02), channel(0.05));
        let rec = regular(0.0, 1.0, &[&lower, &upper]);
        let x = xs(&rec);
        let count = 1 + (lcg.next() * 40.0) as usize;
        let (from, to) = (lcg.next() * n as f64 * 0.5, n as f64 * (0.5 + lcg.next()));
        let edges: Vec<f64> = (0..=count)
            .map(|i| from + (to - from) * i as f64 / count as f64)
            .collect();
        let columns = rec.columns(&edges).expect("valid edges");
        let (a, b) = (rec.series(0).expect("a"), rec.series(1).expect("b"));
        let tests: [fn(f64) -> bool; 2] = [|v| v > -2.0, |v| v > 0.0];
        for valid in tests {
            let lines = a.m4_joint(b, &columns, valid).expect("one recording");
            let lines: Vec<Vec<usize>> = lines.iter().map(<[usize]>::to_vec).collect();
            let expected = scan(&x, &[&lower, &upper], &edges, valid);
            assert_eq!(
                (columns.get_edges().to_vec(), lines),
                expected,
                "case {case}"
            );
        }
    }
}

#[test]
fn joint_m4_over_two_recordings_is_an_error() {
    let rec = regular(0.0, 1.0, &[&A, &B]);
    let other = regular(0.0, 1.0, &[&B]);
    let columns = rec.columns(&[0.0, 10.0]).expect("valid edges");
    let (a, b) = (rec.series(0).expect("a"), other.series(0).expect("b"));
    assert_eq!(
        a.m4_joint(b, &columns, |_| true),
        Err(Error::Recordings {
            left: rec.get_id(),
            right: other.get_id()
        })
    );
    let stale = other.columns(&[0.0, 10.0]).expect("valid edges");
    assert!(matches!(
        a.m4_joint(rec.series(1).expect("b"), &stale, |_| true),
        Err(Error::Stale { .. })
    ));
    let message = Error::<f64>::Recordings {
        left: rec.get_id(),
        right: other.get_id(),
    }
    .to_string();
    assert!(
        message.starts_with("joint query over two recordings: "),
        "{message}"
    );
}

/// A mark over any data: what it holds.
fn mark<'a, X, Y>(data: impl Data<'a, X, Y>) -> Points<'a, X, Y> {
    data.into_points()
}

/// A band key: equality only, no order.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Bin(u8);

#[test]
fn marks_take_collected_pairs_a_series_or_slices() {
    let pairs = vec![(0.0, 1.0), (1.0, 3.0)];
    let owned = mark(pairs.clone());
    assert_eq!((owned.len(), owned.get(1)), (2, Some((1.0, 3.0))));
    assert!(owned.series().is_none());
    let copied = mark(pairs.iter().copied());
    assert_eq!(copied.iter().collect::<Vec<_>>(), pairs);

    let rec = Recording::new(
        Clock::Explicit(vec![0.5, 1.5, 4.0]),
        [[1.0, 2.0, 3.0], [4.0, f64::NAN, 6.0]],
    )
    .expect("a valid recording");
    let indexed = mark(rec.series(1).expect("channel 1"));
    assert_eq!((indexed.len(), indexed.is_empty()), (3, false));
    assert_eq!((indexed.get(2), indexed.get(3)), (Some((4.0, 6.0)), None));
    let view = indexed.series().expect("the indexed view");
    assert_eq!(
        (view.get_recording().get_id(), view.get_channel()),
        (rec.get_id(), 1)
    );
    assert_eq!(view.extent(), Some([4.0, 6.0]));

    let (x, y) = ([0.0, 1.0, 2.0], [5.0, 7.0]);
    let borrowed = mark(series::slices(&x, &y));
    assert_eq!(
        borrowed.iter().collect::<Vec<_>>(),
        [(0.0, 5.0), (1.0, 7.0)]
    );
    assert!(borrowed.series().is_none());
}

#[test]
fn mark_data_needs_no_bound_on_its_keys() {
    let bins = mark(vec![(Bin(1), "a"), (Bin(2), "b")]);
    assert_eq!(bins.get(0), Some((Bin(1), "a")));
    let (keys, labels) = ([Bin(3)], ["c"]);
    let borrowed = mark(series::slices(&keys, &labels));
    assert_eq!(borrowed.iter().collect::<Vec<_>>(), [(Bin(3), "c")]);
    let empty = mark(Vec::<(Bin, &str)>::new());
    assert!(empty.is_empty());
}

#[test]
fn keys_that_are_only_clone_are_read_by_clone() {
    let names = mark(vec![("front".to_string(), 1.0), ("rear".to_string(), 2.0)]);
    assert_eq!(names.get(1), Some(("rear".to_string(), 2.0)));
    let (keys, values) = (["a".to_string()], [3.0]);
    let borrowed = mark(series::slices(&keys, &values));
    assert_eq!(
        borrowed.iter().collect::<Vec<_>>(),
        [("a".to_string(), 3.0)]
    );
}

/// The indexed view of points whose y type is named generically.
fn indexed<'a, X, Y>(points: &Points<'a, X, Y>) -> Option<Points<'a, X, f64>> {
    points.indexed()
}

#[test]
fn the_indexed_view_needs_no_bound_and_keeps_the_channel() {
    let rec = Recording::new(
        Clock::Explicit(vec![0.5, 1.5, 4.0]),
        [[1.0, 2.0, 3.0], [4.0, f64::NAN, 6.0]],
    )
    .expect("a valid recording");
    let points = mark(rec.series(1).expect("channel 1"));
    let view = indexed(&points).expect("a series view");
    assert_eq!(view.get(2), Some((4.0, 6.0)));
    let series = view.series().expect("still indexed");
    assert_eq!(
        (series.get_recording().get_id(), series.get_channel()),
        (rec.get_id(), 1)
    );
    assert!(indexed(&mark(vec![(Bin(1), 1.0)])).is_none());
    let (x, y) = ([0.0], [1.0]);
    assert!(indexed(&mark(series::slices(&x, &y))).is_none());
}

/// A key with neither `Clone` nor `Debug`.
struct Plain;

#[test]
fn points_clone_and_print_their_kind_and_length() {
    let owned = mark(vec![(0.0, 1.0), (1.0, 3.0)]);
    let copy = owned.clone();
    assert_eq!(copy.iter().collect::<Vec<_>>(), [(0.0, 1.0), (1.0, 3.0)]);
    assert_eq!(
        format!("{owned:?}"),
        r#"Points { kind: "collected", len: 2 }"#
    );
    let (x, y) = ([0.0, 1.0, 2.0], [5.0, 7.0]);
    let borrowed = mark(series::slices(&x, &y)).clone();
    assert_eq!(
        format!("{borrowed:?}"),
        r#"Points { kind: "slices", len: 2 }"#
    );
    let rec =
        Recording::new(Clock::Explicit(vec![0.5, 1.5]), [[1.0, 2.0]]).expect("a valid recording");
    let indexed = mark(rec.series(0).expect("channel 0"));
    let view = indexed.clone().series().expect("the indexed view");
    assert_eq!(
        (view.get_recording().get_id(), view.get_channel()),
        (rec.get_id(), 0)
    );
    assert_eq!(
        format!("{indexed:?}"),
        r#"Points { kind: "series", len: 2 }"#
    );
    let plain = mark(vec![(Plain, Plain)]);
    assert_eq!(
        format!("{plain:?}"),
        r#"Points { kind: "collected", len: 1 }"#
    );
}
