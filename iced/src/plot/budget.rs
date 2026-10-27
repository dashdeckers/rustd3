//! [`Budget`]: the frame budget from the last two `RedrawRequested`
//! instants, and the cost model's prediction, calibrated by the raster
//! times a probe measures.

use iced_core::time::{Duration, Instant};

/// The shortest frame budget.
const SHORTEST: Duration = Duration::from_millis(1);

/// The longest frame budget, and the budget until two frames are known:
/// an idle app's long gap must not make a slow render look cheap.
const LONGEST: Duration = Duration::from_micros(16_700);

/// Raster samples folded before the calibration applies.
const WARMUP: u32 = 8;

/// The weight of a new sample in the calibration's moving average.
const ALPHA: f64 = 1.0 / 16.0;

/// The frame interval and the calibration of the raster cost model.
#[derive(Debug, Default)]
pub(crate) struct Budget {
    /// Last two `RedrawRequested` instants.
    last: [Option<Instant>; 2],
    /// Exponentially weighted mean of measured over modelled raster time,
    /// with a probe.
    calibration: Option<f64>,
    /// Raster samples folded into `calibration`.
    samples: u32,
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

    /// Folds one raster time, `measured`, against the model's `model_ms`
    /// for the same cell: the first ratio starts the mean, later ones move
    /// it by 1/16 of their difference.
    pub(crate) fn calibrate(&mut self, measured: Duration, model_ms: f64) {
        let ratio = measured.as_secs_f64() * 1e3 / model_ms;
        if !ratio.is_finite() {
            return;
        }
        self.calibration = Some(match self.calibration {
            None => ratio,
            Some(mean) => mean + (ratio - mean) * ALPHA,
        });
        self.samples += 1;
    }

    /// The predicted raster time of cells the model gives `model_ms`:
    /// times the calibration from the 8th sample on.
    pub(crate) fn predict(&self, model_ms: f64) -> f64 {
        match self.calibration {
            Some(factor) if self.samples >= WARMUP => model_ms * factor,
            Some(_) | None => model_ms,
        }
    }

    /// Whether cells the model gives `model_ms` render inline: their
    /// prediction is at most half the frame budget.
    pub(crate) fn is_inline(&self, model_ms: f64) -> bool {
        // Nanoseconds over 1e6 round once: 6.9 ms halves to exactly 3.45.
        self.predict(model_ms) <= self.interval().as_nanos() as f64 / 1e6 / 2.0
    }
}

#[cfg(test)]
mod tests {
    use iced_core::time::{Duration, Instant};

    use super::Budget;
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
    fn cheap_cells_are_predicted_inline() {
        let budget = Budget::default();
        // 320 × 200 at 2×, 150 items, 16.7 ms: 0.89 ms ≤ 8.35.
        assert!(budget.is_inline(model_ms(256_000, 150)));
        // The wheel figure without its dots at 60 Hz: 7.7 ms ≤ 8.35.
        assert!(budget.is_inline(model_ms(2_918_400, 760)));
        // At 144 Hz the threshold is 3.45 ms.
        let fast = after(Duration::from_micros(6_900));
        assert!(fast.is_inline(3.45));
        assert!(fast.is_inline(model_ms(256_000, 150)));
    }

    #[test]
    fn expensive_cells_are_predicted_for_the_worker() {
        let budget = Budget::default();
        // The whole wheel figure: 21.7 ms > 8.35.
        assert!(!budget.is_inline(model_ms(2_918_400, 6_360)));
        // 200k dots at 1600 × 1000 logical at 2×: 513 ms.
        assert!(!budget.is_inline(model_ms(6_400_000, 200_000)));
        // The figure without its dots at 144 Hz: 7.7 ms > 3.45.
        let fast = after(Duration::from_micros(6_900));
        assert!(!fast.is_inline(model_ms(2_918_400, 760)));
        assert!(!fast.is_inline(3.46));
    }

    #[test]
    fn calibration_applies_after_eight_samples() {
        let cells = 3.0 * model_ms(300 * 200, 100);
        let mut budget = after(Duration::from_millis(2));
        // 1.11 ms > 1 ms: the worker, until the calibration applies.
        for _ in 0..7 {
            budget.calibrate(Duration::from_micros(500), 1.0);
            assert_eq!(budget.predict(cells), cells);
        }
        assert!(!budget.is_inline(cells));
        budget.calibrate(Duration::from_micros(500), 1.0);
        assert!((budget.predict(cells) - 0.555).abs() < 1e-12);
        assert!(budget.is_inline(cells), "(0.36 + 0.75) × 0.5 ≤ 1");
        // The moving average: the first ratio, then 1/16 of each change.
        let mut budget = Budget::default();
        budget.calibrate(Duration::from_millis(1), 1.0);
        budget.calibrate(Duration::from_millis(2), 1.0);
        assert_eq!(budget.calibration, Some(1.0 + 1.0 / 16.0));
        // A modelled time of zero teaches nothing.
        budget.calibrate(Duration::from_millis(1), 0.0);
        assert_eq!(budget.samples, 2);
    }
}
