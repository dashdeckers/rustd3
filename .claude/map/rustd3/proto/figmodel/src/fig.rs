//! Option A: the figure owns its scales; marks, axes and
//! gestures refer to them through typed, copyable handles.

use d3::axis::{self, Axis, Orient};
use d3::color::{Rgb, scheme};
use d3::path::svg;
use d3::scale::{self, Linear};
use d3::shape;
use d3::time::Time;
use jiff::Timestamp;
use std::collections::HashMap;
use std::fmt::Write;
use std::marker::PhantomData;

/// A domain value the figure can take extents of.
pub trait Value: Copy + 'static {
    fn to_f64(self) -> f64;
    fn from_f64(v: f64) -> Self;
}

impl Value for f64 {
    fn to_f64(self) -> f64 {
        self
    }
    fn from_f64(v: f64) -> Self {
        v
    }
}

impl Value for Timestamp {
    fn to_f64(self) -> f64 {
        self.as_millisecond() as f64
    }
    fn from_f64(v: f64) -> Self {
        Timestamp::from_millisecond(v.round() as i64)
            .expect("in range")
    }
}

/// The scales a figure can hold, erased. The real one wraps
/// `AnyContinuous` and adds band, ordinal and the rest.
#[derive(Debug, Clone, PartialEq)]
pub enum AnyScale {
    Linear(Linear),
    Color(Linear<Rgb>),
    Time(Time),
}

/// What a handle knows about its scale at compile time.
pub trait Kind: Into<AnyScale> {
    type Domain: Value;
    type Output;
}

impl Kind for Linear {
    type Domain = f64;
    type Output = f64;
}

impl Kind for Linear<Rgb> {
    type Domain = f64;
    type Output = Rgb;
}

impl Kind for Time {
    type Domain = Timestamp;
    type Output = f64;
}

impl From<Linear> for AnyScale {
    fn from(s: Linear) -> Self {
        AnyScale::Linear(s)
    }
}

impl From<Linear<Rgb>> for AnyScale {
    fn from(s: Linear<Rgb>) -> Self {
        AnyScale::Color(s)
    }
}

impl From<Time> for AnyScale {
    fn from(s: Time) -> Self {
        AnyScale::Time(s)
    }
}

impl AnyScale {
    pub fn domain(&self) -> [f64; 2] {
        match self {
            AnyScale::Linear(s) => s.get_domain(),
            AnyScale::Color(s) => s.get_domain(),
            AnyScale::Time(s) => s
                .get_domain()
                .map(Value::to_f64),
        }
    }

    pub fn with_domain(self, d: [f64; 2]) -> Self {
        match self {
            AnyScale::Linear(s) => {
                AnyScale::Linear(s.domain(d))
            }
            AnyScale::Color(s) => {
                AnyScale::Color(s.domain(d))
            }
            AnyScale::Time(s) => {
                let d = d.map(Timestamp::from_f64);
                AnyScale::Time(s.domain(d))
            }
        }
    }

    fn nice(self) -> Self {
        match self {
            AnyScale::Linear(s) => {
                AnyScale::Linear(s.nice(5))
            }
            other => other,
        }
    }

    fn with_range(self, r: [f64; 2]) -> Self {
        match self {
            AnyScale::Linear(s) => {
                AnyScale::Linear(s.range(r))
            }
            AnyScale::Time(s) => {
                AnyScale::Time(s.range(r))
            }
            color => color,
        }
    }

    /// Positions; handles with `Output = Rgb` never get here.
    pub fn apply(&self, v: f64) -> f64 {
        match self {
            AnyScale::Linear(s) => s.apply(v),
            AnyScale::Time(s) => {
                s.apply(Timestamp::from_f64(v))
            }
            AnyScale::Color(_) => unreachable!(),
        }
    }

    pub fn invert(&self, px: f64) -> f64 {
        match self {
            AnyScale::Linear(s) => s.invert(px),
            AnyScale::Time(s) => s.invert(px).to_f64(),
            AnyScale::Color(_) => unreachable!(),
        }
    }

    fn color(&self, v: f64) -> Rgb {
        match self {
            AnyScale::Color(s) => s.apply(v),
            _ => unreachable!(),
        }
    }

    fn axis(&self, orient: Orient) -> Axis {
        match self {
            AnyScale::Linear(s) => axis::axis(orient, s, 5),
            AnyScale::Time(s) => axis::axis(orient, s, 5),
            AnyScale::Color(_) => unreachable!(),
        }
    }
}

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

impl<S> std::fmt::Debug for ScaleRef<S> {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "ScaleRef({})", self.index)
    }
}

