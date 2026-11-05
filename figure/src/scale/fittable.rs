//! [`Fittable`]: scales the figure can fit to its marks' data.

use rustd3_core::color::Rgb;
use rustd3_core::scale::{
    self,
    Band,
    Continuous,
    Diverging,
    Ordinal,
    Point,
    Quantize,
    Sequential,
    Threshold,
};
use rustd3_core::time::Time;

use crate::scale::Kind;

/// A scale whose domain the figure can fit to its marks' data: what
/// `Figure::fit` and `Figure::fit_with` take. Every kind is one except
/// `Quantile`, whose domain is the data sample itself. Continuous and time
/// scales are fitted with the theme's margin, or rounded to nice values
/// under `Domains::Nice` ([`Kind::niced`]); color scales and quantize
/// to the data's extent with none; band, point, ordinal and threshold
/// scales keep the domain they were built with, since their keys or
/// thresholds are the domain.
pub trait Fittable: Kind {}

impl<T: scale::Transform + 'static, const N: usize> Fittable for Continuous<T, f64, N> {}

impl Fittable for Time {}

impl<K: PartialEq + Clone + 'static> Fittable for Band<K> {}

impl<K: PartialEq + Clone + 'static> Fittable for Point<K> {}

impl<K: PartialEq + Clone + 'static> Fittable for Ordinal<K, Rgb> {}

impl<T: scale::Transform + 'static> Fittable for Sequential<Rgb, T> {}

impl<T: scale::Transform + 'static> Fittable for Diverging<Rgb, T> {}

impl<T: scale::Transform + 'static, const N: usize> Fittable for Continuous<T, Rgb, N> {}

impl Fittable for Quantize<Rgb> {}

impl Fittable for Threshold<Rgb> {}
