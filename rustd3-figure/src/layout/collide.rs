//! Dropping tick labels that would still collide, in two passes. Within an
//! axis, every k-th label is kept for the smallest k without overlap
//! (Vega's parity strategy), counted from each tick's absolute index so
//! that labels do not flicker while the axis pans. Across axes, labels of
//! larger panels are placed first, and later ones that overlap them are
//! dropped. Both passes test oriented boxes ([`Obb`]), so rotated labels
//! collide as what they draw.

use rustd3_core::ticks;

use crate::layout::Obb;

/// Indices this large lose integer precision in `f64`.
const EXACT: f64 = 9_007_199_254_740_992.0;

/// One axis's tick labels as collision candidates.
pub(crate) struct Axis<'b> {
    /// The plot area of the axis's panel, px²: larger panels go first.
    pub(crate) area: f64,
    /// Each label's box in list px, in tick order; `None` for a label that
    /// is not drawn.
    pub(crate) boxes: &'b [Option<Obb>],
    /// Each tick's absolute index, aligned with the boxes.
    pub(crate) indices: &'b [i64],
    /// How each tick follows the one before it, aligned with the boxes;
    /// empty for axes other than time axes.
    pub(crate) restarts: &'b [Restart],
}

/// How a time axis's tick follows the one before it: by its interval's
/// step, or closer. A tick is closer when it is the first stepped day of a
/// month after the month's last, or the first stepped hour after a clock
/// change. For a closer pair, it also says which of the two labels is on
/// the coarser rung of the time format's ladder. That is the one whose
/// tick starts its parent unit (the month of stepped days, the day of
/// stepped hours), and its label shows that unit ("2026-11", "November",
/// a date at midnight).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Restart {
    /// By the step, or the first tick.
    Step,
    /// Closer than the step, and this tick starts its parent unit: the
    /// tick before gives way when their labels collide.
    Starts,
    /// Closer than the step, and this tick does not start its parent
    /// unit: it gives way when their labels collide.
    Follows,
}

