//! The rule, registered before any run: budgets per cell, their verdicts
//! by 95% interval, the attribution of each miss, and the outcome that
//! says whether the image path suffices or a GPU data layer is
//! recommended.

use crate::metrics::{Half, Metrics, Part, Shares};
use crate::scenario::Scenario;
use crate::stats::{Interval, Verdict};

/// The rule and its operational readings, as the report and `list` print
/// them; fixed before any run and never changed after seeing data.
pub const READINGS: &str = "\
**Budgets.** p99 frame interval ≤ 1.5 × the display's frame period;
input-to-sharp p95 under 100 ms for pans and 150 ms for zooms; preview
share under 10% during pans. Each budget is met, missed or inconclusive by
its 95% interval; the 10-minute memory run is one run per configuration.

**Rule.** A missed budget is attributed to the UI thread (resolve, text,
M4, list), raster (the worker's critical path) or upload. The widget's
image path is tiles that pans translate, rendered on a pool, with
envelope fills and marker splatting; if any pan or zoom budget is missed
and attributed to raster or upload on it, a GPU data layer is
recommended. UI-thread misses route to moving list building off the UI
thread, not to a GPU layer. If every budget is met, the image path
suffices.

**Operational readings.**

1. With vsync, intervals are multiples of the period T, so p99 ≤ 1.5 T
   means the 99th-percentile frame was not a missed frame. A dropped
   frame is an interval > 1.5 T, counted as round(interval / T) − 1.
2. Pans are `pan-x`, `pan-y` and the pan half of `4k`; zooms are
   `zoom-lines`, `zoom-pixels` and the zoom half of `4k`.
3. A budget is missed when the 95% CI of the per-run statistic lies
   entirely above it, met when entirely below, inconclusive otherwise
   (rerun that cell with `--runs 10`).
4. Preview share = frames with `previews + missing > 0` / frames, from the
   first input to 300 ms after the last, per run. An outdated axis strip
   is a preview.
5. Attribution of a missed budget over its offending frames or latency
   windows: UI = `update_ms + draw_ms` (stage split from the probe);
   raster = Σ `Raster.duration` of the sharp frame's generation (the
   single worker's critical path; inline renders included and flagged);
   upload = Σ `Frame.uploaded` MB × the run's least-squares slope of frame
   interval on uploaded MB. The largest share names the bucket; the
   report shows all three.
";

/// The weak profile's budgets and how its outcome reads,
/// registered before any run.
pub const WEAK: &str = "\
**Weak profile.** The `weak` plan (`hover`, `pan-x` and
`zoom-lines` on `trace`, `pan-x` and `zoom-lines` on `scatter100k`, at the
native factor, five runs each) runs without a GPU (`--backend tiny-skia`)
and on wgpu pinned to 4 and to 2 cores, each its own report. Budgets: p99
frame interval ≤ 2 T; pan latency p95 ≤ 150 ms; zoom latency p95 ≤ 250
ms; preview share ≤ 25% during pans. Each is met, missed or inconclusive
by the readings above; a miss is attributed as above and becomes an item
for a later release. These profiles never
reopen the GPU-layer question, which stays with the desktop profile.
";

/// Which budgets a plan's cells are held to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// The verdict's: the desktop.
    Desktop,
    /// Weaker machines (the `weak` plan).
    Weak,
}

/// One of the budgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Budget {
    /// p99 frame interval ≤ 1.5 T.
    Frame,
    /// Input-to-sharp p95 under 100 ms on a pan.
    PanLatency,
    /// Input-to-sharp p95 under 150 ms on a zoom.
    ZoomLatency,
    /// Preview share under 10% during a pan.
    Previews,
}

/// What a frame budget reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reading {
    /// The intervals between frames on a display (reading 1).
    Display,
    /// The cost of producing each frame offscreen, its UI work and its
    /// composition: a frame must be produced within the budget.
    Offscreen,
}

/// One budget evaluated on one cell's runs.
#[derive(Debug, Clone, PartialEq)]
pub struct Check {
    /// Which budget.
    pub budget: Budget,
    /// Which part of the runs it reads.
    pub half: Half,
    /// Its limit (ms, or a share).
    pub limit: f64,
    /// The per-run statistic across runs.
    pub interval: Option<Interval>,
    /// Met, missed or inconclusive.
    pub verdict: Verdict,
    /// The shares over the offending frames or windows, summed over runs:
    /// for a miss, and for a single run over the budget (no verdict, but
    /// what a miss would be attributed to).
    pub shares: Shares,
}

