//! [`Reference`]: the CPU reference (D22.9 (6)), a few headless,
//! single-threaded timings of fixed renders that put a submitting machine
//! on a scale with ours, taken at every level after the display
//! measurement, into `results.json` and the report: one `trace` tile
//! mid-zoom (Phase 31g's), the `scatter100k` plot area with its markers
//! filled and splatted (D31.2's), and the software composition of a
//! `trace` frame mid-zoom (Phase 31g's harness). Each is a median of a few
//! repetitions, at factor 1, 1600 × 1000; a panic in one is caught and
//! recorded as not measured.

use std::panic::{self, AssertUnwindSafe};

use serde::{Deserialize, Serialize};

use crate::also;
use crate::workload::Workload;

/// One timing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reference {
    /// Its key: `trace-tile`, `scatter-filled`, `scatter-splatted` or
    /// `composition`.
    pub name: String,
    /// What it times.
    pub what: String,
    /// The median, ms; `None` when it could not be measured.
    pub ms: Option<f64>,
    /// The repetitions the median is of.
    pub repetitions: u32,
}

/// Repetitions of the renders.
const RENDERS: u32 = 7;

/// Repetitions of the composition (the harness's warm ones).
const COMPOSITIONS: u32 = 8;

/// Takes the timings.
pub fn measure() -> Vec<Reference> {
    let n = RENDERS as usize;
    let tile = caught(|| {
        let data = Workload::Trace.data(1.0);
        let (list, tile) = also::zoomed_tile(&data.figure(false), 1.0).ok()?;
        Some(also::median_render_ms(&list, tile, 1.0, false, n))
    });
    let scatter = caught(|| {
        let data = Workload::Scatter100k.data(1.0);
        let (list, area) = also::first_area(&data.figure(false), 1.0)?;
        Some([false, true].map(|splat| also::median_render_ms(&list, area, 1.0, splat, n)))
    });
    let composition = caught(also::composition_ms);
    let entry = |name: &str, what: &str, ms: Option<f64>, repetitions: u32| Reference {
        name: name.to_owned(),
        what: what.to_owned(),
        ms,
        repetitions,
    };
    vec![
        entry(
            "trace-tile",
            "one 256 px tile of trace mid-zoom (3 × 1M samples), rendered",
            tile,
            RENDERS,
        ),
        entry(
            "scatter-filled",
            "the scatter100k plot area (a 1600 × 1000 window), markers filled",
            scatter.map(|[filled, _]| filled),
            RENDERS,
        ),
        entry(
            "scatter-splatted",
            "the same, markers splatted",
            scatter.map(|[_, splatted]| splatted),
            RENDERS,
        ),
        entry(
            "composition",
            "a trace frame mid-zoom composed by iced's software renderer",
            composition,
            COMPOSITIONS,
        ),
    ]
}

/// `measure`'s value, `None` where it panicked.
fn caught<T>(measure: impl FnOnce() -> Option<T>) -> Option<T> {
    panic::catch_unwind(AssertUnwindSafe(measure))
        .ok()
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::caught;

    #[test]
    fn a_timing_that_panics_is_not_measured() {
        let none: Option<f64> = caught(|| panic!("a broken renderer"));
        assert_eq!(none, None);
        assert_eq!(caught(|| Some(4.2)), Some(4.2));
    }
}
