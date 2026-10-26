//! Shape generators. They take geometry (already scaled points) and write into a [`PathContext`].

use crate::path::PathContext;
use std::f64::consts::{FRAC_PI_2, TAU};

/// How consecutive points are joined: d3's curve factories, as a closed set.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Curve {
    #[default]
    Linear,
    /// `curveStep`: the step happens halfway between points.
    Step,
    /// `curveMonotoneX`: smooth, and never overshoots in y.
    MonotoneX,
    /// `curveBasis`: a B-spline that does not pass through inner points.
    Basis,
}

/// Draws one unbroken run of points. `join` continues the current subpath instead of moving.
fn segment(curve: Curve, pts: &[(f64, f64)], ctx: &mut impl PathContext, join: bool) {
    let Some(&(x, y)) = pts.first() else { return };
    if join { ctx.line_to(x, y) } else { ctx.move_to(x, y) }
    match curve {
        Curve::Linear => pts[1..].iter().for_each(|&(x, y)| ctx.line_to(x, y)),
        Curve::Step => {
            for w in pts.windows(2) {
                let ((x0, y0), (x1, y1)) = (w[0], w[1]);
                let xm = (x0 + x1) / 2.0;
                ctx.line_to(xm, y0);
                ctx.line_to(xm, y1);
            }
            if let [.., (x, y)] = pts {
                if pts.len() > 1 { ctx.line_to(*x, *y) }
            }
        }
        Curve::Basis => basis(pts, ctx),
        Curve::MonotoneX => monotone_x(pts, ctx),
    }
}

fn basis(p: &[(f64, f64)], ctx: &mut impl PathContext) {
    fn bez(ctx: &mut impl PathContext, a: (f64, f64), b: (f64, f64), c: (f64, f64)) {
        ctx.bezier_curve_to(
            (2.0 * a.0 + b.0) / 3.0, (2.0 * a.1 + b.1) / 3.0,
            (a.0 + 2.0 * b.0) / 3.0, (a.1 + 2.0 * b.1) / 3.0,
            (a.0 + 4.0 * b.0 + c.0) / 6.0, (a.1 + 4.0 * b.1 + c.1) / 6.0,
        );
    }
    match p.len() {
        0 | 1 => {}
        2 => ctx.line_to(p[1].0, p[1].1),
        n => {
            ctx.line_to((5.0 * p[0].0 + p[1].0) / 6.0, (5.0 * p[0].1 + p[1].1) / 6.0);
            for i in 2..n {
                bez(ctx, p[i - 2], p[i - 1], p[i]);
            }
            bez(ctx, p[n - 2], p[n - 1], p[n - 1]);
            ctx.line_to(p[n - 1].0, p[n - 1].1);
        }
    }
}

fn monotone_x(p: &[(f64, f64)], ctx: &mut impl PathContext) {
    let sign = |x: f64| if x < 0.0 { -1.0 } else { 1.0 };
    let slope3 = |(x0, y0): (f64, f64), (x1, y1): (f64, f64), (x2, y2): (f64, f64)| {
        let (h0, h1) = (x1 - x0, x2 - x1);
        let s0 = (y1 - y0) / if h0 != 0.0 { h0 } else if h1 < 0.0 { -0.0 } else { 0.0 };
        let s1 = (y2 - y1) / if h1 != 0.0 { h1 } else if h0 < 0.0 { -0.0 } else { 0.0 };
        let p = (s0 * h1 + s1 * h0) / (h0 + h1);
        let t = (sign(s0) + sign(s1)) * s0.abs().min(s1.abs()).min(0.5 * p.abs());
        if t.is_nan() { 0.0 } else { t }
    };
    let slope2 = |(x0, y0): (f64, f64), (x1, y1): (f64, f64), t: f64| {
        let h = x1 - x0;
        if h != 0.0 { (3.0 * (y1 - y0) / h - t) / 2.0 } else { t }
    };
    fn seg(ctx: &mut impl PathContext, a: (f64, f64), b: (f64, f64), t0: f64, t1: f64) {
        let dx = (b.0 - a.0) / 3.0;
        ctx.bezier_curve_to(a.0 + dx, a.1 + dx * t0, b.0 - dx, b.1 - dx * t1, b.0, b.1);
    }
    let mut pts: Vec<(f64, f64)> = p.to_vec();
    pts.dedup();
    match pts.len() {
        0 | 1 => {}
        2 => ctx.line_to(pts[1].0, pts[1].1),
        n => {
            let mut t0 = f64::NAN;
            for i in 2..n {
                let t1 = slope3(pts[i - 2], pts[i - 1], pts[i]);
                let from = if i == 2 { slope2(pts[0], pts[1], t1) } else { t0 };
                seg(ctx, pts[i - 2], pts[i - 1], from, t1);
                t0 = t1;
            }
            seg(ctx, pts[n - 2], pts[n - 1], t0, slope2(pts[n - 2], pts[n - 1], t0));
        }
    }
}

