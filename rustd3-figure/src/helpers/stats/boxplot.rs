//! Box plot statistics as matplotlib's `cbook.boxplot_stats` computes them:
//! [`Summary`], [`summary`] and [`Whis`]. The chart is
//! [`stats::boxplot`](super::boxplot()).

pub(super) mod chart;

use rustd3_core::array;

/// What a box plot draws of one group, as plain data (matplotlib's
/// `boxplot_stats`).
#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    /// The first quartile.
    pub q1: f64,
    /// The median.
    pub median: f64,
    /// The third quartile.
    pub q3: f64,
    /// The mean.
    pub mean: f64,
    /// The lower whisker's end: the least value inside the whiskers'
    /// reach, or `q1` when none lies below it.
    pub whisker_low: f64,
    /// The upper whisker's end: the greatest value inside the whiskers'
    /// reach, or `q3` when none lies above it.
    pub whisker_high: f64,
    /// The notch's lower end: `median − 1.57·IQR/√n`.
    pub notch_low: f64,
    /// The notch's upper end: `median + 1.57·IQR/√n`.
    pub notch_high: f64,
    /// The values beyond the whiskers: those below, then those above, each
    /// in the data's order.
    pub fliers: Vec<f64>,
}

/// How far a box plot's whiskers reach.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Whis {
    /// To the most extreme data within `[q1 − k·IQR, q3 + k·IQR]`; 1.5 by
    /// default, Tukey's and matplotlib's.
    Iqr(f64),
    /// To the most extreme data inside the `lo`th and `hi`th percentiles
    /// (0 to 100); everything beyond them is a flier.
    Percentiles(f64, f64),
}

impl Default for Whis {
    /// `Iqr(1.5)`.
    fn default() -> Self {
        Whis::Iqr(1.5)
    }
}

/// Computes matplotlib's `cbook.boxplot_stats` over the finite `values`
/// with `whis`. Quartiles are by R-7 (numpy's and d3's default "linear"),
/// whiskers end at the most extreme data within their reach (never inside
/// the box), and the notch is median ± 1.57·IQR/√n. `None` when no value
/// is finite.
///
/// ```
/// use rustd3_figure::helpers::stats::boxplot::{self, Whis};
///
/// let x = [1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 4.7, 5.1, 5.6, 6.2, 7.9, 9.5, 14.0];
/// let s = boxplot::summary(&x, Whis::default()).expect("finite values");
/// assert_eq!((s.q1, s.median, s.q3), (3.25, 4.4, 5.9));
/// assert_eq!((s.whisker_low, s.whisker_high), (1.2, 9.5));
/// assert_eq!(s.fliers, [14.0]);
/// assert!(boxplot::summary(&[f64::NAN], Whis::default()).is_none());
/// ```
pub fn summary(values: &[f64], whis: Whis) -> Option<Summary> {
    let finite: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    let mut sorted = finite.clone();
    sorted.sort_by(f64::total_cmp);
    let quantile = |p: f64| array::quantile_sorted(&sorted, p);
    let (q1, median, q3) = (quantile(0.25)?, quantile(0.5)?, quantile(0.75)?);
    let iqr = q3 - q1;
    let mean = array::mean(finite.iter().copied())?;
    let (low, high) = match whis {
        Whis::Iqr(k) => (q1 - k * iqr, q3 + k * iqr),
        Whis::Percentiles(lo, hi) => (quantile(lo / 100.0)?, quantile(hi / 100.0)?),
    };
    let whisker_high = sorted
        .iter()
        .copied()
        .rev()
        .find(|v| *v <= high)
        .filter(|v| *v >= q3)
        .unwrap_or(q3);
    let whisker_low = sorted
        .iter()
        .copied()
        .find(|v| *v >= low)
        .filter(|v| *v <= q1)
        .unwrap_or(q1);
    let below = finite.iter().copied().filter(|v| *v < whisker_low);
    let above = finite.iter().copied().filter(|v| *v > whisker_high);
    let notch = 1.57 * iqr / (finite.len() as f64).sqrt();
    Some(Summary {
        q1,
        median,
        q3,
        mean,
        whisker_low,
        whisker_high,
        notch_low: median - notch,
        notch_high: median + notch,
        fliers: below.chain(above).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whiskers_never_reach_inside_the_box() {
        // Tukey's reach is inside the box when k < 0; matplotlib then puts
        // the whiskers at the quartiles.
        let s = summary(&[1.0, 2.0, 3.0, 4.0, 5.0], Whis::Iqr(-1.0)).expect("values");
        assert_eq!((s.whisker_low, s.whisker_high), (s.q1, s.q3));
        assert_eq!(s.fliers, [1.0, 5.0]);
    }

    #[test]
    fn fliers_keep_the_datas_order_lows_first() {
        let values = [
            60.0, 3.0, 4.0, 3.5, 4.0, 4.5, 5.0, -30.0, 4.2, 3.8, 50.0, -40.0,
        ];
        let s = summary(&values, Whis::default()).expect("values");
        assert_eq!(s.fliers, [-30.0, -40.0, 60.0, 50.0]);
    }

    #[test]
    fn values_that_are_not_finite_are_left_out() {
        let s = summary(&[f64::INFINITY, 1.0, f64::NAN, 3.0], Whis::default()).expect("values");
        assert_eq!((s.q1, s.median, s.q3, s.mean), (1.5, 2.0, 2.5, 2.0));
        assert!(summary(&[], Whis::default()).is_none());
    }
}
