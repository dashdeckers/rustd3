//! The drawing list: the figure layer's only output.
//! Geometry is kurbo's and paint is peniko's, so the
//! vello family reads it without conversion. Text
//! leaves the figure as `Label`s here; a shaping pass
//! (typst) turns them into `GlyphRun`s before any
//! backend sees the list.

use super::text::Text;
use d3::color::Rgb;
use d3::path::PathContext;
use kurbo::{Affine, BezPath, Point, Shape, Stroke};
use peniko::{Brush, Color, FontData};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Anchor {
    Start,
    Middle,
    End,
}

/// Unshaped text, as the figure layer places it.
#[derive(Clone, Debug)]
pub struct Label {
    pub text: Text,
    /// The baseline point the anchor sits on.
    pub at: Point,
    pub anchor: Anchor,
    /// Radians about `at`, clockwise on screen.
    pub angle: f64,
    pub size: f32,
    pub bold: bool,
    pub fill: Color,
}

/// One positioned glyph of a run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    pub id: u32,
    /// Offset from the run's origin, in px.
    pub x: f32,
    pub y: f32,
    pub advance: f32,
    /// Byte range of `GlyphRun::text` it shows.
    pub text: (u32, u32),
}

/// Shaped text: what backends draw.
#[derive(Clone, Debug)]
pub struct GlyphRun {
    /// Index into `List::fonts`.
    pub font: usize,
    pub size: f32,
    /// Places the run's origin (start of baseline).
    pub transform: Affine,
    pub fill: Color,
    pub glyphs: Vec<Glyph>,
    /// The source text, for SVG and PDF text.
    pub text: Arc<str>,
}

#[derive(Clone, Debug)]
pub enum Command {
    Fill {
        path: BezPath,
        paint: Brush,
    },
    Stroke {
        path: BezPath,
        style: Stroke,
        paint: Brush,
    },
    /// One shape stamped at many points.
    Markers {
        shape: BezPath,
        at: Vec<Point>,
        paint: Brush,
    },
    Label(Label),
    Glyphs(GlyphRun),
    /// Opens a group; `clip` is in the group's
    /// coordinates, after `transform`.
    Push {
        transform: Affine,
        clip: Option<BezPath>,
    },
    Pop,
}

#[derive(Clone, Debug, Default)]
pub struct List {
    pub size: [f64; 2],
    pub fonts: Vec<FontData>,
    pub commands: Vec<Command>,
    /// Emit dots one Fill each instead of Markers.
    pub no_markers: bool,
}

pub fn color(c: Rgb) -> Color {
    Color::new([
        (c.r / 255.0) as f32,
        (c.g / 255.0) as f32,
        (c.b / 255.0) as f32,
        c.opacity as f32,
    ])
}

impl List {
    pub fn new(size: [f64; 2]) -> Self {
        Self { size, ..Self::default() }
    }

    pub fn fill(&mut self, s: impl Shape, c: Rgb) {
        let path = s.into_path(0.1);
        let paint = Brush::Solid(color(c));
        self.commands.push(Command::Fill { path, paint });
    }

    pub fn stroke(
        &mut self,
        s: impl Shape,
        width: f64,
        dash: Option<[f64; 2]>,
        c: Rgb,
    ) {
        // kurbo defaults to round caps and joins; SVG,
        // tiny-skia and PDF default to butt and miter.
        let mut style = Stroke::new(width)
            .with_caps(kurbo::Cap::Butt)
            .with_join(kurbo::Join::Miter);
        if let Some(d) = dash {
            style = style.with_dashes(0.0, d);
        }
        self.commands.push(Command::Stroke {
            path: s.into_path(0.1),
            style,
            paint: Brush::Solid(color(c)),
        });
    }

    pub fn markers(
        &mut self,
        shape: impl Shape,
        at: Vec<Point>,
        c: Rgb,
    ) {
        if self.no_markers {
            for p in at {
                let t = Affine::translate(p.to_vec2());
                self.fill(t * shape.to_path(0.1), c);
            }
            return;
        }
        self.commands.push(Command::Markers {
            shape: shape.into_path(0.1),
            at,
            paint: Brush::Solid(color(c)),
        });
    }

    pub fn label(&mut self, l: Label) {
        self.commands.push(Command::Label(l));
    }

    pub fn push(&mut self, transform: Affine) {
        let clip = None;
        self.commands.push(Command::Push { transform, clip });
    }

    pub fn push_clip(&mut self, clip: impl Shape) {
        self.commands.push(Command::Push {
            transform: Affine::IDENTITY,
            clip: Some(clip.into_path(0.1)),
        });
    }

    pub fn pop(&mut self) {
        self.commands.push(Command::Pop);
    }
}

impl Label {
    pub fn new(text: impl Into<Text>, x: f64, y: f64) -> Self {
        Label {
            text: text.into(),
            at: Point::new(x, y),
            anchor: Anchor::Start,
            angle: 0.0,
            size: 10.0,
            bold: false,
            fill: Color::BLACK,
        }
    }

