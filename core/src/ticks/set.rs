//! [`Set`]: the ticks a scale chose, with the step and count behind them.

/// Tick values with the step and count they were chosen for: what
/// [`scale::Ticks`](crate::scale::Ticks) returns and labels.
///
/// ```
/// use rustd3_core::scale::{self, Ticks};
///
/// let x = scale::linear().domain([0.0, 100.0]);
/// let set = x.tick_set(5);
/// assert_eq!(set.values(), [0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);
/// assert_eq!((*set.step(), set.count()), (20.0, 5));
///
/// let chosen = set.with_values(vec![0.0, 50.0, 100.0]);
/// assert_eq!((*chosen.step(), chosen.count()), (20.0, 5));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Set<D, S> {
    values: Vec<D>,
    step: S,
    count: usize,
}

impl<D, S> Set<D, S> {
    /// `values`, chosen with `step` for about `count` ticks.
    pub fn new(values: Vec<D>, step: S, count: usize) -> Self {
        Set {
            values,
            step,
            count,
        }
    }

    /// The tick values, in the order of the scale's domain.
    pub fn values(&self) -> &[D] {
        &self.values
    }

    /// The step the values were chosen with: d3's tickFormat step for
    /// numbers.
    pub fn step(&self) -> &S {
        &self.step
    }

    /// The requested count: d3's `tickArguments[0]`, used by log labels.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Replaces the values, keeping step and count: d3's `tickValues`,
    /// which still formats with `tickFormat(tickArguments)`.
    pub fn with_values(self, values: Vec<D>) -> Self {
        Set { values, ..self }
    }
}
