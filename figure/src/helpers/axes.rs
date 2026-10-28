//! Axis defaults: [`defaults`] turns an axis's minor ticks on; its offset
//! and multiplier notation ("+1e9", "1e−7" at the axis end) is already
//! every figure axis's default and stays on.

use crate::axis::Axis;
use crate::scale::Kind;

/// `axis` with minor ticks on ([`Axis::minor`]): core's
/// [`Ticks::minor`](rustd3_core::scale::Ticks::minor) values at the
/// theme's minor tick size and width, without labels (decade multiples on
/// log and symlog axes, fifths of a step on linear ones, none on band,
/// point and time axes). Offset and multiplier notation is left as the
/// axis has it: on, unless `.offset(false)` turned it off.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::{Figure, axis};
/// use rustd3_figure::helpers::axes;
///
/// let mut fig = Figure::new();
/// let y = fig.scale(scale::symlog().constant(1.0).domain([0.0, 5000.0]));
/// fig.panel(0, 0).axis(axes::defaults(axis::left(y)));
/// ```
pub fn defaults<'a, S: Kind>(axis: Axis<'a, S>) -> Axis<'a, S> {
    axis.minor(true)
}
