//! Property tests over hostile inputs: every public entry point that takes
//! user data or parameters, fed NaN, ±infinity, −0.0, subnormals, 1e±300,
//! empty and one-element inputs, reversed and zero-width domains, counts up
//! to `usize::MAX` and random strings, from fixed seeds. Nothing panics,
//! and what comes back keeps the documented rules.

mod color;
mod format;
mod scale;
mod shape;

/// Why `ticks` break the tick rules over `domain`: finite, strictly
/// monotone in the domain's order (so sorted and without duplicates), and
/// inside the domain within a relative 1e-9. A domain with a NaN end has
/// no ticks.
pub(crate) fn tick_violation(ticks: &[f64], domain: [f64; 2]) -> Option<String> {
    if domain.iter().any(|end| end.is_nan()) {
        return (!ticks.is_empty()).then(|| format!("ticks {ticks:?} over a NaN domain"));
    }
    if let Some(v) = ticks.iter().find(|v| !v.is_finite()) {
        return Some(format!("a non-finite tick {v}"));
    }
    let descending = domain[1] < domain[0];
    if ticks.windows(2).any(|pair| {
        if descending {
            pair[1] >= pair[0]
        } else {
            pair[1] <= pair[0]
        }
    }) {
        return Some(format!(
            "ticks out of order or repeated: {:?}",
            &ticks[..ticks.len().min(12)]
        ));
    }
    let [low, high] = if descending {
        [domain[1], domain[0]]
    } else {
        domain
    };
    let (low, high) = (low - 1e-9 * low.abs(), high + 1e-9 * high.abs());
    ticks
        .iter()
        .find(|v| **v < low || **v > high)
        .map(|v| format!("tick {v} outside the domain"))
}
