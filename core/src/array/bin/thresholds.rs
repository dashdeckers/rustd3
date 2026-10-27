//! [`Thresholds`] and d3's threshold-count rules.

use crate::array::{count, deviation, quantile};
use crate::js;

/// How a [`Bin`](crate::array::Bin) chooses its thresholds: about a number
/// of uniform, nicely rounded ones (by one of d3's rules or given), or
/// explicit values.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Thresholds {
    /// d3.thresholdSturges, the default: [`sturges`].
    #[default]
    Sturges,
    /// d3.thresholdScott: [`scott`].
    Scott,
    /// d3.thresholdFreedmanDiaconis: [`freedman_diaconis`].
    FreedmanDiaconis,
    /// About this many uniform thresholds.
    Count(usize),
    /// These thresholds; those outside the domain are dropped.
    Values(Vec<f64>),
}

/// d3.thresholdSturges: ⌈log₂ n⌉ + 1 bins for the n values that are not
/// NaN, at least one.
#[doc(alias = "thresholdSturges")]
pub fn sturges(values: &[f64]) -> usize {
    let n = count(values.iter().copied()) as f64;
    // d3 divides by LN2 rather than calling log2, which can differ in the
    // last place at a power of two and so in the ceiling.
    (n.ln() / std::f64::consts::LN_2).ceil().max(0.0) as usize + 1
}

/// d3.thresholdScott: ⌈(max − min)·∛n / (3.49·σ)⌉ bins, σ the standard
/// deviation of the n values that are not NaN; one when there are none or
/// they do not vary.
#[doc(alias = "thresholdScott")]
pub fn scott(values: &[f64], min: f64, max: f64) -> usize {
    let c = count(values.iter().copied()) as f64;
    let d = deviation(values.iter().copied()).unwrap_or(f64::NAN);
    if c > 0.0 && js::truthy(d) {
        ((max - min) * c.cbrt() / (3.49 * d)).ceil() as usize
    } else {
        1
    }
}

/// d3.thresholdFreedmanDiaconis: ⌈(max − min) / (2·IQR·n^(−1/3))⌉ bins,
/// IQR the interquartile range of the n values that are not NaN; one when
/// there are none or the range is zero.
#[doc(alias = "thresholdFreedmanDiaconis")]
pub fn freedman_diaconis(values: &[f64], min: f64, max: f64) -> usize {
    let c = count(values.iter().copied()) as f64;
    let quartile = |p| quantile(values.iter().copied(), p).unwrap_or(f64::NAN);
    let d = quartile(0.75) - quartile(0.25);
    if c > 0.0 && js::truthy(d) {
        ((max - min) / (2.0 * d * c.powf(-1.0 / 3.0))).ceil() as usize
    } else {
        1
    }
}
