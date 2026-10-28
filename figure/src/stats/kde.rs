//! [`Kde`]: a Gaussian kernel density estimate with Scott's bandwidth.

use std::f64::consts::TAU;

use rustd3_core::array;

/// A Gaussian kernel density estimate with Scott's bandwidth, as scipy's
/// `gaussian_kde` and seaborn's `kdeplot` compute it: `h = σ̂ · n^(−1/5)`,
/// σ̂ the sample standard deviation (n − 1), and the density at x the mean
/// of the n kernels φ((x − xᵢ)/h)/h, φ the standard normal density.
///
/// ```
/// use rustd3_figure::stats::Kde;
///
/// let kde = Kde::new([1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4]).expect("values that vary");
/// let grid = kde.grid(200, 3.0);
/// assert_eq!(grid.len(), 200);
/// assert!((grid[0].0 - (1.2 - 3.0 * kde.bandwidth())).abs() < 1e-12);
/// assert!(Kde::new([2.0, 2.0, f64::NAN]).is_none());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Kde {
    /// The finite values, at least two, not all equal.
    values: Vec<f64>,
    /// Their least and greatest.
    extent: [f64; 2],
    /// The kernel's standard deviation, positive and finite.
    h: f64,
}

impl Kde {
    /// The estimate over the finite values of `values`, with Scott's
    /// bandwidth; `None` when fewer than two are finite or their variance
    /// is zero.
    pub fn new(values: impl IntoIterator<Item = f64>) -> Option<Kde> {
        let values: Vec<f64> = values.into_iter().filter(|v| v.is_finite()).collect();
        let n = values.len();
        let deviation = array::deviation(values.iter().copied())?;
        let h = deviation * (n as f64).powf(-0.2);
        let extent = array::extent(values.iter().copied())?;
        (n >= 2 && h > 0.0 && h.is_finite()).then_some(Kde { values, extent, h })
    }

    /// The bandwidth h, the kernel's standard deviation.
    pub fn bandwidth(&self) -> f64 {
        self.h
    }

    /// The density at `x`.
    pub fn eval(&self, x: f64) -> f64 {
        let sum: f64 = self
            .values
            .iter()
            .map(|v| (-0.5 * ((x - v) / self.h).powi(2)).exp())
            .sum();
        sum / (self.values.len() as f64 * self.h * TAU.sqrt())
    }

    /// `points` positions evenly spaced over `[min − cut·h, max + cut·h]`
    /// (numpy's `linspace`, both ends included) and the density at each:
    /// seaborn's grid, which `density_y` takes at 200 points and `cut = 3`.
    pub fn grid(&self, points: usize, cut: f64) -> Vec<(f64, f64)> {
        let [min, max] = self.extent;
        let (start, stop) = (min - cut * self.h, max + cut * self.h);
        let step = (stop - start) / points.saturating_sub(1).max(1) as f64;
        (0..points)
            .map(|i| {
                let x = if i + 1 == points && points > 1 {
                    stop
                } else {
                    i as f64 * step + start
                };
                (x, self.eval(x))
            })
            .collect()
    }

    /// The same estimate with bandwidth `h` times `adjust` (seaborn's
    /// `bw_adjust`), or Scott's times `adjust` without `h`; as it was when
    /// that is not finite and positive.
    pub(crate) fn rescaled(self, h: Option<f64>, adjust: f64) -> Kde {
        let scaled = h.unwrap_or(self.h) * adjust;
        if scaled.is_finite() && scaled > 0.0 {
            Kde { h: scaled, ..self }
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_few_or_equal_values_have_no_estimate() {
        assert!(Kde::new([]).is_none());
        assert!(Kde::new([1.0]).is_none());
        assert!(Kde::new([1.0, f64::INFINITY, f64::NAN]).is_none());
        assert!(Kde::new([3.0, 3.0, 3.0]).is_none());
        assert!(Kde::new([3.0, 4.0]).is_some());
    }

    #[test]
    fn rescaling_multiplies_the_bandwidth_it_starts_from() {
        let kde = Kde::new([1.0, 2.0, 4.0]).expect("values that vary");
        let scott = kde.bandwidth();
        assert_eq!(kde.clone().rescaled(None, 2.0).bandwidth(), 2.0 * scott);
        assert_eq!(kde.clone().rescaled(Some(0.5), 3.0).bandwidth(), 1.5);
        assert_eq!(kde.clone().rescaled(Some(-1.0), 1.0).bandwidth(), scott);
        assert_eq!(kde.rescaled(None, f64::NAN).bandwidth(), scott);
    }

    #[test]
    fn a_grid_of_one_point_sits_at_the_low_end() {
        let kde = Kde::new([0.0, 1.0]).expect("values that vary");
        let grid = kde.grid(1, 0.0);
        assert_eq!(grid.len(), 1);
        assert_eq!(grid[0].0, 0.0);
        assert!(kde.grid(0, 3.0).is_empty());
    }
}
