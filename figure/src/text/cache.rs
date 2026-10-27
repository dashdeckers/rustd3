//! [`Cache`]: shaped labels kept across figure draws, bounded by
//! generations.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::text::{Error, Fonts, Key, Shaped, batch};

/// Entries unused for this many generations are dropped, and comemo's
/// results unused for this many evictions; an eviction runs once per this
/// many [`Cache::begin`] calls in the process.
const MAX_AGE: u64 = 32;

/// Every [`Cache::begin`] in the process, for comemo's eviction cadence.
static DRAWS: AtomicU64 = AtomicU64::new(0);

/// Shaped labels by key, owned by the caller: a figure's cache, or a
/// widget's tree state. Each figure draw starts a generation; labels
/// unused for 32 generations are dropped, so a long pan does not grow the
/// cache without bound. A label typst rejects is cached as its error.
#[derive(Debug)]
pub struct Cache {
    fonts: Arc<Fonts>,
    entries: HashMap<Key, Entry>,
    generation: u64,
    compiles: u64,
    hits: u64,
    misses: u64,
}

/// One cached label and the generation that last used it.
#[derive(Debug)]
struct Entry {
    shaped: Result<Arc<Shaped>, Error>,
    used: u64,
}

/// What a cache holds and has done.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    /// Labels cached, shaped or failed.
    pub entries: usize,
    /// typst compilations run: one per batch, plus one per label of a
    /// batch that had to be compiled label by label.
    pub compiles: u64,
    /// Keys [`prepare`](Cache::prepare) found cached, or already queued
    /// in the same call.
    pub hits: u64,
    /// Keys [`prepare`](Cache::prepare) shaped.
    pub misses: u64,
}

#[cfg(feature = "fonts")]
impl Default for Cache {
    /// A cache over [`Fonts::bundled`].
    fn default() -> Cache {
        Cache::new(Fonts::bundled())
    }
}

impl Cache {
    /// An empty cache shaping with `fonts`.
    pub fn new(fonts: Arc<Fonts>) -> Self {
        Cache {
            fonts,
            entries: HashMap::new(),
            generation: 0,
            compiles: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// Starts a figure draw: the generation advances and entries unused
    /// for 32 generations are dropped. Every 32nd call in the process runs
    /// `typst::comemo::evict(32)`, which ages typst's process-wide memo
    /// cache and drops results unused for 32 evictions. The cadence counts
    /// the calls of every cache, so with two widgets, or a widget beside
    /// file output, eviction runs more often for each; that only ages
    /// results sooner.
    pub fn begin(&mut self) {
        self.generation += 1;
        let generation = self.generation;
        self.entries
            .retain(|_, entry| generation - entry.used <= MAX_AGE);
        if DRAWS.fetch_add(1, Ordering::Relaxed) % MAX_AGE == MAX_AGE - 1 {
            typst::comemo::evict(MAX_AGE as usize);
        }
    }

    /// Shapes every key not cached, all in one typst document, and marks
    /// every key used in this generation.
    pub fn prepare<'k>(&mut self, keys: impl IntoIterator<Item = &'k Key>) {
        let mut todo: Vec<&Key> = Vec::new();
        let mut queued = HashSet::new();
        for key in keys {
            if let Some(entry) = self.entries.get_mut(key) {
                entry.used = self.generation;
                self.hits += 1;
            } else if queued.insert(key) {
                todo.push(key);
                self.misses += 1;
            } else {
                self.hits += 1;
            }
        }
        if todo.is_empty() {
            return;
        }
        let shaped = batch::shape(&self.fonts, &todo, &mut self.compiles);
        for (key, shaped) in todo.into_iter().zip(shaped) {
            let entry = Entry {
                shaped: shaped.map(Arc::new),
                used: self.generation,
            };
            self.entries.insert(key.clone(), entry);
        }
    }

    /// The label of `key`, if it is cached: shaped, or typst's error.
    pub fn get(&self, key: &Key) -> Option<Result<&Shaped, &Error>> {
        self.entries
            .get(key)
            .map(|entry| entry.shaped.as_ref().map(|shaped| &**shaped))
    }

    /// Every cached label: shaped, or typst's error.
    #[cfg(feature = "fonts")]
    pub(crate) fn entries(&self) -> impl Iterator<Item = (&Key, Result<&Shaped, &Error>)> {
        self.entries
            .iter()
            .map(|(key, entry)| (key, entry.shaped.as_ref().map(|shaped| &**shaped)))
    }

    /// The faces labels are shaped with.
    pub(crate) fn fonts(&self) -> &Fonts {
        &self.fonts
    }

    /// What the cache holds and has done.
    pub fn stats(&self) -> Stats {
        Stats {
            entries: self.entries.len(),
            compiles: self.compiles,
            hits: self.hits,
            misses: self.misses,
        }
    }
}
