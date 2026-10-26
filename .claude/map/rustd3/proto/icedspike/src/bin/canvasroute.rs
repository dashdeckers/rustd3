//! The current RFC route, snapshotted by iced_test on
//! both renderers: the lowered list (text left out)
//! replayed into a canvas::Program, and a with_clip
//! check.
use dlist::lower::{self, Caps, Stats};
use dlist::outline::Outlines;
use dlist::shape::Shaper;
use dlist::{vcpu, wheels};
use figmodel::plot::draw::{Command, List};
use iced::widget::canvas;
use iced::widget::canvas::{Frame, LineCap, LineDash, LineJoin, Path, Stroke};
use iced::{Color, Fill, Point, Rectangle, Renderer, Size, Theme, mouse};
use kurbo::{Affine, BezPath, PathEl};
use peniko::Brush;

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

fn replay(l: &List, f: &mut Frame) {
    let mut t = vec![Affine::IDENTITY];
    for c in &l.commands {
        let cur = *t.last().unwrap();
        match c {
            Command::Fill { path: p, paint } => f.fill(&path(p, cur), color(paint)),
            Command::Stroke { path: p, style, paint } => {
                let k = cur.as_coeffs();
                let s = (k[0] * k[3] - k[1] * k[2]).abs().sqrt() as f32;
                let dash: Vec<f32> = style.dash_pattern.iter().map(|d| *d as f32 * s).collect();
                f.stroke(
                    &path(p, cur),
                    Stroke {
                        style: canvas::Style::Solid(color(paint)),
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
                        line_dash: LineDash { segments: &dash, offset: style.dash_offset as usize },
                    },
                );
            }
            Command::Push { transform, .. } => t.push(cur * *transform),
            Command::Pop => {
                t.pop();
            }
            _ => {}
        }
    }
}

enum Mode {
    Figure(List),
    Clip,
}

struct Prog(Mode);

impl canvas::Program<()> for Prog {
    type State = ();
    fn draw(&self, _: &(), renderer: &Renderer, _: &Theme, bounds: Rectangle, _: mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut f = Frame::new(renderer, bounds.size());
        match &self.0 {
            Mode::Figure(l) => replay(l, &mut f),
            Mode::Clip => {
                f.fill_rectangle(Point::ORIGIN, bounds.size(), Color::WHITE);
                f.with_clip(Rectangle::new(Point::new(400.0, 300.0), Size::new(160.0, 160.0)), |f| {
                    let line = Path::line(Point::new(0.0, 0.0), Point::new(960.0, 760.0));
                    f.stroke(&line, Stroke::default().with_width(12.0).with_color(Color::from_rgb(1.0, 0.0, 0.0)));
                    let disc = Path::circle(Point::new(480.0, 380.0), 200.0);
                    f.fill(&disc, Color::from_rgba(0.0, 0.0, 1.0, 0.3));
                });
            }
        }
        vec![f.into_geometry()]
    }
}

fn snap(p: Prog, name: &str) -> tiny_skia::Pixmap {
    let w = canvas(p).width(Fill).height(Fill);
    let mut sim = iced_test::simulator::Simulator::<(), Theme, Renderer>::with_size(
        iced::Settings::default(),
        (960.0, 760.0),
        w,
    );
    let s = sim.snapshot(&Theme::Light).expect("snapshot");
    for f in std::fs::read_dir("out").unwrap().flatten() {
        let n = f.file_name().into_string().unwrap();
        if n.starts_with(name) {
            let _ = std::fs::remove_file(f.path());
        }
    }
    let _ = s.matches_image(format!("out/{name}"));
    let f = std::fs::read_dir("out")
        .unwrap()
        .flatten()
        .find(|f| f.file_name().into_string().unwrap().starts_with(name))
        .unwrap();
    println!("  {}", f.file_name().into_string().unwrap());
    tiny_skia::Pixmap::load_png(f.path()).unwrap()
}


fn main() {
    let samples = wheels::simulate();
    let size = [960.0, 760.0];
    let mut l = wheels::figure(&samples, false).draw(size);
    Shaper::new().shape(&mut l);
    let mut geo = lower::lower(&l, Caps { glyphs: true, ..lower::ICED }, &mut Outlines::default(), &mut Stats::default());
    geo.commands.retain(|c| !matches!(c, Command::Glyphs(_)));
    // Reference: the same text-free list in vello_cpu at 2x on white.
    let px = vcpu::render(&geo, 2.0);
    let mut reference = tiny_skia::Pixmap::new(1920, 1520).unwrap();
    reference.fill(tiny_skia::Color::WHITE);
    let mut fig = tiny_skia::Pixmap::new(1920, 1520).unwrap();
    fig.data_mut().copy_from_slice(px.data_as_u8_slice());
    reference.draw_pixmap(0, 0, fig.as_ref(), &tiny_skia::PixmapPaint::default(), tiny_skia::Transform::identity(), None);
    std::fs::create_dir_all("out").unwrap();
    let shot = snap(Prog(Mode::Figure(geo)), "canvasfig");
    let (mut worst, mut off, mut hist) = (0u8, 0usize, [0usize; 5]);
    for (p, q) in reference.data().iter().zip(shot.data()) {
        let d = p.abs_diff(*q);
        worst = worst.max(d);
        off += usize::from(d > 16);
        hist[match d { 0..=2 => 0, 3..=16 => 1, 17..=32 => 2, 33..=64 => 3, _ => 4 }] += 1;
    }
    println!(
        "canvas route vs vello_cpu (text left out of both): max {worst}/255, {:.3}% of channels off by >16; >32: {:.3}%, >64: {:.3}%",
        100.0 * off as f64 / reference.data().len() as f64,
        100.0 * (hist[3] + hist[4]) as f64 / reference.data().len() as f64,
        100.0 * hist[4] as f64 / reference.data().len() as f64
    );
    let clip = snap(Prog(Mode::Clip), "canvasclip");
    // Red or blue ink outside the 160 px clip square (at 2x).
    let (w, _) = (clip.width() as usize, clip.height());
    let mut outside = 0;
    for (i, px) in clip.data().chunks(4).enumerate() {
        let (x, y) = (i % w, i / w);
        let inside = (800..1120).contains(&x) && (600..920).contains(&y);
        let inked = px[0] < 250 || px[1] < 250 || px[2] < 250;
        if !inside && inked {
            outside += 1;
        }
    }
    println!("with_clip: {outside} inked pixels outside the clip rectangle");
}
