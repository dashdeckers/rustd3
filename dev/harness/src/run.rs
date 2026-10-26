//! Runs every generated test under `catch_unwind` and folds its records
//! into one outcome per assertion, then reports the counts.

use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::{fmt, panic};

use rustd3_testkit::obs::Obs;

use crate::prelude::Kind;
use crate::record::{self, Mark};
use crate::registry::{self, Assertion, Suite};

/// How many failing ids the report lists without `--failing`.
const FIRST_FAILURES: usize = 20;

/// What happened to one assertion. One inside a loop has several records;
/// it passes iff every record passes and at least one exists.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// Every record passes the tolerant comparison.
    Pass,
    /// The first record that fails it.
    Fail {
        /// Our value.
        actual: Obs,
        /// d3's value.
        expected: Obs,
    },
    /// Skipped by the translator, for this reason.
    Skip(&'static str),
    /// No record: its test panicked first.
    NotReached,
}

/// A contradiction between the generated tables and what the tests
/// recorded: a harness bug, not a porting bug.
#[derive(Debug)]
pub enum Fault {
    /// An id `ASSERTS` lists twice.
    Duplicate(&'static str),
    /// A recorded id `ASSERTS` does not list.
    Unlisted(&'static str),
    /// A record that contradicts its `ASSERTS` entry: a check of a skipped
    /// assertion, or a skip of a translated one or for another reason.
    Contradicts(&'static str),
    /// A record that passes d3's semantics but fails the tolerant format.
    Lenient {
        /// The assertion.
        id: &'static str,
        /// The d3 assertion it came from.
        kind: Kind,
        /// Our value.
        actual: Obs,
        /// d3's value.
        expected: Obs,
    },
}

/// Every suite's outcomes, one per assertion.
#[derive(Debug)]
pub struct Run {
    suites: &'static [Suite],
    outcomes: BTreeMap<&'static str, Outcome>,
    d3: BTreeSet<&'static str>,
    panics: BTreeMap<&'static str, String>,
}

/// Counts over the assertions of a file or a suite.
#[derive(Debug, Default)]
struct Tally {
    assertions: usize,
    skipped: BTreeMap<&'static str, usize>,
    not_reached: usize,
    d3: usize,
    passing: usize,
    failing: usize,
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fault::Duplicate(id) => write!(f, "{id} is listed twice in ASSERTS"),
            Fault::Unlisted(id) => write!(f, "{id} was recorded but is not in ASSERTS"),
            Fault::Contradicts(id) => write!(f, "{id} was recorded against its ASSERTS entry"),
            Fault::Lenient {
                id,
                kind,
                actual,
                expected,
            } => write!(
                f,
                "{id} passes d3's {kind:?} but not the tolerant format: \
                 actual {actual:?}, expected {expected:?}"
            ),
        }
    }
}

impl Run {
    /// Runs every test of `suites` and folds the records, or returns the
    /// faults that make the outcomes meaningless.
    pub fn suites(suites: &'static [Suite]) -> Result<Run, Vec<Fault>> {
        let mut listed: BTreeMap<&'static str, &'static Assertion> = BTreeMap::new();
        let mut faults = Vec::new();
        for assertion in suites.iter().flat_map(|s| s.asserts) {
            if listed.insert(assertion.id, assertion).is_some() {
                faults.push(Fault::Duplicate(assertion.id));
            }
        }
        if !faults.is_empty() {
            return Err(faults);
        }
        let panics = run_tests(suites);
        let mut marks: BTreeMap<&'static str, Vec<Mark>> = BTreeMap::new();
        for record in record::take() {
            if listed.contains_key(record.id) {
                marks.entry(record.id).or_default().push(record.mark);
            } else {
                faults.push(Fault::Unlisted(record.id));
            }
        }
        let (mut outcomes, mut d3) = (BTreeMap::new(), BTreeSet::new());
        for (id, assertion) in listed {
            let (outcome, passes_d3) =
                fold(assertion, marks.remove(id).unwrap_or_default(), &mut faults);
            if passes_d3 {
                d3.insert(id);
            }
            outcomes.insert(id, outcome);
        }
        if !faults.is_empty() {
            return Err(faults);
        }
        Ok(Run {
            suites,
            outcomes,
            d3,
            panics,
        })
    }

