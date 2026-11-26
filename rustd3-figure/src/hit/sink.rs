//! [`Sink`]: where a mark reports the points it draws.

use kurbo::{Point, Vec2};
use rustd3_core::color::Rgb;

/// A point a mark reported: datum index, figure px, and its data.
pub(crate) type Reported = (usize, Point, Option<[f64; 2]>);

/// Where a mark reports the points it draws, for hit testing: each datum's
/// index and where it is drawn, in the panel's coordinates (the plot area
/// spans `[0, w] × [0, h]`, as when drawing). A mark that is a curve also
/// says what it is ([`series`](Sink::series)) and where its points lie in
/// data ([`push_data`](Sink::push_data)), so a readout gives its value at
/// any x ([`Index::read`](crate::hit::Index::read)).
#[derive(Debug)]
pub struct Sink {
    origin: Vec2,
    points: Vec<Reported>,
    series: Option<(Option<String>, Rgb)>,
}

impl Sink {
    /// A sink for a panel whose plot area starts at `origin`, figure px.
    pub(crate) fn new(origin: Point) -> Sink {
        Sink {
            origin: origin.to_vec2(),
            points: Vec::new(),
            series: None,
        }
    }

    /// Reports datum `index` drawn at `at`, in the panel's coordinates.
    pub fn push(&mut self, index: usize, at: Point) {
        self.points.push((index, at + self.origin, None));
    }

    /// Reports datum `index` drawn at `at`, in the panel's coordinates,
    /// whose x and y are `data` in its scales' domains.
    pub fn push_data(&mut self, index: usize, at: Point, data: [f64; 2]) {
        self.points.push((index, at + self.origin, Some(data)));
    }

    /// Says the mark is a curve a readout reads at any x: its legend label,
    /// if it has one, and the color it is drawn in.
    pub fn series(&mut self, label: Option<String>, color: Rgb) {
        self.series = Some((label, color));
    }

    /// The points reported, in figure px, in order, and what the curve is.
    pub(crate) fn into_parts(self) -> (Vec<Reported>, Option<(Option<String>, Rgb)>) {
        (self.points, self.series)
    }
}
