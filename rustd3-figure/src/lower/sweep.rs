//! Sweep gradients for backends without them: with `raster`, an image of
//! the gradient under a clip to what the command covers; without it, the
//! gradient's middle stop.

#[cfg(feature = "raster")]
use std::sync::Arc;

use kurbo::Affine;
#[cfg(feature = "raster")]
use kurbo::{Rect, Shape, Size};
#[cfg(feature = "raster")]
use peniko::{Blob, ImageAlphaType, ImageData, ImageFormat};
use peniko::{Brush, Gradient, GradientKind};

use crate::list::Command;
#[cfg(feature = "raster")]
use crate::list::{Clip, List};
use crate::lower::Warning;

/// Image px per list px of a rasterized sweep.
#[cfg(feature = "raster")]
const DENSITY: f64 = 2.0;
/// The longest side of a rasterized sweep's image, px.
#[cfg(feature = "raster")]
const MAX_SIDE: f64 = 4096.0;
/// How closely a stroke's outline follows it, list px.
#[cfg(feature = "raster")]
const TOLERANCE: f64 = 0.01;

/// Whether `paint` is a sweep gradient.
pub(super) fn is_sweep(paint: &Brush) -> bool {
    matches!(
        paint,
        Brush::Gradient(Gradient {
            kind: GradientKind::Sweep(_),
            ..
        })
    )
}

/// Whether `command` is a fill or stroke painted with a sweep gradient.
pub(super) fn painted(command: &Command) -> bool {
    match command {
        Command::Fill { paint, .. } | Command::Stroke { paint, .. } => is_sweep(paint),
        Command::Markers { .. }
        | Command::Glyphs(_)
        | Command::Image { .. }
        | Command::Push { .. }
        | Command::Pop => false,
    }
}

/// `command`, a fill or stroke painted by a sweep, as a group clipped to
/// what it covers. The group holds an image of the gradient over that
/// area, rendered at 2 px per list px (at most 4,096 px a side) through
/// `to_list`, the group's transform into list px. A command that covers
/// nothing adds nothing.
#[cfg(feature = "raster")]
pub(super) fn lower(
    command: Command,
    to_list: Affine,
    out: &mut Vec<Command>,
    input: usize,
) -> Option<Warning> {
    let (coverage, paint) = match command {
        Command::Fill { path, paint } => (path, paint),
        Command::Stroke { path, style, paint } => {
            let opts = kurbo::StrokeOpts::default();
            (kurbo::stroke(path.iter(), &style, &opts, TOLERANCE), paint)
        }
        other @ (Command::Markers { .. }
        | Command::Glyphs(_)
        | Command::Image { .. }
        | Command::Push { .. }
        | Command::Pop) => {
            out.push(other);
            return None;
        }
    };
    let area = coverage.bounding_box();
    if area.area().is_nan() || area.area() <= 0.0 {
        return None;
    }
    let image = gradient_image(area, to_list, paint);
    out.extend([
        Command::Push {
            transform: Affine::IDENTITY,
            clip: Some(Clip::Path(coverage)),
        },
        Command::Image {
            rect: area,
            image,
            smooth: true,
        },
        Command::Pop,
    ]);
    Some(Warning::SweepRasterized { command: input })
}

/// `command` with its sweep replaced by the gradient's middle stop.
#[cfg(not(feature = "raster"))]
pub(super) fn lower(
    command: Command,
    _to_list: Affine,
    out: &mut Vec<Command>,
    input: usize,
) -> Option<Warning> {
    out.push(match command {
        Command::Fill { path, paint } => Command::Fill {
            path,
            paint: middle(&paint),
        },
        Command::Stroke { path, style, paint } => Command::Stroke {
            path,
            style,
            paint: middle(&paint),
        },
        other @ (Command::Markers { .. }
        | Command::Glyphs(_)
        | Command::Image { .. }
        | Command::Push { .. }
        | Command::Pop) => {
            out.push(other);
            return None;
        }
    });
    Some(Warning::SweepFlattened { command: input })
}

/// The color of the stop at the middle of a gradient's stops (index
/// len / 2); transparent without stops.
#[cfg(not(feature = "raster"))]
fn middle(paint: &Brush) -> Brush {
    match paint {
        Brush::Gradient(gradient) => gradient
            .stops
            .get(gradient.stops.len() / 2)
            .map_or(Brush::default(), |stop| {
                Brush::Solid(stop.color.to_alpha_color::<peniko::color::Srgb>())
            }),
        Brush::Solid(_) | Brush::Image(_) => paint.clone(),
    }
}

/// `paint` over `area` of its group as straight RGBA pixels, at 2 px per
/// list px of `area` through `to_list`.
#[cfg(feature = "raster")]
fn gradient_image(area: Rect, to_list: Affine, paint: Brush) -> ImageData {
    let shown = to_list.transform_rect_bbox(area);
    let side = |v: f64| (v * DENSITY).ceil().clamp(1.0, MAX_SIDE);
    let (width, height) = (side(shown.width()), side(shown.height()));
    let list = List {
        size: Size::new(width, height),
        commands: vec![
            Command::Push {
                transform: Affine::scale_non_uniform(width / area.width(), height / area.height())
                    * Affine::translate(-area.origin().to_vec2()),
                clip: None,
            },
            Command::Fill {
                path: area.to_path(0.1),
                paint,
            },
            Command::Pop,
        ],
        ..List::default()
    };
    let image = crate::raster::render(&list, 1.0)
        .expect("invariant: a finite gradient over 1..=4096 px renders");
    ImageData {
        data: Blob::new(Arc::new(image.rgba)),
        format: ImageFormat::Rgba8,
        alpha_type: ImageAlphaType::Alpha,
        width: image.width,
        height: image.height,
    }
}
