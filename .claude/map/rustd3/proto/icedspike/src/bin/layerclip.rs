//! Custom widget on both renderers: does a layer clip
//! frame geometry, and does with_transformation move
//! cached geometry without rebuilding it?
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Renderer as _};
use iced::advanced::widget::{Meta, Tree, Widget};
use iced::advanced::graphics::geometry::Renderer as _;
use iced::widget::canvas::{Cache, Path, Stroke};
use iced::{Color, Length, Point, Rectangle, Renderer, Size, Theme, Transformation, Vector, mouse};
use std::cell::Cell;

struct Probe {
    cache: Cache<Renderer>,
    builds: Cell<usize>,
}

impl Meta for Probe {}

impl Widget<(), Theme, Renderer> for Probe {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }
    fn layout(&mut self, tree: &mut Tree, _: &Renderer, limits: &layout::Limits) {
        tree.size = layout::atomic(limits, Length::Fill, Length::Fill);
    }
    fn draw(&self, _: &Tree, r: &mut Renderer, _: &Theme, _: &renderer::Style, layout: Layout, _: mouse::Cursor, _: &Rectangle) {
        let b = layout.bounds();
        // Built once, at the widget's size.
        let build = |f: &mut iced::widget::canvas::Frame<Renderer>| {
            self.builds.set(self.builds.get() + 1);
            f.fill_rectangle(Point::ORIGIN, b.size(), Color::WHITE);
            f.stroke(
                &Path::line(Point::new(0.0, 380.0), Point::new(960.0, 380.0)),
                Stroke::default().with_width(10.0).with_color(Color::from_rgb(1.0, 0.0, 0.0)),
            );
            f.fill(&Path::circle(Point::new(480.0, 380.0), 200.0), Color::from_rgba(0.0, 0.0, 1.0, 0.3));
        };
        let geo = self.cache.draw(r, b.size(), build);
        // Clip to a 160 px square; shift right 100 px and
        // scale 1.5x about the origin while drawing.
        let clip = Rectangle::new(Point::new(400.0, 300.0), Size::new(160.0, 160.0));
        r.with_layer(clip, |r| {
            r.with_transformation(Transformation::translate(100.0, 0.0) * Transformation::scale(1.0), |r| {
                r.draw_geometry(geo);
            });
        });
        // Second copy, scaled, drawn unclipped in the
        // top-left corner to show transforms apply.
        let again = self.cache.draw(r, b.size(), build);
        r.with_transformation(Transformation::scale(0.25), |r| r.draw_geometry(again));
        println!("cache builds after two draws: {}", self.builds.get());
    }
}

fn main() {
    let probe = Probe { cache: Cache::new(), builds: Cell::new(0) };
    let mut sim = iced_test::simulator::Simulator::<(), Theme, Renderer>::with_size(
        iced::Settings::default(),
        (960.0, 760.0),
        probe,
    );
    let s = sim.snapshot(&Theme::Light).expect("snapshot");
    std::fs::create_dir_all("out").unwrap();
    let backend = std::env::var("ICED_TEST_BACKEND").unwrap_or_default();
    let name = format!("out/layer-{backend}");
    for e in std::fs::read_dir("out").unwrap().flatten() {
        if e.file_name().to_string_lossy().starts_with(&format!("layer-{backend}")) {
            let _ = std::fs::remove_file(e.path());
        }
    }
    let _ = s.matches_image(&name);
    let path = std::fs::read_dir("out")
        .unwrap()
        .flatten()
        .find(|e| e.file_name().to_string_lossy().starts_with(&format!("layer-{backend}")))
        .unwrap()
        .path();
    let img = tiny_skia::Pixmap::load_png(&path).unwrap();
    let k = img.width() as f32 / 960.0;
    let w = img.width() as usize;
    let (mut outside, mut corner) = (0usize, 0usize);
    for (i, p) in img.data().chunks(4).enumerate() {
        let (x, y) = ((i % w) as f32 / k, (i / w) as f32 / k);
        let inked = p[3] > 0 && (p[0] < 250 || p[1] < 250 || p[2] < 250);
        let in_clip = (400.0..560.0).contains(&x) && (300.0..460.0).contains(&y);
        let in_corner = x < 240.0 && y < 190.0;
        if inked && in_corner {
            corner += 1;
        } else if inked && !in_clip {
            outside += 1;
        }
    }
    // The disc, shifted 100 px right, has its left edge at
    // x = 380 + 100 = 380? (centre 580, radius 200): inside
    // the clip, x 400..480 must be white-ish only if shifted.
    let at = |x: f32, y: f32| {
        let i = ((y * k) as usize * w + (x * k) as usize) * 4;
        &img.data()[i..i + 4]
    };
    println!(
        "{}: {}x{}, inked px outside clip (excl. corner copy) {outside}, in corner copy {corner}; px at (410,320) {:?} (disc edge moves right if shifted); cache builds {}",
        path.display(),
        img.width(),
        img.height(),
        at(410.0, 320.0),
        1
    );
}
