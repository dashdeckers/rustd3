//! Every scale kind over hostile domains, ranges, parameters, inputs and
//! counts: `apply` and `invert` never panic and return a finite value or
//! NaN where the scale clamps, ticks follow the tick rules, `nice` and
//! `tick_format` never panic, and every label formats.

use rustd3_core::jiff::tz::TimeZone;
use rustd3_core::jiff::{SignedDuration, Timestamp};
use rustd3_core::locale::Locale;
use rustd3_core::scale::{self, AnyContinuous, TickRule, Ticks, transform};
use rustd3_core::time;
use rustd3_testkit::hostile::{self, Rng};

use super::tick_violation;

const CASES: usize = 400;

/// Characters of d3-format specifiers.
const SPECIFIER: &str = " <>=^+-($#0123456789,.~eEfgGrsdx%pbocXn";

static LOCALES: [Locale; 12] = [
    Locale::NEUTRAL,
    Locale::EN_US,
    Locale::DE_DE,
    Locale::FR_FR,
    Locale::ES_ES,
    Locale::IT_IT,
    Locale::PT_BR,
    Locale::NL_NL,
    Locale::PL_PL,
    Locale::RU_RU,
    Locale::JA_JP,
    Locale::ZH_CN,
];

fn domain(rng: &mut Rng) -> [f64; 2] {
    match rng.below(6) {
        // Zero-width.
        0 => {
            let v = rng.f64();
            [v, v]
        }
        // Reversed or ordinary.
        1 => [rng.plain(), rng.plain()],
        _ => [rng.f64(), rng.f64()],
    }
}

fn finite_pair(pair: [f64; 2]) -> bool {
    pair.iter().all(|v| v.is_finite())
}

/// A continuous scale with a random transform, parameters, domain, range,
/// clamp and tick rule; its name for failures.
fn continuous(rng: &mut Rng) -> (String, AnyContinuous, [f64; 2], [f64; 2], bool) {
    let p = rng.f64();
    let (name, base): (String, AnyContinuous) = match rng.below(9) {
        0 => ("linear".into(), scale::linear().into()),
        1 => (format!("pow {p}"), scale::pow().exponent(p).into()),
        2 => ("sqrt".into(), scale::sqrt().into()),
        3 => (format!("log base {p}"), scale::log().base(p).into()),
        4 => ("log".into(), scale::log().into()),
        5 => (format!("symlog {p}"), scale::symlog().constant(p).into()),
        6 => ("logit".into(), scale::logit().into()),
        7 => (format!("asinh {p}"), scale::asinh().linear_width(p).into()),
        _ => (
            "func cbrt".into(),
            scale::func(f64::cbrt, |y| y * y * y, TickRule::Data).into(),
        ),
    };
    let d = domain(rng);
    let r = domain(rng);
    let clamp = rng.chance(0.5);
    let mut s = base.domain(d).range(r).clamp(clamp);
    let name = match rng.below(4) {
        0 => {
            s = s.tick_rule(TickRule::Data);
            format!("{name}, data rule")
        }
        1 => {
            s = s.tick_rule(TickRule::Transformed);
            format!("{name}, transformed rule")
        }
        _ => name,
    };
    (name, s, d, r, clamp)
}

