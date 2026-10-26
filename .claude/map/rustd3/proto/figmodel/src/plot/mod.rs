//! Option A, second cut. The figure's table keeps each
//! scale's own type, and marks keep typed data, so band
//! keys and palette keys are user types, not numbers.

mod arrange;
mod axis;
pub mod draw;
mod measure;
mod theme;
mod layout;
mod legend;
mod marks;
mod scales;
mod text;

pub use arrange::{AxisPlan, Sides, axis_trace, cells, label_trace, protrusion_of};
pub use axis::{Axis, bottom, left, right, top};
pub use layout::{EM, Grid, Length, Rect, em, px};
pub use legend::{Inset, Legend, Slot, legend};
pub use marks::{Mark, density_y, dots, histogram};
pub use marks::{line, rule_y};
pub use scales::{Kind, Paint, Position, Scale, Ticks};
pub use text::{Span, Text, span};
pub use theme::{Context, Domains, Style, Theme};
pub use measure::{Estimate, Measure, Measurer};

use d3::color::Rgb;
use std::collections::HashMap;
use std::fmt::Display;
use std::marker::PhantomData;

/// A typed reference to a scale the figure owns.
pub struct ScaleRef<S> {
    index: usize,
    kind: PhantomData<fn() -> S>,
}

impl<S> Clone for ScaleRef<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for ScaleRef<S> {}

impl<S> PartialEq for ScaleRef<S> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

struct Entry {
    scale: Box<dyn Scale>,
    fit: bool,
}

type Key = Box<dyn legend::Draw>;

/// A figure: a table of scales, panels in grid cells,
/// and a layout. Built with `&mut`, drawn with `&`.
#[derive(Default)]
pub struct Figure {
    scales: Vec<Entry>,
    panels: Vec<Panel>,
    grid: Grid,
    title: Option<Text>,
    legends: Vec<(Slot, Key)>,
    theme: Theme,
    measure: Measurer,
}

impl Figure {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn measure(&mut self, m: std::rc::Rc<dyn Measure>) -> &mut Self {
        self.measure = Measurer(m);
        self
    }

    pub fn theme(&mut self, t: Theme) -> &mut Self {
        self.theme = t;
        self
    }

    /// A scale whose domain is the one it was built with.
    pub fn scale<S: Kind>(&mut self, s: S) -> ScaleRef<S> {
        self.add(s, false)
    }

    /// A scale whose domain spans the data of every mark
    /// that uses it, in any panel.
    pub fn fit<S: Kind>(&mut self, s: S) -> ScaleRef<S> {
        self.add(s, true)
    }

    fn add<S: Kind>(&mut self, s: S, fit: bool) -> ScaleRef<S> {
        let scale = Box::new(s);
        self.scales.push(Entry { scale, fit });
        let index = self.scales.len() - 1;
        ScaleRef { index, kind: PhantomData }
    }

    /// The panel in grid cell `(row, col)`, made on first use.
    pub fn panel(&mut self, row: usize, col: usize) -> &mut Panel {
        let at = |p: &Panel| p.cell == (row, col);
        let i = match self.panels.iter().position(at) {
            Some(i) => i,
            None => {
                self.panels.push(Panel::new((row, col)));
                self.panels.len() - 1
            }
        };
        &mut self.panels[i]
    }

    pub fn layout(&mut self, grid: Grid) -> &mut Self {
        self.grid = grid;
        self
    }

    /// A title centered above the grid.
    pub fn title(&mut self, t: impl Into<Text>) -> &mut Self {
        self.title = Some(t.into());
        self
    }

    /// A legend in a figure-level slot, centered.
    pub fn legend<K>(
        &mut self,
        slot: Slot,
        l: Legend<K>,
    ) -> &mut Self
    where
        K: PartialEq + Clone + Display + 'static,
    {
        self.legends.push((slot, Box::new(l)));
        self
    }

    /// Pixels the figure-level slots take from the grid.
    fn reserved(&self) -> [f64; 2] {
        let has = |s| self.legends.iter().any(|l| l.0 == s);
        let t = &self.theme;
        let title = 1.8 * t.figure_title_text();
        let top = if self.title.is_some() { title } else { 0.0 };
        let top = top + if has(Slot::Top) { 2.0 * t.em() } else { 0.0 };
        let bottom = if has(Slot::Bottom) { 2.4 * t.em() } else { 0.0 };
        [top, bottom]
    }

