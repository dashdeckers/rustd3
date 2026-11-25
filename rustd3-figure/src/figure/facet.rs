//! [`Figure::facet`]: another figure taken in as panels of this one.

use crate::figure::{Figure, Id, Ids, Share};
use crate::layout::Cell;
use crate::scale::ScaleRef;

/// Where the handles of a figure taken in go: each position of its table
/// to a position of this figure's.
pub(crate) struct Remap<'m> {
    from: &'m Ids,
    to: Id,
    map: &'m [usize],
}

impl Remap<'_> {
    /// The figure taking the handles in.
    pub(crate) fn target(&self) -> Id {
        self.to
    }

    /// `h` as a handle of the figure taking it in; a handle of neither
    /// figure stays as it is.
    pub(crate) fn apply<S>(&self, h: ScaleRef<S>) -> ScaleRef<S> {
        match h.of(self.from).and_then(|i| self.map.get(i)) {
            Some(&index) => h.moved(self.to, index),
            None => h,
        }
    }
}

impl<'a> Figure<'a> {
    /// Takes in every panel of `other`, its grid moved so that its cell
    /// (0, 0) is cell `at` here (`at`'s span is ignored), with its scales,
    /// so that a plot built as a figure of its own becomes one facet of a
    /// grid. Its marks, axes, legends, zooms and brushes keep drawing on
    /// their scales, wherever those went in this figure's table, and its
    /// handles are this figure's ([`owns`](Figure::owns),
    /// [`handle`](Figure::handle)).
    ///
    /// With [`Share::Scales`], a scale at a table position where a figure
    /// taken in before with `Share::Scales` had a scale of the same type
    /// becomes that scale: one domain fitted to every facet's marks, and
    /// axes on it in a row or column label their ticks on the outer panels
    /// only (see [`Axis::keep_label`](crate::axis::Axis::keep_label)). With
    /// [`Share::None`] every scale stays the facet's own.
    ///
    /// `other`'s title becomes its first panel's title, unless that panel
    /// has one; its figure-slot legends are kept, a legend on a scale whose
    /// slot already holds one on that scale once; what it noted for
    /// [`check`](Figure::check) carries over. Its theme, locale, grid
    /// weights and axis room give way to this figure's.
    ///
    /// ```
    /// use rustd3_core::scale;
    /// use rustd3_figure::mark::line;
    /// use rustd3_figure::{Figure, Share, axis};
    /// use rustd3_figure::layout::Cell;
    ///
    /// // A plot that builds a figure of its own, drawn once per facet.
    /// let plot = |gain: f64| {
    ///     let mut fig = Figure::new();
    ///     let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
    ///     fig.title(format!("gain {gain}"));
    ///     fig.panel(0, 0)
    ///         .mark(line([(0.0, 0.0), (1.0, gain)], x, y))
    ///         .axis(axis::bottom(x))
    ///         .axis(axis::left(y));
    ///     fig
    /// };
    /// let mut grid = Figure::new();
    /// for (col, gain) in [1.0, 2.0, 3.0].into_iter().enumerate() {
    ///     grid.facet(Cell::new(0, col), plot(gain), Share::Scales);
    /// }
    /// ```
    pub fn facet(&mut self, at: Cell, other: Figure<'a>, share: Share) -> &mut Self {
        let from = other.ids();
        let Figure {
            id: _,
            scales,
            panels,
            outside,
            detached: _,
            foreign,
            categorical,
            grid: _,
            title,
            theme: _,
            locale: _,
            legends,
            noted,
            axis_room: _,
            taken,
            shared: _,
        } = other;
        let mut map = Vec::with_capacity(scales.len());
        for (i, entry) in scales.into_iter().enumerate() {
            let kind = entry.scale.as_any().type_id();
            let kept = match share {
                Share::Scales => self
                    .shared
                    .get(i)
                    .copied()
                    .filter(|&j| self.scales[j].scale.as_any().type_id() == kind),
                Share::None => None,
            };
            let j = kept.unwrap_or_else(|| {
                self.scales.push(entry);
                self.scales.len() - 1
            });
            if share == Share::Scales && i == self.shared.len() {
                self.shared.push(j);
            }
            map.push(j);
        }
        // Handles of the figures `other` took in go where theirs went.
        for (id, moved) in taken.iter() {
            let moved = moved.iter().filter_map(|&k| map.get(k).copied()).collect();
            self.taken.push((*id, moved));
        }
        self.taken.push((from.own, map.clone()));
        let remap = Remap {
            from: &from,
            to: self.id,
            map: &map,
        };
        let first = self.panels.len();
        for panel in panels.into_iter().map(|p| p.moved(&at, &remap)) {
            // Moved past the grid's tracks, it is reported, as `panel_at`
            // reports such a cell, and never laid out.
            if panel.cell().within() {
                self.panels.push(panel);
            } else if !self.outside.contains(panel.cell()) {
                self.outside.push(panel.cell().clone());
            }
        }
        if let Some(title) = title
            && let Some(panel) = self.panels.get_mut(first)
            && panel.heading().is_none()
        {
            panel.title(title);
        }
        let ids = self.ids();
        for (slot, mut stored) in legends {
            stored.remap(&remap);
            let [h] = stored.handles();
            let place = h.of(&ids);
            let twice = self
                .legends
                .iter()
                .any(|(s, kept)| *s == slot && kept.handles()[0].of(&ids) == place);
            if !twice {
                self.legends.push((slot, stored));
            }
        }
        self.outside
            .extend(outside.iter().map(|cell| cell.shifted(&at)));
        self.categorical
            .extend(categorical.iter().filter_map(|&i| map.get(i).copied()));
        self.foreign.extend(foreign);
        self.noted.extend(noted);
        self
    }
}
