//! Drawing a figure into a list for a target.

use std::cell::RefCell;

use kurbo::Rect;

use crate::cache::Cache;
use crate::figure::{Figure, Held, Resolved};
use crate::layout;
use crate::list::{self, Bounds, List, Outlines};
use crate::target::Target;

impl Figure<'_> {
    /// The figure as a drawing list of `size` logical px for vector output
    /// ([`Target::Vector`]), with a fresh [`Cache`] over the bundled fonts
    /// and nothing held: what `save_svg` writes. Repeated outputs of one
    /// figure keep a cache and call [`draw_with`](Figure::draw_with).
    #[cfg(feature = "fonts")]
    #[must_use]
    pub fn draw(&self, size: kurbo::Size) -> List {
        self.draw_with(
            &mut Cache::default(),
            size,
            Target::Vector,
            &Held::default(),
        )
    }

    /// The figure as a drawing list of `size` for `target`, with `held`'s
    /// scales: starts a draw of `cache` (the text cache's generation),
    /// resolves, and emits the figure background, then per panel its
    /// background, its marks (inside a group clipped to the plot area), its
    /// border, axes and title, then the figure title, each a part whose
    /// bounds are the union of its commands' bounds (one
    /// [`Bounds`](crate::list::Bounds) of the whole list) and whose hash is
    /// FNV-1a 64 of what it drew where it drew it
    /// ([`Part::hash`](crate::list::Part::hash)). Lines over recordings are thinned through `cache`'s M4 memo, so a
    /// second draw of an unchanged figure runs no M4 query (and shapes no
    /// text).
    pub fn draw_with(
        &self,
        cache: &mut Cache,
        size: kurbo::Size,
        target: Target,
        held: &Held,
    ) -> List {
        cache.begin();
        let resolved = self.resolve(size, cache, held, target);
        self.draw_resolved(cache, &resolved)
    }

    /// As [`draw_with`](Figure::draw_with), laid out in the plot areas
    /// `areas` a gesture holds ([`Figure::resolve_held`]): areas made for
    /// another size or panel count are ignored.
    pub fn draw_held(
        &self,
        cache: &mut Cache,
        size: kurbo::Size,
        target: Target,
        held: &Held,
        areas: &layout::Areas,
    ) -> List {
        cache.begin();
        let resolved = self.resolve_held(size, cache, held, target, areas);
        self.draw_resolved(cache, &resolved)
    }

    /// The list of `resolved`, which this figure's
    /// [`resolve`](Figure::resolve) or [`resolve_held`](Figure::resolve_held)
    /// returned for `cache` after a [`Cache::begin`]: what
    /// [`draw_with`](Figure::draw_with) emits after it resolves, for a
    /// caller that also reads the resolved figure (plot areas, cell edges,
    /// hits) and so resolves once. A `Resolved` of another figure is not
    /// drawn from: this figure is resolved afresh at its size and target,
    /// with nothing held.
    ///
    /// ```
    /// use rustd3_core::scale;
    /// use rustd3_figure::kurbo::Size;
    /// use rustd3_figure::{Figure, Held, Target, axis};
    ///
    /// let mut fig = Figure::new();
    /// let y = fig.scale(scale::linear().domain([0.0, 3.0]));
    /// fig.panel(0, 0).axis(axis::left(y));
    /// # #[cfg(feature = "fonts")] {
    /// let mut cache = rustd3_figure::cache::Cache::default();
    /// let (size, target, held) = (Size::new(640.0, 480.0), Target::Vector, Held::default());
    /// cache.begin();
    /// let resolved = fig.resolve(size, &mut cache, &held, target);
    /// let list = fig.draw_resolved(&mut cache, &resolved);
    /// assert_eq!(list, fig.draw_with(&mut cache, size, target, &held));
    /// # }
    /// ```
    pub fn draw_resolved(&self, cache: &mut Cache, resolved: &Resolved) -> List {
        if resolved.figure() != self.id {
            let own = self.resolve(resolved.size(), cache, &Held::default(), resolved.target());
            return self.draw_resolved(cache, &own);
        }
        let size = resolved.size();
        let mut list = List {
            size,
            ..List::default()
        };
        let (text, frame) = cache.start();
        let frame = RefCell::new(frame);
        self.decorate(resolved, text, &frame, &mut list);
        cache.end(frame.into_inner());
        let bounds = Bounds::of(&list, &mut Outlines::default());
        for part in &mut list.parts {
            part.bounds = part
                .commands
                .clone()
                .filter_map(|i| bounds.get(i))
                .reduce(|a, b| a.union(b))
                .unwrap_or(Rect::ZERO);
        }
        list::parts::hash(&mut list);
        list
    }
}
