//! The thread-local record of what the generated tests asserted.

use std::cell::RefCell;

use rustd3_testkit::obs::Obs;

use crate::prelude::Kind;

/// One call of `check`, `throws` or `skip`.
#[derive(Debug, Clone)]
pub struct Record {
    /// The assertion's id.
    pub id: &'static str,
    /// What the call found.
    pub mark: Mark,
}

/// What a record says about its assertion.
#[derive(Debug, Clone)]
pub enum Mark {
    /// Graded under d3's semantics and under the tolerant format.
    Checked {
        /// The d3 assertion it came from.
        kind: Kind,
        /// Whether it passes d3's semantics.
        d3: bool,
        /// Whether it passes the tolerant format.
        tolerant: bool,
        /// Our value.
        actual: Obs,
        /// d3's value.
        expected: Obs,
    },
    /// Skipped by the translator, for this reason.
    Skipped(&'static str),
}

thread_local! {
    static RECORDS: RefCell<Vec<Record>> = const { RefCell::new(Vec::new()) };
}

/// Appends `record` to this thread's records.
pub fn push(record: Record) {
    RECORDS.with_borrow_mut(|records| records.push(record));
}

/// This thread's records, in the order they were made, leaving none.
pub fn take() -> Vec<Record> {
    RECORDS.with_borrow_mut(std::mem::take)
}
