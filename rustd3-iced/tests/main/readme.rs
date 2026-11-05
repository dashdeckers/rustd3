//! Checks that the root README's `Plot` widget code compiles. The README's
//! last `rust` block must be the `readme_plot` example written with the
//! facade's paths; the facade cannot build it because it has no
//! rustd3-iced. The facade's `tests/main/readme.rs` checks the other blocks
//! against its own examples.

/// The README, whose `rust` blocks are compiled examples.
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

/// `example`'s code below its `//!` header, written with the facade's
/// paths: `rustd3::figure` for rustd3-figure and `rustd3` for rustd3-core.
fn as_in_readme(example: &str) -> String {
    let code: Vec<&str> = example
        .lines()
        .skip_while(|l| l.starts_with("//!"))
        .skip_while(|l| l.is_empty())
        .collect();
    (code.join("\n") + "\n")
        .replace("rustd3_figure", "rustd3::figure")
        .replace("rustd3_core", "rustd3")
}

#[test]
fn the_readme_plot_code_is_the_compiled_example() {
    let blocks = rust_blocks();
    assert_eq!(
        blocks.last(),
        Some(&as_in_readme(include_str!("../../examples/readme_plot.rs")))
    );
}
