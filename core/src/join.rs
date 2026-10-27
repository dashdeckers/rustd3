//! d3-selection's keyed data join as a diff of keys: which marks enter,
//! stay and exit when the data changes (`selection.data(data, key)`).

use std::collections::HashMap;
use std::hash::Hash;

/// The three sides of a keyed join, each in its own order: `enter` and
/// `update` in the new keys' order, `exit` in the old keys'.
///
/// Plain data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Join<K> {
    /// Keys of the new data that no old key matched.
    pub enter: Vec<K>,
    /// Keys of the new data that matched an old key.
    pub update: Vec<K>,
    /// Old keys that no new key matched.
    pub exit: Vec<K>,
}

/// Compares the keys on screen, `old`, with the keys of the next data,
/// `new`, as d3-selection's keyed data join does (`bindKey`,
/// `selection/data.js:31–73`).
///
/// The first occurrence of a key in `new` pairs with its first occurrence
/// in `old`; further copies in `new` enter, and further copies in `old`
/// exit.
///
/// ```
/// use rustd3_core::join::{self, Join};
///
/// let plan = join::join(["a", "b", "c"], ["b", "c", "d"]);
/// assert_eq!(
///     plan,
///     Join { enter: vec!["d"], update: vec!["b", "c"], exit: vec!["a"] }
/// );
///
/// let duplicates = join::join(["a", "a", "b"], ["a", "b", "b"]);
/// assert_eq!(duplicates.enter, ["b"]);
/// assert_eq!(duplicates.update, ["a", "b"]);
/// assert_eq!(duplicates.exit, ["a"]);
/// ```
pub fn join<K: Eq + Hash + Clone>(
    old: impl IntoIterator<Item = K>,
    new: impl IntoIterator<Item = K>,
) -> Join<K> {
    let old: Vec<K> = old.into_iter().collect();
    // The first occurrence of each old key, until a new key takes it.
    let mut first: HashMap<&K, usize> = HashMap::with_capacity(old.len());
    for (i, key) in old.iter().enumerate() {
        first.entry(key).or_insert(i);
    }
    let mut matched = vec![false; old.len()];
    let mut enter = Vec::new();
    let mut update = Vec::new();
    for key in new {
        match first.remove(&key) {
            Some(i) => {
                matched[i] = true;
                update.push(key);
            }
            None => enter.push(key),
        }
    }
    let exit = old
        .into_iter()
        .zip(matched)
        .filter_map(|(key, matched)| (!matched).then_some(key))
        .collect();
    Join {
        enter,
        update,
        exit,
    }
}
