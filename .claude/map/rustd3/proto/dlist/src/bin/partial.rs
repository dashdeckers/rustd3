//! Review: what partial re-rendering buys on the image
//! path. Render only a rectangle of the figure (one
//! panel, or a thin strip exposed by a pan), with and
//! without culling commands by bounding box, and check
//! that integer-pixel shifts render identically.

use dlist::outline::Outlines;
use dlist::shape::Shaper;
use dlist::{vcpu, wheels};
use figmodel::plot::draw::{Command, List};
use kurbo::{Affine, Rect, Shape};
use std::time::Instant;
use vello_cpu::{Glyph, Pixmap, RenderContext, Resources};

/// Device-space bounds of every command (None for
/// Push/Pop, which are always kept).
fn bounds(l: &List, o: &mut Outlines) -> Vec<Option<Rect>> {
    let mut t = vec![Affine::IDENTITY];
    let mut out = Vec::new();
    for c in &l.commands {
        let cur = *t.last().unwrap();
        out.push(match c {
            Command::Fill { path, .. } => Some((cur * path.clone()).bounding_box()),
            Command::Stroke { path, style, .. } => {
                Some((cur * path.clone()).bounding_box().inflate(style.width, style.width))
            }
            Command::Markers { shape, at, .. } => {
                let s = shape.bounding_box();
                at.iter()
                    .map(|q| cur.transform_rect_bbox(s + q.to_vec2()))
                    .reduce(|a, b| a.union(b))
            }
            Command::Glyphs(r) => Some((cur * o.run_path(&l.fonts, r)).bounding_box()),
            Command::Push { transform, .. } => {
                t.push(cur * *transform);
                None
            }
            Command::Pop => {
                t.pop();
                None
            }
            Command::Label(_) => None,
        });
    }
    out
}

/// Renders the part of the figure inside `area`
/// (figure px) at `scale`, optionally skipping
/// commands that cannot touch it.
fn render_area(l: &List, bb: &[Option<Rect>], area: Rect, scale: f64, cull: bool) -> (Pixmap, usize) {
    let (pw, ph) = ((area.width() * scale).ceil() as u16, (area.height() * scale).ceil() as u16);
    let mut ctx = RenderContext::new(pw.max(1), ph.max(1));
    let mut res = Resources::new();
    let base = Affine::scale(scale) * Affine::translate((-area.x0, -area.y0));
    let mut t = vec![base];
    let mut clips = vec![false];
    let mut drawn = 0;
    for (c, b) in l.commands.iter().zip(bb) {
        let cur = *t.last().unwrap();
        if cull {
            if let Some(b) = b {
                if b.intersect(area).area() <= 0.0 && !(b.width() == 0.0 || b.height() == 0.0) {
                    continue;
                }
            }
        }
        ctx.set_transform(cur);
        match c {
            Command::Fill { path, paint } => {
                if let peniko::Brush::Solid(col) = paint {
                    ctx.set_paint(*col);
                }
                ctx.fill_path(path);
                drawn += 1;
            }
            Command::Stroke { path, style, paint } => {
                if let peniko::Brush::Solid(col) = paint {
                    ctx.set_paint(*col);
                }
                ctx.set_stroke(style.clone());
                ctx.stroke_path(path);
                drawn += 1;
            }
            Command::Markers { shape, at, paint } => {
                if let peniko::Brush::Solid(col) = paint {
                    ctx.set_paint(*col);
                }
                for q in at {
                    let p = cur * kurbo::Point::new(q.x, q.y);
                    let local = Affine::scale(1.0 / scale) * p;
                    if cull && !area.inflate(4.0, 4.0).contains(local + area.origin().to_vec2()) {
                        continue;
                    }
                    ctx.set_transform(cur * Affine::translate(q.to_vec2()));
                    ctx.fill_path(shape);
                }
                drawn += 1;
            }
            Command::Glyphs(r) => {
                ctx.set_paint(r.fill);
                ctx.set_transform(cur * r.transform);
                let glyphs = r.glyphs.iter().map(|g| Glyph { id: g.id, x: g.x, y: g.y });
                let _ = ctx.glyph_run(&mut res, &l.fonts[r.font]).font_size(r.size).hint(false).fill_glyphs(glyphs);
                drawn += 1;
            }
            Command::Push { transform, clip } => {
                let next = cur * *transform;
                t.push(next);
                if let Some(p) = clip {
                    ctx.set_transform(next);
                    ctx.push_clip_layer(p);
                }
                clips.push(clip.is_some());
            }
            Command::Pop => {
                t.pop();
                if clips.pop() == Some(true) {
                    ctx.pop_layer();
                }
            }
            Command::Label(_) => {}
        }
    }
    ctx.flush();
    let mut px = Pixmap::new(pw.max(1), ph.max(1));
    ctx.render(&mut px, &mut res);
    (px, drawn)
}

