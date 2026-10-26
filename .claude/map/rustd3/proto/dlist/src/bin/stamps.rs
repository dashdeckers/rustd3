//! Review: markers stamped from pre-rasterized sprites
//! (16 sub-pixel phases per shape and paint) instead of
//! filling one path per dot, inside the same vello_cpu
//! pass so drawing order is kept.
use dlist::shape::Shaper;
use dlist::{vcpu, wheels};
use figmodel::plot::draw::{Command, List};
use kurbo::{Affine, Rect, Shape};
use peniko::{Brush, Extend, ImageQuality, ImageSampler};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use vello_cpu::{Glyph, Image, ImageSource, Pixmap, RenderContext, Resources};

const Q: f64 = 4.0; // sub-pixel phases per axis

fn render(l: &List, scale: f64, sprites: &mut HashMap<(usize, i32, i32), (Arc<Pixmap>, f64, f64)>) -> Pixmap {
    let [w, h] = l.size;
    let (pw, ph) = ((w * scale) as u16, (h * scale) as u16);
    let mut ctx = RenderContext::new(pw, ph);
    let mut res = Resources::new();
    let mut t = vec![Affine::scale(scale)];
    let mut clips = vec![false];
    for (i, c) in l.commands.iter().enumerate() {
        let cur = *t.last().unwrap();
        ctx.set_transform(cur);
        match c {
            Command::Fill { path, paint } => {
                if let Brush::Solid(col) = paint { ctx.set_paint(*col); }
                ctx.fill_path(path);
            }
            Command::Stroke { path, style, paint } => {
                if let Brush::Solid(col) = paint { ctx.set_paint(*col); }
                ctx.set_stroke(style.clone());
                ctx.stroke_path(path);
            }
            Command::Markers { shape, at, paint } => {
                let Brush::Solid(col) = paint else { continue };
                // Device-space shape for this command's linear part.
                let lin = Affine::new([cur.as_coeffs()[0], cur.as_coeffs()[1], cur.as_coeffs()[2], cur.as_coeffs()[3], 0.0, 0.0]);
                let sb = (lin * shape.clone()).bounding_box();
                for q in at {
                    let d = cur * *q;
                    let (fx, fy) = (((d.x - d.x.floor()) * Q).floor() as i32, ((d.y - d.y.floor()) * Q).floor() as i32);
                    let (spr, ox, oy) = sprites.entry((i, fx, fy)).or_insert_with(|| {
                        let off = kurbo::Vec2::new(fx as f64 / Q - sb.x0.floor() + 1.0, fy as f64 / Q - sb.y0.floor() + 1.0);
                        let (sw, sh) = ((sb.width().ceil() + 3.0) as u16, (sb.height().ceil() + 3.0) as u16);
                        let mut c = RenderContext::new(sw, sh);
                        c.set_paint(*col);
                        c.set_transform(Affine::translate(off) * lin);
                        c.fill_path(shape);
                        c.flush();
                        let mut px = Pixmap::new(sw, sh);
                        c.render(&mut px, &mut Resources::new());
                        (Arc::new(px), sb.x0.floor() - 1.0, sb.y0.floor() - 1.0)
                    }).clone();
                    let (x, y) = (d.x.floor() + ox, d.y.floor() + oy);
                    ctx.set_transform(Affine::IDENTITY);
                    ctx.set_paint(Image {
                        image: ImageSource::Pixmap(spr.clone()),
                        sampler: ImageSampler { x_extend: Extend::Pad, y_extend: Extend::Pad, quality: ImageQuality::Low, alpha: 1.0 },
                    });
                    ctx.set_paint_transform(Affine::translate((x, y)));
                    ctx.fill_rect(&Rect::new(x, y, x + spr.width() as f64, y + spr.height() as f64));
                }
                ctx.set_paint_transform(Affine::IDENTITY);
            }
            Command::Glyphs(r) => {
                ctx.set_paint(r.fill);
                ctx.set_transform(cur * r.transform);
                let glyphs = r.glyphs.iter().map(|g| Glyph { id: g.id, x: g.x, y: g.y });
                let _ = ctx.glyph_run(&mut res, &l.fonts[r.font]).font_size(r.size).hint(false).fill_glyphs(glyphs);
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
                if clips.pop() == Some(true) { ctx.pop_layer(); }
            }
            Command::Label(_) => {}
        }
    }
    ctx.flush();
    let mut px = Pixmap::new(pw, ph);
    ctx.render(&mut px, &mut res);
    px
}

fn main() {
    let samples = wheels::simulate();
    let mut l = wheels::figure(&samples, false).draw([960.0, 760.0]);
    Shaper::new().shape(&mut l);
    let dots: usize = l.commands.iter().map(|c| if let Command::Markers { at, .. } = c { at.len() } else { 0 }).sum();
    for s in [1.0, 2.0] {
        let reference = vcpu::render(&l, s);
        let base = (0..5).map(|_| { let t = Instant::now(); std::hint::black_box(vcpu::render(&l, s)); t.elapsed().as_secs_f64() * 1e3 }).fold(f64::MAX, f64::min);
        let mut sprites = HashMap::new();
        let t = Instant::now();
        let first = render(&l, s, &mut sprites);
        let cold = t.elapsed().as_secs_f64() * 1e3;
        let warm = (0..5).map(|_| { let t = Instant::now(); std::hint::black_box(render(&l, s, &mut sprites)); t.elapsed().as_secs_f64() * 1e3 }).fold(f64::MAX, f64::min);
        let (a, b) = (reference.data_as_u8_slice(), first.data_as_u8_slice());
        let worst = a.iter().zip(b).map(|(p, q)| p.abs_diff(*q)).max().unwrap();
        let off = a.iter().zip(b).filter(|(p, q)| p.abs_diff(**q) > 16).count();
        println!(
            "{s}x, {dots} dots: paths {base:.1} ms; stamped {warm:.1} ms warm, {cold:.1} ms cold ({} sprites); max diff {worst}/255, {:.3}% of channels off by >16",
            sprites.len(),
            100.0 * off as f64 / a.len() as f64
        );
    }
}
