//! Tick labels that would still collide are dropped, in two passes. Within
//! an axis, every k-th label is kept for the smallest k without overlap
//! (Vega's parity strategy), counted from each tick's absolute index so
//! that labels do not flicker while the axis pans; a stride never drops
//! more than k − 1 labels in a row, and a time tick that follows its
//! predecessor closer than the interval's step (a run restarted at a month
//! boundary) gives way alone when the two collide. Across axes, labels of
//! larger panels are placed first and later ones that overlap them are
//! dropped. Both passes test oriented boxes ([`Obb`]), so rotated labels
//! collide as what they draw.

use rustd3_core::ticks;

use crate::layout::Obb;

/// Indices this large lose integer precision in `f64`.
const EXACT: f64 = 9_007_199_254_740_992.0;

/// One axis's tick labels as collision candidates: each label's box in list
/// px (`None` for one that is not drawn), its tick's absolute index and
/// whether its tick restarts a run, and the area of its panel's plot area.
pub(crate) struct Axis<'b> {
    /// The plot area of the axis's panel, px²: larger panels go first.
    pub(crate) area: f64,
    /// Each label's box, in tick order.
    pub(crate) boxes: &'b [Option<Obb>],
    /// Each tick's absolute index, aligned with the boxes.
    pub(crate) indices: &'b [i64],
    /// Whether each tick follows the one before it closer than its time
    /// interval's step, aligned with the boxes; empty for other axes.
    pub(crate) restarts: &'b [bool],
}

/// Which labels of `axes` (given in panel order, then axis order) layout
/// keeps. Within each axis, a label whose tick restarts a run gives way
/// first when its box overlaps the label before it: of the two, its tick
/// is the nearer to the boundary that made their spacing uneven (the
/// first day of a month after a month's last stepped day). The candidates
/// are then the labels whose index is a multiple of the smallest stride
/// k ≥ 1 for which those labels pairwise do not overlap (touching is not
/// overlap) and no k labels in a row are left out; when no stride leaves
/// one, the indices are taken as the labels' positions in the set. Across
/// axes, candidates go in order of their panel's plot area (larger first),
/// then panel, axis and label order, and one is kept when its box overlaps
/// no kept box.
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
/// that grid and the values are evenly spaced on it; `None` otherwise:
/// fewer than two values, log or symlog decades, values chosen off the
/// grid.
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

/// `boxes` without the labels whose tick restarts a run (`restarts`) and
/// whose box overlaps the box before it.
fn restart(boxes: &[Option<Obb>], restarts: &[bool]) -> Vec<Option<Obb>> {
    let mut out = boxes.to_vec();
    for (j, pair) in boxes.windows(2).enumerate() {
        if let [Some(before), Some(this)] = pair
            && restarts.get(j + 1) == Some(&true)
            && before.overlaps(this)
        {
            out[j + 1] = None;
        }
    }
    out
}

/// The labels of one axis kept by the smallest stride: those with a box
/// whose index is a multiple of the smallest k ≥ 1 that leaves at least
/// one, no two overlapping and no k in a row out (indices not evenly
/// spaced would otherwise let a stride keep one label of a row); `None`
/// when no k up to one past the span of the indices does.
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

    /// Boxes `width` wide centred `spacing` apart on one line.
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
        let mut restarts = [false; 8];
        restarts[7] = true;
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
