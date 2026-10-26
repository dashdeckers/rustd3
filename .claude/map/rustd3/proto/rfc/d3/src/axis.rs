//! Axis layout: tick positions and labels, computed in core; drawing text is the renderer's job.

use crate::path::PathContext;
use crate::scale::TickSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orient {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tick {
    pub position: f64,
    pub label: String,
}

/// What `d3.axisBottom(scale)` would draw, as data.
#[derive(Debug, Clone, PartialEq)]
pub struct Axis {
    pub orient: Orient,
    pub ticks: Vec<Tick>,
    pub range: [f64; 2],
    pub tick_size: f64,
}

pub fn axis<S: TickSource>(orient: Orient, scale: &S, count: usize) -> Axis {
    let ticks = scale
        .tick_values(count)
        .iter()
        .map(|v| Tick { position: scale.position(v), label: scale.label(v, count) })
        .collect();
    Axis { orient, ticks, range: scale.extent(), tick_size: 6.0 }
}

impl Axis {
    /// The domain line and tick marks, relative to the axis origin.
    pub fn draw(&self, ctx: &mut impl PathContext) {
        let k = if matches!(self.orient, Orient::Top | Orient::Left) { -1.0 } else { 1.0 };
        let vertical = matches!(self.orient, Orient::Left | Orient::Right);
        let pt = |along: f64, across: f64| if vertical { (across, along) } else { (along, across) };
        let [r0, r1] = self.range;
        let segs = [pt(r0, k * self.tick_size), pt(r0, 0.0), pt(r1, 0.0), pt(r1, k * self.tick_size)];
        ctx.move_to(segs[0].0, segs[0].1);
        segs[1..].iter().for_each(|&(x, y)| ctx.line_to(x, y));
        for t in &self.ticks {
            let (a, b) = (pt(t.position, 0.0), pt(t.position, k * self.tick_size));
            ctx.move_to(a.0, a.1);
            ctx.line_to(b.0, b.1);
        }
    }
}