/// A point that may be missing, which breaks the line: d3's `line.defined`.
pub trait MaybePoint {
    fn point(self) -> Option<(f64, f64)>;
}
impl MaybePoint for (f64, f64) {
    fn point(self) -> Option<(f64, f64)> {
        Some(self)
    }
}
impl MaybePoint for Option<(f64, f64)> {
    fn point(self) -> Option<(f64, f64)> {
        self
    }
}

/// `d3.line()` over geometry: map your data to points with iterators, then draw.
#[derive(Debug, Clone, Copy, Default)]
pub struct Line {
    curve: Curve,
}

pub fn line() -> Line {
    Line::default()
}

impl Line {
    pub fn curve(self, curve: Curve) -> Self {
        Self { curve }
    }

    /// Draws the points; `None` items split the line into separate runs.
    pub fn draw<P: MaybePoint>(&self, points: impl IntoIterator<Item = P>, ctx: &mut impl PathContext) {
        let mut run = Vec::new();
        for p in points {
            match p.point() {
                Some(p) => run.push(p),
                None => {
                    segment(self.curve, &run, ctx, false);
                    run.clear();
                }
            }
        }
        segment(self.curve, &run, ctx, false);
    }
}

/// `d3.area()`: each item is `(x, y0, y1)`; `None` breaks the area.
#[derive(Debug, Clone, Copy, Default)]
pub struct Area {
    curve: Curve,
}

pub fn area() -> Area {
    Area::default()
}

impl Area {
    pub fn curve(self, curve: Curve) -> Self {
        Self { curve }
    }

    pub fn draw(&self, items: impl IntoIterator<Item = Option<(f64, f64, f64)>>, ctx: &mut impl PathContext) {
        let curve = self.curve;
        fn flush(curve: Curve, run: &mut Vec<(f64, f64, f64)>, ctx: &mut impl PathContext) {
            if run.is_empty() {
                return;
            }
            let top: Vec<_> = run.iter().map(|&(x, _, y1)| (x, y1)).collect();
            let base: Vec<_> = run.iter().rev().map(|&(x, y0, _)| (x, y0)).collect();
            segment(curve, &top, ctx, false);
            segment(curve, &base, ctx, true);
            ctx.close_path();
            run.clear();
        }
        let mut run: Vec<(f64, f64, f64)> = Vec::new();
        for item in items {
            match item {
                Some(v) => run.push(v),
                None => flush(curve, &mut run, ctx),
            }
        }
        flush(curve, &mut run, ctx);
    }
}

/// The accessor-style alternative: closures that borrow scales for the lifetime `'a`.
pub struct LineOf<'a, D> {
    x: Box<dyn Fn(&D) -> f64 + 'a>,
    y: Box<dyn Fn(&D) -> f64 + 'a>,
    defined: Box<dyn Fn(&D) -> bool + 'a>,
    curve: Curve,
}

pub fn line_of<'a, D>() -> LineOf<'a, D> {
    LineOf { x: Box::new(|_| 0.0), y: Box::new(|_| 0.0), defined: Box::new(|_| true), curve: Curve::Linear }
}

