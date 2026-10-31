//! Marks: the [`Mark`] trait every mark implements, [`Ctx`], what a mark
//! sees while it draws one panel, and the figure's marks: [`dots`],
//! [`line()`], [`area()`], [`ribbon`], [`ribbon_series`], [`bar_y`],
//! [`bar_x`], [`bar_y_spans`], [`bar_x_spans`], [`rect()`],
//! [`histogram()`], [`density_y`], [`density_x`], [`image()`],
//! [`note()`], [`rule_x`] and [`rule_y`].
//!
//! A mark of your own draws as these do with the theme's readers
//! ([`Theme::get_color`](crate::Theme::get_color),
//! [`Theme::get_line_width`](crate::Theme::get_line_width), …),
//! [`list::brush`](crate::list::brush), [`Ctx::snap`] and the [`guard`]
//! band, and labels what it draws through the text kit: it reports each
//! label with [`Fits::label`] while fitting and places the shaped label
//! from [`Ctx::label`] while drawing, numbers written in [`Ctx::locale`]
//! and baselines set from [`Ctx::line`].
//!
//! ```
//! use rustd3_core::scale;
//! use rustd3_figure::Figure;
//! use rustd3_figure::mark::{dots, rule_y};
//!
//! let mut fig = Figure::new();
//! let x = fig.fit(scale::linear());
//! let y = fig.fit(scale::linear());
//! fig.panel(0, 0)
//!     .mark(dots([(0.0, 0.0), (1.0, 2.0), (2.0, 1.0)], x, y))
//!     .mark(rule_y(1.5, y).dash([6.0, 3.0]).label("limit"));
//! ```

mod area;
mod bar;
mod channel;
mod color;
mod density;
mod dots;
mod error;
pub mod guard;
mod histogram;
mod image;
mod line;
mod rect;
mod rule;
mod snap;
mod text;

use std::any::Any;
use std::cell::{RefCell, RefMut};

use kurbo::{BezPath, Cap, Join, Point};
use rustd3_core::series::{Points, m4};

use crate::check::{self, Issue};
use crate::figure::{Fits, Resolved};
use crate::hatch::Hatch;
use crate::hit;
use crate::list::{Clip, List};
use crate::mark::color::Color;
use crate::scale::{Indexed, Kind, Position, ScaleRef};
use crate::text::{Key, Shaped};
use crate::theme::Theme;
use crate::thin::{self, Thin};

pub use area::{Area, Ribbon, area, ribbon, ribbon_series};
pub use bar::{BarX, BarY, bar_x, bar_x_spans, bar_y, bar_y_spans};
pub use density::{DensityX, DensityY, density_x, density_y};
pub use dots::{Dots, dots};
pub use error::Error;
pub use histogram::{Binned, Bins, Histogram, histogram};
pub use image::{Grid, Image, image};
pub use line::{Line, line};
pub use rect::{Rect, rect};
pub use rule::{Rule, rule_x, rule_y};
pub use snap::Snap;
pub use text::{Note, note};

/// A mark: data drawn through the figure's scales in one panel. Its data
/// may borrow for `'a` (a recording's
/// [`Points`], say), so a figure that borrows
/// is built where the data lives.
pub trait Mark<'a>: 'a {
    /// Reports the values each fitted handle must span ([`Fits::add`]),
    /// and the values margins stop at ([`Fits::stick`]).
    fn fit(&self, fits: &mut Fits);

    /// Appends the mark's commands to `list`, in the panel's coordinates:
    /// the plot area spans `[0, w] × [0, h]`, and the figure clips to it.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List);

    /// Reports the points the mark draws, each with its datum's index, in
    /// the panel's coordinates, for hit testing
    /// ([`Figure::hits`](crate::Figure::hits)); none by default.
    fn hits(&self, _ctx: &Ctx<'_>, _sink: &mut hit::Sink) {}

    /// Reports what [`Figure::check`](crate::Figure::check) should know of
    /// the mark, under [`Ctx::panel`] and [`Ctx::mark`]; nothing by
    /// default.
    fn issues(&self, _ctx: &Ctx<'_>, _issues: &mut Vec<Issue>) {}
}

/// What a mark sees while it draws one panel: the resolved scales over the
/// panel's pixel ranges, its plot area, the theme and the device scale,
/// and which panel and mark it is.
pub struct Ctx<'r> {
    resolved: &'r Resolved,
    theme: &'r Theme,
    rect: kurbo::Rect,
    text: &'r crate::text::Cache,
    frame: &'r RefCell<thin::Frame>,
    panel: usize,
    mark: usize,
}

impl<'r> std::fmt::Debug for Ctx<'r> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ctx").finish_non_exhaustive()
    }
}