impl<S> ScaleRef<S> {
    pub fn index(self) -> usize {
        self.index
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Fit {
    Given,
    Data,
}

/// A figure: scales, and panels in grid cells. A value;
/// building it takes `&mut`, drawing it takes `&`.
#[derive(Debug, Clone, Default)]
pub struct Figure {
    pub(crate) scales: Vec<(AnyScale, Fit)>,
    pub(crate) panels: Vec<Panel>,
}

impl Figure {
    pub fn new() -> Self {
        Self::default()
    }

    /// A scale whose domain is the one it was built with.
    pub fn scale<S: Kind>(&mut self, s: S) -> ScaleRef<S> {
        self.add(s.into(), Fit::Given)
    }

    /// A scale whose domain spans the data of every mark
    /// that uses it, in any panel.
    pub fn fit<S: Kind>(&mut self, s: S) -> ScaleRef<S> {
        self.add(s.into(), Fit::Data)
    }

    pub(crate) fn add<S>(
        &mut self,
        s: AnyScale,
        fit: Fit,
    ) -> ScaleRef<S> {
        self.scales.push((s, fit));
        ScaleRef {
            index: self.scales.len() - 1,
            kind: PhantomData,
        }
    }

    /// The panel in grid cell `(row, col)`, made on first use.
    pub fn panel(
        &mut self,
        row: usize,
        col: usize,
    ) -> &mut Panel {
        let found = self
            .panels
            .iter()
            .position(|p| p.cell == (row, col));
        let i = found.unwrap_or_else(|| {
            self.panels.push(Panel::new((row, col)));
            self.panels.len() - 1
        });
        &mut self.panels[i]
    }

    /// A grid whose columns share x and rows share y,
    /// as matplotlib's `sharex="col", sharey="row"`.
    pub fn subplots(
        &mut self,
        rows: usize,
        cols: usize,
    ) -> Grid {
        let xs = (0..cols)
            .map(|_| self.fit(scale::linear()))
            .collect();
        let ys = (0..rows)
            .map(|_| self.fit(scale::linear()))
            .collect();
        Grid { xs, ys }
    }
}

/// Handles made by [`Figure::subplots`].
#[derive(Debug, Clone)]
pub struct Grid {
    pub xs: Vec<ScaleRef<Linear>>,
    pub ys: Vec<ScaleRef<Linear>>,
}

#[derive(Debug, Clone)]
pub(crate) enum Mark {
    Line {
        x: usize,
        y: usize,
        points: Vec<[f64; 2]>,
    },
    Dots {
        x: usize,
        y: usize,
        points: Vec<[f64; 2]>,
    },
    Heatmap {
        color: usize,
        rows: Vec<Vec<f64>>,
    },
    Colorbar {
        color: usize,
    },
}

/// What a drag or wheel in a panel does.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gesture {
    ZoomX { scale: usize },
    BrushX { over: usize, sets: usize },
}

#[derive(Debug, Clone)]
pub struct Panel {
    pub(crate) cell: (usize, usize),
    pub(crate) marks: Vec<Mark>,
    pub(crate) axes: Vec<(Orient, usize)>,
    pub(crate) gesture: Option<Gesture>,
}

fn points<X: Kind, Y: Kind>(
    data: impl IntoIterator<Item = (X::Domain, Y::Domain)>,
) -> Vec<[f64; 2]> {
    data.into_iter()
        .map(|(a, b)| [a.to_f64(), b.to_f64()])
        .collect()
}

impl Panel {
    fn new(cell: (usize, usize)) -> Self {
        Self {
            cell,
            marks: Vec::new(),
            axes: Vec::new(),
            gesture: None,
        }
    }

    pub fn line<X, Y>(
        &mut self,
        data: impl IntoIterator<
            Item = (X::Domain, Y::Domain),
        >,
        x: ScaleRef<X>,
        y: ScaleRef<Y>,
    ) -> &mut Self
    where
        X: Kind<Output = f64>,
        Y: Kind<Output = f64>,
    {
        self.marks.push(Mark::Line {
            x: x.index,
            y: y.index,
            points: points::<X, Y>(data),
        });
        self
    }

    pub fn dots<X, Y>(
        &mut self,
        data: impl IntoIterator<
            Item = (X::Domain, Y::Domain),
        >,
        x: ScaleRef<X>,
        y: ScaleRef<Y>,
    ) -> &mut Self
    where
        X: Kind<Output = f64>,
        Y: Kind<Output = f64>,
    {
        self.marks.push(Mark::Dots {
            x: x.index,
            y: y.index,
            points: points::<X, Y>(data),
        });
        self
    }

    pub fn heatmap<C>(
        &mut self,
        rows: Vec<Vec<f64>>,
        color: ScaleRef<C>,
    ) -> &mut Self
    where
        C: Kind<Domain = f64, Output = Rgb>,
    {
        let color = color.index;
        self.marks.push(Mark::Heatmap { color, rows });
        self
    }

