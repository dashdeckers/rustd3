//! [`check`]: the validation every backend runs before drawing a list.

use kurbo::{BezPath, PathEl, Point, Rect};
use peniko::{Brush, Gradient, GradientKind, ImageData};

use crate::list::{Clip, Command, Error, List, Paints};

/// Checks that every backend can draw `list`: Push and Pop balanced; glyph
/// fonts inside the font table; markers channels as long as their points,
/// with paint indices inside the table; image data of width × height × 4
/// bytes; parts ordered, non-overlapping, balanced and inside the commands
/// (any `hash` is accepted); every coordinate finite.
///
/// # Errors
///
/// The first [`Error`] found, in command order, then part order.
pub fn check(list: &List) -> Result<(), Error> {
    let mut open = 0_usize;
    for (command, c) in list.commands.iter().enumerate() {
        match c {
            Command::Fill { path, paint } => {
                finite_path(command, path)?;
                brush(command, paint)?;
            }
            Command::Stroke { path, style, paint } => {
                finite_path(command, path)?;
                let values = [style.width, style.miter_limit, style.dash_offset];
                finite(
                    command,
                    values.into_iter().chain(style.dash_pattern.iter().copied()),
                )?;
                brush(command, paint)?;
            }
            Command::Markers {
                shape,
                at,
                paint,
                sizes,
                paints,
            } => {
                finite_path(command, shape)?;
                finite(command, at.iter().flat_map(|p| [p.x, p.y]))?;
                brush(command, paint)?;
                if let Some(sizes) = sizes {
                    length(command, at.len(), sizes.len())?;
                    finite(command, sizes.iter().copied())?;
                }
                if let Some(paints) = paints {
                    per_point(command, at.len(), paints)?;
                }
            }
            Command::Glyphs(run) => {
                if run.font >= list.fonts.len() {
                    return Err(Error::Font {
                        command,
                        font: run.font,
                        fonts: list.fonts.len(),
                    });
                }
                finite(command, [f64::from(run.size)])?;
                finite(command, run.transform.as_coeffs())?;
                finite(
                    command,
                    run.glyphs
                        .iter()
                        .flat_map(|g| [g.x, g.y, g.advance].map(f64::from)),
                )?;
                brush(command, &run.fill)?;
            }
            Command::Image { rect, image, .. } => {
                finite_rect(command, *rect)?;
                image_length(command, image)?;
            }
            Command::Push { transform, clip } => {
                finite(command, transform.as_coeffs())?;
                match clip {
                    Some(Clip::Rect(rect)) => finite_rect(command, *rect)?,
                    Some(Clip::Path(path)) => finite_path(command, path)?,
                    None => {}
                }
                open += 1;
            }
            Command::Pop => {
                open = open.checked_sub(1).ok_or(Error::UnmatchedPop { command })?;
            }
        }
    }
    if open > 0 {
        return Err(Error::Unbalanced { open });
    }
    parts(list)
}

/// The parts in order, inside the commands, each balanced.
fn parts(list: &List) -> Result<(), Error> {
    let len = list.commands.len();
    let mut previous = 0;
    for (part, p) in list.parts.iter().enumerate() {
        let (start, end) = (p.commands.start, p.commands.end);
        if start > end || end > len {
            return Err(Error::PartRange {
                part,
                start,
                end,
                len,
            });
        }
        if start < previous {
            return Err(Error::PartOrder {
                part,
                start,
                previous,
            });
        }
        let mut open = 0_usize;
        for (command, c) in list.commands[start..end].iter().enumerate() {
            let command = start + command;
            match c {
                Command::Push { .. } => open += 1,
                Command::Pop => {
                    open = open
                        .checked_sub(1)
                        .ok_or(Error::PartUnbalanced { part, command })?;
                }
                Command::Fill { .. }
                | Command::Stroke { .. }
                | Command::Markers { .. }
                | Command::Glyphs(_)
                | Command::Image { .. } => {}
            }
        }
        if open > 0 {
            return Err(Error::PartUnbalanced { part, command: end });
        }
        previous = end;
    }
    Ok(())
}