    /// Fits domains, then places panels.
    pub fn resolve(&self, size: [f64; 2]) -> Resolved {
        self.resolve_with(size, &HashMap::new())
    }

    /// As `resolve`, with domains a widget keeps for
    /// scales nobody listens to, keyed by scale index.
    pub fn resolve_with(
        &self,
        size: [f64; 2],
        views: &HashMap<usize, [f64; 2]>,
    ) -> Resolved {
        self.resolve_in(size, views, None)
    }

    /// As `resolve_with`, keeping given plot areas: what
    /// the widget does during a gesture, so the plot does
    /// not shift under the pointer while labels change.
    pub fn resolve_in(
        &self,
        size: [f64; 2],
        views: &HashMap<usize, [f64; 2]>,
        frozen: Option<&[Rect]>,
    ) -> Resolved {
        let mut fits = Fits(vec![None; self.scales.len()]);
        for m in self.panels.iter().flat_map(|p| &p.marks) {
            m.fit(&mut fits);
        }
        let scales = self
            .scales
            .iter()
            .zip(fits.0)
            .enumerate()
            .map(|(i, (e, ext))| match (views.get(&i), e.fit, ext) {
                (Some(&d), ..) => e.scale.with_span(d),
                (None, true, Some(d)) => match self.theme.domains {
                    Domains::Nice => nice(&*e.scale, d),
                    Domains::Margin(m) => margin(&*e.scale, d, m),
                },
                _ => e.scale.boxed(),
            })
            .collect();
        let cells: Vec<_> =
            self.panels.iter().map(|p| p.cell).collect();
        let mut r = Resolved {
            size,
            scales,
            rects: vec![],
            plans: vec![],
            keep: vec![],
            passes: 0,
            theme: self.theme.clone(),
            measure: self.measure.clone(),
        };
        let reserved = self.reserved();
        let gaps = self.grid.gaps(self.theme.em());
        let place = |plans: &[Vec<AxisPlan>]| match (frozen, self.grid.fixed()) {
            (Some(f), _) => f.to_vec(),
            (None, Some(m)) => self.grid.rects(size, &cells, reserved, m),
            (None, None) => arrange::rects(
                self.grid.weights(),
                [&gaps[0], &gaps[1]],
                size,
                0.8 * self.theme.em(),
                reserved,
                &cells,
                &arrange::protrusions(self, plans),
            ),
        };
        // Tick counts follow axis lengths, and lengths
        // follow the labels: start from bare tracks, then
        // repeat until the counts stop changing.
        let mut rects = place(&[]);
        let mut last: Option<Vec<Vec<usize>>> = None;
        for pass in 1..=4 {
            r.plans = arrange::plan(self, &r, &rects);
            let counts: Vec<Vec<usize>> = r
                .plans
                .iter()
                .map(|p| p.iter().map(|a| a.count).collect())
                .collect();
            rects = place(&r.plans);
            r.passes = pass;
            if last.as_ref() == Some(&counts) || frozen.is_some() {
                break;
            }
            last = Some(counts);
        }
        r.rects = rects;
        r.keep = arrange::collide(self, &r, &r.plans);
        r
    }

    /// Where the panel in cell `(row, col)` ended up.
    pub fn rect(
        &self,
        r: &Resolved,
        row: usize,
        col: usize,
    ) -> Option<Rect> {
        let at = |p: &Panel| p.cell == (row, col);
        Some(r.rects[self.panels.iter().position(at)?])
    }

    /// What the glue does with a wheel event: every scale
    /// the panel under the cursor zooms gets a new domain.
    pub fn wheel(
        &self,
        r: &Resolved,
        (x, y): (f64, f64),
        lines: f64,
    ) -> Vec<(usize, [f64; 2])> {
        let k = 2f64.powf(-lines * 0.05);
        let hit = r.rects.iter().position(|q| {
            (q.x..=q.x + q.w).contains(&x)
                && (q.y..=q.y + q.h).contains(&y)
        });
        let Some(p) = hit else { return vec![] };
        let q = r.rects[p];
        let at = y - q.y;
        self.panels[p]
            .zoom_y
            .iter()
            .filter_map(|&i| {
                let s = r.scales[i].with_range([q.h, 0.0]);
                let px = [q.h, 0.0].map(|e| at + (e - at) / k);
                let d = [s.invert(px[0])?, s.invert(px[1])?];
                Some((i, d))
            })
            .collect()
    }