    pub fn colorbar<C>(
        &mut self,
        color: ScaleRef<C>,
    ) -> &mut Self
    where
        C: Kind<Domain = f64, Output = Rgb>,
    {
        let color = color.index;
        self.marks.push(Mark::Colorbar { color });
        self
    }

    pub fn axis<S: Kind<Output = f64>>(
        &mut self,
        orient: Orient,
        s: ScaleRef<S>,
    ) -> &mut Self {
        self.axes.push((orient, s.index));
        self
    }

    /// Wheel and drag rescale `x` horizontally.
    pub fn zoom_x<S: Kind<Output = f64>>(
        &mut self,
        x: ScaleRef<S>,
    ) -> &mut Self {
        let scale = x.index;
        self.gesture = Some(Gesture::ZoomX { scale });
        self
    }

    /// A brush along `over` whose selection is the domain
    /// of `sets`: d3's focus and context.
    pub fn brush_x<A, B>(
        &mut self,
        over: ScaleRef<A>,
        sets: ScaleRef<B>,
    ) -> &mut Self
    where
        A: Kind<Output = f64>,
        B: Kind<Domain = A::Domain, Output = f64>,
    {
        self.gesture = Some(Gesture::BrushX {
            over: over.index,
            sets: sets.index,
        });
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn contains(&self, (x, y): (f64, f64)) -> bool {
        (self.x..=self.x + self.w).contains(&x)
            && (self.y..=self.y + self.h).contains(&y)
    }
}

/// The figure after the resolve pass: final domains, and
/// where each panel's plot area is.
#[derive(Debug, Clone)]
pub struct Resolved {
    pub size: [f64; 2],
    pub scales: Vec<AnyScale>,
    pub rects: Vec<Rect>,
}

impl Figure {
    /// Fits domains to data, applies view overrides, and
    /// places panels. `views` is keyed by scale index.
    pub fn resolve(
        &self,
        size: [f64; 2],
        views: &HashMap<usize, [f64; 2]>,
    ) -> Resolved {
        let mut ext: Vec<Option<[f64; 2]>> =
            vec![None; self.scales.len()];
        let mut grow = |i: usize, v: f64| {
            let e = ext[i].get_or_insert([v, v]);
            *e = [e[0].min(v), e[1].max(v)];
        };
        for m in self.panels.iter().flat_map(|p| &p.marks) {
            match m {
                Mark::Line { x, y, points }
                | Mark::Dots { x, y, points } => {
                    for [a, b] in points {
                        grow(*x, *a);
                        grow(*y, *b);
                    }
                }
                Mark::Heatmap { color, rows } => {
                    for v in rows.iter().flatten() {
                        grow(*color, *v);
                    }
                }
                Mark::Colorbar { .. } => {}
            }
        }
        let scales = self
            .scales
            .iter()
            .zip(ext)
            .enumerate()
            .map(|(i, ((s, fit), ext))| {
                let s = match (fit, ext) {
                    (Fit::Data, Some(d)) => {
                        s.clone().with_domain(d).nice()
                    }
                    _ => s.clone(),
                };
                match views.get(&i) {
                    Some(&d) => s.with_domain(d),
                    None => s,
                }
            })
            .collect();
        Resolved {
            size,
            scales,
            rects: self.grid(size),
        }
    }

    /// A uniform grid with fixed margins; problem #3
    /// (layout) replaces this.
    fn grid(&self, [w, h]: [f64; 2]) -> Vec<Rect> {
        let rows = self
            .panels
            .iter()
            .map(|p| p.cell.0 + 1)
            .max()
            .unwrap_or(1);
        let cols = self
            .panels
            .iter()
            .map(|p| p.cell.1 + 1)
            .max()
            .unwrap_or(1);
        let (cw, ch) = (w / cols as f64, h / rows as f64);
        self.panels
            .iter()
            .map(|p| {
                let right = p
                    .axes
                    .iter()
                    .any(|(o, _)| *o == Orient::Right);
                let right = if right { 46.0 } else { 14.0 };
                let (r, c) = p.cell;
                Rect {
                    x: c as f64 * cw + 46.0,
                    y: r as f64 * ch + 12.0,
                    w: cw - 46.0 - right,
                    h: ch - 12.0 - 28.0,
                }
            })
            .collect()
    }
}

impl Resolved {
    /// Scale `i` as panel `p` uses it, along x or y.
    pub fn along(
        &self,
        p: usize,
        i: usize,
        horizontal: bool,
    ) -> AnyScale {
        let r = self.rects[p];
        let range = if horizontal {
            [0.0, r.w]
        } else {
            [r.h, 0.0]
        };
        self.scales[i].clone().with_range(range)
    }

