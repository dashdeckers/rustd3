//! [`Sink`]: where a mark reports the points it draws.

use kurbo::{Point, Vec2};

/// Where a mark reports the points it draws, for hit testing: each datum's
/// index and where it is drawn, in the panel's coordinates (the plot area
/// spans `[0, w] × [0, h]`, as when drawing).
#[derive(Debug)]
pub struct Sink {
    origin: Vec2,
    points: Vec<(usize, Point)>,
}

impl Sink {
    /// A sink for a panel whose plot area starts at `origin`, figure px.
    pub(crate) fn new(origin: Point) -> Sink {
        Sink {
            origin: origin.to_vec2(),
            points: Vec::new(),
        }
    }

    /// Reports datum `index` drawn at `at`, in the panel's coordinates.
    pub fn push(&mut self, index: usize, at: Point) {
        self.points.push((index, at + self.origin));
    }

    /// The points reported, in figure px, in order.
    pub(crate) fn into_points(self) -> Vec<(usize, Point)> {
        self.points
    }
}
