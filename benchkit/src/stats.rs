//! Percentiles, means and [`Interval`]s, which summarize a per-run
//! statistic across runs. Intervals use Student's t at 0.975 (2.776 for
//! n = 5).

/// A per-run statistic across runs: its count, median, mean and 95%
/// confidence interval (`None` below two runs).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    /// The number of runs.
    pub n: usize,
    /// Median of the runs.
    pub median: f64,
    /// Mean of the runs.
    pub mean: f64,
    /// The interval mean ± t(0.975, n − 1)·s/√n, from two runs on.
    pub ci: Option<[f64; 2]>,
}

/// Where an interval lies against a budget (operational reading 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The interval lies entirely below the budget.
    Met,
    /// The interval lies entirely above it.
    Missed,
    /// Neither, or fewer than two runs: rerun the cell with `--runs 10`.
    Inconclusive,
}

impl Interval {
    /// The interval of the finite `values`; `None` when there are none.
    pub fn of(values: &[f64]) -> Option<Interval> {
        let values: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
        let n = values.len();
        let mean = mean(&values)?;
        let median = percentile(&values, 50.0)?;
        let ci = (n >= 2).then(|| {
            let s = sd(&values).unwrap_or(0.0);
            let half = t975(n - 1) * s / (n as f64).sqrt();
            [mean - half, mean + half]
        });
        Some(Interval {
            n,
            median,
            mean,
            ci,
        })
    }

    /// The verdict against `budget` (smaller is better).
    pub fn verdict(&self, budget: f64) -> Verdict {
        match self.ci {
            Some([_, high]) if high < budget => Verdict::Met,
            Some([low, _]) if low > budget => Verdict::Missed,
            Some(_) | None => Verdict::Inconclusive,
        }
    }
}

impl Verdict {
    /// Its name in the report.
    pub fn name(self) -> &'static str {
        match self {
            Verdict::Met => "met",
            Verdict::Missed => "missed",
            Verdict::Inconclusive => "inconclusive",
        }
    }
}

/// The `p`th percentile, linear between closest ranks (numpy's default);
/// `None` for no values.
pub fn percentile(values: &[f64], p: f64) -> Option<f64> {
    let mut sorted: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if sorted.is_empty() {
        return None;
    }
    sorted.sort_by(f64::total_cmp);
    let rank = p / 100.0 * (sorted.len() - 1) as f64;
    let (low, high) = (rank.floor() as usize, rank.ceil() as usize);
    let t = rank - low as f64;
    Some(sorted[low] + (sorted[high] - sorted[low]) * t)
}

/// The mean; `None` for no values.
pub fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

/// The sample standard deviation; `None` below two values.
pub fn sd(values: &[f64]) -> Option<f64> {
    let m = mean(values)?;
    (values.len() >= 2).then(|| {
        let ss: f64 = values.iter().map(|v| (v - m).powi(2)).sum();
        (ss / (values.len() - 1) as f64).sqrt()
    })
}

/// The least-squares slope of `y` on `x`; `None` when `x` does not vary.
pub fn slope(points: &[(f64, f64)]) -> Option<f64> {
    let n = points.len() as f64;
    if points.len() < 2 {
        return None;
    }
    let (mx, my) = (
        points.iter().map(|p| p.0).sum::<f64>() / n,
        points.iter().map(|p| p.1).sum::<f64>() / n,
    );
    let sxx: f64 = points.iter().map(|p| (p.0 - mx).powi(2)).sum();
    let sxy: f64 = points.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
    (sxx > 0.0).then(|| sxy / sxx)
}

/// Student's t at 0.975 for `df` degrees of freedom (`df` ≥ 1).
pub fn t975(df: usize) -> f64 {
    const TABLE: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
        2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
        2.052, 2.048, 2.045, 2.042,
    ];
    match df {
        0 => f64::INFINITY,
        1..=30 => TABLE[df - 1],
        31..=40 => 2.021,
        41..=60 => 2.000,
        61..=120 => 1.980,
        _ => 1.960,
    }
}

#[cfg(test)]
mod tests {
    use super::{Interval, Verdict, percentile, slope, t975};

    #[test]
    fn pan_latencies_of_80_to_100_meet_the_budget() {
        let ci = Interval::of(&[80.0, 85.0, 90.0, 95.0, 100.0]).unwrap();
        assert_eq!((ci.n, ci.mean, ci.median), (5, 90.0, 90.0));
        let [low, high] = ci.ci.unwrap();
        // s = 7.91, half-width 2.776 · 7.91 / √5 = 9.82.
        assert!(
            (low - 80.18).abs() < 0.01 && (high - 99.82).abs() < 0.01,
            "{low} {high}"
        );
        assert_eq!(ci.verdict(100.0), Verdict::Met);
    }

    #[test]
    fn pan_latencies_of_90_to_110_are_inconclusive() {
        let ci = Interval::of(&[90.0, 95.0, 100.0, 105.0, 110.0]).unwrap();
        let [low, high] = ci.ci.unwrap();
        assert!((low - 90.18).abs() < 0.01 && (high - 109.82).abs() < 0.01);
        assert_eq!(ci.verdict(100.0), Verdict::Inconclusive);
        assert_eq!(
            Interval::of(&[120.0, 125.0, 130.0]).unwrap().verdict(100.0),
            Verdict::Missed
        );
        // One run has no interval.
        assert_eq!(
            Interval::of(&[5.0]).unwrap().verdict(100.0),
            Verdict::Inconclusive
        );
        assert_eq!(Interval::of(&[]), None);
    }

    #[test]
    fn percentiles_interpolate_and_slopes_fit() {
        let values = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&values, 50.0), Some(3.0));
        assert_eq!(percentile(&values, 99.0), Some(4.96));
        assert_eq!(percentile(&[], 50.0), None);
        assert_eq!(slope(&[(0.0, 1.0), (2.0, 4.0), (4.0, 7.0)]), Some(1.5));
        assert_eq!(slope(&[(1.0, 1.0), (1.0, 4.0)]), None);
        assert_eq!(t975(4), 2.776);
    }
}
