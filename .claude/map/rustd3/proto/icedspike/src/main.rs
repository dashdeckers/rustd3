//! Review G follow-up: four ways to put the wheel
//! figure's drawing list into iced, measured on the
//! CPU side, plus vello on wgpu 30 checked against
//! vello_cpu (on lavapipe, so GPU times mean nothing).

mod prim;
use dlist::lower::{self, Caps, Stats};
use dlist::outline::Outlines;
use dlist::shape::Shaper;
use dlist::{vcpu, wheels};
use figmodel::plot::draw::{Command, List};
use iced_graphics::core::{Color, Point, Rectangle, Size};
use iced_graphics::geometry::frame::Backend;
use iced_graphics::geometry::{LineCap, LineDash, LineJoin, Path, Stroke, Style};
use kurbo::{Affine, BezPath, PathEl, Shape};
use peniko::Brush;
use std::time::Instant;

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1e3
}

fn path(p: &BezPath, t: Affine) -> Path {
    let p = t * p.clone();
    let pt = |q: kurbo::Point| Point::new(q.x as f32, q.y as f32);
    Path::new(|b| {
        for e in p.elements() {
            match *e {
                PathEl::MoveTo(a) => b.move_to(pt(a)),
                PathEl::LineTo(a) => b.line_to(pt(a)),
                PathEl::QuadTo(c, a) => b.quadratic_curve_to(pt(c), pt(a)),
                PathEl::CurveTo(c1, c2, a) => b.bezier_curve_to(pt(c1), pt(c2), pt(a)),
                PathEl::ClosePath => b.close(),
            }
        }
    })
}

fn color(b: &Brush) -> Color {
    match b {
        Brush::Solid(c) => {
            let [r, g, b, a] = c.components;
            Color::from_rgba(r, g, b, a)
        }
        _ => Color::BLACK,
    }
}

/// Replays a lowered list into any iced frame; the
/// transform is applied to the paths, since iced
/// frames take only translate, rotate and scale.
fn replay<F: Backend>(l: &List, f: &mut F) -> usize {
    let mut t = vec![Affine::IDENTITY];
    let mut n = 0;
    for c in &l.commands {
        let cur = *t.last().unwrap();
        match c {
            Command::Fill { path: p, paint } => {
                f.fill(&path(p, cur), color(paint));
                n += 1;
            }
            Command::Stroke { path: p, style, paint } => {
                let k = cur.as_coeffs();
                let s = (k[0] * k[3] - k[1] * k[2]).abs().sqrt() as f32;
                let dash: Vec<f32> = style.dash_pattern.iter().map(|d| *d as f32 * s).collect();
                let stroke = Stroke {
                    style: Style::Solid(color(paint)),
                    width: style.width as f32 * s,
                    line_cap: match style.start_cap {
                        kurbo::Cap::Butt => LineCap::Butt,
                        kurbo::Cap::Round => LineCap::Round,
                        kurbo::Cap::Square => LineCap::Square,
                    },
                    line_join: match style.join {
                        kurbo::Join::Miter => LineJoin::Miter,
                        kurbo::Join::Round => LineJoin::Round,
                        kurbo::Join::Bevel => LineJoin::Bevel,
                    },
                    line_dash: LineDash {
                        segments: &dash,
                        offset: style.dash_offset as usize,
                    },
                };
                f.stroke(&path(p, cur), stroke);
                n += 1;
            }
            Command::Push { transform, .. } => t.push(cur * *transform),
            Command::Pop => {
                t.pop();
            }
            _ => {}
        }
    }
    n
}

fn mesh_stats(g: &iced_wgpu::geometry::Geometry) -> (usize, usize) {
    match g {
        iced_wgpu::geometry::Geometry::Live { meshes, .. } => meshes.iter().fold((0, 0), |(v, i), m| {
            let (a, b) = match m {
                iced_graphics::Mesh::Solid { buffers, .. } => (buffers.vertices.len(), buffers.indices.len()),
                iced_graphics::Mesh::Gradient { buffers, .. } => (buffers.vertices.len(), buffers.indices.len()),
            };
            (v + a, i + b)
        }),
        _ => (0, 0),
    }
}

fn best(reps: usize, mut f: impl FnMut() -> f64) -> f64 {
    (0..reps).map(|_| f()).fold(f64::MAX, f64::min)
}

/// The list as a vello scene: every command maps
/// one to one, glyph runs included.
fn scene(l: &List, scale: f64) -> vello::Scene {
    use vello::peniko::Fill;
    let mut s = vello::Scene::new();
    let mut t = vec![Affine::scale(scale)];
    let mut clips = vec![false];
    for c in &l.commands {
        let cur = *t.last().unwrap();
        match c {
            Command::Fill { path, paint } => s.fill(Fill::NonZero, cur, paint, None, path),
            Command::Stroke { path, style, paint } => s.stroke(style, cur, paint, None, path),
            Command::Markers { shape, at, paint } => {
                for q in at {
                    s.fill(Fill::NonZero, cur * Affine::translate(q.to_vec2()), paint, None, shape);
                }
            }
            Command::Glyphs(r) => {
                let glyphs = r.glyphs.iter().map(|g| vello::Glyph { id: g.id, x: g.x, y: g.y });
                s.draw_glyphs(&l.fonts[r.font])
                    .font_size(r.size)
                    .transform(cur * r.transform)
                    .brush(r.fill)
                    .draw(Fill::NonZero, glyphs);
            }
            Command::Push { transform, clip } => {
                let next = cur * *transform;
                t.push(next);
                if let Some(p) = clip {
                    s.push_clip_layer(Fill::NonZero, next, p);
                }
                clips.push(clip.is_some());
            }
            Command::Pop => {
                t.pop();
                if clips.pop() == Some(true) {
                    s.pop_layer();
                }
            }
            Command::Label(_) => panic!("shape labels first"),
        }
    }
    s
}