/// The first non-finite value, as an error.
fn finite(command: usize, values: impl IntoIterator<Item = f64>) -> Result<(), Error> {
    match values.into_iter().find(|v| !v.is_finite()) {
        Some(value) => Err(Error::NonFinite { command, value }),
        None => Ok(()),
    }
}

fn finite_rect(command: usize, rect: Rect) -> Result<(), Error> {
    finite(command, [rect.x0, rect.y0, rect.x1, rect.y1])
}

fn finite_path(command: usize, path: &BezPath) -> Result<(), Error> {
    finite(
        command,
        path.elements()
            .iter()
            .flat_map(|el| points(*el))
            .flatten()
            .flat_map(|p| [p.x, p.y]),
    )
}

/// The points of a path element.
fn points(el: PathEl) -> [Option<Point>; 3] {
    match el {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => [Some(p), None, None],
        PathEl::QuadTo(a, b) => [Some(a), Some(b), None],
        PathEl::CurveTo(a, b, c) => [Some(a), Some(b), Some(c)],
        PathEl::ClosePath => [None; 3],
    }
}

/// A paint's geometry finite and its image, if any, of the right length.
fn brush(command: usize, paint: &Brush) -> Result<(), Error> {
    match paint {
        Brush::Solid(color) => finite(command, color.components.map(f64::from)),
        Brush::Gradient(gradient) => finite(command, gradient_values(gradient)),
        Brush::Image(image) => {
            finite(command, [f64::from(image.sampler.alpha)])?;
            image_length(command, &image.image)
        }
    }
}

/// Every number that places a gradient: positions, radii, angles, stops.
fn gradient_values(gradient: &Gradient) -> Vec<f64> {
    let mut values = match gradient.kind {
        GradientKind::Linear(p) => vec![p.start.x, p.start.y, p.end.x, p.end.y],
        GradientKind::Radial(p) => vec![
            p.start_center.x,
            p.start_center.y,
            f64::from(p.start_radius),
            p.end_center.x,
            p.end_center.y,
            f64::from(p.end_radius),
        ],
        GradientKind::Sweep(p) => vec![
            p.center.x,
            p.center.y,
            f64::from(p.start_angle),
            f64::from(p.end_angle),
        ],
    };
    for stop in gradient.stops.iter() {
        values.push(f64::from(stop.offset));
        values.extend(stop.color.components.map(f64::from));
    }
    values
}

fn image_length(command: usize, image: &ImageData) -> Result<(), Error> {
    let expected = u64::from(image.width)
        .saturating_mul(u64::from(image.height))
        .saturating_mul(4);
    let found = image.data.len();
    if u64::try_from(found).ok() == Some(expected) {
        return Ok(());
    }
    Err(Error::ImageLength {
        command,
        width: image.width,
        height: image.height,
        expected,
        found,
    })
}

fn length(command: usize, expected: usize, found: usize) -> Result<(), Error> {
    if expected == found {
        Ok(())
    } else {
        Err(Error::ChannelLength {
            command,
            expected,
            found,
        })
    }
}

fn per_point(command: usize, points: usize, paints: &Paints) -> Result<(), Error> {
    length(command, points, paints.index.len())?;
    for (point, &index) in paints.index.iter().enumerate() {
        if !usize::try_from(index).is_ok_and(|i| i < paints.table.len()) {
            return Err(Error::PaintIndex {
                command,
                point,
                index,
                table: paints.table.len(),
            });
        }
    }
    for paint in &paints.table {
        brush(command, paint)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use kurbo::Affine;

    use super::*;

    #[test]
    fn a_transform_is_checked_coefficient_by_coefficient() {
        let list = List {
            commands: vec![
                Command::Push {
                    transform: Affine::new([1.0, 0.0, 0.0, f64::INFINITY, 0.0, 0.0]),
                    clip: None,
                },
                Command::Pop,
            ],
            ..List::default()
        };
        assert_eq!(
            check(&list),
            Err(Error::NonFinite {
                command: 0,
                value: f64::INFINITY
            })
        );
    }
}
