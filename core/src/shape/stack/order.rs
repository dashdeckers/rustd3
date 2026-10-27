//! d3's stack orders (`order/*.js`).

/// The order layers stack in, bottom first: d3's `stackOrder*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Order {
    /// The columns' own order (d3's `stackOrderNone`).
    #[default]
    None,
    /// The smallest sum of values at the bottom.
    Ascending,
    /// The largest sum of values at the bottom.
    Descending,
    /// The earliest peaks in the middle, later ones outward: for
    /// streamgraphs.
    InsideOut,
    /// The earliest peak at the bottom.
    Appearance,
    /// The columns' order reversed.
    Reverse,
}

impl Order {
    /// d3's order function: the layers' indices bottom first, read from
    /// each row's top (`[_, y1]`); NaN counts as 0.
    ///
    /// ```
    /// use rustd3_core::shape::stack::Order;
    ///
    /// let layers = [vec![[0.0, 1.0], [0.0, 2.0]], vec![[0.0, 0.0], [0.0, 1.0]]];
    /// assert_eq!(Order::Ascending.apply(&layers), [1, 0]);
    /// assert_eq!(Order::Reverse.apply(&layers), [1, 0]);
    /// ```
    pub fn apply(self, layers: &[Vec<[f64; 2]>]) -> Vec<usize> {
        let mut order: Vec<usize> = (0..layers.len()).collect();
        match self {
            Order::None => {}
            Order::Reverse => order.reverse(),
            Order::Ascending => ascending(layers, &mut order),
            Order::Descending => {
                ascending(layers, &mut order);
                order.reverse();
            }
            Order::Appearance => appearance(layers, &mut order),
            Order::InsideOut => order = inside_out(layers),
        }
        order
    }
}

/// d3's `sum(series)`: the tops that are numbers other than 0.
fn sum(layer: &[[f64; 2]]) -> f64 {
    layer
        .iter()
        .map(|&[_, top]| top)
        .filter(|v| *v != 0.0 && !v.is_nan())
        .fold(0.0, |s, v| s + v)
}

/// d3's `peak(series)`: the first row of the greatest top, 0 when no top
/// is a number above −∞.
fn peak(layer: &[[f64; 2]]) -> usize {
    let mut best = (0, f64::NEG_INFINITY);
    for (i, &[_, top]) in layer.iter().enumerate() {
        if top > best.1 {
            best = (i, top);
        }
    }
    best.0
}

/// Sorts `order` stably by the layers' sums (d3's `stackOrderAscending`).
fn ascending(layers: &[Vec<[f64; 2]>], order: &mut [usize]) {
    let sums: Vec<f64> = layers.iter().map(|l| sum(l)).collect();
    order.sort_by(|&a, &b| sums[a].total_cmp(&sums[b]));
}

/// Sorts `order` stably by the layers' peaks (d3's `stackOrderAppearance`).
fn appearance(layers: &[Vec<[f64; 2]>], order: &mut [usize]) {
    let peaks: Vec<usize> = layers.iter().map(|l| peak(l)).collect();
    order.sort_by_key(|&i| peaks[i]);
}

/// d3's `stackOrderInsideOut`: by appearance, each layer onto the lighter
/// of two piles, the bottom pile reversed beneath the top one.
fn inside_out(layers: &[Vec<[f64; 2]>]) -> Vec<usize> {
    let mut by_peak: Vec<usize> = (0..layers.len()).collect();
    appearance(layers, &mut by_peak);
    let (mut top, mut bottom) = (0.0, 0.0);
    let (mut tops, mut bottoms) = (Vec::new(), Vec::new());
    for j in by_peak {
        let s = sum(&layers[j]);
        if top < bottom {
            top += s;
            tops.push(j);
        } else {
            bottom += s;
            bottoms.push(j);
        }
    }
    bottoms.reverse();
    bottoms.extend(tops);
    bottoms
}
