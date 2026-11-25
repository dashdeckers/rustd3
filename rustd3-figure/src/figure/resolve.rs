//! Resolving a figure: domains fitted to the marks' data or held by a
//! gesture, labels measured through the text cache, and the grid laid out
//! in passes.

use std::any::Any;
use std::rc::Rc;

use crate::cache::Cache;
use crate::figure::fits::{Fits, Reported, Reports};
use crate::figure::{Entry, Figure, Held, Resolved};
use crate::layout::{self, Layout};
use crate::scale::{Scale, visible};
use crate::target::Target;
use crate::text::Key;
use crate::theme::Domains;

/// The most rounds of fitting and layout run while windows' column counts
/// move.
const ROUNDS: usize = 3;

/// The tick count nice domains round to for a handle no axis shows: d3's
/// default.
const COUNT: usize = 10;

/// What fitting a figure's handles to all their marks' data gives, before
/// windows and `Fit::Visible` handles settle.
struct Fitted {
    /// Per table entry, its scale: held, fitted, or as built.
    scales: Vec<Box<dyn Scale>>,
    /// Per table entry, the largest x a window handle's marks admit.
    last: Vec<Option<Box<dyn Any>>>,
    /// Per table entry, the panels whose marks place it across their plot
    /// areas.
    across: Vec<Vec<usize>>,
    /// Per table entry, whether it is a `Fit::Visible` handle to settle.
    pending: Vec<bool>,
    /// The labels the marks draw.
    labels: Vec<Key>,
    /// The palette keys the marks draw in.
    keys: Rc<[Reported]>,
    /// The (panel, handle position) of the handles of other figures marks
    /// reported.
    foreign: Vec<(usize, usize)>,
}

