//! [`AnyContinuous`]: a continuous scale whose transform is chosen at run
//! time, and its conversions from each built-in scale.

use crate::scale::{Continuous, transform};

/// A continuous scale under a built-in transform chosen at run time, such
/// as a log toggle or scales stored by id. Every builder and reader of
/// [`Continuous`] works on it; transform-specific builders (`base`,
/// `exponent`, `constant`, `linear_width`) apply before converting.
///
/// ```
/// use rustd3_core::scale::{self, AnyContinuous};
///
/// let log_axis = true;
/// let y: AnyContinuous = if log_axis {
///     scale::log().domain([1.0, 1e3]).into()
/// } else {
///     scale::linear().domain([0.0, 1e3]).into()
/// };
/// assert_eq!(y.ticks(3), [1.0, 10.0, 100.0, 1000.0]);
/// ```
pub type AnyContinuous<R = f64, const N: usize = 2> = Continuous<transform::Any, R, N>;

/// `From<Continuous<transform::X, R, N>> for AnyContinuous<R, N>` for each
/// built-in transform X: a blanket impl would overlap the reflexive one at
/// `X = Any` (E0119).
macro_rules! from_each {
    ($($variant:ident),*) => {
        $(
            impl<R, const N: usize> From<Continuous<transform::$variant, R, N>> for AnyContinuous<R, N> {
                fn from(scale: Continuous<transform::$variant, R, N>) -> Self {
                    scale.map_transform(transform::Any::$variant)
                }
            }
        )*
    };
}

builtin_transforms!(from_each);
