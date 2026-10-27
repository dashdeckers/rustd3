//! The built-in tick rules over hostile domains and counts: every built-in
//! transform with its own rule and with the `Data` and `Transformed` rules,
//! with the axis's minor ticks and labels of each tick set, from a fixed
//! seed, under a 60 s timeout. 2,000 cases by default and
//! 20,000 with `RUSTD3_FUZZ=1` (run in release by `scripts/check.sh`).

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use rustd3_core::locale::Locale;
use rustd3_core::scale::{self, AnyContinuous, TickRule, Ticks};

/// How long the cases may take together.
const BUDGET: Duration = Duration::from_secs(60);
/// The most values a built-in rule returns at any count.
const CAP: usize = 10_000;
/// The counts every case picks from.
const COUNTS: [usize; 9] = [0, 1, 2, 5, 10, 37, 1000, 1_000_000, usize::MAX];

/// Marsaglia's xorshift64: reproducible without a dependency.
struct Xorshift(u64);

impl Xorshift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// An index below `n`.
    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % n as u64).expect("an index below a usize")
    }

    /// A value in [0, 1).
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1_u64 << 53) as f64
    }

    /// A domain end: zeros, subnormals, tiny, unit, huge, extreme and
    /// non-finite values, near 1e9, or uniform in ±1e6.
    fn end(&mut self) -> f64 {
        const FIXED: [f64; 18] = [
            0.0,
            5e-324,
            -5e-324,
            1e-320,
            -1e-320,
            1e-300,
            -1e-300,
            0.5,
            -0.5,
            1.0,
            -1.0,
            1e300,
            -1e300,
            f64::MAX,
            -f64::MAX,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
        ];
        match self.below(FIXED.len() + 2) {
            i if i < FIXED.len() => FIXED[i],
            i if i == FIXED.len() => 1e9 + self.unit() * 1e-3,
            _ => (self.unit() * 2.0 - 1.0) * 1e6,
        }
    }
}

/// Every built-in transform the fuzz runs, as it is named in a failure.
fn transforms() -> Vec<(&'static str, AnyContinuous)> {
    vec![
        ("identity", scale::linear().into()),
        ("pow 2", scale::pow().exponent(2.0).into()),
        ("pow 0.5", scale::sqrt().into()),
        ("log 10", scale::log().into()),
        ("log 2", scale::log().base(2.0).into()),
        ("log e", scale::log().base(std::f64::consts::E).into()),
        ("symlog 1", scale::symlog().into()),
        ("symlog 0.01", scale::symlog().constant(0.01).into()),
        ("logit", scale::logit().into()),
        ("asinh 1", scale::asinh().into()),
        ("asinh 10", scale::asinh().linear_width(10.0).into()),
        (
            "func sqrt data",
            scale::func(f64::sqrt, |y| y * y, TickRule::Data).into(),
        ),
        (
            "func sqrt transformed",
            scale::func(f64::sqrt, |y| y * y, TickRule::Transformed).into(),
        ),
    ]
}

/// Why `values` break a property over `domain`, if they do.
fn violation(values: &[f64], domain: [f64; 2], bound: usize, inside: bool) -> Option<String> {
    if values.len() > bound {
        return Some(format!("{} values, more than {bound}", values.len()));
    }
    if domain.iter().any(|end| end.is_nan()) {
        return (!values.is_empty()).then(|| "values over a NaN domain".to_owned());
    }
    if let Some(v) = values.iter().find(|v| !v.is_finite()) {
        return Some(format!("a non-finite value {v}"));
    }
    let descending = domain[1] < domain[0];
    let out_of_order = values.windows(2).any(|pair| {
        if descending {
            pair[1] > pair[0]
        } else {
            pair[1] < pair[0]
        }
    });
    if out_of_order {
        return Some("values out of domain order".to_owned());
    }
    let [low, high] = if descending {
        [domain[1], domain[0]]
    } else {
        domain
    };
    let (low, high) = (low - 1e-9 * low.abs(), high + 1e-9 * high.abs());
    if inside && let Some(v) = values.iter().find(|v| **v < low || **v > high) {
        return Some(format!("{v} outside the domain"));
    }
    None
}

/// Runs `cases` cases; the count run, or the first violation.
fn run(cases: usize) -> Result<usize, String> {
    let transforms = transforms();
    let rules = [None, Some(TickRule::Data), Some(TickRule::Transformed)];
    let mut rng = Xorshift(0x9E37_79B9_7F4A_7C15);
    for case in 0..cases {
        let (name, base) = &transforms[rng.below(transforms.len())];
        let rule = &rules[rng.below(rules.len())];
        let domain = [rng.end(), rng.end()];
        let count = COUNTS[rng.below(COUNTS.len())];
        let mut s = base.clone().domain(domain);
        if let Some(rule) = rule {
            s = s.tick_rule(rule.clone());
        }
        let bound = CAP.min(count.max(1).saturating_mul(10));
        let ticks = s.ticks(count);
        let minor = s.minor(count);
        let set = s.tick_set(count);
        let axis_minor = Ticks::minor(&s, &set);
        let labels = s.labels(&set, &Locale::NEUTRAL);
        let found = violation(&ticks, domain, bound, true)
            .map(|v| format!("ticks: {v}"))
            .or_else(|| violation(&minor, domain, CAP, false).map(|v| format!("minor: {v}")))
            .or_else(|| {
                violation(&axis_minor, domain, CAP, true).map(|v| format!("axis minor: {v}"))
            })
            .or_else(|| {
                (labels.len() != ticks.len())
                    .then(|| format!("{} labels for {} ticks", labels.len(), ticks.len()))
            });
        if let Some(found) = found {
            return Err(format!(
                "case {case}: {name}, rule {rule:?}, domain {domain:?}, count {count}: {found}"
            ));
        }
    }
    Ok(cases)
}

#[test]
fn built_in_tick_rules_hold_their_properties_on_hostile_domains() {
    let cases = match std::env::var("RUSTD3_FUZZ").as_deref() {
        Ok("1") => 20_000,
        Ok(_) | Err(_) => 2_000,
    };
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || sender.send(run(cases)));
    match receiver.recv_timeout(BUDGET) {
        Ok(Ok(run)) => assert_eq!(run, cases),
        Ok(Err(violation)) => panic!("{violation}"),
        Err(mpsc::RecvTimeoutError::Timeout) => panic!("{cases} cases took over {BUDGET:?}"),
        Err(mpsc::RecvTimeoutError::Disconnected) => panic!("the fuzz thread panicked"),
    }
}
