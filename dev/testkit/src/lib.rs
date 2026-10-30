//! Test support shared by every rustd3 crate: the `Obs` values that d3's
//! translated assertions and our results are compared as, within
//! tolerances; fixtures' JSON read by pointer into them; pixel statistics
//! for raster comparisons; exact text snapshots; time zones from jiff's
//! bundled database; and hostile inputs for property tests. It depends
//! on no workspace crate, so every crate can use it as a dev-dependency.

pub mod fixture;
pub mod hostile;
pub mod obs;
pub mod pixels;
pub mod snapshot;
mod zone;

pub use zone::zone;
