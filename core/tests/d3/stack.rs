//! d3's tests of d3-shape, as assertions on rustd3-core; each names d3's file, line and column.

use rustd3_core::shape;

use super::support::{assert_d3_eq, js_round};

// d3-shape/test/stack-test.js: stack(data) computes the stacked series for the given data
#[test]
fn stack_data_computes_the_stacked_series_for_the_given_data() {
    let mut _data = [
        vec![1.0, 3.0, 5.0, 1.0],
        vec![2.0, 4.0, 2.0, 3.0],
        vec![1.0, 2.0, 4.0, 2.0],
    ];
    assert_eq!(
        {
            let columns = [
                &[1.0, 2.0, 1.0][..],
                &[3.0, 4.0, 2.0][..],
                &[5.0, 2.0, 4.0][..],
                &[1.0, 3.0, 2.0][..],
            ];
            (
                shape::Stack::default().layout(&columns),
                shape::Stack::default().stacking_order(&columns),
            )
        },
        (
            vec![
                vec![[0.0, 1.0], [0.0, 2.0], [0.0, 1.0]],
                vec![[1.0, 4.0], [2.0, 6.0], [1.0, 3.0]],
                vec![[4.0, 9.0], [6.0, 8.0], [3.0, 7.0]],
                vec![[9.0, 10.0], [8.0, 11.0], [7.0, 9.0]]
            ],
            vec![0, 1, 2, 3usize]
        )
    ); // d3-shape/test/stack-test.js:15:3
}

// d3-shape/test/stack-test.js: stack.order(function) sets the order function
#[test]
fn stack_order_function_sets_the_order_function() {
    let mut _data = [
        vec![1.0, 3.0, 5.0, 1.0],
        vec![2.0, 4.0, 2.0, 3.0],
        vec![1.0, 2.0, 4.0, 2.0],
    ];
    assert_eq!(
        {
            let columns = [
                &[1.0, 2.0, 1.0][..],
                &[3.0, 4.0, 2.0][..],
                &[5.0, 2.0, 4.0][..],
                &[1.0, 3.0, 2.0][..],
            ];
            (
                shape::Stack::default()
                    .order(shape::stack::Order::Reverse)
                    .layout(&columns),
                shape::Stack::default()
                    .order(shape::stack::Order::Reverse)
                    .stacking_order(&columns),
            )
        },
        (
            vec![
                vec![[9.0, 10.0], [9.0, 11.0], [8.0, 9.0]],
                vec![[6.0, 9.0], [5.0, 9.0], [6.0, 8.0]],
                vec![[1.0, 6.0], [3.0, 5.0], [2.0, 6.0]],
                vec![[0.0, 1.0], [0.0, 3.0], [0.0, 2.0]]
            ],
            vec![3, 2, 1, 0usize]
        )
    ); // d3-shape/test/stack-test.js:86:3
}

