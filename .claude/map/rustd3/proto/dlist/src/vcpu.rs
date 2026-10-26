//! The list replayed into vello_cpu: kurbo paths, peniko
//! paint and font blobs pass through unconverted.

use figmodel::plot::draw::{Command, List};
use kurbo::Affine;
use peniko::Brush;
use vello_cpu::{Glyph, Pixmap, RenderContext, Resources};

pub fn render(l: &List, scale: f64) -> Pixmap {
    render_threads(l, scale, 0)
}

/// As `render`, with vello_cpu worker threads (0 = none).
pub fn render_threads(l: &List, scale: f64, threads: u16) -> Pixmap {
    render_res(l, scale, threads, &mut Resources::new())
}

/// As `render_threads`, reusing caller-owned resources
/// (glyph caches survive between renders).
pub fn render_res(l: &List, scale: f64, threads: u16, res: &mut Resources) -> Pixmap {
    let [w, h] = l.size;
    let (pw, ph) = ((w * scale) as u16, (h * scale) as u16);
    let settings = vello_cpu::RenderSettings { num_threads: threads, ..Default::default() };
    let mut ctx = RenderContext::new_with(pw, ph, settings);

    let mut t = vec![Affine::scale(scale)];
    let mut clips = vec![false];
    for c in &l.commands {
        let cur = *t.last().unwrap();
        ctx.set_transform(cur);
        match c {
            Command::Fill { path, paint } => {
                set(&mut ctx, paint);
                ctx.fill_path(path);
            }
            Command::Stroke { path, style, paint } => {
                set(&mut ctx, paint);
                ctx.set_stroke(style.clone());
                ctx.stroke_path(path);
            }
            Command::Markers { shape, at, paint } => {
                set(&mut ctx, paint);
                for q in at {
                    ctx.set_transform(
                        cur * Affine::translate(q.to_vec2()),
                    );
                    ctx.fill_path(shape);
                }
            }
            Command::Glyphs(r) => {
                ctx.set_paint(r.fill);
                ctx.set_transform(cur * r.transform);
                let glyphs = r.glyphs.iter().map(|g| Glyph {
                    id: g.id,
                    x: g.x,
                    y: g.y,
                });
                let font = &l.fonts[r.font];
                let _ = ctx
                    .glyph_run(res, font)
                    .font_size(r.size)
                    .hint(false)
                    .fill_glyphs(glyphs);
            }
            Command::Label(_) => panic!("shape labels first"),
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
        }
    }
    ctx.flush();
    let mut px = Pixmap::new(pw, ph);
    ctx.render(&mut px, res);
    px
}

fn set(ctx: &mut RenderContext, b: &Brush) {
    if let Brush::Solid(c) = b {
        ctx.set_paint(*c);
    }
}
