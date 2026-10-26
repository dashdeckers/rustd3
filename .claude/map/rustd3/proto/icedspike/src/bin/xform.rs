//! Does a custom widget's with_transformation move and
//! scale cached frame geometry, on both renderers, and
//! can one cache be drawn twice in a frame?
use iced::advanced::graphics::geometry::Renderer as _;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::widget::{Meta, Tree, Widget};
use iced::widget::canvas::{Cache, Frame};
use iced::{Color, Length, Point, Rectangle, Renderer, Size, Theme, Transformation, mouse};
use std::cell::Cell;

struct Probe {
    a: Cache<Renderer>,
    b: Cache<Renderer>,
    builds: Cell<usize>,
}
impl Meta for Probe {}

fn square(f: &mut Frame<Renderer>) {
    // A 40 px red square at (100, 100).
    f.fill_rectangle(Point::new(100.0, 100.0), Size::new(40.0, 40.0), Color::from_rgb(1.0, 0.0, 0.0));
}

impl Widget<(), Theme, Renderer> for Probe {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }
    fn layout(&mut self, tree: &mut Tree, _: &Renderer, limits: &layout::Limits) {
        tree.size = layout::atomic(limits, Length::Fill, Length::Fill);
    }
    fn draw(&self, _: &Tree, r: &mut Renderer, _: &Theme, _: &renderer::Style, layout: Layout, _: mouse::Cursor, _: &Rectangle) {
        let s = layout.bounds().size();
        let mut build = |f: &mut Frame<Renderer>| {
            self.builds.set(self.builds.get() + 1);
            square(f)
        };
        // 1. as built: square at (100,100)
        r.draw_geometry(self.a.draw(r, s, &mut build));
        // 2. same cache again, translated by (400, 0): (500,100)
        r.with_transformation(Transformation::translate(400.0, 0.0), |r| {
            r.draw_geometry(self.a.draw(r, s, &mut build));
        });
        // 3. a second cache, scaled 2x then translated: (300..380, 400..480)
        r.with_transformation(Transformation::translate(100.0, 200.0) * Transformation::scale(2.0), |r| {
            r.draw_geometry(self.b.draw(r, s, &mut build));
        });
    }
}

fn main() {
    let probe = Probe { a: Cache::new(), b: Cache::new(), builds: Cell::new(0) };
    let mut sim = iced_test::simulator::Simulator::<(), Theme, Renderer>::with_size(iced::Settings::default(), (960.0, 760.0), probe);
    let shot = sim.snapshot(&Theme::Light).expect("snapshot");
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_default();
    let tag = format!("xform-{backend}");
    for e in std::fs::read_dir("out").unwrap().flatten() {
        if e.file_name().to_string_lossy().starts_with(&tag) {
            let _ = std::fs::remove_file(e.path());
        }
    }
    let _ = shot.matches_image(format!("out/{tag}"));
    let path = std::fs::read_dir("out").unwrap().flatten().find(|e| e.file_name().to_string_lossy().starts_with(&tag)).unwrap().path();
    let img = tiny_skia::Pixmap::load_png(&path).unwrap();
    let k = img.width() as f32 / 960.0;
    let red = |x: f32, y: f32| {
        let i = (((y * k) as usize) * img.width() as usize + (x * k) as usize) * 4;
        let p = &img.data()[i..i + 4];
        p[0] > 200 && p[1] < 60 && p[2] < 60
    };
    println!(
        "{backend}: copy 1 {}, copy 2 (same cache, translated) {}, copy 3 (scaled 2x) {} / its far corner {}",
        red(120.0, 120.0),
        red(520.0, 120.0),
        red(310.0, 410.0),
        red(370.0, 470.0),
    );
}
