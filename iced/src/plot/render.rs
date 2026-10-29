//! [`Render`]: where the widget rasterizes changed cells.

/// Where the widget rasterizes changed cells.
///
/// Inline renders happen inside `RedrawRequested`, so the frame shows
/// them at once; pool renders run on the threads the widget owns
/// (`Plot::threads`), whose results arrive as `Event::Waken` while each
/// cell keeps its previous image on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Render {
    /// In order (visible tiles nearest the last gesture input, the
    /// gutters, the ring, which waits for a gesture to end; during a
    /// gesture the gutters and overlays first), inline each
    /// render whose predicted time still fits half the frame budget with
    /// those before it, the rest on the pool; results arrive as
    /// `Event::Waken`. While a wheel burst (until its end) or a brush
    /// lasts every tile renders on the pool, so the UI thread composes and
    /// renders at most the gutters and overlays that show the moving
    /// handles; inline tile renders resume when it ends. A drag's tiles
    /// (the column entering the plot area) render inline when they fit, as
    /// idle ones do: on the pool they would land a frame later.
    ///
    /// The budget is the interval between the last two `RedrawRequested`
    /// instants, clamped to 1 ms ..= 16.7 ms (16.7 ms until two exist);
    /// the prediction is a fixed overhead plus a time per pixel and per
    /// work item that each widget fits to its timed renders, seeded by
    /// [`cost::model_ms`](crate::plot::cost::model_ms)'s constants.
    #[default]
    Auto,
    /// Always on the UI thread, inside `RedrawRequested`, gestures
    /// included.
    Inline,
    /// Always on the pool (tests, the benchmark, diagnostics).
    Worker,
}
