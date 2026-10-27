//! Recordings of instants: Phase 9's suite for `Timestamp` x, with a 1 kHz
//! clock from 2026-10-25T00:00:00Z and column edges from a time scale.

use rustd3_core::jiff::{SignedDuration, Timestamp};
use rustd3_core::series::{Clock, Coordinate, Error, Recording};
use rustd3_core::time;

use super::{Lcg, m4_scan, thinned_by_index, xs};

/// Samples in the long recordings.
const N: usize = 1_000_000;

fn start() -> Timestamp {
    "2026-10-25T00:00:00Z".parse().expect("an RFC 3339 instant")
}

fn millisecond() -> SignedDuration {
    SignedDuration::from_millis(1)
}

/// `t + ns` nanoseconds.
fn plus(t: Timestamp, ns: i128) -> Timestamp {
    Timestamp::from_nanosecond(t.as_nanosecond() + ns).expect("an instant in range")
}

/// A random walk with NaN gaps, so that M4 meets several runs.
fn walk(n: usize) -> Vec<f64> {
    let mut random = Lcg(27);
    let mut y = 0.0;
    (0..n)
        .map(|i| {
            y += random.next() - 0.5;
            if (i / 1_000) % 97 == 13 { f64::NAN } else { y }
        })
        .collect()
}

fn regular(y: &[f64]) -> Recording<Timestamp> {
    let clock = Clock::Regular {
        start: start(),
        period: millisecond(),
    };
    Recording::new(clock, [y]).expect("a valid recording")
}

/// Column edges at `columns + 1` device-pixel boundaries of a time scale
/// over `view`, as a figure asks for them.
fn edges(view: [Timestamp; 2], columns: usize) -> Vec<Timestamp> {
    let x = time::utc().domain(view).range([0.0, columns as f64]);
    (0..=columns).map(|px| x.invert(px as f64)).collect()
}

#[test]
fn a_regular_kilohertz_clock_and_explicit_instants_agree() {
    let y = walk(N);
    let rec = regular(&y);
    let x = xs(&rec);
    assert_eq!(x[0], start());
    assert_eq!(x[N - 1], plus(start(), 999_999_000_000));
    let explicit = Recording::new(Clock::Explicit(x.clone()), [&y]).expect("valid instants");
    let probes = [
        plus(start(), -1),
        start(),
        plus(start(), 1),
        plus(start(), 499_999_500_000),
        plus(start(), 999_999_000_000),
        plus(start(), 999_999_000_001),
    ];
    for t in probes {
        assert_eq!(rec.locate(t), explicit.locate(t), "{t}");
        assert_eq!(rec.locate(t), x.partition_point(|s| *s < t), "{t}");
    }
    for view in [
        [x[0], x[N - 1]],
        [x[N / 3], x[N / 3 + 2_000]],
        [x[10], plus(x[10], 1_500_000)],
    ] {
        let edges = edges(view, 1_000);
        let regular = thinned_by_index(&rec, &edges, |_| true);
        assert_eq!(regular, thinned_by_index(&explicit, &edges, |_| true));
        assert_eq!(regular, m4_scan(&x, &y, &edges, |_| true));
    }
}

#[test]
fn m4_over_a_time_scale_matches_a_full_scan() {
    let y = walk(N);
    let rec = regular(&y);
    let x = xs(&rec);
    let views = [
        [x[0], x[N - 1]],
        [x[0], plus(x[N - 1], 3_600_000_000_000)],
        [plus(x[0], -60_000_000_000), x[N / 2]],
        [x[123_456], x[123_456 + 50_000]],
        [plus(x[700_000], 250_000), plus(x[700_000], 1_000_250_000)],
        [plus(x[5], 100), plus(x[5], 200)],
    ];
    for view in views {
        for columns in [1_000, 2_000] {
            let edges = edges(view, columns);
            let scanned = m4_scan(&x, &y, &edges, |v| v > 0.0);
            assert_eq!(thinned_by_index(&rec, &edges, |v| v > 0.0), scanned);
        }
    }
}

