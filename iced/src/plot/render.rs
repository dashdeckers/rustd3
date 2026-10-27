//! [`Render`]: where the widget rasterizes changed cells.

/// Where the widget rasterizes changed cells.
///
/// Inline renders happen inside `RedrawRequested`, so the frame shows
/// them at once; worker renders run on one thread the widget owns, whose
/// results arrive as `Event::Waken` while each cell keeps its previous
/// image on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Render {
    /// Inline when the predicted cost is at most half the frame budget,
    /// else on the worker thread; results arrive as `Event::Waken`.
    ///
    /// The budget is the interval between the last two `RedrawRequested`
    /// instants, clamped to 1 ms ..= 16.7 ms (16.7 ms until two exist);
    /// the prediction sums [`cost::model_ms`](crate::plot::cost::model_ms)
    /// over the changed cells, times the calibration a `Probe`'s raster
    /// times give from the 8th sample on.
    #[default]
    Auto,
    /// Always on the UI thread, inside `RedrawRequested`.
    Inline,
    /// Always on the worker thread (tests, the benchmark, diagnostics).
    Worker,
}
