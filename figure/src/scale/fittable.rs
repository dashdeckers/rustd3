//! [`Fittable`]: scales the figure can fit to its marks' data.

use rustd3_core::color::Rgb;
use rustd3_core::scale::{self, Band, Continuous, Ordinal, Point};
use rustd3_core::time::Time;

use crate::scale::Kind;

/// A scale whose domain the figure can fit to its marks' data: what
/// `Figure::fit` and `Figure::fit_with` take. Continuous and time scales
/// are fitted with the theme's margin; band, point and ordinal scales keep
/// the domain they were built with, since their keys are the domain.
pub trait Fittable: Kind {}

impl<T: scale::Transform + 'static, const N: usize> Fittable for Continuous<T, f64, N> {}

impl Fittable for Time {}

impl<K: PartialEq + Clone + 'static> Fittable for Band<K> {}

impl<K: PartialEq + Clone + 'static> Fittable for Point<K> {}

impl<K: PartialEq + Clone + 'static> Fittable for Ordinal<K, Rgb> {}
