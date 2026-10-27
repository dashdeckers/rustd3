//! [`Command`]: one drawing operation of a list.

use kurbo::{Affine, BezPath, Point, Rect};
use peniko::{Brush, ImageData};

use crate::list::{Clip, GlyphRun, Paints};

/// One drawing operation. Paints (gradients, images) are positioned in the
/// space of the group the command is in, for markers and glyph runs too.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Command {
    /// A path filled under the nonzero rule.
    Fill {
        /// The path, in the group's space.
        path: BezPath,
        /// Its paint.
        paint: Brush,
    },
    /// A path stroked; caps and joins are explicit (see
    /// [`stroke`](crate::list::stroke)).
    Stroke {
        /// The path, in the group's space.
        path: BezPath,
        /// Width, caps, joins and dashes.
        style: kurbo::Stroke,
        /// Its paint.
        paint: Brush,
    },
    /// One shape filled at many points.
    Markers {
        /// The shape, centred on the origin at size 1.
        shape: BezPath,
        /// Where each copy is centred.
        at: Vec<Point>,
        /// The paint of every point when `paints` is absent.
        paint: Brush,
        /// Per-point uniform scale of `shape`; len == at.len().
        sizes: Option<Vec<f64>>,
        /// Per-point paints; when present, `paint` is unused.
        paints: Option<Paints>,
    },
    /// Shaped text.
    Glyphs(GlyphRun),
    /// Straight or premultiplied pixels stretched over `rect`.
    Image {
        /// Where the image lands, in the group's space.
        rect: Rect,
        /// The pixels.
        image: ImageData,
        /// Bilinear sampling when true, nearest otherwise.
        smooth: bool,
    },
    /// Opens a group: its transform applies after the enclosing groups',
    /// and its clip, in the group's space, to everything until its `Pop`.
    Push {
        /// The group's space in the enclosing group's.
        transform: Affine,
        /// What the group clips to, if anything.
        clip: Option<Clip>,
    },
    /// Closes the innermost open group.
    Pop,
}
