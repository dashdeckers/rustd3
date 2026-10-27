//! [`Position`]: scales that place values along x or y.

use crate::scale::{Indexed, Kind};

/// A scale that places domain values along x or y, in px.
pub trait Position: Kind<Output = f64> {
    /// Where `v` lies along the range; `None` for a key outside the
    /// domain. For band scales it is the band's start.
    fn position(&self, v: &Self::Domain) -> Option<f64>;

    /// The width of each band; 0 for scales without bands.
    fn bandwidth(&self) -> f64 {
        0.0
    }

    /// The bridge to core's indexed recordings: `Some` for scales whose
    /// domain is a `series::Coordinate`, so generic marks can fit and thin
    /// `Series` data on them; `None` for band, point and ordinal scales.
    fn indexed(&self) -> Option<&dyn Indexed<Self::Domain>> {
        None
    }
}