fn best(n: usize, mut f: impl FnMut() -> f64) -> f64 {
    (0..n).map(|_| f()).fold(f64::MAX, f64::min)
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1e3
}

fn main() {
    let samples = wheels::simulate();
    let size = [960.0, 760.0];
    let fig = wheels::figure(&samples, false);
    let r = fig.resolve(size);
    let mut l = fig.draw(size);
    Shaper::new().shape(&mut l);
    let mut o = Outlines::default();
    let bb = bounds(&l, &mut o);
    let p = r.rects[0];
    let panel = Rect::new(p.x, p.y, p.x + p.w, p.y + p.h);
    let s = 2.0;
    println!("panel 0: {:.0} x {:.0} px of 960 x 760 ({:.1}% of the figure)", p.w, p.h, 100.0 * p.w * p.h / (960.0 * 760.0));

    let full = best(3, || {
        let t = Instant::now();
        std::hint::black_box(vcpu::render(&l, s));
        ms(t)
    });
    println!("whole figure at {s}x: {full:.1} ms");
    for cull in [false, true] {
        let mut n = 0;
        let t = best(3, || {
            let t = Instant::now();
            let (_, d) = render_area(&l, &bb, panel, s, cull);
            n = d;
            ms(t)
        });
        println!("one panel at {s}x, cull {cull}: {t:.2} ms ({n} commands drawn)");
    }
    for w in [8.0, 32.0] {
        let strip = Rect::new(panel.x1 - w, panel.y0, panel.x1, panel.y1);
        let mut n = 0;
        let t = best(5, || {
            let t = Instant::now();
            let (_, d) = render_area(&l, &bb, strip, s, true);
            n = d;
            ms(t)
        });
        println!("{w} px strip of the panel (pan exposure) at {s}x, culled: {t:.2} ms ({n} commands)");
    }
    // Text share: the figure without glyph runs.
    let mut no_text = l.clone();
    no_text.commands.retain(|c| !matches!(c, Command::Glyphs(_)));
    let nt = best(3, || {
        let t = Instant::now();
        std::hint::black_box(vcpu::render(&no_text, s));
        ms(t)
    });
    println!("whole figure without text at {s}x: {nt:.1} ms (text {:.1} ms)", full - nt);
    // Static chrome vs data: drop markers and dashed fits (data) to see chrome cost.
    let mut chrome = l.clone();
    chrome.commands.retain(|c| !matches!(c, Command::Markers { .. }));
    let ch = best(3, || {
        let t = Instant::now();
        std::hint::black_box(vcpu::render(&chrome, s));
        ms(t)
    });
    println!("whole figure without the dots at {s}x: {ch:.1} ms");

    // Translation equivariance: render the panel, then the
    // same panel shifted by k device px; overlapping parts
    // must match for a scroll-blit to be exact.
    for k in [7.0, 7.5] {
        let (a, _) = render_area(&l, &bb, panel, s, false);
        let shifted = Rect::new(panel.x0 + k / s, panel.y0, panel.x1 + k / s, panel.y1);
        let (b, _) = render_area(&l, &bb, shifted, s, false);
        let (w, h) = (a.width() as usize, a.height() as usize);
        let kk = k.floor() as usize;
        let (ad, bd) = (a.data_as_u8_slice(), b.data_as_u8_slice());
        let mut worst = 0u8;
        for y in 0..h {
            for x in 0..w - kk - 1 {
                for c in 0..4 {
                    let pa = ad[(y * w + x + kk) * 4 + c];
                    let pb = bd[(y * w + x) * 4 + c];
                    worst = worst.max(pa.abs_diff(pb));
                }
            }
        }
        println!("shift by {k} device px: overlap differs by at most {worst}/255");
    }
}