// d3-shape/test/stack-test.js: stack.offset(function) sets the offset function
#[test]
fn stack_offset_function_sets_the_offset_function() {
    let mut _data = [
        vec![1.0, 3.0, 5.0, 1.0],
        vec![2.0, 4.0, 2.0, 3.0],
        vec![1.0, 2.0, 4.0, 2.0],
    ];
    assert_eq!(
        {
            let columns = [
                &[1.0, 2.0, 1.0][..],
                &[3.0, 4.0, 2.0][..],
                &[5.0, 2.0, 4.0][..],
                &[1.0, 3.0, 2.0][..],
            ];
            (
                shape::Stack::default()
                    .offset(shape::stack::Offset::Expand)
                    .layout(&columns),
                shape::Stack::default()
                    .offset(shape::stack::Offset::Expand)
                    .stacking_order(&columns),
            )
        }
        .0
        .iter()
        .map(|l| l
            .iter()
            .map(|p| p.map(|v| js_round(v * 1e6) / 1e6))
            .collect::<Vec<_>>())
        .collect::<Vec<_>>(),
        [
            vec![
                [(0.0 / 10.0), (1.0 / 10.0)],
                [(0.0 / 11.0), (2.0 / 11.0)],
                [(0.0 / 9.0), (1.0 / 9.0)]
            ],
            vec![
                [(1.0 / 10.0), (4.0 / 10.0)],
                [(2.0 / 11.0), (6.0 / 11.0)],
                [(1.0 / 9.0), (3.0 / 9.0)]
            ],
            vec![
                [(4.0 / 10.0), (9.0 / 10.0)],
                [(6.0 / 11.0), (8.0 / 11.0)],
                [(3.0 / 9.0), (7.0 / 9.0)]
            ],
            vec![
                [(9.0 / 10.0), (10.0 / 10.0)],
                [(8.0 / 11.0), (11.0 / 11.0)],
                [(7.0 / 9.0), (9.0 / 9.0)]
            ]
        ]
        .iter()
        .map(|l| l
            .iter()
            .map(|p| p.map(|v| js_round(v * 1e6) / 1e6))
            .collect::<Vec<_>>())
        .collect::<Vec<_>>()
    ); // d3-shape/test/stack-test.js:104:3
}

// d3-shape/test/diverging-test.js: stackOffsetDiverging(series, order) applies a zero baseline, ignoring existing offsets
#[test]
fn stackoffsetdiverging_series_order_applies_a_zero_baseline_ignoring_existing_offsets() {
    let mut series = vec![
        vec![[1.0, 2.0], [2.0, 4.0], [3.0, 4.0]],
        vec![[0.0, 3.0], [0.0, 4.0], [0.0, 2.0]],
        vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]],
    ];
    {
        let order = shape::stack::Order::None.apply(&series);
        shape::stack::Offset::Diverging.apply(&mut series, &order);
    };
    assert_eq!(
        series,
        vec![
            vec![[0.0, 1.0], [0.0, 2.0], [0.0, 1.0]],
            vec![[1.0, 4.0], [2.0, 6.0], [1.0, 3.0]],
            vec![[4.0, 9.0], [6.0, 8.0], [3.0, 7.0]]
        ]
    ); // d3-shape/test/offset/diverging-test.js:11:3
}

// d3-shape/test/diverging-test.js: stackOffsetDiverging(series, order) handles a single series
#[test]
fn stackoffsetdiverging_series_order_handles_a_single_series() {
    let mut series = vec![vec![[1.0, 2.0], [2.0, 4.0], [3.0, 4.0]]];
    {
        let order = shape::stack::Order::None.apply(&series);
        shape::stack::Offset::Diverging.apply(&mut series, &order);
    };
    assert_eq!(series, vec![vec![[0.0, 1.0], [0.0, 2.0], [0.0, 1.0]]]); // d3-shape/test/offset/diverging-test.js:23:3
}

// d3-shape/test/diverging-test.js: stackOffsetDiverging(series, order) treats NaN as zero
#[test]
fn stackoffsetdiverging_series_order_treats_nan_as_zero() {
    let mut series = vec![
        vec![[0.0, 1.0], [0.0, 2.0], [0.0, 1.0]],
        vec![[0.0, 3.0], [0.0, f64::NAN], [0.0, 2.0]],
        vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]],
    ];
    {
        let order = shape::stack::Order::None.apply(&series);
        shape::stack::Offset::Diverging.apply(&mut series, &order);
    };
    assert!((series[1][1][1]).is_nan()); // d3-shape/test/offset/diverging-test.js:35:3
}