    pub fn anchor(self, anchor: Anchor) -> Self {
        Self { anchor, ..self }
    }

    pub fn size(self, size: f32) -> Self {
        Self { size, ..self }
    }

    pub fn bold(self) -> Self {
        Self { bold: true, ..self }
    }

    pub fn fill(self, c: Rgb) -> Self {
        Self { fill: color(c), ..self }
    }

    pub fn angle(self, angle: f64) -> Self {
        Self { angle, ..self }
    }
}

/// d3's path drawing, recorded into a kurbo path.
#[derive(Default)]
pub struct Pen(pub BezPath);

impl PathContext for Pen {
    fn move_to(&mut self, x: f64, y: f64) {
        self.0.move_to((x, y));
    }

    fn line_to(&mut self, x: f64, y: f64) {
        self.0.line_to((x, y));
    }

    fn quadratic_curve_to(
        &mut self,
        x1: f64,
        y1: f64,
        x: f64,
        y: f64,
    ) {
        self.0.quad_to((x1, y1), (x, y));
    }

    fn bezier_curve_to(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x: f64,
        y: f64,
    ) {
        self.0.curve_to((x1, y1), (x2, y2), (x, y));
    }

    fn arc(
        &mut self,
        x: f64,
        y: f64,
        r: f64,
        a0: f64,
        a1: f64,
        ccw: bool,
    ) {
        use std::f64::consts::TAU;
        let start = (x + r * a0.cos(), y + r * a0.sin());
        if self.0.elements().is_empty() {
            self.0.move_to(start);
        } else {
            self.0.line_to(start);
        }
        let mut sweep = a1 - a0;
        if ccw {
            sweep = -(-sweep).rem_euclid(TAU);
        } else {
            sweep = sweep.rem_euclid(TAU);
        }
        if (a1 - a0).abs() >= TAU {
            sweep = if ccw { -TAU } else { TAU };
        }
        let arc = kurbo::Arc::new((x, y), (r, r), a0, sweep, 0.0);
        arc.to_cubic_beziers(0.1, |p1, p2, p| {
            self.0.curve_to(p1, p2, p);
        });
    }

    fn close_path(&mut self) {
        self.0.close_path();
    }
}

