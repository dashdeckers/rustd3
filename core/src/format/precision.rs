//! d3-format's precision suggestions: how many digits a format needs to
//! tell values `step` apart. `None` where d3 returns NaN: a step (or
//! value) of zero, NaN or ±∞.

use crate::format::decimal;

/// d3.precisionFixed: decimal places for fixed-point values `step` apart
/// (`precision_fixed(0.01)` is 2).
#[doc(alias = "precisionFixed")]
pub fn precision_fixed(step: f64) -> Option<usize> {
    decimal::exponent(step).map(|e| decimal::count(-e))
}

/// d3.precisionPrefix: decimal places for values `step` apart formatted
/// with `value`'s SI prefix (`precision_prefix(1e5, 1.3e6)` is 1, "1.3M").
#[doc(alias = "precisionPrefix")]
pub fn precision_prefix(step: f64, value: f64) -> Option<usize> {
    let value = decimal::exponent(value)?;
    let step = decimal::exponent(step)?;
    Some(decimal::count(decimal::si_exponent(value) - step))
}

/// d3.precisionRound: significant digits for values `step` apart up to
/// `max` (`precision_round(0.01, 1.01)` is 3).
#[doc(alias = "precisionRound")]
pub fn precision_round(step: f64, max: f64) -> Option<usize> {
    let step = step.abs();
    let max = decimal::exponent(max.abs() - step)?;
    let step = decimal::exponent(step)?;
    Some(decimal::count(max - step) + 1)
}