// d3-shape/test/diverging-test.js: stackOffsetDiverging(series, order) observes the specified order
#[test]
fn stackoffsetdiverging_series_order_observes_the_specified_order() {
    let mut series = vec![
        vec![[0.0, 1.0], [0.0, 2.0], [0.0, 1.0]],
        vec![[0.0, 3.0], [0.0, 4.0], [0.0, 2.0]],
        vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]],
    ];
    {
        let order = shape::stack::Order::Reverse.apply(&series);
        shape::stack::Offset::Diverging.apply(&mut series, &order);
    };
    assert_eq!(
        series,
        vec![
            vec![[8.0, 9.0], [6.0, 8.0], [6.0, 7.0]],
            vec![[5.0, 8.0], [2.0, 6.0], [4.0, 6.0]],
            vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]]
        ]
    ); // d3-shape/test/offset/diverging-test.js:51:3
}

// d3-shape/test/diverging-test.js: stackOffsetDiverging(series, order) puts negative values below zero, in order
#[test]
fn stackoffsetdiverging_series_order_puts_negative_values_below_zero_in_order() {
    let mut series = vec![
        vec![[0.0, 1.0], [0.0, -2.0], [0.0, -1.0]],
        vec![[0.0, -3.0], [0.0, -4.0], [0.0, -2.0]],
        vec![[0.0, -5.0], [0.0, -2.0], [0.0, 4.0]],
    ];
    {
        let order = shape::stack::Order::None.apply(&series);
        shape::stack::Offset::Diverging.apply(&mut series, &order);
    };
    assert_eq!(
        series,
        vec![
            vec![[0.0, 1.0], [-2.0, 0.0], [-1.0, 0.0]],
            vec![[-3.0, 0.0], [-6.0, -2.0], [-3.0, -1.0]],
            vec![[-8.0, -3.0], [-8.0, -6.0], [0.0, 4.0]]
        ]
    ); // d3-shape/test/offset/diverging-test.js:65:3
}

// d3-shape/test/diverging-test.js: stackOffsetDiverging(series, order) puts zero values at zero, in order
#[test]
fn stackoffsetdiverging_series_order_puts_zero_values_at_zero_in_order() {
    let mut series = vec![
        vec![[0.0, 1.0], [0.0, 2.0], [0.0, -1.0]],
        vec![[0.0, 3.0], [0.0, 0.0], [0.0, 0.0]],
        vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]],
    ];
    {
        let order = shape::stack::Order::None.apply(&series);
        shape::stack::Offset::Diverging.apply(&mut series, &order);
    };
    assert_eq!(
        series,
        vec![
            vec![[0.0, 1.0], [0.0, 2.0], [-1.0, 0.0]],
            vec![[1.0, 4.0], [0.0, 0.0], [0.0, 0.0]],
            vec![[4.0, 9.0], [2.0, 4.0], [0.0, 4.0]]
        ]
    ); // d3-shape/test/offset/diverging-test.js:79:3
}

// d3-shape/test/expand-test.js: stackOffsetExpand(series, order) expands to fill [0, 1]
#[test]
fn stackoffsetexpand_series_order_expands_to_fill_0_1() {
    let mut series = vec![
        vec![[0.0, 1.0], [0.0, 2.0], [0.0, 1.0]],
        vec![[0.0, 3.0], [0.0, 4.0], [0.0, 2.0]],
        vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]],
    ];
    {
        let order = shape::stack::Order::None.apply(&series);
        shape::stack::Offset::Expand.apply(&mut series, &order);
    };
    assert_eq!(
        series,
        vec![
            vec![
                [(0.0 / 9.0), (1.0 / 9.0)],
                [(0.0 / 8.0), (2.0 / 8.0)],
                [(0.0 / 7.0), (1.0 / 7.0)]
            ],
            vec![
                [(1.0 / 9.0), (4.0 / 9.0)],
                [(2.0 / 8.0), (6.0 / 8.0)],
                [(1.0 / 7.0), (3.0 / 7.0)]
            ],
            vec![
                [(4.0 / 9.0), (9.0 / 9.0)],
                [(6.0 / 8.0), (8.0 / 8.0)],
                [(3.0 / 7.0), (7.0 / 7.0)]
            ]
        ]
    ); // d3-shape/test/offset/expand-test.js:11:3
}