impl<'r> Ctx<'r> {
    /// What the marks of the panel whose plot area is `rect` see; they
    /// thin through the draw's `frame`.
    pub(crate) fn new(
        resolved: &'r Resolved,
        theme: &'r Theme,
        rect: kurbo::Rect,
        text: &'r crate::text::Cache,
        frame: &'r RefCell<thin::Frame>,
    ) -> Ctx<'r> {
        Ctx {
            resolved,
            theme,
            rect,
            text,
            frame,
            panel: 0,
            mark: 0,
        }
    }

    /// The same view for mark `mark` of panel `panel`.
    pub(crate) fn at(&self, panel: usize, mark: usize) -> Ctx<'r> {
        Ctx {
            panel,
            mark,
            ..*self
        }
    }
}

impl Ctx<'_> {
    /// `h`'s resolved scale across the plot area, over [0, w]; `None` for
    /// a handle of another figure, which a mark draws nothing for.
    #[must_use]
    pub fn x<S: Position>(&self, h: ScaleRef<S>) -> Option<S> {
        self.ranged(h, [0.0, self.rect.width()])
    }

    /// `h`'s resolved scale up the plot area, over [h, 0]; `None` for a
    /// handle of another figure.
    #[must_use]
    pub fn y<S: Position>(&self, h: ScaleRef<S>) -> Option<S> {
        self.ranged(h, [self.rect.height(), 0.0])
    }

    /// `h`'s resolved scale as it is, before any panel's range (a palette,
    /// say); `None` for a handle of another figure.
    #[must_use]
    pub fn raw<S: Kind>(&self, h: ScaleRef<S>) -> Option<S> {
        self.scale(h).cloned()
    }

    /// The plot area, in list px.
    #[must_use]
    pub fn rect(&self) -> kurbo::Rect {
        self.rect
    }

    /// The figure's theme.
    #[must_use]
    pub fn theme(&self) -> &Theme {
        self.theme
    }

    /// The panel's index, in the order of first use.
    #[must_use]
    pub fn panel(&self) -> usize {
        self.panel
    }

    /// The mark's index in its panel, in insertion order.
    #[must_use]
    pub fn mark(&self) -> usize {
        self.mark
    }

    /// Device pixels per list px of the target drawn for: 1 for vector
    /// output.
    #[must_use]
    pub fn scale_factor(&self) -> f64 {
        self.resolved.target().scale_factor()
    }

    /// How the target drawn for lays edges and hairlines on its pixel grid:
    /// a custom mark snaps its filled boxes ([`Snap::rect`]), stroke widths
    /// ([`Snap::width`]) and the centres of its axis-parallel lines
    /// ([`Snap::line`]) through it to come out as crisp as the rect and
    /// rule marks on raster targets; vector output keeps everything as it
    /// is.
    #[must_use]
    pub fn snap(&self) -> Snap {
        Snap::of(self.resolved.target())
    }

    /// Thinning columns per list px: one per device pixel on screen, four
    /// per device pixel for files and per list pixel for vector output.
    pub(crate) fn columns(&self) -> f64 {
        self.resolved.target().columns()
    }

    /// What the marks draw data over, in the panel's coordinates: the plot
    /// area `[0, w] × [0, h]`, grown on every side by the overscan a widget
    /// holds ([`Held::overscan`](crate::Held::overscan)).
    pub(crate) fn view(&self) -> kurbo::Rect {
        let o = self.resolved.overscan();
        self.rect.with_origin(kurbo::Point::ORIGIN).inflate(o, o)
    }

    /// The guard band geometry is clipped to: [`guard::band`] of the plot
    /// area, grown by the overscan.
    pub(crate) fn band(&self) -> kurbo::Rect {
        let o = self.resolved.overscan();
        guard::band(self.rect.width(), self.rect.height()).inflate(o, o)
    }

    /// The column grid lines on the x handle at table position `x` thin
    /// on and anchor their dashes to: the one a widget holds for it
    /// ([`Held::grids`](crate::Held::grids)), else, when the plot area is
    /// grown, the grid from its left edge at the target's density (the
    /// grid a widget holds before a drag moves it); `None` otherwise (the
    /// file outputs, which thin on the target's own columns).
    pub(crate) fn grid(&self, x: usize) -> Option<thin::Grid> {
        self.resolved.grid(x).or_else(|| {
            (self.resolved.overscan() > 0.0).then_some(thin::Grid {
                origin: 0.0,
                density: self.columns(),
            })
        })
    }

    /// The thinning column edges of the plot area for a line on the x
    /// handle at table position `x` whose scale's bridge is `indexed`: on
    /// [`Ctx::grid`] over the view, else `round(w × columns)` columns
    /// across it ([`Target`](crate::Target)'s columns per list px).
    pub(crate) fn edges<D>(&self, indexed: &dyn Indexed<D>, x: usize) -> Option<Vec<D>> {
        let view = self.view();
        match self.grid(x) {
            Some(grid) => thin::edges_on(indexed, [view.x0, view.x1], grid),
            None => thin::edges(indexed, self.rect.width(), self.columns()),
        }
    }

    /// The draw's thinning frame.
    pub(crate) fn frame(&self) -> RefMut<'_, thin::Frame> {
        self.frame.borrow_mut()
    }

    /// `h`'s resolved scale, borrowed; `None` for a handle of another
    /// figure.
    pub(crate) fn scale<S: Kind>(&self, h: ScaleRef<S>) -> Option<&S> {
        self.resolved.scale(h)
    }

    /// The shaped label of `key`, which the mark reported while fitting
    /// ([`Fits::label`]); `None` when it was not reported or typst rejected
    /// it. A custom mark places it into its list with
    /// [`Shaped::place`], sized by [`Shaped::width`] and its line box
    /// ([`Ctx::line`]).
    #[must_use]
    pub fn label(&self, key: &Key) -> Option<&Shaped> {
        self.text.get(key)?.ok()
    }

    /// The figure's locale ([`Figure::locale`](crate::Figure::locale)). A
    /// custom mark that writes numbers (value labels, annotations) formats
    /// them in it, as axes write their tick labels.
    #[must_use]
    pub fn locale(&self) -> &rustd3_core::locale::Locale {
        self.resolved.locale()
    }

    /// The line box labels of `size` px share: the text face's ascender
    /// above the baseline and descender below it, px, as typst reads them.
    /// A custom mark that sets labels by their line (as tick labels and the
    /// helpers' value labels sit), not by each label's ink box, places
    /// their baselines from it: labels with and without descenders then
    /// line up.
    #[must_use]
    pub fn line(&self, size: f64) -> [f64; 2] {
        self.text.fonts().line(size)
    }

    /// `h`'s resolved scale over `range`.
    fn ranged<S: Kind>(&self, h: ScaleRef<S>, range: [f64; 2]) -> Option<S> {
        let ranged = self
            .scale(h)?
            .with_range(range)
            .as_any()
            .downcast_ref::<S>()
            .expect("invariant: a scale's with_range keeps its type")
            .clone();
        Some(ranged)
    }
}