    /// The figure as SVG; text is plain `<text>` here.
    pub fn svg(&self, fig: &Figure) -> String {
        let [w, h] = self.size;
        let mut o = format!(
            "<svg xmlns='http://www.w3.org/2000/svg' \
             width='{w}' height='{h}' \
             font-family='sans-serif' font-size='10'>\
             <rect width='100%' height='100%' fill='white'/>"
        );
        for (p, panel) in fig.panels.iter().enumerate() {
            let r = self.rects[p];
            let _ = write!(
                o,
                "<clipPath id='c{p}'><rect width='{}' \
                 height='{}'/></clipPath>\
                 <g transform='translate({},{})'>\
                 <g clip-path='url(#c{p})'>",
                r.w, r.h, r.x, r.y
            );
            for (k, m) in panel.marks.iter().enumerate() {
                self.mark(&mut o, p, k, m);
            }
            o += "</g>";
            if let Some(Gesture::BrushX { over, sets }) =
                panel.gesture
            {
                let s = self.along(p, over, true);
                let [a, b] = self.scales[sets]
                    .domain()
                    .map(|v| s.apply(v));
                let _ = write!(
                    o,
                    "<rect x='{a}' width='{}' height='{}' \
                     fill='steelblue' fill-opacity='0.2'/>",
                    b - a,
                    r.h
                );
            }
            for &(orient, i) in &panel.axes {
                let horizontal = matches!(
                    orient,
                    Orient::Bottom | Orient::Top
                );
                let a = self.along(p, i, horizontal).axis(orient);
                let at = match orient {
                    Orient::Bottom => (0.0, r.h),
                    Orient::Right => (r.w, 0.0),
                    _ => (0.0, 0.0),
                };
                draw_axis(&mut o, &a, at);
            }
            o += "</g>";
        }
        o + "</svg>"
    }

    fn mark(&self, o: &mut String, p: usize, k: usize, m: &Mark) {
        let r = self.rects[p];
        let c = scheme::CATEGORY10[k % 10];
        match m {
            Mark::Line { x, y, points } => {
                let sx = self.along(p, *x, true);
                let sy = self.along(p, *y, false);
                let d = svg(|ctx| {
                    let pts = points.iter().map(|[a, b]| {
                        (sx.apply(*a), sy.apply(*b))
                    });
                    shape::line().draw(pts, ctx)
                });
                let _ = write!(
                    o,
                    "<path d='{d}' fill='none' \
                     stroke='{c}' stroke-width='1.5'/>"
                );
            }
            Mark::Dots { x, y, points } => {
                let sx = self.along(p, *x, true);
                let sy = self.along(p, *y, false);
                for [a, b] in points {
                    let _ = write!(
                        o,
                        "<circle cx='{:.1}' cy='{:.1}' r='2' \
                         fill='{c}'/>",
                        sx.apply(*a),
                        sy.apply(*b)
                    );
                }
            }
            Mark::Heatmap { color, rows } => {
                let s = &self.scales[*color];
                let (dy, n) = (r.h / rows.len() as f64, rows.len());
                for (i, row) in rows.iter().enumerate() {
                    let dx = r.w / row.len() as f64;
                    for (j, v) in row.iter().enumerate() {
                        let _ = write!(
                            o,
                            "<rect x='{}' y='{}' width='{dx}' \
                             height='{dy}' fill='{}'/>",
                            j as f64 * dx,
                            (n - 1 - i) as f64 * dy,
                            s.color(*v)
                        );
                    }
                }
            }
            Mark::Colorbar { color } => {
                let s = &self.scales[*color];
                let pos = scale::linear()
                    .domain(s.domain())
                    .range([r.h, 0.0]);
                for i in 0..32 {
                    let y0 = r.h * (1.0 - (i + 1) as f64 / 32.0);
                    let v = pos.invert(y0 + r.h / 64.0);
                    let _ = write!(
                        o,
                        "<rect y='{y0}' width='14' height='{}' \
                         fill='{}'/>",
                        r.h / 32.0 + 0.5,
                        s.color(v)
                    );
                }
                let a = axis::axis(Orient::Right, &pos, 5);
                draw_axis(o, &a, (14.0, 0.0));
            }
        }
    }
}

fn draw_axis(o: &mut String, a: &Axis, (x, y): (f64, f64)) {
    let d = svg(|ctx| a.draw(ctx));
    let _ = write!(
        o,
        "<g transform='translate({x},{y})'><path d='{d}' \
         fill='none' stroke='#555'/>"
    );
    for t in &a.ticks {
        let (tx, ty, anchor) = match a.orient {
            Orient::Bottom => (t.position, 18.0, "middle"),
            Orient::Top => (t.position, -9.0, "middle"),
            Orient::Left => (-9.0, t.position + 3.5, "end"),
            Orient::Right => (9.0, t.position + 3.5, "start"),
        };
        let _ = write!(
            o,
            "<text x='{tx:.1}' y='{ty:.1}' \
             text-anchor='{anchor}'>{}</text>",
            t.label
        );
    }
    o.push_str("</g>");
}
