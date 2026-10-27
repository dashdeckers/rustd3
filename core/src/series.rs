//! Long data: recordings of `f64` channels on one clock, held in chunks
//! that appends never move and that `drop_front` removes whole; `Copy`
//! views of one channel; and M4 thinning per column, answered by an exact
//! index that grows with every append.
//!
//! ```
//! use rustd3_core::series::{Clock, Error, Recording};
//!
//! let mut rec = Recording::new(Clock::Explicit(vec![0.0, 1.0, 2.0]), [[5.0, f64::NAN, 4.0]])?;
//! assert_eq!(rec.locate(0.5), 1);
//! assert_eq!(
//!     rec.extend(&[0.5], [[1.0]]),
//!     Err(Error::Unordered { index: 3, previous: 2.0, next: 0.5 }),
//! );
//! assert_eq!(rec.len(), 3);
//! # Ok::<(), Error>(())
//! ```

mod channel;
mod chunks;
mod clock;
mod columns;
mod data;
mod error;
mod index;
pub mod m4;
mod points;
mod recording;
mod runs;
mod slices;
mod view;

pub use clock::{Clock, Coordinate};
pub use columns::Columns;
pub use data::Data;
pub use error::Error;
pub use points::Points;
pub use recording::{Id, Recording};
pub use slices::{Slices, slices};
pub use view::Series;

/// Samples per chunk: storage grows and drops by whole chunks.
const CHUNK: usize = 65_536;

/// Samples per leaf of the M4 index; a chunk holds whole blocks.
const BLOCK: usize = 64;
