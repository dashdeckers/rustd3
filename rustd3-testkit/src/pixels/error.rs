//! Why an image cannot be built or two images compared.

/// Why an image could not be built or two images compared.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The buffer does not hold `width × height × 4` bytes.
    #[error("{width}×{height} needs {expected} bytes, got {actual}")]
    Length {
        /// Width in pixels.
        width: u32,
        /// Height in pixels.
        height: u32,
        /// The byte count the size needs.
        expected: usize,
        /// The byte count given.
        actual: usize,
    },
    /// The images' sizes differ.
    #[error("sizes differ: {a:?} against {b:?}")]
    Size {
        /// The first image's width and height.
        a: (u32, u32),
        /// The second image's width and height.
        b: (u32, u32),
    },
}
