//! [`Fits`]: the data extents and sticky values marks report per fitted
//! handle, kept in each scale's own domain type.

use std::any::Any;

use crate::figure;
use crate::scale::{Kind, Scale, ScaleRef};
use crate::text::Key;
use crate::theme::Theme;

/// What marks report while a figure resolves: per fitted handle, the
/// extent of the values the scale admits and the sticky values a margin
/// stops at, in the scale's own domain type (no `Timestamp` passes through
/// `f64`). Handles that are not fitted, or that a gesture holds, ignore
/// what is reported. The figure's own marks also report the labels they
/// draw, so that they are shaped in the figure's one batch of text.
pub struct Fits {
    figure: figure::Id,
    /// One per table entry: a `Slot<S>` for the fitted ones.
    slots: Vec<Option<Box<dyn Any>>>,
    /// One per table entry: the scale as held, or as built.
    scales: Vec<Box<dyn Scale>>,
    /// The figure's theme, which label styles come from.
    theme: Theme,
    /// The labels marks draw.
    labels: Vec<Key>,
}

/// One fitted handle's scale as built, its extent so far and its sticky
/// values.
struct Slot<S: Kind> {
    scale: S,
    extent: Option<[S::Domain; 2]>,
    sticky: Vec<S::Domain>,
}

/// How a fitted entry of the table is fitted: functions instantiated for
/// its type when it is added, so the table stays erased.
#[derive(Clone, Copy)]
pub(super) struct Fitter {
    /// A slot for the entry's scale.
    start: fn(&dyn Scale) -> Box<dyn Any>,
    /// The scale of a filled slot, fitted with a margin.
    finish: fn(Box<dyn Any>, f64) -> Box<dyn Scale>,
}

impl Fitter {
    /// The fitter of scales of type `S`.
    pub(super) fn of<S: Kind>() -> Fitter {
        Fitter {
            start: start::<S>,
            finish: finish::<S>,
        }
    }

    /// An empty slot for `scale`.
    pub(super) fn start(self, scale: &dyn Scale) -> Box<dyn Any> {
        (self.start)(scale)
    }

    /// The scale of `slot`, fitted with `margin`.
    pub(super) fn finish(self, slot: Box<dyn Any>, margin: f64) -> Box<dyn Scale> {
        (self.finish)(slot, margin)
    }
}

impl Fits {
    /// Reports for figure `figure` drawn with `theme`: one slot and one
    /// scale (as held, or as built) per table entry.
    pub(super) fn new(
        figure: figure::Id,
        slots: Vec<Option<Box<dyn Any>>>,
        scales: Vec<Box<dyn Scale>>,
        theme: Theme,
    ) -> Fits {
        Fits {
            figure,
            slots,
            scales,
            theme,
            labels: Vec::new(),
        }
    }

    /// Widens `h`'s extent by `v` when its scale admits it (finite and
    /// valid under the transform; log: > 0).
    pub fn add<S: Kind>(&mut self, h: ScaleRef<S>, v: &S::Domain) {
        if let Some(slot) = self.slot(h)
            && slot.scale.admits(v)
        {
            S::merge(&mut slot.extent, v);
        }
    }

    /// A value `h`'s margin stops at, as matplotlib's sticky edges: bars
    /// and areas stick at their base. It does not widen the extent, so a
    /// mark adds its base too.
    pub fn stick<S: Kind>(&mut self, h: ScaleRef<S>, v: S::Domain) {
        if let Some(slot) = self.slot(h) {
            slot.sticky.push(v);
        }
    }

    /// `h`'s scale before fitting: as a gesture holds it, or as built (a
    /// mark's bridge to its x scale's index, its y scale's admission test).
    pub(crate) fn scale<S: Kind>(&self, h: ScaleRef<S>) -> &S {
        self.scales[h.of(self.figure)]
            .as_any()
            .downcast_ref()
            .expect("invariant: a handle's scale has the handle's type")
    }

    /// The figure's theme.
    pub(crate) fn theme(&self) -> &Theme {
        &self.theme
    }

    /// A label a mark draws, shaped with the figure's other labels.
    pub(crate) fn label(&mut self, key: Key) {
        self.labels.push(key);
    }

    /// The slots, one per table entry, and the labels marks draw.
    pub(super) fn into_parts(self) -> (Vec<Option<Box<dyn Any>>>, Vec<Key>) {
        (self.slots, self.labels)
    }

    /// `h`'s slot, if it is fitted and not held.
    fn slot<S: Kind>(&mut self, h: ScaleRef<S>) -> Option<&mut Slot<S>> {
        let index = h.of(self.figure);
        self.slots.get_mut(index)?.as_mut()?.downcast_mut()
    }
}

/// An empty slot for `scale`, which is an `S`.
fn start<S: Kind>(scale: &dyn Scale) -> Box<dyn Any> {
    let scale = scale
        .as_any()
        .downcast_ref::<S>()
        .expect("invariant: an entry's fitter is made for its scale's type");
    Box::new(Slot {
        scale: scale.clone(),
        extent: None,
        sticky: Vec::new(),
    })
}

/// The scale of `slot`, a `Slot<S>`, fitted to its extent and sticky
/// values with `margin`.
fn finish<S: Kind>(slot: Box<dyn Any>, margin: f64) -> Box<dyn Scale> {
    let slot = slot
        .downcast::<Slot<S>>()
        .expect("invariant: an entry's slot is made by its fitter");
    Box::new(
        slot.scale
            .fitted(slot.extent.as_ref(), &slot.sticky, margin),
    )
}
