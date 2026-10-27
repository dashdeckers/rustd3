//! Resolving a figure: domains fitted to the marks' data or held by a
//! gesture, labels measured through the text cache, and the grid laid out
//! in passes.

use crate::figure::fits::{Fits, Reports};
use crate::figure::{Cache, Entry, Figure, Held, Resolved};
use crate::layout::{self, Layout};
use crate::scale::{Scale, visible, window};
use crate::target::Target;
use crate::text::Key;

/// Rounds of fitting and layout at most while windows' column counts
/// move.
const ROUNDS: usize = 3;

/// What fitting a figure's handles to all their marks' data gives, before
/// windows and `Fit::Visible` handles settle: per table entry its scale
/// (held, fitted, or as built), the largest x a window handle's marks
/// admit, the panels whose marks place it across, and whether it is a
/// `Fit::Visible` handle to settle; and the labels the marks draw.
struct Fitted {
    scales: Vec<Box<dyn Scale>>,
    last: Vec<Option<f64>>,
    across: Vec<Vec<usize>>,
    pending: Vec<bool>,
    labels: Vec<Key>,
}

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

    /// Resolves for `size`, in `areas` when they fit it: fits every
    /// handle, then settles windows and `Fit::Visible` handles and lays the
    /// grid out, again while the column counts the windows were fitted in
    /// are not those of the plot areas laid out (at most [`ROUNDS`] times).
    /// The first round fits each window in the count the cache remembers
    /// for it, as `All` when it remembers none; the cache keeps the last
    /// round's counts and `Fit::Visible` memory.
    fn resolve_in(
        &self,
        size: kurbo::Size,
        cache: &mut Cache,
        held: &Held,
        target: Target,
        areas: Option<&layout::Areas>,
    ) -> Resolved {
        let fitted = self.fitted(held);
        let rects = areas.and_then(|a| a.fit(size, self.panels.len()));
        let title = self
            .title
            .as_ref()
            .map(|t| Key::new(t, &self.theme.figure_title()));
        let windowed: Vec<bool> = self
            .scales
            .iter()
            .zip(&fitted.across)
            .map(|(entry, panels)| entry.window.is_some() && !panels.is_empty())
            .collect();
        let mut columns: Vec<Option<u32>> = self
            .scales
            .iter()
            .zip(&windowed)
            .enumerate()
            .map(|(i, (entry, &windowed))| {
                windowed
                    .then(|| cache.windows().get((i, entry.scale.as_any().type_id())))
                    .flatten()
            })
            .collect();
        let mut round = 1;
        loop {
            let mut memory = cache.visible().clone();
            let scales = self.settled(&fitted, &columns, &mut memory);
            let locale = self.locale.clone();
            let mut resolved = Resolved::new(self.id, scales, locale, target, size);
            resolved.bind(&self.panels, windowed.clone());
            let layout = Layout::new(self, &resolved, cache.text(), &fitted.labels, size, rects);
            resolved.place(layout, title.clone(), rects.is_some());
            let laid = self.columns(&fitted, &windowed, &resolved);
            if laid == columns || round == ROUNDS {
                *cache.visible() = memory;
                for (i, (entry, laid)) in self.scales.iter().zip(laid).enumerate() {
                    if let Some(laid) = laid {
                        cache
                            .windows()
                            .set((i, entry.scale.as_any().type_id()), laid);
                    }
                }
                return resolved;
            }
            columns = laid;
            round += 1;
        }
    }

    /// Every scale of the table held, fitted to all its marks' data, or as
    /// built, `Fit::Visible` handles not yet settled; per entry the largest
    /// x a window's marks admit and the panels whose marks place it across;
    /// which entries are `Fit::Visible` handles to settle; and the labels
    /// the marks draw.
    fn fitted(&self, held: &Held) -> Fitted {
        let margin = self.theme.margin();
        let unheld = |i: usize, entry: &Entry| held.scale(i, &*entry.scale).is_none();
        let pending: Vec<bool> = self
            .scales
            .iter()
            .enumerate()
            .map(|(i, entry)| entry.visible && entry.fitter.is_some() && unheld(i, entry))
            .collect();
        let slots = self
            .scales
            .iter()
            .zip(&pending)
            .enumerate()
            .map(|(i, (entry, &pending))| {
                let fitter = entry.fitter.filter(|_| unheld(i, entry) && !pending)?;
                Some(fitter.start(&*entry.scale))
            })
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
        let reports = self.report(Fits::new(self.id, slots, built, self.theme.clone()));
        let mut last = vec![None; self.scales.len()];
        let scales = self
            .scales
            .iter()
            .zip(reports.slots)
            .enumerate()
            .map(|(i, (entry, slot))| {
                match (held.scale(i, &*entry.scale), entry.fitter.zip(slot)) {
                    (Some(scale), _) => scale.boxed(),
                    (None, Some((fitter, slot))) => {
                        if entry.window.is_some() {
                            last[i] = fitter.last(&*slot);
                        }
                        fitter.finish(slot, margin)
                    }
                    (None, None) => entry.scale.boxed(),
                }
            })
            .collect();
        Fitted {
            scales,
            last,
            across: reports.across,
            pending,
            labels: reports.labels,
        }
    }

    /// The scales of `fitted` with each window fitted in its count of
    /// `columns` (windows without one, or whose ends the scale does not
    /// admit, stay fitted to all their data), then the `Fit::Visible`
    /// handles fitted to the data the resolved x scales show, stepped
    /// through `memory`.
    fn settled(
        &self,
        fitted: &Fitted,
        columns: &[Option<u32>],
        memory: &mut visible::Memory,
    ) -> Vec<Box<dyn Scale>> {
        let mut scales: Vec<Box<dyn Scale>> = fitted.scales.iter().map(|s| s.boxed()).collect();
        for (i, entry) in self.scales.iter().enumerate() {
            let (Some(span), Some(last), Some(&Some(count)), Some(fitter)) =
                (entry.window, fitted.last[i], columns.get(i), entry.fitter)
            else {
                continue;
            };
            let shown =
                window::ends(last, span, count).and_then(|ends| fitter.window(&*entry.scale, ends));
            if let Some(shown) = shown {
                scales[i] = shown;
            }
        }
        if fitted.pending.contains(&true) {
            let slots = self
                .scales
                .iter()
                .zip(&fitted.pending)
                .map(|(entry, &pending)| {
                    let fitter = entry.fitter.filter(|_| pending)?;
                    Some(fitter.start(&*entry.scale))
                })
                .collect();
            let resolved = scales.iter().map(|s| s.boxed()).collect();
            let fits = Fits::new(self.id, slots, resolved, self.theme.clone())
                .visible(fitted.pending.clone());
            let reports = self.report(fits);
            let margin = self.theme.margin();
            for (i, (entry, slot)) in self.scales.iter().zip(reports.slots).enumerate() {
                if let Some((fitter, slot)) = entry.fitter.zip(slot) {
                    scales[i] = fitter.settle(slot, margin, memory, i);
                }
            }
        }
        scales
    }

    /// Per table entry, the column count of a window `windowed` marks: the
    /// thinning columns of the widest plot area of `resolved` among the
    /// panels whose marks place it across, at least 1; `None` for the
    /// others.
    fn columns(&self, fitted: &Fitted, windowed: &[bool], resolved: &Resolved) -> Vec<Option<u32>> {
        let per_px = resolved.target().columns();
        let areas = resolved.plot_areas();
        fitted
            .across
            .iter()
            .zip(windowed)
            .map(|(panels, &windowed)| {
                if !windowed {
                    return None;
                }
                let widest = panels
                    .iter()
                    .filter_map(|&p| areas.get(p))
                    .map(|area| (area.width() * per_px).round())
                    .fold(0.0, f64::max);
                (widest >= 1.0 && widest <= f64::from(u32::MAX)).then_some(widest as u32)
            })
            .collect()
    }

    /// What every mark reports to `fits`, panel by panel.
    fn report(&self, mut fits: Fits) -> Reports {
        for (i, panel) in self.panels.iter().enumerate() {
            fits.enter(i);
            for mark in panel.marks() {
                mark.fit(&mut fits);
            }
        }
        fits.into_reports()
    }
}