    pub fn svg(&self, size: [f64; 2]) -> String {
        self.svg_with(size, &HashMap::new())
    }

    pub fn svg_with(
        &self,
        size: [f64; 2],
        views: &HashMap<usize, [f64; 2]>,
    ) -> String {
        self.draw_with(size, views).svg()
    }

    pub fn draw(&self, size: [f64; 2]) -> draw::List {
        self.draw_with(size, &HashMap::new())
    }

    /// The figure as a drawing list: the one output
    /// every backend reads.
    pub fn draw_with(
        &self,
        size: [f64; 2],
        views: &HashMap<usize, [f64; 2]>,
    ) -> draw::List {
        let r = self.resolve_with(size, views);
        let [w, h] = size;
        let mut o = draw::List::new(size);
        o.fill(kurbo::Rect::new(0.0, 0.0, w, h), WHITE);
        for (i, p) in self.panels.iter().enumerate() {
            let c = Ctx { r: &r, rect: r.rects[i] };
            p.draw(&c, &r.plans[i], &r.keep[i], &mut o);
        }
        // Slots center on the grid, not the canvas.
        let left = r.rects.iter().map(|q| q.x);
        let right = r.rects.iter().map(|q| q.x + q.w);
        let left = left.fold(f64::INFINITY, f64::min);
        let right = right.fold(0.0, f64::max);
        let cx = (left + right) / 2.0;
        let th = &self.theme;
        let ft = th.figure_title_text();
        if let Some(t) = &self.title {
            o.label(
                draw::Label::new(t.clone(), cx, 1.3 * ft)
                    .anchor(draw::Anchor::Middle)
                    .size(ft as f32)
                    .fill(th.ink())
                    .bold(),
            );
        }
        let top = if self.title.is_some() { 1.8 * ft } else { 0.0 } + 1.5 * th.em();
        for (slot, l) in &self.legends {
            let y = match slot {
                Slot::Top => top,
                Slot::Bottom => h - 0.9 * th.em(),
            };
            l.row(&r, cx, y, &mut o);
        }
        o
    }
}

const WHITE: Rgb = Rgb::hex(0xffffff);
const GRAY: Rgb = Rgb::hex(0x8c8c8c);

/// Fitted domains are extended to nice numbers, as d3's
/// `scale.nice()`; the theme decides this (problem #6).
/// Extends a fitted span by `m` of itself on each side,
/// except at zero, where bars and areas start.
fn margin(s: &dyn Scale, [a, b]: [f64; 2], m: f64) -> Box<dyn Scale> {
    let pad = m * (b - a).abs().max(f64::EPSILON);
    let a = if a == 0.0 { a } else { a - pad };
    let b = if b == 0.0 { b } else { b + pad };
    s.with_span([a, b])
}

fn nice(s: &dyn Scale, [a, b]: [f64; 2]) -> Box<dyn Scale> {
    let nice = d3::scale::linear()
        .domain([a, b])
        .nice(5)
        .get_domain();
    s.with_span(nice)
}

/// Data extents per scale, collected from marks.
pub struct Fits(Vec<Option<[f64; 2]>>);

impl Fits {
    pub fn add<S: Kind>(&mut self, s: ScaleRef<S>, v: &S::Domain) {
        if let Some(v) = S::number(v) {
            let e = self.0[s.index].get_or_insert([v, v]);
            *e = [e[0].min(v), e[1].max(v)];
        }
    }
}

/// Final domains and panel rectangles.
pub struct Resolved {
    pub size: [f64; 2],
    scales: Vec<Box<dyn Scale>>,
    pub rects: Vec<Rect>,
    /// Per panel and axis: shown, labeled, offset.
    pub plans: Vec<Vec<AxisPlan>>,
    pub theme: Theme,
    pub measure: Measurer,
    /// Per panel, axis and tick label: kept after the
    /// collision pass.
    pub keep: Vec<Vec<Vec<bool>>>,
    /// Layout passes until tick counts settled.
    pub passes: usize,
}

impl Resolved {
    /// A scale as resolved, before any panel's range.
    pub fn raw<S: Kind>(&self, h: ScaleRef<S>) -> S {
        let s = self.scales[h.index].as_any();
        s.downcast_ref::<S>().expect("same kind").clone()
    }
}

