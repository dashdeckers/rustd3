//! [`Display`]: what a level learns about the display in a few seconds,
//! before its profiles run.
//!
//! One short windowed run on iced's own choice of renderer
//! (`bench_gestures --plan display`, which is `hover` on `trace` for 3 s
//! with data ÷ 100) gives three values. The refresh rate is its idle
//! frames' median interval, when they are paced. The scale factor is the
//! one the window reports, and the backend is the one iced chose. Where no
//! window can open, the level runs headless and says why. Where the frames
//! are not paced (a VM, Xvfb, a remote desktop), it runs at 60 Hz and
//! every frame budget assumes that rate.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::record::{self, Record};
use crate::stats::percentile;

/// What the level knows of the display, and how it knows it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Display {
    /// Why the level runs headless; `None` when windows open.
    pub headless: Option<String>,
    /// The refresh rate the runs use, Hz.
    pub refresh_hz: f64,
    /// Where the rate comes from.
    pub refresh: Source,
    /// The scale factor the runs use.
    pub native_factor: f32,
    /// Where the factor comes from.
    pub factor: Source,
    /// The renderer iced chose on its own (`wgpu` or `tiny-skia`), when
    /// known.
    pub backend: Option<String>,
}

/// Where a display value comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The command line (`--refresh-hz`, `--native-factor`).
    Given,
    /// The display measurement.
    Measured,
    /// Neither: 60 Hz, factor 1.
    Assumed,
}

/// What the display measurement's run showed.
#[derive(Debug, Clone, PartialEq)]
pub struct Measured {
    /// The idle frames' rate, when they were paced.
    pub hz: Option<f64>,
    /// The factor the window reported.
    pub factor: Option<f32>,
    /// The renderer iced chose.
    pub backend: Option<String>,
}

/// The rate a display that paces no frames runs at.
pub const ASSUMED_HZ: f64 = 60.0;

impl Source {
    /// Its name, as `results.json` writes it.
    pub fn name(self) -> &'static str {
        match self {
            Source::Given => "given",
            Source::Measured => "measured",
            Source::Assumed => "assumed",
        }
    }
}

impl Display {
    /// The display from the command line's `hz` and `factor` and from
    /// `measured`, which is `Ok` with what the window showed or `Err` with
    /// why the level runs headless. What is given wins, and what the window
    /// did not show is assumed (60 Hz, factor 1).
    pub fn decide(
        hz: Option<f64>,
        factor: Option<f32>,
        measured: Result<Measured, String>,
    ) -> Display {
        let (headless, measured) = match measured {
            Ok(measured) => (None, measured),
            Err(reason) => (
                Some(reason),
                Measured {
                    hz: None,
                    factor: None,
                    backend: None,
                },
            ),
        };
        let (refresh_hz, refresh) = match (hz, measured.hz) {
            (Some(hz), _) => (hz, Source::Given),
            (None, Some(hz)) => (hz, Source::Measured),
            (None, None) => (ASSUMED_HZ, Source::Assumed),
        };
        let (native_factor, factor) = match (factor, measured.factor) {
            (Some(factor), _) => (factor, Source::Given),
            (None, Some(factor)) => (factor, Source::Measured),
            (None, None) => (1.0, Source::Assumed),
        };
        Display {
            headless,
            refresh_hz,
            refresh,
            native_factor,
            factor,
            backend: measured.backend,
        }
    }

    /// Whether iced's own choice is the software renderer (no GPU
    /// adapter), so the native profile is the software one.
    pub fn software(&self) -> bool {
        self.backend.as_deref() == Some("tiny-skia")
    }

    /// Whether windows open and their frames pace no rate, so every frame
    /// budget assumes the rate the level runs at.
    pub fn unpaced(&self) -> bool {
        self.headless.is_none() && self.refresh == Source::Assumed
    }

    /// One line on what the level runs at.
    pub fn describe(&self) -> String {
        let rate = match self.refresh {
            Source::Given => format!("{} Hz (given)", self.refresh_hz),
            Source::Measured => format!("{} Hz (measured from idle frames)", self.refresh_hz),
            Source::Assumed if self.headless.is_some() => {
                format!("{} Hz (virtual frames, headless)", self.refresh_hz)
            }
            Source::Assumed => format!(
                "{} Hz (assumed: the display paced no frames, as in a VM, Xvfb or a remote \
                 desktop; every frame budget assumes this rate)",
                self.refresh_hz
            ),
        };
        let factor = match self.factor {
            Source::Given => format!("scale factor {} (given)", self.native_factor),
            Source::Measured => format!("scale factor {} (the window's)", self.native_factor),
            Source::Assumed => format!("scale factor {} (assumed)", self.native_factor),
        };
        let backend = match (&self.backend, self.software()) {
            (Some(_), true) => {
                "iced chose the software renderer (no GPU adapter): the native profile is the \
                 software one"
                    .to_owned()
            }
            (Some(backend), false) => format!("iced chose {backend}"),
            (None, _) => "iced's renderer unknown".to_owned(),
        };
        let mode = self
            .headless
            .as_ref()
            .map_or_else(|| "windowed".to_owned(), |why| format!("headless: {why}"));
        format!("{mode}; {rate}; {factor}; {backend}")
    }
}

