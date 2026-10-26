//! Layout by protrusions: each panel reports how far its
//! decorations reach past its plot area, each grid track
//! takes the largest reach of its panels, and plot areas
//! share what is left by weight. Then one greedy pass
//! drops tick labels that would overlap.

use super::axis::{self, Draw};
use super::{Ctx, Figure, Panel, Rect, Resolved};
use d3::axis::Orient;

/// Gap between two axes stacked on one side.
/// Height a panel title takes above its plot area.

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct AxisPlan {
    /// Drawn at all: an inner, stacked axis is dropped.
    pub shown: bool,
    pub labeled: bool,
    pub offset: f64,
    pub thickness: f64,
    /// Ticks asked of the scale.
    pub count: usize,
}

/// Top, right, bottom, left.
pub type Sides = [f64; 4];

fn side(o: Orient) -> usize {
    match o {
        Orient::Top => 0,
        Orient::Right => 1,
        Orient::Bottom => 2,
        Orient::Left => 3,
    }
}

/// Whether another panel further out on the same row (or
/// column) shows the same scale on the same side: then
/// this axis is inner, and only the outer one is labeled.
fn inner(fig: &Figure, p: &Panel, a: &dyn Draw) -> bool {
    let (r, c) = p.cell;
    fig.panels.iter().any(|q| {
        let (qr, qc) = q.cell;
        let beyond = match a.orient() {
            Orient::Left => qr == r && qc < c,
            Orient::Right => qr == r && qc > c,
            Orient::Top => qc == c && qr < r,
            Orient::Bottom => qc == c && qr > r,
        };
        beyond
            && q.axes.iter().any(|b| {
                b.orient() == a.orient() && b.scale() == a.scale()
            })
    })
}

/// Labels, drops and stacks every axis. Tick counts
/// follow the axis lengths in `rects`.
pub(crate) fn plan(
    fig: &Figure,
    r: &Resolved,
    rects: &[Rect],
) -> Vec<Vec<AxisPlan>> {
    fig.panels
        .iter()
        .zip(rects)
        .map(|(p, rect)| {
            let mut reach = [0.0; 4];
            p.axes
                .iter()
                .map(|a| {
                    let s = side(a.orient());
                    let stacked = reach[s] > 0.0;
                    let inner = inner(fig, p, &**a);
                    let labeled = a.labels().unwrap_or(!inner);
                    let explicit = a.labels().is_some();
                    let outward = a.outward().is_some() || stacked;
                    let shown = explicit || !(inner && outward);
                    let len = match a.orient() {
                        Orient::Top | Orient::Bottom => rect.w,
                        _ => rect.h,
                    };
                    let count = a
                        .count()
                        .unwrap_or_else(|| axis::auto_count(a.orient(), len));
                    let thickness =
                        axis::thickness(&**a, r, labeled, count);
                    let offset = a.outward().unwrap_or(if stacked {
                        reach[s] + 0.8 * r.theme.em()
                    } else {
                        0.0
                    });
                    if shown {
                        reach[s] = offset + thickness;
                    }
                    AxisPlan { shown, labeled, offset, thickness, count }
                })
                .collect()
        })
        .collect()
}

/// How far each panel's decorations reach past its plot
/// area, per side.
pub(crate) fn protrusions(
    fig: &Figure,
    plans: &[Vec<AxisPlan>],
) -> Vec<Sides> {
    fig.panels
        .iter()
        .zip(plans)
        .map(|(p, plan)| {
            let mut s = [0.0; 4];
            for (a, ap) in p.axes.iter().zip(plan) {
                if ap.shown {
                    let i = side(a.orient());
                    s[i] = f64::max(s[i], ap.offset + ap.thickness);
                }
            }
            if p.title.is_some() {
                let t = &fig.theme;
                s[0] += 1.2 * t.title_text() + t.title_pad();
            }
            s
        })
        .collect()
}

/// Greedy collision removal over every tick label of the
/// figure: labels of larger panels win, then earlier
/// axes, then earlier labels, as Vega's greedy overlap
/// strategy. Returns keep flags per panel, axis, label.
pub(crate) fn collide(
    fig: &Figure,
    r: &Resolved,
    plans: &[Vec<AxisPlan>],
) -> Vec<Vec<Vec<bool>>> {
    let mut cands = Vec::new();
    let mut keep: Vec<Vec<Vec<bool>>> = Vec::new();
    for (p, (panel, plan)) in fig.panels.iter().zip(plans).enumerate() {
        let c = Ctx { r, rect: r.rects[p] };
        let area = c.rect.w * c.rect.h;
        let mut per_axis = Vec::new();
        for (i, (a, ap)) in panel.axes.iter().zip(plan).enumerate() {
            if !(ap.shown && ap.labeled) {
                per_axis.push(vec![]);
                continue;
            }
            let ls = axis::labels(&**a, &c, ap.offset, ap.count);
            per_axis.push(vec![false; ls.len()]);
            // Within one axis, keep every k-th label for the
            // smallest k without overlap (Vega's parity), so
            // spacing stays regular; across axes, greedy.
            let boxes: Vec<[f64; 4]> = ls.iter().map(|l| l.bbox).collect();
            let k = stride(&boxes);
            for (j, b) in boxes.into_iter().enumerate() {
                if j % k == 0 {
                    cands.push((area, p, i, j, b));
                }
            }
        }
        keep.push(per_axis);
    }
    cands.sort_by(|a, b| {
        b.0.total_cmp(&a.0)
            .then(a.1.cmp(&b.1))
            .then(a.2.cmp(&b.2))
            .then(a.3.cmp(&b.3))
    });
    let mut placed: Vec<[f64; 4]> = Vec::new();
    for (_, p, i, j, bbox) in cands {
        if placed.iter().all(|q| !hit(q, &bbox)) {
            placed.push(bbox);
            keep[p][i][j] = true;
        }
    }
    keep
}

