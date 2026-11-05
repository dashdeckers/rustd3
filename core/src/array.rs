//! d3-array's statistics over `f64` values: counts, extremes, sums, means,
//! variances and quantiles, which take any iterator of numbers (a slice
//! through `.iter().copied()`) and skip NaN as d3 skips null, undefined and
//! NaN, returning `None` where d3 returns undefined for want of a value;
//! d3's `range` and bisection over ascending slices; and [`Bin`], d3's
//! histogram layout.
//!
//! ```
//! use rustd3_core::array;
//!
//! let values = [3.0, f64::NAN, -1.0, 7.0];
//! assert_eq!(array::count(values), 3);
//! assert_eq!(array::extent(values), Some([-1.0, 7.0]));
//! assert_eq!(array::mean([1.0, 2.0, f64::NAN, 4.0]), Some(2.3333333333333335));
//! assert_eq!(array::min(Vec::new()), None);
//! assert_eq!(array::quantile([3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0], 0.25), Some(7.25));
//! assert_eq!(array::range(0.0, 1.0, 0.2), [0.0, 0.2, 0.4, 0.6000000000000001, 0.8]);
//! assert_eq!(array::bisect_right(&[1.0, 2.0, 2.0, 3.0], 2.0), 3);
//! ```

pub mod bin;

mod bisect;
mod count;
mod deviation;
mod extent;
mod histogram;
mod max;
mod mean;
mod median;
mod min;
mod quantile;
mod quickselect;
mod range;
mod sum;
mod variance;

pub use bisect::{bisect_left, bisect_right};
pub use count::count;
pub use deviation::deviation;
pub use extent::extent;
pub use histogram::{Bin, bin};
pub use max::max;
pub use mean::mean;
pub use median::median;
pub use min::min;
pub use quantile::{quantile, quantile_sorted};
use quickselect::quickselect;
pub use range::range;
pub use sum::sum;
pub use variance::variance;
