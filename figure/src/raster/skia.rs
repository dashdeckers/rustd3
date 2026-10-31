//! The tiny-skia fallback: the list lowered to outlines for its glyphs and
//! replayed into a tiny-skia pixmap, path clips through masks.

use kurbo::{Affine, BezPath, PathEl, Point, Rect};
use peniko::color::Srgb;
use peniko::{Brush, Color, Extend, Gradient, GradientKind, ImageData};
use tiny_skia as ts;

use crate::list::{self, Clip, Command, List, Paints};
use crate::lower::{self, Caps};
use crate::pixels;
use crate::raster::{self, Error, Image};

/// What tiny-skia draws natively: everything but glyph runs.
const CAPS: Caps = Caps {
    glyphs: false,
    markers: true,
    clip_paths: true,
    sweep: true,
};

/// The whole of `list` at `scale`, rendered by tiny-skia: round(width ×
/// scale) × round(height × scale) px of straight RGBA, as
/// `raster::render` gives with vello_cpu. Image
/// brushes are [`list::Error::Unsupported`]; a degenerate gradient (one
/// tiny-skia cannot build, such as a linear gradient whose ends coincide)
/// draws nothing.
///
/// # Errors
///
/// [`Error::Empty`] for a list of no device
/// pixel, [`Error::TooLarge`] for one
/// tiny-skia cannot allocate, and [`list::Error`] for a list that fails
/// [`list::check`] or holds an image brush.
pub fn render(list: &List, scale: f64) -> Result<Image, Error> {
    list::check(list)?;
    let area = Rect::from_origin_size(Point::ORIGIN, list.size);
    let (width, height) = raster::device(area, scale)?;
    let lowered = lower::lower(list, CAPS)?.list;
    let mut pixmap = ts::Pixmap::new(u32::from(width), u32::from(height))
        .expect("invariant: a pixmap of 1..=65,408 px a side");
    Replay {
        pixmap: &mut pixmap,
        groups: vec![(Affine::scale(scale), None)],
    }
    .run(&lowered)?;
    let rgba = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect();
    Ok(Image {
        width: u32::from(width),
        height: u32::from(height),
        rgba,
    })
}

/// One render into a pixmap.
struct Replay<'a> {
    pixmap: &'a mut ts::Pixmap,
    /// Each open group's device transform and the clip mask in force.
    groups: Vec<(Affine, Option<ts::Mask>)>,
}