// d3-shape/test/expand-test.js: stackOffsetExpand(series, order) treats NaN as zero
#[test]
fn stackoffsetexpand_series_order_treats_nan_as_zero() {
    let mut series = vec![
        vec![[0.0, 1.0], [0.0, 2.0], [0.0, 1.0]],
        vec![[0.0, 3.0], [0.0, f64::NAN], [0.0, 2.0]],
        vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]],
    ];
    {
        let order = shape::stack::Order::None.apply(&series);
        shape::stack::Offset::Expand.apply(&mut series, &order);
    };
    assert!((series[1][1][1]).is_nan()); // d3-shape/test/offset/expand-test.js:25:3
}

// d3-shape/test/expand-test.js: stackOffsetExpand(series, order) observes the specified order
#[test]
fn stackoffsetexpand_series_order_observes_the_specified_order() {
    let mut series = vec![
        vec![[0.0, 1.0], [0.0, 2.0], [0.0, 1.0]],
        vec![[0.0, 3.0], [0.0, 4.0], [0.0, 2.0]],
        vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]],
    ];
    {
        let order = shape::stack::Order::Reverse.apply(&series);
        shape::stack::Offset::Expand.apply(&mut series, &order);
    };
    assert_eq!(
        series,
        vec![
            vec![
                [(8.0 / 9.0), (9.0 / 9.0)],
                [(6.0 / 8.0), (8.0 / 8.0)],
                [(6.0 / 7.0), (7.0 / 7.0)]
            ],
            vec![
                [(5.0 / 9.0), (8.0 / 9.0)],
                [(2.0 / 8.0), (6.0 / 8.0)],
                [(4.0 / 7.0), (6.0 / 7.0)]
            ],
            vec![
                [(0.0 / 9.0), (5.0 / 9.0)],
                [(0.0 / 8.0), (2.0 / 8.0)],
                [(0.0 / 7.0), (4.0 / 7.0)]
            ]
        ]
    ); // d3-shape/test/offset/expand-test.js:41:3
}

// d3-shape/test/none-test.js: stackOffsetNone(series, order) stacks upon the first layer’s existing positions
#[test]
fn stackoffsetnone_series_order_stacks_upon_the_first_layer_s_existing_positions() {
    let mut series = vec![
        vec![[1.0, 2.0], [2.0, 4.0], [3.0, 4.0]],
        vec![[0.0, 3.0], [0.0, 4.0], [0.0, 2.0]],
        vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]],
    ];
    {
        let order = shape::stack::Order::None.apply(&series);
        shape::stack::Offset::None.apply(&mut series, &order);
    };
    assert_eq!(
        series,
        vec![
            vec![[1.0, 2.0], [2.0, 4.0], [3.0, 4.0]],
            vec![[2.0, 5.0], [4.0, 8.0], [4.0, 6.0]],
            vec![[5.0, 10.0], [8.0, 10.0], [6.0, 10.0]]
        ]
    ); // d3-shape/test/offset/none-test.js:11:3
}

// d3-shape/test/none-test.js: stackOffsetNone(series, order) treats NaN as zero
#[test]
fn stackoffsetnone_series_order_treats_nan_as_zero() {
    let mut series = vec![
        vec![[0.0, 1.0], [0.0, 2.0], [0.0, 1.0]],
        vec![[0.0, 3.0], [0.0, f64::NAN], [0.0, 2.0]],
        vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]],
    ];
    {
        let order = shape::stack::Order::None.apply(&series);
        shape::stack::Offset::None.apply(&mut series, &order);
    };
    assert!((series[1][1][1]).is_nan()); // d3-shape/test/offset/none-test.js:25:3
}

