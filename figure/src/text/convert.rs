//! A typst page frame as glyph runs and fills: positions in px, y down,
//! relative to the first text item's baseline.

use std::sync::Arc;

use kurbo::{Affine, BezPath, Cap, Join, StrokeOpts};
use peniko::{Brush, Color};
use typst::layout::{Frame, FrameItem, Transform};
use typst::text::TextItem;
use typst::visualize::{self, CurveItem, FixedStroke, Geometry, LineCap, LineJoin, Paint};

use crate::list::{Glyph, GlyphRun};
use crate::text::Fonts;
use crate::text::shaped::{Item, Shaped};

/// Stroke outlines of typst's shapes are flattened to within this, px.
const TOLERANCE: f64 = 0.01;

/// The page `frame` as a shaped label. Text and shapes typst paints with
/// a gradient or tiling take `fill`. The box is the frame's: its width is
/// the advance width, its height the ink's under `"bounds"` text edges. The
/// ascent is the baseline of the first frame that has one, the label's
/// `box`, which typst sets at its first line's baseline (the page frame's
/// own baseline is unset and would be the height); a label that starts
/// with a fraction or a radical has its first text item well above it.
pub(crate) fn page(fonts: &Arc<Fonts>, frame: &Frame, fill: Color) -> Shaped {
    let mut walk = Walk {
        fonts,
        fill,
        items: Vec::new(),
        ascent: None,
        missing: 0,
    };
    walk.frame(frame, Affine::IDENTITY);
    let height = frame.height().to_pt();
    let ascent = walk.ascent.unwrap_or(height);
    let shift = Affine::translate((0.0, -ascent));
    let items: Vec<Item> = walk
        .items
        .into_iter()
        .map(|item| match item {
            Item::Run(run) => Item::Run(GlyphRun {
                transform: shift * run.transform,
                ..run
            }),
            Item::Fill { path, paint } => Item::Fill {
                path: shift * path,
                paint,
            },
        })
        .collect();
    let last = last(&items);
    Shaped::new(
        fonts.clone(),
        items,
        [frame.width().to_pt(), ascent, height - ascent, last],
        walk.missing,
    )
}

/// The last line's baseline below the first: the lowest origin among the
/// runs set at the label's largest size (scripts are smaller), 0 for one
/// line.
fn last(items: &[Item]) -> f64 {
    let runs = || {
        items.iter().filter_map(|item| match item {
            Item::Run(run) => Some(run),
            Item::Fill { .. } => None,
        })
    };
    let size = runs().map(|run| run.size).fold(0.0, f32::max);
    runs()
        .filter(|run| run.size == size)
        .map(|run| run.transform.translation().y)
        .fold(0.0, f64::max)
}

/// What a walk over a frame has found so far.
struct Walk<'a> {
    fonts: &'a Fonts,
    fill: Color,
    items: Vec<Item>,
    /// The first baseline a frame has.
    ascent: Option<f64>,
    /// Glyphs with id 0.
    missing: usize,
}

