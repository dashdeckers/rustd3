//! [`Plan`]: which cells (scenario × workload × factor) a run covers, how
//! often, how long each scenario lasts and how much data each workload
//! holds.

use std::fmt;

use iced::Size;

use crate::scenario::Scenario;
use crate::workload::Workload;

/// A registered set of cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    /// Every scenario × workload once at factor 2, durations ÷ 5, data ÷
    /// 100, `soak` 20 s: the headless check.
    Smoke,
    /// `default`'s cells twice each; not valid for the verdict.
    Quick,
    /// The verdict's cells: five runs each, `soak` once.
    Default,
    /// D22.4's weaker machines: `hover`, `pan-x` and `zoom-lines` on
    /// `trace`, `pan-x` and `zoom-lines` on `scatter100k`, at the native
    /// factor, five runs each, held to the weak profile's budgets; run
    /// without a GPU (`--backend tiny-skia`) and on 4 and 2 cores.
    Weak,
    /// Every scenario × workload × factor, five runs each.
    Full,
}

/// One scenario × workload × factor, run `runs` times; a workload that
/// does not support the scenario makes it "n/a" (reported, never run).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    /// What happens.
    pub scenario: Scenario,
    /// To which figure.
    pub workload: Workload,
    /// At which scale factor.
    pub factor: f32,
    /// How many cold processes.
    pub runs: u32,
}

/// Restrictions the command line puts on a plan.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Selection {
    /// Only these scenarios.
    pub scenarios: Option<Vec<Scenario>>,
    /// Only these workloads.
    pub workloads: Option<Vec<Workload>>,
    /// Only these factors; `smoke` runs at these instead of 2.
    pub factors: Option<Vec<f32>>,
    /// Runs per cell (every cell but `soak`'s).
    pub runs: Option<u32>,
}

/// The four factors the plans measure.
pub const FACTORS: [f32; 4] = [1.0, 1.25, 1.5, 2.0];

/// The workloads besides `trace` that `quick` and `default` run `hover`,
/// `pan-x` and `zoom-lines` on, as D22.1 registered them before any run;
/// `wheels` (Phase 34) runs in `smoke` and `full`.
const REGISTERED: [Workload; 4] = [
    Workload::Scatter100k,
    Workload::Scatter1m,
    Workload::Grid16,
    Workload::LabelChurn,
];

/// Process start, data, first frame and the 2 s settle, per run: the
/// estimate's allowance on real hardware.
const OVERHEAD_MS: f64 = 4_000.0;

impl Plan {
    /// Every plan.
    pub const ALL: [Plan; 5] = [
        Plan::Smoke,
        Plan::Quick,
        Plan::Default,
        Plan::Weak,
        Plan::Full,
    ];

    /// The plan's name on the command line.
    pub fn name(self) -> &'static str {
        match self {
            Plan::Smoke => "smoke",
            Plan::Quick => "quick",
            Plan::Default => "default",
            Plan::Weak => "weak",
            Plan::Full => "full",
        }
    }

    /// The plan named `name`.
    pub fn parse(name: &str) -> Option<Plan> {
        Plan::ALL.into_iter().find(|p| p.name() == name)
    }

    /// The plan's cells with the display's `native` factor, restricted by
    /// `selection`, in report order.
    pub fn cells(self, native: f32, selection: &Selection) -> Vec<Cell> {
        let runs = match self {
            Plan::Smoke => 1,
            Plan::Quick => 2,
            Plan::Default | Plan::Weak | Plan::Full => 5,
        };
        let cell = |scenario, workload, factor| Cell {
            scenario,
            workload,
            factor,
            runs: if scenario == Scenario::Soak { 1 } else { runs },
        };
        let mut cells = Vec::new();
        match self {
            Plan::Smoke => {
                let factors = selection.factors.clone().unwrap_or_else(|| vec![2.0]);
                for scenario in Scenario::ALL {
                    for workload in Workload::ALL {
                        for &factor in &factors {
                            cells.push(cell(scenario, workload, factor));
                        }
                    }
                }
            }
            Plan::Quick | Plan::Default => {
                for scenario in Scenario::ALL {
                    if scenario != Scenario::Soak {
                        cells.push(cell(scenario, Workload::Trace, native));
                    }
                }
                for scenario in [Scenario::Hover, Scenario::PanX, Scenario::ZoomLines] {
                    for workload in REGISTERED {
                        cells.push(cell(scenario, workload, native));
                    }
                }
                for scenario in [Scenario::PanX, Scenario::ZoomLines] {
                    for factor in FACTORS {
                        if factor != native {
                            cells.push(cell(scenario, Workload::Trace, factor));
                        }
                    }
                }
                cells.push(cell(Scenario::Soak, Workload::Trace, native));
            }
            Plan::Weak => {
                for scenario in [Scenario::Hover, Scenario::PanX, Scenario::ZoomLines] {
                    cells.push(cell(scenario, Workload::Trace, native));
                }
                for scenario in [Scenario::PanX, Scenario::ZoomLines] {
                    cells.push(cell(scenario, Workload::Scatter100k, native));
                }
            }
            Plan::Full => {
                let mut factors = FACTORS.to_vec();
                if !factors.contains(&native) {
                    factors.push(native);
                }
                for scenario in Scenario::ALL {
                    for workload in Workload::ALL {
                        for &factor in &factors {
                            cells.push(cell(scenario, workload, factor));
                        }
                    }
                }
            }
        }
        cells.retain(|cell| {
            keep(&selection.scenarios, &cell.scenario)
                && keep(&selection.workloads, &cell.workload)
                && (self == Plan::Smoke || keep(&selection.factors, &cell.factor))
        });
        if let Some(runs) = selection.runs {
            for cell in &mut cells {
                if cell.scenario != Scenario::Soak {
                    cell.runs = runs;
                }
            }
        }
        cells
    }

    /// How long `scenario` lasts in this plan, ms.
    pub fn duration_ms(self, scenario: Scenario) -> f64 {
        match (self, scenario) {
            (Plan::Smoke, Scenario::Soak) => 20_000.0,
            (Plan::Smoke, _) => scenario.duration_ms() / 5.0,
            (Plan::Quick | Plan::Default | Plan::Weak | Plan::Full, _) => scenario.duration_ms(),
        }
    }

    /// The workloads' data size in this plan (1 = full).
    pub fn data_scale(self) -> f64 {
        match self {
            Plan::Smoke => 0.01,
            Plan::Quick | Plan::Default | Plan::Weak | Plan::Full => 1.0,
        }
    }

    /// The estimated wall time of `cells` on real hardware, s: each run's
    /// duration plus 4 s of start, data and settle; "n/a" cells cost
    /// nothing.
    pub fn estimate_s(self, cells: &[Cell]) -> f64 {
        cells
            .iter()
            .filter(|cell| cell.workload.supports(cell.scenario))
            .map(|cell| f64::from(cell.runs) * (self.duration_ms(cell.scenario) + OVERHEAD_MS))
            .sum::<f64>()
            / 1e3
    }
}

