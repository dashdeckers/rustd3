//! [`Hover`]: the drawn point under the pointer, as `Plot::on_hover`
//! reports it, and [`Hits`]: the hit index the widget keeps per resolve
//! and the hover it last reported.
//!
//! The index holds the points the marks of the last drawing list draw. It
//! is rebuilt only when a refresh builds a list whose cells changed (a
//! list whose cells all hash as before draws the same points), so pointer
//! moves between redraws query it and resolve nothing; only for a widget
//! that reads it (the highlight, `Plot::on_hover`), and not while a
//! gesture lasts, whose frames hit-test nothing: the first refresh after
//! the gesture builds it, and until then nothing is hit-tested.

use iced_core::Point;
use rustd3_figure::{hit, kurbo};

/// Logical px within which the nearest drawn point is hovered.
pub(crate) const RADIUS: f64 = 12.0;

/// The drawn point nearest the pointer, within 12 logical px, in the panel
/// whose plot area the pointer is over. Plain data, like `Pointer`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct Hover {
    /// Flat panel index (`hit::Hit::panel`); `Resolved::panel_cell` maps it back.
    pub panel: usize,
    /// Index of the mark in the panel's insertion order.
    pub mark: usize,
    /// Index of the item in the mark's data (thinned marks: the sample index).
    pub index: usize,
    /// Pointer position in widget coordinates.
    pub at: Point,
}

/// The hit index of the last list, the hover now and the one last
/// reported.
#[derive(Debug, Default)]
pub(crate) struct Hits {
    /// The points the last list's marks draw, list px.
    index: hit::Index,
    /// The hover under the pointer at the last hit test.
    now: Option<Hover>,
    /// What was last reported: panel, mark and index.
    reported: Option<[usize; 3]>,
}

impl Hover {
    /// What identifies the hovered point: panel, mark and index.
    fn key(&self) -> [usize; 3] {
        [self.panel, self.mark, self.index]
    }
}

impl Hits {
    /// Replaces the index with that of a new list.
    pub(crate) fn rebuild(&mut self, index: hit::Index) {
        self.index = index;
    }

    /// The point of panel `panel` nearest `at` (list px) within
    /// [`RADIUS`].
    pub(crate) fn nearest(&self, at: kurbo::Point, panel: usize) -> Option<hit::Hit> {
        self.index
            .nearest(at, RADIUS)
            .filter(|hit| hit.panel == panel)
    }

    /// The hover a hit test found; true when its point differs from the
    /// last hit test's, so the widget redraws and the frame reports it.
    pub(crate) fn set(&mut self, now: Option<Hover>) -> bool {
        let changed = now.map(|hover| hover.key()) != self.now.map(|hover| hover.key());
        self.now = now;
        changed
    }

    /// The hover to report, once per change of the hovered point: the
    /// hover now when its panel, mark and index differ from the last
    /// reported (`None` once when it was lost).
    pub(crate) fn report(&mut self) -> Option<Option<Hover>> {
        let key = self.now.map(|hover| hover.key());
        (key != self.reported).then(|| {
            self.reported = key;
            self.now
        })
    }
}

#[cfg(test)]
mod tests {
    use iced_core::Point;

    use super::{Hits, Hover};

    /// A hover of point `index` of mark 0 in panel 0, at `x`.
    fn over(index: usize, x: f32) -> Option<Hover> {
        Some(Hover {
            panel: 0,
            mark: 0,
            index,
            at: Point::new(x, 10.0),
        })
    }

    #[test]
    fn a_hovered_point_is_reported_once_and_its_loss_once() {
        let mut hits = Hits::default();
        assert_eq!(hits.report(), None, "nothing hovered, nothing reported");
        assert!(hits.set(over(3, 20.0)));
        assert_eq!(hits.report(), Some(over(3, 20.0)));
        // Moving within the point changes nothing to report.
        assert!(!hits.set(over(3, 21.0)));
        assert_eq!(hits.report(), None);
        assert!(hits.set(over(4, 30.0)));
        assert!(hits.set(None));
        // Only the latest change of a frame is reported.
        assert_eq!(hits.report(), Some(None));
        assert_eq!(hits.report(), None);
    }
}
