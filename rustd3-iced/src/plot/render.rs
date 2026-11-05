//! [`Render`]: where the widget rasterizes changed cells.

/// Where the widget rasterizes changed cells.
///
/// Inline renders happen inside `RedrawRequested`, so the frame shows
/// them at once. Pool renders run on the threads the widget owns
/// (`Plot::threads`). Their results arrive as `Event::Waken`, and until
/// then each cell keeps its previous image on screen.
///
/// ```no_run
/// # mod rustd3 { pub use rustd3_figure as figure; }
/// use rustd3::figure::Figure;
/// use rustd3_iced::Plot;
/// use rustd3_iced::plot::{Caching, Render};
///
/// fn view(fig: Figure<'_>) -> Plot<'_, ()> {
///     // Every render on the pool, every kept image checked: a diagnostic.
///     Plot::new(fig).render(Render::Worker).caching(Caching::Verify)
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Render {
    /// Inline when a render's predicted time, added to those before it,
    /// still fits half the frame budget, and on the pool otherwise. Pool
    /// results arrive as `Event::Waken`. Renders go in order: visible
    /// tiles nearest the last gesture input, then the gutters, then the
    /// ring, which waits for a gesture to end. During a gesture the
    /// gutters and overlays go first.
    ///
    /// While a wheel burst (until its end) or a brush lasts, every tile
    /// renders on the pool. The UI thread then composes and renders at
    /// most the gutters and overlays that show the moving handles, and
    /// inline tile renders resume when the gesture ends. A drag's tiles
    /// (the column entering the plot area) render inline when they fit, as
    /// idle ones do, since on the pool they would land a frame later.
    ///
    /// The budget is the interval between the last two `RedrawRequested`
    /// instants, clamped to 1 ms ..= 16.7 ms (16.7 ms until two exist).
    /// The prediction is a fixed overhead plus a time per pixel and per
    /// work item. Each widget fits it to its own timed renders, with the
    /// tiles fitted apart from the gutters and overlays. Two constants
    /// seed it, fitted to a 1920 × 1520 device px figure of 5,600 dots.
    #[default]
    Auto,
    /// Always on the UI thread, inside `RedrawRequested`, gestures
    /// included.
    Inline,
    /// Always on the pool (tests, the benchmark, diagnostics).
    Worker,
}
