//! How the bars of several series share a key: [`Mode`]; the chart is
//! [`chart::bar`](super::bar()).

pub(super) mod preset;

/// How the bars of several series stand at each key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Side by side: the key's band split into one sub-band per series, in
    /// the series' order, each bar from 0 (seaborn's `barplot` with `hue`).
    #[default]
    Grouped,
    /// On top of each other in the series' order, the first from 0: d3's
    /// stack with offset none (matplotlib's bars with `bottom`).
    Stacked,
    /// Positive values stacked up from 0 and negative ones down from it, in
    /// the series' order: d3's `stackOffsetDiverging`.
    Diverging,
}
