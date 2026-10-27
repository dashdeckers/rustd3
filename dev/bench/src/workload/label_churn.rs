//! The `label-churn` workload: `trace` whose bottom axes take 16 ticks
//! labelled `t+{v:.4} s`, so pans and zooms bring label strings the text
//! cache has not seen. Phase 34 reruns it with real time formats.

/// How a workload labels its x axes.
#[derive(Debug, Clone, Copy)]
pub struct Labels {
    /// The tick label of a value.
    pub format: fn(&f64) -> String,
    /// The tick count asked for.
    pub ticks: usize,
}

/// The bottom axes' format and tick count.
pub const LABELS: Labels = Labels {
    format: churn,
    ticks: 16,
};

/// A tick label: `t+12.3400 s`.
fn churn(v: &f64) -> String {
    format!("t+{v:.4} s")
}
