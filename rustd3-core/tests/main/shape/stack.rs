//! Stacks against d3-shape 3.2.0's `stack()`, its orders and offsets, run
//! in JavaScript over the same columns as rows.

use rustd3_core::shape::{self, Stack, stack};

#[test]
fn stack_columns_example() {
    let apples = [3.0, 2.0, 1.0];
    let bananas = [1.0, 1.0, 4.0];
    let layers = shape::stack(&[&apples, &bananas]);
    assert_eq!(layers[0], [[0.0, 3.0], [0.0, 2.0], [0.0, 1.0]]);
    assert_eq!(layers[1], [[3.0, 4.0], [2.0, 3.0], [1.0, 5.0]]);
}

#[test]
fn expand_scales_each_row_to_fill_zero_to_one() {
    let layers = Stack::default()
        .offset(stack::Offset::Expand)
        .layout(&[&[3.0, 2.0, 1.0], &[1.0, 1.0, 4.0]]);
    assert_eq!(
        layers[0],
        [[0.0, 0.75], [0.0, 0.6666666666666666], [0.0, 0.2]]
    );
}

#[test]
fn diverging_stacks_negative_values_down_from_zero() {
    let layers = Stack::default()
        .offset(stack::Offset::Diverging)
        .layout(&[&[3.0, -2.0, 1.0], &[-1.0, 1.0, 4.0]]);
    assert_eq!(
        layers,
        [
            vec![[0.0, 3.0], [-2.0, 0.0], [0.0, 1.0]],
            vec![[-1.0, 0.0], [0.0, 1.0], [1.0, 5.0]]
        ]
    );
}

#[test]
fn silhouette_centres_each_row_on_zero() {
    let layers = Stack::default()
        .offset(stack::Offset::Silhouette)
        .layout(&[&[1.0, 2.0, 4.0], &[2.0, 3.0, 1.0]]);
    assert_eq!(layers[0], [[-1.5, -0.5], [-2.5, -0.5], [-2.5, 1.5]]);
}

#[test]
fn wiggle_moves_the_baseline_against_the_layers_change() {
    let layers = Stack::default()
        .offset(stack::Offset::Wiggle)
        .layout(&[&[1.0, 2.0, 4.0], &[2.0, 3.0, 1.0]]);
    assert_eq!(
        layers,
        [
            vec![[0.0, 1.0], [-1.1, 0.8999999999999999], [-2.1, 1.9]],
            vec![[1.0, 3.0], [0.8999999999999999, 3.9], [1.9, 2.9]]
        ]
    );
}

#[test]
fn inside_out_puts_the_earliest_peaks_in_the_middle() {
    let columns: [&[f64]; 3] = [&[1.0, 2.0, 3.0], &[5.0, 6.0, 7.0], &[2.0, 1.0, 3.0]];
    let inside_out = Stack::default().order(stack::Order::InsideOut);
    // d3's per-series index is 1, 2, 0.
    assert_eq!(inside_out.stacking_order(&columns), [2, 0, 1]);
    let layers = inside_out.layout(&columns);
    assert_eq!(layers[2][0], [0.0, 2.0]);
    assert_eq!(layers[0][0], [2.0, 3.0]);
}

#[test]
fn longer_columns_are_cut_to_the_shortest() {
    let layers = shape::stack(&[&[1.0, 2.0, 3.0], &[4.0]]);
    assert_eq!(layers, [vec![[0.0, 1.0]], vec![[1.0, 5.0]]]);
    assert!(shape::stack(&[]).is_empty());
}

#[test]
fn a_nan_value_stacks_as_zero_and_stays_nan() {
    let layers = shape::stack(&[&[1.0, 2.0], &[f64::NAN, 1.0], &[3.0, 1.0]]);
    assert!(layers[1][0][1].is_nan());
    assert_eq!(layers[2][0], [1.0, 4.0]);
}

#[test]
fn offsets_ignore_order_indices_past_the_last_layer() {
    let mut layers = vec![vec![[0.0, 1.0]], vec![[0.0, 2.0]]];
    stack::Offset::None.apply(&mut layers, &[0, 7, 1]);
    assert_eq!(layers, [vec![[0.0, 1.0]], vec![[1.0, 3.0]]]);
}
