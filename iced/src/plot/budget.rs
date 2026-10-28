//! [`Budget`]: the frame budget from the last two `RedrawRequested`
//! instants, and the widget's cost model: a fixed overhead per render plus
//! a time per pixel and per work item, fitted to every render the widget
//! makes, each timed by the widget itself whether or not a probe is
//! attached (D31.8), and seeded by [`cost`]'s constants (D22.4). The constants were
//! fitted on one machine; the fit follows the one the widget runs on, its
//! cores and its render paths, and learns the overhead small renders pay
//! (D22.1 measured 2.78 times the constants' time over a run's renders,
//! mostly small cells, against 0.18 for one large cell).
//!
//! The fit is an exponentially weighted least-squares one: each new render
//! weighs 1/16 of it (Phase 19's moving average), and a prior holds each
//! coefficient at its seed with the weight of a tenth of a render, so a
//! direction the renders never vary in (only tiles of one size, say) keeps
//! its seed while the renders decide the rest.

use iced_core::time::{Duration, Instant};

use crate::plot::cost;

/// The shortest frame budget.
const SHORTEST: Duration = Duration::from_millis(1);

/// The longest frame budget, and the budget until two frames are known:
/// an idle app's long gap must not make a slow render look cheap.
const LONGEST: Duration = Duration::from_micros(16_700);

/// The weight a render keeps for each newer one: a new render weighs
/// 1/16 of the fit.
const FORGET: f64 = 15.0 / 16.0;

/// Pixels in one unit of the fit's pixel term: a tile.
const PIXELS: f64 = 65_536.0;

/// Work items in one unit of the fit's work term.
const ITEMS: f64 = 256.0;

/// The prior's weight on each coefficient, in renders.
const PRIOR: f64 = 0.1;

/// The seed: no overhead, and the constants' ms per unit of pixels and of
/// work.
const SEED: [f64; 3] = [
    0.0,
    PIXELS / cost::PIXELS_PER_MS,
    ITEMS / cost::ITEMS_PER_MS,
];

/// The frame interval and the fitted cost model.
#[derive(Debug)]
pub(crate) struct Budget {
    /// Last two `RedrawRequested` instants.
    last: [Option<Instant>; 2],
    /// Σ w x xᵀ over the timed renders, x = (1, pixels / PIXELS, work /
    /// ITEMS), each render's w falling by `FORGET` per newer one.
    xx: [[f64; 3]; 3],
    /// Σ w x y, y the render's time in ms.
    xy: [f64; 3],
    /// The fitted overhead in ms, and ms per unit of pixels and of work.
    theta: [f64; 3],
}

impl Default for Budget {
    fn default() -> Self {
        Budget {
            last: [None; 2],
            xx: [[0.0; 3]; 3],
            xy: [0.0; 3],
            theta: SEED,
        }
    }
}

impl Budget {
    /// A `RedrawRequested` at `now`. The same instant again, which the
    /// runtime sends for each of up to three passes of one frame, or an
    /// earlier one changes nothing.
    pub(crate) fn frame(&mut self, now: Instant) {
        if self.last[1].is_none_or(|last| now > last) {
            self.last = [self.last[1], Some(now)];
        }
    }

    /// The frame budget: the interval between the last two instants,
    /// clamped to 1 ms ..= 16.7 ms; 16.7 ms until two exist.
    pub(crate) fn interval(&self) -> Duration {
        match self.last {
            [Some(earlier), Some(later)] => later
                .saturating_duration_since(earlier)
                .clamp(SHORTEST, LONGEST),
            [None | Some(_), _] => LONGEST,
        }
    }

    /// Folds one render of `pixels` and `work` that took `measured` into
    /// the fit.
    pub(crate) fn calibrate(&mut self, measured: Duration, pixels: u64, work: u64) {
        let y = measured.as_secs_f64() * 1e3;
        let x = features(pixels, work);
        if !(y.is_finite() && x.iter().all(|v| v.is_finite())) {
            return;
        }
        for i in 0..3 {
            for j in 0..3 {
                self.xx[i][j] = self.xx[i][j] * FORGET + x[i] * x[j];
            }
            self.xy[i] = self.xy[i] * FORGET + x[i] * y;
        }
        let mut a = self.xx;
        let mut b = self.xy;
        for i in 0..3 {
            a[i][i] += PRIOR;
            b[i] += PRIOR * SEED[i];
        }
        if let Some(theta) = solve(a, b) {
            self.theta = theta.map(|v| v.max(0.0));
        }
    }

