//! `anim`: the update pattern's lifecycle trace (100 ms transitions, 20 ms
//! stagger, cubic in-out), and `Animated`'s and `Marks`' own rules, at
//! fixed offsets from one base instant.

use std::fmt::Write;
use std::time::{Duration, Instant};

use rustd3_core::anim::{Animated, Marks, Phase, Timing};
use rustd3_core::ease::{Ease, Mode};
use rustd3_core::interpolate::Interpolate;

/// The update pattern's mark: everything that animates is a field.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Bar {
    y: f64,
    width: f64,
    opacity: f64,
    value: f64,
}

impl Interpolate for Bar {
    fn lerp(&self, to: &Self, t: f64) -> Self {
        let f = |a: f64, b: f64| a.lerp(&b, t);
        Bar {
            y: f(self.y, to.y),
            width: f(self.width, to.width),
            opacity: f(self.opacity, to.opacity),
            value: f(self.value, to.value),
        }
    }
}

/// The base instant the tests offset from.
#[allow(
    clippy::disallowed_methods,
    reason = "a base instant for offsets; values do not depend on it"
)]
fn base() -> Instant {
    Instant::now()
}

/// The instant `n` ms after `t0`.
fn ms(t0: Instant, n: u64) -> Instant {
    t0 + Duration::from_millis(n)
}

/// The update pattern's targets for the top bars `(name, value)`: a row each,
/// width = value (an identity x scale), opaque.
fn bars<'a>(data: &[(&'a str, f64)]) -> Vec<(&'a str, Bar)> {
    data.iter()
        .enumerate()
        .map(|(row, &(name, value))| {
            let bar = Bar {
                y: 50.0 * row as f64,
                width: value,
                opacity: 1.0,
                value,
            };
            (name, bar)
        })
        .collect()
}

/// The marks' widths at `now`, as `a=7.4(exit)`.
fn widths(marks: &Marks<&str, Bar>, now: Instant) -> String {
    marks
        .values(now)
        .map(|(name, bar, phase)| {
            let exit = match phase {
                Phase::Present => "",
                Phase::Exiting => "(exit)",
            };
            format!("{name}={:.1}{exit}", bar.width)
        })
        .collect::<Vec<_>>()
        .join("  ")
}

#[test]
fn use_case_2_lifecycle() {
    let t0 = base();
    let timing = Timing {
        duration: Duration::from_millis(100),
        stagger: Duration::from_millis(20),
        ..Timing::default()
    };
    // Enter: grow from zero width in its row; exit: shrink and fade where
    // it is.
    let enter = |t: &Bar| Bar { width: 0.0, ..*t };
    let exit = |b: &Bar| Bar {
        width: 0.0,
        opacity: 0.0,
        ..*b
    };
    let mut marks = Marks::default();
    let mut trace = String::new();
    let mut sample = |marks: &Marks<&str, Bar>, n: u64| {
        let now = ms(t0, n);
        assert_eq!(marks.is_animating(now), n < 180, "is_animating at {n} ms");
        writeln!(trace, "t={n:>3}ms  {}", widths(marks, now)).unwrap();
    };

    marks.update(
        bars(&[("a", 10.0), ("b", 20.0)]),
        ms(t0, 0),
        timing,
        enter,
        exit,
    );
    sample(&marks, 0);
    sample(&marks, 50);
    marks.update(
        bars(&[("b", 40.0), ("c", 30.0)]),
        ms(t0, 60),
        timing,
        enter,
        exit,
    );
    sample(&marks, 60);
    sample(&marks, 110);
    sample(&marks, 180);
    marks.prune(ms(t0, 180));
    let now = ms(t0, 180);
    let animating = if marks.is_animating(now) {
        ", animating"
    } else {
        ", nothing animating"
    };
    writeln!(
        trace,
        "after prune at 180 ms:\n{}{animating}",
        widths(&marks, now)
    )
    .unwrap();

    assert_eq!(
        trace,
        "\
t=  0ms  a=0.0  b=0.0
t= 50ms  a=5.0  b=2.2
t= 60ms  a=7.4(exit)  b=5.1  c=0.0
t=110ms  a=3.7(exit)  b=22.6  c=3.2
t=180ms  a=0.0(exit)  b=40.0  c=30.0
after prune at 180 ms:
b=40.0  c=30.0, nothing animating
"
    );
}