// d3-shape/test/none-test.js: stackOffsetNone(series, order) observes the specified order
#[test]
fn stackoffsetnone_series_order_observes_the_specified_order() {
    let mut series = vec![
        vec![[0.0, 1.0], [0.0, 2.0], [0.0, 1.0]],
        vec![[0.0, 3.0], [0.0, 4.0], [0.0, 2.0]],
        vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]],
    ];
    {
        let order = shape::stack::Order::Reverse.apply(&series);
        shape::stack::Offset::None.apply(&mut series, &order);
    };
    assert_eq!(
        series,
        vec![
            vec![[8.0, 9.0], [6.0, 8.0], [6.0, 7.0]],
            vec![[5.0, 8.0], [2.0, 6.0], [4.0, 6.0]],
            vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]]
        ]
    ); // d3-shape/test/offset/none-test.js:41:3
}

// d3-shape/test/silhouette-test.js: stackOffsetSilhouette(series, order) centers the stack around zero
#[test]
fn stackoffsetsilhouette_series_order_centers_the_stack_around_zero() {
    let mut series = vec![
        vec![[0.0, 1.0], [0.0, 2.0], [0.0, 1.0]],
        vec![[0.0, 3.0], [0.0, 4.0], [0.0, 2.0]],
        vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]],
    ];
    {
        let order = shape::stack::Order::None.apply(&series);
        shape::stack::Offset::Silhouette.apply(&mut series, &order);
    };
    assert_eq!(
        series,
        vec![
            vec![
                [(0.0 - (9.0 / 2.0)), (1.0 - (9.0 / 2.0))],
                [(0.0 - (8.0 / 2.0)), (2.0 - (8.0 / 2.0))],
                [(0.0 - (7.0 / 2.0)), (1.0 - (7.0 / 2.0))]
            ],
            vec![
                [(1.0 - (9.0 / 2.0)), (4.0 - (9.0 / 2.0))],
                [(2.0 - (8.0 / 2.0)), (6.0 - (8.0 / 2.0))],
                [(1.0 - (7.0 / 2.0)), (3.0 - (7.0 / 2.0))]
            ],
            vec![
                [(4.0 - (9.0 / 2.0)), (9.0 - (9.0 / 2.0))],
                [(6.0 - (8.0 / 2.0)), (8.0 - (8.0 / 2.0))],
                [(3.0 - (7.0 / 2.0)), (7.0 - (7.0 / 2.0))]
            ]
        ]
    ); // d3-shape/test/offset/silhouette-test.js:11:3
}

// d3-shape/test/silhouette-test.js: stackOffsetSilhouette(series, order) treats NaN as zero
#[test]
fn stackoffsetsilhouette_series_order_treats_nan_as_zero() {
    let mut series = vec![
        vec![[0.0, 1.0], [0.0, 2.0], [0.0, 1.0]],
        vec![[0.0, 3.0], [0.0, f64::NAN], [0.0, 2.0]],
        vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]],
    ];
    {
        let order = shape::stack::Order::None.apply(&series);
        shape::stack::Offset::Silhouette.apply(&mut series, &order);
    };
    assert!((series[1][1][1]).is_nan()); // d3-shape/test/offset/silhouette-test.js:25:3
}

