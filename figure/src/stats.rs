//! Statistics the figure's marks compute from their data: [`Kde`], the
//! Gaussian kernel density estimate behind
//! [`density_y`](crate::mark::density_y) and
//! [`density_x`](crate::mark::density_x).

mod kde;

pub use kde::Kde;
