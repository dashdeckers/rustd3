//! [`Cache`]: what a caller keeps between draws of a figure, and its
//! [`Stats`].

use std::sync::Arc;

use crate::scale::visible;
use crate::{text, thin};

/// What a caller keeps between draws of a figure: the text cache, so no
/// label is shaped twice, the M4 memo, so an unchanged view is not thinned
/// twice, and the last domains of `Fit::Visible` handles, so they shrink
/// by steps. The iced widget keeps one in its tree state; file outputs make
/// a temporary one.
#[derive(Debug)]
pub struct Cache {
    text: text::Cache,
    memo: thin::Memo,
    visible: visible::Memory,
}

/// What a figure cache holds and has done. Plain data.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    /// The text cache's.
    pub text: text::Stats,
    /// M4 queries run for thinned lines: one per line whose recording,
    /// revision, channel or column grid the memo had not seen.
    pub m4_queries: u64,
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
            memo: thin::Memo::default(),
            visible: visible::Memory::default(),
        }
    }

    /// Starts a figure draw: the text cache's generation advances
    /// ([`text::Cache::begin`]), and memoized M4 output unused for 32 draws
    /// is dropped.
    pub fn begin(&mut self) {
        self.text.begin();
        self.memo.begin();
    }

    /// What the cache holds and has done.
    pub fn stats(&self) -> Stats {
        Stats {
            text: self.text.stats(),
            m4_queries: self.memo.queries(),
        }
    }

    /// The last domains of `Fit::Visible` handles.
    pub(crate) fn visible(&mut self) -> &mut visible::Memory {
        &mut self.visible
    }

    /// The text cache, for resolving.
    pub(crate) fn text(&mut self) -> &mut text::Cache {
        &mut self.text
    }

    /// The text cache, and a frame for one draw's thinning over the memo,
    /// which [`end`](Cache::end) takes back.
    pub(crate) fn start(&mut self) -> (&text::Cache, thin::Frame) {
        let memo = std::mem::take(&mut self.memo);
        (&self.text, thin::Frame::new(memo))
    }

    /// Keeps the memo of a draw's `frame` for the next draw.
    pub(crate) fn end(&mut self, frame: thin::Frame) {
        self.memo = frame.into_memo();
    }
}
