//! [`Fits`]: the data extents and sticky values marks report per fitted
//! handle, kept in each scale's own domain type.

use std::any::{Any, TypeId};

use rustd3_core::locale::Locale;

use crate::figure;
use crate::hatch::Hatch;
use crate::legend::Swatch;
use crate::scale::{Kind, Position, Scale, ScaleRef, visible, window};
use crate::text::Key;
use crate::theme::Theme;

/// What marks report while a figure resolves: per fitted handle, the
/// extent of the values the scale admits and the sticky values a margin
/// stops at, in the scale's own domain type (no `Timestamp` passes through
/// `f64`). Handles that are not fitted, or that a gesture holds, ignore
/// what is reported. The figure's own marks also report the labels they
/// draw, so that they are shaped in the figure's one batch of text, and
/// the palette keys they draw in, with what they draw (a line, dots, a
/// filled shape) and their hatch, which legend swatches repeat. A handle
/// of another figure is ignored, and recorded with the panel whose mark
/// reported it.
pub struct Fits {
    figure: figure::Id,
    /// One per table entry: a `Slot<S>` for the fitted ones.
    slots: Vec<Option<Box<dyn Any>>>,
    /// One per table entry: the scale as held, or as built.
    scales: Vec<Box<dyn Scale>>,
    /// The figure's theme, which label styles come from.
    theme: Theme,
    /// The figure's locale, which marks write their numbers in.
    locale: Locale,
    /// The labels marks draw.
    labels: Vec<Key>,
    /// The palette keys marks draw in.
    keys: Vec<Reported>,
    /// In the pass that fits `Fit::Visible` handles to what x shows: per
    /// table entry, whether it is such a handle (so its scale is not
    /// resolved yet); `None` in the first pass.
    pending: Option<Vec<bool>>,
    /// The panel whose marks report now.
    panel: usize,
    /// Per table entry, the panels whose marks place it across their plot
    /// areas (their x), each once.
    across: Vec<Vec<usize>>,
    /// The (panel, handle position) of each handle of another figure a
    /// mark reported, each once.
    foreign: Vec<(usize, usize)>,
}

impl std::fmt::Debug for Fits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fits").finish_non_exhaustive()
    }
}

/// What the marks reported: per table entry its slot, filled when it is
/// fitted and not held, and the panels that place it across; the labels
/// marks draw; and the palette keys they draw in.
pub(super) struct Reports {
    pub(super) slots: Vec<Option<Box<dyn Any>>>,
    pub(super) across: Vec<Vec<usize>>,
    pub(super) labels: Vec<Key>,
    pub(super) keys: Vec<Reported>,
    pub(super) foreign: Vec<(usize, usize)>,
}

/// A mark drawn in a palette's color for a key: the palette's position in
/// the scale table, the key (the palette's domain type, boxed), the swatch
/// that stands for the mark in a legend, and its hatch.
pub(crate) struct Reported {
    pub(crate) palette: usize,
    pub(crate) key: Box<dyn Any>,
    pub(crate) glyph: Swatch,
    pub(crate) hatch: Option<Hatch>,
}

/// One fitted handle's scale as built, its extent so far and its sticky
/// values.
struct Slot<S: Kind> {
    scale: S,
    extent: Option<[S::Domain; 2]>,
    sticky: Vec<S::Domain>,
}

/// How a fitted entry of the table is fitted: functions instantiated for
/// its type when it is added, so the table stays erased.
#[derive(Clone, Copy)]
pub(super) struct Fitter {
    /// A slot for the entry's scale.
    start: fn(&dyn Scale) -> Box<dyn Any>,
    /// The scale of a filled slot, fitted with a margin.
    finish: fn(Box<dyn Any>, f64) -> Box<dyn Scale>,
    /// The scale of a filled slot, rounded outward to nice values for a
    /// tick count.
    nice: fn(Box<dyn Any>, usize) -> Box<dyn Scale>,
    /// The scale of a filled slot of a `Fit::Visible` handle, fitted with
    /// a margin and stepped from what the memory holds for its index.
    settle: Settle,
    /// The largest value a filled slot's extent holds, boxed.
    last: fn(&dyn Any) -> Option<Box<dyn Any>>,
    /// The scale as built showing a window.
    window: Window,
}

/// [`Fitter::window`]'s function: the scale as built, the last value its
/// marks admit, the window's span and its column count.
type Window = fn(&dyn Scale, &dyn Any, &dyn Any, u32) -> Option<Box<dyn Scale>>;

/// [`Fitter::settle`]'s function: a slot, the margin, the memory and the
/// entry's index.
type Settle = fn(Box<dyn Any>, f64, &mut visible::Memory, usize) -> Box<dyn Scale>;

