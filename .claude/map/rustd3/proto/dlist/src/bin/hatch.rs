//! Hatching as clipped line geometry: every bar of the
//! wheel figure's histograms gets 45° lines, clipped to
//! the bar, anchored to the figure so neighbours line up.
use dlist::outline::Outlines;
use dlist::shape::Shaper;
use dlist::{svg, vcpu, wheels};
use figmodel::plot::draw::{Command, List};
use kurbo::{Affine, BezPath, Cap, Join, PathEl, Rect, Shape, Stroke};
use peniko::Brush;
use std::time::Instant;

fn as_rect(p: &BezPath) -> Option<Rect> {
    let pts: Vec<_> = p.elements().iter().filter_map(|e| match e {
        PathEl::MoveTo(a) | PathEl::LineTo(a) => Some(*a),
        PathEl::ClosePath => None,
        _ => Some(kurbo::Point::new(f64::NAN, 0.0)),
    }).collect();
    if pts.len() != 4 || pts.iter().any(|q| q.x.is_nan()) { return None; }
    let r = p.bounding_box();
    pts.iter().all(|q| (q.x == r.x0 || q.x == r.x1) && (q.y == r.y0 || q.y == r.y1)).then_some(r)
}

/// Lines x - y = k * spacing (45°), in figure coordinates,
/// covering `bb` (given in figure coordinates).
fn lines(bb: Rect, spacing: f64) -> BezPath {
    let mut p = BezPath::new();
    let (k0, k1) = (((bb.x0 - bb.y1) / spacing).floor() as i64, ((bb.x1 - bb.y0) / spacing).ceil() as i64);
    for k in k0..=k1 {
        let c = k as f64 * spacing; // x = y + c
        p.move_to((bb.y0 + c, bb.y0));
        p.line_to((bb.y1 + c, bb.y1));
    }
    p
}

fn hatched(l: &List, spacing: f64, width: f64) -> (List, usize) {
    let mut out = List::new(l.size);
    out.fonts = l.fonts.clone();
    let mut t = vec![Affine::IDENTITY];
    let mut n = 0;
    for c in &l.commands {
        let cur = *t.last().unwrap();
        out.commands.push(c.clone());
        match c {
            Command::Push { transform, .. } => t.push(cur * *transform),
            Command::Pop => { t.pop(); }
            Command::Fill { path, .. } => {
                if let Some(r) = as_rect(path) {
                    let fr = cur.transform_rect_bbox(r);
                    if fr.height() > 3.0 && fr.width() > 2.0 && fr.width() < 40.0 {
                        n += 1;
                        // lines live in figure space: push an inverse
                        // transform so they stay anchored to the figure.
                        out.commands.push(Command::Push { transform: Affine::IDENTITY, clip: Some(r.to_path(0.1)) });
                        out.commands.push(Command::Push { transform: cur.inverse(), clip: None });
                        let mut s = Stroke::new(width);
                        s.start_cap = Cap::Butt; s.end_cap = Cap::Butt; s.join = Join::Miter;
                        out.commands.push(Command::Stroke { path: lines(fr, spacing), style: s, paint: Brush::Solid(peniko::Color::from_rgba8(38, 38, 38, 200)) });
                        out.commands.push(Command::Pop);
                        out.commands.push(Command::Pop);
                    }
                }
            }
            _ => {}
        }
    }
    (out, n)
}

fn best(f: impl Fn() -> f64) -> f64 { (0..5).map(|_| f()).fold(f64::MAX, f64::min) }

fn main() {
    let samples = wheels::simulate();
    let mut l = wheels::figure(&samples, false).draw([960.0, 760.0]);
    Shaper::new().shape(&mut l);
    let pt = 4.0 / 3.0;
    let t = Instant::now();
    let (h, bars) = hatched(&l, 6.0 * pt, 0.8 * pt);
    let build = t.elapsed().as_secs_f64() * 1e3;
    let mut o = Outlines::default();
    let (a, b) = (svg::outlined(&l, &mut o).len(), svg::outlined(&h, &mut o).len());
    for s in [1.0, 2.0] {
        let plain = best(|| { let t = Instant::now(); std::hint::black_box(vcpu::render(&l, s)); t.elapsed().as_secs_f64() * 1e3 });
        let hat = best(|| { let t = Instant::now(); std::hint::black_box(vcpu::render(&h, s)); t.elapsed().as_secs_f64() * 1e3 });
        println!("{s}x: {plain:.1} ms plain, {hat:.1} ms with {bars} hatched bars");
    }
    println!("hatch geometry built in {build:.2} ms; SVG {:.0} KB -> {:.0} KB", a as f64 / 1024.0, b as f64 / 1024.0);
    let px = vcpu::render(&h, 2.0);
    let mut img = tiny_skia::Pixmap::new(px.width() as u32, px.height() as u32).unwrap();
    img.data_mut().copy_from_slice(px.data_as_u8_slice());
    std::fs::create_dir_all("out/hatch").unwrap();
    img.save_png("out/hatch/wheels_hatched.png").unwrap();
    std::fs::write("out/hatch/wheels_hatched.svg", svg::outlined(&h, &mut o)).unwrap();
}
