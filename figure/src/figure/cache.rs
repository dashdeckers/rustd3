//! [`Cache`]: what a caller keeps between draws of a figure, and its
//! [`Stats`].

use std::sync::Arc;

use crate::text;

/// What a caller keeps between draws of a figure: the text cache, so no
/// label is shaped twice. The iced widget keeps one in its tree state;
/// file outputs make a temporary one.
#[derive(Debug)]
pub struct Cache {
    text: text::Cache,
}

/// What a figure cache holds and has done. Plain data.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    /// The text cache's.
    pub text: text::Stats,
}

#[cfg(feature = "fonts")]
impl Default for Cache {
    /// A cache over [`text::Fonts::bundled`].
    fn default() -> Cache {
        Cache::new(text::Fonts::bundled())
    }
}

impl Cache {
    /// An empty cache shaping text with `fonts`.
    pub fn new(fonts: Arc<text::Fonts>) -> Self {
        Cache {
            text: text::Cache::new(fonts),
        }
    }

    /// Starts a figure draw: the text cache's generation advances
    /// ([`text::Cache::begin`]).
    pub fn begin(&mut self) {
        self.text.begin();
    }

    /// What the cache holds and has done.
    pub fn stats(&self) -> Stats {
        Stats {
            text: self.text.stats(),
        }
    }

    /// The text cache, for resolving.
    pub(super) fn text(&mut self) -> &mut text::Cache {
        &mut self.text
    }
}
