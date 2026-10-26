//! Review G: fills iced cannot paint (radial, sweep,
//! >8-stop gradients) lowered to images rasterized by
//! vello_cpu, composited as iced's draw_image would.
use kurbo::{Affine, Circle, Rect, Shape};
use peniko::{color::palette::css, Color, ColorStop, Gradient};
use std::time::Instant;
use vello_cpu::{Pixmap, RenderContext};

fn viridis_like(n: usize) -> Vec<ColorStop> {
    (0..n)
        .map(|i| {
            let t = i as f32 / (n - 1) as f32;
            let c = Color::new([0.27 + 0.7 * t * t, 0.0 + 0.9 * t, 0.33 + 0.4 * (3.0 * t).sin() * (1.0 - t), 1.0]);
            ColorStop::from((t, c))
        })
        .collect()
}

fn shapes() -> Vec<(kurbo::BezPath, Gradient)> {
    let radial = Gradient::new_radial((200.0, 150.0), 90.0)
        .with_stops([css::WHITE, css::STEEL_BLUE, css::MIDNIGHT_BLUE]);
    let sweep = Gradient::new_sweep((480.0, 150.0), 0.0, std::f32::consts::TAU)
        .with_stops([css::RED, css::YELLOW, css::LIME, css::CYAN, css::BLUE, css::MAGENTA, css::RED]);
    let mut ring = Circle::new((480.0, 150.0), 100.0).to_path(0.1);
    ring.extend(Affine::IDENTITY * Circle::new((480.0, 150.0), 55.0).to_path(0.1).reverse_subpaths());
    let bar = Gradient::new_linear((80.0, 320.0), (600.0, 320.0)).with_stops(viridis_like(32).as_slice());
    vec![
        (Circle::new((200.0, 150.0), 90.0).to_path(0.1), radial),
        (ring, sweep),
        (Rect::new(80.0, 300.0, 600.0, 340.0).to_path(0.1), bar),
    ]
}

trait Rev {
    fn reverse_subpaths(self) -> kurbo::BezPath;
}
impl Rev for kurbo::BezPath {
    fn reverse_subpaths(self) -> kurbo::BezPath {
        // reverse winding so the inner circle is a hole
        let pts: Vec<_> = self.elements().to_vec();
        let mut flat = Vec::new();
        kurbo::flatten(pts, 0.05, |e| flat.push(e));
        let mut v: Vec<kurbo::Point> = flat
            .iter()
            .filter_map(|e| match e {
                kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) => Some(*p),
                _ => None,
            })
            .collect();
        v.reverse();
        let mut out = kurbo::BezPath::new();
        out.move_to(v[0]);
        for p in &v[1..] {
            out.line_to(*p);
        }
        out.close_path();
        out
    }
}

fn to_ts(px: &Pixmap) -> tiny_skia::Pixmap {
    let mut out = tiny_skia::Pixmap::new(px.width() as u32, px.height() as u32).unwrap();
    out.data_mut().copy_from_slice(px.data_as_u8_slice());
    out
}

fn main() {
    let (w, h, s) = (680u16, 380u16, 2.0);
    let (pw, ph) = ((w as f64 * s) as u16, (h as f64 * s) as u16);
    // Reference: everything drawn directly.
    let mut ctx = RenderContext::new(pw, ph);
    ctx.set_paint(css::WHITE);
    ctx.fill_rect(&Rect::new(0.0, 0.0, pw as f64, ph as f64));
    for (p, g) in shapes() {
        ctx.set_transform(Affine::scale(s));
        ctx.set_paint(g);
        ctx.fill_path(&p);
    }
    ctx.flush();
    let mut direct = Pixmap::new(pw, ph);
    ctx.render(&mut direct, &mut vello_cpu::Resources::new());
    // Lowered: each fill rasterized alone into a
    // sprite covering its device-pixel bounds.
    let mut sprites = Vec::new();
    let t = Instant::now();
    for (p, g) in shapes() {
        let bb = (Affine::scale(s) * p.clone()).bounding_box();
        let (x0, y0) = (bb.x0.floor(), bb.y0.floor());
        let (sw, sh) = ((bb.x1.ceil() - x0) as u16, (bb.y1.ceil() - y0) as u16);
        let mut c = RenderContext::new(sw, sh);
        c.set_transform(Affine::translate((-x0, -y0)) * Affine::scale(s));
        c.set_paint(g);
        c.fill_path(&p);
        c.flush();
        let mut px = Pixmap::new(sw, sh);
        c.render(&mut px, &mut vello_cpu::Resources::new());
        sprites.push((x0 as i32, y0 as i32, px));
    }
    let make = t.elapsed().as_secs_f64() * 1e3;
    let bytes: usize = sprites.iter().map(|s| s.2.data_as_u8_slice().len()).sum();
    let mut lowered = tiny_skia::Pixmap::new(pw as u32, ph as u32).unwrap();
    lowered.fill(tiny_skia::Color::WHITE);
    for (x, y, px) in &sprites {
        lowered.draw_pixmap(*x, *y, to_ts(px).as_ref(), &tiny_skia::PixmapPaint::default(), tiny_skia::Transform::identity(), None);
    }
    let a = to_ts(&direct);
    let (mut worst, mut off) = (0u8, 0usize);
    for (p, q) in a.data().iter().zip(lowered.data()) {
        let d = p.abs_diff(*q);
        worst = worst.max(d);
        off += usize::from(d > 2);
    }
    println!(
        "3 sprites at {s}x: {make:.2} ms, {:.0} KiB; max diff {worst}/255, {:.3}% of channels off by >2",
        bytes as f64 / 1024.0,
        100.0 * off as f64 / a.data().len() as f64
    );
    std::fs::create_dir_all("out/paints").unwrap();
    a.save_png("out/paints/direct.png").unwrap();
    lowered.save_png("out/paints/lowered.png").unwrap();
}
