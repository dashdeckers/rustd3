//! `passing.txt`: every assertion id that must keep passing, and per test
//! file the number of tolerant passes it must keep (its floor).
//!
//! Line 1 is a `#` comment; then `#floor <package>/test/<file>.js <n>`
//! lines sorted by file; then the ids, sorted by file, line and column.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::{fmt, fs, io};

use crate::registry;
use crate::run::Outcome;

const HEADER: &str = "# d3 assertions that must keep passing, and per test file the passes it \
                      must keep. Written by rustd3-harness --bless after reading the diff.";
const FLOOR: &str = "#floor ";

/// The ids `passing.txt` lists and the floors it sets.
#[derive(Debug, Default)]
pub struct Ratchet {
    listed: BTreeSet<String>,
    floors: BTreeMap<String, usize>,
}

/// What a run means for the ratchet.
#[derive(Debug)]
pub struct Verdict {
    /// Listed ids that no longer pass, and files below their floors.
    pub regressions: Vec<Regression>,
    /// Ids that pass but are not listed.
    pub new: Vec<&'static str>,
}

/// A way the run falls short of `passing.txt`.
#[derive(Debug, PartialEq)]
pub enum Regression {
    /// A listed id fails.
    Fails(String),
    /// A listed id is now a skip, for this reason.
    Skipped(String, &'static str),
    /// A listed id was not reached: its test panicked first.
    NotReached(String),
    /// A listed id is no longer translated.
    Missing(String),
    /// A file passes fewer assertions than its floor.
    Floor {
        /// The test file, `<package>/test/<file>.js`.
        file: String,
        /// Its tolerant passes in this run.
        passing: usize,
        /// The passes `passing.txt` requires.
        floor: usize,
    },
}

/// What `--bless` changed in `passing.txt`.
#[derive(Debug, PartialEq)]
pub struct Change {
    /// Ids listed now that were not before.
    pub added: Vec<String>,
    /// Ids no longer listed.
    pub removed: Vec<String>,
    /// Floors set or changed, with their new values.
    pub floors: Vec<(String, usize)>,
}

impl fmt::Display for Regression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Regression::Fails(id) => write!(f, "{id} fails"),
            Regression::Skipped(id, reason) => write!(f, "{id} is skipped: {reason}"),
            Regression::NotReached(id) => write!(f, "{id} was not reached"),
            Regression::Missing(id) => write!(f, "{id} is no longer translated"),
            Regression::Floor {
                file,
                passing,
                floor,
            } => {
                write!(f, "{file} passes {passing}, below its floor of {floor}")
            }
        }
    }
}

impl Regression {
    /// Whether blessing would remove a listed id.
    pub fn drops_an_id(&self) -> bool {
        match self {
            Regression::Fails(_)
            | Regression::Skipped(..)
            | Regression::NotReached(_)
            | Regression::Missing(_) => true,
            Regression::Floor { .. } => false,
        }
    }
}

impl Ratchet {
    /// Reads `passing.txt`.
    pub fn load(path: &Path) -> io::Result<Self> {
        let text = fs::read_to_string(path)?;
        Self::parse(&text).map_err(|line| {
            let message = format!(
                "{}:{line}: a #floor line needs a file and a count",
                path.display()
            );
            io::Error::new(io::ErrorKind::InvalidData, message)
        })
    }

    /// Compares `outcomes` with the listed ids and the floors.
    pub fn check(&self, outcomes: &BTreeMap<&'static str, Outcome>) -> Verdict {
        let mut listed: Vec<&String> = self.listed.iter().collect();
        listed.sort_by(|a, b| registry::location(a).cmp(&registry::location(b)));
        let mut regressions = Vec::new();
        for id in listed {
            match outcomes.get(id.as_str()) {
                None => regressions.push(Regression::Missing(id.clone())),
                Some(Outcome::Pass) => {}
                Some(Outcome::Fail { .. }) => regressions.push(Regression::Fails(id.clone())),
                Some(Outcome::Skip(reason)) => {
                    regressions.push(Regression::Skipped(id.clone(), reason));
                }
                Some(Outcome::NotReached) => regressions.push(Regression::NotReached(id.clone())),
            }
        }
        let passing = passing_per_file(outcomes);
        for (file, &floor) in &self.floors {
            let passing = passing.get(file.as_str()).copied().unwrap_or(0);
            if passing < floor {
                regressions.push(Regression::Floor {
                    file: file.clone(),
                    passing,
                    floor,
                });
            }
        }
        let mut new: Vec<&'static str> = outcomes
            .iter()
            .filter(|(id, outcome)| **outcome == Outcome::Pass && !self.listed.contains(**id))
            .map(|(id, _)| *id)
            .collect();
        new.sort_by_key(|id| registry::location(id));
        Verdict { regressions, new }
    }

