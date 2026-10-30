//! [`Region`]: what the pointer is over, a plot area or an axis, and the
//! [`Map`] of regions of one resolved figure.
//!
//! Hit testing reports points, not axes, so each axis's strip comes from
//! the resolved layout: its panel's plot area, its side, and its
//! plan's offset and thickness, along the plot area's edge.

use rustd3_core::axis::Orient;
use rustd3_figure::Resolved;
use rustd3_figure::kurbo::{Point, Rect};

use crate::plot::domains::Key;

/// What the pointer is over.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Region {
    /// The plot area of a panel.
    Area { panel: usize },
    /// The strip an axis of a panel draws in.
    Axis {
        panel: usize,
        key: Key,
        along: Along,
    },
    /// Neither.
    Outside,
}

/// The direction a handle maps along.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Along {
    /// Across: x.
    X,
    /// Up: y.
    Y,
}

/// The regions of one resolved figure, in list px (the widget's logical
/// px), and the handles each panel zooms.
#[derive(Debug, Clone, Default)]
pub(crate) struct Map {
    /// Plot areas, in panel order.
    areas: Vec<Rect>,
    /// Shown axes: their strip and region.
    axes: Vec<(Rect, Region)>,
    /// Per panel, the handles zoomed along x and along y.
    zooms: Vec<[Vec<Key>; 2]>,
}

impl Along {
    /// Index 0 for x and 1 for y.
    pub(crate) fn index(self) -> usize {
        match self {
            Along::X => 0,
            Along::Y => 1,
        }
    }

    /// The (k, offset) of `t` along this direction.
    pub(crate) fn pick(self, t: rustd3_core::interact::Transform) -> (f64, f64) {
        match self {
            Along::X => (t.k, t.x),
            Along::Y => (t.k, t.y),
        }
    }

    /// The direction an axis on side `orient` maps along.
    fn of(orient: Orient) -> Along {
        match orient {
            Orient::Top | Orient::Bottom => Along::X,
            Orient::Left | Orient::Right => Along::Y,
        }
    }
}

impl Region {
    /// The panel the region belongs to.
    pub(crate) fn panel(self) -> Option<usize> {
        match self {
            Region::Area { panel } | Region::Axis { panel, .. } => Some(panel),
            Region::Outside => None,
        }
    }
}

impl Map {
    /// The regions of `resolved`: its plot areas, the strips of its shown
    /// axes, and the handles its panels zoom (those whose scale it lacks
    /// are left out).
    pub(crate) fn of(resolved: &Resolved) -> Map {
        let key = |index: usize| resolved.scale_at(index).map(|scale| Key::new(index, scale));
        let areas = resolved.plot_areas().to_vec();
        let mut axes = Vec::new();
        for (panel, area) in areas.iter().enumerate() {
            for axis in 0.. {
                let (Some(plan), Some((orient, index))) =
                    (resolved.plan(panel, axis), resolved.axis(panel, axis))
                else {
                    break;
                };
                if let (true, Some(key)) = (plan.shown, key(index)) {
                    let strip = strip(*area, orient, plan.offset, plan.thickness);
                    let along = Along::of(orient);
                    axes.push((strip, Region::Axis { panel, key, along }));
                }
            }
        }
        let zooms = (0..areas.len())
            .map(|panel| {
                let (x, y) = resolved.zooms(panel);
                [x, y].map(|indices| indices.iter().filter_map(|&i| key(i)).collect())
            })
            .collect();
        Map { areas, axes, zooms }
    }

    /// The region at `at` (list px): a plot area first, then an axis strip.
    pub(crate) fn at(&self, at: Point) -> Region {
        if let Some(panel) = self.areas.iter().position(|area| inside(*area, at)) {
            return Region::Area { panel };
        }
        self.axes
            .iter()
            .find(|(strip, _)| inside(*strip, at))
            .map_or(Region::Outside, |&(_, region)| region)
    }

    /// The handles a gesture over `region` rescales, with the direction
    /// each maps along: over a plot area, every handle its panel zooms
    /// along x and along y (each once); over an axis, its handle if the
    /// panel zooms it along that direction.
    pub(crate) fn tracks(&self, region: Region) -> Vec<(Key, Along)> {
        match region {
            Region::Area { panel } => {
                let Some([x, y]) = self.zooms.get(panel) else {
                    return Vec::new();
                };
                let mut tracks: Vec<(Key, Along)> = Vec::new();
                let all = x
                    .iter()
                    .map(|&key| (key, Along::X))
                    .chain(y.iter().map(|&key| (key, Along::Y)));
                for (key, along) in all {
                    if tracks.iter().all(|&(k, _)| k != key) {
                        tracks.push((key, along));
                    }
                }
                tracks
            }
            Region::Axis { panel, key, along } => self
                .zooms
                .get(panel)
                .filter(|zooms| zooms[along.index()].contains(&key))
                .map(|_| vec![(key, along)])
                .unwrap_or_default(),
            Region::Outside => Vec::new(),
        }
    }

