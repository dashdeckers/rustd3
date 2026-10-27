//! [`Any`]: a built-in transform chosen at run time.

use crate::format::{InvalidFormat, Locale};
use crate::scale::TickFormat;
use crate::scale::Transform;
use crate::scale::transform::{Asinh, Func, Identity, Log, Logit, Pow, Symlog};

/// A built-in transform chosen at run time: the transform of
/// [`AnyContinuous`](crate::scale::AnyContinuous), which forwards every
/// method to the transform it holds.
#[derive(Clone, Debug)]
pub enum Any {
    /// The identity, under linear scales.
    Identity(Identity),
    /// A power, under pow and sqrt scales.
    Pow(Pow),
    /// A logarithm, under log scales.
    Log(Log),
    /// The symmetric log, under symlog scales.
    Symlog(Symlog),
    /// The logit, under logit scales.
    Logit(Logit),
    /// The inverse hyperbolic sine, under asinh scales.
    Asinh(Asinh),
    /// A pair of closures, under func scales.
    Func(Func),
}

/// `impl Transform for Any`, each method a match over the built-in
/// transforms that forwards to the one held.
macro_rules! forward_each {
    ($($variant:ident),*) => {
        impl Transform for Any {
            fn forward(&self, x: f64) -> f64 {
                match self { $(Any::$variant(t) => t.forward(x),)* }
            }

            fn inverse(&self, y: f64) -> f64 {
                match self { $(Any::$variant(t) => t.inverse(y),)* }
            }

            fn ticks(&self, domain: [f64; 2], count: usize) -> Vec<f64> {
                match self { $(Any::$variant(t) => t.ticks(domain, count),)* }
            }

            fn minor(&self, major: &[f64]) -> Vec<f64> {
                match self { $(Any::$variant(t) => t.minor(major),)* }
            }

            fn for_domain(&self, domain: [f64; 2]) -> Any {
                match self { $(Any::$variant(t) => Any::$variant(t.for_domain(domain)),)* }
            }

            fn nice(&self, domain: [f64; 2], count: usize) -> [f64; 2] {
                match self { $(Any::$variant(t) => t.nice(domain, count),)* }
            }

            fn tick_format(
                &self,
                locale: &Locale,
                domain: [f64; 2],
                count: usize,
                specifier: Option<&str>,
            ) -> Result<TickFormat, InvalidFormat> {
                match self {
                    $(Any::$variant(t) => t.tick_format(locale, domain, count, specifier),)*
                }
            }
        }
    };
}

builtin_transforms!(forward_each);
