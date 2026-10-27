//! d3-array's statistics over `f64` values: counts, extremes, sums, means,
//! variances. Each takes any iterator of numbers (a slice through
//! `.iter().copied()`) and skips NaN as d3 skips null, undefined and NaN;
//! where d3 returns undefined for want of a value, these return `None`.
//!
//! ```
//! use rustd3_core::array;
//!
//! let values = [3.0, f64::NAN, -1.0, 7.0];
//! assert_eq!(array::count(values), 3);
//! assert_eq!(array::extent(values), Some([-1.0, 7.0]));
//! assert_eq!(array::mean([1.0, 2.0, f64::NAN, 4.0]), Some(2.3333333333333335));
//! assert_eq!(array::min(Vec::new()), None);
//! ```

mod count;
mod deviation;
mod extent;
mod max;
mod mean;
mod min;
mod sum;
mod variance;

pub use count::count;
pub use deviation::deviation;
pub use extent::extent;
pub use max::max;
pub use mean::mean;
pub use min::min;
pub use sum::sum;
pub use variance::variance;
