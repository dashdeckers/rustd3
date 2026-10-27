//! Drawing a figure into a list for a target.

use kurbo::Rect;

use crate::figure::{Cache, Figure, Held};
use crate::list::{Bounds, List, Outlines};
use crate::target::Target;

impl Figure<'_> {
    /// The figure as a drawing list of `size` for `target`, with `held`'s
    /// scales: starts a draw of `cache` (the text cache's generation),
    /// resolves, and emits the figure background, then per panel its
    /// background, its marks (inside a group clipped to the plot area), its
    /// border, axes and title, then the figure title, each a part whose
    /// bounds are the union of its commands' bounds. Parts' hashes are 0.
    pub fn draw_with(
        &self,
        cache: &mut Cache,
        size: kurbo::Size,
        target: Target,
        held: &Held,
    ) -> List {
        cache.begin();
        let resolved = self.resolve(size, cache, held, target);
        let mut list = List {
            size,
            ..List::default()
        };
        self.decorate(&resolved, cache.text(), &mut list);
        let bounds = Bounds::of(&list, &mut Outlines::default());
        for part in &mut list.parts {
            part.bounds = part
                .commands
                .clone()
                .filter_map(|i| bounds.get(i))
                .reduce(|a, b| a.union(b))
                .unwrap_or(Rect::ZERO);
        }
        list
    }
}
