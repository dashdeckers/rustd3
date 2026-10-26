//! Test support shared by every rustd3 crate: the `Obs` values that d3's
//! translated assertions and our results are compared as, within
//! tolerances; pixel statistics for raster comparisons; exact text
//! snapshots; and time zones from jiff's bundled database. It depends on
//! no workspace crate, so every crate can use it as a dev-dependency.
