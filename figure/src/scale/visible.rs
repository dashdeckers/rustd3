//! [`Memory`]: what `Fit::Visible` handles remember between resolves, and
//! the step rule that keeps their domains still while the data wobbles.

use std::any::TypeId;
use std::collections::HashMap;

/// Each step a domain shrinks by is this share of its previous span.
const STEPS: f64 = 8.0;

/// The last domain of each `Fit::Visible` handle, keyed by its index in
/// the scale table and its scale's type, which survive a figure's rebuild:
/// kept in [`figure::Cache`](crate::Cache), so the widget's cache carries
/// it from frame to frame.
#[derive(Debug, Default)]
pub(crate) struct Memory {
    domains: HashMap<(usize, TypeId), [f64; 2]>,
}

impl Memory {
    /// The domain of handle `key` now that its data pads to `padded`,
    /// remembered for the next resolve: `padded` the first time; then each
    /// end grows at once to `padded`'s, or moves inward by whole steps of
    /// one eighth of the last domain's span while `padded` still fits.
    /// Domains keep their orientation.
    pub(crate) fn settle(&mut self, key: (usize, TypeId), padded: [f64; 2]) -> [f64; 2] {
        let settled = match self.domains.get(&key) {
            Some(&last) => step(last, padded),
            None => padded,
        };
        self.domains.insert(key, settled);
        settled
    }

    /// The last domain of handle `key`, if it has one.
    pub(crate) fn last(&self, key: (usize, TypeId)) -> Option<[f64; 2]> {
        self.domains.get(&key).copied()
    }
}

/// The domain after `last` toward `padded` (both in increasing or both in
/// decreasing order): each end grows at once past `padded`'s, or moves
/// inward by the most whole steps of `last`'s span / 8 that keep `padded`
/// inside; `padded` when the two disagree in order or a span is not
/// finite and positive.
fn step(last: [f64; 2], padded: [f64; 2]) -> [f64; 2] {
    let reversed = padded[0] > padded[1];
    if reversed != (last[0] > last[1]) {
        return padded;
    }
    let order = |[a, b]: [f64; 2]| if reversed { [b, a] } else { [a, b] };
    let ([lo, hi], [low, high]) = (order(last), order(padded));
    let step = (hi - lo) / STEPS;
    if !(step.is_finite() && step > 0.0 && low.is_finite() && high.is_finite()) {
        return padded;
    }
    let lo = if low < lo {
        low
    } else {
        lo + ((low - lo) / step).floor() * step
    };
    let hi = if high > hi {
        high
    } else {
        hi - ((hi - high) / step).floor() * step
    };
    order([lo, hi])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_domain_shrinks_by_whole_eighths_and_grows_at_once() {
        // The ramp y = x: x over [0, 100], then [40, 60], then [0, 100].
        let shrunk = step([-5.0, 105.0], [39.0, 61.0]);
        assert_eq!(shrunk, [36.25, 63.75]);
        // 27.5 / 8 = 3.4375 would cut into [39, 61]: it stays.
        assert_eq!(step(shrunk, [39.0, 61.0]), shrunk);
        assert_eq!(step(shrunk, [-5.0, 105.0]), [-5.0, 105.0]);
        // One end grows while the other shrinks.
        assert_eq!(step([0.0, 80.0], [35.0, 90.0]), [30.0, 90.0]);
    }

    #[test]
    fn reversed_domains_step_the_same_way() {
        assert_eq!(step([105.0, -5.0], [61.0, 39.0]), [63.75, 36.25]);
        assert_eq!(step([105.0, -5.0], [39.0, 61.0]), [39.0, 61.0]);
    }

    #[test]
    fn a_handle_remembers_its_domain_by_index_and_type() {
        let mut memory = Memory::default();
        let key = (3, TypeId::of::<f64>());
        assert_eq!(memory.settle(key, [-5.0, 105.0]), [-5.0, 105.0]);
        assert_eq!(memory.settle(key, [39.0, 61.0]), [36.25, 63.75]);
        assert_eq!(memory.last(key), Some([36.25, 63.75]));
        let other = (3, TypeId::of::<f32>());
        assert_eq!(memory.settle(other, [39.0, 61.0]), [39.0, 61.0]);
    }
}
