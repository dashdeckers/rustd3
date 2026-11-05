//! The list as text for snapshots. Numbers go through the SVG writer's
//! formatter, so dumps are byte-stable.

use std::fmt::Write;

use kurbo::{Cap, Join, Shape};
use peniko::{Brush, Extend, Gradient, GradientKind};

use crate::list::{Clip, Command, List};
use crate::svg::number;

/// What a dump shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Options {
    glyphs: bool,
}

impl Options {
    /// Whether runs print their glyph ids, transforms and fills. Without
    /// them (the default), a run prints only its font, size and text, which
    /// suits lists made before typst shapes text.
    #[must_use]
    pub fn glyphs(self, on: bool) -> Self {
        Options { glyphs: on }
    }
}

/// The dump of `list`; see [`list::dump`](crate::list::dump).
pub(super) fn write(list: &List, options: Options) -> String {
    let mut out = format!(
        "size {}x{} fonts {} parts {}\n",
        number::px(list.size.width),
        number::px(list.size.height),
        list.fonts.len(),
        list.parts.len()
    );
    for (i, command) in list.commands.iter().enumerate() {
        let line = match command {
            Command::Fill { path, paint } => {
                format!("fill {} {}", number::path(path, number::px), brush(paint))
            }
            Command::Stroke { path, style, paint } => format!(
                "stroke {} {} {}",
                number::path(path, number::px),
                stroke(style),
                brush(paint)
            ),
            Command::Markers {
                shape,
                at,
                paint,
                sizes,
                paints,
            } => {
                let b = shape.bounding_box();
                let mut line = format!(
                    "markers n{} bbox({}) {}",
                    at.len(),
                    [b.x0, b.y0, b.x1, b.y1].map(number::unit).join(","),
                    brush(paint)
                );
                if let Some(sizes) = sizes {
                    let sizes: Vec<String> = sizes.iter().map(|s| number::unit(*s)).collect();
                    let _ = write!(line, " sizes[{}]", sizes.join(","));
                }
                if let Some(paints) = paints {
                    let index: Vec<String> = paints.index.iter().map(u32::to_string).collect();
                    let _ = write!(line, " paints[{}:{}]", paints.table.len(), index.join(","));
                }
                line
            }
            Command::Glyphs(run) => {
                let mut line = format!(
                    "glyphs font{} {} {:?}",
                    run.font,
                    number::unit(f64::from(run.size)),
                    run.text
                );
                if options.glyphs {
                    let ids: Vec<String> = run.glyphs.iter().map(|g| g.id.to_string()).collect();
                    let _ = write!(
                        line,
                        " ids[{}] {} {}",
                        ids.join(","),
                        number::matrix(run.transform),
                        brush(&run.fill)
                    );
                }
                line
            }
            Command::Image {
                rect,
                image,
                smooth,
            } => format!(
                "image rect({}) {}x{} {}",
                [rect.x0, rect.y0, rect.x1, rect.y1]
                    .map(number::px)
                    .join(","),
                image.width,
                image.height,
                if *smooth { "smooth" } else { "nearest" }
            ),
            Command::Push { transform, clip } => {
                let clip = match clip {
                    Some(Clip::Rect(r)) => {
                        format!(
                            " rect({})",
                            [r.x0, r.y0, r.x1, r.y1].map(number::px).join(",")
                        )
                    }
                    Some(Clip::Path(p)) => format!(" path({})", number::path(p, number::px)),
                    None => String::new(),
                };
                format!("push {}{clip}", number::matrix(*transform))
            }
            Command::Pop => "pop".to_owned(),
        };
        let _ = writeln!(out, "{i} {line}");
    }
    for (i, part) in list.parts.iter().enumerate() {
        let b = part.bounds;
        let _ = writeln!(
            out,
            "part {i} {}..{} bounds({})",
            part.commands.start,
            part.commands.end,
            [b.x0, b.y0, b.x1, b.y1].map(number::px).join(",")
        );
    }
    out
}

/// Width, caps, join and dashes: `w1.5 butt miter 4 dash[6,3]@0`.
fn stroke(style: &kurbo::Stroke) -> String {
    let cap = |cap: Cap| match cap {
        Cap::Butt => "butt",
        Cap::Round => "round",
        Cap::Square => "square",
    };
    let caps = if style.start_cap == style.end_cap {
        cap(style.start_cap).to_owned()
    } else {
        format!("{}/{}", cap(style.start_cap), cap(style.end_cap))
    };
    let join = match style.join {
        Join::Miter => format!("miter {}", number::unit(style.miter_limit)),
        Join::Round => "round".to_owned(),
        Join::Bevel => "bevel".to_owned(),
    };
    let mut s = format!("w{} {caps} {join}", number::unit(style.width));
    if !style.dash_pattern.is_empty() {
        let dashes: Vec<String> = style
            .dash_pattern
            .iter()
            .map(|d| number::unit(*d))
            .collect();
        let _ = write!(
            s,
            " dash[{}]@{}",
            dashes.join(","),
            number::unit(style.dash_offset)
        );
    }
    s
}

/// `#rrggbb`, `#rrggbbaa` when not opaque; a gradient's geometry, stops
/// and extend; an image brush's size.
fn brush(paint: &Brush) -> String {
    match paint {
        Brush::Solid(color) => hex(color.to_rgba8().to_u8_array()),
        Brush::Gradient(gradient) => gradient_text(gradient),
        Brush::Image(image) => format!("image({}x{})", image.image.width, image.image.height),
    }
}

fn gradient_text(gradient: &Gradient) -> String {
    let numbers = |values: &[f64]| -> String {
        values
            .iter()
            .map(|v| number::unit(*v))
            .collect::<Vec<_>>()
            .join(",")
    };
    let mut s = match gradient.kind {
        GradientKind::Linear(p) => format!(
            "linear({})",
            numbers(&[p.start.x, p.start.y, p.end.x, p.end.y])
        ),
        GradientKind::Radial(p) => format!(
            "radial({})",
            numbers(&[
                p.start_center.x,
                p.start_center.y,
                f64::from(p.start_radius),
                p.end_center.x,
                p.end_center.y,
                f64::from(p.end_radius),
            ])
        ),
        GradientKind::Sweep(p) => format!(
            "sweep({})",
            numbers(&[
                p.center.x,
                p.center.y,
                f64::from(p.start_angle),
                f64::from(p.end_angle),
            ])
        ),
    };
    let stops: Vec<String> = gradient
        .stops
        .iter()
        .map(|stop| {
            let color = stop.color.to_alpha_color::<peniko::color::Srgb>();
            format!(
                "{} {}",
                number::unit(f64::from(stop.offset)),
                hex(color.to_rgba8().to_u8_array())
            )
        })
        .collect();
    let _ = write!(s, "[{}]", stops.join(","));
    match gradient.extend {
        Extend::Pad => {}
        Extend::Repeat => s.push_str(" repeat"),
        Extend::Reflect => s.push_str(" reflect"),
    }
    s
}

fn hex([r, g, b, a]: [u8; 4]) -> String {
    if a == u8::MAX {
        format!("#{r:02x}{g:02x}{b:02x}")
    } else {
        format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
    }
}
