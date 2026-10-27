//! Resolving a figure: domains fitted to the marks' data or held by a
//! gesture, labels measured through the text cache, and the grid laid out
//! in passes.

use crate::figure::fits::Fits;
use crate::figure::{Cache, Figure, Held, Resolved};
use crate::layout::{self, Layout};
use crate::panel::Panel;
use crate::scale::Scale;
use crate::target::Target;
use crate::text::Key;

impl Figure<'_> {
    /// Fits every fitted handle's domain to the data of every mark that
    /// uses it (held handles keep the held scale), and lays the grid out
    /// for `size`, measuring every label through `cache`'s text cache, one
    /// batch per layout pass. `target` is kept in the result.
    ///
    /// Each pass takes every axis's tick count from its length in the last
    /// placement (one tick per 80 px across, per 35 px up, at least 2;
    /// `.ticks(n)` fixes it) and places the plot areas again, until the
    /// counts settle (at most four passes): each panel reaches past its
    /// plot area by its axes (tick, tick pad, the labels' extent and, with
    /// a title, the label pad and the title's ink height, at the axis's
    /// offset), by tick labels that overhang its ends and by its title
    /// (title pad and ink height, above everything on top); each column
    /// takes the largest reach of its panels per side and each row
    /// likewise, and the rest is split by weight inside 0.8 em of padding,
    /// below the figure title's ink height and 8 px.
    pub fn resolve(
        &self,
        size: kurbo::Size,
        cache: &mut Cache,
        held: &Held,
        target: Target,
    ) -> Resolved {
        self.resolve_in(size, cache, held, target, None)
    }

    /// As [`resolve`](Figure::resolve), keeping the plot areas `areas`
    /// that a gesture took at its start ([`Resolved::areas`]), so the plot
    /// does not shift under the pointer: tick counts come from their
    /// lengths in one pass, labels and collisions are recomputed, and
    /// labels may reach past the old reaches. Areas made for another size
    /// or panel count are ignored: the layout is fresh, and
    /// [`Resolved::held`] is false.
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
    /// let mut cache = rustd3_figure::Cache::default();
    /// let size = Size::new(640.0, 480.0);
    /// let target = Target::Screen { scale: 1.0 };
    /// let start = fig.resolve(size, &mut cache, &Held::default(), target);
    /// let mut held = Held::default();
    /// held.hold(y, scale::linear().domain([0.0, 30_000.0]));
    /// let areas = start.areas();
    /// let during = fig.resolve_held(size, &mut cache, &held, target, &areas);
    /// assert!(during.held());
    /// assert_eq!(during.plot_areas(), start.plot_areas());
    /// # }
    /// ```
    pub fn resolve_held(
        &self,
        size: kurbo::Size,
        cache: &mut Cache,
        held: &Held,
        target: Target,
        areas: &layout::Areas,
    ) -> Resolved {
        self.resolve_in(size, cache, held, target, Some(areas))
    }

    /// Resolves for `size`, in `areas` when they fit it.
    fn resolve_in(
        &self,
        size: kurbo::Size,
        cache: &mut Cache,
        held: &Held,
        target: Target,
        areas: Option<&layout::Areas>,
    ) -> Resolved {
        let (scales, marked) = self.fitted(held);
        let mut resolved = Resolved::new(self.id, scales, target, size);
        let rects = areas.and_then(|a| a.fit(size, self.panels.len()));
        let layout = Layout::new(self, &resolved, cache.text(), &marked, size, rects);
        let title = self
            .title
            .as_ref()
            .map(|t| Key::new(t, &self.theme.figure_title()));
        resolved.place(layout, title, rects.is_some());
        resolved
    }

    /// Every scale of the table as resolved (held, fitted to the marks'
    /// data, or as built), and the labels the marks draw.
    fn fitted(&self, held: &Held) -> (Vec<Box<dyn Scale>>, Vec<Key>) {
        let slots = self
            .scales
            .iter()
            .enumerate()
            .map(
                |(i, entry)| match (held.scale(i, &*entry.scale), entry.fitter) {
                    (None, Some(fitter)) => Some(fitter.start(&*entry.scale)),
                    (Some(_), _) | (None, None) => None,
                },
            )
            .collect();
        let built = self
            .scales
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                held.scale(i, &*entry.scale)
                    .unwrap_or(&*entry.scale)
                    .boxed()
            })
            .collect();
        let mut fits = Fits::new(self.id, slots, built, self.theme.clone());
        for mark in self.panels.iter().flat_map(Panel::marks) {
            mark.fit(&mut fits);
        }
        let margin = self.theme.margin();
        let (slots, labels) = fits.into_parts();
        let scales = self
            .scales
            .iter()
            .zip(slots)
            .enumerate()
            .map(|(i, (entry, slot))| {
                match (held.scale(i, &*entry.scale), entry.fitter.zip(slot)) {
                    (Some(scale), _) => scale.boxed(),
                    (None, Some((fitter, slot))) => fitter.finish(slot, margin),
                    (None, None) => entry.scale.boxed(),
                }
            })
            .collect();
        (scales, labels)
    }
}
