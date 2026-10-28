//! [`Channel`]: one value per point, drawn through a handle (a dots
//! mark's `fill_by` and `size_by`).

use rustd3_core::color::Rgb;

use crate::figure::Fits;
use crate::mark::{Ctx, Error};
use crate::scale::{Paint, Position, ScaleRef};

/// One value per point of a mark, resolved through a handle when the
/// figure draws: into a color (`fill_by`) or a radius in px (`size_by`).
pub(crate) struct Channel<T> {
    values: Vec<f64>,
    through: Box<dyn Through<T>>,
}

/// A handle with its scale's type erased, which resolves values into `T`.
trait Through<T> {
    /// Reports `values` to the handle's fitting.
    fn fit(&self, values: &[f64], fits: &mut Fits);

    /// Each of `values` through the resolved scale: `None` where it gives
    /// nothing (a color it lacks, a size that is not finite).
    fn resolve(&self, values: &[f64], ctx: &Ctx<'_>) -> Vec<Option<T>>;
}

/// A color handle.
struct Fill<C: Paint<Domain = f64>>(ScaleRef<C>);

/// A numeric handle whose output is a radius in px.
struct Size<S: Position<Domain = f64>>(ScaleRef<S>);

impl<C: Paint<Domain = f64>> Through<Rgb> for Fill<C> {
    fn fit(&self, values: &[f64], fits: &mut Fits) {
        for v in values {
            fits.add(self.0, v);
        }
    }

    fn resolve(&self, values: &[f64], ctx: &Ctx<'_>) -> Vec<Option<Rgb>> {
        let scale = ctx.scale(self.0);
        values.iter().map(|v| scale.paint(v)).collect()
    }
}

impl<S: Position<Domain = f64>> Through<f64> for Size<S> {
    fn fit(&self, values: &[f64], fits: &mut Fits) {
        for v in values {
            fits.add(self.0, v);
        }
    }

    fn resolve(&self, values: &[f64], ctx: &Ctx<'_>) -> Vec<Option<f64>> {
        let scale = ctx.scale(self.0);
        values
            .iter()
            .map(|v| scale.position(v).filter(|r| r.is_finite()))
            .collect()
    }
}

impl Channel<Rgb> {
    /// `values` colored through `palette`.
    pub(crate) fn fill<C: Paint<Domain = f64>>(palette: ScaleRef<C>, values: Vec<f64>) -> Self {
        Channel {
            values,
            through: Box::new(Fill(palette)),
        }
    }
}

impl Channel<f64> {
    /// `values` sized through `radius`, whose output is a radius in px.
    pub(crate) fn size<S: Position<Domain = f64>>(radius: ScaleRef<S>, values: Vec<f64>) -> Self {
        Channel {
            values,
            through: Box::new(Size(radius)),
        }
    }
}

impl<T> Channel<T> {
    /// The channel for a mark of `points` points: [`Error::ChannelLength`]
    /// when it has another number of values, which is never truncated.
    pub(crate) fn of(self, points: usize) -> Result<Self, Error> {
        if self.values.len() == points {
            Ok(self)
        } else {
            Err(Error::ChannelLength {
                points,
                values: self.values.len(),
            })
        }
    }

    /// Reports the values to the handle's fitting.
    pub(crate) fn fit(&self, fits: &mut Fits) {
        self.through.fit(&self.values, fits);
    }

    /// Each point's value through the resolved handle, in point order:
    /// `None` where the handle gives nothing.
    pub(crate) fn resolve(&self, ctx: &Ctx<'_>) -> Vec<Option<T>> {
        self.through.resolve(&self.values, ctx)
    }
}

#[cfg(test)]
mod tests {
    use rustd3_core::scale::{self, Linear};

    use super::*;

    /// A handle of figure 0 at index 0, for channels that are never drawn.
    fn handle() -> ScaleRef<Linear> {
        let mut fig = crate::Figure::new();
        fig.scale(scale::linear())
    }

    #[test]
    fn a_channel_of_another_length_than_the_points_is_an_error() {
        let size = Channel::size(handle(), vec![1.0, 2.0]);
        assert!(size.of(2).is_ok());
        let size = Channel::size(handle(), vec![1.0, 2.0]);
        assert!(matches!(
            size.of(3),
            Err(Error::ChannelLength {
                points: 3,
                values: 2
            })
        ));
    }
}
