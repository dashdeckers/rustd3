//! d3-scale's continuous scales as plain values: built by consuming
//! builders, read by `apply`, `invert`, `ticks` and `tick_format`, and
//! generic over a public [`Transform`] and the range type; d3's identity
//! scale; its band, point and ordinal scales over an explicit domain of
//! keys; [`Ticks`], what an axis needs from any of them; and [`Pad`], a
//! domain fitted to data as matplotlib autoscales an axis.
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

/// Calls `$then!` with the names of the built-in transforms, the variants
/// of `transform::Any`: one list for its forwarding and for the
/// conversions into `AnyContinuous`.
macro_rules! builtin_transforms {
    ($then:ident) => {
        $then!(Identity, Pow, Log, Symlog, Logit, Asinh, Func);
    };
}

mod any;
mod band;
mod continuous;
mod identity;
mod mapping;
mod ordinal;
mod pad;
mod piecewise;
mod point;
mod sequential;
mod tick_format;
mod tick_rule;
mod ticks;
pub mod transform;

pub(crate) use continuous::polymap;

pub use any::AnyContinuous;
pub use band::{Band, band};
pub use continuous::Continuous;
pub use identity::{Identity, identity};
pub use mapping::Transform;
pub use ordinal::{Ordinal, ordinal};
pub use pad::Pad;
pub use piecewise::piecewise;
pub use point::{Point, point};
pub use sequential::{Sequential, sequential};
pub use tick_format::{TickFormat, tick_format};
pub use tick_rule::TickRule;
pub use ticks::Ticks;

/// d3's scaleLinear: a continuous scale under the identity transform.
pub type Linear<R = f64, const N: usize = 2> = Continuous<transform::Identity, R, N>;

/// d3's scalePow: a continuous scale under a power transform.
pub type Pow<R = f64, const N: usize = 2> = Continuous<transform::Pow, R, N>;

/// d3's scaleLog: a continuous scale under a log transform.
pub type Log<R = f64, const N: usize = 2> = Continuous<transform::Log, R, N>;

/// d3's scaleSymlog: a continuous scale under the symmetric log transform.
pub type Symlog<R = f64, const N: usize = 2> = Continuous<transform::Symlog, R, N>;

/// A continuous scale under the logit transform, for probabilities.
pub type Logit<R = f64, const N: usize = 2> = Continuous<transform::Logit, R, N>;

/// A continuous scale under the inverse hyperbolic sine transform.
pub type Asinh<R = f64, const N: usize = 2> = Continuous<transform::Asinh, R, N>;

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

/// d3.scalePow(): exponent 1, domain [0, 1], range [0, 1].
#[doc(alias = "scalePow")]
pub fn pow() -> Pow {
    continuous(transform::Pow::new(1.0))
}

/// d3.scaleSqrt(): `pow()` with exponent 0.5.
#[doc(alias = "scaleSqrt")]
pub fn sqrt() -> Pow {
    continuous(transform::Pow::new(0.5))
}

/// d3.scaleLog(): base 10, domain [1, 10], range [0, 1].
#[doc(alias = "scaleLog")]
pub fn log() -> Log {
    continuous(transform::Log::new(10.0)).domain([1.0, 10.0])
}

/// d3.scaleSymlog(): constant 1, domain [0, 1], range [0, 1]; ticks by
/// decades past the constant where d3 ticks linearly.
///
/// ```
/// use rustd3_core::scale;
///
/// let v = scale::symlog().constant(1.0).domain([0.0, 1e6]);
/// assert_eq!(v.ticks(10), [0.0, 1.0, 10.0, 100.0, 1e3, 1e4, 1e5, 1e6]);
/// ```
#[doc(alias = "scaleSymlog")]
pub fn symlog() -> Symlog {
    continuous(transform::Symlog::new(1.0))
}

/// A logit scale for probabilities: domain [0, 1] (which the transform
/// sends to ±∞, so set a domain inside (0, 1)) and range [0, 1]; ticks at
/// 0.5 and decades towards 0 and 1.
///
/// ```
/// use rustd3_core::scale;
///
/// let p = scale::logit().domain([0.001, 0.999]);
/// assert_eq!(p.ticks(10), [0.001, 0.01, 0.1, 0.5, 0.9, 0.99, 0.999]);
/// ```
pub fn logit() -> Logit {
    continuous(transform::Logit)
}

/// An inverse hyperbolic sine scale: linear width 1, domain [0, 1], range
/// [0, 1]; ticks at signed powers of ten, as matplotlib's asinh scale.
///
/// ```
/// use rustd3_core::scale;
///
/// let a = scale::asinh().domain([-1000.0, 1000.0]);
/// assert_eq!(a.ticks(10), [-100.0, -10.0, -1.0, 0.0, 1.0, 10.0, 100.0]);
/// ```
pub fn asinh() -> Asinh {
    continuous(transform::Asinh::new(1.0))
}

/// A scale under the closures `forward` and `inverse`, ticked by `rule`:
/// the choice of rule is part of the signature.
///
/// ```
/// use rustd3_core::scale::{self, TickRule};
///
/// let s = scale::func(f64::sqrt, |y| y * y, TickRule::Data).domain([0.0, 100.0]);
/// assert_eq!(s.ticks(5), [0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);
/// ```
pub fn func(
    forward: impl Fn(f64) -> f64 + Send + Sync + 'static,
    inverse: impl Fn(f64) -> f64 + Send + Sync + 'static,
    rule: TickRule,
) -> Continuous<transform::Func> {
    continuous(transform::Func::new(forward, inverse, rule))
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
        let step = crate::ticks::tick_increment(start, stop, count);
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

/// `keys` in order, each kept at its first position: d3's categorical
/// domains, which an `InternMap` dedupes.
fn unique<K: PartialEq>(keys: impl IntoIterator<Item = K>) -> Vec<K> {
    let mut out: Vec<K> = Vec::new();
    for key in keys {
        if !out.contains(&key) {
            out.push(key);
        }
    }
    out
}
