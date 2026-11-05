//! [`Memo`]: M4 output kept across draws.

use std::collections::HashMap;
use std::fmt;

use rustd3_core::series::{self, m4};

/// Entries unused for this many draws are dropped, as in the text cache.
const MAX_AGE: u64 = 32;

/// M4 output keyed by recording, revision, channel (two for a joint
/// query), column grid and validity test, kept in the figure's cache
/// across draws. A redraw of an unchanged view queries nothing, and an
/// append bumps the revision and misses.
#[derive(Default)]
pub(crate) struct Memo {
    entries: HashMap<Key, Entry>,
    generation: u64,
    queries: u64,
}

/// What one thinned line depends on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Key {
    /// The recording.
    pub(crate) recording: series::Id,
    /// Its revision.
    pub(crate) revision: u64,
    /// The channel.
    pub(crate) channel: usize,
    /// The upper edge's channel of a joint query; `None` for one channel.
    pub(crate) upper: Option<usize>,
    /// The column edges' hash.
    pub(crate) grid: u64,
    /// The validity test's fingerprint.
    pub(crate) valid: u64,
}

/// One line's M4 output and the draw that last used it.
struct Entry {
    lines: m4::Lines,
    used: u64,
}

/// `Memo { entries: 3, generation: 12, queries: 5 }`.
impl fmt::Debug for Memo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Memo")
            .field("entries", &self.entries.len())
            .field("generation", &self.generation)
            .field("queries", &self.queries)
            .finish()
    }
}

impl Memo {
    /// Starts a draw: entries unused for 32 draws are dropped.
    pub(crate) fn begin(&mut self) {
        self.generation += 1;
        let generation = self.generation;
        self.entries
            .retain(|_, entry| generation - entry.used <= MAX_AGE);
    }

    /// The M4 queries run: one per miss.
    pub(crate) fn queries(&self) -> u64 {
        self.queries
    }

    /// The output kept for `key`, marked used in this draw.
    pub(super) fn get(&mut self, key: &Key) -> Option<&m4::Lines> {
        let entry = self.entries.get_mut(key)?;
        entry.used = self.generation;
        Some(&entry.lines)
    }

    /// Keeps `lines`, the output of one M4 query, for `key`.
    pub(super) fn insert(&mut self, key: Key, lines: m4::Lines) {
        self.queries += 1;
        let used = self.generation;
        self.entries.insert(key, Entry { lines, used });
    }
}