#[test]
fn continuous_scales_never_panic_and_keep_their_rules() {
    hostile::run("continuous", 0xC0_17, CASES, |rng| {
        let (name, s, _, r, clamp) = continuous(rng);
        let s = if rng.chance(0.3) {
            s.nice(rng.count())
        } else {
            s
        };
        let d = s.get_domain();
        let x = rng.f64();
        let y = s.apply(x);
        let back = s.invert(rng.f64());
        if clamp && finite_pair(d) && finite_pair(r) && r.iter().all(|v| v.abs() <= 1e300) {
            assert!(
                y.is_finite() || y.is_nan(),
                "{name} {d:?} → {r:?}: apply({x}) = {y}"
            );
            assert!(
                back.is_finite() || back.is_nan(),
                "{name} {d:?} → {r:?}: invert = {back}"
            );
        }
        let count = rng.count();
        let ticks = s.ticks(count);
        if let Some(v) = tick_violation(&ticks, d) {
            panic!("{name} {d:?} ticks({count}): {v}");
        }
        s.minor(count);
        let set = s.tick_set(count);
        let minor = Ticks::minor(&s, &set);
        assert!(
            minor.iter().all(|m| m.is_finite()),
            "{name} {d:?}: minor {minor:?}"
        );
        let locale = rng.pick(&LOCALES);
        let labels = s.labels(&set, locale);
        assert_eq!(labels.len(), set.get_values().len(), "{name} {d:?}");
        for v in set.get_values() {
            s.position(v, 0.5);
        }
        let spec = rng.chance(0.5).then(|| rng.string(SPECIFIER, 8));
        if let Ok(f) = s.tick_format_in(locale.get_number(), count, spec.as_deref()) {
            let _ = f.format_all(&ticks);
            let _ = f.format(rng.f64());
        }
        s.admits(x);
    });
}

#[test]
fn band_and_point_scales_never_panic() {
    hostile::run("band", 0xBA_7D, CASES, |rng| {
        let n = rng.small(20);
        let keys: Vec<u32> = (0..n)
            .map(|_| u32::try_from(rng.below(10)).unwrap_or(0))
            .collect();
        let range = domain(rng);
        let b = scale::band(keys.clone())
            .range(range)
            .round(rng.chance(0.3))
            .padding_inner(rng.f64())
            .padding_outer(rng.f64())
            .align(rng.f64());
        let b = if rng.chance(0.3) {
            b.padding(rng.f64())
        } else {
            b
        };
        for k in keys.iter().chain([&99]) {
            b.apply(k);
        }
        let _ = b.bandwidth();
        let _ = b.step();
        let set = b.tick_set(rng.count());
        b.labels(&set, &Locale::NEUTRAL);
        for v in set.get_values() {
            b.position(v, rng.f64());
        }
        let p = scale::point(keys.clone())
            .range_round(range)
            .padding(rng.f64())
            .align(rng.f64());
        for k in keys.iter().chain([&99]) {
            p.apply(k);
        }
        let _ = p.bandwidth();
        let set = p.tick_set(rng.count());
        p.labels(&set, &Locale::NEUTRAL);
    });
}

#[test]
fn ordinal_scales_never_panic() {
    hostile::run("ordinal", 0x0D_17, CASES, |rng| {
        let keys: Vec<u8> = (0..rng.small(12))
            .map(|_| u8::try_from(rng.below(6)).unwrap_or(0))
            .collect();
        let range: Vec<f64> = rng.values(5);
        let mut o = scale::ordinal(keys.clone(), range);
        if rng.chance(0.3) {
            o = o.unknown(rng.f64());
        }
        for k in keys.iter().chain([&200]) {
            o.apply(k);
        }
        o.apply_or_insert(u8::try_from(rng.below(256)).unwrap_or(0));
        let set = o.tick_set(rng.count());
        o.labels(&set, &Locale::NEUTRAL);
        for v in set.get_values() {
            o.position(v, 0.0);
        }
        o.extent();
    });
}

