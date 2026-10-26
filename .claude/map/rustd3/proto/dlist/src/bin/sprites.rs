//! Review A: text in iced as cached images. Each glyph
//! run rasterized once into a sprite (vello_cpu), or
//! the same run sent through SVG and resvg, as iced's
//! `Svg` would do it.

use dlist::outline::Outlines;
use dlist::shape::Shaper;
use dlist::{svg, vcpu, wheels};
use figmodel::plot::draw::{Command, GlyphRun, List};
use kurbo::{Affine, Shape};
use resvg::{tiny_skia, usvg};
use std::time::Instant;

/// Every glyph run with its full transform.
fn runs(l: &List) -> Vec<GlyphRun> {
    let mut t = vec![Affine::IDENTITY];
    let mut out = Vec::new();
    for c in &l.commands {
        let cur = *t.last().unwrap();
        match c {
            Command::Push { transform, .. } => t.push(cur * *transform),
            Command::Pop => {
                t.pop();
            }
            Command::Glyphs(r) => {
                let mut r = r.clone();
                r.transform = cur * r.transform;
                out.push(r);
            }
            _ => {}
        }
    }
    out
}

/// A list holding one run, moved so its ink starts at
/// a pixel corner, at `scale` device px per px.
fn alone(l: &List, r: &GlyphRun, o: &mut Outlines, scale: f64) -> List {
    let bb = o.run_path(&l.fonts, r).bounding_box();
    let (x0, y0) = ((bb.x0 * scale).floor() / scale, (bb.y0 * scale).floor() / scale);
    let mut one = List::new([bb.x1 - x0 + 1.0, bb.y1 - y0 + 1.0]);
    one.fonts = l.fonts.clone();
    let mut r = r.clone();
    r.transform = Affine::translate((-x0, -y0)) * r.transform;
    one.commands.push(Command::Glyphs(r));
    one
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1e3
}

fn main() {
    let samples = wheels::simulate();
    let size = [960.0, 760.0];
    let shaper = Shaper::new();
    let fig = wheels::figure(&samples, false);
    let mut l = fig.draw(size);
    shaper.shape(&mut l);
    let runs = runs(&l);
    let mut o = Outlines::default();
    let opt = usvg::Options::default();
    println!("{} glyph runs", runs.len());
    for scale in [1.0, 2.0] {
        let lists: Vec<List> = runs.iter().map(|r| alone(&l, r, &mut o, scale)).collect();
        let px: usize = lists
            .iter()
            .map(|s| ((s.size[0] * scale) as usize) * ((s.size[1] * scale) as usize))
            .sum();
        // Sprites straight from the list.
        let t = Instant::now();
        for s in &lists {
            std::hint::black_box(vcpu::render(s, scale));
        }
        let sprite = ms(t);
        // The same runs as SVG documents through resvg.
        let docs: Vec<String> = lists.iter().map(|s| svg::outlined(s, &mut o)).collect();
        let t = Instant::now();
        for (s, d) in lists.iter().zip(&docs) {
            let tree = usvg::Tree::from_str(d, &opt).unwrap();
            let (w, h) = ((s.size[0] * scale) as u32, (s.size[1] * scale) as u32);
            let mut p = tiny_skia::Pixmap::new(w.max(1), h.max(1)).unwrap();
            resvg::render(&tree, tiny_skia::Transform::from_scale(scale as f32, scale as f32), &mut p.as_mut());
            std::hint::black_box(p);
        }
        let viasvg = ms(t);
        println!(
            "scale {scale}: sprites {sprite:.2} ms, via SVG+resvg {viasvg:.2} ms, {:.0} KiB RGBA",
            (px * 4) as f64 / 1024.0
        );
    }
    // The whole figure, both ways.
    let t = Instant::now();
    std::hint::black_box(vcpu::render(&l, 1.0));
    let whole = ms(t);
    let t = Instant::now();
    let d = svg::outlined(&l, &mut o);
    let ser = ms(t);
    let t = Instant::now();
    let tree = usvg::Tree::from_str(&d, &opt).unwrap();
    let parse = ms(t);
    let t = Instant::now();
    let mut p = tiny_skia::Pixmap::new(960, 760).unwrap();
    resvg::render(&tree, tiny_skia::Transform::identity(), &mut p.as_mut());
    let rast = ms(t);
    println!(
        "whole figure: vello_cpu {whole:.1} ms; SVG {:.0} KiB written {ser:.1} ms, parsed {parse:.1} ms, resvg {rast:.1} ms",
        d.len() as f64 / 1024.0
    );
}
