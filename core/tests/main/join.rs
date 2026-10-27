//! `join`: the keyed diff against d3-selection 3.0.0's keyed data join
//! (`bindKey`, `selection/data.js:31–73`), run in Node over stand-in nodes
//! from the vendored sources.

use rustd3_core::join::{self, Join};

/// The join with these sides.
fn sides<'a>(enter: &[&'a str], update: &[&'a str], exit: &[&'a str]) -> Join<&'a str> {
    Join {
        enter: enter.to_vec(),
        update: update.to_vec(),
        exit: exit.to_vec(),
    }
}

#[test]
fn join_matches_d3_data_join() {
    let rows: [(&[&str], &[&str], Join<&str>); 6] = [
        // Part I, Data and identity.
        (
            &["a", "b", "c"],
            &["b", "c", "d"],
            sides(&["d"], &["b", "c"], &["a"]),
        ),
        (
            &["a", "a", "b"],
            &["a", "b", "b"],
            sides(&["b"], &["a", "b"], &["a"]),
        ),
        // The second copy on each side pairs with nothing.
        (&["a", "a"], &["a", "a"], sides(&["a"], &["a"], &["a"])),
        // Exits keep the old order, duplicates and unmatched keys mixed.
        (
            &["x", "a", "y", "a", "b"],
            &["b", "a", "c"],
            sides(&["c"], &["b", "a"], &["x", "y", "a"]),
        ),
        (&[], &["a", "a"], sides(&["a", "a"], &[], &[])),
        (&["a", "a"], &[], sides(&[], &[], &["a", "a"])),
    ];
    for (old, new, expected) in rows {
        assert_eq!(
            join::join(old.iter().copied(), new.iter().copied()),
            expected,
            "old {old:?}, new {new:?}"
        );
    }
}

#[test]
fn owned_keys_move_into_the_sides() {
    let old = vec![String::from("a"), String::from("b")];
    let new = vec![String::from("b"), String::from("c")];
    let plan = join::join(old, new);
    assert_eq!(plan.enter, ["c"]);
    assert_eq!(plan.update, ["b"]);
    assert_eq!(plan.exit, ["a"]);
}
