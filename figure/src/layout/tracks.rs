//! Tracks: one axis of the grid, from weights, gaps and reaches.

/// The start and length of each track along one axis of the grid.
/// `reaches[i]` is how far the decorations of track `i`'s panels reach
/// before and after its plot areas (the largest of its panels'). `gaps` is
/// the whitespace between neighboring tracks, and the tracks fill
/// `[start, total]`. Reaches and gaps are fixed, what remains (never below
/// 0) is split by `weights` (1 where missing), and positions are running
/// sums.
pub(crate) fn tracks(
    weights: &[f64],
    gaps: &[f64],
    reaches: &[[f64; 2]],
    [start, total]: [f64; 2],
) -> Vec<[f64; 2]> {
    let n = reaches.len();
    // Weights relative to the largest, so that subnormal or huge ones split
    // the room as their ratios say instead of overflowing.
    let largest = (0..n)
        .map(|i| weights.get(i).copied().unwrap_or(1.0))
        .fold(0.0, f64::max);
    let weight = |i: usize| {
        let w = weights.get(i).copied().unwrap_or(1.0);
        if largest > 0.0 { w / largest } else { w }
    };
    let gap = |i: usize| gaps.get(i).copied().unwrap_or(0.0);
    let fixed = reaches.iter().map(|[low, high]| low + high).sum::<f64>()
        + (0..n.saturating_sub(1)).map(gap).sum::<f64>();
    let sum: f64 = (0..n).map(weight).sum();
    let free = (total - fixed).max(0.0);
    let per = if sum > 0.0 { free / sum } else { 0.0 };
    let mut at = start;
    (0..n)
        .map(|i| {
            let [low, high] = reaches[i];
            at += low;
            let track = [at, per * weight(i)];
            at += track[1] + high + if i + 1 < n { gap(i) } else { 0.0 };
            track
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wheel_figures_columns_split_what_reaches_and_gaps_leave() {
        let weights = [1.0, 0.25, 1.0, 0.25];
        let gaps = [4.0, 16.0, 4.0];
        let reaches = [[94.0, 0.0], [0.0, 0.0], [5.0, 0.0], [0.0, 0.0]];
        let columns = tracks(&weights, &gaps, &reaches, [8.0, 944.0]);
        // fixed 123, free 821: 328.4 per unit of weight.
        let expected = [[102.0, 328.4], [434.4, 82.1], [537.5, 328.4], [869.9, 82.1]];
        for (track, want) in columns.iter().zip(expected) {
            assert!((track[0] - want[0]).abs() < 1e-9, "{track:?}");
            assert!((track[1] - want[1]).abs() < 1e-9, "{track:?}");
        }
        // FL's side panel to FR's scatter: the gap plus FR's reach.
        let between = columns[2][0] - (columns[1][0] + columns[1][1]);
        assert!((between - 21.0).abs() < 1e-9, "{between}");
    }

    #[test]
    fn tracks_never_get_a_negative_length() {
        let columns = tracks(&[], &[10.0], &[[60.0, 0.0], [60.0, 0.0]], [0.0, 100.0]);
        assert_eq!(columns, [[60.0, 0.0], [130.0, 0.0]]);
    }
}