/// Where `v` lies on `scale`: at the middle of its band on band scales;
/// `None` for a key outside the domain or a position that is not finite.
fn at<S: Position>(scale: &S, v: &<S as Kind>::Domain) -> Option<f64> {
    across(scale, v, None, 0.5)
}

/// Where `v` lies across its band on `scale`, dodged into `slot` (see
/// [`dodge`]): `share` of the way across the slot (0.5: its middle); the
/// position itself on scales without bands. `None` for a key outside the
/// domain or a position that is not finite.
fn across<S: Position>(
    scale: &S,
    v: &<S as Kind>::Domain,
    slot: Option<(usize, usize)>,
    share: f64,
) -> Option<f64> {
    let (start, width) = dodge((scale.position(v)?, scale.bandwidth()), slot);
    let p = start + width * share;
    p.is_finite().then_some(p)
}

/// A channel value as a scale's domain value: `Some` when the domain is
/// `f64`, which it is whenever a mark's data is a recording channel.
fn value<D: Clone + 'static>(v: f64) -> Option<D> {
    (&v as &dyn Any).downcast_ref::<D>().cloned()
}

/// Whether `scale` admits the channel value `v`.
fn admits<S: Kind>(scale: &S, v: f64) -> bool {
    value::<<S as Kind>::Domain>(v).is_some_and(|v| scale.admits(&v))
}