/// What the rule concludes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Every budget met.
    Sufficient,
    /// Only UI-thread misses.
    UiBound,
    /// A pan or zoom budget missed on raster or upload.
    RasterBound,
    /// No miss decides it and some budget is inconclusive.
    Pending,
    /// The cells hold no budget (`cold`, `soak`).
    Unmeasured,
}

impl Budget {
    /// Its name in the report.
    pub fn name(self) -> &'static str {
        match self {
            Budget::Frame => "frame p99",
            Budget::PanLatency => "pan latency p95",
            Budget::ZoomLatency => "zoom latency p95",
            Budget::Previews => "preview share",
        }
    }

    /// The budgets `scenario` is held to, with the half each reads.
    pub fn of(scenario: Scenario) -> Vec<(Budget, Half)> {
        match scenario {
            Scenario::Hover | Scenario::Live | Scenario::Resize => {
                vec![(Budget::Frame, Half::Whole)]
            }
            Scenario::PanX | Scenario::PanY => vec![
                (Budget::Frame, Half::Whole),
                (Budget::PanLatency, Half::Whole),
                (Budget::Previews, Half::Whole),
            ],
            Scenario::ZoomLines | Scenario::ZoomPixels => vec![
                (Budget::Frame, Half::Whole),
                (Budget::ZoomLatency, Half::Whole),
            ],
            Scenario::FourK => vec![
                (Budget::Frame, Half::Whole),
                (Budget::PanLatency, Half::Pan),
                (Budget::Previews, Half::Pan),
                (Budget::ZoomLatency, Half::Zoom),
            ],
            // `drag-resize` came after the rule was registered: report-only.
            Scenario::Cold | Scenario::Soak | Scenario::DragResize => Vec::new(),
        }
    }
}

impl Profile {
    /// The profile of the plan named `plan`: weak for the weak and the
    /// software plans.
    pub fn of(plan: &str) -> Profile {
        if plan == crate::plan::Plan::Weak.name() || plan == crate::plan::Plan::Software.name() {
            Profile::Weak
        } else {
            Profile::Desktop
        }
    }

    /// Its name on the command line (`--budgets`) and in `meta.json`.
    pub fn name(self) -> &'static str {
        match self {
            Profile::Desktop => "desktop",
            Profile::Weak => "weak",
        }
    }

    /// The profile named `name`.
    pub fn parse(name: &str) -> Option<Profile> {
        [Profile::Desktop, Profile::Weak]
            .into_iter()
            .find(|p| p.name() == name)
    }

    /// `budget`'s limit at a frame period of `period_ms`: ms, or a share.
    pub fn limit(self, budget: Budget, period_ms: f64) -> f64 {
        match (self, budget) {
            (Profile::Desktop, Budget::Frame) => 1.5 * period_ms,
            (Profile::Desktop, Budget::PanLatency) => 100.0,
            (Profile::Desktop, Budget::ZoomLatency) => 150.0,
            (Profile::Desktop, Budget::Previews) => 0.10,
            (Profile::Weak, Budget::Frame) => 2.0 * period_ms,
            (Profile::Weak, Budget::PanLatency) => 150.0,
            (Profile::Weak, Budget::ZoomLatency) => 250.0,
            (Profile::Weak, Budget::Previews) => 0.25,
        }
    }
}

impl Check {
    /// `budget` over `half` of `runs` at a frame period of `period_ms`,
    /// held to `profile`'s limit, its frame budget read as `reading` says.
    pub fn evaluate(
        (budget, half): (Budget, Half),
        runs: &[Metrics],
        (period_ms, reading): (f64, Reading),
        profile: Profile,
    ) -> Check {
        let limit = profile.limit(budget, period_ms);
        let values: Vec<f64> = runs
            .iter()
            .filter_map(|run| match (budget, reading) {
                (Budget::Frame, Reading::Display) => run.interval(99.0),
                (Budget::Frame, Reading::Offscreen) => run.part(Part::cost, 99.0),
                (Budget::PanLatency | Budget::ZoomLatency, _) => run.latency(half, 95.0),
                (Budget::Previews, _) => run.preview_share(half),
            })
            .collect();
        let interval = Interval::of(&values);
        let verdict = interval.map_or(Verdict::Inconclusive, |i| i.verdict(limit));
        let mut shares = Shares::default();
        let single_over = interval.is_some_and(|i| i.ci.is_none() && i.mean > limit);
        if verdict == Verdict::Missed || single_over {
            for run in runs {
                shares.add(match (budget, reading) {
                    (Budget::Frame, Reading::Display) => run.interval_shares(),
                    (Budget::Frame, Reading::Offscreen) => run.cost_shares(limit),
                    (Budget::PanLatency | Budget::ZoomLatency, _) => {
                        run.latency_shares(half, limit)
                    }
                    (Budget::Previews, _) => run.preview_shares(half),
                });
            }
        }
        Check {
            budget,
            half,
            limit,
            interval,
            verdict,
            shares,
        }
    }
}

