//! The symlog, logit and asinh tick rules against matplotlib 3.11.2's
//! locators, from `dev/fixtures/scales.json` (written by `scales.py`).

use std::collections::BTreeMap;

use rustd3_core::scale;
use rustd3_testkit::obs::{Obs, ToObs, Tolerance};
use serde_json::Value;

/// The fixture, read at compile time so the test needs no working directory.
const SCALES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../dev/fixtures/scales.json"
));

/// One case: its domain and count, and the ticks matplotlib placed inside
/// the domain.
struct Case<'a> {
    value: &'a Value,
    domain: [f64; 2],
    count: usize,
    ticks: Obs,
}

impl<'a> Case<'a> {
    fn new(value: &'a Value) -> Case<'a> {
        let domain = value["domain"]
            .as_array()
            .and_then(|d| Some([d.first()?.as_f64()?, d.get(1)?.as_f64()?]))
            .expect("a domain of two numbers");
        let count = value["count"]
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .expect("a count");
        let ticks = Obs::try_from(&value["ticks"]).expect("ticks as an array of numbers");
        Case {
            value,
            domain,
            count,
            ticks,
        }
    }

    fn parameter(&self, name: &str) -> f64 {
        self.value[name].as_f64().expect("a numeric parameter")
    }

    /// The case's skip reason, or None when it is compared.
    fn reason(&self) -> Option<&str> {
        match self.value["compared"].as_bool().expect("a compared flag") {
            true => None,
            false => Some(self.value["reason"].as_str().expect("a reason")),
        }
    }

    /// Whether `ours` matches matplotlib's ticks within a relative 1e-9.
    fn matches(&self, ours: &[f64]) -> bool {
        ours.to_obs()
            .approx_eq(&self.ticks, &Tolerance::relative(1e-9))
    }
}

fn cases<'a>(fixture: &'a Value, locator: &str) -> Vec<Case<'a>> {
    fixture[locator]
        .as_array()
        .expect("an array of cases")
        .iter()
        .map(Case::new)
        .collect()
}

#[test]
fn ticks_match_matplotlibs_locators_in_every_compared_case() {
    let fixture: Value = serde_json::from_str(SCALES).expect("valid JSON");
    assert_eq!(fixture["matplotlib"], "3.11.2");
    let mut compared = 0;
    let mut skipped: BTreeMap<String, usize> = BTreeMap::new();
    let mut failures = Vec::new();
    let mut grade = |locator: &str, case: &Case, ours: Vec<f64>| {
        if let Some(reason) = case.reason() {
            *skipped.entry(reason.to_owned()).or_default() += 1;
        } else if case.matches(&ours) {
            compared += 1;
        } else {
            failures.push(format!("{locator} {:?}: ours {ours:?}", case.domain));
        }
    };
    for case in cases(&fixture, "symlog") {
        let s = scale::symlog()
            .constant(case.parameter("constant"))
            .domain(case.domain);
        grade("symlog", &case, s.ticks(case.count));
    }
    for case in cases(&fixture, "logit") {
        assert_eq!(case.reason(), None, "every logit case is compared");
        grade(
            "logit",
            &case,
            scale::logit().domain(case.domain).ticks(case.count),
        );
    }
    // asinh cases with a domain end at an exact power of ten are recorded,
    // not compared: matplotlib's tick there depends on its libm, and the
    // port ticks the end (`asinh_ticks_signed_powers_of_ten`).
    for case in cases(&fixture, "asinh") {
        let s = scale::asinh()
            .linear_width(case.parameter("linear_width"))
            .domain(case.domain);
        grade("asinh", &case, s.ticks(case.count));
    }
    println!("matplotlib fixtures: {compared} compared, skipped {skipped:?}");
    assert!(failures.is_empty(), "{failures:#?}");
    assert_eq!(compared, 22);
    assert_eq!(
        skipped,
        BTreeMap::from([
            ("asinh-power".to_owned(), 6),
            ("symlog-linear".to_owned(), 3),
            ("symlog-thinned".to_owned(), 1)
        ])
    );
}