impl Replay<'_> {
    fn run(mut self, list: &List) -> Result<(), Error> {
        for (i, command) in list.commands.iter().enumerate() {
            let (transform, mask) = self
                .groups
                .last()
                .expect("invariant: the root group is never popped");
            let (transform, mask) = (*transform, mask.as_ref());
            match command {
                Command::Fill { path: p, paint } => {
                    if let (Some(p), Some(paint)) = (path(p), shader(i, paint, Affine::IDENTITY)?) {
                        self.pixmap.fill_path(
                            &p,
                            &paint,
                            ts::FillRule::Winding,
                            xf(transform),
                            mask,
                        );
                    }
                }
                Command::Stroke {
                    path: p,
                    style,
                    paint,
                } => {
                    if let (Some(p), Some(paint)) = (path(p), shader(i, paint, Affine::IDENTITY)?) {
                        self.pixmap
                            .stroke_path(&p, &paint, &stroke(style), xf(transform), mask);
                    }
                }
                Command::Markers {
                    shape,
                    at,
                    paint,
                    sizes,
                    paints,
                } => {
                    let Some(shape) = path(shape) else {
                        continue;
                    };
                    for (k, p) in at.iter().enumerate() {
                        let place = Affine::translate(p.to_vec2())
                            * Affine::scale(sizes.as_ref().map_or(1.0, |s| s[k]));
                        let brush = point_paint(paint, paints.as_ref(), k);
                        if let Some(paint) = shader(i, brush, place.inverse())? {
                            self.pixmap.fill_path(
                                &shape,
                                &paint,
                                ts::FillRule::Winding,
                                xf(transform * place),
                                mask,
                            );
                        }
                    }
                }
                Command::Image {
                    rect,
                    image,
                    smooth,
                } => {
                    let Some(source) = pixmap(i, image)? else {
                        continue;
                    };
                    let rect = rect.abs();
                    let place = Affine::translate(rect.origin().to_vec2())
                        * Affine::scale_non_uniform(
                            rect.width() / f64::from(image.width),
                            rect.height() / f64::from(image.height),
                        );
                    let quality = if *smooth {
                        ts::FilterQuality::Bilinear
                    } else {
                        ts::FilterQuality::Nearest
                    };
                    let paint = ts::Paint {
                        shader: ts::Pattern::new(
                            source.as_ref(),
                            ts::SpreadMode::Pad,
                            quality,
                            1.0,
                            xf(place),
                        ),
                        ..ts::Paint::default()
                    };
                    if let Some(r) = ts_rect(rect) {
                        self.pixmap.fill_rect(r, &paint, xf(transform), mask);
                    }
                }
                Command::Push {
                    transform: inner,
                    clip,
                } => {
                    let next = transform * *inner;
                    let mask = match clip {
                        Some(clip) => Some(self.clip(mask, next, clip)),
                        None => mask.cloned(),
                    };
                    self.groups.push((next, mask));
                }
                Command::Pop => {
                    self.groups.pop();
                }
                // Lowered away by `CAPS`.
                Command::Glyphs(_) => {}
            }
        }
        Ok(())
    }

    /// `mask` (or the whole pixmap) intersected with `clip` under
    /// `transform`, anti-aliased.
    fn clip(&self, mask: Option<&ts::Mask>, transform: Affine, clip: &Clip) -> ts::Mask {
        let (width, height) = (self.pixmap.width(), self.pixmap.height());
        let mut mask = mask.cloned().unwrap_or_else(|| {
            let mut full = ts::Mask::new(width, height).expect("invariant: the pixmap's size");
            if let Some(r) = ts::Rect::from_xywh(0.0, 0.0, width as f32, height as f32) {
                full.fill_path(
                    &ts::PathBuilder::from_rect(r),
                    ts::FillRule::Winding,
                    false,
                    ts::Transform::identity(),
                );
            }
            full
        });
        let shape = match clip {
            Clip::Rect(r) => path(&kurbo::Shape::to_path(r, 0.1)),
            Clip::Path(p) => path(p),
        };
        match shape {
            Some(shape) => mask.intersect_path(&shape, ts::FillRule::Winding, true, xf(transform)),
            // A clip with no area hides everything inside it.
            None => mask.clear(),
        }
        mask
    }
}

/// A point's paint: its entry of the table, or the command's paint.
fn point_paint<'a>(paint: &'a Brush, paints: Option<&'a Paints>, k: usize) -> &'a Brush {
    paints.map_or(paint, |p| &p.table[p.index[k] as usize])
}

/// `brush` as a tiny-skia paint positioned by `inverse` (the inverse of
/// the command's own placement, so that paints sit in the group's space);
/// `None` for a gradient tiny-skia cannot build.
fn shader(
    command: usize,
    brush: &Brush,
    inverse: Affine,
) -> Result<Option<ts::Paint<'static>>, Error> {
    let shader = match brush {
        Brush::Solid(color) => Some(ts::Shader::SolidColor(color_of(*color))),
        Brush::Gradient(gradient) => gradient_shader(gradient, xf(inverse)),
        Brush::Image(_) => {
            return Err(list::Error::Unsupported {
                command,
                what: "an image brush",
            }
            .into());
        }
    };
    Ok(shader.map(|shader| ts::Paint {
        shader,
        anti_alias: true,
        ..ts::Paint::default()
    }))
}

