//! The README's code blocks are examples that compile. Its `rust` blocks,
//! in order, are `examples/figure_png.rs`, `examples/core_level.rs` and
//! rustd3-iced's `readme_plot`, each below its `//!` header, so the README
//! cannot drift from code that builds. rustd3-iced's tests check the last
//! block, with the facade's paths.

const README: &str = include_str!("../../../README.md");

/// The README's code blocks fenced as `rust`, in order.
fn rust_blocks() -> Vec<String> {
    let mut blocks = Vec::new();
    let mut lines = README.lines();
    while let Some(line) = lines.next() {
        if line == "```rust" {
            let block: Vec<&str> = lines.by_ref().take_while(|l| *l != "```").collect();
            blocks.push(block.join("\n") + "\n");
        }
    }
    blocks
}

/// `example`'s code below its `//!` header.
fn code(example: &str) -> String {
    let lines: Vec<&str> = example
        .lines()
        .skip_while(|l| l.starts_with("//!"))
        .skip_while(|l| l.is_empty())
        .collect();
    lines.join("\n") + "\n"
}

#[test]
fn the_readme_rust_blocks_are_the_facade_examples() {
    let blocks = rust_blocks();
    assert_eq!(
        blocks.len(),
        3,
        "a figure to PNG, the core level, the Plot widget"
    );
    assert_eq!(
        blocks[0],
        code(include_str!("../../examples/figure_png.rs"))
    );
    assert_eq!(
        blocks[1],
        code(include_str!("../../examples/core_level.rs"))
    );
}

#[test]
fn the_readme_code_has_no_hidden_lines() {
    for block in rust_blocks() {
        assert!(
            block
                .lines()
                .all(|l| !l.trim_start().starts_with("# ") && l.trim() != "#"),
            "{block}"
        );
    }
}