// d3-shape/test/silhouette-test.js: stackOffsetSilhouette(series, order) observes the specified order
#[test]
fn stackoffsetsilhouette_series_order_observes_the_specified_order() {
    let mut series = vec![
        vec![[0.0, 1.0], [0.0, 2.0], [0.0, 1.0]],
        vec![[0.0, 3.0], [0.0, 4.0], [0.0, 2.0]],
        vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]],
    ];
    {
        let order = shape::stack::Order::Reverse.apply(&series);
        shape::stack::Offset::Silhouette.apply(&mut series, &order);
    };
    assert_eq!(
        series,
        vec![
            vec![
                [(8.0 - (9.0 / 2.0)), (9.0 - (9.0 / 2.0))],
                [(6.0 - (8.0 / 2.0)), (8.0 - (8.0 / 2.0))],
                [(6.0 - (7.0 / 2.0)), (7.0 - (7.0 / 2.0))]
            ],
            vec![
                [(5.0 - (9.0 / 2.0)), (8.0 - (9.0 / 2.0))],
                [(2.0 - (8.0 / 2.0)), (6.0 - (8.0 / 2.0))],
                [(4.0 - (7.0 / 2.0)), (6.0 - (7.0 / 2.0))]
            ],
            vec![
                [(0.0 - (9.0 / 2.0)), (5.0 - (9.0 / 2.0))],
                [(0.0 - (8.0 / 2.0)), (2.0 - (8.0 / 2.0))],
                [(0.0 - (7.0 / 2.0)), (4.0 - (7.0 / 2.0))]
            ]
        ]
    ); // d3-shape/test/offset/silhouette-test.js:41:3
}

// d3-shape/test/wiggle-test.js: stackOffsetWiggle(series, order) minimizes weighted wiggle
#[test]
fn stackoffsetwiggle_series_order_minimizes_weighted_wiggle() {
    let mut series = vec![
        vec![[0.0, 1.0], [0.0, 2.0], [0.0, 1.0]],
        vec![[0.0, 3.0], [0.0, 4.0], [0.0, 2.0]],
        vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]],
    ];
    {
        let order = shape::stack::Order::None.apply(&series);
        shape::stack::Offset::Wiggle.apply(&mut series, &order);
    };
    assert_eq!(
        series
            .iter()
            .map(|l| l
                .iter()
                .map(|p| p.map(|v| js_round(v * 1e6) / 1e6))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [
            vec![[0.0, 1.0], [-1.0, 1.0], [0.7857143, 1.7857143]],
            vec![[1.0, 4.0], [1.0, 5.0], [1.7857143, 3.7857143]],
            vec![[4.0, 9.0], [5.0, 7.0], [3.7857143, 7.7857143]]
        ]
        .iter()
        .map(|l| l
            .iter()
            .map(|p| p.map(|v| js_round(v * 1e6) / 1e6))
            .collect::<Vec<_>>())
        .collect::<Vec<_>>()
    ); // d3-shape/test/offset/wiggle-test.js:11:3
}

// d3-shape/test/wiggle-test.js: stackOffsetWiggle(series, order) treats NaN as zero
#[test]
fn stackoffsetwiggle_series_order_treats_nan_as_zero() {
    let mut series = vec![
        vec![[0.0, 1.0], [0.0, 2.0], [0.0, 1.0]],
        vec![[0.0, f64::NAN], [0.0, f64::NAN], [0.0, f64::NAN]],
        vec![[0.0, 3.0], [0.0, 4.0], [0.0, 2.0]],
        vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]],
    ];
    {
        let order = shape::stack::Order::None.apply(&series);
        shape::stack::Offset::Wiggle.apply(&mut series, &order);
    };
    assert!((series[1][0][1]).is_nan()); // d3-shape/test/offset/wiggle-test.js:26:3
}

