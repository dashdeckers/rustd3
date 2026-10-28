//! The root README's iced code compiles: its `text` blocks, which the
//! facade's doctests cannot build (the facade has no rustd3-iced), are the
//! `readme_canvas` and `readme_plot` examples with the facade's paths.

/// The README, which is also the facade's crate documentation.
const README: &str = include_str!("../../../../README.md");

/// The README's code blocks fenced as `text`, in order.
fn text_blocks() -> Vec<String> {
    let mut blocks = Vec::new();
    let mut lines = README.lines();
    while let Some(line) = lines.next() {
        if line == "```text" {
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
fn the_readme_iced_code_is_the_compiled_examples() {
    let examples = [
        include_str!("../../examples/readme_canvas.rs"),
        include_str!("../../examples/readme_plot.rs"),
    ]
    .map(as_in_readme);
    assert_eq!(text_blocks(), examples);
}
