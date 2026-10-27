//! [`Render`]: where the widget rasterizes changed cells.

/// Where the widget rasterizes changed cells.
///
/// Inline renders happen inside `RedrawRequested`, so the frame shows
/// them at once; worker renders run on one thread the widget owns, whose
/// results arrive as `Event::Waken` while each cell keeps its previous
/// image on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Render {
    /// Always on the UI thread, inside `RedrawRequested`.
    #[default]
    Inline,
    /// Always on the worker thread (tests, the benchmark, diagnostics).
    Worker,
}
