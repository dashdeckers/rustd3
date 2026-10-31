//! [`Domains`]: the scales gestures changed, keyed by handle position and
//! scale type ([`Key`]): those the widget owns, those it published during
//! the active gesture, which handles a gesture touched, and each handle's
//! core `Zoom` and preview view.
//!
//! Core's `Zoom` keeps d3's scale extent [0, ∞], which does not bound a
//! gesture's k; each handle's zoom here is built with the extent [1e-6,
//! 1e6], so k stays finite across a long wheel burst, while the figure's
//! pixel operations still reject degenerate spans (a step they reject
//! leaves the gesture where it was).

use std::any::TypeId;
use std::collections::{HashMap, HashSet};

use rustd3_core::interact::Zoom;
use rustd3_figure::scale::Scale;
use rustd3_figure::{Held, Resolved};

use crate::plot::preview::{Affine1, View};

/// The scale extent of each handle's zoom: k of one gesture.
const SCALE_EXTENT: [f64; 2] = [1e-6, 1e6];

/// A handle as the widget knows it across rebuilds of the figure: its
/// position in the scale table and its scale's type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Key {
    index: usize,
    scale: TypeId,
}

/// Widget-owned and published domains, the touched set, and each handle's
/// zoom and preview view.
#[derive(Default)]
pub(crate) struct Domains {
    /// Handles nobody listens to: the widget owns them.
    owned: HashMap<Key, Box<dyn Scale>>,
    /// Published during the active gesture, until it ends.
    pending: HashMap<Key, Box<dyn Scale>>,
    /// Changed since the last reset.
    touched: HashSet<Key>,
    /// One core zoom per handle, persistent.
    zooms: HashMap<Key, Zoom>,
    /// Each handle's preview view.
    views: HashMap<Key, View>,
    /// Preview epochs started.
    epochs: u64,
}

impl Key {
    /// The handle at table position `index` whose scale is `scale`.
    pub(crate) fn new(index: usize, scale: &dyn Scale) -> Key {
        Key {
            index,
            scale: scale.as_any().type_id(),
        }
    }

    /// The handle at table position `index` whose scale has type `S`.
    pub(crate) fn of<S: Scale>(index: usize) -> Key {
        Key {
            index,
            scale: TypeId::of::<S>(),
        }
    }

    /// The handle's position in the scale table.
    pub(crate) fn index(self) -> usize {
        self.index
    }

    /// Whether `resolved` has a scale of this key's type at its position.
    fn names(self, resolved: &Resolved) -> bool {
        resolved
            .get_scale_at(self.index)
            .is_some_and(|scale| scale.as_any().type_id() == self.scale)
    }
}

impl Domains {
    /// The overrides a refresh resolves with: every owned and pending
    /// scale, pending winning.
    pub(crate) fn held(&self) -> Held {
        let mut held = Held::default();
        for (key, scale) in self.owned.iter().chain(&self.pending) {
            held.hold_erased(key.index, scale.boxed());
        }
        held
    }

    /// The scale `key` shows now: owned, else pending, else as `resolved`
    /// has it; `None` when `resolved` has no scale of the key's type there.
    pub(crate) fn current(&self, key: Key, resolved: &Resolved) -> Option<Box<dyn Scale>> {
        let kept = self.owned.get(&key).or_else(|| self.pending.get(&key));
        match kept {
            Some(scale) => Some(scale.boxed()),
            None => key
                .names(resolved)
                .then(|| resolved.get_scale_at(key.index))
                .flatten()
                .map(Scale::boxed),
        }
    }

    /// Stores what a gesture made of `key`: published (`listened`) scales
    /// as pending until the gesture ends, others as the widget's own.
    pub(crate) fn store(&mut self, key: Key, scale: Box<dyn Scale>, listened: bool) {
        if listened {
            self.pending.insert(key, scale);
        } else {
            self.owned.insert(key, scale);
        }
        self.touched.insert(key);
    }

    /// The zoom of `key`, made on first use.
    pub(crate) fn zoom(&mut self, key: Key) -> &mut Zoom {
        self.zooms
            .entry(key)
            .or_insert_with(|| Zoom::default().scale_extent(SCALE_EXTENT))
    }

    /// The gesture ended: published domains are the app's from now on.
    pub(crate) fn release(&mut self) {
        self.pending.clear();
    }

    /// Whether a gesture changed `key` since its last reset.
    pub(crate) fn is_touched(&self, key: Key) -> bool {
        self.touched.contains(&key)
    }

    /// Returns `key` to its fitted domain: the widget's own scale and the
    /// published one go, so the figure resolves the handle as it is built;
    /// it counts as untouched; its zoom and its preview view start afresh.
    pub(crate) fn reset(&mut self, key: Key) {
        self.owned.remove(&key);
        self.pending.remove(&key);
        self.touched.remove(&key);
        self.zooms.remove(&key);
        self.views.remove(&key);
    }

    /// Drops everything kept for handles whose position names a scale of
    /// another type in `resolved`, or none: the app reordered its handles.
    pub(crate) fn prune(&mut self, resolved: &Resolved) {
        self.owned.retain(|key, _| key.names(resolved));
        self.pending.retain(|key, _| key.names(resolved));
        self.touched.retain(|key| key.names(resolved));
        self.zooms.retain(|key, _| key.names(resolved));
        self.views.retain(|key, _| key.names(resolved));
    }

    /// The preview view of `key`, started (the identity, a new epoch)
    /// when it has none.
    pub(crate) fn view(&mut self, key: Key) -> View {
        let epochs = &mut self.epochs;
        *self.views.entry(key).or_insert_with(|| {
            *epochs += 1;
            View {
                epoch: *epochs,
                map: Affine1::IDENTITY,
            }
        })
    }

    /// The preview view of `key`, if it has one.
    pub(crate) fn peek(&self, key: Key) -> Option<View> {
        self.views.get(&key).copied()
    }

    /// Sets the map of `key`'s preview view, in its current epoch.
    pub(crate) fn move_view(&mut self, key: Key, map: Affine1) {
        let view = self.view(key);
        self.views.insert(key, View { map, ..view });
    }

    /// The scale the widget owns for `key`.
    #[cfg(test)]
    pub(crate) fn owned(&self, key: Key) -> Option<&dyn Scale> {
        self.owned.get(&key).map(|scale| &**scale)
    }

    /// The scale published for `key` during the active gesture.
    pub(crate) fn published(&self, key: Key) -> Option<&dyn Scale> {
        self.pending.get(&key).map(|scale| &**scale)
    }
}