/// Counts by command kind, for the RFC's tables.
pub fn census(l: &List) -> Vec<(&'static str, usize)> {
    let mut v: Vec<(&'static str, usize)> = Vec::new();
    for c in &l.commands {
        let k = match c {
            Command::Fill { .. } => "Fill",
            Command::Stroke { .. } => "Stroke",
            Command::Markers { .. } => "Markers",
            Command::Label(_) => "Label",
            Command::Glyphs(_) => "Glyphs",
            Command::Push { clip: Some(_), .. } => "Push+clip",
            Command::Push { .. } => "Push",
            Command::Pop => "Pop",
        };
        match v.iter_mut().find(|(n, _)| *n == k) {
            Some(e) => e.1 += 1,
            None => v.push((k, 1)),
        }
    }
    v
}

/// `#rrggbb` and an opacity attribute, if not opaque.
pub fn svg_paint(attr: &str, p: &Brush) -> String {
    let Brush::Solid(c) = p else {
        return format!(" {attr}='gray'");
    };
    let [r, g, b, a] = c.to_rgba8().to_u8_array();
    let mut s = format!(" {attr}='#{r:02x}{g:02x}{b:02x}'");
    if a < 255 {
        let o = c.components[3];
        s += &format!(" {attr}-opacity='{o:.2}'");
    }
    s
}

/// Path data with two decimals.
pub fn svg_d(p: &BezPath) -> String {
    use kurbo::PathEl::*;
    use std::fmt::Write;
    let mut d = String::new();
    for e in p.elements() {
        let _ = match e {
            MoveTo(a) => write!(d, "M{:.2},{:.2}", a.x, a.y),
            LineTo(a) => write!(d, "L{:.2},{:.2}", a.x, a.y),
            QuadTo(a, b) => write!(
                d,
                "Q{:.2},{:.2},{:.2},{:.2}",
                a.x, a.y, b.x, b.y
            ),
            CurveTo(a, b, c) => write!(
                d,
                "C{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}",
                a.x, a.y, b.x, b.y, c.x, c.y
            ),
            ClosePath => write!(d, "Z"),
        };
    }
    d
}

pub fn svg_stroke(s: &Stroke) -> String {
    let mut o = format!(" stroke-width='{}'", s.width);
    let cap = match s.start_cap {
        kurbo::Cap::Butt => "",
        kurbo::Cap::Round => " stroke-linecap='round'",
        kurbo::Cap::Square => " stroke-linecap='square'",
    };
    let join = match s.join {
        kurbo::Join::Miter => "",
        kurbo::Join::Round => " stroke-linejoin='round'",
        kurbo::Join::Bevel => " stroke-linejoin='bevel'",
    };
    o += cap;
    o += join;
    if !s.dash_pattern.is_empty() {
        let d: Vec<String> =
            s.dash_pattern.iter().map(|v| v.to_string()).collect();
        o += &format!(" stroke-dasharray='{}'", d.join(" "));
    }
    o
}

pub fn svg_matrix(t: &Affine) -> String {
    let [a, b, c, d, e, f] = t.as_coeffs();
    format!("matrix({a} {b} {c} {d} {e} {f})")
}

impl List {
    /// SVG with text as `<text>` elements: no fonts
    /// needed here, but the viewer shapes the text
    /// again with whatever fonts it has.
    pub fn svg(&self) -> String {
        self.svg_custom("", text_run)
    }

    /// SVG with `defs` up front and glyph runs written
    /// by `glyphs`.
    pub fn svg_custom(
        &self,
        defs: &str,
        mut glyphs: impl FnMut(&GlyphRun, &mut String),
    ) -> String {
        use std::fmt::Write;
        let [w, h] = self.size;
        let mut o = format!(
            "<svg xmlns='http://www.w3.org/2000/svg' \
             xmlns:xlink='http://www.w3.org/1999/xlink' \
             width='{w}' height='{h}' \
             font-family='DejaVu Sans' font-size='10'>{defs}"
        );
        let mut open: Vec<u8> = Vec::new();
        let mut ids = 0;
        for c in &self.commands {
            match c {
                Command::Fill { path, paint } => {
                    let _ = write!(
                        o,
                        "<path d='{}'{}/>",
                        svg_d(path),
                        svg_paint("fill", paint)
                    );
                }
                Command::Stroke { path, style, paint } => {
                    let _ = write!(
                        o,
                        "<path d='{}' fill='none'{}{}/>",
                        svg_d(path),
                        svg_paint("stroke", paint),
                        svg_stroke(style)
                    );
                }
                Command::Markers { shape, at, paint } => {
                    ids += 1;
                    let _ = write!(
                        o,
                        "<defs><path id='m{ids}' d='{}'/></defs>\
                         <g{}>",
                        svg_d(shape),
                        svg_paint("fill", paint)
                    );
                    for p in at {
                        let _ = write!(
                            o,
                            "<use xlink:href='#m{ids}' \
                             x='{:.1}' y='{:.1}'/>",
                            p.x, p.y
                        );
                    }
                    o += "</g>";
                }
                Command::Label(l) => {
                    let anchor = match l.anchor {
                        Anchor::Start => "start",
                        Anchor::Middle => "middle",
                        Anchor::End => "end",
                    };
                    let [r, g, b, _] = l.fill.to_rgba8().to_u8_array();
                    let rot = if l.angle != 0.0 {
                        format!(
                            " transform='rotate({} {} {})'",
                            l.angle.to_degrees(),
                            l.at.x,
                            l.at.y
                        )
                    } else {
                        String::new()
                    };
                    let bold = if l.bold {
                        " font-weight='bold'"
                    } else {
                        ""
                    };
                    let _ = write!(
                        o,
                        "<text x='{:.2}' y='{:.2}' \
                         text-anchor='{anchor}' \
                         font-size='{}' \
                         fill='#{r:02x}{g:02x}{b:02x}'\
                         {bold}{rot}>{}</text>",
                        l.at.x,
                        l.at.y,
                        l.size,
                        l.text.svg()
                    );
                }
                Command::Glyphs(g) => glyphs(g, &mut o),
                Command::Push { transform, clip } => {
                    let _ = write!(
                        o,
                        "<g transform='{}'>",
                        svg_matrix(transform)
                    );
                    let mut n = 1;
                    if let Some(p) = clip {
                        ids += 1;
                        let _ = write!(
                            o,
                            "<clipPath id='c{ids}'>\
                             <path d='{}'/></clipPath>\
                             <g clip-path='url(#c{ids})'>",
                            svg_d(p)
                        );
                        n = 2;
                    }
                    open.push(n);
                }
                Command::Pop => {
                    for _ in 0..open.pop().unwrap_or(0) {
                        o += "</g>";
                    }
                }
            }
        }
        o + "</svg>"
    }
}

/// A run as `<text>`, one x per glyph: exact only when
/// glyphs map 1:1 to characters.
pub fn text_run(g: &GlyphRun, o: &mut String) {
    use std::fmt::Write;
    let xs: Vec<String> =
        g.glyphs.iter().map(|q| format!("{:.2}", q.x)).collect();
    let [r, gg, b, _] = g.fill.to_rgba8().to_u8_array();
    let _ = write!(
        o,
        "<text transform='{}' x='{}' y='0' font-size='{}' \
         fill='#{r:02x}{gg:02x}{b:02x}'>{}</text>",
        svg_matrix(&g.transform),
        xs.join(" "),
        g.size,
        xml(&g.text)
    );
}

pub fn xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
