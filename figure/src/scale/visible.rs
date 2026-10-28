//! [`Memory`]: what `Fit::Visible` handles remember between resolves, and
//! the step rule that keeps their domains still while the data wobbles.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use rustd3_core::jiff::Timestamp;

/// Each step a domain shrinks by is this share of its previous span.
const STEPS: f64 = 8.0;

/// The last domain of each `Fit::Visible` handle, keyed by its index in
/// the scale table and its scale's type, which survive a figure's rebuild:
/// kept in [`figure::Cache`](crate::Cache), so the widget's cache carries
/// it from frame to frame.
#[derive(Debug, Clone, Default)]
pub(crate) struct Memory {
    domains: HashMap<(usize, TypeId), Domain>,
}

/// A remembered domain: numbers, or instants as `i128` nanoseconds from
/// the epoch, so no `Timestamp` passes through `f64`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Domain {
    /// An `f64` domain's ends.
    Number([f64; 2]),
    /// A time domain's ends, in nanoseconds from the epoch.
    Time([i128; 2]),
}

impl Domain {
    /// `ends` remembered, when they are numbers or instants.
    pub(crate) fn of<D: 'static>(ends: &[D; 2]) -> Option<Domain> {
        let any = ends as &dyn Any;
        if let Some(&numbers) = any.downcast_ref::<[f64; 2]>() {
            return Some(Domain::Number(numbers));
        }
        let instants = any.downcast_ref::<[Timestamp; 2]>()?;
        Some(Domain::Time(instants.map(Timestamp::as_nanosecond)))
    }

    /// The ends as `[D; 2]`, when `D` is the remembered type and, for
    /// instants, both lie in jiff's range.
    pub(crate) fn ends<D: Clone + 'static>(self) -> Option<[D; 2]> {
        match self {
            Domain::Number(numbers) => (&numbers as &dyn Any).downcast_ref().cloned(),
            Domain::Time([a, b]) => {
                let instants = [instant(a)?, instant(b)?];
                (&instants as &dyn Any).downcast_ref().cloned()
            }
        }
    }
}

impl Memory {
    /// The domain of handle `key` now that its data pads to `padded`,
    /// remembered for the next resolve: `padded` the first time; then each
    /// end grows at once to `padded`'s, or moves inward by whole steps of
    /// one eighth of the last domain's span while `padded` still fits
    /// (whole nanoseconds for instants). Domains keep their orientation.
    pub(crate) fn settle(&mut self, key: (usize, TypeId), padded: Domain) -> Domain {
        let settled = match (self.domains.get(&key), padded) {
            (Some(&Domain::Number(last)), Domain::Number(padded)) => {
                Domain::Number(step(last, padded))
            }
            (Some(&Domain::Time(last)), Domain::Time(padded)) => {
                Domain::Time(step_ns(last, padded))
            }
            (Some(&Domain::Number(_)) | Some(&Domain::Time(_)) | None, padded) => padded,
        };
        self.domains.insert(key, settled);
        settled
    }

    /// The last domain of handle `key`, if it has one.
    pub(crate) fn last(&self, key: (usize, TypeId)) -> Option<Domain> {
        self.domains.get(&key).copied()
    }
}

/// The instant `ns` nanoseconds from the epoch, when it lies in jiff's
/// range (jiff's own constructor does not check it, D27.2 (7)).
pub(crate) fn instant(ns: i128) -> Option<Timestamp> {
    let range = Timestamp::MIN.as_nanosecond()..=Timestamp::MAX.as_nanosecond();
    range
        .contains(&ns)
        .then(|| Timestamp::from_nanosecond(ns).ok())
        .flatten()
}

/// [`step`] in whole nanoseconds: each inward step is a whole eighth of
/// `last`'s span, rounded down.
fn step_ns(last: [i128; 2], padded: [i128; 2]) -> [i128; 2] {
    let reversed = padded[0] > padded[1];
    if reversed != (last[0] > last[1]) {
        return padded;
    }
    let order = |[a, b]: [i128; 2]| if reversed { [b, a] } else { [a, b] };
    let ([lo, hi], [low, high]) = (order(last), order(padded));
    let step = (hi - lo) / 8;
    if step <= 0 {
        return padded;
    }
    let lo = if low < lo {
        low
    } else {
        lo + (low - lo) / step * step
    };
    let hi = if high > hi {
        high
    } else {
        hi - (hi - high) / step * step
    };
    order([lo, hi])
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
        let number = Domain::Number;
        assert_eq!(
            memory.settle(key, number([-5.0, 105.0])),
            number([-5.0, 105.0])
        );
        assert_eq!(
            memory.settle(key, number([39.0, 61.0])),
            number([36.25, 63.75])
        );
        assert_eq!(memory.last(key), Some(number([36.25, 63.75])));
        let other = (3, TypeId::of::<f32>());
        assert_eq!(
            memory.settle(other, number([39.0, 61.0])),
            number([39.0, 61.0])
        );
    }

    #[test]
    fn instants_step_by_whole_nanoseconds() {
        // The ramp's rows in nanoseconds: steps of 110 / 8 → 13 ns.
        assert_eq!(step_ns([-5, 105], [39, 61]), [34, 66]);
        assert_eq!(step_ns([105, -5], [61, 39]), [66, 34]);
        assert_eq!(step_ns([0, 80], [35, 90]), [30, 90]);
        // A span under 8 ns has no step.
        assert_eq!(step_ns([0, 7], [3, 4]), [3, 4]);
        let mut memory = Memory::default();
        let key = (0, TypeId::of::<Timestamp>());
        let t = Domain::Time([1_000, 2_000]);
        assert_eq!(memory.settle(key, t), t);
        assert_eq!(
            memory.settle(key, Domain::Number([0.0, 1.0])),
            Domain::Number([0.0, 1.0])
        );
    }

    #[test]
    fn remembered_ends_come_back_in_their_own_type() {
        let t = Timestamp::UNIX_EPOCH;
        let day = [t, t + rustd3_core::jiff::SignedDuration::from_hours(24)];
        let remembered = Domain::of(&day).expect("instants");
        assert_eq!(remembered, Domain::Time([0, 86_400_000_000_000]));
        assert_eq!(remembered.ends::<Timestamp>(), Some(day));
        assert_eq!(remembered.ends::<f64>(), None);
        assert_eq!(Domain::of(&[1.0, 2.0]), Some(Domain::Number([1.0, 2.0])));
        assert_eq!(Domain::of(&["a", "b"]), None);
        assert_eq!(Domain::Time([i128::MAX, 0]).ends::<Timestamp>(), None);
    }
}