#[test]
fn quantize_quantile_and_threshold_scales_never_panic() {
    hostile::run("binned", 0xB1_2E, CASES, |rng| {
        let range: Vec<u16> = (0..rng.small(8))
            .map(|_| u16::try_from(rng.below(4)).unwrap_or(0))
            .collect();
        let x = rng.f64();
        let count = rng.count();

        let d = domain(rng);
        let mut q = scale::quantize(d, range.clone());
        if rng.chance(0.3) {
            q = q.nice(rng.count());
        }
        let _ = q.apply(x);
        let _ = q.thresholds();
        for r in &range {
            q.invert_extent(r);
        }
        let ticks = q.ticks(count);
        if let Some(v) = tick_violation(&ticks, q.get_domain()) {
            panic!("quantize {:?} ticks({count}): {v}", q.get_domain());
        }
        if let Ok(f) = q.tick_format(count, Some(&rng.string(SPECIFIER, 6))) {
            let _ = f.format_all(&ticks);
        }
        let set = q.tick_set(count);
        q.labels(&set, &Locale::NEUTRAL);

        let sample = rng.values(30);
        let qq = scale::quantile(sample, range.clone());
        let _ = qq.apply(x);
        let _ = qq.quantiles();
        for r in &range {
            qq.invert_extent(r);
        }
        let set = qq.tick_set(count);
        qq.labels(&set, &Locale::NEUTRAL);

        let thresholds = rng.values(8);
        let t = scale::threshold(thresholds, range.clone());
        let _ = t.apply(x);
        for r in &range {
            t.invert_extent(r);
        }
        let set = t.tick_set(count);
        t.labels(&set, &Locale::NEUTRAL);
        for v in set.get_values() {
            t.position(v, 0.0);
        }
    });
}

#[test]
fn sequential_and_diverging_scales_never_panic() {
    hostile::run("sequential", 0x5E_90, CASES, |rng| {
        let d = domain(rng);
        let clamp = rng.chance(0.5);
        let p = rng.f64();
        let count = rng.count();
        let x = rng.f64();
        let s = scale::sequential(d, |t| t).clamp(clamp);
        let checks = |name: &str, y: f64, ticks: Vec<f64>, d: [f64; 2]| {
            if clamp && finite_pair(d) {
                assert!(
                    y.is_finite() || y.is_nan(),
                    "{name} {d:?}: apply({x}) = {y}"
                );
            }
            if let Some(v) = tick_violation(&ticks, d) {
                panic!("{name} {d:?} ticks({count}): {v}");
            }
        };
        let s = if rng.chance(0.3) { s.nice(count) } else { s };
        checks("sequential", s.apply(x), s.ticks(count), s.get_domain());
        let set = s.tick_set(count);
        s.labels(&set, &Locale::NEUTRAL);
        Ticks::minor(&s, &set);
        if let Ok(f) = s.tick_format(count, None) {
            let _ = f.format_all(&s.ticks(count));
        }
        let log = s.clone().transform(transform::Log::new(10.0)).base(p);
        checks(
            "sequential log",
            log.apply(x),
            log.ticks(count),
            log.get_domain(),
        );
        log.labels(&log.tick_set(count), &Locale::NEUTRAL);
        let pow = s.clone().transform(transform::Pow::new(1.0)).exponent(p);
        checks(
            "sequential pow",
            pow.apply(x),
            pow.ticks(count),
            pow.get_domain(),
        );
        let symlog = s.transform(transform::Symlog::new(1.0)).constant(p);
        checks(
            "sequential symlog",
            symlog.apply(x),
            symlog.ticks(count),
            symlog.get_domain(),
        );

        let d3 = [rng.f64(), rng.f64(), rng.f64()];
        let dv = scale::diverging(d3, |t| t).clamp(clamp);
        let dv = if rng.chance(0.3) { dv.nice(count) } else { dv };
        let y = dv.apply(x);
        let [a, _, c] = dv.get_domain();
        if clamp && dv.get_domain().iter().all(|v| v.is_finite()) {
            assert!(
                y.is_finite() || y.is_nan(),
                "diverging {d3:?}: apply({x}) = {y}"
            );
        }
        let ticks = dv.ticks(count);
        if let Some(v) = tick_violation(&ticks, [a, c]) {
            panic!("diverging {:?} ticks({count}): {v}", dv.get_domain());
        }
        let set = dv.tick_set(count);
        dv.labels(&set, &Locale::NEUTRAL);
        Ticks::minor(&dv, &set);
        if let Ok(f) = dv.tick_format(count, None) {
            let _ = f.format_all(&ticks);
        }
        let log = dv.transform(transform::Log::new(10.0)).base(p);
        log.apply(x);
        log.ticks(count);
    });
}

