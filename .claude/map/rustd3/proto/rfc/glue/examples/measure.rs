//! Can the glue measure label text with iced's own text stack, headless?
use iced::advanced::text::{self, Paragraph as _, Text};
use iced::{Font, Pixels, Size};

type Paragraph = <iced::Renderer as text::Renderer>::Paragraph;

fn measure(label: &str, size: f32) -> Size {
    Paragraph::with_text(Text {
        content: label,
        bounds: Size::INFINITE,
        size: Pixels(size),
        line_height: text::LineHeight::default(),
        font: Font::DEFAULT,
        align_x: text::Alignment::Default,
        align_y: iced::alignment::Vertical::Top,
        shaping: text::Shaping::Advanced,
        wrapping: text::Wrapping::None,
        ellipsis: text::Ellipsis::default(),
        hint_factor: None,
    })
    .min_bounds()
}

fn main() {
    let _ = measure("warm up the font system", 11.0);
    let t = std::time::Instant::now();
    let mut w = 0.0;
    for k in 0..20 {
        for i in 0..40 {
            w += measure(&format!("{:.1}", 7000.0 + k as f64 * 9.1 + i as f64 * 0.1), 11.0).width;
        }
    }
    println!("cosmic-text via iced Paragraph: 40 new labels in {:?} (avg of 20, total width {w:.0})", t.elapsed() / 20);
    for label in ["0", "10", "1,000,000.5", "Oct 25", "03 AM", "Beschleunigung [m/s²]", "مرحبا", "数据"] {
        let s = measure(label, 11.0);
        println!("{label:24} {:6.2} x {:5.2} px", s.width, s.height);
    }
}
