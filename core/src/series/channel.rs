//! One channel of a recording: its samples in chunks.

use crate::series::chunks::Chunks;

/// One channel's samples; NaN is a gap.
#[derive(Debug, Clone, Default)]
pub(crate) struct Channel {
    values: Chunks<f64>,
}

impl Channel {
    /// Appends `values`.
    pub(crate) fn extend(&mut self, values: &[f64]) {
        self.values.extend(values);
    }

    /// Removes the first `chunks` chunks.
    pub(crate) fn drop_front(&mut self, chunks: usize) {
        self.values.drop_front(chunks);
    }
}