impl Fitter {
    /// The fitter of scales of type `S`.
    pub(super) fn of<S: Kind>() -> Fitter {
        Fitter {
            start: start::<S>,
            finish: finish::<S>,
            nice: nice::<S>,
            settle: settle::<S>,
            last: last::<S>,
            window: window::<S>,
        }
    }

    /// The largest value `slot`, a filled slot of this fitter's, holds,
    /// boxed: a window's last x.
    pub(super) fn last(self, slot: &dyn Any) -> Option<Box<dyn Any>> {
        (self.last)(slot)
    }

    /// `scale`, this fitter's type as built, showing exactly the newest
    /// `span` up to `last` in `columns` whole columns
    /// ([`window::shown`]), with no margin; `None` when the domain is
    /// neither numbers nor instants or the scale does not admit both ends.
    pub(super) fn window(
        self,
        scale: &dyn Scale,
        last: &dyn Any,
        span: &dyn Any,
        columns: u32,
    ) -> Option<Box<dyn Scale>> {
        (self.window)(scale, last, span, columns)
    }

    /// The scale of `slot`, entry `index`'s, fitted with `margin` as a
    /// `Fit::Visible` handle through `memory`: [`visible::Memory::settle`]
    /// for numbers and instants, the last domain when no value is in view.
    pub(super) fn settle(
        self,
        slot: Box<dyn Any>,
        margin: f64,
        memory: &mut visible::Memory,
        index: usize,
    ) -> Box<dyn Scale> {
        (self.settle)(slot, margin, memory, index)
    }

    /// An empty slot for `scale`.
    pub(super) fn start(self, scale: &dyn Scale) -> Box<dyn Any> {
        (self.start)(scale)
    }

    /// The scale of `slot`, fitted with `margin`.
    pub(super) fn finish(self, slot: Box<dyn Any>, margin: f64) -> Box<dyn Scale> {
        (self.finish)(slot, margin)
    }

    /// The scale of `slot`, rounded outward to nice values for `count`
    /// ticks ([`Kind::niced`]).
    pub(super) fn nice(self, slot: Box<dyn Any>, count: usize) -> Box<dyn Scale> {
        (self.nice)(slot, count)
    }
}

impl Fits {
    /// Reports for figure `figure` drawn with `theme` and `locale`: one
    /// slot and one scale (as held, or as built) per table entry.
    pub(super) fn new(
        figure: figure::Id,
        slots: Vec<Option<Box<dyn Any>>>,
        scales: Vec<Box<dyn Scale>>,
        theme: Theme,
        locale: Locale,
    ) -> Fits {
        let across = vec![Vec::new(); scales.len()];
        Fits {
            figure,
            slots,
            scales,
            theme,
            locale,
            labels: Vec::new(),
            keys: Vec::new(),
            pending: None,
            panel: 0,
            across,
            foreign: Vec::new(),
        }
    }

    /// The same reports, for the pass that fits the `Fit::Visible` handles
    /// marked in `pending` (one flag per table entry) to what x shows,
    /// once every other handle is resolved.
    pub(super) fn visible(self, pending: Vec<bool>) -> Fits {
        Fits {
            pending: Some(pending),
            ..self
        }
    }

    /// In the pass that fits `Fit::Visible` handles, for a mark on `x` and
    /// `y`: `x`'s resolved scale over [0, 1] when `y` is such a handle
    /// (the values `x` shows are those it places in [0, 1]); `None` in the
    /// first pass, for other `y`s, and when `x` is itself a `Fit::Visible`
    /// handle (no cycles: `y` then sees all of its data).
    pub(crate) fn window<X: Position, Y: Kind>(
        &mut self,
        x: ScaleRef<X>,
        y: ScaleRef<Y>,
    ) -> Option<X> {
        let Some(index) = x.of(self.figure) else {
            self.note(x.get_index());
            return None;
        };
        let pending = self.pending.as_ref()?;
        if pending.get(index).copied().unwrap_or(true) {
            return None;
        }
        self.slot(y)?;
        self.scale(x)?
            .with_range([0.0, 1.0])
            .as_any()
            .downcast_ref::<X>()
            .cloned()
    }

    /// Widens `h`'s extent by `v` when its scale admits it (finite and
    /// valid under the transform; log: > 0).
    pub fn add<S: Kind>(&mut self, h: ScaleRef<S>, v: &S::Domain) {
        if let Some(slot) = self.slot(h)
            && slot.scale.admits(v)
        {
            S::merge(&mut slot.extent, v);
        }
    }

    /// A value `h`'s margin stops at, as matplotlib's sticky edges: bars
    /// and areas stick at their base. It does not widen the extent, so a
    /// mark adds its base too.
    pub fn stick<S: Kind>(&mut self, h: ScaleRef<S>, v: S::Domain) {
        if let Some(slot) = self.slot(h) {
            slot.sticky.push(v);
        }
    }

