//! [`Probe`], the channel a `Plot` reports its samples on.

use std::sync::mpsc;

use iced_core::time::Instant;

use crate::plot::probe::Sample;

/// Where a `Plot` reports its frames: stage timings, raster times and
/// what each frame showed, timed by an app-supplied clock and sent to a
/// sink as [`Sample`]s.
#[derive(Debug, Clone)]
pub struct Probe {
    clock: fn() -> Instant,
    sink: mpsc::Sender<Sample>,
}

impl Probe {
    /// A probe whose samples are timed by `clock` and sent to `sink`. A
    /// closed receiver drops them.
    pub fn new(clock: fn() -> Instant, sink: mpsc::Sender<Sample>) -> Self {
        Probe { clock, sink }
    }

    /// The probe's clock.
    pub(crate) fn clock(&self) -> fn() -> Instant {
        self.clock
    }

    /// Reads the probe's clock.
    pub(crate) fn now(&self) -> Instant {
        (self.clock)()
    }

    /// Sends `sample`, unless the receiver is gone.
    pub(crate) fn send(&self, sample: Sample) {
        let _ = self.sink.send(sample);
    }
}
