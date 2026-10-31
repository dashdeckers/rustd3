//! [`Held`]: the scales a gesture holds, and how a widget's tiles thin
//! and reach past the view.

use std::any::TypeId;
use std::collections::HashMap;

use crate::figure::Figure;
use crate::scale::{Kind, Scale, ScaleRef};
use crate::thin;

/// The scales a gesture holds, keyed by handle index and scale type, which
/// survive a figure's rebuild: resolving uses a held scale as it is and
/// skips fitting it, so new data does not pull the view away. An entry
/// whose type differs from the figure's scale at that index is ignored.
///
/// A widget that translates rendered tiles also sets the column grids its
/// lines thin on ([`grids`](Held::grids)) and how far plot areas reach past
/// the view ([`overscan`](Held::overscan)).
#[derive(Default)]
pub struct Held {
    scales: HashMap<usize, (TypeId, Box<dyn Scale>)>,
    grids: HashMap<usize, thin::Grid>,
    overscan: f64,
}

impl std::fmt::Debug for Held {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Held").finish_non_exhaustive()
    }
}

impl Held {
    /// Holds `scale` for handle `h`.
    pub fn hold<S: Kind>(&mut self, h: ScaleRef<S>, scale: S) {
        self.scales
            .insert(h.index(), (TypeId::of::<S>(), Box::new(scale)));
    }

    /// Holds `scale` for the handle at table position `index`, keyed by
    /// the scale's own type: what a widget holds for handles it knows only
    /// by position. Resolving uses it where the figure's scale at `index`
    /// has that type, as [`hold`](Held::hold) does.
    pub fn hold_erased(&mut self, index: usize, scale: Box<dyn Scale>) {
        let kind = scale.as_any().type_id();
        self.scales.insert(index, (kind, scale));
    }

    /// Lets go of what `h` holds; fitting applies again.
    pub fn release<S: Kind>(&mut self, h: ScaleRef<S>) {
        if self.get(h).is_some() {
            self.scales.remove(&h.index());
        }
    }

    /// The scale `h` holds; `None` when it holds none or one of another
    /// type.
    #[must_use]
    pub fn get<S: Kind>(&self, h: ScaleRef<S>) -> Option<&S> {
        let (kind, scale) = self.scales.get(&h.index())?;
        if *kind == TypeId::of::<S>() {
            scale.as_any().downcast_ref()
        } else {
            None
        }
    }

    /// Drops the entries whose index or type does not match a scale of
    /// `fig`.
    pub fn prune(&mut self, fig: &Figure<'_>) {
        self.scales.retain(|&index, (kind, _)| {
            fig.scales
                .get(index)
                .is_some_and(|entry| entry.scale.as_any().type_id() == *kind)
        });
    }

    /// Column grids to thin with, per handle index (additive): a line whose
    /// x handle has one thins on its columns instead of columns from its
    /// plot area's left edge at the target's density, and a dashed one
    /// sets its dash offset so that the pattern's phase is 0 at the first
    /// sample of the column containing the grid's origin (a millionth of a
    /// px past it, so that rounding cannot start the dash on the segment
    /// before), measured along the emitted polyline. Replaces the grids set
    /// before.
    pub fn grids(&mut self, grids: Vec<(usize, thin::Grid)>) -> &mut Held {
        self.grids = grids.into_iter().collect();
        self
    }

    /// The column grid of the handle at table position `handle`, if one is
    /// set.
    #[must_use]
    pub fn grids_for(&self, handle: usize) -> Option<thin::Grid> {
        self.grids.get(&handle).copied()
    }

    /// Extends each plot area's data range and clip by `px` list px, for
    /// plot-area tile jobs (additive): its background and grid lines cover
    /// the grown area, its marks are clipped to it and draw the data there
    /// (lines thin the columns of the grown view and keep one sample
    /// beyond it; without a grid of their own they thin and anchor dashes
    /// on the grid from the plot area's left edge at the target's
    /// density), so tiles around the view show what a pan brings in. The
    /// rest of the figure draws as it would without; a width that is not
    /// finite and positive extends nothing.
    pub fn overscan(&mut self, px: f64) -> &mut Held {
        self.overscan = if px.is_finite() && px > 0.0 { px } else { 0.0 };
        self
    }

    /// Every column grid, by handle index.
    pub(super) fn grid_table(&self) -> &HashMap<usize, thin::Grid> {
        &self.grids
    }

    /// How far plot areas reach past the view, list px.
    pub(super) fn overscanned(&self) -> f64 {
        self.overscan
    }

    /// The held scale for entry `index` of a table, if it has `scale`'s
    /// type.
    pub(super) fn scale(&self, index: usize, scale: &dyn Scale) -> Option<&dyn Scale> {
        let (kind, held) = self.scales.get(&index)?;
        (*kind == scale.as_any().type_id()).then_some(&**held)
    }
}
