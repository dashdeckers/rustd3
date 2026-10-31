//! [`Reference`]: the CPU reference, a few headless,
//! single-threaded timings of fixed renders that put a submitting machine
//! on a scale with ours, taken at every level after the display
//! measurement, into `results.json` and the report: one `trace` tile
//! mid-zoom, the `scatter100k` plot area with its markers filled and
//! splatted, and the software composition of a `trace` frame mid-zoom,
//! which needs iced and is
//! not measured in a build without the widget. Each is a median of a few
//! repetitions, at factor 1, 1600 × 1000; a panic in one is caught and
//! recorded as not measured.

use std::panic::{self, AssertUnwindSafe};
use std::time::Instant;

use rustd3_figure::cache::Cache;
use rustd3_figure::kurbo::Rect;
use rustd3_figure::list::{Bounds, List, Outlines};
use rustd3_figure::raster::{self, Settings, Simd};
use rustd3_figure::{Figure, Held, Target};
use serde::{Deserialize, Serialize};

use crate::stats::percentile;
use crate::workload::{WINDOW, Workload};

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
        let (list, tile) = zoomed_tile(&data.figure(false), 1.0).ok()?;
        Some(median_render_ms(&list, tile, 1.0, false, n))
    });
    let scatter = caught(|| {
        let data = Workload::Scatter100k.data(1.0);
        let (list, area) = first_area(&data.figure(false), 1.0)?;
        Some([false, true].map(|splat| median_render_ms(&list, area, 1.0, splat, n)))
    });
    let composition = composition();
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

/// `fig`'s list at `k` with the top plot area's layer drawn as the pool
/// draws it mid-zoom (the widget's overscan of two tiles, the x domain a
/// zoom of ×1.6 about the area's centre gives, 1600 × 1000), and the
/// 256-device-px tile in its second column across the area's middle,
/// list px.
pub fn zoomed_tile(fig: &Figure<'_>, k: f64) -> Result<(List, Rect), &'static str> {
    let target = Target::Screen { scale: k };
    let size = WINDOW;
    let mut cache = Cache::default();
    let fresh = fig.resolve(size, &mut cache, &Held::default(), target);
    let &area = fresh.get_plot_areas().first().ok_or("no plot area")?;
    let range = [area.x0, area.x1];
    let zoomed = fresh
        .get_scale_at(0)
        .and_then(|x| x.zoom_px(range, area.center().x, 1.6))
        .ok_or("the time handle does not zoom")?;
    let mut held = Held::default();
    held.hold_erased(0, zoomed);
    held.overscan(520.0 / k);
    cache.begin();
    let resolved = fig.resolve(size, &mut cache, &held, target);
    let list = fig.draw_resolved(&mut cache, &resolved);
    let side = 256.0 / k;
    let middle = area.center().y;
    let tile = Rect::new(
        area.x0 + side,
        middle - side / 2.0,
        area.x0 + 2.0 * side,
        middle + side / 2.0,
    );
    Ok((list, tile))
}

/// `fig`'s list at `k`, 1600 × 1000, and its first plot area, list px.
pub fn first_area(fig: &Figure<'_>, k: f64) -> Option<(List, Rect)> {
    let target = Target::Screen { scale: k };
    let size = WINDOW;
    let mut cache = Cache::default();
    let resolved = fig.resolve(size, &mut cache, &Held::default(), target);
    let &area = resolved.get_plot_areas().first()?;
    Some((fig.draw_resolved(&mut cache, &resolved), area))
}

/// The median of `n` single-threaded renders of `rect` of `list` at `k`,
/// markers splatted when `splat`, after one to warm the renderer, ms.
pub fn median_render_ms(list: &List, rect: Rect, k: f64, splat: bool, n: usize) -> f64 {
    let bounds = Bounds::of(list, &mut Outlines::default());
    let mut renderer = raster::Renderer::new(Settings::default().simd(Simd::Detect));
    renderer.splat(splat);
    let _ = renderer.render(list, &bounds, rect, k);
    let times: Vec<f64> = (0..n)
        .map(|_| {
            let begin = Instant::now();
            let _ = renderer.render(list, &bounds, rect, k);
            begin.elapsed().as_secs_f64() * 1e3
        })
        .collect();
    percentile(&times, 50.0).unwrap_or(0.0)
}

/// The software composition's median, ms (`None` where it panicked).
#[cfg(feature = "widget")]
fn composition() -> Option<f64> {
    caught(crate::also::composition_ms)
}

/// The software composition needs iced: not measured without the widget.
#[cfg(not(feature = "widget"))]
fn composition() -> Option<f64> {
    None
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
