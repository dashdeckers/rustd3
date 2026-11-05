//! [`Kde`]: a Gaussian kernel density estimate with Scott's bandwidth,
//! exact or, over many values, binned.

use std::f64::consts::TAU;

use rustd3_core::array;

/// A Gaussian kernel density estimate with Scott's bandwidth, as scipy's
/// `gaussian_kde` and seaborn's `kdeplot` compute it: `h = σ̂ · n^(−1/5)`,
/// σ̂ the sample standard deviation (n − 1), and the density at x the mean
/// of the n kernels φ((x − xᵢ)/h)/h, φ the standard normal density.
///
/// Over more than [`Kde::BINNED`] values, [`grid`](Kde::grid) bins the
/// values linearly onto a fine grid aligned with its points and convolves
/// the bins with the kernel ([`binned`](Kde::binned)). Over a million
/// values that takes milliseconds where the exact sum takes seconds, and
/// stays within a relative 1e-3 of it.
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

/// The least number of binned-grid nodes per bandwidth. Linear binning's
/// relative error at a kernel u bandwidths away is then at most
/// (u² − 1) / (8 · 128²), 5e-5 three bandwidths out.
const NODES_PER_BANDWIDTH: f64 = 128.0;

/// How far the binned convolution reaches, in bandwidths: beyond it the
/// kernel, exp(−u²/2), is below 1e-297 and adds nothing an `f64` keeps.
const REACH: f64 = 37.0;

/// The most points a grid has: 2³¹ or more are not allocated.
const MOST_POINTS: usize = i32::MAX as usize;

/// The most nodes a binned grid takes (32 MiB of counts); values spread
/// over more bandwidths than that allows are estimated exactly.
const MOST_NODES: usize = 1 << 22;

impl Kde {
    /// Above this many values, [`grid`](Kde::grid) is
    /// [`binned`](Kde::binned); at or below it, [`exact`](Kde::exact).
    pub const BINNED: usize = 10_000;

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
    #[must_use]
    pub fn bandwidth(&self) -> f64 {
        self.h
    }

    /// The density at `x`.
    #[must_use]
    pub fn eval(&self, x: f64) -> f64 {
        let sum: f64 = self
            .values
            .iter()
            .map(|v| (-0.5 * ((x - v) / self.h).powi(2)).exp())
            .sum();
        sum / (self.values.len() as f64 * self.h * TAU.sqrt())
    }

    /// `points` positions evenly spaced over `[min − cut·h, max + cut·h]`
    /// (numpy's `linspace`, both ends included), each with its density.
    /// This is seaborn's grid, which `density_y` takes at 200 points and
    /// `cut = 3`. It is [`exact`](Kde::exact) for at most [`Kde::BINNED`]
    /// values and [`binned`](Kde::binned) above. Like every grid of this
    /// type it is empty for 2³¹ points or more, since such a grid is not
    /// allocated.
    #[must_use]
    pub fn grid(&self, points: usize, cut: f64) -> Vec<(f64, f64)> {
        if self.values.len() > Kde::BINNED {
            self.binned(points, cut)
        } else {
            self.exact(points, cut)
        }
    }

    /// [`grid`](Kde::grid)'s positions with the density at each summed over
    /// every value: `points` × n kernels.
    #[must_use]
    pub fn exact(&self, points: usize, cut: f64) -> Vec<(f64, f64)> {
        self.support(points, cut)
            .into_iter()
            .map(|x| (x, self.eval(x)))
            .collect()
    }

    /// [`grid`](Kde::grid)'s positions with the density at each computed
    /// from the values binned. Each value's weight is split linearly
    /// between the two nearest nodes of a grid at least 128 nodes per
    /// bandwidth fine, whose every m-th node is a position. Each position's
    /// density is the bins convolved with the kernel out to 37 bandwidths,
    /// with the kernel weights computed once. It takes n + `points` × 9,500
    /// steps or so, where [`exact`](Kde::exact) takes `points` × n kernels,
    /// and stays within a relative 1e-3 of it. With fewer than two points,
    /// a negative `cut` or values spread over more than about 32,000
    /// bandwidths, it is `exact`.
    #[must_use]
    pub fn binned(&self, points: usize, cut: f64) -> Vec<(f64, f64)> {
        let support = self.support(points, cut);
        let (Some(&start), Some(&stop)) = (support.first(), support.last()) else {
            return Vec::new();
        };
        let step = (stop - start) / (points.saturating_sub(1)) as f64;
        let per = (step * NODES_PER_BANDWIDTH / self.h).ceil();
        let fits = points >= 2
            && cut >= 0.0
            && step > 0.0
            && per.is_finite()
            && per * (points - 1) as f64 + 1.0 <= MOST_NODES as f64;
        if !fits {
            return self.exact(points, cut);
        }
        let per = (per as usize).max(1);
        let nodes = (points - 1) * per + 1;
        let delta = step / per as f64;
        let mut bins = vec![0.0; nodes];
        for v in &self.values {
            let t = ((v - start) / delta).max(0.0);
            let j = (t.floor() as usize).min(nodes - 1);
            let frac = (t - j as f64).clamp(0.0, 1.0);
            match bins.get_mut(j + 1) {
                Some(next) => {
                    *next += frac;
                    bins[j] += 1.0 - frac;
                }
                None => bins[j] += 1.0,
            }
        }
        let reach = ((REACH * self.h / delta).ceil() as usize).min(nodes - 1);
        let kernel: Vec<f64> = (0..=reach)
            .map(|k| (-0.5 * (k as f64 * delta / self.h).powi(2)).exp())
            .collect();
        let norm = self.values.len() as f64 * self.h * TAU.sqrt();
        support
            .into_iter()
            .enumerate()
            .map(|(i, x)| {
                let at = i * per;
                let (low, high) = (at.saturating_sub(reach), (at + reach).min(nodes - 1));
                let sum: f64 = (low..=high).map(|j| bins[j] * kernel[j.abs_diff(at)]).sum();
                (x, sum / norm)
            })
            .collect()
    }

