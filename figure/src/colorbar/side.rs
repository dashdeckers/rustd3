//! [`Side`]: a colorbar's long axis as an axis on one side of its bar, so
//! its tick labels and title are measured, placed and drawn as an axis's.

use rustd3_core::axis::Orient;
use rustd3_core::color::Rgb;

use crate::axis;
use crate::colorbar::Draw;
use crate::figure::Resolved;
use crate::layout::collide;
use crate::scale::ScaleRef;
use crate::text::Text;

/// A colorbar's long axis on side `orient` of its bar, its labels those of
/// a binned scale's edges when `binned`.
pub(crate) struct Side<'g, 'a> {
    pub(crate) bar: &'g (dyn Draw<'a> + 'a),
    pub(crate) orient: Orient,
    pub(crate) binned: bool,
}

impl<'g, 'a: 'g> axis::Draw<'g> for Side<'g, 'a> {
    fn orient(&self) -> Orient {
        self.orient
    }

    fn handle(&self) -> usize {
        self.bar.handle()
    }

    fn scale(&self) -> ScaleRef<()> {
        self.bar.scale()
    }

    /// Untinted.
    fn tint_handle(&self) -> Option<ScaleRef<()>> {
        None
    }

    /// Untinted: the theme's ink.
    fn tint(&self, _resolved: &Resolved) -> Option<Rgb> {
        None
    }

    /// Always labelled.
    fn labels(&self) -> Option<bool> {
        Some(true)
    }

    fn count(&self) -> Option<usize> {
        self.bar.count()
    }

    /// On its bar.
    fn outward(&self) -> Option<f64> {
        None
    }

    /// Unrotated.
    fn rotation(&self) -> f64 {
        0.0
    }

    fn texts(&self, resolved: &Resolved, count: usize) -> Vec<String> {
        self.bar.texts(resolved, count, self.binned)
    }

    fn formats(&self) -> bool {
        self.bar.formats()
    }

    /// Each tick at its share of the bar's interior, `length` px long
    /// ([`along`]).
    fn positions(&self, resolved: &Resolved, length: f64, count: usize) -> Vec<Option<f64>> {
        self.bar
            .shares(resolved, count)
            .into_iter()
            .map(|s| s.map(|s| along(self.orient, length, s)))
            .collect()
    }

    /// None: a colorbar draws no minor ticks.
    fn minors(&self, _resolved: &Resolved, _length: f64, _count: usize) -> Vec<f64> {
        Vec::new()
    }

    /// The ticks' places in their set: colorbar labels are not thinned.
    fn indices(&self, resolved: &Resolved, count: usize) -> Vec<i64> {
        (0..self.bar.shares(resolved, count).len() as i64).collect()
    }

    /// None: no colorbar ticks are instants.
    fn restarts(&self, _resolved: &Resolved, _count: usize) -> Vec<collide::Restart> {
        Vec::new()
    }

    /// None: colorbars label their ticks in full.
    fn notation(&self, _resolved: &Resolved, _count: usize) -> Option<String> {
        None
    }

    fn title(&self) -> Option<&Text> {
        self.bar.title()
    }
}

/// Where a tick at `share` of a bar's interior `length` px long from its
/// low end lies, px from the interior's top or left edge, its ticks on side
/// `orient`: up a vertical bar from its bottom, across a horizontal one
/// from its left.
pub(crate) fn along(orient: Orient, length: f64, share: f64) -> f64 {
    if axis::across(orient) {
        length * share
    } else {
        length * (1.0 - share)
    }
}
