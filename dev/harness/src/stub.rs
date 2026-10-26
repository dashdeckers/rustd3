//! A stand-in for rustd3-core's port of d3-ease.

/// d3-ease's `easeLinear`. It stands in for the port so the harness can
/// prove translation, grading and the ratchet before core has code; Phase 3
/// deletes it and grades `Ease::Linear` instead.
pub fn ease_linear(t: f64) -> f64 {
    t
}
