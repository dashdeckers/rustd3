//! Supertraits that close the scale protocol's traits: public traits in a
//! private module, so no other crate can name them or implement the
//! traits that require them. The core implements each for exactly the
//! types that implement its public counterpart.

/// Closes [`series::Data`](crate::series::Data).
pub trait Data<'a, X, Y> {}

/// Closes [`series::Coordinate`](crate::series::Coordinate).
pub trait Coordinate {}

/// Closes [`scale::Ticks`](crate::scale::Ticks).
pub trait Ticks {}

/// Closes [`scale::Pad`](crate::scale::Pad).
pub trait Pad {}

/// Closes [`interact::Rescale`](crate::interact::Rescale).
pub trait Rescale {}
