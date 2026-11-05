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
    /// The display measurement that starts every level: one `hover` on
    /// `trace` at the native factor for 3 s, with data ÷ 100. Its frames
    /// give the refresh rate, and its window gives the factor and the
    /// backend iced chose.
    Display,
    /// The quick level's cells: `hover`, `pan-x` and `zoom-lines` on
    /// `trace` and `scatter100k`, and `drag-resize` on `trace`, at the
    /// native factor, three runs each (report-only). Its software profile
    /// selects `pan-x` and `zoom-lines` on `trace`.
    Quick,
    /// The verdict's cells: five runs each, `soak` once.
    Default,
    /// Weaker machines: `hover`, `pan-x` and `zoom-lines` on `trace`, and
    /// `pan-x` and `zoom-lines` on `scatter100k`, at the native factor,
    /// five runs each, held to the weak profile's budgets. It runs without
    /// a GPU (`--backend tiny-skia`) and on 4 and 2 cores.
    Weak,
    /// The zooms, pans and window drags, five runs each, at the native
    /// factor unless named:
    /// - `zoom-lines` and `zoom-pixels` on `trace`;
    /// - `zoom-lines` on `trace` at the other three factors, and on
    ///   `scatter100k`, `scatter1m`, `grid16` and `label-churn`;
    /// - `pan-x` on `trace` and `scatter1m`;
    /// - `drag-resize` on `trace` and `grid16`.
    ///
    /// It is report-only. It is held to the desktop budgets to show where
    /// the cells stand, but it is not the rule's run.
    Focus,
    /// Every scenario × workload × factor, five runs each.
    Full,
    /// The software path offscreen: `hover`, `pan-x`, `zoom-lines` and
    /// `drag-resize` on `trace`, `scatter100k` and `grid16` at factors 1
    /// and 2, three runs each. It runs headless on iced's software
    /// renderer, with every frame composed into an offscreen pixmap, and is
    /// held to the weak budgets (report-only). Its cells keep 1600 × 1000
    /// logical px at both factors (it runs with a native factor of 2, so
    /// neither is shrunk).
    Software,
}

/// One scenario × workload × factor, run `runs` times. A workload that
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
/// `pan-x` and `zoom-lines` on, as registered before any run. `wheels`
/// runs in `smoke` and `full`.
const REGISTERED: [Workload; 4] = [
    Workload::Scatter100k,
    Workload::Scatter1m,
    Workload::Grid16,
    Workload::LabelChurn,
];

/// The estimate's allowance per run on real hardware for process start,
/// data, the first frame and the 2 s settle.
pub const OVERHEAD_MS: f64 = 4_000.0;

/// The software plan's factors.
pub const SOFTWARE_FACTORS: [f32; 2] = [1.0, 2.0];

/// The software plan's workloads.
const SOFTWARE_WORKLOADS: [Workload; 3] =
    [Workload::Trace, Workload::Scatter100k, Workload::Grid16];

/// The software plan's scenarios.
const SOFTWARE_SCENARIOS: [Scenario; 4] = [
    Scenario::Hover,
    Scenario::PanX,
    Scenario::ZoomLines,
    Scenario::DragResize,
];

/// How long the display measurement hovers, in ms. With the driver's 2 s
/// settle, that is about 300 frame intervals at 60 Hz, or 150 at 30 Hz.
const DISPLAY_MS: f64 = 3_000.0;

impl Plan {
    /// Every plan.
    pub const ALL: [Plan; 8] = [
        Plan::Smoke,
        Plan::Display,
        Plan::Quick,
        Plan::Default,
        Plan::Weak,
        Plan::Focus,
        Plan::Full,
        Plan::Software,
    ];