/// Why no window can open here, known without trying. On a Unix other
/// than macOS, that is when neither `WAYLAND_DISPLAY` nor `DISPLAY` names
/// a display (SSH, CI, a console). An empty value counts as unset, as
/// winit reads them.
pub fn unreachable(
    os: &str,
    wayland_display: Option<&OsStr>,
    display: Option<&OsStr>,
) -> Option<String> {
    let set = |var: Option<&OsStr>| var.is_some_and(|v| !v.is_empty());
    let windowing = matches!(os, "windows" | "macos" | "ios" | "android");
    (!windowing && !set(wayland_display) && !set(display)).then(|| {
        "no display: WAYLAND_DISPLAY and DISPLAY are unset (SSH, CI or a console)".to_owned()
    })
}

/// [`unreachable()`] for this process.
pub fn unreachable_here() -> Option<String> {
    unreachable(
        std::env::consts::OS,
        std::env::var_os("WAYLAND_DISPLAY").as_deref(),
        std::env::var_os("DISPLAY").as_deref(),
    )
}

/// What the display measurement's raw files under `raw` show; `None`
/// when no run wrote one.
pub fn read(raw: &Path) -> Option<Measured> {
    let files: Vec<PathBuf> = fs::read_dir(raw)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "jsonl"))
        .collect();
    let runs: Vec<Vec<Record>> = files
        .iter()
        .filter_map(|path| record::read(path).ok())
        .collect();
    if runs.is_empty() {
        return None;
    }
    let header = |run: &Vec<Record>| {
        run.iter().find_map(|record| match record {
            Record::Run(header) => Some(header.clone()),
            Record::Frame { .. }
            | Record::Input { .. }
            | Record::Stage { .. }
            | Record::Raster { .. }
            | Record::Mismatch { .. }
            | Record::Sys { .. }
            | Record::Readback { .. }
            | Record::Harness { .. }
            | Record::End { .. } => None,
        })
    };
    let headers: Vec<record::Run> = runs.iter().filter_map(header).collect();
    Some(Measured {
        hz: idle_hz(&runs),
        factor: window_factor(&headers),
        backend: headers.first().map(|header| header.backend.clone()),
    })
}

/// The refresh rate the runs' frames show: the median interval between
/// frames, in Hz, when at least half the intervals lie within 5 % of it
/// and it lies within 20–500 Hz. A display paces frames to its vblanks
/// (in the user's hover frames, 96 % lie within 5 % of 16.67 ms). Without
/// one, as under Xvfb, they spread (13–29 % here) and give no rate.
fn idle_hz(runs: &[Vec<Record>]) -> Option<f64> {
    let mut intervals = Vec::new();
    for records in runs {
        let at: Vec<f64> = records
            .iter()
            .filter_map(|record| {
                if let Record::Frame { at_ms, .. } = record {
                    Some(*at_ms)
                } else {
                    None
                }
            })
            .collect();
        intervals.extend(at.windows(2).map(|w| w[1] - w[0]).filter(|&d| d > 0.0));
    }
    let period = percentile(&intervals, 50.0)?;
    let paced = intervals
        .iter()
        .filter(|&&d| (d - period).abs() <= 0.05 * period)
        .count();
    let hz = 1e3 / period;
    (2 * paced >= intervals.len() && (20.0..=500.0).contains(&hz))
        .then(|| (hz * 100.0).round() / 100.0)
}

