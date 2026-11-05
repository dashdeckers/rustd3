//! Markers as one fill per point.

use kurbo::{Affine, BezPath, Point};
use peniko::Brush;

use crate::list::{Command, Paints};
use crate::lower::sweep;

/// One fill per point: the shape scaled by the point's size and centered on
/// it, painted from the point's paint.
pub(super) fn fills(
    shape: &BezPath,
    at: &[Point],
    paint: &Brush,
    sizes: Option<&[f64]>,
    paints: Option<&Paints>,
) -> Vec<Command> {
    at.iter()
        .enumerate()
        .map(|(k, p)| {
            let size = sizes.map_or(1.0, |s| s[k]);
            let place = Affine::translate(p.to_vec2()) * Affine::scale(size);
            Command::Fill {
                path: place * shape.clone(),
                paint: paints
                    .map_or(paint, |t| &t.table[t.index[k] as usize])
                    .clone(),
            }
        })
        .collect()
}

/// Whether any point is painted with a sweep gradient.
pub(super) fn painted_by_sweep(paint: &Brush, paints: Option<&Paints>) -> bool {
    match paints {
        Some(paints) => paints.table.iter().any(sweep::is_sweep),
        None => sweep::is_sweep(paint),
    }
}
