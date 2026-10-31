//! d3-format's precision suggestions: how many digits a format needs to
//! tell values `step` apart. `None` where d3 returns NaN: a step (or
//! value) of zero, NaN or ±∞.

use crate::format::decimal;

/// d3.precisionFixed: decimal places for fixed-point values `step` apart
/// (`precision_fixed(0.01)` is 2).
#[doc(alias = "precisionFixed")]
#[must_use]
pub fn precision_fixed(step: f64) -> Option<usize> {
    decimal::exponent(step).map(|e| decimal::count(-e))
}

/// [`precision_fixed`] for a step measured between values of magnitude up
/// to `max`: the step is read to 12 significant digits of `max` first, so
/// that the float noise of a difference (0.3 − 0.2 = 0.09999999999999998;
/// 1e9 + 0.3 − (1e9 + 0.2) = 0.09999990463256836) adds no place. `None`
/// for a step that is zero, not finite or below that resolution.
pub(crate) fn precision_fixed_between(step: f64, max: f64) -> Option<usize> {
    let step = step.abs();
    let digits = 12 + decimal::exponent(step)? - decimal::exponent(max.abs().max(step))?;
    let digits = usize::try_from(digits).ok().filter(|digits| *digits > 0)?;
    decimal::parts(step, digits).map(|(_, e)| decimal::count(-e))
}

/// d3.precisionPrefix: decimal places for values `step` apart formatted
/// with `value`'s SI prefix (`precision_prefix(1e5, 1.3e6)` is 1, "1.3M").
#[doc(alias = "precisionPrefix")]
#[must_use]
pub fn precision_prefix(step: f64, value: f64) -> Option<usize> {
    let value = decimal::exponent(value)?;
    let step = decimal::exponent(step)?;
    Some(decimal::count(decimal::si_exponent(value) - step))
}

/// d3.precisionRound: significant digits for values `step` apart up to
/// `max` (`precision_round(0.01, 1.01)` is 3).
#[doc(alias = "precisionRound")]
#[must_use]
pub fn precision_round(step: f64, max: f64) -> Option<usize> {
    let step = step.abs();
    let max = decimal::exponent(max.abs() - step)?;
    let step = decimal::exponent(step)?;
    Some(decimal::count(max - step) + 1)
}
