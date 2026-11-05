//! Supertraits that seal the figure's scale protocol. They are public
//! traits in a private module, so no other crate can name them. `Scale`
//! seals every capability trait built on it (`Kind`, `Position`, `Invert`,
//! `Paint`, `Fittable`, `mark::Binned`), and core's sealed `Ticks` seals
//! `colorbar::Ramp`.

/// Seals [`scale::Scale`](crate::scale::Scale).
pub trait Scale {}

/// Seals [`scale::Indexed`](crate::scale::Indexed).
pub trait Indexed<D> {}
