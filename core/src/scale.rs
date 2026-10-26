//! d3-scale's continuous scales as plain values: built by consuming
//! builders, read by `apply`, `invert`, `ticks` and `tick_format`, and
//! generic over a public [`Transform`] and the range type.
//!
//! ```
//! use rustd3_core::scale;
//!
//! let x = scale::linear().domain([0.0, 100.0]).range([0.0, 960.0]);
//! assert_eq!(x.apply(25.0), 240.0);
//! assert_eq!(x.invert(480.0), 50.0);
//!
//! let y = scale::linear().domain([0.13, 9.71]).range([300.0, 0.0]).nice(5);
//! let f = y.tick_format(5, None)?;
//! assert_eq!(y.get_domain(), [0.0, 10.0]);
//! assert_eq!(f.format_all(&y.ticks(5)), ["0", "2", "4", "6", "8", "10"]);
//! # Ok::<(), rustd3_core::format::InvalidFormat>(())
//! ```

mod continuous;
mod tick_format;
pub mod transform;

pub use continuous::Continuous;
pub use tick_format::{TickFormat, tick_format};
pub use transform::Transform;

use crate::ticks;

/// d3's scaleLinear: a continuous scale under the identity transform.
pub type Linear<R = f64, const N: usize = 2> = Continuous<transform::Identity, R, N>;

/// A continuous scale under `transform` with domain [0, 1] and range
/// [0, 1], d3's `transformer()` made public.
pub fn continuous<T: Transform>(transform: T) -> Continuous<T> {
    Continuous::new(transform)
}

/// d3.scaleLinear(): domain [0, 1], range [0, 1].
#[doc(alias = "scaleLinear")]
pub fn linear() -> Linear {
    continuous(transform::Identity)
}

/// d3-scale's linear nice (linear.js): `domain` extended to multiples of
/// the tick increment for `count` ticks, written back only once the
/// increment stops changing within ten tries (d3-array's `nice` writes
/// back every try).
pub(crate) fn nice_linear(domain: [f64; 2], count: usize) -> [f64; 2] {
    let reverse = domain[1] < domain[0];
    let [mut start, mut stop] = if reverse {
        [domain[1], domain[0]]
    } else {
        domain
    };
    let count = count as f64;
    let mut previous = None;
    for _ in 0..10 {
        let step = ticks::tick_increment(start, stop, count);
        if previous == Some(step) {
            return if reverse {
                [stop, start]
            } else {
                [start, stop]
            };
        } else if step > 0.0 {
            start = (start / step).floor() * step;
            stop = (stop / step).ceil() * step;
        } else if step < 0.0 {
            start = (start * step).ceil() / step;
            stop = (stop * step).floor() / step;
        } else {
            break;
        }
        previous = Some(step);
    }
    domain
}
