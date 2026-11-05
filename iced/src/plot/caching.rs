//! [`Caching`]: how the widget reuses rendered images.

/// How the widget reuses rendered images (a public diagnostic option).
///
/// Every cell of the canvas carries a hash of what its parts drew. With
/// caching on, a refresh renders the cells whose hash changed and keeps
/// the others' images. The benchmark turns it off to measure what the
/// hashes save.
///
/// While a gesture lasts, the tiles of the overscan ring that changed
/// wait until it ends. A tile whose render a newer zoom step overtook
/// shows its result as a preview until the newest one lands. Neither is a
/// kept image, so `Verify` checks neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Caching {
    /// Re-render a cell only when its hash changed.
    #[default]
    On,
    /// Re-render every cell whenever the list changes, that is, whenever
    /// any cell's hash changed. An unchanged list still renders nothing.
    Off,
    /// As `On`, then re-render every kept or translated slot after the
    /// real jobs. A slot that differs by more than 1/255 is replaced and
    /// reported as
    /// [`probe::Sample::Mismatch`](crate::plot::probe::Sample::Mismatch).
    /// This checks what the hashes and translations keep, for the
    /// benchmark and tests. The checks report no `Raster` sample and do
    /// not feed the cost model's fit, though they do slow the renders it
    /// times. It is a diagnostic, whose timings are not meaningful.
    Verify,
}
