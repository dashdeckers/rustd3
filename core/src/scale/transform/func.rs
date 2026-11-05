//! A transform from two closures and the tick rule the caller chose.

use std::fmt;
use std::sync::Arc;

use crate::format::{InvalidFormat, Locale};
use crate::scale::{self, TickFormat, TickRule, Transform};

/// One direction of a [`Func`]: a closure shared between clones.
type Map = Arc<dyn Fn(f64) -> f64 + Send + Sync>;

/// A transform from a forward and an inverse closure, ticked by the
/// [`TickRule`] it was built with and labeled to the places of its ticks'
/// smallest gap. See [`scale::func`](crate::scale::func).
#[derive(Clone)]
pub struct Func {
    forward: Map,
    inverse: Map,
    rule: TickRule,
}

impl fmt::Debug for Func {
    /// The rule; the closures have no Debug.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Func")
            .field("rule", &self.rule)
            .finish_non_exhaustive()
    }
}

impl Func {
    /// The transform `forward`, `inverse`, ticked by `rule`.
    pub(crate) fn new(
        forward: impl Fn(f64) -> f64 + Send + Sync + 'static,
        inverse: impl Fn(f64) -> f64 + Send + Sync + 'static,
        rule: TickRule,
    ) -> Func {
        Func {
            forward: Arc::new(forward),
            inverse: Arc::new(inverse),
            rule,
        }
    }
}

impl Transform for Func {
    fn forward(&self, x: f64) -> f64 {
        (self.forward)(x)
    }

    fn inverse(&self, y: f64) -> f64 {
        (self.inverse)(y)
    }

    fn ticks(&self, domain: [f64; 2], count: usize) -> Vec<f64> {
        self.rule.ticks(self, domain, count)
    }

    /// With no specifier, `,f` to the places of the smallest gap between
    /// this transform's ticks, labeling every tick, as for a scale whose
    /// tick rule replaces its transform's. A given specifier, or fewer than
    /// two ticks, takes d3's linear tick format. Under [`TickRule::Data`]
    /// the two agree.
    fn tick_format(
        &self,
        locale: &Locale,
        domain: [f64; 2],
        count: usize,
        specifier: Option<&str>,
    ) -> Result<TickFormat, InvalidFormat> {
        if specifier.is_none()
            && let Some(format) = TickFormat::spaced(locale, &self.ticks(domain, count))
        {
            return Ok(format);
        }
        scale::tick_format(locale, domain, count, specifier).map(TickFormat::from)
    }
}
