//! The decision rule, registered before any run: budgets per cell, their
//! verdicts by 95% interval, the attribution of each miss, and the
//! outcome that decides Phase 31.

use crate::metrics::{Half, Metrics, Shares};
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
M4, list), raster (the worker's critical path) or upload. If any pan or
zoom budget is missed and attributed to raster or upload, Phase 31 is
built (tiles, translation, the pool, envelope fills, splatting) and this
benchmark is rerun; a GPU data layer is recommended only if that rerun
still misses. UI-thread misses route to moving list building off the UI
thread, not to a GPU layer. If every budget is met, Phase 31 is skipped.

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
            Scenario::Cold | Scenario::Soak => Vec::new(),
        }
    }
}

impl Check {
    /// `budget` over `half` of `runs` at a frame period of `period_ms`.
    pub fn evaluate(budget: Budget, half: Half, runs: &[Metrics], period_ms: f64) -> Check {
        let limit = match budget {
            Budget::Frame => 1.5 * period_ms,
            Budget::PanLatency => 100.0,
            Budget::ZoomLatency => 150.0,
            Budget::Previews => 0.10,
        };
        let values: Vec<f64> = runs
            .iter()
            .filter_map(|run| match budget {
                Budget::Frame => run.interval(99.0),
                Budget::PanLatency | Budget::ZoomLatency => run.latency(half, 95.0),
                Budget::Previews => run.preview_share(half),
            })
            .collect();
        let interval = Interval::of(&values);
        let verdict = interval.map_or(Verdict::Inconclusive, |i| i.verdict(limit));
        let mut shares = Shares::default();
        let single_over = interval.is_some_and(|i| i.ci.is_none() && i.mean > limit);
        if verdict == Verdict::Missed || single_over {
            for run in runs {
                shares.add(match budget {
                    Budget::Frame => run.interval_shares(),
                    Budget::PanLatency | Budget::ZoomLatency => run.latency_shares(half, limit),
                    Budget::Previews => run.preview_shares(half),
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

    /// The outcome's line in the report.
    pub fn line(self) -> &'static str {
        match self {
            Outcome::Sufficient => "image path sufficient — Phase 31 skipped",
            Outcome::UiBound => {
                "UI-thread bound → move list building off the UI thread; Phase 31 skipped"
            }
            Outcome::RasterBound => {
                "raster/upload bound → build Phase 31 (tiles, translation, pool, envelope fills, \
                 splatting) and rerun"
            }
            Outcome::Pending => {
                "no verdict yet: some budget is inconclusive and none missed; rerun the \
                 inconclusive cells with --runs 10"
            }
            Outcome::Unmeasured => "no verdict: the selected cells hold no budget",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Budget, Check, Outcome};
    use crate::metrics::tests::{frame, header, input};
    use crate::metrics::{Half, Metrics, Shares};
    use crate::scenario::Scenario;
    use crate::stats::Verdict;

    #[test]
    fn the_p99_budget_is_one_and_a_half_periods() {
        // 144 Hz: T = 6.94 ms, budget 10.4 ms; 60 Hz: T = 16.67, 25.0 ms.
        for (hz, budget) in [(144.0, 10.42), (60.0, 25.0)] {
            let check = Check::evaluate(Budget::Frame, Half::Whole, &[], 1e3 / hz);
            assert!((check.limit - budget).abs() < 0.01, "{}", check.limit);
            assert_eq!(check.verdict, Verdict::Inconclusive);
        }
    }

    #[test]
    fn a_raster_bound_pan_miss_builds_phase_31() {
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
        let check = Check::evaluate(Budget::PanLatency, Half::Whole, &runs, 16.67);
        assert_eq!(check.verdict, Verdict::Missed);
        assert_eq!(check.shares.bucket(), "raster");
        assert_eq!(
            Outcome::of(&[(Scenario::PanX, check.clone())]),
            Outcome::RasterBound
        );
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
