//! [`Figure::facet`]: another figure taken in as panels of this one.

use crate::figure::{Figure, Id, Ids, Share};
use crate::layout::Cell;
use crate::panel::Panel;
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
    /// With [`Share::Scales`], a scale an axis, zoom or brush of `other` is
    /// on, at a table position where a facet taken in before with
    /// `Share::Scales` had such a scale of the same type, becomes that
    /// scale: one domain fitted to every facet's marks, and axes on it in a
    /// row or column label their ticks on the outer panels only (see
    /// [`Axis::keep_label`](crate::axis::Axis::keep_label)). Its palettes
    /// and other scales stay its own, so its legends list its own entries.
    /// With [`Share::None`] every scale stays the facet's own.
    ///
    /// `other`'s title becomes its first panel's title, unless that panel
    /// has one (a caller that lifts it to the grid takes it first,
    /// [`take_title`](Figure::take_title)); a title set on the panel after
    /// `facet` replaces it. Its inset legends stay in its panel; its
    /// figure-slot legends are kept, and with `Share::Scales` a slot
    /// legend on the facet table position of one an earlier facet brought
    /// to the same slot is kept once. What it noted for
    /// [`check`](Figure::check) carries over. Its theme, locale
    /// ([`get_locale`](Figure::get_locale)), grid weights and axis room give
    /// way to this figure's; it takes the rows and columns
    /// [`get_extent`](Figure::get_extent) gives from `at`.
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
            slotted: _,
        } = other;
        // Only the scales its axes, zooms and brushes are on are shared: its
        // palettes, and with them its legends, stay its own.
        let positional: Vec<usize> = panels
            .iter()
            .flat_map(Panel::positional)
            .filter_map(|h| h.of(&from))
            .collect();
        let mut map = Vec::with_capacity(scales.len());
        for (i, entry) in scales.into_iter().enumerate() {
            let kind = entry.scale.as_any().type_id();
            let shares = share == Share::Scales && positional.contains(&i);
            let kept = self
                .shared
                .get(&i)
                .copied()
                .filter(|&j| shares && self.scales[j].scale.as_any().type_id() == kind);
            let j = kept.unwrap_or_else(|| {
                self.scales.push(entry);
                self.scales.len() - 1
            });
            if shares {
                self.shared.entry(i).or_insert(j);
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
        // Facets that share scales are drawn by one plot: a slot legend of the
        // same slot on the same facet table position is one legend, kept once.
        for (slot, mut stored) in legends {
            let [h] = stored.handles();
            let place = h.of(&from);
            let seen = (slot, place.unwrap_or(usize::MAX));
            if share == Share::Scales && place.is_some() {
                if self.slotted.contains(&seen) {
                    continue;
                }
                self.slotted.push(seen);
            }
            stored.remap(&remap);
            self.legends.push((slot, stored));
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