    /// `points` positions evenly spaced over `[min − cut·h, max + cut·h]`,
    /// both ends included.
    fn support(&self, points: usize, cut: f64) -> Vec<f64> {
        if points > MOST_POINTS {
            return Vec::new();
        }
        let [min, max] = self.extent;
        let (start, stop) = (min - cut * self.h, max + cut * self.h);
        let step = (stop - start) / points.saturating_sub(1).max(1) as f64;
        (0..points)
            .map(|i| {
                if i + 1 == points && points > 1 {
                    stop
                } else {
                    i as f64 * step + start
                }
            })
            .collect()
    }

    /// The same estimate with bandwidth `h` times `adjust` (seaborn's
    /// `bw_adjust`), or Scott's times `adjust` without `h`; unchanged when
    /// that product is not finite and positive.
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

    /// xorshift64* from `seed`, uniform in [0, 1).
    fn uniform(seed: u64) -> impl FnMut() -> f64 {
        let mut x = seed.max(1);
        move || {
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1_u64 << 53) as f64
        }
    }

    /// `n` draws of a standard normal (Box–Muller), from `seed`.
    fn normal(n: usize, seed: u64) -> Vec<f64> {
        let mut u = uniform(seed);
        (0..n)
            .map(|_| {
                let (a, b) = (u().max(1e-300), u());
                (-2.0 * a.ln()).sqrt() * (TAU * b).cos()
            })
            .collect()
    }

    /// The largest relative difference of the binned grid from the exact
    /// one, which must share its positions.
    fn worst(kde: &Kde) -> f64 {
        let (binned, exact) = (kde.binned(200, 3.0), kde.exact(200, 3.0));
        assert_eq!(binned.len(), 200);
        binned
            .iter()
            .zip(&exact)
            .map(|(b, e)| {
                assert_eq!(b.0, e.0, "the same positions");
                ((b.1 - e.1) / e.1).abs()
            })
            .fold(0.0, f64::max)
    }

    #[test]
    fn a_binned_bimodal_estimate_is_within_a_thousandth_of_the_exact_one() {
        // Two modes 6 apart, 60:40, with a near-empty valley between and
        // one far value: 100k values.
        let mut u = uniform(7);
        let values: Vec<f64> = normal(100_000, 11)
            .into_iter()
            .map(|z| if u() < 0.6 { z - 3.0 } else { 0.5 * z + 3.0 })
            .chain([9.0])
            .collect();
        let kde = Kde::new(values).expect("values that vary");
        let worst = worst(&kde);
        assert!(worst <= 1e-3, "{worst}");
        assert_eq!(kde.grid(200, 3.0), kde.binned(200, 3.0));
    }

    #[test]
    fn at_the_threshold_the_two_estimates_agree() {
        // At BINNED values the grid is exact, one more and it is binned;
        // on the same values the two agree within a thousandth.
        let values = normal(Kde::BINNED + 1, 3);
        let at = Kde::new(values[..Kde::BINNED].iter().copied()).expect("values that vary");
        let above = Kde::new(values).expect("values that vary");
        assert_eq!(at.grid(200, 3.0), at.exact(200, 3.0));
        assert_eq!(above.grid(200, 3.0), above.binned(200, 3.0));
        for kde in [&at, &above] {
            let worst = worst(kde);
            assert!(worst <= 1e-3, "{worst}");
        }
    }

    #[test]
    fn a_grid_of_an_enormous_count_is_empty() {
        let kde = Kde::new([1.0, 2.0, 4.0]).expect("values that vary");
        for points in [1 << 31, usize::MAX] {
            assert!(kde.grid(points, 3.0).is_empty());
            assert!(kde.exact(points, 3.0).is_empty());
            assert!(kde.binned(points, 3.0).is_empty());
        }
    }

    #[test]
    fn small_inputs_and_odd_grids_stay_exact() {
        // The scipy fixture's 15 values are far below the threshold.
        let kde = Kde::new([1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 14.0]).expect("values");
        assert_eq!(kde.grid(200, 3.0), kde.exact(200, 3.0));
        // A negative cut or a single point fall back to the exact sum.
        assert_eq!(kde.binned(50, -1.0), kde.exact(50, -1.0));
        assert_eq!(kde.binned(1, 3.0), kde.exact(1, 3.0));
        assert!(kde.binned(0, 3.0).is_empty());
        // A cut of 0 puts the greatest value on the last node.
        let worst = kde
            .binned(64, 0.0)
            .iter()
            .zip(kde.exact(64, 0.0))
            .map(|(b, e)| ((b.1 - e.1) / e.1).abs())
            .fold(0.0, f64::max);
        assert!(worst <= 1e-3, "{worst}");
    }

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
