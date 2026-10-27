//! [`Ticks`] for every continuous scale with an `f64` range, piecewise and
//! [`AnyContinuous`](crate::scale::AnyContinuous) ones included, and for
//! those with a color range, whose positions lie in [0, 1].

use crate::color::Rgb;
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

    /// The scale's default tick format for the set's count under `locale`
    /// ([`Continuous::tick_format_in`] with no specifier): d3's
    /// `,.{precision}f` for linear-like transforms, d3's blanked log labels
    /// for log, `,f` to the places of the ticks' smallest gap under a tick
    /// rule.
    ///
    /// # Panics
    ///
    /// When the transform's `tick_format` rejects its own default
    /// specifier, which [`Transform::tick_format`] rules out.
    fn labels(&self, ticks: &ticks::Set<f64, f64>, locale: &Locale) -> Vec<String> {
        super::labels(
            self.tick_format_in(locale.number(), ticks.count(), None),
            ticks,
            locale,
        )
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

/// A color range positions ticks in [0, 1] for a colorbar: ticks, step,
/// labels and minor ticks as for an `f64` range, the position the end
/// stops' transformed fraction.
///
/// ```
/// use rustd3_core::color::Rgb;
/// use rustd3_core::scale::{self, Ticks};
///
/// let heat = scale::linear().domain([0.0, 50.0]).range([Rgb::hex(0xffffff), Rgb::hex(0x4682b4)]);
/// assert_eq!(heat.position(&25.0, 0.0), Some(0.5));
/// assert_eq!(heat.extent(), [0.0, 1.0]);
/// ```
impl<T: Transform, const N: usize> Ticks for Continuous<T, Rgb, N> {
    type Domain = f64;
    type Step = f64;

    /// The scale's `ticks(count)`, by its tick rule.
    fn tick_set(&self, count: usize) -> ticks::Set<f64, f64> {
        let d = self.get_domain();
        let step = ticks::tick_step(d[0], d[N - 1], count as f64);
        ticks::Set::new(self.ticks(count), step, count)
    }

    /// Where `value` lies between the end stops in transformed space,
    /// before clamping and the colors; `None` for NaN.
    fn position(&self, value: &f64, _offset: f64) -> Option<f64> {
        Some(self.fraction(*value)).filter(|t| !value.is_nan() && !t.is_nan())
    }

    /// As for an `f64` range: the scale's default tick format for the
    /// set's count under `locale`.
    fn labels(&self, ticks: &ticks::Set<f64, f64>, locale: &Locale) -> Vec<String> {
        super::labels(
            self.tick_format_in(locale.number(), ticks.count(), None),
            ticks,
            locale,
        )
    }

    /// As for an `f64` range.
    fn minor(&self, ticks: &ticks::Set<f64, f64>) -> Vec<f64> {
        let d = self.get_domain();
        super::minor(self.transform(), [d[0], d[N - 1]], ticks.values())
    }

    /// `[0, 1]`: a colorbar maps it to pixels.
    fn extent(&self) -> [f64; 2] {
        [0.0, 1.0]
    }
}