    /// `h`'s scale before fitting: as a gesture holds it, or as built (a
    /// mark's bridge to its x scale's index, its y scale's admission test);
    /// `None` for a handle of another figure.
    pub(crate) fn scale<S: Kind>(&self, h: ScaleRef<S>) -> Option<&S> {
        self.scales.get(h.of(self.figure)?)?.as_any().downcast_ref()
    }

    /// The figure's theme, as [`Ctx::get_theme`](crate::mark::Ctx::get_theme) gives
    /// it while drawing. A custom mark that reports labels builds their
    /// [`Key`]s here from the theme's colors and sizes, so that the keys it
    /// looks up while drawing are the ones it reported.
    #[must_use]
    pub fn get_theme(&self) -> &Theme {
        &self.theme
    }

    /// The figure's locale ([`Figure::locale`](crate::Figure::locale)), as
    /// [`Ctx::get_locale`](crate::mark::Ctx::get_locale) gives it while drawing. A
    /// custom mark that labels numbers writes them in it here, so that the
    /// labels it reports are the ones it draws.
    #[must_use]
    pub fn get_locale(&self) -> &Locale {
        &self.locale
    }

    /// A label a mark draws, shaped with the figure's other labels in the
    /// figure's one batch of typst text. A custom mark that draws text
    /// reports each label's [`Key`] here while fitting and reads its shape
    /// back with [`Ctx::get_label`](crate::mark::Ctx::get_label) while drawing, as
    /// the rule marks' labels and the helpers' value labels do; a label not
    /// reported is not shaped.
    ///
    /// ```
    /// use rustd3_core::color::Rgb;
    /// use rustd3_figure::list::List;
    /// use rustd3_figure::mark::{Ctx, Mark};
    /// use rustd3_figure::text::{Anchor, Key, Style, Text, anchor};
    /// use rustd3_figure::kurbo::Point;
    /// use rustd3_figure::{Figure, Fits};
    ///
    /// /// A word at the middle of the plot area.
    /// struct Word(Key);
    ///
    /// impl<'a> Mark<'a> for Word {
    ///     fn fit(&self, fits: &mut Fits) {
    ///         fits.label(self.0.clone());
    ///     }
    ///
    ///     fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
    ///         let Some(shaped) = ctx.get_label(&self.0) else { return };
    ///         let [ascender, descender] = ctx.line(16.0);
    ///         let r = ctx.get_rect();
    ///         // The line box's middle at the plot area's.
    ///         let baseline = r.height() / 2.0 + (ascender - descender) / 2.0;
    ///         let at = Point::new(r.width() / 2.0, baseline);
    ///         let anchor = Anchor { h: anchor::Horizontal::Middle, v: anchor::Vertical::Baseline };
    ///         shaped.place(list, at, anchor, 0.0);
    ///     }
    /// }
    ///
    /// let style = Style {
    ///     size: 16.0,
    ///     bold: false,
    ///     fill: Rgb::hex(0x262626),
    ///     align: anchor::Horizontal::Middle,
    /// };
    /// let mut fig = Figure::new();
    /// fig.panel(0, 0).mark(Word(Key::new(&Text::from("centre"), &style)));
    /// # #[cfg(feature = "fonts")] {
    /// use rustd3_figure::list::Command;
    ///
    /// let list = fig.draw(rustd3_figure::kurbo::Size::new(200.0, 100.0));
    /// assert!(list.commands.iter().any(|c| matches!(c, Command::Glyphs(_))));
    /// # }
    /// ```
    pub fn label(&mut self, key: Key) {
        self.labels.push(key);
    }

    /// A mark drawn in palette `palette`'s (its table position) color for
    /// `key` stands as `glyph` in a legend and is hatched with `hatch`,
    /// which legend swatches of that key repeat.
    pub(crate) fn key(
        &mut self,
        palette: ScaleRef<()>,
        key: Box<dyn Any>,
        glyph: Swatch,
        hatch: Option<&Hatch>,
    ) {
        let Some(palette) = palette.of(self.figure) else {
            self.note(palette.get_index());
            return;
        };
        self.keys.push(Reported {
            palette,
            key,
            glyph,
            hatch: hatch.cloned(),
        });
    }

    /// The marks of panel `panel` report next.
    pub(super) fn enter(&mut self, panel: usize) {
        self.panel = panel;
    }

    /// `h` is the x of a mark of the panel reporting now: placed across
    /// its plot area, which a `Fit::Window` handle's columns are counted
    /// in.
    pub(crate) fn across<S: Kind>(&mut self, h: ScaleRef<S>) {
        let Some(index) = h.of(self.figure) else {
            self.note(h.get_index());
            return;
        };
        if let Some(panels) = self.across.get_mut(index)
            && !panels.contains(&self.panel)
        {
            panels.push(self.panel);
        }
    }