    /// The predicted time of a render of `pixels` and `work`, ms.
    pub(crate) fn predict(&self, pixels: u64, work: u64) -> f64 {
        let x = features(pixels, work);
        (0..3).map(|i| self.theta[i] * x[i]).sum()
    }

    /// Whether renders predicted to take `ms` together render inline: at
    /// most half the frame budget.
    pub(crate) fn is_inline(&self, ms: f64) -> bool {
        // Nanoseconds over 1e6 round once: 6.9 ms halves to exactly 3.45.
        ms <= self.interval().as_nanos() as f64 / 1e6 / 2.0
    }
}

/// A render's terms: 1, its pixels in tiles and its work in units.
fn features(pixels: u64, work: u64) -> [f64; 3] {
    [1.0, pixels as f64 / PIXELS, work as f64 / ITEMS]
}

/// The solution of `a θ = b` by Gaussian elimination with partial
/// pivoting; `None` for a singular `a`.
fn solve(mut a: [[f64; 3]; 3], mut b: [f64; 3]) -> Option<[f64; 3]> {
    for col in 0..3 {
        let pivot = (col..3).max_by(|&i, &j| a[i][col].abs().total_cmp(&a[j][col].abs()))?;
        if a[pivot][col].abs() < 1e-12 {
            return None;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        let top = a[col];
        for row in col + 1..3 {
            let f = a[row][col] / top[col];
            for (v, above) in a[row].iter_mut().zip(top).skip(col) {
                *v -= f * above;
            }
            b[row] -= f * b[col];
        }
    }
    let mut theta = [0.0; 3];
    for row in (0..3).rev() {
        let rest: f64 = (row + 1..3).map(|k| a[row][k] * theta[k]).sum();
        theta[row] = (b[row] - rest) / a[row][row];
    }
    theta.iter().all(|v| v.is_finite()).then_some(theta)
}

#[cfg(test)]
mod tests {
    use iced_core::time::{Duration, Instant};

    use super::{Budget, solve};
    use crate::plot::cost::model_ms;

    #[expect(clippy::disallowed_methods, reason = "test clock")]
    fn clock() -> Instant {
        Instant::now()
    }

    /// A budget whose last two frames were `interval` apart.
    fn after(interval: Duration) -> Budget {
        let t = clock();
        let mut budget = Budget::default();
        budget.frame(t);
        budget.frame(t + interval);
        budget
    }

    #[test]
    fn budget_is_the_last_interval_clamped() {
        let micros = |budget: &Budget| budget.interval().as_micros();
        assert_eq!(micros(&after(Duration::from_micros(6_900))), 6_900);
        assert_eq!(micros(&after(Duration::from_secs(2))), 16_700);
        assert_eq!(micros(&after(Duration::from_micros(400))), 1_000);
        let mut budget = Budget::default();
        assert_eq!(micros(&budget), 16_700);
        let t = clock();
        budget.frame(t);
        assert_eq!(micros(&budget), 16_700, "one instant only");
        // The runtime's passes of one frame share its instant.
        budget.frame(t + Duration::from_micros(8_300));
        budget.frame(t + Duration::from_micros(8_300));
        assert_eq!(micros(&budget), 8_300);
        budget.frame(t);
        assert_eq!(micros(&budget), 8_300, "an earlier instant");
        // The last two count.
        budget.frame(t + Duration::from_micros(15_200));
        assert_eq!(micros(&budget), 6_900);
    }

    #[test]
    fn the_seed_is_the_cost_model() {
        let budget = Budget::default();
        for (pixels, work) in [(256_000, 150), (2_918_400, 760), (6_400_000, 200_000)] {
            let (a, b) = (budget.predict(pixels, work), model_ms(pixels, work));
            assert!((a - b).abs() < 1e-9 * b, "{a} against {b}");
        }
    }

    #[test]
    fn cheap_cells_are_predicted_inline() {
        let budget = Budget::default();
        // 320 × 200 at 2×, 150 items, 16.7 ms: 0.89 ms ≤ 8.35.
        assert!(budget.is_inline(budget.predict(256_000, 150)));
        // The wheel figure without its dots at 60 Hz: 7.7 ms ≤ 8.35.
        assert!(budget.is_inline(budget.predict(2_918_400, 760)));
        // At 144 Hz the threshold is 3.45 ms.
        let fast = after(Duration::from_micros(6_900));
        assert!(fast.is_inline(3.45));
        assert!(fast.is_inline(fast.predict(256_000, 150)));
    }

    #[test]
    fn expensive_cells_are_predicted_for_the_worker() {
        let budget = Budget::default();
        // The whole wheel figure: 21.7 ms > 8.35.
        assert!(!budget.is_inline(budget.predict(2_918_400, 6_360)));
        // 200k dots at 1600 × 1000 logical at 2×: 513 ms.
        assert!(!budget.is_inline(budget.predict(6_400_000, 200_000)));
        // The figure without its dots at 144 Hz: 7.7 ms > 3.45.
        let fast = after(Duration::from_micros(6_900));
        assert!(!fast.is_inline(fast.predict(2_918_400, 760)));
        assert!(!fast.is_inline(3.46));
    }

    #[test]
    fn the_cost_model_learns_an_overhead_and_its_rates_from_every_render() {
        // A machine where a render costs 0.4 ms whatever it draws, 0.1 ms
        // per 65,536 px and 0.5 ms per 256 items: small renders cost far
        // more than the constants say, a large one less.
        let truth = |pixels: u64, work: u64| {
            0.4 + 0.1 * pixels as f64 / 65_536.0 + 0.5 * work as f64 / 256.0
        };
        let renders = [
            (65_536, 0),
            (65_536, 900),
            (64_000, 120),
            (2_000, 40),
            (1_600_000, 30),
            (900, 0),
        ];
        let mut budget = Budget::default();
        for n in 0..96 {
            let (pixels, work) = renders[n % renders.len()];
            let ms = truth(pixels, work);
            budget.calibrate(Duration::from_secs_f64(ms / 1e3), pixels, work);
        }
        for (pixels, work) in [(1_800, 60), (65_536, 400), (6_400_000, 2_000)] {
            let (fit, real) = (budget.predict(pixels, work), truth(pixels, work));
            assert!(
                (fit - real).abs() < 0.03 * real,
                "{pixels} px, {work}: {fit} against {real}"
            );
        }
        // An overhead of 0.4 ms each: 21 small renders no longer fit half
        // of 16.7 ms, which the constants alone (0.02 ms each) allowed.
        let small: f64 = (0..21).map(|_| budget.predict(1_800, 0)).sum();
        assert!(!budget.is_inline(small), "{small} ms");
        let seeded = Budget::default();
        let constants: f64 = (0..21).map(|_| seeded.predict(1_800, 0)).sum();
        assert!(seeded.is_inline(constants), "{constants} ms");
    }

    #[test]
    fn renders_of_one_shape_keep_the_other_seeds() {
        // Only ring tiles: one size, no work. Their time is learnt; the
        // work rate keeps its seed.
        let mut budget = Budget::default();
        for _ in 0..64 {
            budget.calibrate(Duration::from_micros(900), 65_536, 0);
        }
        assert!((budget.predict(65_536, 0) - 0.9).abs() < 0.02);
        let per_item = (budget.predict(65_536, 2_560) - budget.predict(65_536, 0)) / 2_560.0;
        assert!((per_item - 1.0 / 400.0).abs() < 1e-4, "{per_item}");
    }

    #[test]
    fn a_singular_system_has_no_solution() {
        let a = [[1.0, 2.0, 3.0], [2.0, 4.0, 6.0], [0.0, 0.0, 1.0]];
        assert_eq!(solve(a, [1.0, 2.0, 3.0]), None);
        let a = [[2.0, 0.0, 0.0], [0.0, 4.0, 0.0], [1.0, 0.0, 1.0]];
        assert_eq!(solve(a, [2.0, 8.0, 4.0]), Some([1.0, 2.0, 3.0]));
    }
}
