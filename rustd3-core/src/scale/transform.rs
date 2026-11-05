//! The built-in transforms: what a continuous scale applies to its domain
//! before interpolating, each with the tick rule, nice, minor ticks and
//! tick format that suit it. The trait they implement is
//! [`scale::Transform`](crate::scale::Transform).

mod any;
mod asinh;
mod func;
mod identity;
mod log;
mod logit;
mod pow;
mod symlog;

pub use any::Any;
pub use asinh::Asinh;
pub use func::Func;
pub use identity::Identity;
pub use log::Log;
pub use logit::Logit;
pub use pow::Pow;
pub use symlog::Symlog;
