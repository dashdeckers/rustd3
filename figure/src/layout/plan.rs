//! [`Plan`]: how layout places one axis, and the tick count an axis's
//! length asks for.

use rustd3_core::axis::Orient;

use crate::axis;

/// px per tick along a horizontal axis: Observable Plot 0.6.17's spacing.
const ACROSS: f64 = 80.0;

/// px per tick along a vertical axis.
const UP: f64 = 35.0;

/// How layout places one axis of a panel: whether it is drawn, whether
/// with its tick labels and title, how far outside the plot area it sits,
/// how far past that it reaches, and how many ticks it asks its scale for.
/// Plain data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plan {
    /// Drawn at all.
    pub shown: bool,
    /// Drawn with its tick labels and title.
    pub labeled: bool,
    /// px between the plot area and the axis line.
    pub offset: f64,
    /// px from the axis line to the far edge of its decorations.
    pub thickness: f64,
    /// The tick count asked of the scale.
    pub count: usize,
}

/// The tick count of an axis on side `orient` that is `length` px long:
/// one tick per 80 px across, per 35 px up, at least 2.
pub(crate) fn count(orient: Orient, length: f64) -> usize {
    let spacing = if axis::across(orient) { ACROSS } else { UP };
    let count = (length / spacing).round();
    if count > 2.0 {
        // Saturates for lengths beyond any canvas.
        count as usize
    } else {
        2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_follow_length_one_tick_per_80_px_across_and_35_px_up() {
        assert_eq!(count(Orient::Bottom, 328.4), 4);
        assert_eq!(count(Orient::Top, 100.0), 2);
        assert_eq!(count(Orient::Left, 300.0), 9);
        assert_eq!(count(Orient::Right, 0.0), 2);
        assert_eq!(count(Orient::Left, f64::NAN), 2);
    }
}
