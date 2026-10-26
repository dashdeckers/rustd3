//! The keyed join that replaces `selection.data(data, key)`: which keys enter, stay and exit.

use std::collections::HashSet;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Join<K> {
    pub enter: Vec<K>,
    pub update: Vec<K>,
    pub exit: Vec<K>,
}

/// Compares the keys on screen with the keys in the next data, preserving each side's order.
pub fn join<K: Eq + Hash + Clone>(old: impl IntoIterator<Item = K>, new: impl IntoIterator<Item = K>) -> Join<K> {
    let old: Vec<K> = old.into_iter().collect();
    let new: Vec<K> = new.into_iter().collect();
    let old_set: HashSet<&K> = old.iter().collect();
    let new_set: HashSet<&K> = new.iter().collect();
    Join {
        enter: new.iter().filter(|k| !old_set.contains(k)).cloned().collect(),
        update: new.iter().filter(|k| old_set.contains(k)).cloned().collect(),
        exit: old.iter().filter(|k| !new_set.contains(k)).cloned().collect(),
    }
}
