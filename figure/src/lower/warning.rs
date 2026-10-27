//! [`Warning`]: what lowering could not draw faithfully.

/// What [`lower`](crate::lower::lower) could not draw faithfully, naming
/// the input command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Warning {
    /// A sweep gradient drawn as an embedded image (`raster` on).
    SweepRasterized {
        /// The input command.
        command: usize,
    },
    /// Without `raster`: the sweep drawn in its middle stop's color.
    SweepFlattened {
        /// The input command.
        command: usize,
    },
    /// A path clip on a backend without `clip_paths`: left in place and
    /// drawn unclipped by that backend.
    UnsupportedClip {
        /// The input command.
        command: usize,
    },
}
