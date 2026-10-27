//! [`Ticks`] for every continuous scale with an `f64` range, piecewise and
//! [`AnyContinuous`](crate::scale::AnyContinuous) ones included.

use crate::locale::Locale;
use crate::scale::{Continuous, Ticks, Transform};
use crate::ticks;

/// Ticks act on the end stops, as in d3; the step is d3's tickFormat step
/// and labels are the transform's tick format under the axis locale.
impl<T: Transform, const N: usize> Ticks for Continuous<T, f64, N> {
    type Domain = f64;
    type Step = f64;

    /// The scale's `ticks(count)`, by its tick rule.
    fn tick_set(&self, count: usize) -> ticks::Set<f64, f64> {
        let d = self.get_domain();
        let step = ticks::tick_step(d[0], d[N - 1], count as f64);
        ticks::Set::new(self.ticks(count), step, count)
    }

    /// `apply(value)`, non-finite results included; the offset is unused
    /// (continuous scales have no bandwidth).
    fn position(&self, value: &f64, _offset: f64) -> Option<f64> {
        Some(self.apply(*value))
    }

    /// The transform's default tick format for the set's count under
    /// `locale`: d3's `,.{precision}f` for linear-like transforms, d3's
    /// blanked log labels for log.
    ///
    /// # Panics
    ///
    /// When the transform's `tick_format` rejects its own default
    /// specifier, which [`Transform::tick_format`] rules out.
    fn labels(&self, ticks: &ticks::Set<f64, f64>, locale: &Locale) -> Vec<String> {
        self.tick_format_in(locale.number(), ticks.count(), None)
            .expect("invariant: a transform's default tick format specifier parses")
            .labels(ticks, locale)
    }

    /// The transform's minor ticks between the majors extended by one step
    /// on each side in transformed space, inside the domain and off the
    /// majors; none for fewer than two majors.
    fn minor(&self, ticks: &ticks::Set<f64, f64>) -> Vec<f64> {
        let d = self.get_domain();
        super::minor(self.transform(), [d[0], d[N - 1]], ticks.values())
    }

    fn extent(&self) -> [f64; 2] {
        let r = self.get_range();
        [r[0], r[N - 1]]
    }
}