    /// The plan's name on the command line.
    pub fn name(self) -> &'static str {
        match self {
            Plan::Smoke => "smoke",
            Plan::Display => "display",
            Plan::Quick => "quick",
            Plan::Default => "default",
            Plan::Weak => "weak",
            Plan::Focus => "focus",
            Plan::Full => "full",
            Plan::Software => "software",
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
            Plan::Smoke | Plan::Display => 1,
            Plan::Quick | Plan::Software => 3,
            Plan::Default | Plan::Weak | Plan::Focus | Plan::Full => 5,
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
            Plan::Display => cells.push(cell(Scenario::Hover, Workload::Trace, native)),
            Plan::Quick => {
                for workload in [Workload::Trace, Workload::Scatter100k] {
                    for scenario in [Scenario::Hover, Scenario::PanX, Scenario::ZoomLines] {
                        cells.push(cell(scenario, workload, native));
                    }
                    if workload == Workload::Trace {
                        cells.push(cell(Scenario::DragResize, workload, native));
                    }
                }
            }
            Plan::Default => {
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
            Plan::Focus => {
                for scenario in [Scenario::ZoomLines, Scenario::ZoomPixels] {
                    cells.push(cell(scenario, Workload::Trace, native));
                }
                for factor in FACTORS {
                    if factor != native {
                        cells.push(cell(Scenario::ZoomLines, Workload::Trace, factor));
                    }
                }
                for workload in REGISTERED {
                    cells.push(cell(Scenario::ZoomLines, workload, native));
                }
                for workload in [Workload::Trace, Workload::Scatter1m] {
                    cells.push(cell(Scenario::PanX, workload, native));
                }
                for workload in [Workload::Trace, Workload::Grid16] {
                    cells.push(cell(Scenario::DragResize, workload, native));
                }
            }
            Plan::Software => {
                for factor in SOFTWARE_FACTORS {
                    for workload in SOFTWARE_WORKLOADS {
                        for scenario in SOFTWARE_SCENARIOS {
                            cells.push(cell(scenario, workload, factor));
                        }
                    }
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
            (Plan::Display, _) => DISPLAY_MS,
            (
                Plan::Quick
                | Plan::Default
                | Plan::Weak
                | Plan::Focus
                | Plan::Full
                | Plan::Software,
                _,
            ) => scenario.duration_ms(),
        }
    }

    /// The workloads' data size in this plan (1 = full).
    pub fn data_scale(self) -> f64 {
        match self {
            Plan::Smoke | Plan::Display => 0.01,
            Plan::Quick
            | Plan::Default
            | Plan::Weak
            | Plan::Focus
            | Plan::Full
            | Plan::Software => 1.0,
        }
    }

    /// The estimated wall time of `cells` on real hardware, in s: each
    /// run's duration plus 4 s of start, data and settle. "n/a" cells cost
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

/// Whether `list` admits `item`. `None` admits everything.
fn keep<T: PartialEq>(list: &Option<Vec<T>>, item: &T) -> bool {
    list.as_ref().is_none_or(|list| list.contains(item))
}

impl Cell {
    /// Whether the workload supports the scenario (else "n/a").
    pub fn applies(&self) -> bool {
        self.workload.supports(self.scenario)
    }

    /// The window's logical size on a display of factor `native`. At the
    /// native factor it is the scenario's. At another factor k (the factor
    /// cells) it is the standard window's 1600 × 1000 device px, that is
    /// 1600 / k × 1000 / k logical px, so it fits a 1080p display. A
    /// workload with a size of its own (the wheel figure) keeps it wherever
    /// the scenario does not ask for the 4k window.
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
            | Scenario::DragResize
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
        // (A) holds `drag-resize`, report-only: no budget reads it, so the
        // rule's cells are the registered ones.
        let cells = Plan::Default.cells(2.0, &Selection::default());
        assert_eq!(cells.len(), 10 + 12 + 6 + 1);
        let runs: u32 = cells.iter().map(|c| c.runs).sum();
        assert_eq!(runs, 28 * 5 + 1);
        let budgeted = cells
            .iter()
            .filter(|c| !crate::rule::Budget::of(c.scenario).is_empty())
            .count();
        assert_eq!(budgeted, 8 + 12 + 6);
        // (C): pan-x and zoom-lines on the trace at the three other factors.
        let other: Vec<f32> = cells
            .iter()
            .filter(|c| c.scenario == Scenario::PanX && c.workload == Workload::Trace)
            .map(|c| c.factor)
            .collect();
        assert_eq!(other, [2.0, 1.0, 1.25, 1.5]);
        assert!(cells.iter().all(|c| c.applies()));
        // About 47 minutes: 140 runs of about 16 s and the 10-minute soak.
        let minutes = Plan::Default.estimate_s(&cells) / 60.0;
        assert!((45.0..49.0).contains(&minutes), "{minutes}");
        // A native factor outside the four adds one more factor to (C).
        assert_eq!(
            Plan::Default.cells(1.75, &Selection::default()).len(),
            10 + 12 + 8 + 1
        );
    }

    #[test]
    fn quick_plan_has_d22_9s_cells() {
        let cells = Plan::Quick.cells(1.0, &Selection::default());
        let named: Vec<(Scenario, Workload)> =
            cells.iter().map(|c| (c.scenario, c.workload)).collect();
        assert_eq!(
            named,
            [
                (Scenario::Hover, Workload::Trace),
                (Scenario::PanX, Workload::Trace),
                (Scenario::ZoomLines, Workload::Trace),
                (Scenario::DragResize, Workload::Trace),
                (Scenario::Hover, Workload::Scatter100k),
                (Scenario::PanX, Workload::Scatter100k),
                (Scenario::ZoomLines, Workload::Scatter100k),
            ]
        );
        assert!(
            cells
                .iter()
                .all(|c| c.factor == 1.0 && c.runs == 3 && c.applies())
        );
        // About 5.4 minutes on real hardware; the software profile's two
        // cells about 1.6.
        let minutes = Plan::Quick.estimate_s(&cells) / 60.0;
        assert!((5.0..6.0).contains(&minutes), "{minutes}");
        let software = Selection {
            scenarios: Some(vec![Scenario::PanX, Scenario::ZoomLines]),
            workloads: Some(vec![Workload::Trace]),
            ..Selection::default()
        };
        let cells = Plan::Quick.cells(1.0, &software);
        assert_eq!(cells.len(), 2);
        let minutes = Plan::Quick.estimate_s(&cells) / 60.0;
        assert!((1.5..1.7).contains(&minutes), "{minutes}");
    }

    #[test]
    fn the_display_plan_is_one_short_hover() {
        let cells = Plan::Display.cells(1.25, &Selection::default());
        assert_eq!(cells.len(), 1);
        let cell = cells[0];
        assert_eq!(
            (cell.scenario, cell.workload, cell.factor, cell.runs),
            (Scenario::Hover, Workload::Trace, 1.25, 1)
        );
        assert_eq!(Plan::Display.duration_ms(Scenario::Hover), 3_000.0);
        assert_eq!(Plan::Display.data_scale(), 0.01);
        assert_eq!(Plan::parse("display"), Some(Plan::Display));
    }

    #[test]
    fn factor_cells_keep_1600_by_1000_device_px() {
        // On a display of factor 1 the factor cells at 1.25, 1.5
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
    fn focus_plan_has_d22_5s_cells() {
        let cells = Plan::Focus.cells(1.0, &Selection::default());
        let named: Vec<(Scenario, Workload, f32)> = cells
            .iter()
            .map(|c| (c.scenario, c.workload, c.factor))
            .collect();
        assert_eq!(
            named,
            [
                (Scenario::ZoomLines, Workload::Trace, 1.0),
                (Scenario::ZoomPixels, Workload::Trace, 1.0),
                (Scenario::ZoomLines, Workload::Trace, 1.25),
                (Scenario::ZoomLines, Workload::Trace, 1.5),
                (Scenario::ZoomLines, Workload::Trace, 2.0),
                (Scenario::ZoomLines, Workload::Scatter100k, 1.0),
                (Scenario::ZoomLines, Workload::Scatter1m, 1.0),
                (Scenario::ZoomLines, Workload::Grid16, 1.0),
                (Scenario::ZoomLines, Workload::LabelChurn, 1.0),
                (Scenario::PanX, Workload::Trace, 1.0),
                (Scenario::PanX, Workload::Scatter1m, 1.0),
                (Scenario::DragResize, Workload::Trace, 1.0),
                (Scenario::DragResize, Workload::Grid16, 1.0),
            ]
        );
        assert!(cells.iter().all(|c| c.runs == 5 && c.applies()));
        // The factor cells keep 1600 × 1000 device px.
        assert!(cells.iter().all(|c| {
            let w = c.window(1.0);
            (w.width * c.factor - 1600.0).abs() < 1e-3
        }));
        // About 17 minutes on real hardware.
        let minutes = Plan::Focus.estimate_s(&cells) / 60.0;
        assert!((16.0..18.0).contains(&minutes), "{minutes}");
        assert_eq!(Plan::parse("focus"), Some(Plan::Focus));
    }

    #[test]
    fn software_plan_has_d22_12s_cells() {
        // Whatever the display: factors 1 and 2, each at 1600 × 1000
        // logical px with the native factor 2 the profile runs at.
        let cells = Plan::Software.cells(1.0, &Selection::default());
        assert_eq!(cells.len(), 4 * 3 * 2);
        assert!(cells.iter().all(|c| c.applies() && c.runs == 3));
        let factors: Vec<f32> = cells.iter().map(|c| c.factor).collect();
        assert_eq!(factors[..12], [1.0; 12]);
        assert_eq!(factors[12..], [2.0; 12]);
        for cell in Plan::Software.cells(2.0, &Selection::default()) {
            assert_eq!(cell.window(2.0), Size::new(1600.0, 1000.0), "{cell}");
        }
        assert_eq!(
            (cells[0].scenario, cells[0].workload),
            (Scenario::Hover, Workload::Trace)
        );
        assert!(
            cells
                .iter()
                .any(|c| c.workload == Workload::Grid16 && c.scenario == Scenario::DragResize)
        );
        assert_eq!(Plan::parse("software"), Some(Plan::Software));
        // One run a cell, as the quick level runs it: about 6.4 minutes.
        let one = Selection {
            runs: Some(1),
            ..Selection::default()
        };
        let minutes = Plan::Software.estimate_s(&Plan::Software.cells(2.0, &one)) / 60.0;
        assert!((6.0..7.0).contains(&minutes), "{minutes}");
    }

    #[test]
    fn smoke_plan_covers_every_scenario_and_workload() {
        let cells = Plan::Smoke.cells(2.0, &Selection::default());
        assert_eq!(cells.len(), 66);
        // `live` and `soak` need recordings, which the scatters and
        // `wheels` lack. `pan-x` needs an x that pans, which `wheels` lacks.
        assert_eq!(cells.iter().filter(|c| c.applies()).count(), 59);
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
        assert_eq!(cells.len(), 22);
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