    /// Every assertion's outcome, by id.
    pub fn outcomes(&self) -> &BTreeMap<&'static str, Outcome> {
        &self.outcomes
    }

    /// The panic message of the test that holds `id`, if that test panicked.
    pub fn panic_of(&self, id: &str) -> Option<&str> {
        let assertion = self
            .suites
            .iter()
            .flat_map(|s| s.asserts)
            .find(|a| a.id == id)?;
        self.panics.get(assertion.test).map(String::as_str)
    }

    /// One line per test file, in suite order:
    /// `<file>: translated T / passing P / skipped S`.
    pub fn file_lines(&self) -> Vec<String> {
        self.suites
            .iter()
            .flat_map(|suite| self.files(suite))
            .map(|(file, tally)| format!("{file}: {}", tally.headline()))
            .collect()
    }

    /// Prints, per suite and per file, the headline and every count; then
    /// the failing ids with actual and expected (only those starting with
    /// `failing`, all of them, when given; else the first 20); then the
    /// tests that panicked.
    pub fn report(&self, failing: Option<&str>) {
        for suite in self.suites {
            println!("suite {}", suite.name);
            let mut total = Tally::default();
            for (file, tally) in self.files(suite) {
                println!("  {file}: {}", tally.headline());
                println!("    {}", tally.counts());
                total.merge(&tally);
            }
            println!("  {}: {}", suite.name, total.counts());
        }
        let failures: Vec<_> = self
            .suites
            .iter()
            .flat_map(|s| s.asserts)
            .filter(|a| failing.is_none_or(|prefix| a.id.starts_with(prefix)))
            .filter_map(|a| match &self.outcomes[a.id] {
                Outcome::Fail { actual, expected } => Some((a, actual, expected)),
                Outcome::Pass | Outcome::Skip(_) | Outcome::NotReached => None,
            })
            .collect();
        let shown = if failing.is_some() {
            failures.len()
        } else {
            FIRST_FAILURES
        };
        if !failures.is_empty() {
            println!(
                "\nfailing ({} of {}):",
                shown.min(failures.len()),
                failures.len()
            );
        }
        for (assertion, actual, expected) in failures.iter().take(shown) {
            println!(
                "  {}: actual {actual:?}, expected {expected:?}",
                assertion.id
            );
            println!("    in {}", assertion.test);
        }
        if !self.panics.is_empty() {
            println!("\ntests that panicked:");
        }
        for (test, message) in &self.panics {
            println!("  {test}: {message}");
        }
    }

    /// The files of `suite`, in order, with their tallies.
    fn files(&self, suite: &Suite) -> Vec<(&'static str, Tally)> {
        let mut files: Vec<(&'static str, Tally)> = Vec::new();
        for assertion in suite.asserts {
            let (file, _, _) = registry::location(assertion.id);
            let index = match files.iter().position(|(f, _)| *f == file) {
                Some(index) => index,
                None => {
                    files.push((file, Tally::default()));
                    files.len() - 1
                }
            };
            let outcome = &self.outcomes[assertion.id];
            files[index].1.add(outcome, self.d3.contains(assertion.id));
        }
        files
    }
}

impl Tally {
    fn add(&mut self, outcome: &Outcome, passes_d3: bool) {
        self.assertions += 1;
        self.d3 += usize::from(passes_d3);
        match outcome {
            Outcome::Pass => self.passing += 1,
            Outcome::Fail { .. } => self.failing += 1,
            Outcome::Skip(reason) => {
                let category = reason.split_once(':').map_or(*reason, |(c, _)| c);
                *self.skipped.entry(category).or_default() += 1;
            }
            Outcome::NotReached => self.not_reached += 1,
        }
    }

    fn merge(&mut self, other: &Tally) {
        self.assertions += other.assertions;
        for (category, n) in &other.skipped {
            *self.skipped.entry(category).or_default() += n;
        }
        self.not_reached += other.not_reached;
        self.d3 += other.d3;
        self.passing += other.passing;
        self.failing += other.failing;
    }