#[test]
fn the_lifecycle_rows_follow_cubic_in_out_exactly() {
    // cubic in-out(0.3) = 0.108 and (0.6) = 0.744; b at 110 is
    // 5.12 + (40 − 5.12) · 0.5.
    let t0 = base();
    let timing = Timing {
        duration: Duration::from_millis(100),
        stagger: Duration::from_millis(20),
        ..Timing::default()
    };
    let mut marks = Marks::default();
    marks.update(
        [("a", 10.0), ("b", 20.0)],
        ms(t0, 0),
        timing,
        |_| 0.0,
        |_| 0.0,
    );
    let at = |marks: &Marks<&str, f64>, n| -> Vec<f64> {
        marks.values(ms(t0, n)).map(|(_, value, _)| value).collect()
    };
    let close = |actual: &[f64], expected: &[f64]| {
        assert_eq!(actual.len(), expected.len());
        for (a, e) in actual.iter().zip(expected) {
            assert!((a - e).abs() < 1e-12, "{actual:?} against {expected:?}");
        }
    };
    close(&at(&marks, 50), &[5.0, 2.16]);
    close(&at(&marks, 60), &[7.44, 5.12]);
    marks.update(
        [("b", 40.0), ("c", 30.0)],
        ms(t0, 60),
        timing,
        |_| 0.0,
        |_| 0.0,
    );
    close(&at(&marks, 110), &[3.72, 22.56, 3.24]);
}

#[test]
fn the_default_timing_is_d3s_transition() {
    let timing = Timing::default();
    assert_eq!(timing.duration, Duration::from_millis(250));
    assert_eq!(timing.delay, Duration::ZERO);
    assert_eq!(timing.stagger, Duration::ZERO);
    assert_eq!(timing.ease, Ease::Cubic(Mode::InOut));
}

#[test]
fn a_new_value_rests_and_a_delayed_one_waits_where_it_was() {
    let t0 = base();
    let mut value = Animated::new(4.0, ms(t0, 0));
    assert_eq!(value.value(ms(t0, 0)), 4.0);
    assert!(!value.is_animating(ms(t0, 0)));
    let timing = Timing {
        duration: Duration::from_millis(100),
        delay: Duration::from_millis(30),
        ease: Ease::Linear,
        ..Timing::default()
    };
    value.go(8.0, ms(t0, 10), timing, Duration::from_millis(10));
    assert_eq!(*value.target(), 8.0);
    assert_eq!(value.value(ms(t0, 49)), 4.0);
    assert!(value.is_animating(ms(t0, 49)));
    assert_eq!(value.value(ms(t0, 100)), 6.0);
    assert_eq!(value.value(ms(t0, 150)), 8.0);
    assert!(value.is_animating(ms(t0, 149)));
    assert!(!value.is_animating(ms(t0, 150)));
}

#[test]
fn a_zero_duration_jumps_at_its_start() {
    let t0 = base();
    let mut value = Animated::new(0.0, ms(t0, 0));
    let timing = Timing {
        duration: Duration::ZERO,
        delay: Duration::from_millis(20),
        ..Timing::default()
    };
    value.go(1.0, ms(t0, 0), timing, Duration::ZERO);
    assert_eq!(value.value(ms(t0, 19)), 0.0);
    assert!(value.is_animating(ms(t0, 19)));
    assert_eq!(value.value(ms(t0, 20)), 1.0);
    assert!(!value.is_animating(ms(t0, 20)));
}