fn gradient_shader(gradient: &Gradient, transform: ts::Transform) -> Option<ts::Shader<'static>> {
    let stops = gradient
        .stops
        .iter()
        .map(|s| ts::GradientStop::new(s.offset, color_of(s.color.to_alpha_color::<Srgb>())))
        .collect();
    let mode = match gradient.extend {
        Extend::Pad => ts::SpreadMode::Pad,
        Extend::Repeat => ts::SpreadMode::Repeat,
        Extend::Reflect => ts::SpreadMode::Reflect,
    };
    match gradient.kind {
        GradientKind::Linear(p) => {
            ts::LinearGradient::new(point(p.start), point(p.end), stops, mode, transform)
        }
        GradientKind::Radial(p) => ts::RadialGradient::new(
            point(p.start_center),
            p.start_radius,
            point(p.end_center),
            p.end_radius,
            stops,
            mode,
            transform,
        ),
        // tiny-skia takes degrees; both measure clockwise in y-down.
        GradientKind::Sweep(p) => ts::SweepGradient::new(
            point(p.center),
            p.start_angle.to_degrees(),
            p.end_angle.to_degrees(),
            stops,
            mode,
            transform,
        ),
    }
}

/// The image as a premultiplied pixmap; `None` for an empty one.
fn pixmap(command: usize, image: &ImageData) -> Result<Option<ts::Pixmap>, Error> {
    let Some(size) = ts::IntSize::from_wh(image.width, image.height) else {
        return Ok(None);
    };
    let rgba =
        pixels::premultiplied(image).map_err(|what| list::Error::Unsupported { command, what })?;
    Ok(ts::Pixmap::from_vec(rgba, size))
}

fn stroke(style: &kurbo::Stroke) -> ts::Stroke {
    let mut dashes: Vec<f32> = style.dash_pattern.iter().map(|d| *d as f32).collect();
    // tiny-skia takes an even count; SVG repeats an odd list once.
    if dashes.len() % 2 == 1 {
        dashes.extend_from_within(..);
    }
    ts::Stroke {
        width: style.width as f32,
        miter_limit: style.miter_limit as f32,
        line_cap: match style.start_cap {
            kurbo::Cap::Butt => ts::LineCap::Butt,
            kurbo::Cap::Round => ts::LineCap::Round,
            kurbo::Cap::Square => ts::LineCap::Square,
        },
        line_join: match style.join {
            kurbo::Join::Miter => ts::LineJoin::Miter,
            kurbo::Join::Round => ts::LineJoin::Round,
            kurbo::Join::Bevel => ts::LineJoin::Bevel,
        },
        dash: ts::StrokeDash::new(dashes, style.dash_offset as f32),
    }
}

/// A kurbo path in tiny-skia's f32; `None` when empty.
fn path(p: &BezPath) -> Option<ts::Path> {
    let mut builder = ts::PathBuilder::new();
    for el in p.elements() {
        match *el {
            PathEl::MoveTo(a) => builder.move_to(a.x as f32, a.y as f32),
            PathEl::LineTo(a) => builder.line_to(a.x as f32, a.y as f32),
            PathEl::QuadTo(a, b) => builder.quad_to(a.x as f32, a.y as f32, b.x as f32, b.y as f32),
            PathEl::CurveTo(a, b, c) => builder.cubic_to(
                a.x as f32, a.y as f32, b.x as f32, b.y as f32, c.x as f32, c.y as f32,
            ),
            PathEl::ClosePath => builder.close(),
        }
    }
    builder.finish()
}

fn ts_rect(r: Rect) -> Option<ts::Rect> {
    ts::Rect::from_ltrb(r.x0 as f32, r.y0 as f32, r.x1 as f32, r.y1 as f32)
}

fn point(p: Point) -> ts::Point {
    ts::Point::from_xy(p.x as f32, p.y as f32)
}

fn xf(t: Affine) -> ts::Transform {
    let [a, b, c, d, e, f] = t.as_coeffs().map(|v| v as f32);
    ts::Transform::from_row(a, b, c, d, e, f)
}

fn color_of(color: Color) -> ts::Color {
    let [r, g, b, a] = color.components.map(|c| c.clamp(0.0, 1.0));
    ts::Color::from_rgba(r, g, b, a).unwrap_or(ts::Color::TRANSPARENT)
}