    fn skipped(&self) -> usize {
        self.skipped.values().sum()
    }

    fn headline(&self) -> String {
        let translated = self.assertions - self.skipped();
        format!(
            "translated {translated} / passing {} / skipped {}",
            self.passing,
            self.skipped()
        )
    }

    fn counts(&self) -> String {
        let categories: Vec<_> = self
            .skipped
            .iter()
            .map(|(c, n)| format!("{c}: {n}"))
            .collect();
        let by = if categories.is_empty() {
            String::new()
        } else {
            format!(" ({})", categories.join(", "))
        };
        format!(
            "{} assertions: {} translated, {} skipped{by}, {} not reached; \
             {} pass d3's semantics, {} passing, {} failing",
            self.assertions,
            self.assertions - self.skipped(),
            self.skipped(),
            self.not_reached,
            self.d3,
            self.passing,
            self.failing
        )
    }
}

/// Folds an assertion's records into its outcome and whether it passes
/// d3's own semantics, noting contradictions in `faults`.
fn fold(assertion: &Assertion, marks: Vec<Mark>, faults: &mut Vec<Fault>) -> (Outcome, bool) {
    let id = assertion.id;
    if let Some(reason) = assertion.skip {
        let agrees = |mark: &Mark| match mark {
            Mark::Skipped(r) => *r == reason,
            Mark::Checked { .. } => false,
        };
        if !marks.iter().all(agrees) {
            faults.push(Fault::Contradicts(id));
        }
        return (Outcome::Skip(reason), false);
    }
    let (mut checked, mut passes_d3, mut failure) = (false, true, None);
    for mark in marks {
        match mark {
            Mark::Skipped(_) => faults.push(Fault::Contradicts(id)),
            Mark::Checked {
                kind,
                d3,
                tolerant,
                actual,
                expected,
            } => {
                checked = true;
                passes_d3 &= d3;
                if tolerant {
                    continue;
                }
                if d3 {
                    faults.push(Fault::Lenient {
                        id,
                        kind,
                        actual,
                        expected,
                    });
                } else if failure.is_none() {
                    failure = Some(Outcome::Fail { actual, expected });
                }
            }
        }
    }
    match (checked, failure) {
        (false, _) => (Outcome::NotReached, false),
        (true, Some(failure)) => (failure, passes_d3),
        (true, None) => (Outcome::Pass, passes_d3),
    }
}

/// Runs every test, quietly, and returns the panic messages by test name.
fn run_tests(suites: &[Suite]) -> BTreeMap<&'static str, String> {
    let hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let mut panics = BTreeMap::new();
    for test in suites.iter().flat_map(|s| s.tests) {
        if let Err(payload) = panic::catch_unwind(test.run) {
            panics
                .entry(test.name)
                .or_insert_with(|| message(payload.as_ref()));
        }
    }
    panic::set_hook(hook);
    panics
}