/// Whether `list` (none: everything) admits `item`.
fn keep<T: PartialEq>(list: &Option<Vec<T>>, item: &T) -> bool {
    list.as_ref().is_none_or(|list| list.contains(item))
}

impl Cell {
    /// Whether the workload supports the scenario (else "n/a").
    pub fn applies(&self) -> bool {
        self.workload.supports(self.scenario)
    }

    /// The window's logical size on a display of factor `native`: the
    /// scenario's; at another factor k (the factor cells) the standard
    /// window's 1600 × 1000 device px, 1600 / k × 1000 / k logical px, so
    /// it fits a 1080p display (D22.3). A workload with a size of its own
    /// (the wheel figure) keeps it wherever the scenario does not ask for
    /// the 4k window.
    pub fn window(&self, native: f32) -> Size {
        let window = self.scenario.window(self.factor);
        match self.scenario {
            Scenario::FourK => window,
            Scenario::Hover
            | Scenario::PanX
            | Scenario::PanY
            | Scenario::ZoomLines
            | Scenario::ZoomPixels
            | Scenario::Live
            | Scenario::Resize
            | Scenario::Cold
            | Scenario::Soak => match self.workload.size() {
                Some(size) => size,
                None if self.factor != native => {
                    Size::new(window.width / self.factor, window.height / self.factor)
                }
                None => window,
            },
        }
    }

    /// The cell's key in file names: `<scenario>__<workload>__f<factor>`.
    pub fn key(&self) -> String {
        format!(
            "{}__{}__f{}",
            self.scenario.name(),
            self.workload.name(),
            self.factor
        )
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} × {} × {}",
            self.scenario.name(),
            self.workload.name(),
            self.factor
        )
    }
}

#[cfg(test)]
mod tests {
    use iced::Size;

    use super::{Plan, Selection};
    use crate::scenario::Scenario;
    use crate::workload::Workload;

    #[test]
    fn default_plan_has_the_pre_registered_cells() {
        let cells = Plan::Default.cells(2.0, &Selection::default());
        assert_eq!(cells.len(), 9 + 12 + 6 + 1);
        let runs: u32 = cells.iter().map(|c| c.runs).sum();
        assert_eq!(runs, 27 * 5 + 1);
        // (C): pan-x and zoom-lines on the trace at the three other factors.
        let other: Vec<f32> = cells
            .iter()
            .filter(|c| c.scenario == Scenario::PanX && c.workload == Workload::Trace)
            .map(|c| c.factor)
            .collect();
        assert_eq!(other, [2.0, 1.0, 1.25, 1.5]);
        assert!(cells.iter().all(|c| c.applies()));
        // About 46 minutes: 135 runs of about 16 s and the 10-minute soak.
        let minutes = Plan::Default.estimate_s(&cells) / 60.0;
        assert!((44.0..48.0).contains(&minutes), "{minutes}");
        // A native factor outside the four adds one more factor to (C).
        assert_eq!(
            Plan::Default.cells(1.75, &Selection::default()).len(),
            9 + 12 + 8 + 1
        );
        assert_eq!(Plan::Quick.cells(2.0, &Selection::default())[0].runs, 2);
    }