// d3-shape/test/wiggle-test.js: stackOffsetWiggle(series, order) observes the specified order
#[test]
fn stackoffsetwiggle_series_order_observes_the_specified_order() {
    let mut series = vec![
        vec![[0.0, 1.0], [0.0, 2.0], [0.0, 1.0]],
        vec![[0.0, 3.0], [0.0, 4.0], [0.0, 2.0]],
        vec![[0.0, 5.0], [0.0, 2.0], [0.0, 4.0]],
    ];
    {
        let order = shape::stack::Order::Reverse.apply(&series);
        shape::stack::Offset::Wiggle.apply(&mut series, &order);
    };
    assert_eq!(
        series
            .iter()
            .map(|l| l
                .iter()
                .map(|p| p.map(|v| js_round(v * 1e6) / 1e6))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [
            vec![[8.0, 9.0], [8.0, 10.0], [7.21428571, 8.21428571]],
            vec![[5.0, 8.0], [4.0, 8.0], [5.21428571, 7.21428571]],
            vec![[0.0, 5.0], [2.0, 4.0], [1.21428571, 5.21428571]]
        ]
        .iter()
        .map(|l| l
            .iter()
            .map(|p| p.map(|v| js_round(v * 1e6) / 1e6))
            .collect::<Vec<_>>())
        .collect::<Vec<_>>()
    ); // d3-shape/test/offset/wiggle-test.js:45:3
}

// d3-shape/test/appearance-test.js: stackOrderAppearance(series) returns an order by appearance
#[test]
fn stackorderappearance_series_returns_an_order_by_appearance() {
    assert_d3_eq(
        &shape::stack::Order::Appearance.apply(&[
            vec![[0.0, 0.0], [0.0, 0.0], [0.0, 1.0]],
            vec![[0.0, 3.0], [0.0, 2.0], [0.0, 0.0]],
            vec![[0.0, 0.0], [0.0, 4.0], [0.0, 0.0]],
        ]),
        &vec![1.0, 2.0, 0.0],
    ); // d3-shape/test/order/appearance-test.js:5:3
}

// d3-shape/test/appearance-test.js: stackOrderAppearance(series) treats NaN values as zero
#[test]
fn stackorderappearance_series_treats_nan_values_as_zero() {
    assert_d3_eq(
        &shape::stack::Order::Appearance.apply(&[
            vec![[0.0, f64::NAN], [0.0, f64::NAN], [0.0, 1.0]],
            vec![[0.0, 3.0], [0.0, 2.0], [0.0, f64::NAN]],
            vec![[0.0, f64::NAN], [0.0, 4.0], [0.0, f64::NAN]],
        ]),
        &vec![1.0, 2.0, 0.0],
    ); // d3-shape/test/order/appearance-test.js:13:3
}

// d3-shape/test/ascending-test.js: stackOrderAscending(series) returns an order by sum
#[test]
fn stackorderascending_series_returns_an_order_by_sum() {
    assert_d3_eq(
        &shape::stack::Order::Ascending.apply(&[
            vec![[0.0, 1.0], [0.0, 2.0], [0.0, 3.0]],
            vec![[0.0, 2.0], [0.0, 3.0], [0.0, 4.0]],
            vec![[0.0, 0.0], [0.0, 1.0], [0.0, 2.0]],
        ]),
        &vec![2.0, 0.0, 1.0],
    ); // d3-shape/test/order/ascending-test.js:5:3
}

// d3-shape/test/ascending-test.js: stackOrderAscending(series) treats NaN values as zero
#[test]
fn stackorderascending_series_treats_nan_values_as_zero() {
    assert_d3_eq(
        &shape::stack::Order::Ascending.apply(&[
            vec![[0.0, 1.0], [0.0, 2.0], [0.0, f64::NAN], [0.0, 3.0]],
            vec![[0.0, 2.0], [0.0, 3.0], [0.0, f64::NAN], [0.0, 4.0]],
            vec![[0.0, 0.0], [0.0, 1.0], [0.0, f64::NAN], [0.0, 2.0]],
        ]),
        &vec![2.0, 0.0, 1.0],
    ); // d3-shape/test/order/ascending-test.js:13:3
}

// d3-shape/test/descending-test.js: stackOrderDescending(series) returns an order by sum
#[test]
fn stackorderdescending_series_returns_an_order_by_sum() {
    assert_d3_eq(
        &shape::stack::Order::Descending.apply(&[
            vec![[0.0, 1.0], [0.0, 2.0], [0.0, 3.0]],
            vec![[0.0, 2.0], [0.0, 3.0], [0.0, 4.0]],
            vec![[0.0, 0.0], [0.0, 1.0], [0.0, 2.0]],
        ]),
        &vec![1.0, 0.0, 2.0],
    ); // d3-shape/test/order/descending-test.js:5:3
}

// d3-shape/test/descending-test.js: stackOrderDescending(series) treats NaN values as zero
#[test]
fn stackorderdescending_series_treats_nan_values_as_zero() {
    assert_d3_eq(
        &shape::stack::Order::Descending.apply(&[
            vec![[0.0, 1.0], [0.0, 2.0], [0.0, 3.0], [0.0, f64::NAN]],
            vec![[0.0, 2.0], [0.0, 3.0], [0.0, 4.0], [0.0, f64::NAN]],
            vec![[0.0, 0.0], [0.0, 1.0], [0.0, 2.0], [0.0, f64::NAN]],
        ]),
        &vec![1.0, 0.0, 2.0],
    ); // d3-shape/test/order/descending-test.js:13:3
}

// d3-shape/test/insideOut-test.js: stackOrderInsideOut(series) returns an order by appearance
#[test]
fn stackorderinsideout_series_returns_an_order_by_appearance() {
    assert_d3_eq(
        &shape::stack::Order::InsideOut.apply(&[
            vec![
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 1.0],
            ],
            vec![
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 2.0],
                [0.0, 0.0],
            ],
            vec![
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 3.0],
                [0.0, 0.0],
                [0.0, 0.0],
            ],
            vec![
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 4.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
            ],
            vec![
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 5.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
            ],
            vec![
                [0.0, 0.0],
                [0.0, 6.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
            ],
            vec![
                [0.0, 7.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
            ],
        ]),
        &vec![2.0, 3.0, 6.0, 5.0, 4.0, 1.0, 0.0],
    ); // d3-shape/test/order/insideOut-test.js:5:3
}