fn gpu(l: &List, scale: f64) -> Option<(Vec<u8>, u32, u32, f64, f64)> {
    use vello::wgpu;
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).ok()?;
    println!("  adapter: {}", adapter.get_info().name);
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()?;
    let mut r = vello::Renderer::new(&device, vello::RendererOptions::default()).ok()?;
    let (w, h) = ((l.size[0] * scale) as u32, (l.size[1] * scale) as u32);
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let t = Instant::now();
    let s = scene(l, scale);
    let encode = ms(t);
    let params = vello::RenderParams {
        base_color: peniko::Color::TRANSPARENT,
        width: w,
        height: h,
        antialiasing_method: vello::AaConfig::Area,
    };
    let t = Instant::now();
    r.render_to_texture(&device, &queue, &s, &view, &params).ok()?;
    let stride = (w * 4).div_ceil(256) * 256;
    let buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (stride * h) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(stride), rows_per_image: None } },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    queue.submit([enc.finish()]);
    let slice = buf.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
    let render = ms(t);
    let data = slice.get_mapped_range().ok()?;
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for row in 0..h {
        let a = (row * stride) as usize;
        out.extend_from_slice(&data[a..a + (w * 4) as usize]);
    }
    Some((out, w, h, encode, render))
}

fn main() {
    let samples = wheels::simulate();
    let size = [960.0, 760.0];
    let shaper = Shaper::new();
    let fig = wheels::figure(&samples, false);
    let mut l = fig.draw(size);
    shaper.shape(&mut l);
    let glyph_runs = l.commands.iter().filter(|c| matches!(c, Command::Glyphs(_))).count();
    println!("wheel figure: {} commands, {glyph_runs} glyph runs", l.commands.len());

    // Lowered for iced: markers to fills, clips cut;
    // glyphs either kept (drawn as images) or outlined.
    let mut o = Outlines::default();
    let mut st = Stats::default();
    let no_text = Caps { glyphs: true, ..lower::ICED };
    let geo = lower::lower(&l, no_text, &mut o, &mut st);
    let outl = lower::lower(&l, lower::ICED, &mut o, &mut Stats::default());
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(size[0] as f32, size[1] as f32));

    for (name, list) in [("geometry without text", &geo), ("geometry with text outlines", &outl)] {
        let mut stats = (0, 0, 0);
        let w = best(5, || {
            let t = Instant::now();
            let mut f = iced_wgpu::geometry::Frame::new(bounds);
            let n = replay(list, &mut f);
            let g = f.into_geometry();
            let (v, i) = mesh_stats(&g);
            stats = (n, v, i);
            ms(t)
        });
        let s = best(5, || {
            let t = Instant::now();
            let mut f = iced_tiny_skia::geometry::Frame::new(bounds);
            replay(list, &mut f);
            std::hint::black_box(f.into_geometry());
            ms(t)
        });
        println!(
            "{name}: {} draws; wgpu frame (lyon) {w:.1} ms, {} vertices, {} indices; tiny-skia frame {s:.1} ms",
            stats.0, stats.1, stats.2
        );
    }

    // Whole figure as one image, as a fallback.
    for scale in [1.0, 2.0] {
        let r = best(3, || {
            let t = Instant::now();
            std::hint::black_box(vcpu::render(&l, scale));
            ms(t)
        });
        println!("vello_cpu whole figure at {scale}x: {r:.1} ms");
    }

    // vello on wgpu 30 (same version as iced master).
    let enc = best(5, || {
        let t = Instant::now();
        std::hint::black_box(scene(&l, 1.0));
        ms(t)
    });
    println!("vello scene encoding (CPU): {enc:.2} ms");
    match gpu(&l, 1.0) {
        Some((px, w, h, _, render)) => {
            let reference = vcpu::render(&l, 1.0);
            let a = reference.data_as_u8_slice();
            let (mut worst, mut off) = (0u8, 0usize);
            for (p, q) in a.iter().zip(&px) {
                let d = p.abs_diff(*q);
                worst = worst.max(d);
                off += usize::from(d > 16);
            }
            println!(
                "vello GPU (lavapipe, timing meaningless: {render:.0} ms) vs vello_cpu: max {worst}/255, {:.3}% of channels off by >16",
                100.0 * off as f64 / a.len() as f64
            );
            let mut img = tiny_skia::Pixmap::new(w, h).unwrap();
            img.data_mut().copy_from_slice(&px);
            // vello writes straight alpha; premultiply for tiny-skia's PNG writer.
            std::fs::create_dir_all("out").unwrap();
            let _ = img.save_png("out/vello_gpu.png");
        }
        None => println!("no wgpu adapter"),
    }
}