    #[test]
    fn factor_cells_keep_1600_by_1000_device_px() {
        // D22.3: on a display of factor 1 the factor cells at 1.25, 1.5
        // and 2 ask for 1600 × 1000 device px, and the native cells keep
        // the standard window.
        let cells = Plan::Default.cells(1.0, &Selection::default());
        for cell in &cells {
            let window = cell.window(1.0);
            let device = (window.width * cell.factor, window.height * cell.factor);
            if cell.scenario == Scenario::FourK {
                assert_eq!(device, (3840.0, 2160.0), "{cell}");
            } else {
                assert!(
                    (device.0 - 1600.0).abs() < 1e-3 && (device.1 - 1000.0).abs() < 1e-3,
                    "{cell}: {device:?}"
                );
            }
        }
        let at = |factor: f32| {
            cells
                .iter()
                .find(|c| c.scenario == Scenario::PanX && c.factor == factor)
                .expect("a pan cell")
                .window(1.0)
        };
        assert_eq!(at(1.0), Size::new(1600.0, 1000.0));
        assert_eq!(at(1.25), Size::new(1280.0, 800.0));
        assert_eq!(at(2.0), Size::new(800.0, 500.0));
        // On a display of factor 2 the native cells keep 1600 × 1000
        // logical px.
        let native = Plan::Default.cells(2.0, &Selection::default());
        assert_eq!(native[1].window(2.0), Size::new(1600.0, 1000.0));
    }

    #[test]
    fn weak_plan_has_d22_4s_cells() {
        let cells = Plan::Weak.cells(1.25, &Selection::default());
        let named: Vec<(Scenario, Workload)> =
            cells.iter().map(|c| (c.scenario, c.workload)).collect();
        assert_eq!(
            named,
            [
                (Scenario::Hover, Workload::Trace),
                (Scenario::PanX, Workload::Trace),
                (Scenario::ZoomLines, Workload::Trace),
                (Scenario::PanX, Workload::Scatter100k),
                (Scenario::ZoomLines, Workload::Scatter100k),
            ]
        );
        assert!(
            cells
                .iter()
                .all(|c| c.factor == 1.25 && c.runs == 5 && c.applies())
        );
        assert_eq!(Plan::Weak.data_scale(), 1.0);
        // About 6.5 minutes a profile on real hardware.
        let minutes = Plan::Weak.estimate_s(&cells) / 60.0;
        assert!((6.0..7.0).contains(&minutes), "{minutes}");
    }

    #[test]
    fn smoke_plan_covers_every_scenario_and_workload() {
        let cells = Plan::Smoke.cells(2.0, &Selection::default());
        assert_eq!(cells.len(), 60);
        // `live` and `soak` need recordings: the scatters and `wheels` have
        // none; `pan-x` needs an x that pans, which `wheels` lacks.
        assert_eq!(cells.iter().filter(|c| c.applies()).count(), 53);
        assert!(cells.iter().all(|c| c.factor == 2.0 && c.runs == 1));
        assert_eq!(Plan::Smoke.duration_ms(Scenario::PanX), 2_400.0);
        assert_eq!(Plan::Smoke.duration_ms(Scenario::Soak), 20_000.0);
        assert_eq!(Plan::Smoke.data_scale(), 0.01);
        let one = Selection {
            workloads: Some(vec![Workload::Trace]),
            factors: Some(vec![1.0, 1.5]),
            ..Selection::default()
        };
        let cells = Plan::Smoke.cells(2.0, &one);
        assert_eq!(cells.len(), 20);
        assert!(cells.iter().all(|c| c.factor != 2.0));
    }

    #[test]
    fn the_wheel_figure_keeps_its_size_except_in_the_4k_window() {
        let cells = Plan::Smoke.cells(2.0, &Selection::default());
        let window = |scenario, workload| {
            let cell = cells
                .iter()
                .find(|c| c.scenario == scenario && c.workload == workload)
                .expect("a cell");
            cell.window(2.0)
        };
        assert_eq!(
            window(Scenario::PanX, Workload::Wheels),
            Size::new(960.0, 760.0)
        );
        assert_eq!(
            window(Scenario::Resize, Workload::Wheels),
            Size::new(960.0, 760.0)
        );
        assert_eq!(
            window(Scenario::FourK, Workload::Wheels),
            Size::new(1920.0, 1080.0)
        );
        assert_eq!(
            window(Scenario::PanX, Workload::Trace),
            Size::new(1600.0, 1000.0)
        );
        // The verdict's plan is the one registered: no wheel cell.
        let default = Plan::Default.cells(2.0, &Selection::default());
        assert!(default.iter().all(|c| c.workload != Workload::Wheels));
    }
}
