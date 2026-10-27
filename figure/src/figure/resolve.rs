//! Resolving a figure: domains fitted to the marks' data or held by a
//! gesture, labels measured through the text cache in one batch, and the
//! grid laid out by protrusions.

use crate::figure::fits::Fits;
use crate::figure::{Cache, Figure, Held, Resolved};
use crate::layout;
use crate::panel::Panel;
use crate::scale::Scale;
use crate::target::Target;
use crate::text::Key;

impl Figure<'_> {
    /// Fits every fitted handle's domain to the data of every mark that
    /// uses it (held handles keep the held scale), measures every label
    /// through `cache`'s text cache in one batch, and lays the grid out for
    /// `size`. `target` is kept in the result.
    ///
    /// Plot areas sit inside 0.8 em of padding on every side, below the
    /// figure title's ink height and 8 px when there is a title.
    pub fn resolve(
        &self,
        size: kurbo::Size,
        cache: &mut Cache,
        held: &Held,
        target: Target,
    ) -> Resolved {
        let mut resolved = Resolved::new(self.id, self.fitted(held), target);
        let title = self
            .title
            .as_ref()
            .map(|t| Key::new(t, &self.theme.figure_title()));
        let text = cache.text();
        text.prepare(title.iter());
        let reserve = title
            .as_ref()
            .and_then(|key| text.get(key)?.ok())
            .map_or(0.0, |s| s.ascent() + s.descent() + self.theme.title_pad());
        resolved.place(self.areas(size, reserve));
        resolved
    }

    /// Every scale of the table as resolved: held, fitted to the marks'
    /// data, or as built.
    fn fitted(&self, held: &Held) -> Vec<Box<dyn Scale>> {
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
        let mut fits = Fits::new(self.id, slots);
        for mark in self.panels.iter().flat_map(Panel::marks) {
            mark.fit(&mut fits);
        }
        let margin = self.theme.margin();
        self.scales
            .iter()
            .zip(fits.into_slots())
            .enumerate()
            .map(|(i, (entry, slot))| {
                match (held.scale(i, &*entry.scale), entry.fitter.zip(slot)) {
                    (Some(scale), _) => scale.boxed(),
                    (None, Some((fitter, slot))) => fitter.finish(slot, margin),
                    (None, None) => entry.scale.boxed(),
                }
            })
            .collect()
    }

    /// The plot areas of the panels for `size`, with `reserve` px above
    /// the grid for the figure title.
    fn areas(&self, size: kurbo::Size, reserve: f64) -> Vec<kurbo::Rect> {
        let em = self.theme.em();
        let pad = 0.8 * em;
        let rows = tracks(
            self.grid.row_weights().len(),
            self.panels.iter().map(|p| p.cell().0),
        );
        let cols = tracks(
            self.grid.col_weights().len(),
            self.panels.iter().map(|p| p.cell().1),
        );
        let ys = layout::tracks(
            self.grid.row_weights(),
            &self.grid.row_gaps_px(rows, em),
            &vec![[0.0; 2]; rows],
            [pad + reserve, size.height - 2.0 * pad - reserve],
        );
        let xs = layout::tracks(
            self.grid.col_weights(),
            &self.grid.col_gaps_px(cols, em),
            &vec![[0.0; 2]; cols],
            [pad, size.width - 2.0 * pad],
        );
        self.panels
            .iter()
            .map(|p| {
                let ([y, h], [x, w]) = (ys[p.cell().0], xs[p.cell().1]);
                kurbo::Rect::new(x, y, x + w, y + h)
            })
            .collect()
    }
}

/// The number of tracks along one axis: as many as there are weights, or
/// as the panels' cells need.
fn tracks(weights: usize, cells: impl Iterator<Item = usize>) -> usize {
    cells.map(|c| c + 1).fold(weights, usize::max)
}