/// Reports `points` on `x` and `y`, and `x` as placed across the plot
/// area: a recording channel through the x scale's index (its first and
/// last x, and its y extremes among the values `y` admits), read in
/// O(log n); anything else pair by pair. When `y` is fitted to what `x`
/// shows (`Fit::Visible`), only the y values of the points `x` shows.
fn fit<'a, X: Position, Y: Position>(
    points: &Points<'a, <X as Kind>::Domain, <Y as Kind>::Domain>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    fits: &mut Fits,
) {
    fits.across(x);
    if let Some(window) = fits.window(x, y) {
        visible(points, &window, y, fits);
        return;
    }
    let extent = points.indexed().and_then(|channel| {
        let ys = fits.scale(y)?;
        fits.scale(x)?
            .indexed()?
            .series_extent(&channel, None, &|v| admits(ys, v))
    });
    match extent {
        Some(extent) => {
            for v in &extent.x {
                fits.add(x, v);
            }
            for v in extent.y.iter().flatten() {
                if let Some(v) = value(*v) {
                    fits.add(y, &v);
                }
            }
        }
        None => {
            for (a, b) in points.iter() {
                fits.add(x, &a);
                fits.add(y, &b);
            }
        }
    }
}

/// Reports to `y` the y values of `points` whose x `window` shows (it
/// places them in [0, 1], ends included): a recording channel's extremes
/// through the x scale's index over `window`'s domain, in O(log n);
/// anything else pair by pair.
fn visible<'a, X: Position, Y: Position>(
    points: &Points<'a, <X as Kind>::Domain, <Y as Kind>::Domain>,
    window: &X,
    y: ScaleRef<Y>,
    fits: &mut Fits,
) {
    if let (Some(channel), Some(indexed), Some(ys)) =
        (points.indexed(), window.indexed(), fits.scale(y))
    {
        let extent = indexed.series_extent(&channel, Kind::domain(window), &|v| admits(ys, v));
        for v in extent.iter().flat_map(|e| e.y).flatten() {
            if let Some(v) = value(v) {
                fits.add(y, &v);
            }
        }
        return;
    }
    for (a, b) in points.iter() {
        if window
            .position(&a)
            .is_some_and(|t| (0.0..=1.0).contains(&t))
        {
            fits.add(y, &b);
        }
    }
}

/// M4's sample indices per column of the view ([`Ctx::edges`]) for
/// `points` on `sx` (the scale of handle `x`) and `sy`, when they thin:
/// [`Thin::Auto`], an x scale with an index bridge, and a recording
/// channel for data.
fn thinned<X: Position, Y: Position>(
    points: &Points<'_, <X as Kind>::Domain, <Y as Kind>::Domain>,
    (thin, x): (Thin, ScaleRef<X>),
    ctx: &Ctx<'_>,
    sx: &X,
    sy: &Y,
) -> Option<m4::Lines> {
    if thin == Thin::Off {
        return None;
    }
    let channel = points.indexed()?;
    let indexed = sx.indexed()?;
    let edges = ctx.edges(indexed, x.index())?;
    indexed.thin(&channel, &edges, &|v| admits(sy, v), &mut ctx.frame())
}

/// The issues every mark over `points` in `color` may have: collected data
/// over 100,000 points, and a key its palette lacks.
fn issues<X, Y>(points: &Points<'_, X, Y>, color: &Color, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
    let (panel, mark) = (ctx.panel(), ctx.mark());
    if points.is_collected() && points.len() > check::LARGE {
        issues.push(Issue::LargeOwned {
            panel,
            mark,
            points: points.len(),
        });
    }
    unknown(color, ctx, issues);
}

/// A key `color`'s palette lacks.
fn unknown(color: &Color, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
    if !color.known(ctx) {
        issues.push(Issue::UnknownKey {
            panel: ctx.panel(),
            mark: ctx.mark(),
        });
    }
}

/// Reports `count` keys outside `handle`'s band or point domain.
fn outside_count(handle: usize, count: usize, issues: &mut Vec<Issue>) {
    if count > 0 {
        issues.push(Issue::OutsideDomain { handle, count });
    }
}