/// Which labels of `axes`, given in panel order then axis order, layout
/// keeps.
///
/// Within each axis, of two time ticks closer than their interval's step
/// whose boxes overlap, the label on the finer rung gives way first. That
/// is the month's last stepped day before the first of the next month,
/// whose label shows the month, or the first stepped hour after a clock
/// change rather than the day's midnight. The candidates are then the
/// labels whose index is a multiple of the smallest stride k ≥ 1 for which
/// those labels do not overlap pairwise (touching is not overlap) and no k
/// labels in a row are left out. When no stride works on the ticks'
/// indices, the labels' positions in the set are used as indices instead.
///
/// Across axes, candidates go in order of their panel's plot area (larger
/// first), then panel, axis and label order. One is kept when its box
/// overlaps no kept box.
pub(crate) fn collide(axes: &[Axis<'_>]) -> Vec<Vec<bool>> {
    let mut keep: Vec<Vec<bool>> = axes.iter().map(|a| vec![false; a.boxes.len()]).collect();
    let mut candidates: Vec<(usize, usize)> = Vec::new();
    for (i, axis) in axes.iter().enumerate() {
        let boxes = restart(axis.boxes, axis.restarts);
        let positions: Vec<i64> = (0..boxes.len()).map(|j| j as i64).collect();
        let chosen = parity(&boxes, axis.indices).or_else(|| parity(&boxes, &positions));
        candidates.extend(chosen.into_iter().flatten().map(|j| (i, j)));
    }
    candidates.sort_by(|a, b| axes[b.0].area.total_cmp(&axes[a.0].area));
    let mut kept: Vec<Obb> = Vec::new();
    for (i, j) in candidates {
        let Some(bounds) = axes[i].boxes[j] else {
            continue;
        };
        if kept.iter().all(|k| !k.overlaps(&bounds)) {
            kept.push(bounds);
            keep[i][j] = true;
        }
    }
    keep
}

/// Each value's index on the grid of d3's ticks for `domain` at about
/// `count` ticks (its value over d3's tick step), when every value lies on
/// that grid and the values are evenly spaced on it. `None` otherwise, as
/// for fewer than two values, log or symlog decades, or values chosen off
/// the grid.
pub(crate) fn grid(values: &[f64], [a, b]: [f64; 2], count: usize) -> Option<Vec<i64>> {
    if values.len() < 2 {
        return None;
    }
    let increment = ticks::tick_increment(a.min(b), a.max(b), count as f64);
    if !increment.is_finite() || increment == 0.0 {
        return None;
    }
    let mut indices = Vec::with_capacity(values.len());
    for &v in values {
        // d3's ticks are i / −increment for steps below 1, i × increment
        // otherwise.
        let r = if increment < 0.0 {
            v * -increment
        } else {
            v / increment
        };
        let i = r.round();
        let tolerance = 1e-6 + 8.0 * f64::EPSILON * r.abs();
        if !((r - i).abs() <= tolerance && i.abs() < EXACT) {
            return None;
        }
        indices.push(i as i64);
    }
    let step = indices[1] - indices[0];
    let even = step != 0 && indices.windows(2).all(|w| w[1] - w[0] == step);
    even.then_some(indices)
}

/// `boxes` without the label on the finer rung of each pair of ticks
/// closer than their interval's step (`restarts`) whose boxes overlap: the
/// earlier when the later starts its parent unit, else the later.
fn restart(boxes: &[Option<Obb>], restarts: &[Restart]) -> Vec<Option<Obb>> {
    let mut out = boxes.to_vec();
    for (j, pair) in boxes.windows(2).enumerate() {
        let [Some(before), Some(this)] = pair else {
            continue;
        };
        if !before.overlaps(this) {
            continue;
        }
        match restarts.get(j + 1) {
            Some(Restart::Starts) => out[j] = None,
            Some(Restart::Follows) => out[j + 1] = None,
            Some(Restart::Step) | None => {}
        }
    }
    out
}

/// The labels of one axis kept by the smallest stride. They are the labels
/// with a box whose index is a multiple of the smallest k ≥ 1 that keeps at
/// least one, with no two overlapping and no k in a row left out. The last
/// condition matters for indices that are not evenly spaced, which would
/// otherwise let a stride keep one label of a row. `None` when no k up to
/// one past the span of the indices works.
fn parity(boxes: &[Option<Obb>], indices: &[i64]) -> Option<Vec<usize>> {
    let placed: Vec<(usize, i64, Obb)> = boxes
        .iter()
        .zip(indices)
        .enumerate()
        .filter_map(|(j, (b, &i))| Some((j, i, (*b)?)))
        .collect();
    let low = placed.iter().map(|p| p.1).min()?;
    let high = placed.iter().map(|p| p.1).max()?;
    let span = high.saturating_sub(low).saturating_add(1);
    (1..=span.saturating_add(1)).find_map(|k| {
        let on = |p: &(usize, i64, Obb)| p.1.rem_euclid(k) == 0;
        let chosen: Vec<&(usize, i64, Obb)> = placed.iter().filter(|p| on(p)).collect();
        let apart = chosen
            .iter()
            .enumerate()
            .all(|(n, a)| chosen[n + 1..].iter().all(|b| !a.2.overlaps(&b.2)));
        let planned = placed.split(|p| on(p)).all(|out| (out.len() as i64) < k);
        (!chosen.is_empty() && apart && planned).then(|| chosen.iter().map(|p| p.0).collect())
    })
}

#[cfg(test)]
mod tests {
    use kurbo::{Rect, Vec2};

    use super::*;

    /// Boxes `width` wide centered `spacing` apart on one line.
    fn row(widths: &[f64], spacing: f64) -> Vec<Option<Obb>> {
        widths
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let x = i as f64 * spacing;
                Some(Obb::rect(Rect::new(x - w / 2.0, 0.0, x + w / 2.0, 10.0)))
            })
            .collect()
    }

    #[test]
    fn grid_indices_are_values_over_the_tick_step() {
        let values: Vec<f64> = (0..=20).map(|i| f64::from(i) * 5.0).collect();
        let expected: Vec<i64> = (0..=20).collect();
        assert_eq!(grid(&values, [0.0, 100.0], 20), Some(expected));
        let panned: Vec<f64> = (1..=20).map(|i| f64::from(i) * 5.0).collect();
        let expected: Vec<i64> = (1..=20).collect();
        assert_eq!(grid(&panned, [2.5, 102.5], 20), Some(expected));
        // d3's ticks of [1e9, 1e9 + 1]: i / 5.
        let far: Vec<f64> = (0..=5).map(|i| (5e9 + f64::from(i)) / 5.0).collect();
        assert_eq!(
            grid(&far, [1e9, 1e9 + 1.0], 7),
            Some((5_000_000_000..=5_000_000_005).collect())
        );
    }

    #[test]
    fn grid_is_none_for_decades_and_single_values() {
        let decades = [1.0, 10.0, 100.0, 1000.0];
        assert_eq!(grid(&decades, [1.0, 1000.0], 5), None);
        let log = [1.0, 2.0, 5.0, 10.0, 20.0, 50.0];
        assert_eq!(grid(&log, [1.0, 50.0], 10), None);
        assert_eq!(grid(&[3.0], [0.0, 10.0], 5), None);
    }

    #[test]
    fn rotated_labels_keep_a_smaller_stride_than_flat_ones() {
        // 60 × 15 px labels every 25 px: flat ones keep every third, at
        // 45° (21.2 px apart at least) every one.
        let at = |deg: f64| -> Vec<Option<Obb>> {
            (0..8)
                .map(|i| {
                    let local = Rect::new(-30.0, -7.5, 30.0, 7.5);
                    let rotated = Obb::rotated(local, f64::to_radians(deg));
                    Some(rotated + Vec2::new(25.0 * f64::from(i), 0.0))
                })
                .collect()
        };
        let indices: Vec<i64> = (0..8).collect();
        assert_eq!(parity(&at(0.0), &indices), Some(vec![0, 3, 6]));
        assert_eq!(parity(&at(45.0), &indices), Some((0..8).collect()));
        assert_eq!(parity(&at(30.0), &indices), Some(vec![0, 2, 4, 6]));
    }

    #[test]
    fn the_stride_counts_from_the_absolute_index() {
        // Ticks every 10 px; labels 18.66 px wide but the last two 28.
        let widths = [18.66, 18.66, 18.66, 18.66, 28.0, 28.0];
        let boxes = row(&widths, 10.0);
        let indices: Vec<i64> = (16..22).collect();
        let kept = parity(&boxes, &indices);
        // k = 2 keeps 16, 18, 20, whose last two overlap; k = 3 keeps 18, 21.
        assert_eq!(kept, Some(vec![2, 5]));
    }

    #[test]
    fn a_stride_never_leaves_k_labels_in_a_row_out() {
        // Two-day ticks 30 px apart, labels 40 px wide: every day of one
        // month odd, the next month's first even. k = 2 would keep that
        // one alone, k = 3, 5 leave three and five out in a row; k = 7
        // keeps 7 and 14.
        let boxes = row(&[40.0; 8], 30.0);
        let indices = [1, 3, 5, 7, 9, 11, 13, 14];
        assert_eq!(parity(&boxes, &indices), Some(vec![3, 7]));
    }

    #[test]
    fn a_tick_that_restarts_a_run_gives_way_alone() {
        // Seven ticks 54 px apart and an eighth 27 px after the seventh,
        // labels 36 px wide: the eighth drops, the seven keep a stride of 1.
        let mut boxes = row(&[36.0; 7], 54.0);
        boxes.push(Some(Obb::rect(Rect::new(
            6.0 * 54.0 + 9.0,
            0.0,
            6.0 * 54.0 + 45.0,
            10.0,
        ))));
        let indices = [1, 3, 5, 7, 9, 11, 13, 14];
        let mut restarts = [Restart::Step; 8];
        restarts[7] = Restart::Follows;
        let axes = [Axis {
            area: 1.0,
            boxes: &boxes,
            indices: &indices,
            restarts: &restarts,
        }];
        let kept = [true, true, true, true, true, true, true, false];
        assert_eq!(collide(&axes), [kept]);
        // Without the restart, the stride drops all of them but the last.
        let axes = [Axis {
            restarts: &[],
            ..axes[0]
        }];
        assert_ne!(collide(&axes), [kept]);
    }

    #[test]
    fn of_a_close_pair_the_label_on_the_finer_rung_gives_way() {
        // The same row, the eighth tick a month's first day: the seventh
        // (the month's last stepped day) gives way, and the six before it
        // and the eighth keep a stride of 1.
        let mut boxes = row(&[36.0; 7], 54.0);
        boxes.push(Some(Obb::rect(Rect::new(
            6.0 * 54.0 + 9.0,
            0.0,
            6.0 * 54.0 + 45.0,
            10.0,
        ))));
        let indices = [1, 3, 5, 7, 9, 11, 13, 14];
        let mut restarts = [Restart::Step; 8];
        restarts[7] = Restart::Starts;
        let axes = [Axis {
            area: 1.0,
            boxes: &boxes,
            indices: &indices,
            restarts: &restarts,
        }];
        let kept = [true, true, true, true, true, true, false, true];
        assert_eq!(collide(&axes), [kept]);
    }

    #[test]
    fn indices_that_leave_no_label_fall_back_to_set_positions() {
        let boxes = row(&[30.0, 30.0, 30.0], 10.0);
        // Indices 1, 7, 13: k = 7 keeps 7 alone.
        assert_eq!(parity(&boxes, &[1, 7, 13]), Some(vec![1]));
        // No k from 1 to 6 divides 1009 or 1013 but 1, where they overlap.
        let boxes = row(&[30.0, 30.0], 10.0);
        assert_eq!(parity(&boxes, &[1009, 1013]), None);
        let axes = [Axis {
            area: 1.0,
            boxes: &boxes,
            indices: &[1009, 1013],
            restarts: &[],
        }];
        assert_eq!(collide(&axes), [[true, false]]);
    }

    #[test]
    fn larger_panels_keep_their_labels_across_axes() {
        let small = [Some(Obb::rect(Rect::new(0.0, 0.0, 20.0, 10.0)))];
        let large = [Some(Obb::rect(Rect::new(10.0, 5.0, 30.0, 15.0)))];
        let axes = [
            Axis {
                area: 100.0,
                boxes: &small,
                indices: &[0],
                restarts: &[],
            },
            Axis {
                area: 400.0,
                boxes: &large,
                indices: &[0],
                restarts: &[],
            },
        ];
        assert_eq!(collide(&axes), [[false], [true]]);
    }
}