    /// Rewrites `path` with every passing id. Floor lines are kept verbatim,
    /// unless `floors`: then every translated file's floor becomes its
    /// current tolerant-passing count, new files included.
    pub fn bless(
        path: &Path,
        outcomes: &BTreeMap<&'static str, Outcome>,
        floors: bool,
    ) -> io::Result<Change> {
        let old = match Self::load(path) {
            Ok(old) => old,
            Err(e) if e.kind() == io::ErrorKind::NotFound => Self::default(),
            Err(e) => return Err(e),
        };
        let mut new_floors = old.floors.clone();
        let mut changed = Vec::new();
        if floors {
            for (file, passing) in passing_per_file(outcomes) {
                if new_floors.insert(file.to_owned(), passing) != Some(passing) {
                    changed.push((file.to_owned(), passing));
                }
            }
        }
        let mut ids: Vec<&str> = outcomes
            .iter()
            .filter(|(_, outcome)| **outcome == Outcome::Pass)
            .map(|(id, _)| *id)
            .collect();
        ids.sort_by_key(|id| registry::location(id));
        let mut text = format!("{HEADER}\n");
        for (file, floor) in &new_floors {
            text.push_str(&format!("{FLOOR}{file} {floor}\n"));
        }
        for id in &ids {
            text.push_str(id);
            text.push('\n');
        }
        fs::write(path, text)?;
        let listed: BTreeSet<&str> = ids.iter().copied().collect();
        Ok(Change {
            added: ids
                .iter()
                .filter(|id| !old.listed.contains(**id))
                .map(|id| (*id).to_owned())
                .collect(),
            removed: old
                .listed
                .iter()
                .filter(|id| !listed.contains(id.as_str()))
                .cloned()
                .collect(),
            floors: changed,
        })
    }

    /// The ratchet in `text`, or the 1-based number of a malformed line.
    fn parse(text: &str) -> Result<Self, usize> {
        let mut ratchet = Self::default();
        for (number, line) in (1_usize..).zip(text.lines()) {
            if let Some(floor) = line.strip_prefix(FLOOR) {
                let (file, n) = floor.rsplit_once(' ').ok_or(number)?;
                let n = n.parse().map_err(|_| number)?;
                ratchet.floors.insert(file.to_owned(), n);
            } else if !line.starts_with('#') && !line.trim().is_empty() {
                ratchet.listed.insert(line.to_owned());
            }
        }
        Ok(ratchet)
    }
}

/// Tolerant passes per test file, for every file `outcomes` has.
fn passing_per_file<'a>(outcomes: &BTreeMap<&'a str, Outcome>) -> BTreeMap<&'a str, usize> {
    let mut passing = BTreeMap::new();
    for (id, outcome) in outcomes {
        let (file, _, _) = registry::location(id);
        *passing.entry(file).or_default() += usize::from(*outcome == Outcome::Pass);
    }
    passing
}

#[cfg(test)]
mod tests {
    use rustd3_testkit::obs::Obs;

    use super::*;

    const A: &str = "d3-x/test/a.js";

    fn ratchet(text: &str) -> Ratchet {
        Ratchet::parse(text).unwrap()
    }