/// Reports the keys of `points` that `x` or `y` cannot place (band and
/// point keys outside their domain, which draw nothing), one issue per
/// handle with their count. Recording channels, whose x is a coordinate
/// on a continuous scale, are not read.
fn outside<X: Position, Y: Position>(
    points: &Points<'_, <X as Kind>::Domain, <Y as Kind>::Domain>,
    (x, y): (ScaleRef<X>, ScaleRef<Y>),
    ctx: &Ctx<'_>,
    issues: &mut Vec<Issue>,
) {
    if points.indexed().is_some() {
        return;
    }
    let (Some(sx), Some(sy)) = (ctx.scale(x), ctx.scale(y)) else {
        return;
    };
    let (mut across, mut up) = (0, 0);
    for (a, b) in points.iter() {
        across += usize::from(sx.position(&a).is_none());
        up += usize::from(sy.position(&b).is_none());
    }
    outside_count(x.index(), across, issues);
    outside_count(y.index(), up, issues);
}

/// Whether `p` lies in `bounds`, edges included.
pub(crate) fn within(bounds: kurbo::Rect, p: Point) -> bool {
    (bounds.x0..=bounds.x1).contains(&p.x) && (bounds.y0..=bounds.y1).contains(&p.y)
}

/// Where `v` lies along `scale`, a value axis, clamped to `[lo, hi]` (a
/// guard band's extent along it). A value the scale's transform does not
/// take (0 or less on a log scale) lies at the end of the guard band on
/// the side of the domain it lies beyond, as matplotlib clips it. `None`
/// for NaN.
pub(crate) fn reach<S: Position<Domain = f64>>(
    scale: &S,
    v: f64,
    [lo, hi]: [f64; 2],
) -> Option<f64> {
    if v.is_nan() {
        return None;
    }
    let p = scale.position(&v)?;
    if !p.is_nan() {
        return Some(p.clamp(lo, hi));
    }
    let [d0, d1] = Kind::domain(scale)?;
    let (low, high) = (d0.min(d1), d0.max(d1));
    let (beyond, other) = if v < low {
        (low, high)
    } else if v > high {
        (high, low)
    } else {
        return None;
    };
    let toward = scale.position(&beyond)? > scale.position(&other)?;
    Some(if toward { hi } else { lo })
}

/// The part of the band `(start, width)` a dodged mark takes in slot
/// `(k, n)`: the k-th of n equal sub-bands, in the palette's order; the
/// whole band without a slot.
fn dodge((start, width): (f64, f64), slot: Option<(usize, usize)>) -> (f64, f64) {
    match slot {
        Some((k, n)) if n > 0 => {
            let w = width / n as f64;
            (start + k as f64 * w, w)
        }
        Some(_) | None => (start, width),
    }
}

/// The runs of consecutive `Some` items, in order.
fn runs<T>(items: impl IntoIterator<Item = Option<T>>) -> Vec<Vec<T>> {
    let mut runs = vec![Vec::new()];
    for item in items {
        match item {
            Some(item) => runs
                .last_mut()
                .expect("invariant: one run is open")
                .push(item),
            None => {
                if runs.last().is_some_and(|run| !run.is_empty()) {
                    runs.push(Vec::new());
                }
            }
        }
    }
    runs.retain(|run| !run.is_empty());
    runs
}

/// Appends the closed polygon through `points` to `path`, clipped to
/// `band`; nothing when fewer than three vertices are left.
fn polygon(path: &mut BezPath, points: &[Point], band: kurbo::Rect) {
    let clipped = guard::polygon(points, band);
    let Some((&first, rest)) = clipped.split_first() else {
        return;
    };
    if rest.len() < 2 {
        return;
    }
    path.move_to(first);
    for &p in rest {
        path.line_to(p);
    }
    path.close_path();
}

/// A mark's stroke: `width` px, butt caps, round joins (matplotlib's
/// lines), `dash` from offset 0 at the first point drawn.
fn stroke(width: f64, dash: &[f64]) -> kurbo::Stroke {
    kurbo::Stroke::new(width)
        .with_caps(Cap::Butt)
        .with_join(Join::Round)
        .with_dashes(0.0, dash.iter().copied())
}

/// A dash pattern of on and off lengths in px: `None` unless every length
/// is finite and not negative and their sum is positive.
pub(crate) fn dashes(pattern: impl IntoIterator<Item = f64>) -> Option<Vec<f64>> {
    let pattern: Vec<f64> = pattern.into_iter().collect();
    let valid = pattern.iter().all(|v| v.is_finite() && *v >= 0.0);
    (valid && pattern.iter().sum::<f64>() > 0.0).then_some(pattern)
}

