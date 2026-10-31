//! [`Ticks`] for d3's identity scale, which ticks as a linear scale over
//! its stops.

use crate::locale::Locale;
use crate::scale::{self, Ticks, transform};
use crate::ticks;

/// The continuous impl with the stops as domain and extent.
impl Ticks for scale::Identity {
    type Domain = f64;
    type Step = f64;

    /// The scale's `ticks(count)` with d3's tickFormat step.
    fn tick_set(&self, count: usize) -> ticks::Set<f64, f64> {
        let [start, stop] = self.get_domain();
        let step = ticks::tick_step(start, stop, count as f64);
        ticks::Set::new(self.ticks(count), step, count)
    }

    /// `apply(value)`; the offset is unused.
    fn position(&self, value: &f64, _offset: f64) -> Option<f64> {
        Some(self.apply(*value))
    }

    /// d3's linear tick format for the set's count under `locale`.
    fn labels(&self, ticks: &ticks::Set<f64, f64>, locale: &Locale) -> Vec<String> {
        self.tick_format_in(locale.number(), ticks.count(), None)
            .expect("invariant: the default tick format specifier parses")
            .labels(ticks, locale)
    }

    /// Five equal steps between majors extended by one step on each side,
    /// inside the stops and off the majors.
    fn minor(&self, ticks: &ticks::Set<f64, f64>) -> Vec<f64> {
        super::minor(&transform::Identity, self.get_domain(), ticks.values())
    }

    fn extent(&self) -> [f64; 2] {
        self.get_range()
    }
}

impl crate::sealed::Ticks for scale::Identity {}
