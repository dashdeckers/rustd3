//! [`Held`]: the scales a gesture holds.

use std::any::TypeId;
use std::collections::HashMap;

use crate::figure::Figure;
use crate::scale::{Kind, Scale, ScaleRef};

/// The scales a gesture holds, keyed by handle index and scale type, which
/// survive a figure's rebuild: resolving uses a held scale as it is and
/// skips fitting it, so new data does not pull the view away. An entry
/// whose type differs from the figure's scale at that index is ignored.
#[derive(Default)]
pub struct Held {
    scales: HashMap<usize, (TypeId, Box<dyn Scale>)>,
}

impl Held {
    /// Holds `scale` for handle `h`.
    pub fn hold<S: Kind>(&mut self, h: ScaleRef<S>, scale: S) {
        self.scales
            .insert(h.index(), (TypeId::of::<S>(), Box::new(scale)));
    }

    /// Lets go of what `h` holds; fitting applies again.
    pub fn release<S: Kind>(&mut self, h: ScaleRef<S>) {
        if self.get(h).is_some() {
            self.scales.remove(&h.index());
        }
    }

    /// The scale `h` holds; `None` when it holds none or one of another
    /// type.
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

    /// The held scale for entry `index` of a table, if it has `scale`'s
    /// type.
    pub(super) fn scale(&self, index: usize, scale: &dyn Scale) -> Option<&dyn Scale> {
        let (kind, held) = self.scales.get(&index)?;
        (*kind == scale.as_any().type_id()).then_some(&**held)
    }
}