#[test]
fn drop_front_and_split_appends_equal_rebuilds() {
    let y = walk(200_000);
    let mut rec = regular(&y);
    let revision = rec.revision();
    assert_eq!(rec.drop_front(150_000), 131_072);
    assert_eq!(
        (rec.len(), rec.offset(), rec.revision()),
        (68_928, 131_072, revision + 1)
    );
    assert_eq!(rec.x(0), Some(plus(start(), 131_072_000_000)));
    let kept = &y[131_072..];
    let rebuilt = Recording::new(Clock::Explicit(xs(&rec)), [kept]).expect("valid instants");
    let view = [
        rec.x(0).expect("a sample"),
        rec.x(68_927).expect("a sample"),
    ];
    let edges = edges(view, 1_000);
    assert_eq!(
        thinned_by_index(&rec, &edges, |_| true),
        thinned_by_index(&rebuilt, &edges, |_| true)
    );
    let whole = regular(&y);
    let edges = self::edges([start(), plus(start(), 200_000_000_000)], 1_000);
    let expected = thinned_by_index(&whole, &edges, |_| true);
    for pieces in [
        &[1, 65_535, 1][..],
        &[65_536, 65_536],
        &[7, 100_000, 3],
        &[199_999],
    ] {
        let mut split = regular(&[]);
        let mut explicit = Recording::new(Clock::Explicit(Vec::new()), [[0.0; 0]]).expect("empty");
        let mut done = 0;
        for &n in pieces
            .iter()
            .chain([&(200_000 - pieces.iter().sum::<usize>())])
        {
            let piece = &y[done..done + n];
            split.extend(&[], [piece]).expect("a valid append");
            let instants: Vec<Timestamp> = (done..done + n)
                .map(|i| plus(start(), i as i128 * 1_000_000))
                .collect();
            explicit.extend(&instants, [piece]).expect("a valid append");
            done += n;
        }
        assert_eq!(
            thinned_by_index(&split, &edges, |_| true),
            expected,
            "{pieces:?}"
        );
        assert_eq!(
            thinned_by_index(&explicit, &edges, |_| true),
            expected,
            "{pieces:?}"
        );
    }
}

#[test]
fn instants_past_jiffs_range_and_empty_periods_are_rejected() {
    let near_end = plus(Timestamp::MAX, -2_500_000_000);
    let second = SignedDuration::from_secs(1);
    let clock = |start| Clock::Regular {
        start,
        period: second,
    };
    let mut rec = Recording::new(clock(near_end), [[0.0; 3]]).expect("three seconds fit");
    assert_eq!(
        rec.extend(&[], [[0.0]]),
        Err(Error::Invalid {
            index: 3,
            value: Timestamp::MAX
        })
    );
    assert_eq!(rec.len(), 3);
    assert_eq!(
        Recording::new(clock(Timestamp::MAX), [[0.0; 1]]).err(),
        Some(Error::Invalid {
            index: 0,
            value: Timestamp::MAX
        })
    );
    for period in [SignedDuration::ZERO, SignedDuration::from_nanos(-1)] {
        let clock = Clock::Regular {
            start: start(),
            period,
        };
        assert_eq!(
            Recording::new(clock, [[0.0; 1]]).err(),
            Some(Error::Period(period))
        );
    }
    let explicit = Recording::new(Clock::Explicit(vec![start(), Timestamp::MAX]), [[0.0; 2]]);
    assert_eq!(
        explicit.err(),
        Some(Error::Invalid {
            index: 1,
            value: Timestamp::MAX
        })
    );
    assert!(Timestamp::MIN.is_valid() && !Timestamp::MAX.is_valid());
}

#[test]
fn the_index_is_the_exact_first_sample_at_or_after_a_value() {
    let (t0, ms) = (start(), millisecond());
    assert_eq!(Timestamp::index(t0, ms, plus(t0, -5)), 0);
    assert_eq!(Timestamp::index(t0, ms, t0), 0);
    assert_eq!(Timestamp::index(t0, ms, plus(t0, 1)), 1);
    assert_eq!(Timestamp::index(t0, ms, plus(t0, 1_000_000)), 1);
    assert_eq!(Timestamp::index(t0, ms, plus(t0, 1_000_001)), 2);
    let nano = SignedDuration::from_nanos(1);
    assert_eq!(
        Timestamp::index(Timestamp::MIN, nano, Timestamp::MAX),
        u64::MAX
    );
    assert_eq!(Timestamp::at(t0, ms, 1_000), plus(t0, 1_000_000_000));
    assert_eq!(Timestamp::at(t0, ms, u64::MAX), Timestamp::MAX);
    assert_eq!(
        Timestamp::at(t0, SignedDuration::MAX, u64::MAX),
        Timestamp::MAX
    );
    assert!(Timestamp::is_valid_period(nano) && !Timestamp::is_valid_period(SignedDuration::ZERO));
}
