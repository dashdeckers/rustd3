//! The wheel figure in iced's shader widget, drawn by
//! vello on iced's wgpu device, snapshotted headlessly
//! by iced_test (wgpu on lavapipe here).
#[path = "../prim.rs"]
mod prim;

use dlist::shape::Shaper;
use dlist::{vcpu, wheels};
use figmodel::plot::draw::{Command, List};
use iced::widget::shader;
use iced::{Fill, Rectangle, Theme, mouse};
use kurbo::Affine;
use std::sync::Arc;

fn scene(l: &List) -> vello::Scene {
    use vello::peniko::Fill;
    let mut s = vello::Scene::new();
    let mut t = vec![Affine::IDENTITY];
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

struct Plot(Arc<prim::SceneBox>);

impl shader::Program<()> for Plot {
    type State = ();
    type Primitive = prim::Figure;
    fn draw(&self, _: &(), _: mouse::Cursor, _: Rectangle) -> prim::Figure {
        prim::Figure { scene: self.0.clone() }
    }
}

fn main() {
    let samples = wheels::simulate();
    let size = [960.0, 760.0];
    let mut l = wheels::figure(&samples, false).draw(size);
    Shaper::new().shape(&mut l);
    let plot = Plot(Arc::new(prim::SceneBox(scene(&l))));
    let view = shader(plot).width(Fill).height(Fill);
    let mut sim = iced_test::simulator::Simulator::<(), Theme, iced::Renderer>::with_size(
        iced::Settings::default(),
        (960.0, 760.0),
        view,
    );
    let snap = sim.snapshot(&Theme::Light).expect("snapshot");
    std::fs::create_dir_all("out").unwrap();
    let _ = std::fs::remove_file("out/plotwidget-wgpu.png");
    let _ = std::fs::remove_file("out/plotwidget-tiny-skia.png");
    let _ = snap.matches_image("out/plotwidget");
    // Reference: the same list through vello_cpu on white.
    let px = vcpu::render(&l, 2.0);
    let mut reference = tiny_skia::Pixmap::new(1920, 1520).unwrap();
    reference.fill(tiny_skia::Color::WHITE);
    let mut fig = tiny_skia::Pixmap::new(1920, 1520).unwrap();
    fig.data_mut().copy_from_slice(px.data_as_u8_slice());
    reference.draw_pixmap(0, 0, fig.as_ref(), &tiny_skia::PixmapPaint::default(), tiny_skia::Transform::identity(), None);
    reference.save_png("out/reference.png").unwrap();
    for f in std::fs::read_dir("out").unwrap().flatten() {
        let name = f.file_name().into_string().unwrap();
        if name.starts_with("plotwidget") && name.ends_with(".png") {
            let shot = tiny_skia::Pixmap::load_png(f.path()).unwrap();
            let (mut worst, mut off) = (0u8, 0usize);
            for (p, q) in reference.data().iter().zip(shot.data()) {
                let d = p.abs_diff(*q);
                worst = worst.max(d);
                off += usize::from(d > 16);
            }
            println!(
                "{name} ({}x{}) vs vello_cpu on white: max {worst}/255, {:.3}% of channels off by >16",
                shot.width(),
                shot.height(),
                100.0 * off as f64 / reference.data().len() as f64
            );
        }
    }
}
