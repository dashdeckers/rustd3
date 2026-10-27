//! Resolving a figure: domains fitted to the marks' data or held by a
//! gesture, labels measured through the text cache in one batch, and the
//! grid laid out by protrusions.

use crate::axis::{self, Labels};
use crate::figure::fits::Fits;
use crate::figure::{Cache, Figure, Held, Resolved};
use crate::layout::{self, Reach};
use crate::panel::Panel;
use crate::scale::Scale;
use crate::target::Target;
use crate::text::{self, Key, Shaped};

impl Figure<'_> {
    /// Fits every fitted handle's domain to the data of every mark that
    /// uses it (held handles keep the held scale), measures every label
    /// through `cache`'s text cache in one batch, and lays the grid out for
    /// `size`. `target` is kept in the result.
    ///
    /// Layout is one protrusion pass: each panel reaches past its plot area
    /// by its axes (tick, tick pad, the labels' extent and, with a title,
    /// the label pad and the title's ink height, at the axis's offset), by
    /// tick labels that overhang its ends (measured on the plot areas laid
    /// out without them) and by its title (title pad and ink height, above
    /// everything on top); each column takes the largest reach of its
    /// panels per side and each row likewise, and the rest is split by
    /// weight inside 0.8 em of padding, below the figure title's ink height
    /// and 8 px.
    pub fn resolve(
        &self,
        size: kurbo::Size,
        cache: &mut Cache,
        held: &Held,
        target: Target,
    ) -> Resolved {
        let mut resolved = Resolved::new(self.id, self.fitted(held), target);
        let theme = &self.theme;
        let labels: Vec<Vec<Labels>> = self
            .panels
            .iter()
            .map(|p| {
                p.axes()
                    .iter()
                    .map(|a| Labels::new(&**a, &resolved, theme))
                    .collect()
            })
            .collect();
        let titles: Vec<Option<Key>> = self
            .panels
            .iter()
            .map(|p| p.heading().map(|t| Key::new(t, &theme.title())))
            .collect();
        let title = self
            .title
            .as_ref()
            .map(|t| Key::new(t, &theme.figure_title()));
        let text = cache.text();
        text.prepare(
            labels
                .iter()
                .flatten()
                .flat_map(Labels::keys)
                .chain(titles.iter().flatten())
                .chain(&title),
        );
        let text = &*text;
        let reserve = title
            .as_ref()
            .and_then(|key| get(text, key))
            .map_or(0.0, |s| ink(s) + theme.title_pad());
        let titled = |reach: &Reach, title: &Option<Key>| {
            let mut reach = *reach;
            if let Some(s) = title.as_ref().and_then(|key| get(text, key)) {
                reach.top += theme.title_pad() + ink(s);
            }
            reach
        };
        let axes: Vec<Reach> = labels
            .iter()
            .map(|axes| {
                let mut reach = Reach::default();
                for a in axes {
                    reach.widen(a.orient(), a.reach(text, theme));
                }
                reach
            })
            .collect();
        let bare: Vec<Reach> = axes
            .iter()
            .zip(&titles)
            .map(|(r, t)| titled(r, t))
            .collect();
        let bare = self.areas(size, reserve, &bare);
        let reaches: Vec<Reach> = self
            .panels
            .iter()
            .zip(&labels)
            .zip(axes)
            .zip(&bare)
            .zip(&titles)
            .map(|((((panel, labels), mut reach), area), title)| {
                for (a, l) in panel.axes().iter().zip(labels) {
                    let length = if axis::across(l.orient()) {
                        area.width()
                    } else {
                        area.height()
                    };
                    let positions = a.positions(&resolved, length);
                    reach.overhang(l.orient(), l.overhang(&positions, length, text));
                }
                titled(&reach, title)
            })
            .collect();
        resolved.place(self.areas(size, reserve, &reaches));
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
    /// the grid for the figure title and each panel reaching `reaches`
    /// past its plot area.
    fn areas(&self, size: kurbo::Size, reserve: f64, reaches: &[Reach]) -> Vec<kurbo::Rect> {
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
        let mut row_reach = vec![[0.0_f64; 2]; rows];
        let mut col_reach = vec![[0.0_f64; 2]; cols];
        for (panel, reach) in self.panels.iter().zip(reaches) {
            let (r, c) = panel.cell();
            let row = &mut row_reach[r];
            *row = [row[0].max(reach.top), row[1].max(reach.bottom)];
            let col = &mut col_reach[c];
            *col = [col[0].max(reach.left), col[1].max(reach.right)];
        }
        let ys = layout::tracks(
            self.grid.row_weights(),
            &self.grid.row_gaps_px(rows, em),
            &row_reach,
            [pad + reserve, size.height - 2.0 * pad - reserve],
        );
        let xs = layout::tracks(
            self.grid.col_weights(),
            &self.grid.col_gaps_px(cols, em),
            &col_reach,
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

/// `key`'s shaped label; `None` when typst rejected it.
fn get<'c>(text: &'c text::Cache, key: &Key) -> Option<&'c Shaped> {
    text.get(key)?.ok()
}

/// The ink height of a shaped label.
fn ink(shaped: &Shaped) -> f64 {
    shaped.ascent() + shaped.descent()
}