/// What a mark or axis sees while drawing one panel.
pub struct Ctx<'a> {
    r: &'a Resolved,
    pub rect: Rect,
}

impl Ctx<'_> {
    /// Scale `h` across this panel's width.
    pub fn x<S: Kind>(&self, h: ScaleRef<S>) -> S {
        self.ranged(h, [0.0, self.rect.w])
    }

    /// Scale `h` up this panel's height.
    pub fn y<S: Kind>(&self, h: ScaleRef<S>) -> S {
        self.ranged(h, [self.rect.h, 0.0])
    }

    pub fn raw<S: Kind>(&self, h: ScaleRef<S>) -> S {
        self.r.raw(h)
    }

    fn ranged<S: Kind>(&self, h: ScaleRef<S>, r: [f64; 2]) -> S {
        let s = self.r.scales[h.index].with_range(r);
        let s = s.as_any().downcast_ref::<S>();
        s.expect("same kind").clone()
    }
}

/// One plot area: marks, axes, a title.
#[derive(Default)]
pub struct Panel {
    cell: (usize, usize),
    marks: Vec<Box<dyn Mark>>,
    axes: Vec<Box<dyn axis::Draw>>,
    title: Option<Text>,
    border: Option<Rgb>,
    legends: Vec<(Inset, Key)>,
    zoom_y: Vec<usize>,
}

impl Panel {
    fn new(cell: (usize, usize)) -> Self {
        Self { cell, ..Self::default() }
    }

    pub fn mark(&mut self, m: impl Mark) -> &mut Self {
        self.marks.push(Box::new(m));
        self
    }

    /// Takes an axis by value or by reference, so one
    /// axis value can be placed on many panels.
    pub fn axis<S: Ticks>(
        &mut self,
        a: impl Into<Axis<S>>,
    ) -> &mut Self {
        self.axes.push(Box::new(a.into()));
        self
    }

    /// Wheel over this panel rescales `s` vertically, in
    /// every panel that uses it. Call again to zoom
    /// several scales together, e.g. both twins.
    pub fn zoom_y<S: Position>(
        &mut self,
        s: ScaleRef<S>,
    ) -> &mut Self {
        self.zoom_y.push(s.index);
        self
    }

    pub fn title(&mut self, t: impl Into<Text>) -> &mut Self {
        self.title = Some(t.into());
        self
    }

    /// A frame around the plot area.
    pub fn border(&mut self, color: Rgb) -> &mut Self {
        self.border = Some(color);
        self
    }

    /// A legend boxed into a corner of this panel.
    pub fn legend<K>(
        &mut self,
        at: Inset,
        l: Legend<K>,
    ) -> &mut Self
    where
        K: PartialEq + Clone + Display + 'static,
    {
        self.legends.push((at, Box::new(l)));
        self
    }

    fn draw(
        &self,
        c: &Ctx,
        plans: &[AxisPlan],
        keep: &[Vec<bool>],
        o: &mut draw::List,
    ) {
        let Rect { x, y, w, h } = c.rect;
        let area = kurbo::Rect::new(0.0, 0.0, w, h);
        let th = &c.r.theme;
        o.push(kurbo::Affine::translate((x, y)));
        o.fill(area, th.panel());
        if let Some(g) = th.grid() {
            for (a, p) in self.axes.iter().zip(plans) {
                if p.shown && p.offset == 0.0 {
                    axis::grid(&**a, c, p.count, g, th.grid_width(), o);
                }
            }
        }
        o.push_clip(area);
        for m in &self.marks {
            m.draw(c, o);
        }
        o.pop();
        if let Some(b) = self.border {
            o.stroke(area, th.axis_width(), None, b);
        }
        for ((a, p), k) in self.axes.iter().zip(plans).zip(keep) {
            if p.shown {
                axis::draw(&**a, c, p.offset, p.labeled, p.count, k, o);
            }
        }
        for (at, l) in &self.legends {
            l.inset(c.r, *at, w, h, o);
        }
        if let Some(t) = &self.title {
            o.label(
                draw::Label::new(t.clone(), w / 2.0, -th.title_pad())
                    .anchor(draw::Anchor::Middle)
                    .size(th.title_text() as f32)
                    .fill(th.ink())
                    .bold(),
            );
        }
        o.pop();
    }
}
