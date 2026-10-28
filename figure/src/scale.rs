//! What the figure needs from a scale: an object-safe [`Scale`] for its
//! table, which delegates gestures to core's `Rescale`; [`ScaleRef`], the
//! typed handle marks and axes hold; and typed capability traits for them ([`Kind`], [`Position`],
//! [`Paint`], [`Fittable`], [`Invert`], [`Indexed`]), implemented for
//! core's continuous scales with an `f64` range, its time scales, and its
//! band, point and ordinal color scales.
//!
//! ```
//! use rustd3_core::scale;
//! use rustd3_figure::scale::{Kind, Position, Scale};
//!
//! let x = scale::linear().domain([0.0, 10.0]);
//! let fitted = x.fitted(Some(&[0.0, 10.0]), &[], 0.05);
//! assert_eq!(Kind::domain(&fitted), Some([-0.5, 10.5]));
//! let panned = x.pan_px([0.0, 100.0], 10.0).expect("a linear scale pans");
//! let panned = panned.as_any().downcast_ref::<scale::Linear>().expect("its own type");
//! assert_eq!(panned.position(&0.0), Some(10.0));
//! ```

mod band;
mod continuous;
mod fit;
mod fittable;
mod handle;
mod indexed;
mod invert;
mod kind;
mod ordinal;
mod paint;
mod point;
mod position;
mod time;
pub(crate) mod visible;
pub(crate) mod window;

use std::any::Any;

pub use fit::Fit;
pub use fittable::Fittable;
pub use handle::ScaleRef;
pub use indexed::{Indexed, SeriesExtent};
pub use invert::Invert;
pub use kind::Kind;
pub use paint::Paint;
pub use position::Position;

/// The object-safe face of a scale, so one table holds every kind; its
/// pixel gestures delegate to core's `Rescale` on a copy set to a panel's
/// range, so each scale keeps its own precision.
pub trait Scale: Any {
    /// The scale as `Any`, to recover its type.
    fn as_any(&self) -> &dyn Any;

    /// A boxed copy.
    fn boxed(&self) -> Box<dyn Scale>;

    /// A copy whose range runs from `range[0]` to `range[1]`, keeping
    /// everything else (interpolator, clamp, unknown value, rounding). A
    /// scale whose range is not positional (a color scale) is copied as it
    /// is.
    fn with_range(&self, range: [f64; 2]) -> Box<dyn Scale>;

    /// Core's `Rescale::pan_px(dx)` on a copy set to `range`; `None` for
    /// scales without it (band, point, ordinal) and for a step core
    /// rejects.
    fn pan_px(&self, range: [f64; 2], dx: f64) -> Option<Box<dyn Scale>>;

    /// Core's `Rescale::zoom_px(anchor, factor)` on a copy set to `range`,
    /// around `anchor` px; `None` for scales without it and for a step
    /// core rejects.
    fn zoom_px(&self, range: [f64; 2], anchor: f64, factor: f64) -> Option<Box<dyn Scale>>;
}

/// `ends` spread over `N` stops: the first and last are `ends` exactly, and
/// each inner stop keeps its share of the way between `old`'s first and
/// last (equal shares when those coincide or are not finite).
fn stops<const N: usize>(old: &[f64; N], [start, end]: [f64; 2]) -> [f64; N] {
    let span = old[N - 1] - old[0];
    std::array::from_fn(|i| {
        if i == 0 {
            start
        } else if i == N - 1 {
            end
        } else {
            let share = if span != 0.0 && span.is_finite() {
                (old[i] - old[0]) / span
            } else {
                i as f64 / (N - 1) as f64
            };
            start + share * (end - start)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_stops_are_the_ends_exactly() {
        assert_eq!(stops(&[3.0, 7.0], [0.1, 0.7]), [0.1, 0.7]);
    }

    #[test]
    fn inner_stops_keep_their_share_of_the_span() {
        assert_eq!(
            stops(&[0.0, 25.0, 100.0], [0.0, 400.0]),
            [0.0, 100.0, 400.0]
        );
        assert_eq!(
            stops(&[100.0, 75.0, 0.0], [0.0, 400.0]),
            [0.0, 100.0, 400.0]
        );
    }

    #[test]
    fn inner_stops_of_a_flat_extent_are_spread_evenly() {
        assert_eq!(stops(&[1.0, 1.0, 1.0], [0.0, 10.0]), [0.0, 5.0, 10.0]);
    }
}
