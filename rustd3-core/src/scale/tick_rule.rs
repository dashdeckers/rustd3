//! [`TickRule`]: where a continuous scale's ticks go when its transform's
//! own rule is not the one wanted.

use std::fmt;
use std::sync::Arc;

use crate::scale::Transform;
use crate::ticks;

/// Where ticks go, independent of the transform's own rule: the rule
/// [`func`](crate::scale::func) takes, and what `.tick_rule(…)` puts on any
/// continuous scale.
///
/// ```
/// use rustd3_core::scale::{self, TickRule};
///
/// let s = scale::sqrt().domain([0.0, 100.0]).tick_rule(TickRule::Transformed);
/// assert_eq!(s.ticks(5), [0.0, 4.0, 16.0, 36.0, 64.0, 100.0]);
/// ```
#[derive(Clone)]
pub enum TickRule {
    /// d3's linear ticks in transformed space, mapped back.
    Transformed,
    /// d3's linear ticks in data space (d3's meaning for every scale).
    Data,
    /// The caller's rule: end stops (domain order) and count to tick values.
    Custom(Arc<dyn Fn([f64; 2], usize) -> Vec<f64> + Send + Sync>),
}

impl fmt::Debug for TickRule {
    /// `Custom` prints as `Custom(..)`: a function has no Debug.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            TickRule::Transformed => "Transformed",
            TickRule::Data => "Data",
            TickRule::Custom(_) => "Custom(..)",
        })
    }
}

impl TickRule {
    /// The caller's rule, from the end stops in domain order and the count
    /// to the tick values, returned as the rule writes them.
    pub fn custom(rule: impl Fn([f64; 2], usize) -> Vec<f64> + Send + Sync + 'static) -> Self {
        TickRule::Custom(Arc::new(rule))
    }

    /// About `count` ticks over `domain` by this rule, for a scale under
    /// `transform`.
    pub fn ticks<T: Transform>(&self, transform: &T, domain: [f64; 2], count: usize) -> Vec<f64> {
        match self {
            TickRule::Transformed => ticks::transformed(transform, domain, count),
            TickRule::Data => ticks::linear(domain, count),
            TickRule::Custom(rule) => rule(domain, count),
        }
    }
}
