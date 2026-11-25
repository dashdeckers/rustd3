//! [`Plan`]: how layout places one axis. An axis is inner when a panel
//! further out on its side shows the same handle there: it keeps its ticks
//! and loses its labels and title. Axes on one side stack outward in
//! insertion order, and a stacked axis that is inner is not drawn.

use std::ops::Range;

use rustd3_core::axis::Orient;

use crate::axis::{self, Draw};
use crate::panel::Panel;

/// px per tick along a horizontal axis: Observable Plot 0.6.17's spacing.
const ACROSS: f64 = 80.0;

/// px per tick along a vertical axis.
const UP: f64 = 35.0;

/// px between a stacked axis and the decorations of the axis before it.
const GAP: f64 = 8.0;

/// How layout places one axis of a panel. Plain data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plan {
    /// Whether the axis is drawn at all.
    pub shown: bool,
    /// Whether it is drawn with its tick labels and title.
    pub labeled: bool,
    /// px between the plot area and the axis line.
    pub offset: f64,
    /// px from the axis line to the far edge of its decorations.
    pub thickness: f64,
    /// The tick count asked of the scale.
    pub count: usize,
}

/// Whether an axis is drawn, and with its labels and title: what its
/// panel's place in the grid and the axes before it decide.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Role {
    /// Whether the axis is drawn at all.
    pub(crate) shown: bool,
    /// Whether it is drawn with its tick labels and title.
    pub(crate) labeled: bool,
    /// Whether it is drawn with its title.
    pub(crate) titled: bool,
}

/// Each panel's axes' roles, in insertion order. An axis is inner when a
/// panel further out on its side shows the same handle on the same side:
/// left of it in a row it spans for a left axis, or below it in a column
/// it spans for a bottom axis. It is stacked when an axis drawn before it
/// is on its side. Inner axes lose their labels, and inner axes that are
/// stacked or moved outward are not drawn. `.labels(b)` sets the labels
/// and draws the axis whatever its place. `.keep_label(true)` keeps the
/// title of an axis drawn without its labels.
pub(crate) fn roles(panels: &[Panel<'_>]) -> Vec<Vec<Role>> {
    panels
        .iter()
        .map(|panel| {
            let mut drawn = [false; 4];
            panel
                .axes()
                .iter()
                .map(|a| {
                    let side = side(a.orient());
                    let inner = inner(panels, panel, &**a);
                    let outward = a.outward().is_some() || drawn[side];
                    let labeled = a.labels().unwrap_or(!inner);
                    let role = Role {
                        shown: a.labels().is_some() || !(inner && outward),
                        labeled,
                        titled: labeled || a.keeps_label(),
                    };
                    drawn[side] |= role.shown;
                    role
                })
                .collect()
        })
        .collect()
}

/// One panel's plans from its axes' roles, counts and thicknesses: each
/// drawn axis on a side sits 8 px past the offset and thickness of the
/// last drawn axis before it on that side, or on the plot area, unless
/// `.outward(px)` sets its offset.
pub(crate) fn stack<'a>(
    axes: &[Box<dyn Draw<'a> + 'a>],
    roles: &[Role],
    counts: &[usize],
    thicknesses: &[f64],
) -> Vec<Plan> {
    let mut reach: [Option<f64>; 4] = [None; 4];
    axes.iter()
        .zip(roles)
        .zip(counts.iter().zip(thicknesses))
        .map(|((a, role), (&count, &thickness))| {
            let side = side(a.orient());
            let offset = a
                .outward()
                .unwrap_or_else(|| reach[side].map_or(0.0, |r| r + GAP));
            if role.shown {
                reach[side] = Some(offset + thickness);
            }
            Plan {
                shown: role.shown,
                labeled: role.labeled,
                offset,
                thickness,
                count,
            }
        })
        .collect()
}

/// Whether another of `panels`, further out on `axis`'s side of `panel`
/// (beyond its columns in rows it overlaps, for a left or right axis;
/// beyond its rows in columns it overlaps, for a top or bottom axis),
/// shows the same handle on that side.
fn inner<'a>(panels: &[Panel<'a>], panel: &Panel<'a>, axis: &(dyn Draw<'a> + 'a)) -> bool {
    let orient = axis.orient();
    let (rows, cols) = (panel.cell().rows(), panel.cell().cols());
    let overlap = |a: &Range<usize>, b: &Range<usize>| a.start < b.end && b.start < a.end;
    panels.iter().any(|q| {
        let (r, c) = (q.cell().rows(), q.cell().cols());
        let beyond = match orient {
            Orient::Left => overlap(&r, &rows) && c.end <= cols.start,
            Orient::Right => overlap(&r, &rows) && c.start >= cols.end,
            Orient::Top => overlap(&c, &cols) && r.end <= rows.start,
            Orient::Bottom => overlap(&c, &cols) && r.start >= rows.end,
        };
        beyond
            && q.axes()
                .iter()
                .any(|b| b.orient() == orient && b.handle() == axis.handle())
    })
}

/// The index of side `orient`: top, right, bottom, left.
fn side(orient: Orient) -> usize {
    match orient {
        Orient::Top => 0,
        Orient::Right => 1,
        Orient::Bottom => 2,
        Orient::Left => 3,
    }
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
