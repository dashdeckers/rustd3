//! Recordings: storage, clocks and appends.

use rustd3_core::series::{Clock, Error, Recording};

/// The prototypes' generator: a 64-bit LCG, uniform in [0, 1).
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
fn xs(rec: &Recording<f64>) -> Vec<f64> {
    (0..rec.len()).map(|i| rec.x(i).expect("i < len")).collect()
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
    assert_eq!((rec.len(), rec.revision()), (1, 0));
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
        (rec.len(), rec.revision(), xs(&rec)),
        (2, 0, vec![0.0, 1.0])
    );
    rec.extend(&[1.0, 3.0], [[7.0, 8.0]]).unwrap();
    assert_eq!(
        (rec.len(), rec.revision(), xs(&rec)),
        (4, 1, vec![0.0, 1.0, 1.0, 3.0])
    );
}

#[test]
fn clones_get_fresh_ids() {
    let rec = regular(0.0, 1.0, &[&[1.0, 2.0]]);
    let clone = rec.clone();
    let other = regular(0.0, 1.0, &[&[1.0, 2.0]]);
    assert_ne!(rec.id(), clone.id());
    assert_ne!(rec.id(), other.id());
    assert_ne!(clone.id(), other.id());
    assert_eq!(
        (clone.len(), clone.revision(), xs(&clone)),
        (2, 0, xs(&rec))
    );
}

#[test]
fn the_revision_counts_appends_and_drops_that_dropped() {
    let mut rec = regular(0.0, 1.0, &[&[0.0; 100]]);
    assert_eq!(rec.revision(), 0);
    rec.extend(&[], [vec![1.0; 200_000]]).unwrap();
    rec.extend(&[], [[0.0; 0]]).unwrap();
    assert_eq!(rec.revision(), 2);
    assert_eq!(rec.drop_front(65_535), 0);
    assert_eq!(rec.revision(), 2);
    assert_eq!(rec.drop_front(65_536), 65_536);
    assert_eq!(rec.revision(), 3);
}

#[test]
fn drop_front_drops_whole_chunks_and_keeps_the_regular_numbering() {
    let mut rec = regular(0.0, 0.1, &[&vec![0.0; 200_000], &vec![1.0; 200_000]]);
    let revision = rec.revision();
    assert_eq!(rec.drop_front(150_000), 131_072);
    assert_eq!(
        (rec.len(), rec.offset(), rec.revision()),
        (68_928, 131_072, revision + 1)
    );
    assert_eq!(rec.x(0), Some(13107.2));
    assert_eq!(rec.x(68_928), None);
    assert_eq!(rec.drop_front(usize::MAX), 65_536);
    assert_eq!((rec.len(), rec.offset()), (3_392, 196_608));
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
