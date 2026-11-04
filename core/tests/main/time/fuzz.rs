//! Time scales over hostile domains, counts and zones: ticks, nice,
//! ticks with a stepped interval and the pixel gestures, each case under
//! a 1 s timeout optimized, from a fixed seed. 150 cases by default and
//! 3,000 with `RUSTD3_FUZZ=1`, in release
//! (`cargo test --release -p rustd3-core --test main fuzz`).

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use rustd3_core::interact::Rescale;
use rustd3_core::jiff::Timestamp;
use rustd3_core::jiff::civil::Weekday;
use rustd3_core::time::{self, Interval, Time};
use rustd3_testkit::zone;

use crate::fuzz::Xorshift;

/// How long one case may take: 1 s optimized; unoptimized code runs some
/// 15 times slower, and a stepped week over 6,700 years takes 1.4 s there.
const PER_CASE: Duration = Duration::from_secs(if cfg!(debug_assertions) { 15 } else { 1 });
/// The most ticks any call returns.
const CAP: usize = 10_000;
/// The counts every case picks from.
const COUNTS: [usize; 10] = [0, 1, 2, 4, 10, 37, 1000, 100_000, 1_000_000, usize::MAX];
/// Zones with daylight saving time, offsets off the hour, a skipped
/// midnight and a 30-minute shift; `None` is `time::utc()`.
const ZONES: [Option<&str>; 7] = [
    None,
    Some("UTC"),
    Some("America/Los_Angeles"),
    Some("Europe/Berlin"),
    Some("Asia/Kathmandu"),
    Some("America/Sao_Paulo"),
    Some("Australia/Lord_Howe"),
];

/// What the fuzz thread reports.
enum Event {
    /// A case starts; its inputs.
    Case(String),
    /// The running case broke a property.
    Violation(String),
    /// Every case ran.
    Done,
}

impl Xorshift {
    /// An instant: jiff's ends and their neighbours, transitions, an
    /// offset from `near` of nanoseconds to centuries, or anywhere.
    fn instant(&mut self, near: Timestamp) -> Timestamp {
        const FIXED: [&str; 6] = [
            "1970-01-01T00:00:00Z",
            "2026-10-25T00:59:59.999999999Z",
            "2026-03-29T01:00:00Z",
            "2016-10-16T03:00:00Z",
            "0001-01-01T00:00:00Z",
            "2026-10-25T12:00:00.000000001Z",
        ];
        let (min, max) = (
            Timestamp::MIN.as_nanosecond(),
            Timestamp::MAX.as_nanosecond(),
        );
        let ns = match self.below(12) {
            0 => min,
            1 => min + 1,
            2 => max,
            3 => max - 1,
            4 => FIXED[self.below(FIXED.len())]
                .parse::<Timestamp>()
                .expect("an RFC 3339 instant")
                .as_nanosecond(),
            5 | 6 => near.as_nanosecond() + (self.unit() * 2_000.0 - 1_000.0) as i128,
            7 => near.as_nanosecond() + ((self.unit() * 2.0 - 1.0) * 8.64e13) as i128,
            8 => near.as_nanosecond() + ((self.unit() * 2.0 - 1.0) * 3.2e18) as i128,
            _ => min + (self.unit() * (max - min) as f64) as i128,
        };
        Timestamp::from_nanosecond(ns.clamp(min, max)).expect("clamped to jiff's range")
    }

    /// A unit with a step from 0 to 60, as d3's `every`.
    fn interval(&mut self) -> Interval {
        let k = u32::try_from(self.below(61)).expect("a step below 61");
        let units = [
            Interval::Nanosecond(k),
            Interval::Microsecond(k),
            Interval::Millisecond(k),
            Interval::Second(k),
            Interval::Minute(k),
            Interval::Hour(k),
            Interval::Day(k),
            Interval::UnixDay(k),
            Interval::Week(Weekday::Thursday, k),
            Interval::Month(k),
            Interval::Year(k),
        ];
        units[self.below(units.len())]
    }

    /// A pixel position or factor, finite or not.
    fn pixels(&mut self) -> f64 {
        const FIXED: [f64; 8] = [0.0, -0.0, 1e-300, 1e300, f64::INFINITY, f64::NAN, 2.0, 0.5];
        match self.below(FIXED.len() + 1) {
            i if i < FIXED.len() => FIXED[i],
            _ => (self.unit() * 2.0 - 1.0) * 2_000.0,
        }
    }
}