#[test]
fn extreme_delays_and_durations_do_not_panic() {
    let t0 = base();
    let mut value = Animated::new(0.0, ms(t0, 0));
    // A start past what Instant holds is now.
    let late = Timing {
        duration: Duration::from_millis(100),
        delay: Duration::MAX,
        ease: Ease::Linear,
        ..Timing::default()
    };
    value.go(10.0, ms(t0, 0), late, Duration::MAX);
    assert_eq!(value.value(ms(t0, 50)), 5.0);
    let endless = Timing {
        duration: Duration::MAX,
        ..Timing::default()
    };
    value.go(20.0, ms(t0, 100), endless, Duration::ZERO);
    assert!(value.is_animating(ms(t0, 3_600_000)));
    assert_eq!(value.value(ms(t0, 100)), 10.0);

    let mut marks = Marks::default();
    let staggered = Timing {
        stagger: Duration::MAX,
        ..Timing::default()
    };
    marks.update(
        [(1, 1.0), (2, 2.0), (3, 3.0)],
        ms(t0, 0),
        staggered,
        |_| 0.0,
        |_| 0.0,
    );
    let values: Vec<_> = marks
        .values(ms(t0, 250))
        .map(|(_, value, _)| value)
        .collect();
    assert_eq!(values, [1.0, 2.0, 3.0]);
}

#[test]
fn a_key_that_returns_while_exiting_turns_back_from_where_it_is() {
    let t0 = base();
    let linear = Timing {
        duration: Duration::from_millis(100),
        ease: Ease::Linear,
        ..Timing::default()
    };
    let mut marks = Marks::default();
    marks.update([("a", 10.0)], ms(t0, 0), linear, |_| 0.0, |_| 0.0);
    marks.update([], ms(t0, 100), linear, |_| 0.0, |&v| -v);
    let first: Vec<_> = marks.values(ms(t0, 150)).collect();
    assert_eq!(first, [(&"a", 0.0, Phase::Exiting)]);
    marks.update([("a", 20.0)], ms(t0, 150), linear, |_| 0.0, |_| 0.0);
    let back: Vec<_> = marks.values(ms(t0, 200)).collect();
    assert_eq!(back, [(&"a", 10.0, Phase::Present)]);
    marks.prune(ms(t0, 400));
    assert_eq!(marks.values(ms(t0, 400)).count(), 1);
}

#[test]
fn prune_drops_only_finished_exits() {
    let t0 = base();
    let linear = Timing {
        duration: Duration::from_millis(100),
        ease: Ease::Linear,
        ..Timing::default()
    };
    let mut marks = Marks::default();
    marks.update(
        [("a", 1.0), ("b", 2.0)],
        ms(t0, 0),
        linear,
        |_| 0.0,
        |_| 0.0,
    );
    marks.update([("b", 3.0)], ms(t0, 100), linear, |_| 0.0, |_| 0.0);
    marks.prune(ms(t0, 150));
    let keys: Vec<_> = marks.values(ms(t0, 150)).map(|(key, ..)| *key).collect();
    assert_eq!(keys, ["a", "b"]);
    marks.prune(ms(t0, 200));
    let keys: Vec<_> = marks.values(ms(t0, 200)).map(|(key, ..)| *key).collect();
    assert_eq!(keys, ["b"]);
}

#[test]
fn stagger_counts_positions_and_a_repeated_key_keeps_one_mark() {
    let t0 = base();
    let timing = Timing {
        duration: Duration::from_millis(100),
        stagger: Duration::from_millis(20),
        ease: Ease::Linear,
        ..Timing::default()
    };
    let mut marks = Marks::default();
    marks.update(
        [("a", 10.0), ("b", 10.0), ("a", 30.0), ("c", 10.0)],
        ms(t0, 0),
        timing,
        |_| 0.0,
        |_| 0.0,
    );
    // a's later target, at position 2, waits 40 ms; c, at 3, 60 ms.
    let values: Vec<_> = marks.values(ms(t0, 90)).collect();
    assert_eq!(
        values,
        [
            (&"a", 15.0, Phase::Present),
            (&"b", 7.0, Phase::Present),
            (&"c", 3.0, Phase::Present),
        ]
    );
}
