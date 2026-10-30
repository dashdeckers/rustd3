//! [`Experiment`]: the flags that narrow or vary a level for one question
//! on one machine: which profiles run (`--only`),
//! which of their cells (`--scenario`, `--workload`, `--factor`), how many
//! runs a cell (`--runs`), the widget's `--render`, `--threads` and
//! `--caching`, and a [`Sweep`] of one of those three, each value its own
//! profile `<profile>@<key>=<value>` in the one archive. A level records it
//! in `results.json` and its reports.

use serde::{Deserialize, Serialize};

/// What the command line narrows or varies; the default runs the level as
/// it is.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Experiment {
    /// `--only`: the profiles that run, by name; `None` runs them all.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub only: Option<Vec<String>>,
    /// `--scenario`: the widget profiles' cells of these scenarios only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenarios: Option<Vec<String>>,
    /// `--workload`: of these workloads only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workloads: Option<Vec<String>>,
    /// `--factor`: at these factors only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub factors: Option<Vec<f32>>,
    /// `--runs`: runs a cell, in place of each profile's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runs: Option<u32>,
    /// `--render`: `auto`, `inline` or `worker`, for the widget's
    /// profiles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render: Option<String>,
    /// `--threads`: the widget's render pool, 1 to 64.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threads: Option<usize>,
    /// `--caching`: `on`, `off` or `verify`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caching: Option<String>,
    /// `--sweep KEY=V1,V2,…`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sweep: Option<Sweep>,
}

/// Each selected widget profile once per value of one of the widget's
/// settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sweep {
    /// The setting.
    pub key: Key,
    /// Its values, in the order given.
    pub values: Vec<String>,
}

/// A setting a sweep varies: the flag of `run` it sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Key {
    /// `--render auto|inline|worker`.
    Render,
    /// `--threads N`.
    Threads,
    /// `--caching on|off|verify`.
    Caching,
}

/// One run of a profile the experiment asks for: the suffix of its name
/// (`@threads=2`, empty without a sweep) and the flags `run` gets for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    /// What follows the profile's name.
    pub suffix: String,
    /// `--render`, `--threads`, `--caching` and their values.
    pub args: Vec<String>,
}

/// Why an experiment's flags are rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A profile `--only` does not know.
    Profile(String),
    /// A sweep that is not `KEY=V1,V2,…` with a known key and valid values.
    Sweep(String),
    /// A sweep of a setting also given on its own.
    Twice(Key),
}

/// The profiles `--only` takes: the figure profile and the widget's.
pub const PROFILES: [&str; 7] = [
    "figure",
    "software",
    "native",
    "nogpu",
    "nogpu-x11",
    "4cores",
    "2cores",
];

/// The values `--render` takes.
pub const RENDERS: [&str; 3] = ["auto", "inline", "worker"];

/// The values `--caching` takes.
pub const CACHINGS: [&str; 3] = ["on", "off", "verify"];

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Profile(name) => write!(
                f,
                "--only does not know the profile {name:?} (it takes {})",
                PROFILES.join(", ")
            ),
            Error::Sweep(text) => write!(
                f,
                "--sweep takes KEY=V1,V2,… with KEY render (auto, inline, worker), threads (1 to \
                 64) or caching (on, off, verify), not {text:?}"
            ),
            Error::Twice(key) => write!(
                f,
                "--sweep {} and --{} together: the sweep sets it",
                key.name(),
                key.name()
            ),
        }
    }
}

impl std::error::Error for Error {}

impl Key {
    /// Its name in `--sweep` and in a profile's name.
    pub fn name(self) -> &'static str {
        match self {
            Key::Render => "render",
            Key::Threads => "threads",
            Key::Caching => "caching",
        }
    }

    /// Whether `value` is one it takes.
    fn admits(self, value: &str) -> bool {
        match self {
            Key::Render => RENDERS.contains(&value),
            Key::Caching => CACHINGS.contains(&value),
            Key::Threads => value.parse::<usize>().is_ok_and(|n| (1..=64).contains(&n)),
        }
    }
}

impl Sweep {
    /// `KEY=V1,V2,…`.
    pub fn parse(text: &str) -> Result<Sweep, Error> {
        let bad = || Error::Sweep(text.to_owned());
        let (key, values) = text.split_once('=').ok_or_else(bad)?;
        let key = [Key::Render, Key::Threads, Key::Caching]
            .into_iter()
            .find(|k| k.name() == key)
            .ok_or_else(bad)?;
        let values: Vec<String> = values.split(',').map(|v| v.trim().to_owned()).collect();
        if values.iter().all(|v| key.admits(v)) {
            Ok(Sweep { key, values })
        } else {
            Err(bad())
        }
    }
}

impl Experiment {
    /// Whether it changes nothing.
    pub fn is_empty(&self) -> bool {
        *self == Experiment::default()
    }

    /// Checks what one flag cannot: every `--only` name is a profile, and
    /// no setting is swept and given.
    pub fn checked(self) -> Result<Experiment, Error> {
        if let Some(name) = self
            .only
            .iter()
            .flatten()
            .find(|name| !PROFILES.contains(&name.as_str()))
        {
            return Err(Error::Profile(name.clone()));
        }
        if let Some(sweep) = &self.sweep {
            let given = match sweep.key {
                Key::Render => self.render.is_some(),
                Key::Threads => self.threads.is_some(),
                Key::Caching => self.caching.is_some(),
            };
            if given {
                return Err(Error::Twice(sweep.key));
            }
        }
        Ok(self)
    }