    fn outcomes(entries: &[(&'static str, Outcome)]) -> BTreeMap<&'static str, Outcome> {
        entries.iter().cloned().collect()
    }

    fn fail() -> Outcome {
        Outcome::Fail {
            actual: Obs::Num(1.0),
            expected: Obs::Num(2.0),
        }
    }

    fn scratch(test: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("rustd3-harness").join(test);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("passing.txt");
        if path.exists() {
            fs::remove_file(&path).unwrap();
        }
        path
    }

    #[test]
    fn listed_ids_that_still_pass_are_no_regression() {
        let r = ratchet("# c\n#floor d3-x/test/a.js 1\nd3-x/test/a.js:1:1\n");
        let verdict = r.check(&outcomes(&[("d3-x/test/a.js:1:1", Outcome::Pass)]));
        assert!(verdict.regressions.is_empty());
        assert!(verdict.new.is_empty());
    }

    #[test]
    fn a_listed_id_that_fails_is_a_regression() {
        let r = ratchet("#\nd3-x/test/a.js:1:1\n");
        let verdict = r.check(&outcomes(&[("d3-x/test/a.js:1:1", fail())]));
        assert_eq!(
            verdict.regressions,
            [Regression::Fails("d3-x/test/a.js:1:1".to_owned())]
        );
    }

    #[test]
    fn a_listed_id_that_becomes_a_skip_is_a_regression() {
        let r = ratchet("#\nd3-x/test/a.js:1:1\n");
        let verdict = r.check(&outcomes(&[(
            "d3-x/test/a.js:1:1",
            Outcome::Skip("gap: phase 26"),
        )]));
        assert_eq!(
            verdict.regressions,
            [Regression::Skipped(
                "d3-x/test/a.js:1:1".to_owned(),
                "gap: phase 26"
            )]
        );
    }

    #[test]
    fn a_listed_id_that_disappears_is_a_regression() {
        let r = ratchet("#\nd3-x/test/a.js:1:1\n");
        let verdict = r.check(&outcomes(&[("d3-x/test/a.js:2:1", Outcome::Pass)]));
        assert_eq!(
            verdict.regressions,
            [Regression::Missing("d3-x/test/a.js:1:1".to_owned())]
        );
        assert_eq!(verdict.new, ["d3-x/test/a.js:2:1"]);
    }

    #[test]
    fn a_listed_id_that_is_not_reached_is_a_regression() {
        let r = ratchet("#\nd3-x/test/a.js:1:1\n");
        let verdict = r.check(&outcomes(&[("d3-x/test/a.js:1:1", Outcome::NotReached)]));
        assert_eq!(
            verdict.regressions,
            [Regression::NotReached("d3-x/test/a.js:1:1".to_owned())]
        );
    }

    #[test]
    fn a_file_below_its_floor_is_a_regression() {
        let r = ratchet("#\n#floor d3-x/test/a.js 2\n#floor d3-x/test/gone.js 1\n");
        let verdict = r.check(&outcomes(&[
            ("d3-x/test/a.js:1:1", Outcome::Pass),
            ("d3-x/test/a.js:2:1", fail()),
        ]));
        assert_eq!(
            verdict.regressions,
            [
                Regression::Floor {
                    file: A.to_owned(),
                    passing: 1,
                    floor: 2
                },
                Regression::Floor {
                    file: "d3-x/test/gone.js".to_owned(),
                    passing: 0,
                    floor: 1
                },
            ]
        );
        assert!(verdict.regressions.iter().all(|r| !r.drops_an_id()));
    }

    #[test]
    fn a_malformed_floor_names_its_line() {
        assert!(matches!(
            Ratchet::parse("#\n#floor d3-x/test/a.js\n"),
            Err(2)
        ));
        assert!(matches!(
            Ratchet::parse("#\n#floor d3-x/test/a.js many\n"),
            Err(2)
        ));
    }

    #[test]
    fn bless_sorts_ids_by_line_and_column_numerically() {
        let path = scratch("bless_sorts_ids_by_line_and_column_numerically");
        let change = Ratchet::bless(
            &path,
            &outcomes(&[
                ("d3-x/test/a.js:10:3", Outcome::Pass),
                ("d3-x/test/a.js:6:12", Outcome::Pass),
                ("d3-x/test/a.js:6:3", Outcome::Pass),
                ("d3-x/test/a.js:7:3", fail()),
                ("d3-x/test/b.js:1:1", Outcome::Skip("js: x")),
            ]),
            true,
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            format!(
                "{HEADER}\n#floor d3-x/test/a.js 3\n#floor d3-x/test/b.js 0\n\
                 d3-x/test/a.js:6:3\nd3-x/test/a.js:6:12\nd3-x/test/a.js:10:3\n"
            )
        );
        assert_eq!(change.added.len(), 3);
        assert!(change.removed.is_empty());
        assert_eq!(
            change.floors,
            [(A.to_owned(), 3), ("d3-x/test/b.js".to_owned(), 0)]
        );
    }

    #[test]
    fn bless_keeps_floors_verbatim_unless_asked() {
        let path = scratch("bless_keeps_floors_verbatim_unless_asked");
        fs::write(
            &path,
            "# old\n#floor d3-x/test/a.js 5\nd3-x/test/a.js:1:1\nd3-x/test/a.js:2:1\n",
        )
        .unwrap();
        let now = outcomes(&[
            ("d3-x/test/a.js:1:1", Outcome::Pass),
            ("d3-x/test/a.js:2:1", fail()),
            ("d3-x/test/a.js:3:1", Outcome::Pass),
        ]);
        let change = Ratchet::bless(&path, &now, false).unwrap();
        assert_eq!(change.added, ["d3-x/test/a.js:3:1"]);
        assert_eq!(change.removed, ["d3-x/test/a.js:2:1"]);
        assert!(change.floors.is_empty());
        let blessed = Ratchet::load(&path).unwrap();
        assert_eq!(blessed.floors, BTreeMap::from([(A.to_owned(), 5)]));
        let change = Ratchet::bless(&path, &now, true).unwrap();
        assert_eq!(change.floors, [(A.to_owned(), 2)]);
        assert!(change.added.is_empty() && change.removed.is_empty());
    }
}
