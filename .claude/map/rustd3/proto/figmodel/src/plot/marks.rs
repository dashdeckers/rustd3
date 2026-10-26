//! Marks are values: typed data plus the handles of the
//! scales they use. Panels store them boxed.

use super::draw::{Anchor, Label, List, Pen};
use super::text::Text;
use super::{Ctx, Fits, GRAY, Paint, Position, ScaleRef};
use d3::color::Rgb;
use kurbo::{BezPath, Circle, Point, Rect};
use std::rc::Rc;

pub trait Mark: 'static {
    /// Reports data extents for scales that fit.
    fn fit(&self, f: &mut Fits);
    fn draw(&self, c: &Ctx, o: &mut List);
}

/// A fill resolved at draw time: the color, and the key's
/// slot among the palette's keys, for dodging.
type Fill = Rc<dyn Fn(&Ctx) -> (Rgb, Option<(usize, usize)>)>;

fn fill<C: Paint>(c: ScaleRef<C>, key: C::Domain) -> Fill {
    Rc::new(move |ctx| {
        let s = ctx.raw(c);
        (s.paint(&key).unwrap_or(GRAY), s.slot(&key))
    })
}

fn paint(f: &Option<Fill>, c: &Ctx) -> (Rgb, Option<(usize, usize)>) {
    f.as_ref().map_or((GRAY, None), |f| f(c))
}

/// Where a mark sits inside a band: the whole band, or
/// its key's sub-band when dodged.
fn slot(
    start: f64,
    bandwidth: f64,
    dodge: Option<(usize, usize)>,
) -> (f64, f64) {
    match dodge {
        Some((k, n)) => {
            let w = bandwidth / n as f64;
            (start + k as f64 * w, w)
        }
        None => (start, bandwidth),
    }
}

/// A deterministic jitter in [0, 1).
fn jitter(i: usize) -> f64 {
    let mut z = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z ^= z >> 29;
    z = z.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^= z >> 32;
    (z >> 11) as f64 / (1u64 << 53) as f64
}

pub struct Dots<X: Position, Y: Position> {
    data: Vec<(X::Domain, Y::Domain)>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    fill: Option<Fill>,
    jitter: f64,
    dodge: bool,
}