fn hit(a: &[f64; 4], b: &[f64; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

/// The smallest stride whose labels do not overlap.
fn stride(boxes: &[[f64; 4]]) -> usize {
    (1..=boxes.len().max(1))
        .find(|&k| {
            let kept: Vec<_> = boxes.iter().step_by(k).collect();
            kept.windows(2).all(|w| !hit(w[0], w[1]))
        })
        .unwrap_or(1)
}

/// Plot areas from track weights, gaps between the
/// decorated boxes, and the largest protrusion per track.
pub(crate) fn rects(
    weights: [&[f64]; 2],
    gaps: [&[f64]; 2],
    [w, h]: [f64; 2],
    pad: f64,
    reserved: [f64; 2],
    cells: &[(usize, usize)],
    prot: &[Sides],
) -> Vec<Rect> {
    let rows = cells.iter().map(|c| c.0 + 1).max().unwrap_or(1);
    let cols = cells.iter().map(|c| c.1 + 1).max().unwrap_or(1);
    let (mut lo_c, mut hi_c) = (vec![0.0; cols], vec![0.0; cols]);
    let (mut lo_r, mut hi_r) = (vec![0.0; rows], vec![0.0; rows]);
    for (&(r, c), s) in cells.iter().zip(prot) {
        lo_c[c] = f64::max(lo_c[c], s[3]);
        hi_c[c] = f64::max(hi_c[c], s[1]);
        lo_r[r] = f64::max(lo_r[r], s[0]);
        hi_r[r] = f64::max(hi_r[r], s[2]);
    }
    let xs = tracks(weights[1], gaps[1], &lo_c, &hi_c, pad, w - 2.0 * pad);
    let ys = tracks(
        weights[0],
        gaps[0],
        &lo_r,
        &hi_r,
        pad + reserved[0],
        h - 2.0 * pad - reserved[0] - reserved[1],
    );
    cells
        .iter()
        .map(|&(r, c)| Rect {
            x: xs[c].0,
            y: ys[r].0,
            w: xs[c].1,
            h: ys[r].1,
        })
        .collect()
}

fn tracks(
    weights: &[f64],
    gaps: &[f64],
    lo: &[f64],
    hi: &[f64],
    start: f64,
    total: f64,
) -> Vec<(f64, f64)> {
    let n = lo.len();
    let weight = |i: usize| weights.get(i).copied().unwrap_or(1.0);
    let gap = |i: usize| gaps.get(i).copied().unwrap_or(12.0);
    let fixed: f64 = (0..n).map(|i| lo[i] + hi[i]).sum::<f64>()
        + (0..n.saturating_sub(1)).map(gap).sum::<f64>();
    let sum: f64 = (0..n).map(weight).sum();
    let free = (total - fixed).max(0.0);
    let mut at = start;
    (0..n)
        .map(|i| {
            at += lo[i];
            let len = free * weight(i) / sum;
            let span = (at, len);
            at += len + hi[i] + if i + 1 < n { gap(i) } else { 0.0 };
            span
        })
        .collect()
}

/// Each panel's measured reach per side, for tests and
/// for a widget that wants to know how far labels grew.
pub fn protrusion_of(fig: &Figure, r: &Resolved) -> Vec<Sides> {
    protrusions(fig, &r.plans)
}

/// Every tick label the collision pass saw: panel, axis,
/// text, figure-space box, kept. For explaining layout.
pub fn label_trace(
    fig: &Figure,
    r: &Resolved,
) -> Vec<(usize, usize, String, [f64; 4], bool)> {
    let mut out = Vec::new();
    for (p, (panel, plan)) in fig.panels.iter().zip(&r.plans).enumerate() {
        let c = Ctx { r, rect: r.rects[p] };
        for (i, (a, ap)) in panel.axes.iter().zip(plan).enumerate() {
            if !(ap.shown && ap.labeled) {
                continue;
            }
            for (j, l) in axis::labels(&**a, &c, ap.offset, ap.count).into_iter().enumerate() {
                out.push((p, i, l.text, l.bbox, r.keep[p][i][j]));
            }
        }
    }
    out
}

/// Axis plans with orientation, for explaining layout.
pub fn axis_trace(fig: &Figure, r: &Resolved) -> Vec<(usize, &'static str, AxisPlan)> {
    let mut out = Vec::new();
    for (p, (panel, plan)) in fig.panels.iter().zip(&r.plans).enumerate() {
        for (a, ap) in panel.axes.iter().zip(plan) {
            let o = match a.orient() {
                Orient::Top => "top",
                Orient::Right => "right",
                Orient::Bottom => "bottom",
                Orient::Left => "left",
            };
            out.push((p, o, *ap));
        }
    }
    out
}

/// Panel cells, for explaining layout.
pub fn cells(fig: &Figure) -> Vec<(usize, usize)> {
    fig.panels.iter().map(|p| p.cell).collect()
}