/// Why `ticks` break a property over `x`'s domain, if they do: more than
/// `bound`, out of the domain or its order, or not boundaries of
/// `interval` in the scale's zone.
fn violation(x: &Time, ticks: &[Timestamp], bound: usize, interval: Interval) -> Option<String> {
    if ticks.len() > bound {
        return Some(format!("{} ticks, more than {bound}", ticks.len()));
    }
    let [d0, d1] = x.get_domain();
    let (low, high) = if d1 < d0 { (d1, d0) } else { (d0, d1) };
    if let Some(t) = ticks.iter().find(|&&t| t < low || t > high) {
        return Some(format!("{t} outside the domain"));
    }
    let ordered = ticks.windows(2).all(|pair| {
        if d1 < d0 {
            pair[1] < pair[0]
        } else {
            pair[0] < pair[1]
        }
    });
    if !ordered {
        return Some("ticks out of domain order".to_owned());
    }
    let zone = x.get_zone();
    let off = ticks.iter().find(|&&t| interval.floor(t, zone) != Some(t));
    off.map(|t| format!("{t} is no boundary of {interval:?}"))
}

/// Why `moved`, a gesture's result on `x`, breaks a property, if it does:
/// a span under 1 ns or against the domain's order.
fn gesture(x: &Time, moved: Option<Time>) -> Option<String> {
    let [d0, d1] = moved?.get_domain();
    let [o0, o1] = x.get_domain();
    let flipped = (d1 < d0) != (o1 < o0);
    (d0 == d1 || flipped).then(|| format!("a gesture gave {d0} … {d1}"))
}

/// One case's checks; the first violation.
fn check(x: &Time, count: usize, interval: Interval, rng: &mut Xorshift) -> Option<String> {
    let tick_interval = x.tick_interval(count);
    let bound = CAP.min(count.saturating_mul(10));
    if let Some(v) = violation(x, &x.ticks(count), bound, tick_interval) {
        return Some(format!("ticks({count}), {tick_interval:?}: {v}"));
    }
    if let Some(v) = violation(x, &x.ticks_with(interval), CAP, interval) {
        return Some(format!("ticks_with: {v}"));
    }
    let nice = x.clone().nice(count).get_domain();
    let [d0, d1] = x.get_domain();
    let outward = if d1 < d0 {
        nice[0] >= d0 && nice[1] <= d1
    } else {
        nice[0] <= d0 && nice[1] >= d1
    };
    if !outward {
        return Some(format!("nice({count}) gave {} … {}", nice[0], nice[1]));
    }
    let (anchor, factor, dx) = (rng.pixels(), rng.pixels(), rng.pixels());
    gesture(x, x.zoom_px(anchor, factor))
        .map(|v| format!("zoom_px({anchor}, {factor}): {v}"))
        .or_else(|| gesture(x, x.pan_px(dx)).map(|v| format!("pan_px({dx}): {v}")))
}

/// Runs `cases` cases, reporting each to `events`.
fn run(cases: usize, events: &mpsc::Sender<Event>) {
    let mut rng = Xorshift(0x2027_0B27_D3D3_F00D);
    let report = |event| events.send(event).is_ok();
    for case in 0..cases {
        let name = ZONES[rng.below(ZONES.len())];
        let base = name.map_or_else(time::utc, |name| time::time(zone(name)));
        let start = rng.instant(Timestamp::UNIX_EPOCH);
        let domain = [start, rng.instant(start)];
        let count = COUNTS[rng.below(COUNTS.len())];
        let interval = rng.interval();
        let inputs = format!("case {case}: {name:?}, {domain:?}, count {count}, {interval:?}");
        if !report(Event::Case(inputs)) {
            return;
        }
        let x = base.domain(domain).range([0.0, 960.0]);
        if let Some(v) = check(&x, count, interval, &mut rng) {
            report(Event::Violation(v));
            return;
        }
    }
    report(Event::Done);
}

#[test]
fn time_ticks_hold_their_properties_on_hostile_domains() {
    let cases = match std::env::var("RUSTD3_FUZZ").as_deref() {
        Ok("1") => 3_000,
        Ok(_) | Err(_) => 150,
    };
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || run(cases, &sender));
    let mut current = String::from("before the first case");
    loop {
        match receiver.recv_timeout(PER_CASE) {
            Ok(Event::Case(inputs)) => current = inputs,
            Ok(Event::Violation(v)) => panic!("{current}: {v}"),
            Ok(Event::Done) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => panic!("{current} took over {PER_CASE:?}"),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                panic!("{current}: the fuzz thread panicked")
            }
        }
    }
}