    /// The plot area of `panel`, list px.
    pub(crate) fn area(&self, panel: usize) -> Option<Rect> {
        self.areas.get(panel).copied()
    }

    /// The handle whose view a preview of `panel` follows along `along`:
    /// the first it zooms that way, else the first of its axes that way.
    pub(crate) fn lead(&self, panel: usize, along: Along) -> Option<Key> {
        let zoomed = self
            .zooms
            .get(panel)
            .and_then(|zooms| zooms[along.index()].first().copied());
        zoomed.or_else(|| {
            self.axes.iter().find_map(|&(_, region)| match region {
                Region::Axis {
                    panel: p,
                    key,
                    along: a,
                } if p == panel && a == along => Some(key),
                Region::Area { .. } | Region::Axis { .. } | Region::Outside => None,
            })
        })
    }
}

/// The strip an axis on side `orient` of `area` draws in: from `offset`
/// to `offset + thickness` px outside the plot area's edge, along it.
fn strip(area: Rect, orient: Orient, offset: f64, thickness: f64) -> Rect {
    let (near, far) = (offset, offset + thickness);
    match orient {
        Orient::Left => Rect::new(area.x0 - far, area.y0, area.x0 - near, area.y1),
        Orient::Right => Rect::new(area.x1 + near, area.y0, area.x1 + far, area.y1),
        Orient::Top => Rect::new(area.x0, area.y0 - far, area.x1, area.y0 - near),
        Orient::Bottom => Rect::new(area.x0, area.y1 + near, area.x1, area.y1 + far),
    }
}

/// Whether `at` lies in `rect`, its far edges excluded.
fn inside(rect: Rect, at: Point) -> bool {
    (rect.x0..rect.x1).contains(&at.x) && (rect.y0..rect.y1).contains(&at.y)
}

#[cfg(test)]
mod tests {
    use rustd3_core::axis::Orient;
    use rustd3_core::scale;
    use rustd3_figure::kurbo::{Point, Rect, Size};
    use rustd3_figure::{Figure, Held, Target, axis};

    use super::{Along, Map, Region, strip};
    use crate::plot::domains::Key;
    use crate::plot::state;

    #[test]
    fn axis_strips_lie_outside_the_plot_area_along_its_edge() {
        let area = Rect::new(50.0, 20.0, 450.0, 280.0);
        assert_eq!(
            strip(area, Orient::Left, 0.0, 30.0),
            Rect::new(20.0, 20.0, 50.0, 280.0)
        );
        assert_eq!(
            strip(area, Orient::Bottom, 4.0, 25.0),
            Rect::new(50.0, 284.0, 450.0, 309.0)
        );
        assert_eq!(
            strip(area, Orient::Right, 0.0, 10.0),
            Rect::new(450.0, 20.0, 460.0, 280.0)
        );
        assert_eq!(
            strip(area, Orient::Top, 0.0, 10.0),
            Rect::new(50.0, 10.0, 450.0, 20.0)
        );
    }

    #[test]
    fn regions_are_plot_areas_then_axis_strips() {
        let mut fig = Figure::new();
        let x = fig.scale(scale::linear().domain([0.0, 10.0]));
        let y = fig.scale(scale::log().domain([1.0, 100.0]));
        fig.panel(0, 0)
            .axis(axis::left(y))
            .axis(axis::bottom(x))
            .zoom_x(x);
        let mut cache = state::cache();
        cache.begin();
        let size = Size::new(480.0, 320.0);
        let resolved = fig.resolve(size, &mut cache, &Held::default(), Target::Vector);
        let map = Map::of(&resolved);
        let area = resolved.plot_areas()[0];
        let (kx, ky) = (
            Key::of::<scale::Linear>(x.index()),
            Key::of::<scale::Log>(y.index()),
        );
        assert_eq!(map.at(area.center()), Region::Area { panel: 0 });
        let left = Point::new(area.x0 - 5.0, area.center().y);
        let below = Point::new(area.center().x, area.y1 + 5.0);
        assert_eq!(
            map.at(left),
            Region::Axis {
                panel: 0,
                key: ky,
                along: Along::Y
            }
        );
        assert_eq!(
            map.at(below),
            Region::Axis {
                panel: 0,
                key: kx,
                along: Along::X
            }
        );
        assert_eq!(map.at(Point::new(1.0, 1.0)), Region::Outside);
        // The panel zooms x only: its area rescales x, its bottom axis x,
        // its left axis nothing.
        assert_eq!(map.tracks(map.at(area.center())), [(kx, Along::X)]);
        assert_eq!(map.tracks(map.at(below)), [(kx, Along::X)]);
        assert!(map.tracks(map.at(left)).is_empty());
        assert!(map.tracks(Region::Outside).is_empty());
        assert_eq!(map.area(0), Some(area));
        // Previews of the panel follow x, which it zooms, and y, its axis.
        assert_eq!(map.lead(0, Along::X), Some(kx));
        assert_eq!(map.lead(0, Along::Y), Some(ky));
        assert_eq!(map.lead(1, Along::X), None);
    }
}