/// The factor most of the windows reported.
fn window_factor(headers: &[record::Run]) -> Option<f32> {
    let mut counts: Vec<(f32, usize)> = Vec::new();
    for factor in headers.iter().filter_map(|header| header.window_factor) {
        match counts.iter_mut().find(|(f, _)| (*f - factor).abs() < 1e-3) {
            Some((_, n)) => *n += 1,
            None => counts.push((factor, 1)),
        }
    }
    counts.into_iter().max_by_key(|&(_, n)| n).map(|(f, _)| f)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::{Display, Measured, Source, idle_hz, unreachable};
    use crate::record::Record;

    /// Frames every `period` ms from 0 for `n` frames.
    fn frames(period: f64, n: u32) -> Vec<Record> {
        (0..n)
            .map(|k| Record::Frame {
                frame: u64::from(k),
                at_ms: f64::from(k) * period,
                generation: 1,
                images: 1,
                missing: 0,
                previews: 0,
                uploaded: 0,
                update_ms: 0.1,
                draw_ms: Some(0.1),
            })
            .collect()
    }

    #[test]
    fn paced_idle_frames_give_the_refresh_rate_and_unpaced_ones_none() {
        assert_eq!(idle_hz(&[frames(1e3 / 144.0, 300)]), Some(144.0));
        assert_eq!(idle_hz(&[frames(1.0, 300)]), None, "1,000 Hz");
        // Frames 8, 13 and 17 ms apart in turn: paced by nothing (Xvfb).
        let mut jitter = frames(1.0, 300);
        let mut at = 0.0;
        for (k, record) in jitter.iter_mut().enumerate() {
            if let Record::Frame { at_ms, .. } = record {
                *at_ms = at;
            }
            at += [8.0, 13.0, 17.0][k % 3];
        }
        assert_eq!(idle_hz(&[jitter]), None, "unpaced");
        assert_eq!(idle_hz(&[]), None);
    }

    #[cfg(feature = "widget")]
    #[test]
    fn the_measurement_reads_the_rate_the_factor_and_the_backend() {
        use std::fs;

        use super::read;
        use crate::metrics::tests::header;
        use crate::record;

        let raw = std::env::temp_dir().join(format!("rustd3-display-{}", std::process::id()));
        fs::create_dir_all(&raw).unwrap();
        assert_eq!(read(&raw), None, "no run wrote a file");
        let mut records = vec![header("hover")];
        if let Record::Run(run) = &mut records[0] {
            run.window_factor = Some(1.25);
            run.backend = "tiny-skia".to_owned();
        }
        records.extend(frames(1e3 / 60.0, 200));
        record::write(&raw.join("hover__trace__f1__r1.jsonl"), &records).unwrap();
        assert_eq!(
            read(&raw),
            Some(Measured {
                hz: Some(60.0),
                factor: Some(1.25),
                backend: Some("tiny-skia".to_owned()),
            })
        );
        fs::remove_dir_all(&raw).unwrap();
    }

    #[test]
    fn what_is_given_wins_then_what_is_measured_then_60_hz_at_factor_1() {
        let measured = || {
            Ok(Measured {
                hz: Some(143.98),
                factor: Some(2.0),
                backend: Some("wgpu".to_owned()),
            })
        };
        let given = Display::decide(Some(144.0), Some(1.5), measured());
        assert_eq!(
            (
                given.refresh_hz,
                given.refresh,
                given.native_factor,
                given.factor
            ),
            (144.0, Source::Given, 1.5, Source::Given)
        );
        let paced = Display::decide(None, None, measured());
        assert_eq!(
            (
                paced.refresh_hz,
                paced.refresh,
                paced.native_factor,
                paced.factor
            ),
            (143.98, Source::Measured, 2.0, Source::Measured)
        );
        assert!(!paced.software() && !paced.unpaced() && paced.headless.is_none());
        // Xvfb, a VM: the window opens, its frames pace nothing.
        let unpaced = Display::decide(
            None,
            None,
            Ok(Measured {
                hz: None,
                factor: Some(1.0),
                backend: Some("wgpu".to_owned()),
            }),
        );
        assert_eq!(
            (unpaced.refresh_hz, unpaced.refresh),
            (60.0, Source::Assumed)
        );
        assert!(unpaced.unpaced());
        assert!(unpaced.describe().contains("every frame budget assumes"));
        // `--refresh-hz` on an unpaced display: given, not flagged.
        assert!(!Display::decide(Some(60.0), None, Ok(unpaced_measure())).unpaced());
        // No GPU adapter: iced falls back to tiny-skia.
        let software = Display::decide(
            None,
            None,
            Ok(Measured {
                hz: Some(60.0),
                factor: Some(1.0),
                backend: Some("tiny-skia".to_owned()),
            }),
        );
        assert!(software.software());
        // No window: headless, the reason kept, nothing flagged unpaced.
        let headless = Display::decide(None, Some(2.0), Err("no display".to_owned()));
        assert_eq!(headless.headless.as_deref(), Some("no display"));
        assert_eq!(
            (headless.refresh_hz, headless.native_factor, headless.factor),
            (60.0, 2.0, Source::Given)
        );
        assert!(!headless.unpaced() && !headless.software());
    }

    fn unpaced_measure() -> Measured {
        Measured {
            hz: None,
            factor: Some(1.0),
            backend: Some("wgpu".to_owned()),
        }
    }

    #[test]
    fn without_a_display_variable_a_unix_desktop_has_no_window_to_open() {
        let x = Some(OsStr::new(":0"));
        let wayland = Some(OsStr::new("wayland-1"));
        assert!(unreachable("linux", None, None).is_some(), "SSH, CI");
        assert!(unreachable("linux", Some(OsStr::new("")), Some(OsStr::new(""))).is_some());
        assert!(unreachable("freebsd", None, None).is_some());
        assert_eq!(unreachable("linux", None, x), None, "X11");
        assert_eq!(unreachable("linux", wayland, None), None, "Wayland");
        // Windows and macOS always have a window system to try.
        assert_eq!(unreachable("windows", None, None), None);
        assert_eq!(unreachable("macos", None, None), None);
    }
}
