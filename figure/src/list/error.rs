//! [`Error`]: why a list cannot be drawn.

/// Why a list cannot be drawn: each variant names the command (or part)
/// and the rejected value.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Groups still open at the end of the list.
    #[error("{open} group(s) still open at the end of the list")]
    Unbalanced {
        /// How many.
        open: usize,
    },
    /// A `Pop` with no group open.
    #[error("command {command}: Pop without an open group")]
    UnmatchedPop {
        /// The `Pop`.
        command: usize,
    },
    /// A glyph run's font outside the list's font table.
    #[error("command {command}: font {font} of a table of {fonts}")]
    Font {
        /// The run.
        command: usize,
        /// The font index.
        font: usize,
        /// The table's length.
        fonts: usize,
    },
    /// A markers channel (`sizes`, `paints.index`) whose length is not the
    /// point count.
    #[error("command {command}: {found} values for {expected} points")]
    ChannelLength {
        /// The markers command.
        command: usize,
        /// The point count.
        expected: usize,
        /// The channel's length.
        found: usize,
    },
    /// A per-point paint index outside the paint table.
    #[error("command {command}: point {point} has paint {index} of a table of {table}")]
    PaintIndex {
        /// The markers command.
        command: usize,
        /// The point.
        point: usize,
        /// Its index.
        index: u32,
        /// The table's length.
        table: usize,
    },
    /// Image data whose length is not width × height × 4.
    #[error(
        "command {command}: {found} bytes for a {width} × {height} px image, expected {expected}"
    )]
    ImageLength {
        /// The command holding the image.
        command: usize,
        /// The image's width.
        width: u32,
        /// The image's height.
        height: u32,
        /// width × height × 4, saturating.
        expected: u64,
        /// The data's length.
        found: usize,
    },
    /// A coordinate, size, transform or paint parameter that is NaN or
    /// infinite.
    #[error("command {command}: non-finite value {value}")]
    NonFinite {
        /// The command.
        command: usize,
        /// The first such value.
        value: f64,
    },
    /// A part whose range is reversed or reaches past the commands.
    #[error("part {part}: commands {start}..{end} outside 0..{len}")]
    PartRange {
        /// The part's index.
        part: usize,
        /// Its first command.
        start: usize,
        /// One past its last command.
        end: usize,
        /// The command count.
        len: usize,
    },
    /// A part that starts before the previous one ends.
    #[error("part {part}: starts at command {start}, before the previous part ends at {previous}")]
    PartOrder {
        /// The part's index.
        part: usize,
        /// Its first command.
        start: usize,
        /// Where the previous part ends.
        previous: usize,
    },
    /// A part whose Push and Pop are not balanced inside it.
    #[error("part {part}: groups unbalanced at command {command}")]
    PartUnbalanced {
        /// The part's index.
        part: usize,
        /// The `Pop` without a `Push` in the part, or the part's end when
        /// a group stays open.
        command: usize,
    },
    /// A command a backend cannot draw; lowering removes most of these.
    #[error("command {command}: {what} is not supported here")]
    Unsupported {
        /// The command.
        command: usize,
        /// What it holds.
        what: &'static str,
    },
}
