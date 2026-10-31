//! Supertraits that close the figure's scale protocol: public traits in a
//! private module, so no other crate can name them. `Scale` closes every
//! capability trait built on it (`Kind`, `Position`, `Invert`, `Paint`,
//! `Fittable`, `mark::Binned`); core's sealed `Ticks` closes
//! `colorbar::Ramp`.

/// Closes [`scale::Scale`](crate::scale::Scale).
pub trait Scale {}

/// Closes [`scale::Indexed`](crate::scale::Indexed).
pub trait Indexed<D> {}
