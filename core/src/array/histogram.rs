//! d3.bin (bin.js): [`Bin`], which lays values out into buckets.

use crate::array::bin::{Bucket, Thresholds, freedman_diaconis, scott, sturges};
use crate::array::{bisect_right, extent};
use crate::ticks;

/// d3's bin generator: a domain (the values' extent unless set) cut at
/// thresholds into [`Bucket`]s that hold the indices of the values in
/// them. Values outside the domain, and NaN, fall in no bucket.
///
/// ```
/// use rustd3_core::array::{self, bin::Thresholds};
///
/// let data = [0.0, 1.0, 1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0];
/// let lengths = |b: &array::bin::Bucket| (b.x0, b.x1, b.indices.len());
/// let sturges: Vec<_> = array::bin().layout(&data).iter().map(lengths).collect();
/// assert_eq!(sturges, [(0.0, 5.0, 5), (5.0, 10.0, 2), (10.0, 15.0, 1), (15.0, 20.0, 0), (20.0, 25.0, 1)]);
/// let given = array::bin().thresholds(Thresholds::Values(vec![5.0, 10.0])).layout(&data);
/// assert_eq!(given.iter().map(lengths).collect::<Vec<_>>(), [(0.0, 5.0, 5), (5.0, 10.0, 2), (10.0, 21.0, 2)]);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Bin {
    domain: Option<[f64; 2]>,
    thresholds: Thresholds,
}

/// d3.bin(): the values' extent, niced, cut by Sturges' rule.
#[must_use]
pub fn bin() -> Bin {
    Bin::default()
}

impl Bin {
    /// A fixed domain, which is not niced: values outside it are dropped.
    #[must_use]
    pub fn domain(self, domain: [f64; 2]) -> Self {
        Bin {
            domain: Some(domain),
            ..self
        }
    }

    /// The threshold rule.
    #[must_use]
    pub fn thresholds(self, thresholds: Thresholds) -> Self {
        Bin { thresholds, ..self }
    }

    /// The buckets of `values` (bin.js `histogram(data)`). By count, the
    /// thresholds are d3's ticks over the domain, which with the default
    /// domain is first niced to them, and the last bucket is widened by
    /// one step when its upper edge falls on the greatest value; values
    /// then fall into buckets by their quotient with the step when the
    /// first tick starts the domain, by bisection otherwise. Thresholds
    /// outside the domain are dropped. No value that is not NaN gives one
    /// empty bucket with NaN edges.
    #[must_use]
    pub fn layout(&self, values: &[f64]) -> Vec<Bucket> {
        let [mut x0, mut x1] = self
            .domain
            .or_else(|| extent(values.iter().copied()))
            .unwrap_or([f64::NAN; 2]);
        let count = match &self.thresholds {
            Thresholds::Sturges => sturges(values),
            Thresholds::Scott => scott(values, x0, x1),
            Thresholds::FreedmanDiaconis => freedman_diaconis(values, x0, x1),
            Thresholds::Count(n) => *n,
            Thresholds::Values(thresholds) => return assign(values, thresholds, [x0, x1], None),
        };
        let tn = count as f64;
        let max = x1;
        if self.domain.is_none() {
            [x0, x1] = ticks::nice(x0, x1, tn);
        }
        let mut thresholds = ticks::ticks(x0, x1, tn);
        let step = thresholds
            .first()
            .is_some_and(|&t| t <= x0)
            .then(|| ticks::tick_increment(x0, x1, tn));
        if thresholds.last().is_some_and(|&t| t >= x1) {
            if max >= x1 && self.domain.is_none() {
                let step = ticks::tick_increment(x0, x1, tn);
                if step.is_finite() && step > 0.0 {
                    x1 = ((x1 / step).floor() + 1.0) * step;
                } else if step.is_finite() && step < 0.0 {
                    x1 = ((x1 * -step).ceil() + 1.0) / -step;
                }
            } else {
                thresholds.pop();
            }
        }
        assign(values, &thresholds, [x0, x1], step)
    }
}

/// The buckets [x0, t₀), [t₀, t₁), …, [tₘ₋₁, x1] of the `thresholds` inside
/// the domain, and the indices of `values` in each: by `step` when it is
/// finite (d3's quantized path, negative for fractional steps), by
/// bisection otherwise.
fn assign(
    values: &[f64],
    thresholds: &[f64],
    [x0, x1]: [f64; 2],
    step: Option<f64>,
) -> Vec<Bucket> {
    let start = thresholds.iter().take_while(|&&t| t <= x0).count();
    let end = thresholds.len() - thresholds.iter().rev().take_while(|&&t| t > x1).count();
    let inside = thresholds.get(start..end).unwrap_or_default();
    let m = inside.len();
    let mut buckets: Vec<Bucket> = (0..=m)
        .map(|i| Bucket {
            x0: if i > 0 { inside[i - 1] } else { x0 },
            x1: if i < m { inside[i] } else { x1 },
            indices: Vec::new(),
        })
        .collect();
    for (index, &x) in values.iter().enumerate() {
        if !(x0 <= x && x <= x1) {
            continue;
        }
        let bucket = match step.filter(|s| s.is_finite()) {
            None => bisect_right(inside, x),
            Some(s) if s > 0.0 => ((x - x0) / s).floor() as usize,
            Some(s) if s < 0.0 => {
                let j = ((x0 - x) * s).floor() as usize;
                // A fractional step may land one bucket short.
                j + usize::from(inside.get(j).is_some_and(|&t| t <= x))
            }
            // d3 assigns nothing when the step is zero.
            Some(_) => continue,
        };
        buckets[bucket.min(m)].indices.push(index);
    }
    buckets
}
