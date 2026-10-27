//! Long data: recordings of `f64` channels on one clock, held in chunks
//! that appends never move and that `drop_front` removes whole.
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
mod error;
mod recording;

pub use clock::{Clock, Coordinate};
pub use error::Error;
pub use recording::{Id, Recording};

/// Samples per chunk: storage grows and drops by whole chunks.
const CHUNK: usize = 65_536;