// d3-shape/test/insideOut-test.js: stackOrderInsideOut(series) treats NaN values as zero
#[test]
fn stackorderinsideout_series_treats_nan_values_as_zero() {
    assert_d3_eq(
        &shape::stack::Order::InsideOut.apply(&[
            vec![
                [0.0, 0.0],
                [0.0, f64::NAN],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 1.0],
            ],
            vec![
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, f64::NAN],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 2.0],
                [0.0, 0.0],
            ],
            vec![
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 3.0],
                [0.0, 0.0],
                [0.0, 0.0],
            ],
            vec![
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 4.0],
                [0.0, f64::NAN],
                [0.0, 0.0],
                [0.0, 0.0],
            ],
            vec![
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 5.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, f64::NAN],
                [0.0, 0.0],
            ],
            vec![
                [0.0, f64::NAN],
                [0.0, 6.0],
                [0.0, 0.0],
                [0.0, f64::NAN],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
            ],
            vec![
                [0.0, 7.0],
                [0.0, f64::NAN],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
            ],
        ]),
        &vec![2.0, 3.0, 6.0, 5.0, 4.0, 1.0, 0.0],
    ); // d3-shape/test/order/insideOut-test.js:17:3
}

// d3-shape/test/none-test.js: stackOrderNone(series) returns [0, 1, … series.length - 1]
#[test]
fn stackordernone_series_returns_0_1_series_length_1() {
    assert_d3_eq(
        &shape::stack::Order::None.apply(&vec![Vec::new(); 4.0 as usize]),
        &vec![0.0, 1.0, 2.0, 3.0],
    ); // d3-shape/test/order/none-test.js:5:3
}

// d3-shape/test/reverse-test.js: stackOrderReverse(series) returns [series.length - 1, series.length - 2, … 0]
#[test]
fn stackorderreverse_series_returns_series_length_1_series_length_2_0() {
    assert_d3_eq(
        &shape::stack::Order::Reverse.apply(&vec![Vec::new(); 4.0 as usize]),
        &vec![3.0, 2.0, 1.0, 0.0],
    ); // d3-shape/test/order/reverse-test.js:5:3
}