/// Whether a budget of `scenario` is a pan or zoom budget (reading 2):
/// any budget of a pan or zoom scenario.
fn pan_or_zoom(scenario: Scenario) -> bool {
    matches!(
        scenario,
        Scenario::PanX
            | Scenario::PanY
            | Scenario::ZoomLines
            | Scenario::ZoomPixels
            | Scenario::FourK
    )
}

impl Check {
    /// Whether the report attributes it: a miss, or one run over budget.
    pub fn attributed(&self) -> bool {
        self.verdict == Verdict::Missed
            || self
                .interval
                .is_some_and(|i| i.ci.is_none() && i.mean > self.limit)
    }
}

impl Outcome {
    /// The outcome of every cell's checks.
    pub fn of(checks: &[(Scenario, Check)]) -> Outcome {
        if checks.is_empty() {
            return Outcome::Unmeasured;
        }
        let missed = |c: &&(Scenario, Check)| c.1.verdict == Verdict::Missed;
        let raster = checks.iter().filter(missed).any(|(scenario, c)| {
            pan_or_zoom(*scenario) && matches!(c.shares.bucket(), "raster" | "upload")
        });
        if raster {
            Outcome::RasterBound
        } else if checks
            .iter()
            .filter(missed)
            .any(|(_, c)| c.shares.bucket() == "ui")
        {
            Outcome::UiBound
        } else if checks
            .iter()
            .any(|(_, c)| c.verdict == Verdict::Inconclusive)
        {
            Outcome::Pending
        } else {
            Outcome::Sufficient
        }
    }

