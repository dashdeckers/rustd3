//! [`Event`]: what a drag reports.

/// What [`Drag`](crate::interact::Drag) reports: d3-drag's `start`,
/// `drag` and `end` events. Positions are the subject's: the pointer's
/// plus the subject's offset from the press.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// A press picked a subject.
    Start {
        /// The subject's horizontal position.
        x: f64,
        /// The subject's vertical position.
        y: f64,
    },
    /// The pointer moved.
    Drag {
        /// The subject's horizontal position.
        x: f64,
        /// The subject's vertical position.
        y: f64,
        /// The pointer's horizontal movement since the previous event.
        dx: f64,
        /// The pointer's vertical movement since the previous event.
        dy: f64,
    },
    /// The left button was released. The position is the last move's,
    /// since a release carries none.
    End {
        /// The subject's horizontal position.
        x: f64,
        /// The subject's vertical position.
        y: f64,
        /// Whether the pointer traveled beyond the click distance, in
        /// which case d3 suppresses the click.
        moved: bool,
    },
}
