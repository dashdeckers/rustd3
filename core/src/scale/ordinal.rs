//! d3's scaleOrdinal: [`Ordinal`], keys of an explicit domain mapped to
//! range values in turn.

use std::fmt;

use crate::locale::Locale;
use crate::scale::{self, Ticks};
use crate::ticks;

/// d3.scaleOrdinal with an explicit domain: the key at index i maps to
/// `range[i % len]`, and any other key to the unknown value when one is
/// set. d3 appends an unseen key on every call; here that growth is the
/// opt-in [`apply_or_insert`](Ordinal::apply_or_insert), which needs the
/// scale mutably. Keys compare with `PartialEq`, so a NaN key never matches
/// (and `apply_or_insert` appends it again each time), where d3's
/// SameValueZero finds it.
///
/// ```
/// use rustd3_core::scale;
///
/// let shape = scale::ordinal(["cpu", "mem"], ["circle", "square"]);
/// assert_eq!(shape.apply(&"mem"), Some(&"square"));
/// assert_eq!(shape.apply(&"disk"), None);
///
/// let mut grow = scale::ordinal(Vec::<&str>::new(), [1.0, 2.0]);
/// assert_eq!(grow.apply_or_insert("disk"), Some(&1.0));
/// assert_eq!(grow.apply_or_insert("net"), Some(&2.0));
/// assert_eq!(grow.get_domain(), ["disk", "net"]);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Ordinal<K, V> {
    keys: Vec<K>,
    range: Vec<V>,
    unknown: Option<V>,
}

/// d3's `scaleOrdinal(keys, range)`, with no unknown value; duplicate keys
/// keep their first position.
#[doc(alias = "scaleOrdinal")]
pub fn ordinal<K: PartialEq, V>(
    keys: impl IntoIterator<Item = K>,
    range: impl IntoIterator<Item = V>,
) -> Ordinal<K, V> {
    Ordinal {
        keys: scale::unique(keys),
        range: range.into_iter().collect(),
        unknown: None,
    }
}

impl<K: PartialEq, V> Ordinal<K, V> {
    /// d3's `unknown(v)`: what [`apply`](Ordinal::apply) returns for a key
    /// outside the domain.
    #[must_use]
    pub fn unknown(self, value: V) -> Self {
        Ordinal {
            unknown: Some(value),
            ..self
        }
    }

    /// d3's `domain(keys)`: the keys in order, duplicates keeping their
    /// first position.
    #[must_use]
    pub fn domain(self, keys: impl IntoIterator<Item = K>) -> Self {
        Ordinal {
            keys: scale::unique(keys),
            ..self
        }
    }

    /// d3's `range(values)`: the values keys map to, reused in turn when
    /// there are more keys.
    #[must_use]
    pub fn range(self, range: impl IntoIterator<Item = V>) -> Self {
        Ordinal {
            range: range.into_iter().collect(),
            ..self
        }
    }

    /// `range[i % len]` for a key at index i; else the unknown value. `None`
    /// for a key outside the domain without an unknown value, and for any
    /// key while the range is empty.
    pub fn apply(&self, key: &K) -> Option<&V> {
        match self.keys.iter().position(|k| k == key) {
            Some(i) => self.cycled(i),
            None => self.unknown.as_ref(),
        }
    }

    /// d3's implicit growth, opt in: an unseen key is appended first.
    /// `None` only when the range is empty.
    pub fn apply_or_insert(&mut self, key: K) -> Option<&V> {
        let i = match self.keys.iter().position(|k| *k == key) {
            Some(i) => i,
            None => {
                self.keys.push(key);
                self.keys.len() - 1
            }
        };
        self.cycled(i)
    }

    /// d3's `domain()`: the keys, in order.
    pub fn get_domain(&self) -> &[K] {
        &self.keys
    }

    /// d3's `range()`.
    pub fn get_range(&self) -> &[V] {
        &self.range
    }

    /// The range value for the key at index `i`, cycling; `None` for an
    /// empty range.
    fn cycled(&self, i: usize) -> Option<&V> {
        self.range.get(i.checked_rem(self.range.len())?)
    }
}

/// Over numbers, an ordinal scale places its keys on an axis. Ordinal
/// scales over colors have none: legends read their keys.
impl<K: Clone + PartialEq + fmt::Display> Ticks for Ordinal<K, f64> {
    type Domain = K;
    type Step = ();

    /// Every key, whatever the count.
    fn tick_set(&self, count: usize) -> ticks::Set<K, ()> {
        ticks::Set::new(self.keys.clone(), (), count)
    }

    /// The key's value (d3-axis's `number`); the offset is unused.
    fn position(&self, value: &K, _offset: f64) -> Option<f64> {
        self.apply(value).copied()
    }

    /// Each key's `Display`, d3's identity format; the locale is unused.
    fn labels(&self, ticks: &ticks::Set<K, ()>, _locale: &Locale) -> Vec<String> {
        ticks.get_values().iter().map(ToString::to_string).collect()
    }

    /// The first and last range value; NaN for an empty range.
    fn extent(&self) -> [f64; 2] {
        match self.range.as_slice() {
            [first, .., last] => [*first, *last],
            [only] => [*only, *only],
            [] => [f64::NAN, f64::NAN],
        }
    }
}

impl<K: Clone + PartialEq + fmt::Display> crate::sealed::Ticks for Ordinal<K, f64> {}
