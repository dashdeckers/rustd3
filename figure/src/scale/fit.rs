//! [`Fit`]: how a fitted handle's domain follows its data.

/// How a fitted handle's domain follows its data, typed by the scale's
/// span (`Kind::Span`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fit<Span> {
    /// Every value of every mark that uses the handle.
    All,
    /// The newest `span` of x, for live data: the window is the view, with
    /// no margin. For an x handle whose widest panel has a plot area `w`
    /// thinning columns wide (device px on screen, Phase 15's count for
    /// files and vector output), with `col = span / w` and `last` the
    /// largest x its marks admit, the domain is `[right − w·col, right]`
    /// where `right = ceil(last / col)·col`: appends move it by whole
    /// columns, so the thinning columns of one update are the last ones
    /// shifted by a whole count and a streaming line does not shimmer. The
    /// columns line up on linear scales; on others the window spans `span`
    /// in data units all the same. Recordings on different clocks share
    /// one `last`, their newest x of all.
    ///
    /// A panel narrower than the widest one using the handle gets columns
    /// that are not whole device pixels: its pixels are right, only its
    /// column edges do not repeat between updates. Horizontal axes on the
    /// handle reserve room for a tick label at either end of the plot
    /// area, so ticks sliding past its ends do not move the layout.
    ///
    /// With no admitted data, a span that is not finite and positive, or a
    /// window whose ends the scale does not admit (a log scale's left end
    /// below 0), the handle fits as `All`; so it does on a y handle, which
    /// no mark places across a plot area. A gesture's held domain wins
    /// until a reset releases it. Meaningless on a categorical handle,
    /// whose span is `()`.
    Window(Span),
    /// y fitted to the data inside its x handles' resolved domains, held
    /// still by steps of one eighth of its span; as `All` on an x handle.
    Visible,
}