    /// The outcome's line in a report of `profile`: on the desktop a
    /// raster or upload miss recommends the GPU data layer; the weak
    /// profiles never decide it.
    pub fn line(self, profile: Profile) -> &'static str {
        match (profile, self) {
            (Profile::Desktop, Outcome::Sufficient) => "image path sufficient — no GPU data layer",
            (Profile::Desktop, Outcome::UiBound) => {
                "UI-thread bound → move list building off the UI thread (0.2); no GPU data layer"
            }
            (Profile::Desktop, Outcome::RasterBound) => {
                "raster/upload bound on the tiled image path → GPU data layer recommended"
            }
            (Profile::Weak, Outcome::Sufficient) => "weak profile: every budget met",
            (Profile::Weak, Outcome::UiBound) => {
                "weak profile: missed on the UI thread → an item for a later release"
            }
            (Profile::Weak, Outcome::RasterBound) => {
                "weak profile: missed on raster/upload → an item for a later release (never the \
                 GPU-layer question)"
            }
            (Profile::Desktop | Profile::Weak, Outcome::Pending) => {
                "no verdict yet: some budget is inconclusive and none missed; rerun the \
                 inconclusive cells with --runs 10"
            }
            (Profile::Desktop | Profile::Weak, Outcome::Unmeasured) => {
                "no verdict: the selected cells hold no budget"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Budget, Check, Outcome, Profile, Reading};
    use crate::metrics::tests::{frame, header, input};
    use crate::metrics::{Half, Metrics, Shares};
    use crate::scenario::Scenario;
    use crate::stats::Verdict;

    #[test]
    fn the_p99_budget_is_one_and_a_half_periods() {
        // 144 Hz: T = 6.94 ms, budget 10.4 ms; 60 Hz: T = 16.67, 25.0 ms.
        for (hz, budget) in [(144.0, 10.42), (60.0, 25.0)] {
            let frame = (Budget::Frame, Half::Whole);
            let check = Check::evaluate(frame, &[], (1e3 / hz, Reading::Display), Profile::Desktop);
            assert!((check.limit - budget).abs() < 0.01, "{}", check.limit);
            assert_eq!(check.verdict, Verdict::Inconclusive);
        }
    }

    #[test]
    fn the_weak_profile_has_d22_4s_budgets() {
        let t = 1e3 / 60.0;
        let limits = [
            Budget::Frame,
            Budget::PanLatency,
            Budget::ZoomLatency,
            Budget::Previews,
        ]
        .map(|budget| Profile::Weak.limit(budget, t));
        assert_eq!(limits, [2.0 * t, 150.0, 250.0, 0.25]);
        assert_eq!(Profile::of("weak"), Profile::Weak);
        assert_eq!(Profile::of("default"), Profile::Desktop);
        for profile in [Profile::Desktop, Profile::Weak] {
            assert_eq!(Profile::parse(profile.name()), Some(profile));
        }
        assert_eq!(Profile::parse("laptop"), None);
        // Its outcome never asks for the GPU layer.
        let line = Outcome::RasterBound.line(Profile::Weak);
        assert!(
            line.contains("a later release") && !line.contains("recommended"),
            "{line}"
        );
    }

    #[test]
    fn offscreen_the_frame_budget_reads_each_frames_cost() {
        // The offscreen run's frames cost 7, 5.5, 6.5 and 6.5 ms 16 ms
        // apart: its intervals are the pacer's, its p99 cost 7 ms.
        let runs: Vec<Metrics> = (0..2)
            .map(|_| Metrics::of(&crate::metrics::tests::offscreen()).unwrap())
            .collect();
        let frame = (Budget::Frame, Half::Whole);
        let cost = Check::evaluate(
            frame,
            &runs,
            (1e3 / 60.0, Reading::Offscreen),
            Profile::Weak,
        );
        let mean = cost.interval.map(|i| i.mean).unwrap();
        assert!((mean - 6.985).abs() < 1e-9, "{mean}");
        assert_eq!(cost.verdict, Verdict::Met);
        let shown = Check::evaluate(frame, &runs, (1e3 / 60.0, Reading::Display), Profile::Weak);
        assert!(shown.interval.map(|i| i.mean).unwrap() > 15.0);
        // Over a 5 ms budget: missed, composition in the upload share.
        let tight = Check::evaluate(frame, &runs, (2.5, Reading::Offscreen), Profile::Weak);
        assert_eq!(tight.verdict, Verdict::Missed);
        assert_eq!(tight.shares.bucket(), "upload");
        assert_eq!(Profile::of("software"), Profile::Weak);
    }

    #[test]
    fn a_raster_bound_pan_miss_after_phase_31_recommends_the_gpu_layer() {
        // Five runs whose pan latencies are all about 200 ms, raster-bound.
        let runs: Vec<Metrics> = (0..5)
            .map(|k| {
                let records = [
                    header("pan-x"),
                    frame(1, 1000.0, 7, 9),
                    frame(2, 1200.0 + k as f64, 7, 0),
                    input(1000.0, 1),
                    crate::record::Record::Raster {
                        frame: 2,
                        generation: 7,
                        cell: Some(4),
                        tile: None,
                        pixels: 1_000_000,
                        work: 100,
                        queued_ms: 0.0,
                        ms: 190.0,
                        inline: false,
                    },
                ];
                Metrics::of(&records).unwrap()
            })
            .collect();
        let pan = (Budget::PanLatency, Half::Whole);
        let check = Check::evaluate(pan, &runs, (16.67, Reading::Display), Profile::Desktop);
        assert_eq!(check.verdict, Verdict::Missed);
        assert_eq!(check.shares.bucket(), "raster");
        let outcome = Outcome::of(&[(Scenario::PanX, check.clone())]);
        assert_eq!(outcome, Outcome::RasterBound);
        assert!(
            outcome
                .line(Profile::Desktop)
                .contains("GPU data layer recommended")
        );
        // The weak profile's budget is 150 ms: the same runs miss it too.
        let weak = Check::evaluate(pan, &runs, (16.67, Reading::Display), Profile::Weak);
        assert_eq!((weak.limit, weak.verdict), (150.0, Verdict::Missed));
        // The same miss on the UI thread routes elsewhere.
        let ui = Check {
            shares: Shares {
                ui_ms: 1e3,
                ..check.shares
            },
            ..check.clone()
        };
        assert_eq!(Outcome::of(&[(Scenario::PanX, ui)]), Outcome::UiBound);
        let met = Check {
            verdict: Verdict::Met,
            ..check
        };
        assert_eq!(Outcome::of(&[(Scenario::PanX, met)]), Outcome::Sufficient);
    }
}