impl Walk<'_> {
    /// `frame`'s items, placed by `at`.
    fn frame(&mut self, frame: &Frame, at: Affine) {
        for (pos, item) in frame.items() {
            let here = at * Affine::translate((pos.x.to_pt(), pos.y.to_pt()));
            match item {
                FrameItem::Group(group) => {
                    let inner = here * affine(group.transform);
                    if group.frame.has_baseline() {
                        let baseline =
                            inner * kurbo::Point::new(0.0, group.frame.baseline().to_pt());
                        self.ascent.get_or_insert(baseline.y);
                    }
                    self.frame(&group.frame, inner);
                }
                FrameItem::Text(text) => self.text(text, here),
                FrameItem::Shape(shape, _) => self.shape(shape, here),
                FrameItem::Image(..) | FrameItem::Link(..) | FrameItem::Tag(_) => {}
            }
        }
    }

    /// One run; glyph positions from the pen, advances and offsets.
    fn text(&mut self, text: &TextItem, at: Affine) {
        let size = text.size;
        let mut pen = 0.0;
        let glyphs = text
            .glyphs
            .iter()
            .map(|g| {
                let advance = g.x_advance.at(size).to_pt();
                let glyph = Glyph {
                    id: u32::from(g.id),
                    x: (pen + g.x_offset.at(size).to_pt()) as f32,
                    y: -g.y_offset.at(size).to_pt() as f32,
                    advance: advance as f32,
                    text: u32::from(g.range.start)..u32::from(g.range.end),
                };
                pen += advance;
                glyph
            })
            .collect();
        self.missing += text.glyphs.iter().filter(|g| g.id == 0).count();
        let font = self
            .fonts
            .index(text.font.font())
            .expect("invariant: typst shapes only with the faces its world serves");
        self.items.push(Item::Run(GlyphRun {
            font,
            size: size.to_pt() as f32,
            transform: at,
            fill: self.paint(&text.fill),
            glyphs,
            text: Arc::from(text.text.as_str()),
        }));
    }

    /// A shape's fill and its stroke's outline, each as a fill.
    fn shape(&mut self, shape: &visualize::Shape, at: Affine) {
        let path = geometry(&shape.geometry);
        if let Some(fill) = &shape.fill {
            self.fill(at * path.clone(), fill);
        }
        if let Some(stroke) = &shape.stroke {
            let outline = kurbo::stroke(path, &style(stroke), &StrokeOpts::default(), TOLERANCE);
            self.fill(at * outline, &stroke.paint);
        }
    }

    fn fill(&mut self, path: BezPath, paint: &Paint) {
        let paint = self.paint(paint);
        self.items.push(Item::Fill { path, paint });
    }

    /// A solid paint's color in 8-bit sRGB, as every output draws it;
    /// any other paint is the label's fill.
    fn paint(&self, paint: &Paint) -> Brush {
        let color = match paint {
            Paint::Solid(color) => {
                let (r, g, b, a) = color.to_rgb().into_components();
                let [r, g, b, a] = [r, g, b, a].map(|c| (c * 255.0).round() as u8);
                Color::from_rgba8(r, g, b, a)
            }
            Paint::Gradient(_) | Paint::Tiling(_) => self.fill,
        };
        Brush::Solid(color)
    }
}

/// typst's `Transform` as an affine map.
fn affine(t: Transform) -> Affine {
    Affine::new([
        t.sx.get(),
        t.ky.get(),
        t.kx.get(),
        t.sy.get(),
        t.tx.to_pt(),
        t.ty.to_pt(),
    ])
}

/// A shape's geometry as a path in its own space.
fn geometry(geometry: &Geometry) -> BezPath {
    match geometry {
        Geometry::Line(to) => {
            let mut path = BezPath::new();
            path.move_to((0.0, 0.0));
            path.line_to((to.x.to_pt(), to.y.to_pt()));
            path
        }
        Geometry::Rect(size) => kurbo::Shape::to_path(
            &kurbo::Rect::new(0.0, 0.0, size.x.to_pt(), size.y.to_pt()),
            TOLERANCE,
        ),
        Geometry::Curve(curve) => {
            let p = |p: &typst::layout::Point| (p.x.to_pt(), p.y.to_pt());
            let mut path = BezPath::new();
            for item in &curve.0 {
                match item {
                    CurveItem::Move(a) => path.move_to(p(a)),
                    CurveItem::Line(a) => path.line_to(p(a)),
                    CurveItem::Cubic(a, b, c) => path.curve_to(p(a), p(b), p(c)),
                    CurveItem::Close => path.close_path(),
                }
            }
            path
        }
    }
}

/// typst's stroke as kurbo's.
fn style(stroke: &FixedStroke) -> kurbo::Stroke {
    let cap = match stroke.cap {
        LineCap::Butt => Cap::Butt,
        LineCap::Round => Cap::Round,
        LineCap::Square => Cap::Square,
    };
    let join = match stroke.join {
        LineJoin::Miter => Join::Miter,
        LineJoin::Round => Join::Round,
        LineJoin::Bevel => Join::Bevel,
    };
    let style = kurbo::Stroke::new(stroke.thickness.to_pt())
        .with_caps(cap)
        .with_join(join)
        .with_miter_limit(stroke.miter_limit.get());
    match &stroke.dash {
        Some(dash) => style.with_dashes(dash.phase.to_pt(), dash.array.iter().map(|d| d.to_pt())),
        None => style,
    }
}
