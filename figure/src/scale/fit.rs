//! [`Fit`]: how a fitted handle's domain follows its data.

/// How a fitted handle's domain follows its data, typed by the scale's
/// span (`Kind::Span`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fit<Span> {
    /// Every value of every mark that uses the handle.
    All,
    /// The last `span` of x (live data). Not implemented yet: resolves
    /// exactly as `All`. Meaningless on a categorical handle, whose span is
    /// `()`.
    Window(Span),
    /// y fitted to the data inside the visible x range. Not implemented
    /// yet: resolves exactly as `All`.
    Visible,
}