    /// Whether the profile `name` runs.
    pub fn runs(&self, name: &str) -> bool {
        self.only
            .as_ref()
            .is_none_or(|only| only.iter().any(|n| n == name))
    }

    /// The runs of each selected widget profile: one, or one per value of
    /// the sweep; each with the widget's settings as flags.
    pub fn variants(&self) -> Vec<Variant> {
        let mut base = Vec::new();
        if let Some(render) = &self.render {
            base.extend(["--render".to_owned(), render.clone()]);
        }
        if let Some(threads) = self.threads {
            base.extend(["--threads".to_owned(), threads.to_string()]);
        }
        if let Some(caching) = &self.caching {
            base.extend(["--caching".to_owned(), caching.clone()]);
        }
        match &self.sweep {
            None => vec![Variant {
                suffix: String::new(),
                args: base,
            }],
            Some(sweep) => sweep
                .values
                .iter()
                .map(|value| {
                    let mut args = base.clone();
                    args.extend([format!("--{}", sweep.key.name()), value.clone()]);
                    Variant {
                        suffix: format!("@{}={value}", sweep.key.name()),
                        args,
                    }
                })
                .collect(),
        }
    }

    /// What it sets, a phrase each (empty when nothing).
    pub fn describe(&self) -> Vec<String> {
        let list = |items: &[String]| items.join(", ");
        let mut lines = Vec::new();
        if let Some(only) = &self.only {
            lines.push(format!("only the profiles {}", list(only)));
        }
        if let Some(scenarios) = &self.scenarios {
            lines.push(format!("the scenarios {}", list(scenarios)));
        }
        if let Some(workloads) = &self.workloads {
            lines.push(format!("the workloads {}", list(workloads)));
        }
        if let Some(factors) = &self.factors {
            let factors: Vec<String> = factors.iter().map(f32::to_string).collect();
            lines.push(format!("the factors {}", list(&factors)));
        }
        if let Some(runs) = self.runs {
            lines.push(format!("{runs} runs a cell"));
        }
        if let Some(render) = &self.render {
            lines.push(format!("render {render}"));
        }
        if let Some(threads) = self.threads {
            lines.push(format!("{threads} render threads"));
        }
        if let Some(caching) = &self.caching {
            lines.push(format!("caching {caching}"));
        }
        if let Some(sweep) = &self.sweep {
            lines.push(format!(
                "a sweep of {} over {}, a profile per value",
                sweep.key.name(),
                list(&sweep.values)
            ));
        }
        lines
    }
}

/// The profile a name with a sweep's suffix (`software@threads=2`) is a
/// variant of, and the suffix's value (`threads=2`) if any.
pub fn split(name: &str) -> (&str, Option<&str>) {
    match name.split_once('@') {
        Some((base, value)) => (base, Some(value)),
        None => (name, None),
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, Experiment, Key, Sweep, split};

    #[test]
    fn a_sweep_names_a_setting_and_its_values() {
        let sweep = Sweep::parse("threads=1,2,8").unwrap();
        assert_eq!(
            (sweep.key, sweep.values.clone()),
            (Key::Threads, vec!["1".into(), "2".into(), "8".into()])
        );
        assert!(Sweep::parse("render=inline,worker").is_ok());
        assert!(Sweep::parse("caching=on,verify").is_ok());
        for bad in [
            "threads",
            "threads=0",
            "threads=1,65",
            "render=gpu",
            "pacing=bench",
        ] {
            assert!(Sweep::parse(bad).is_err(), "{bad}");
        }
        let experiment = Experiment {
            sweep: Some(sweep),
            caching: Some("off".to_owned()),
            ..Experiment::default()
        };
        let variants = experiment.variants();
        let suffixes: Vec<&str> = variants.iter().map(|v| v.suffix.as_str()).collect();
        assert_eq!(suffixes, ["@threads=1", "@threads=2", "@threads=8"]);
        assert_eq!(
            variants[2].args,
            ["--caching", "off", "--threads", "8"].map(str::to_owned)
        );
        assert_eq!(split("software@threads=8"), ("software", Some("threads=8")));
        assert_eq!(split("native"), ("native", None));
    }

    #[test]
    fn only_names_known_profiles_and_a_setting_is_swept_or_given() {
        let only = |names: &[&str]| Experiment {
            only: Some(names.iter().map(|n| (*n).to_owned()).collect()),
            ..Experiment::default()
        };
        let checked = only(&["software", "figure"]).checked().unwrap();
        assert!(checked.runs("software") && checked.runs("figure") && !checked.runs("native"));
        assert_eq!(
            only(&["gpu"]).checked(),
            Err(Error::Profile("gpu".to_owned()))
        );
        let twice = Experiment {
            threads: Some(2),
            sweep: Some(Sweep::parse("threads=1,2").unwrap()),
            ..Experiment::default()
        };
        assert_eq!(twice.checked(), Err(Error::Twice(Key::Threads)));
        let empty = Experiment::default();
        assert!(empty.is_empty() && empty.runs("nogpu") && empty.describe().is_empty());
        assert_eq!(empty.variants().len(), 1);
    }
}