/// Appends `hatch`, when set, over a shape of the panel `ctx` draws:
/// clipped by `clip`, over its bounding box `bounds` in the panel's
/// coordinates, its lines anchored to the figure.
fn hatch(hatch: Option<&Hatch>, clip: Clip, bounds: kurbo::Rect, ctx: &Ctx<'_>, list: &mut List) {
    if let Some(hatch) = hatch {
        let origin = ctx.rect().origin().to_vec2();
        hatch.draw(clip, bounds, origin, ctx.theme(), list);
    }
}

/// A positive, finite length in px.
fn positive(px: f64) -> Option<f64> {
    (px.is_finite() && px > 0.0).then_some(px)
}

#[cfg(test)]
mod tests {
    use rustd3_core::scale;

    use super::*;

    #[test]
    fn marks_sit_at_the_middle_of_a_band() {
        let band = scale::band(["a", "b"]).range([0.0, 100.0]);
        assert_eq!(at(&band, &"b"), Some(75.0));
        assert_eq!(at(&band, &"z"), None);
        let log = scale::log().range([0.0, 100.0]);
        assert_eq!(at(&log, &-1.0), None);
        assert_eq!(at(&log, &10.0), Some(100.0));
    }

    #[test]
    fn channel_values_reach_only_f64_domains() {
        assert_eq!(value::<f64>(2.5), Some(2.5));
        assert_eq!(value::<&str>(2.5), None);
        assert!(admits(&scale::log(), 3.0));
        assert!(!admits(&scale::log(), -3.0));
        assert!(!admits(&scale::band(["a"]), 0.0));
    }

    #[test]
    fn dodged_marks_take_their_slot_of_the_band() {
        assert_eq!(dodge((10.0, 30.0), Some((2, 3))), (30.0, 10.0));
        assert_eq!(dodge((10.0, 30.0), None), (10.0, 30.0));
        assert_eq!(dodge((10.0, 30.0), Some((0, 0))), (10.0, 30.0));
    }

    #[test]
    fn reaches_clamp_to_the_guard_band_and_drop_nan() {
        let band = [-100.0, 200.0];
        let log = scale::log().domain([1.0, 10.0]).range([100.0, 0.0]);
        assert_eq!(reach(&log, 0.0, band), Some(200.0));
        assert_eq!(reach(&log, -1.0, band), Some(200.0));
        assert_eq!(reach(&log, 10.0, band), Some(0.0));
        assert_eq!(reach(&log, 1e9, band), Some(-100.0));
        assert_eq!(reach(&log, f64::NAN, band), None);
        let flipped = scale::log().domain([10.0, 1.0]).range([100.0, 0.0]);
        assert_eq!(reach(&flipped, 0.0, band), Some(-100.0));
        let negative = scale::log().domain([-10.0, -1.0]).range([100.0, 0.0]);
        assert_eq!(reach(&negative, 0.0, band), Some(-100.0));
    }

    #[test]
    fn runs_split_at_every_gap() {
        let items = [None, Some(1), Some(2), None, None, Some(3), None];
        assert_eq!(runs(items), [vec![1, 2], vec![3]]);
        assert!(runs::<u8>([None, None]).is_empty());
    }

    #[test]
    fn invalid_dash_patterns_and_lengths_are_rejected() {
        assert_eq!(dashes([6.0, 3.0]), Some(vec![6.0, 3.0]));
        assert_eq!(dashes([0.0, 0.0]), None);
        assert_eq!(dashes([4.0, -1.0]), None);
        assert_eq!(dashes([f64::NAN]), None);
        assert_eq!(dashes([]), None);
        assert_eq!(positive(2.0), Some(2.0));
        assert_eq!([0.0, -1.0, f64::INFINITY].map(positive), [None; 3]);
    }

    #[test]
    fn mark_strokes_are_butt_capped_with_round_joins() {
        let s = stroke(2.0, &[6.0, 3.0]);
        assert_eq!(
            (s.start_cap, s.end_cap, s.join),
            (Cap::Butt, Cap::Butt, Join::Round)
        );
        assert_eq!(
            (s.dash_offset, s.dash_pattern.as_slice()),
            (0.0, &[6.0, 3.0][..])
        );
    }
}