impl Figure<'_> {
    /// Fits every fitted handle's domain to the data of every mark that
    /// uses it (held handles keep the held scale), and lays the grid out
    /// for `size`, measuring every label through `cache`'s text cache, one
    /// batch per layout pass. `target` is kept in the result. A side of
    /// `size` that is not a finite number of zero or more is taken as 0:
    /// such a figure has no room for its panels, which
    /// [`check`](Figure::check) reports.
    ///
    /// Each pass takes every axis's tick count from its length in the last
    /// placement, then places the plot areas again, until the counts settle
    /// (at most four passes). The count is one tick per 80 px across or per
    /// 35 px up, at least 2, unless `.ticks(n)` fixes it. Each panel
    /// reaches past its plot area by:
    /// - its axes: tick, tick pad, the labels' extent and, with a title,
    ///   the label pad and the title's ink height, at the axis's offset;
    /// - tick labels that overhang its ends;
    /// - its title: title pad and ink height above everything on top. The
    ///   titles of one grid row share a baseline, so a title also reaches
    ///   at least to its ink top when its last baseline sits the title pad
    ///   and the text face's descender above the row's top-side axes.
    ///
    /// Each column takes the largest reach of its panels per side, and each
    /// row likewise. The rest is split by weight inside 0.8 em of padding,
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
    /// that a gesture took at its start ([`Resolved::get_areas`]), so the
    /// plot does not shift under the pointer. Tick counts come from the
    /// axes' lengths in one pass, labels and collisions are recomputed, and
    /// labels may reach past the old reaches. Areas made for another size
    /// or panel count are ignored: the layout is fresh, and
    /// [`Resolved::get_held`] is false.
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
    /// let size = Size::new(640.0, 480.0);
    /// let target = Target::Screen { scale: 1.0 };
    /// let start = fig.resolve(size, &mut cache, &Held::default(), target);
    /// let mut held = Held::default();
    /// held.hold(y, scale::linear().domain([0.0, 30_000.0]));
    /// let areas = start.get_areas();
    /// let during = fig.resolve_held(size, &mut cache, &held, target, &areas);
    /// assert!(during.get_held());
    /// assert_eq!(during.get_plot_areas(), start.get_plot_areas());
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

    /// Resolves for `size`, in `areas` when they fit it. Fits every handle,
    /// then settles windows and `Fit::Visible` handles and lays the grid
    /// out. Settling and layout repeat while the column counts the windows
    /// were fitted in differ from those of the plot areas laid out, at most
    /// [`ROUNDS`] times. The first round fits each window in the count the
    /// cache remembers for it, or as `All` when it remembers none. The
    /// cache keeps the last round's counts and `Fit::Visible` memory.
    fn resolve_in(
        &self,
        size: kurbo::Size,
        cache: &mut Cache,
        held: &Held,
        target: Target,
        areas: Option<&layout::Areas>,
    ) -> Resolved {
        let side = |v: f64| if v.is_finite() && v >= 0.0 { v } else { 0.0 };
        let size = kurbo::Size::new(side(size.width), side(size.height));
        let rects = areas.and_then(|a| a.fit(size, self.panels.len()));
        let counts = self.nice_counts(size, target, rects);
        let fitted = self.fitted(held, &counts);
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
            let mut resolved = Resolved::new(self.ids(), scales, locale, target, size);
            resolved.bind(&self.panels, windowed.clone());
            resolved.keyed(Rc::clone(&fitted.keys));
            resolved.foreign(fitted.foreign.clone());
            resolved.thin_on(held.grid_table().clone(), held.overscanned());
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

    /// Per table entry, the tick count its domain is rounded to under
    /// `Domains::Nice`; `None` under margins, and for entries that are not
    /// `Fit::All` handles. The count is the one layout's first pass gives
    /// the first axis, in panel order, that shows the entry: its
    /// `.ticks(n)`, else one from its length in the first placement, or in
    /// `rects` when held. It is d3's [`COUNT`] when no axis shows the
    /// entry. The rounded domains are settled before the passes iterate.
    fn nice_counts(
        &self,
        size: kurbo::Size,
        target: Target,
        rects: Option<&[kurbo::Rect]>,
    ) -> Vec<Option<usize>> {
        let rounded =
            |entry: &Entry| entry.fitter.is_some() && entry.window.is_none() && !entry.visible;
        if !matches!(self.theme.get_domains(), Domains::Nice) {
            return vec![None; self.scales.len()];
        }
        let counts = layout::first_counts(self, size, target, rects);
        self.scales
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                rounded(entry).then(|| {
                    self.panels
                        .iter()
                        .zip(&counts)
                        .flat_map(|(p, counts)| p.axes().iter().zip(counts))
                        .find(|(a, _)| a.handle() == i)
                        .map_or(COUNT, |(_, &count)| count)
                })
            })
            .collect()
    }

    /// The margin fitted domains get: the theme's under
    /// `Domains::Margin`, none under `Domains::Nice`.
    fn margin(&self) -> f64 {
        match self.theme.get_domains() {
            Domains::Margin(m) => m,
            Domains::Nice => 0.0,
        }
    }

    /// The table's scales fitted to all their marks' data, with the theme's
    /// margin or rounded for `counts`, and what the marks reported. Held
    /// scales stay as held, scales not fitted stay as built, and
    /// `Fit::Visible` handles are not yet settled.
    fn fitted(&self, held: &Held, counts: &[Option<usize>]) -> Fitted {
        let margin = self.margin();
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
        let reports = self.report(Fits::new(
            self.ids(),
            slots,
            built,
            self.theme.clone(),
            self.locale.clone(),
        ));
        let mut last: Vec<Option<Box<dyn Any>>> = self.scales.iter().map(|_| None).collect();
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
                        match counts.get(i).copied().flatten() {
                            Some(count) => fitter.nice(slot, count),
                            None => fitter.finish(slot, margin),
                        }
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
            keys: reports.keys.into(),
            foreign: reports.foreign,
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
            let (Some(span), Some(last), Some(&Some(count)), Some(fitter)) = (
                entry.window.as_deref(),
                fitted.last[i].as_deref(),
                columns.get(i),
                entry.fitter,
            ) else {
                continue;
            };
            if let Some(shown) = fitter.window(&*entry.scale, last, span, count) {
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
            let fits = Fits::new(
                self.ids(),
                slots,
                resolved,
                self.theme.clone(),
                self.locale.clone(),
            )
            .visible(fitted.pending.clone());
            let reports = self.report(fits);
            let margin = self.margin();
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
        let areas = resolved.get_plot_areas();
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
