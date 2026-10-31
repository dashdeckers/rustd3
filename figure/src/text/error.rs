//! [`Error`]: what typst text can fail with.

/// A label typst cannot shape into one box, or font bytes typst cannot
/// read. A label's error is cached like its shape, so it is not retried
/// every frame.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// typst rejects the label's markup.
    #[error("typst rejects {markup:?}: {message}")]
    Markup {
        /// The label's snippet, as its key holds it.
        markup: String,
        /// typst's error messages, joined by "; ".
        message: String,
    },
    /// The label lays out as other than one page (its markup holds a page
    /// break, for one).
    #[error("{markup:?} lays out as {pages} pages, not one")]
    Pages {
        /// The label's snippet, as its key holds it.
        markup: String,
        /// How many pages it made.
        pages: usize,
    },
    /// Extra font bytes hold no face typst can read.
    #[error("extra font {index} holds no font face")]
    Font {
        /// The bytes' position among the extra fonts.
        index: usize,
    },
}