/// A scatter of `(x, y)` pairs.
pub fn dots<X: Position, Y: Position>(
    data: impl IntoIterator<Item = (X::Domain, Y::Domain)>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> Dots<X, Y> {
    Dots {
        data: data.into_iter().collect(),
        x,
        y,
        fill: None,
        jitter: 0.0,
        dodge: false,
    }
}

impl<X: Position, Y: Position> Dots<X, Y> {
    pub fn fill<C: Paint>(self, c: ScaleRef<C>, key: C::Domain) -> Self {
        Self { fill: Some(fill(c, key)), ..self }
    }

    /// Spreads dots across this fraction of their band.
    pub fn jitter(self, jitter: f64) -> Self {
        Self { jitter, ..self }
    }

    /// Moves dots into their fill key's sub-band.
    pub fn dodge(self) -> Self {
        Self { dodge: true, ..self }
    }
}

impl<X: Position, Y: Position> Mark for Dots<X, Y> {
    fn fit(&self, f: &mut Fits) {
        for (a, b) in &self.data {
            f.add(self.x, a);
            f.add(self.y, b);
        }
    }

    fn draw(&self, c: &Ctx, o: &mut List) {
        let (sx, sy) = (c.x(self.x), c.y(self.y));
        let (color, key) = paint(&self.fill, c);
        let key = key.filter(|_| self.dodge);
        let mut at = Vec::with_capacity(self.data.len());
        for (i, (a, b)) in self.data.iter().enumerate() {
            let (Some(px), Some(py)) = (sx.position(a), sy.position(b))
            else {
                continue;
            };
            let (start, w) = slot(px, sx.bandwidth(), key);
            let t = 0.5 + self.jitter * (jitter(i) - 0.5);
            at.push(Point::new(start + w * t, py));
        }
        let dot = Circle::new((0.0, 0.0), c.r.theme.dot());
        o.markers(dot, at, color.with_opacity(0.6));
    }
}

pub struct Histogram<X: Position, Y: Position<Domain = f64>> {
    counts: Vec<(X::Domain, f64)>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    fill: Option<Fill>,
    dodge: bool,
}

/// Bars counting how often each key of a band scale occurs.
pub fn histogram<X, Y>(
    keys: impl IntoIterator<Item = X::Domain>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> Histogram<X, Y>
where
    X: Position,
    X::Domain: PartialEq,
    Y: Position<Domain = f64>,
{
    let mut counts: Vec<(X::Domain, f64)> = Vec::new();
    for k in keys {
        match counts.iter_mut().find(|(c, _)| *c == k) {
            Some((_, n)) => *n += 1.0,
            None => counts.push((k, 1.0)),
        }
    }
    Histogram { counts, x, y, fill: None, dodge: false }
}

impl<X, Y> Histogram<X, Y>
where
    X: Position,
    Y: Position<Domain = f64>,
{
    pub fn fill<C: Paint>(self, c: ScaleRef<C>, key: C::Domain) -> Self {
        Self { fill: Some(fill(c, key)), ..self }
    }

    /// Moves bars into their fill key's sub-band.
    pub fn dodge(self) -> Self {
        Self { dodge: true, ..self }
    }
}

impl<X, Y> Mark for Histogram<X, Y>
where
    X: Position,
    Y: Position<Domain = f64>,
{
    fn fit(&self, f: &mut Fits) {
        f.add(self.y, &0.0);
        for (_, n) in &self.counts {
            f.add(self.y, n);
        }
    }

    fn draw(&self, c: &Ctx, o: &mut List) {
        let (sx, sy) = (c.x(self.x), c.y(self.y));
        let (color, key) = paint(&self.fill, c);
        let key = key.filter(|_| self.dodge);
        let base = sy.position(&0.0).unwrap_or(c.rect.h);
        for (k, n) in &self.counts {
            let (Some(px), Some(top)) = (sx.position(k), sy.position(n))
            else {
                continue;
            };
            let (x, w) = slot(px, sx.bandwidth(), key);
            o.fill(Rect::new(x, top, x + w, base), color);
        }
    }
}

pub struct DensityY<Y, W>
where
    Y: Position<Domain = f64>,
    W: Position<Domain = f64>,
{
    values: Vec<f64>,
    y: ScaleRef<Y>,
    w: ScaleRef<W>,
    fill: Option<Fill>,
}

/// A vertical kernel density estimate of `values` along
/// `y`, peak-normalized to 1 on the width scale `w`.
pub fn density_y<Y, W>(
    values: impl IntoIterator<Item = f64>,
    y: ScaleRef<Y>,
    w: ScaleRef<W>,
) -> DensityY<Y, W>
where
    Y: Position<Domain = f64>,
    W: Position<Domain = f64>,
{
    let values = values.into_iter().collect();
    DensityY { values, y, w, fill: None }
}

impl<Y, W> DensityY<Y, W>
where
    Y: Position<Domain = f64>,
    W: Position<Domain = f64>,
{
    pub fn fill<C: Paint>(self, c: ScaleRef<C>, key: C::Domain) -> Self {
        Self { fill: Some(fill(c, key)), ..self }
    }
}

impl<Y, W> Mark for DensityY<Y, W>
where
    Y: Position<Domain = f64>,
    W: Position<Domain = f64>,
{
    fn fit(&self, f: &mut Fits) {
        for v in &self.values {
            f.add(self.y, v);
        }
    }

    fn draw(&self, c: &Ctx, o: &mut List) {
        let (sy, sw) = (c.y(self.y), c.x(self.w));
        let (color, _) = paint(&self.fill, c);
        let n = self.values.len() as f64;
        if n < 2.0 {
            return;
        }
        let mean = self.values.iter().sum::<f64>() / n;
        let var = self
            .values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / n;
        // Scott's rule, as seaborn's default.
        let h = 1.06 * var.sqrt() * n.powf(-0.2);
        // Over the data, not the view, so zooming does
        // not reshape the curve.
        let (min, max) = self.values.iter().fold(
            (f64::INFINITY, f64::NEG_INFINITY),
            |(a, b), v| (a.min(*v), b.max(*v)),
        );
        let (lo, hi) = (min - 3.0 * h, max + 3.0 * h);
        let grid: Vec<f64> = (0..=96)
            .map(|i| lo + (hi - lo) * i as f64 / 96.0)
            .collect();
        let dens: Vec<f64> = grid
            .iter()
            .map(|g| {
                self.values
                    .iter()
                    .map(|v| (-0.5 * ((g - v) / h).powi(2)).exp())
                    .sum()
            })
            .collect();
        let peak = dens.iter().cloned().fold(0.0, f64::max);
        let p = |w: f64, v: f64| {
            let x = sw.position(&w).unwrap_or(0.0);
            let y = sy.position(&v).unwrap_or(0.0);
            Point::new(x, y)
        };
        let mut d = BezPath::new();
        d.move_to(p(0.0, lo));
        for (g, v) in grid.iter().zip(&dens) {
            d.line_to(p(v / peak, *g));
        }
        d.line_to(p(0.0, hi));
        d.close_path();
        o.fill(d.clone(), color.with_opacity(0.35));
        o.stroke(d, c.r.theme.pt(0.75), None, color);
    }
}

pub struct Line<X: Position, Y: Position> {
    data: Vec<(X::Domain, Y::Domain)>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    stroke: Option<Fill>,
    dash: Option<[f64; 2]>,
    dodge: bool,
}

/// A polyline through `(x, y)` pairs, in order.
pub fn line<X: Position, Y: Position>(
    data: impl IntoIterator<Item = (X::Domain, Y::Domain)>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> Line<X, Y> {
    Line {
        data: data.into_iter().collect(),
        x,
        y,
        stroke: None,
        dash: None,
        dodge: false,
    }
}

impl<X: Position, Y: Position> Line<X, Y> {
    pub fn stroke<C: Paint>(self, c: ScaleRef<C>, key: C::Domain) -> Self {
        Self { stroke: Some(fill(c, key)), ..self }
    }

    /// Dash and gap lengths in pixels.
    pub fn dash(self, dash: [f64; 2]) -> Self {
        Self { dash: Some(dash), ..self }
    }

    /// Runs through the stroke key's sub-band centers.
    pub fn dodge(self) -> Self {
        Self { dodge: true, ..self }
    }
}

impl<X: Position, Y: Position> Mark for Line<X, Y> {
    fn fit(&self, f: &mut Fits) {
        for (a, b) in &self.data {
            f.add(self.x, a);
            f.add(self.y, b);
        }
    }

    fn draw(&self, c: &Ctx, o: &mut List) {
        let (sx, sy) = (c.x(self.x), c.y(self.y));
        let (color, key) = paint(&self.stroke, c);
        let key = key.filter(|_| self.dodge);
        let pts = self.data.iter().filter_map(|(a, b)| {
            let (start, w) = slot(sx.position(a)?, sx.bandwidth(), key);
            Some((start + w / 2.0, sy.position(b)?))
        });
        let mut pen = Pen::default();
        d3::shape::line().draw(pts, &mut pen);
        o.stroke(pen.0, c.r.theme.line_width(), self.dash, color);
    }
}

pub struct RuleY<Y: Position> {
    value: Y::Domain,
    y: ScaleRef<Y>,
    color: Rgb,
    dash: Option<[f64; 2]>,
    label: Option<Text>,
}

/// A horizontal line across the panel at `value`.
pub fn rule_y<Y: Position>(
    value: Y::Domain,
    y: ScaleRef<Y>,
) -> RuleY<Y> {
    RuleY { value, y, color: GRAY, dash: None, label: None }
}

impl<Y: Position> RuleY<Y> {
    pub fn color(self, color: Rgb) -> Self {
        Self { color, ..self }
    }

    pub fn dash(self, dash: [f64; 2]) -> Self {
        Self { dash: Some(dash), ..self }
    }

    /// Text at the right end, above the line.
    pub fn label(self, t: impl Into<Text>) -> Self {
        Self { label: Some(t.into()), ..self }
    }
}

impl<Y: Position> Mark for RuleY<Y> {
    fn fit(&self, f: &mut Fits) {
        f.add(self.y, &self.value);
    }

    fn draw(&self, c: &Ctx, o: &mut List) {
        let sy = c.y(self.y);
        let Some(y) = sy.position(&self.value) else {
            return;
        };
        let (w, color) = (c.rect.w, self.color);
        let rule = kurbo::Line::new((0.0, y), (w, y));
        o.stroke(rule, c.r.theme.pt(1.25), self.dash, color);
        if let Some(t) = &self.label {
            o.label(
                Label::new(t.clone(), w - 4.0, y - 3.0)
                    .anchor(Anchor::End)
                    .size(c.r.theme.tick_text() as f32)
                    .fill(color),
            );
        }
    }
}