impl<'a, D> LineOf<'a, D> {
    pub fn x(self, f: impl Fn(&D) -> f64 + 'a) -> Self {
        Self { x: Box::new(f), ..self }
    }
    pub fn y(self, f: impl Fn(&D) -> f64 + 'a) -> Self {
        Self { y: Box::new(f), ..self }
    }
    pub fn defined(self, f: impl Fn(&D) -> bool + 'a) -> Self {
        Self { defined: Box::new(f), ..self }
    }
    pub fn curve(self, curve: Curve) -> Self {
        Self { curve, ..self }
    }
    pub fn draw(&self, data: &[D], ctx: &mut impl PathContext) {
        line().curve(self.curve).draw(data.iter().map(|d| (self.defined)(d).then(|| ((self.x)(d), (self.y)(d)))), ctx);
    }
}

/// One wedge of a pie: d3's arc datum, holding an index rather than a reference to the datum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slice {
    pub index: usize,
    pub value: f64,
    pub start_angle: f64,
    pub end_angle: f64,
}

/// `d3.pie()`: turns values into angles, largest first by default.
#[derive(Debug, Clone, Copy)]
pub struct Pie {
    sort_descending: bool,
    pad_angle: f64,
}

pub fn pie() -> Pie {
    Pie { sort_descending: true, pad_angle: 0.0 }
}

impl Pie {
    pub fn sorted(self, sort_descending: bool) -> Self {
        Self { sort_descending, ..self }
    }

    pub fn pad_angle(self, pad_angle: f64) -> Self {
        Self { pad_angle, ..self }
    }

    pub fn layout(&self, values: impl IntoIterator<Item = f64>) -> Vec<Slice> {
        let values: Vec<f64> = values.into_iter().collect();
        let sum: f64 = values.iter().filter(|v| **v > 0.0).sum();
        let mut order: Vec<usize> = (0..values.len()).collect();
        if self.sort_descending {
            order.sort_by(|&a, &b| values[b].total_cmp(&values[a]));
        }
        let n = values.len() as f64;
        let k = if sum > 0.0 { (TAU - self.pad_angle * n) / sum } else { 0.0 };
        let mut angle = 0.0;
        let mut slices = vec![Slice { index: 0, value: 0.0, start_angle: 0.0, end_angle: 0.0 }; values.len()];
        for i in order {
            let v = values[i];
            let end = angle + v.max(0.0) * k + self.pad_angle;
            slices[i] = Slice { index: i, value: v, start_angle: angle, end_angle: end };
            angle = end;
        }
        slices
    }
}

/// `d3.arc()` for annular sectors; angles are clockwise from 12 o'clock, as in d3.
#[derive(Debug, Clone, Copy, Default)]
pub struct Arc {
    inner: f64,
    outer: f64,
}

pub fn arc() -> Arc {
    Arc::default()
}

impl Arc {
    pub fn inner_radius(self, inner: f64) -> Self {
        Self { inner, ..self }
    }
    pub fn outer_radius(self, outer: f64) -> Self {
        Self { outer, ..self }
    }

    pub fn draw(&self, s: &Slice, ctx: &mut impl PathContext) {
        let (a0, a1) = (s.start_angle - FRAC_PI_2, s.end_angle - FRAC_PI_2);
        ctx.arc(0.0, 0.0, self.outer, a0, a1, false);
        if self.inner > 0.0 {
            ctx.arc(0.0, 0.0, self.inner, a1, a0, true);
        } else {
            ctx.line_to(0.0, 0.0);
        }
        ctx.close_path();
    }

    /// The midpoint of the slice, for labels.
    pub fn centroid(&self, s: &Slice) -> (f64, f64) {
        let r = (self.inner + self.outer) / 2.0;
        let a = (s.start_angle + s.end_angle) / 2.0 - FRAC_PI_2;
        (r * a.cos(), r * a.sin())
    }
}

/// `d3.stack()` over columns (one slice per series), returning `[y0, y1]` per point.
pub fn stack(series: &[&[f64]]) -> Vec<Vec<[f64; 2]>> {
    let n = series.first().map_or(0, |s| s.len());
    let mut base = vec![0.0; n];
    series
        .iter()
        .map(|s| {
            s.iter()
                .zip(base.iter_mut())
                .map(|(v, b)| {
                    let y0 = *b;
                    *b += v;
                    [y0, *b]
                })
                .collect()
        })
        .collect()
}
