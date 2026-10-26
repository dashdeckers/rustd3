//! Problem #4, second half: a non-rectangular clip
//! lowered to geometry, and a 30-frame zoom sequence.

use d3::color::Rgb;
use dlist::lower::{self, ICED};
use dlist::outline::Outlines;
use dlist::shape::Shaper;
use dlist::{diff, skia, vcpu, wheels};
use figmodel::plot::draw::{census, Anchor, Label, List};
use kurbo::{Affine, BezPath, Circle, Point};
use std::collections::HashMap;
use std::time::Instant;

fn polar(shaper: &Shaper) -> List {
    let mut l = List::new([400.0, 400.0]);
    l.fill(kurbo::Rect::new(0.0, 0.0, 400.0, 400.0), Rgb::hex(0xffffff));
    l.push(Affine::translate((200.0, 200.0)));
    l.fill(Circle::new((0.0, 0.0), 150.0), Rgb::hex(0xf4f4f8));
    l.push_clip(Circle::new((0.0, 0.0), 150.0));
    let mut seed = 7u32;
    let mut u = || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) as f64 / (1u32 << 24) as f64
    };
    for i in 0..36 {
        let a = i as f64 * std::f64::consts::TAU / 36.0;
        let line = kurbo::Line::new((0.0, 0.0), (190.0 * a.cos(), 190.0 * a.sin()));
        l.stroke(line, 1.0, Some([4.0, 3.0]), Rgb::hex(0x8c8c8c));
    }
    let mut spiral = BezPath::new();
    for i in 0..400 {
        let t = i as f64 / 40.0;
        let p = Point::new(18.0 * t * t.cos(), 18.0 * t * t.sin());
        if i == 0 { spiral.move_to(p) } else { spiral.line_to(p) }
    }
    l.stroke(spiral, 2.0, None, Rgb::hex(0x4c72b0));
    let at: Vec<Point> = (0..2000)
        .map(|_| Point::new(360.0 * u() - 180.0, 360.0 * u() - 180.0))
        .collect();
    l.markers(Circle::new((0.0, 0.0), 2.0), at, Rgb::hex(0xdd8452).with_opacity(0.6));
    l.label(
        Label::new("clipped at the rim", 60.0, -130.0)
            .size(14.0)
            .anchor(Anchor::Start),
    );
    l.pop();
    l.pop();
    shaper.shape(&mut l);
    l
}

fn main() {
    let shaper = Shaper::new();

    println!("== non-rectangular clip (polar panel)");
    let l = polar(&shaper);
    let mut o = Outlines::default();
    let px = skia::render(&l, 2.0, &mut o);
    lower::PRUNE.store(false, std::sync::atomic::Ordering::Relaxed);
    let t = Instant::now();
    let low0 = lower::lower(&l, ICED, &mut o, &mut lower::Stats::default());
    let t0 = t.elapsed().as_secs_f64() * 1e3;
    lower::PRUNE.store(true, std::sync::atomic::Ordering::Relaxed);
    let mut st = lower::Stats::default();
    let t = Instant::now();
    let low = lower::lower(&l, ICED, &mut o, &mut st);
    let t_low = t.elapsed().as_secs_f64() * 1e3;
    let px0 = skia::render(&low0, 2.0, &mut o);
    println!("without pre-check: {t0:.1} ms");
    let px2 = skia::render(&low, 2.0, &mut o);
    px.save_png("out/polar.png").unwrap();
    px2.save_png("out/polar_lowered.png").unwrap();
    let d0 = diff(px0.data(), px2.data());
    println!("pre-check vs no pre-check: max {} >16: {:.4}%", d0.max, 100.0 * d0.over16);
    let d = diff(px.data(), px2.data());
    println!(
        "lowered in {t_low:.1} ms: {st:?}; {:?}",
        census(&low)
    );
    println!(
        "mask clip vs geometric clip: max {} mean {:.3} >16: {:.3}%",
        d.max, d.mean, 100.0 * d.over16
    );

    println!(
        "anyrender recording scene from the polar list: {} commands",
        anyrender_check(&l)
    );

    println!("\n== 30-frame zoom over RL (GIF-style)");
    let samples = wheels::simulate();
    let fig = wheels::figure(&samples, false);
    let size = [960.0, 760.0];
    let r = fig.resolve(size);
    let rl = fig.rect(&r, 2, 0).unwrap();
    let at = (rl.x + rl.w / 2.0, rl.y + 0.75 * rl.h);
    let frames: Vec<HashMap<usize, [f64; 2]>> = (0..30)
        .map(|i| {
            fig.wheel(&r, at, -(i as f64) * 1.5)
                .into_iter()
                .collect()
        })
        .collect();
    let mut o = Outlines::default();
    let (mut t_draw, mut t_shape, mut t_skia, mut t_vc, mut t_svg, mut t_rs) =
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let ms = |t: Instant| t.elapsed().as_secs_f64() * 1e3;
    for v in &frames {
        let t = Instant::now();
        let mut l = fig.draw_with(size, v);
        t_draw += ms(t);
        let t = Instant::now();
        shaper.shape(&mut l);
        t_shape += ms(t);
        let t = Instant::now();
        let _ = skia::render(&l, 1.0, &mut o);
        t_skia += ms(t);
        let t = Instant::now();
        let _ = vcpu::render(&l, 1.0);
        t_vc += ms(t);
        let t = Instant::now();
        let s = dlist::svg::outlined(&l, &mut o);
        t_svg += ms(t);
        let t = Instant::now();
        let tree = resvg::usvg::Tree::from_str(
            &s,
            &resvg::usvg::Options::default(),
        )
        .unwrap();
        let mut px = tiny_skia::Pixmap::new(960, 760).unwrap();
        resvg::render(&tree, tiny_skia::Transform::identity(), &mut px.as_mut());
        t_rs += ms(t);
    }
    let n = frames.len() as f64;
    println!(
        "per frame at 1×: draw {:.1} ms, shape {:.1} ms, \
         tiny-skia {:.1} ms, vello_cpu {:.1} ms, \
         SVG write {:.1} ms + resvg {:.1} ms",
        t_draw / n, t_shape / n, t_skia / n, t_vc / n, t_svg / n, t_rs / n
    );
}

#[allow(dead_code)]
fn anyrender_check(l: &List) -> usize {
    let mut scene = anyrender::Scene::new();
    dlist::anyr::replay(l, &mut scene, Affine::IDENTITY);
    scene.commands.len()
}