fn message(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|s| (*s).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a panic without a message".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{check, skip};
    use crate::registry::Test;

    fn passing() {
        check("t/test/a.js:1:1", Kind::Strict, &1.0, &1.0);
        check("t/test/a.js:2:1", Kind::InDelta(1e-6), &0.1, &0.1000001);
        skip("t/test/a.js:3:1", "js: coercion");
    }

    fn looping() {
        for x in [1.0, 2.0] {
            check("t/test/a.js:4:1", Kind::Strict, &x, &x);
            check("t/test/a.js:5:1", Kind::Strict, &x, &1.0);
        }
    }

    fn panicking() {
        check("t/test/b.js:1:1", Kind::Strict, &(0.1 + 0.2), &0.3);
        panic!("boom");
    }

    const ASSERTS: &[Assertion] = &[
        Assertion {
            id: "t/test/a.js:1:1",
            test: "a",
            skip: None,
        },
        Assertion {
            id: "t/test/a.js:2:1",
            test: "a",
            skip: None,
        },
        Assertion {
            id: "t/test/a.js:3:1",
            test: "a",
            skip: Some("js: coercion"),
        },
        Assertion {
            id: "t/test/a.js:4:1",
            test: "loop",
            skip: None,
        },
        Assertion {
            id: "t/test/a.js:5:1",
            test: "loop",
            skip: None,
        },
        Assertion {
            id: "t/test/b.js:1:1",
            test: "b",
            skip: None,
        },
        Assertion {
            id: "t/test/b.js:2:1",
            test: "b",
            skip: None,
        },
    ];
    const TESTS: &[Test] = &[
        Test {
            name: "a",
            run: passing,
        },
        Test {
            name: "loop",
            run: looping,
        },
        Test {
            name: "b",
            run: panicking,
        },
    ];
    const SUITES: &[Suite] = &[Suite {
        name: "t",
        tests: TESTS,
        asserts: ASSERTS,
    }];

    #[test]
    fn records_fold_into_one_outcome_per_assertion() {
        let run = Run::suites(SUITES).unwrap();
        let outcomes: Vec<_> = run.outcomes().values().cloned().collect();
        assert_eq!(
            outcomes,
            [
                Outcome::Pass,
                Outcome::Pass,
                Outcome::Skip("js: coercion"),
                Outcome::Pass,
                Outcome::Fail {
                    actual: Obs::Num(2.0),
                    expected: Obs::Num(1.0)
                },
                Outcome::Pass,
                Outcome::NotReached,
            ]
        );
        assert_eq!(
            run.d3.iter().copied().collect::<Vec<_>>(),
            ["t/test/a.js:1:1", "t/test/a.js:2:1", "t/test/a.js:4:1"]
        );
        assert_eq!(run.panic_of("t/test/b.js:2:1"), Some("boom"));
        assert_eq!(run.panic_of("t/test/a.js:1:1"), None);
    }

    #[test]
    fn file_lines_count_translated_passing_and_skipped() {
        let run = Run::suites(SUITES).unwrap();
        assert_eq!(
            run.file_lines(),
            [
                "t/test/a.js: translated 4 / passing 3 / skipped 1",
                "t/test/b.js: translated 2 / passing 1 / skipped 0",
            ]
        );
    }

    fn unlisted() {
        check("t/test/c.js:9:9", Kind::Strict, &1.0, &1.0);
    }

    fn contradicting() {
        skip("t/test/c.js:1:1", "js: another reason");
        check("t/test/c.js:2:1", Kind::PathEq, "M0.0000001,0", "M0,0");
    }

    #[test]
    fn contradictions_between_tables_and_records_are_faults() {
        const ASSERTS: &[Assertion] = &[
            Assertion {
                id: "t/test/c.js:1:1",
                test: "c",
                skip: Some("js: coercion"),
            },
            Assertion {
                id: "t/test/c.js:2:1",
                test: "c",
                skip: None,
            },
        ];
        const TESTS: &[Test] = &[
            Test {
                name: "c",
                run: contradicting,
            },
            Test {
                name: "u",
                run: unlisted,
            },
        ];
        const SUITES: &[Suite] = &[Suite {
            name: "c",
            tests: TESTS,
            asserts: ASSERTS,
        }];
        let faults = Run::suites(SUITES).unwrap_err();
        let shown: Vec<_> = faults.iter().map(ToString::to_string).collect();
        assert_eq!(
            shown,
            [
                "t/test/c.js:9:9 was recorded but is not in ASSERTS",
                "t/test/c.js:1:1 was recorded against its ASSERTS entry",
                "t/test/c.js:2:1 passes d3's PathEq but not the tolerant format: \
                 actual Str(\"M0.0000001,0\"), expected Str(\"M0,0\")",
            ]
        );
    }

    #[test]
    fn duplicate_ids_are_faults_before_anything_runs() {
        const ASSERTS: &[Assertion] = &[
            Assertion {
                id: "t/test/d.js:1:1",
                test: "d",
                skip: None,
            },
            Assertion {
                id: "t/test/d.js:1:1",
                test: "d",
                skip: None,
            },
        ];
        const SUITES: &[Suite] = &[Suite {
            name: "d",
            tests: &[],
            asserts: ASSERTS,
        }];
        let faults = Run::suites(SUITES).unwrap_err();
        assert!(matches!(faults[..], [Fault::Duplicate("t/test/d.js:1:1")]));
    }
}
