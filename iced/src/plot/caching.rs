//! [`Caching`]: how the widget reuses rendered images.

/// How the widget reuses rendered images (a public diagnostic option).
///
/// Every cell of the canvas carries a hash of what its parts drew; with
/// caching on, a refresh renders the cells whose hash changed and keeps
/// the others' images. The benchmark turns it off to measure what the
/// hashes save.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Caching {
    /// Re-render a cell only when its hash changed.
    #[default]
    On,
    /// Re-render every cell whenever the list changes: whenever any
    /// cell's hash changed. An unchanged list still renders nothing.
    Off,
    /// As `On`, then re-render every kept or translated slot after the
    /// real jobs, replace it if it differs by more than 1/255, and report
    /// [`probe::Sample::Mismatch`](crate::plot::probe::Sample::Mismatch):
    /// a check of what the hashes and translations keep, for the
    /// benchmark and tests. The checks report no `Raster` sample and do
    /// not feed the cost model's fit, though they do slow the renders it
    /// times: a diagnostic, whose timings are not meaningful.
    Verify,
}
