//! [`Reach`]: how far a panel's decorations reach past its plot area.

use rustd3_core::axis::Orient;

/// How far a panel's decorations reach past its plot area on each side,
/// px: axes with their labels and titles, tick labels that overhang the
/// panel's ends, the panel title.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct Reach {
    /// Above the plot area.
    pub(crate) top: f64,
    /// Right of it.
    pub(crate) right: f64,
    /// Below it.
    pub(crate) bottom: f64,
    /// Left of it.
    pub(crate) left: f64,
}

impl Reach {
    /// Widens side `orient` to at least `reach`.
    pub(crate) fn widen(&mut self, orient: Orient, reach: f64) {
        let side = match orient {
            Orient::Top => &mut self.top,
            Orient::Right => &mut self.right,
            Orient::Bottom => &mut self.bottom,
            Orient::Left => &mut self.left,
        };
        *side = side.max(reach);
    }

    /// Tick labels of an axis on side `orient` that reach `before` and
    /// `after` past the ends of the panel, along the axis, widen the sides
    /// they cross.
    pub(crate) fn overhang(&mut self, orient: Orient, [before, after]: [f64; 2]) {
        let (start, end) = match orient {
            Orient::Top | Orient::Bottom => (Orient::Left, Orient::Right),
            Orient::Left | Orient::Right => (Orient::Top, Orient::Bottom),
        };
        self.widen(start, before);
        self.widen(end, after);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overhangs_widen_the_perpendicular_sides() {
        let mut reach = Reach {
            left: 36.0,
            bottom: 24.0,
            ..Reach::default()
        };
        reach.overhang(Orient::Bottom, [4.0, 9.0]);
        reach.overhang(Orient::Left, [5.5, 0.0]);
        let expected = Reach {
            top: 5.5,
            right: 9.0,
            bottom: 24.0,
            left: 36.0,
        };
        assert_eq!(reach, expected);
    }
}
