//! d3's stack offsets (`offset/*.js`).

/// Where a stack's baseline is: d3's `stackOffset*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Offset {
    /// A zero baseline, each layer on the previous one's top (d3's
    /// `stackOffsetNone`).
    #[default]
    None,
    /// Each row scaled to fill [0, 1]: proportions.
    Expand,
    /// Positive values stacked up from zero and negative ones down from it.
    Diverging,
    /// Each row centered on zero.
    Silhouette,
    /// The baseline shifted to minimize the layers' weighted wiggle: d3's
    /// streamgraph offset, usually with `Order::InsideOut`.
    Wiggle,
}

impl Offset {
    /// d3's offset function: sets each row's `[y0, y1]` in place, `order`
    /// bottom first as [`Order::apply`](crate::shape::stack::Order::apply)
    /// returns it. A NaN top stacks as 0 and keeps NaN. Rows are the
    /// shortest layer's length, and indices in `order` past the last layer
    /// are ignored.
    ///
    /// ```
    /// use rustd3_core::shape::stack::{Offset, Order};
    ///
    /// let mut layers = vec![vec![[0.0, 3.0], [0.0, -2.0]], vec![[0.0, -1.0], [0.0, 1.0]]];
    /// let order = Order::None.apply(&layers);
    /// Offset::Diverging.apply(&mut layers, &order);
    /// assert_eq!(layers, [vec![[0.0, 3.0], [-2.0, 0.0]], vec![[-1.0, 0.0], [0.0, 1.0]]]);
    /// ```
    pub fn apply(self, layers: &mut [Vec<[f64; 2]>], order: &[usize]) {
        let order: Vec<usize> = order
            .iter()
            .copied()
            .filter(|&i| i < layers.len())
            .collect();
        let rows = layers.iter().map(Vec::len).min().unwrap_or(0);
        if order.is_empty() || rows == 0 {
            return;
        }
        match self {
            Offset::None => none(layers, &order, rows),
            Offset::Expand => {
                expand(layers, rows);
                none(layers, &order, rows);
            }
            Offset::Diverging => diverging(layers, &order, rows),
            Offset::Silhouette => {
                silhouette(layers, &order, rows);
                none(layers, &order, rows);
            }
            Offset::Wiggle => {
                wiggle(layers, &order, rows);
                none(layers, &order, rows);
            }
        }
    }
}

/// JavaScript's `x || 0` for a number: 0 for NaN and ±0.
fn or_zero(x: f64) -> f64 {
    if x.is_nan() || x == 0.0 { 0.0 } else { x }
}

/// The sum of a row's tops, NaN as 0 (d3's `y += series[i][j][1] || 0`).
fn row_sum(layers: &[Vec<[f64; 2]>], j: usize) -> f64 {
    layers.iter().fold(0.0, |y, l| y + or_zero(l[j][1]))
}

/// `offset/none.js`: each layer starts where the one below ends (where it
/// starts, if that end is NaN), keeping the bottom layer as it is.
fn none(layers: &mut [Vec<[f64; 2]>], order: &[usize], rows: usize) {
    for pair in order.windows(2) {
        let bases: Vec<f64> = layers[pair[0]][..rows]
            .iter()
            .map(|&[y0, y1]| if y1.is_nan() { y0 } else { y1 })
            .collect();
        for (row, base) in layers[pair[1]].iter_mut().zip(bases) {
            row[0] = base;
            row[1] += base;
        }
    }
}

/// `offset/expand.js`, before `none`: each row's tops over their sum, where
/// the sum is not 0 or NaN.
fn expand(layers: &mut [Vec<[f64; 2]>], rows: usize) {
    for j in 0..rows {
        let y = row_sum(layers, j);
        if y != 0.0 && !y.is_nan() {
            for layer in layers.iter_mut() {
                layer[j][1] /= y;
            }
        }
    }
}

/// `offset/diverging.js`: in order, positive spans up from the row's
/// positive total, negative ones down from its negative total, others at 0.
fn diverging(layers: &mut [Vec<[f64; 2]>], order: &[usize], rows: usize) {
    let (mut positive, mut negative) = (vec![0.0; rows], vec![0.0; rows]);
    for &i in order {
        for (j, row) in layers[i][..rows].iter_mut().enumerate() {
            let dy = row[1] - row[0];
            if dy > 0.0 {
                row[0] = positive[j];
                positive[j] += dy;
                row[1] = positive[j];
            } else if dy < 0.0 {
                row[1] = negative[j];
                negative[j] += dy;
                row[0] = negative[j];
            } else {
                *row = [0.0, dy];
            }
        }
    }
}

/// `offset/silhouette.js`, before `none`: the bottom layer starts at minus
/// half its row's total.
fn silhouette(layers: &mut [Vec<[f64; 2]>], order: &[usize], rows: usize) {
    for j in 0..rows {
        let y = -row_sum(layers, j) / 2.0;
        let row = &mut layers[order[0]][j];
        row[0] = y;
        row[1] += y;
    }
}

/// `offset/wiggle.js`, before `none`: the bottom layer's baseline moved row
/// by row against the weighted change of the layers above it (Byron and
/// Wattenberg's streamgraph baseline).
fn wiggle(layers: &mut [Vec<[f64; 2]>], order: &[usize], rows: usize) {
    let top = |layers: &[Vec<[f64; 2]>], i: usize, j: usize| or_zero(layers[order[i]][j][1]);
    let mut y = 0.0;
    for j in 1..rows {
        let (mut s1, mut s2) = (0.0, 0.0);
        for i in 0..order.len() {
            let (sij0, sij1) = (top(layers, i, j), top(layers, i, j - 1));
            let mut s3 = (sij0 - sij1) / 2.0;
            for k in 0..i {
                s3 += top(layers, k, j) - top(layers, k, j - 1);
            }
            s1 += sij0;
            s2 += s3 * sij0;
        }
        let row = &mut layers[order[0]][j - 1];
        row[0] = y;
        row[1] += y;
        if s1 != 0.0 && !s1.is_nan() {
            y -= s2 / s1;
        }
    }
    let row = &mut layers[order[0]][rows - 1];
    row[0] = y;
    row[1] += y;
}