    /// What the marks reported.
    pub(super) fn into_reports(self) -> Reports {
        Reports {
            slots: self.slots,
            across: self.across,
            labels: self.labels,
            keys: self.keys,
            foreign: self.foreign,
        }
    }

    /// `h`'s slot, if it is fitted and not held.
    fn slot<S: Kind>(&mut self, h: ScaleRef<S>) -> Option<&mut Slot<S>> {
        let Some(index) = h.of(self.figure) else {
            self.note(h.get_index());
            return None;
        };
        self.slots.get_mut(index)?.as_mut()?.downcast_mut()
    }

    /// Records handle position `handle` of another figure, reported by a
    /// mark of the panel reporting now.
    fn note(&mut self, handle: usize) {
        if !self.foreign.contains(&(self.panel, handle)) {
            self.foreign.push((self.panel, handle));
        }
    }
}

/// An empty slot for `scale`, which is an `S`.
fn start<S: Kind>(scale: &dyn Scale) -> Box<dyn Any> {
    let scale = scale
        .as_any()
        .downcast_ref::<S>()
        .expect("invariant: an entry's fitter is made for its scale's type");
    Box::new(Slot {
        scale: scale.clone(),
        extent: None,
        sticky: Vec::new(),
    })
}

/// The scale of `slot`, a `Slot<S>` of entry `index`, fitted to its extent
/// and sticky values with `margin`, then stepped from the domain `memory`
/// last gave it, when its domain is numbers or instants; with nothing in
/// view, the domain `memory` last gave it, if any.
fn settle<S: Kind>(
    slot: Box<dyn Any>,
    margin: f64,
    memory: &mut visible::Memory,
    index: usize,
) -> Box<dyn Scale> {
    let slot = slot
        .downcast::<Slot<S>>()
        .expect("invariant: an entry's slot is made by its fitter");
    let key = (index, TypeId::of::<S>());
    let fitted = match slot.extent.as_ref() {
        Some(extent) => slot.scale.fitted(Some(extent), &slot.sticky, margin),
        None => match memory
            .last(key)
            .and_then(visible::Domain::ends::<S::Domain>)
        {
            Some(last) => return Box::new(slot.scale.fitted(Some(&last), &[], 0.0)),
            None => return Box::new(slot.scale.fitted(None, &slot.sticky, margin)),
        },
    };
    let padded = Kind::domain(&fitted).and_then(|d| visible::Domain::of(&d));
    let Some(padded) = padded else {
        return Box::new(fitted);
    };
    match memory.settle(key, padded).ends::<S::Domain>() {
        Some(settled) => Box::new(slot.scale.fitted(Some(&settled), &[], 0.0)),
        None => Box::new(fitted),
    }
}

/// The upper end of `slot`'s extent, a `Slot<S>`'s, boxed.
fn last<S: Kind>(slot: &dyn Any) -> Option<Box<dyn Any>> {
    let slot = slot
        .downcast_ref::<Slot<S>>()
        .expect("invariant: an entry's slot is made by its fitter");
    let [_, high] = slot.extent.as_ref()?;
    Some(Box::new(high.clone()))
}

/// `scale`, an `S`, showing exactly the window of `span` up to `last` in
/// `columns` columns, when its domain is numbers or instants and it
/// admits both ends.
fn window<S: Kind>(
    scale: &dyn Scale,
    last: &dyn Any,
    span: &dyn Any,
    columns: u32,
) -> Option<Box<dyn Scale>> {
    let scale = scale
        .as_any()
        .downcast_ref::<S>()
        .expect("invariant: an entry's fitter is made for its scale's type");
    let shown = window::shown(last, span, columns)?;
    let ends = shown.downcast_ref::<[S::Domain; 2]>()?;
    if !ends.iter().all(|v| scale.admits(v)) {
        return None;
    }
    Some(Box::new(scale.fitted(Some(ends), &[], 0.0)))
}

/// The scale of `slot`, a `Slot<S>`, fitted to its extent and sticky
/// values with `margin`.
fn finish<S: Kind>(slot: Box<dyn Any>, margin: f64) -> Box<dyn Scale> {
    let slot = slot
        .downcast::<Slot<S>>()
        .expect("invariant: an entry's slot is made by its fitter");
    Box::new(
        slot.scale
            .fitted(slot.extent.as_ref(), &slot.sticky, margin),
    )
}

/// The scale of `slot`, a `Slot<S>`, fitted to its extent and sticky
/// values with its ends rounded outward for `count` ticks.
fn nice<S: Kind>(slot: Box<dyn Any>, count: usize) -> Box<dyn Scale> {
    let slot = slot
        .downcast::<Slot<S>>()
        .expect("invariant: an entry's slot is made by its fitter");
    Box::new(slot.scale.niced(slot.extent.as_ref(), &slot.sticky, count))
}