#[test]
fn identity_scales_never_panic() {
    hostile::run("identity", 0x1D_E7, CASES, |rng| {
        let d = domain(rng);
        let count = rng.count();
        let s = scale::identity().domain(d);
        let s = if rng.chance(0.3) { s.nice(count) } else { s };
        let _ = s.apply(rng.f64());
        let _ = s.invert(rng.f64());
        let ticks = s.ticks(count);
        if let Some(v) = tick_violation(&ticks, s.get_domain()) {
            panic!("identity {:?} ticks({count}): {v}", s.get_domain());
        }
        if let Ok(f) = s.tick_format(count, Some(&rng.string(SPECIFIER, 6))) {
            let _ = f.format_all(&ticks);
        }
    });
}

/// A timestamp anywhere in jiff's range, at its ends, or near now.
fn instant(rng: &mut Rng) -> Timestamp {
    let min = Timestamp::MIN.as_nanosecond();
    let max = Timestamp::MAX.as_nanosecond();
    let ns = match rng.below(5) {
        0 => min,
        1 => max,
        2 => 1_700_000_000_000_000_000 + i128::from(rng.bits() % 1_000_000_000_000_000),
        3 => 1_700_000_000_000_000_000 + i128::from(rng.bits() % 1_000_000),
        _ => min + i128::from(rng.bits()) * ((max - min) / i128::from(u64::MAX)),
    };
    Timestamp::from_nanosecond(ns.clamp(min, max)).unwrap_or(Timestamp::UNIX_EPOCH)
}

fn zone(rng: &mut Rng) -> TimeZone {
    match rng.below(4) {
        0 => TimeZone::UTC,
        1 => rustd3_testkit::zone("Asia/Kathmandu"),
        2 => rustd3_testkit::zone("America/St_Johns"),
        _ => rustd3_testkit::zone("Pacific/Kiritimati"),
    }
}

#[test]
fn time_scales_never_panic_and_keep_their_rules() {
    hostile::run("time", 0x71_3E, CASES, |rng| {
        let start = instant(rng);
        let end = if rng.chance(0.2) {
            start
        } else if rng.chance(0.3) {
            start
                .checked_add(SignedDuration::from_nanos(
                    i64::try_from(rng.below(1_000_000)).unwrap_or(0),
                ))
                .unwrap_or(start)
        } else {
            instant(rng)
        };
        let zone = zone(rng);
        let utc = rng.chance(0.3);
        let base = if utc { time::utc() } else { time::time(zone) };
        let s = base
            .domain([start, end])
            .range(domain(rng))
            .clamp(rng.chance(0.5));
        let count = rng.count();
        let s = if rng.chance(0.3) { s.nice(count) } else { s };
        let [a, b] = s.get_domain();
        let _ = s.apply(instant(rng));
        let _ = s.invert(rng.f64());
        let ticks = s.ticks(count);
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        let ordered = ticks
            .windows(2)
            .all(|w| if a <= b { w[0] < w[1] } else { w[0] > w[1] });
        assert!(ordered, "time {a}..{b} ticks({count}) out of order");
        assert!(
            ticks.iter().all(|t| lo <= *t && *t <= hi),
            "time {a}..{b} ticks({count}) outside the domain"
        );
        let set = s.tick_set(count);
        let locale = rng.pick(&LOCALES);
        let labels = s.labels(&set, locale);
        assert_eq!(labels.len(), set.get_values().len());
        for t in set.get_values().iter().take(50) {
            let _ = s.tick_label(*t, locale.get_time());
            s.position(t, 0.5);
        }
        let _ = s.tick_interval(count);
    });
}
