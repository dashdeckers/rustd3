//! d3's tests of d3-array, as assertions on rustd3-core; each names d3's file, line and column.

use rustd3_core::array;

use super::support::assert_d3_eq;

// d3-array/test/count-test.js: count() accepts an iterable
#[test]
fn count_accepts_an_iterable() {
    assert_eq!(array::count(vec![1.0, 2.0]) as f64, 2.0); // d3-array/test/count-test.js:5:3
}

// d3-array/test/count-test.js: count() ignores NaN, null
#[test]
fn count_ignores_nan_null() {
    assert_eq!(
        array::count(
            [Some(f64::NAN), Option::<f64>::None, Some(0.0), Some(1.0)]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ) as f64,
        2.0
    ); // d3-array/test/count-test.js:11:3
}

// d3-array/test/count-test.js: count() accepts an accessor
#[test]
fn count_accepts_an_accessor() {
    assert_eq!(
        array::count(
            [Some(f64::NAN), Option::<f64>::None, Some(0.0), Some(1.0)]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ) as f64,
        2.0
    ); // d3-array/test/count-test.js:19:3
    assert_eq!(
        array::count(
            [Some(f64::NAN), Some(18.0), Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ) as f64,
        1.0
    ); // d3-array/test/count-test.js:20:3
}

// d3-array/test/min-test.js: min(array) returns the least numeric value for numbers
#[test]
fn min_array_returns_the_least_numeric_value_for_numbers() {
    assert_eq!(array::min(vec![1.0]), Some(1.0)); // d3-array/test/min-test.js:5:3
    assert_eq!(array::min(vec![5.0, 1.0, 2.0, 3.0, 4.0]), Some(1.0)); // d3-array/test/min-test.js:6:3
    assert_eq!(array::min(vec![20.0, 3.0]), Some(3.0)); // d3-array/test/min-test.js:7:3
    assert_eq!(array::min(vec![3.0, 20.0]), Some(3.0)); // d3-array/test/min-test.js:8:3
}

// d3-array/test/min-test.js: min(array) ignores null, undefined and NaN
#[test]
fn min_array_ignores_null_undefined_and_nan() {
    assert_eq!(
        array::min(vec![f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0]),
        Some(1.0)
    ); // d3-array/test/min-test.js:19:3
    assert_eq!(
        array::min(vec![1.0, 2.0, 3.0, 4.0, 5.0, f64::NAN]),
        Some(1.0)
    ); // d3-array/test/min-test.js:21:3
    assert_eq!(
        array::min(
            [
                Some(10.0),
                Option::<f64>::None,
                Some(3.0),
                Option::<f64>::None,
                Some(5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        Some(3.0)
    ); // d3-array/test/min-test.js:23:3
    assert_eq!(
        array::min(
            [
                Some(-1.0),
                Option::<f64>::None,
                Some(-3.0),
                Option::<f64>::None,
                Some(-5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        Some(-5.0)
    ); // d3-array/test/min-test.js:24:3
}

// d3-array/test/min-test.js: min(array) returns undefined if the array contains no numbers
#[test]
fn min_array_returns_undefined_if_the_array_contains_no_numbers() {
    assert_eq!(array::min(Vec::<f64>::new()), None); // d3-array/test/min-test.js:35:3
    assert_eq!(
        array::min(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/min-test.js:36:3
    assert_eq!(
        array::min(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/min-test.js:37:3
    assert_eq!(array::min(vec![f64::NAN]), None); // d3-array/test/min-test.js:38:3
    assert_eq!(array::min(vec![f64::NAN, f64::NAN]), None); // d3-array/test/min-test.js:39:3
}

// d3-array/test/min-test.js: min(array, f) returns the least numeric value for numbers
#[test]
fn min_array_f_returns_the_least_numeric_value_for_numbers() {
    assert_eq!(array::min(vec![1.0]), Some(1.0)); // d3-array/test/min-test.js:43:3
    assert_eq!(array::min(vec![5.0, 1.0, 2.0, 3.0, 4.0]), Some(1.0)); // d3-array/test/min-test.js:44:3
    assert_eq!(array::min(vec![20.0, 3.0]), Some(3.0)); // d3-array/test/min-test.js:45:3
    assert_eq!(array::min(vec![3.0, 20.0]), Some(3.0)); // d3-array/test/min-test.js:46:3
}

// d3-array/test/min-test.js: min(array, f) ignores null, undefined and NaN
#[test]
fn min_array_f_ignores_null_undefined_and_nan() {
    assert_eq!(
        array::min(vec![f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0]),
        Some(1.0)
    ); // d3-array/test/min-test.js:57:3
    assert_eq!(
        array::min(vec![1.0, 2.0, 3.0, 4.0, 5.0, f64::NAN]),
        Some(1.0)
    ); // d3-array/test/min-test.js:59:3
    assert_eq!(
        array::min(
            [
                Some(10.0),
                Option::<f64>::None,
                Some(3.0),
                Option::<f64>::None,
                Some(5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        Some(3.0)
    ); // d3-array/test/min-test.js:61:3
    assert_eq!(
        array::min(
            [
                Some(-1.0),
                Option::<f64>::None,
                Some(-3.0),
                Option::<f64>::None,
                Some(-5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        Some(-5.0)
    ); // d3-array/test/min-test.js:62:3
}

// d3-array/test/min-test.js: min(array, f) returns undefined if the array contains no observed values
#[test]
fn min_array_f_returns_undefined_if_the_array_contains_no_observed_values() {
    assert_eq!(array::min(Vec::<f64>::new()), None); // d3-array/test/min-test.js:73:3
    assert_eq!(
        array::min(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/min-test.js:74:3
    assert_eq!(
        array::min(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/min-test.js:75:3
    assert_eq!(array::min(vec![f64::NAN]), None); // d3-array/test/min-test.js:76:3
    assert_eq!(array::min(vec![f64::NAN, f64::NAN]), None); // d3-array/test/min-test.js:77:3
}

// d3-array/test/max-test.js: max(array) returns the greatest numeric value for numbers
#[test]
fn max_array_returns_the_greatest_numeric_value_for_numbers() {
    assert_eq!(array::max(vec![1.0]), Some(1.0)); // d3-array/test/max-test.js:5:3
    assert_eq!(array::max(vec![5.0, 1.0, 2.0, 3.0, 4.0]), Some(5.0)); // d3-array/test/max-test.js:6:3
    assert_eq!(array::max(vec![20.0, 3.0]), Some(20.0)); // d3-array/test/max-test.js:7:3
    assert_eq!(array::max(vec![3.0, 20.0]), Some(20.0)); // d3-array/test/max-test.js:8:3
}

// d3-array/test/max-test.js: max(array) ignores null, undefined and NaN
#[test]
fn max_array_ignores_null_undefined_and_nan() {
    assert_eq!(
        array::max(vec![f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0]),
        Some(5.0)
    ); // d3-array/test/max-test.js:19:3
    assert_eq!(
        array::max(vec![1.0, 2.0, 3.0, 4.0, 5.0, f64::NAN]),
        Some(5.0)
    ); // d3-array/test/max-test.js:21:3
    assert_eq!(
        array::max(
            [
                Some(10.0),
                Option::<f64>::None,
                Some(3.0),
                Option::<f64>::None,
                Some(5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        Some(10.0)
    ); // d3-array/test/max-test.js:23:3
    assert_eq!(
        array::max(
            [
                Some(-1.0),
                Option::<f64>::None,
                Some(-3.0),
                Option::<f64>::None,
                Some(-5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        Some(-1.0)
    ); // d3-array/test/max-test.js:24:3
}

// d3-array/test/max-test.js: max(array) returns undefined if the array contains no numbers
#[test]
fn max_array_returns_undefined_if_the_array_contains_no_numbers() {
    assert_eq!(array::max(Vec::<f64>::new()), None); // d3-array/test/max-test.js:35:3
    assert_eq!(
        array::max(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/max-test.js:36:3
    assert_eq!(
        array::max(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/max-test.js:37:3
    assert_eq!(array::max(vec![f64::NAN]), None); // d3-array/test/max-test.js:38:3
    assert_eq!(array::max(vec![f64::NAN, f64::NAN]), None); // d3-array/test/max-test.js:39:3
}

// d3-array/test/max-test.js: max(array, f) returns the greatest numeric value for numbers
#[test]
fn max_array_f_returns_the_greatest_numeric_value_for_numbers() {
    assert_eq!(array::max(vec![1.0]), Some(1.0)); // d3-array/test/max-test.js:43:3
    assert_eq!(array::max(vec![5.0, 1.0, 2.0, 3.0, 4.0]), Some(5.0)); // d3-array/test/max-test.js:44:3
    assert_eq!(array::max(vec![20.0, 3.0]), Some(20.0)); // d3-array/test/max-test.js:45:3
    assert_eq!(array::max(vec![3.0, 20.0]), Some(20.0)); // d3-array/test/max-test.js:46:3
}

// d3-array/test/max-test.js: max(array, f) ignores null, undefined and NaN
#[test]
fn max_array_f_ignores_null_undefined_and_nan() {
    assert_eq!(
        array::max(vec![f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0]),
        Some(5.0)
    ); // d3-array/test/max-test.js:57:3
    assert_eq!(
        array::max(vec![1.0, 2.0, 3.0, 4.0, 5.0, f64::NAN]),
        Some(5.0)
    ); // d3-array/test/max-test.js:59:3
    assert_eq!(
        array::max(
            [
                Some(10.0),
                Option::<f64>::None,
                Some(3.0),
                Option::<f64>::None,
                Some(5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        Some(10.0)
    ); // d3-array/test/max-test.js:61:3
    assert_eq!(
        array::max(
            [
                Some(-1.0),
                Option::<f64>::None,
                Some(-3.0),
                Option::<f64>::None,
                Some(-5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        Some(-1.0)
    ); // d3-array/test/max-test.js:62:3
}

// d3-array/test/max-test.js: max(array, f) returns undefined if the array contains no observed values
#[test]
fn max_array_f_returns_undefined_if_the_array_contains_no_observed_values() {
    assert_eq!(array::max(Vec::<f64>::new()), None); // d3-array/test/max-test.js:73:3
    assert_eq!(
        array::max(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/max-test.js:74:3
    assert_eq!(
        array::max(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/max-test.js:75:3
    assert_eq!(array::max(vec![f64::NAN]), None); // d3-array/test/max-test.js:76:3
    assert_eq!(array::max(vec![f64::NAN, f64::NAN]), None); // d3-array/test/max-test.js:77:3
}

// d3-array/test/extent-test.js: extent(array) returns the least and greatest numeric values for numbers
#[test]
fn extent_array_returns_the_least_and_greatest_numeric_values_for_numbers() {
    assert_d3_eq(
        &{
            let e_ = array::extent(vec![1.0]);
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        &vec![1.0, 1.0],
    ); // d3-array/test/extent-test.js:5:3
    assert_d3_eq(
        &{
            let e_ = array::extent(vec![5.0, 1.0, 2.0, 3.0, 4.0]);
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        &vec![1.0, 5.0],
    ); // d3-array/test/extent-test.js:6:3
    assert_d3_eq(
        &{
            let e_ = array::extent(vec![20.0, 3.0]);
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        &vec![3.0, 20.0],
    ); // d3-array/test/extent-test.js:7:3
    assert_d3_eq(
        &{
            let e_ = array::extent(vec![3.0, 20.0]);
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        &vec![3.0, 20.0],
    ); // d3-array/test/extent-test.js:8:3
}

// d3-array/test/extent-test.js: extent(array) ignores null, undefined and NaN
#[test]
fn extent_array_ignores_null_undefined_and_nan() {
    assert_d3_eq(
        &{
            let e_ = array::extent(vec![f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0]);
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        &vec![1.0, 5.0],
    ); // d3-array/test/extent-test.js:19:3
    assert_d3_eq(
        &{
            let e_ = array::extent(vec![1.0, 2.0, 3.0, 4.0, 5.0, f64::NAN]);
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        &vec![1.0, 5.0],
    ); // d3-array/test/extent-test.js:21:3
    assert_d3_eq(
        &{
            let e_ = array::extent(
                [
                    Some(10.0),
                    Option::<f64>::None,
                    Some(3.0),
                    Option::<f64>::None,
                    Some(5.0),
                    Some(f64::NAN),
                ]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>(),
            );
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        &vec![3.0, 10.0],
    ); // d3-array/test/extent-test.js:23:3
    assert_d3_eq(
        &{
            let e_ = array::extent(
                [
                    Some(-1.0),
                    Option::<f64>::None,
                    Some(-3.0),
                    Option::<f64>::None,
                    Some(-5.0),
                    Some(f64::NAN),
                ]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>(),
            );
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        &vec![-5.0, -1.0],
    ); // d3-array/test/extent-test.js:24:3
}

// d3-array/test/extent-test.js: extent(array) returns undefined if the array contains no numbers
#[test]
fn extent_array_returns_undefined_if_the_array_contains_no_numbers() {
    assert_eq!(
        {
            let e_ = array::extent(Vec::<f64>::new());
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        [Option::<f64>::None, Option::<f64>::None]
    ); // d3-array/test/extent-test.js:35:3
    assert_eq!(
        {
            let e_ = array::extent(
                [Option::<f64>::None]
                    .iter()
                    .map(|v| v.unwrap_or(f64::NAN))
                    .collect::<Vec<f64>>(),
            );
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        [Option::<f64>::None, Option::<f64>::None]
    ); // d3-array/test/extent-test.js:36:3
    assert_eq!(
        {
            let e_ = array::extent(
                [Option::<f64>::None]
                    .iter()
                    .map(|v| v.unwrap_or(f64::NAN))
                    .collect::<Vec<f64>>(),
            );
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        [Option::<f64>::None, Option::<f64>::None]
    ); // d3-array/test/extent-test.js:37:3
    assert_eq!(
        {
            let e_ = array::extent(vec![f64::NAN]);
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        [Option::<f64>::None, Option::<f64>::None]
    ); // d3-array/test/extent-test.js:38:3
    assert_eq!(
        {
            let e_ = array::extent(vec![f64::NAN, f64::NAN]);
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        [Option::<f64>::None, Option::<f64>::None]
    ); // d3-array/test/extent-test.js:39:3
}

// d3-array/test/extent-test.js: extent(array, f) returns the least and greatest numeric value for numbers
#[test]
fn extent_array_f_returns_the_least_and_greatest_numeric_value_for_numbers() {
    assert_d3_eq(
        &{
            let e_ = array::extent(vec![1.0]);
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        &vec![1.0, 1.0],
    ); // d3-array/test/extent-test.js:43:3
    assert_d3_eq(
        &{
            let e_ = array::extent(vec![5.0, 1.0, 2.0, 3.0, 4.0]);
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        &vec![1.0, 5.0],
    ); // d3-array/test/extent-test.js:44:3
    assert_d3_eq(
        &{
            let e_ = array::extent(vec![20.0, 3.0]);
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        &vec![3.0, 20.0],
    ); // d3-array/test/extent-test.js:45:3
    assert_d3_eq(
        &{
            let e_ = array::extent(vec![3.0, 20.0]);
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        &vec![3.0, 20.0],
    ); // d3-array/test/extent-test.js:46:3
}

// d3-array/test/extent-test.js: extent(array, f) ignores null, undefined and NaN
#[test]
fn extent_array_f_ignores_null_undefined_and_nan() {
    assert_d3_eq(
        &{
            let e_ = array::extent(vec![f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0]);
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        &vec![1.0, 5.0],
    ); // d3-array/test/extent-test.js:57:3
    assert_d3_eq(
        &{
            let e_ = array::extent(vec![1.0, 2.0, 3.0, 4.0, 5.0, f64::NAN]);
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        &vec![1.0, 5.0],
    ); // d3-array/test/extent-test.js:59:3
    assert_d3_eq(
        &{
            let e_ = array::extent(
                [
                    Some(10.0),
                    Option::<f64>::None,
                    Some(3.0),
                    Option::<f64>::None,
                    Some(5.0),
                    Some(f64::NAN),
                ]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>(),
            );
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        &vec![3.0, 10.0],
    ); // d3-array/test/extent-test.js:61:3
    assert_d3_eq(
        &{
            let e_ = array::extent(
                [
                    Some(-1.0),
                    Option::<f64>::None,
                    Some(-3.0),
                    Option::<f64>::None,
                    Some(-5.0),
                    Some(f64::NAN),
                ]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>(),
            );
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        &vec![-5.0, -1.0],
    ); // d3-array/test/extent-test.js:62:3
}

// d3-array/test/extent-test.js: extent(array, f) returns undefined if the array contains no observed values
#[test]
fn extent_array_f_returns_undefined_if_the_array_contains_no_observed_values() {
    assert_eq!(
        {
            let e_ = array::extent(Vec::<f64>::new());
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        [Option::<f64>::None, Option::<f64>::None]
    ); // d3-array/test/extent-test.js:73:3
    assert_eq!(
        {
            let e_ = array::extent(
                [Option::<f64>::None]
                    .iter()
                    .map(|v| v.unwrap_or(f64::NAN))
                    .collect::<Vec<f64>>(),
            );
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        [Option::<f64>::None, Option::<f64>::None]
    ); // d3-array/test/extent-test.js:74:3
    assert_eq!(
        {
            let e_ = array::extent(
                [Option::<f64>::None]
                    .iter()
                    .map(|v| v.unwrap_or(f64::NAN))
                    .collect::<Vec<f64>>(),
            );
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        [Option::<f64>::None, Option::<f64>::None]
    ); // d3-array/test/extent-test.js:75:3
    assert_eq!(
        {
            let e_ = array::extent(vec![f64::NAN]);
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        [Option::<f64>::None, Option::<f64>::None]
    ); // d3-array/test/extent-test.js:76:3
    assert_eq!(
        {
            let e_ = array::extent(vec![f64::NAN, f64::NAN]);
            [e_.map(|e| e[0]), e_.map(|e| e[1])]
        },
        [Option::<f64>::None, Option::<f64>::None]
    ); // d3-array/test/extent-test.js:77:3
}

// d3-array/test/sum-test.js: sum(array) returns the sum of the specified numbers
#[test]
fn sum_array_returns_the_sum_of_the_specified_numbers() {
    assert_eq!(array::sum(vec![1.0]), 1.0); // d3-array/test/sum-test.js:5:3
    assert_eq!(array::sum(vec![5.0, 1.0, 2.0, 3.0, 4.0]), 15.0); // d3-array/test/sum-test.js:6:3
    assert_eq!(array::sum(vec![20.0, 3.0]), 23.0); // d3-array/test/sum-test.js:7:3
    assert_eq!(array::sum(vec![3.0, 20.0]), 23.0); // d3-array/test/sum-test.js:8:3
}

// d3-array/test/sum-test.js: sum(array) ignores null, undefined and NaN
#[test]
fn sum_array_ignores_null_undefined_and_nan() {
    assert_eq!(array::sum(vec![f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0]), 15.0); // d3-array/test/sum-test.js:26:3
    assert_eq!(array::sum(vec![1.0, 2.0, 3.0, 4.0, 5.0, f64::NAN]), 15.0); // d3-array/test/sum-test.js:27:3
    assert_eq!(
        array::sum(
            [
                Some(10.0),
                Option::<f64>::None,
                Some(3.0),
                Option::<f64>::None,
                Some(5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        18.0
    ); // d3-array/test/sum-test.js:28:3
}

// d3-array/test/sum-test.js: sum(array) returns zero if there are no numbers
#[test]
fn sum_array_returns_zero_if_there_are_no_numbers() {
    assert_eq!(array::sum(Vec::<f64>::new()), 0.0); // d3-array/test/sum-test.js:32:3
    assert_eq!(array::sum(vec![f64::NAN]), 0.0); // d3-array/test/sum-test.js:33:3
    assert_eq!(
        array::sum(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        0.0
    ); // d3-array/test/sum-test.js:34:3
    assert_eq!(
        array::sum(
            [Option::<f64>::None, Some(f64::NAN)]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        0.0
    ); // d3-array/test/sum-test.js:35:3
}

// d3-array/test/sum-test.js: sum(array, f) returns the sum of the specified numbers
#[test]
fn sum_array_f_returns_the_sum_of_the_specified_numbers() {
    assert_eq!(array::sum(vec![1.0]), 1.0); // d3-array/test/sum-test.js:40:3
    assert_eq!(array::sum(vec![5.0, 1.0, 2.0, 3.0, 4.0]), 15.0); // d3-array/test/sum-test.js:41:3
    assert_eq!(array::sum(vec![20.0, 3.0]), 23.0); // d3-array/test/sum-test.js:42:3
    assert_eq!(array::sum(vec![3.0, 20.0]), 23.0); // d3-array/test/sum-test.js:43:3
}

// d3-array/test/sum-test.js: sum(array, f) ignores null, undefined and NaN
#[test]
fn sum_array_f_ignores_null_undefined_and_nan() {
    assert_eq!(array::sum(vec![f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0]), 15.0); // d3-array/test/sum-test.js:61:3
    assert_eq!(array::sum(vec![1.0, 2.0, 3.0, 4.0, 5.0, f64::NAN]), 15.0); // d3-array/test/sum-test.js:62:3
    assert_eq!(
        array::sum(
            [
                Some(10.0),
                Option::<f64>::None,
                Some(3.0),
                Option::<f64>::None,
                Some(5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        18.0
    ); // d3-array/test/sum-test.js:63:3
}

// d3-array/test/sum-test.js: sum(array, f) returns zero if there are no numbers
#[test]
fn sum_array_f_returns_zero_if_there_are_no_numbers() {
    assert_eq!(array::sum(Vec::<f64>::new()), 0.0); // d3-array/test/sum-test.js:67:3
    assert_eq!(array::sum(vec![f64::NAN]), 0.0); // d3-array/test/sum-test.js:68:3
    assert_eq!(
        array::sum(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        0.0
    ); // d3-array/test/sum-test.js:69:3
    assert_eq!(
        array::sum(
            [Option::<f64>::None, Some(f64::NAN)]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        0.0
    ); // d3-array/test/sum-test.js:70:3
}

// d3-array/test/mean-test.js: mean(array) returns the mean value for numbers
#[test]
fn mean_array_returns_the_mean_value_for_numbers() {
    assert_eq!(array::mean(vec![1.0]), Some(1.0)); // d3-array/test/mean-test.js:6:3
    assert_eq!(array::mean(vec![5.0, 1.0, 2.0, 3.0, 4.0]), Some(3.0)); // d3-array/test/mean-test.js:7:3
    assert_eq!(array::mean(vec![20.0, 3.0]), Some(11.5)); // d3-array/test/mean-test.js:8:3
    assert_eq!(array::mean(vec![3.0, 20.0]), Some(11.5)); // d3-array/test/mean-test.js:9:3
}

// d3-array/test/mean-test.js: mean(array) ignores null, undefined and NaN
#[test]
fn mean_array_ignores_null_undefined_and_nan() {
    assert_eq!(
        array::mean(vec![f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0]),
        Some(3.0)
    ); // d3-array/test/mean-test.js:13:3
    assert_eq!(
        array::mean(vec![1.0, 2.0, 3.0, 4.0, 5.0, f64::NAN]),
        Some(3.0)
    ); // d3-array/test/mean-test.js:14:3
    assert_eq!(
        array::mean(
            [
                Some(10.0),
                Option::<f64>::None,
                Some(3.0),
                Option::<f64>::None,
                Some(5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        Some(6.0)
    ); // d3-array/test/mean-test.js:15:3
}

// d3-array/test/mean-test.js: mean(array) returns undefined if the array contains no observed values
#[test]
fn mean_array_returns_undefined_if_the_array_contains_no_observed_values() {
    assert_eq!(array::mean(Vec::<f64>::new()), None); // d3-array/test/mean-test.js:19:3
    assert_eq!(
        array::mean(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/mean-test.js:20:3
    assert_eq!(
        array::mean(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/mean-test.js:21:3
    assert_eq!(array::mean(vec![f64::NAN]), None); // d3-array/test/mean-test.js:22:3
    assert_eq!(array::mean(vec![f64::NAN, f64::NAN]), None); // d3-array/test/mean-test.js:23:3
}

// d3-array/test/mean-test.js: mean(array, f) returns the mean value for numbers
#[test]
fn mean_array_f_returns_the_mean_value_for_numbers() {
    assert_eq!(array::mean(vec![1.0]), Some(1.0)); // d3-array/test/mean-test.js:40:3
    assert_eq!(array::mean(vec![5.0, 1.0, 2.0, 3.0, 4.0]), Some(3.0)); // d3-array/test/mean-test.js:41:3
    assert_eq!(array::mean(vec![20.0, 3.0]), Some(11.5)); // d3-array/test/mean-test.js:42:3
    assert_eq!(array::mean(vec![3.0, 20.0]), Some(11.5)); // d3-array/test/mean-test.js:43:3
}

// d3-array/test/mean-test.js: mean(array, f) ignores null, undefined and NaN
#[test]
fn mean_array_f_ignores_null_undefined_and_nan() {
    assert_eq!(
        array::mean(vec![f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0]),
        Some(3.0)
    ); // d3-array/test/mean-test.js:47:3
    assert_eq!(
        array::mean(vec![1.0, 2.0, 3.0, 4.0, 5.0, f64::NAN]),
        Some(3.0)
    ); // d3-array/test/mean-test.js:48:3
    assert_eq!(
        array::mean(
            [
                Some(10.0),
                Option::<f64>::None,
                Some(3.0),
                Option::<f64>::None,
                Some(5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        Some(6.0)
    ); // d3-array/test/mean-test.js:49:3
}

// d3-array/test/mean-test.js: mean(array, f) returns undefined if the array contains no observed values
#[test]
fn mean_array_f_returns_undefined_if_the_array_contains_no_observed_values() {
    assert_eq!(array::mean(Vec::<f64>::new()), None); // d3-array/test/mean-test.js:53:3
    assert_eq!(
        array::mean(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/mean-test.js:54:3
    assert_eq!(
        array::mean(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/mean-test.js:55:3
    assert_eq!(array::mean(vec![f64::NAN]), None); // d3-array/test/mean-test.js:56:3
    assert_eq!(array::mean(vec![f64::NAN, f64::NAN]), None); // d3-array/test/mean-test.js:57:3
}

// d3-array/test/variance-test.js: variance(array) returns the variance of the specified numbers
#[test]
fn variance_array_returns_the_variance_of_the_specified_numbers() {
    assert_eq!(array::variance(vec![5.0, 1.0, 2.0, 3.0, 4.0]), Some(2.5)); // d3-array/test/variance-test.js:5:3
    assert_eq!(array::variance(vec![20.0, 3.0]), Some(144.5)); // d3-array/test/variance-test.js:6:3
    assert_eq!(array::variance(vec![3.0, 20.0]), Some(144.5)); // d3-array/test/variance-test.js:7:3
}

// d3-array/test/variance-test.js: variance(array) ignores null, undefined and NaN
#[test]
fn variance_array_ignores_null_undefined_and_nan() {
    assert_eq!(
        array::variance(vec![f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0]),
        Some(2.5)
    ); // d3-array/test/variance-test.js:11:3
    assert_eq!(
        array::variance(vec![1.0, 2.0, 3.0, 4.0, 5.0, f64::NAN]),
        Some(2.5)
    ); // d3-array/test/variance-test.js:12:3
    assert_eq!(
        array::variance(
            [
                Some(10.0),
                Option::<f64>::None,
                Some(3.0),
                Option::<f64>::None,
                Some(5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        Some(13.0)
    ); // d3-array/test/variance-test.js:13:3
}

// d3-array/test/variance-test.js: variance(array) can handle large numbers without overflowing
#[test]
fn variance_array_can_handle_large_numbers_without_overflowing() {
    assert_eq!(array::variance(vec![f64::MAX, f64::MAX]), Some(0.0)); // d3-array/test/variance-test.js:17:3
    assert_eq!(array::variance(vec![-f64::MAX, -f64::MAX]), Some(0.0)); // d3-array/test/variance-test.js:18:3
}

// d3-array/test/variance-test.js: variance(array) returns undefined if the array has fewer than two numbers
#[test]
fn variance_array_returns_undefined_if_the_array_has_fewer_than_two_numbers() {
    assert_eq!(array::variance(vec![1.0]), None); // d3-array/test/variance-test.js:22:3
    assert_eq!(array::variance(Vec::<f64>::new()), None); // d3-array/test/variance-test.js:23:3
    assert_eq!(
        array::variance(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/variance-test.js:24:3
    assert_eq!(
        array::variance(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/variance-test.js:25:3
    assert_eq!(array::variance(vec![f64::NAN]), None); // d3-array/test/variance-test.js:26:3
    assert_eq!(array::variance(vec![f64::NAN, f64::NAN]), None); // d3-array/test/variance-test.js:27:3
}

// d3-array/test/variance-test.js: variance(array, f) returns the variance of the specified numbers
#[test]
fn variance_array_f_returns_the_variance_of_the_specified_numbers() {
    assert_eq!(array::variance(vec![5.0, 1.0, 2.0, 3.0, 4.0]), Some(2.5)); // d3-array/test/variance-test.js:31:3
    assert_eq!(array::variance(vec![20.0, 3.0]), Some(144.5)); // d3-array/test/variance-test.js:32:3
    assert_eq!(array::variance(vec![3.0, 20.0]), Some(144.5)); // d3-array/test/variance-test.js:33:3
}

// d3-array/test/variance-test.js: variance(array, f) ignores null, undefined and NaN
#[test]
fn variance_array_f_ignores_null_undefined_and_nan() {
    assert_eq!(
        array::variance(vec![f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0]),
        Some(2.5)
    ); // d3-array/test/variance-test.js:37:3
    assert_eq!(
        array::variance(vec![1.0, 2.0, 3.0, 4.0, 5.0, f64::NAN]),
        Some(2.5)
    ); // d3-array/test/variance-test.js:38:3
    assert_eq!(
        array::variance(
            [
                Some(10.0),
                Option::<f64>::None,
                Some(3.0),
                Option::<f64>::None,
                Some(5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        Some(13.0)
    ); // d3-array/test/variance-test.js:39:3
}

// d3-array/test/variance-test.js: variance(array, f) can handle large numbers without overflowing
#[test]
fn variance_array_f_can_handle_large_numbers_without_overflowing() {
    assert_eq!(array::variance(vec![f64::MAX, f64::MAX]), Some(0.0)); // d3-array/test/variance-test.js:43:3
    assert_eq!(array::variance(vec![-f64::MAX, -f64::MAX]), Some(0.0)); // d3-array/test/variance-test.js:44:3
}

// d3-array/test/variance-test.js: variance(array, f) returns undefined if the array has fewer than two numbers
#[test]
fn variance_array_f_returns_undefined_if_the_array_has_fewer_than_two_numbers() {
    assert_eq!(array::variance(vec![1.0]), None); // d3-array/test/variance-test.js:48:3
    assert_eq!(array::variance(Vec::<f64>::new()), None); // d3-array/test/variance-test.js:49:3
    assert_eq!(
        array::variance(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/variance-test.js:50:3
    assert_eq!(
        array::variance(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/variance-test.js:51:3
    assert_eq!(array::variance(vec![f64::NAN]), None); // d3-array/test/variance-test.js:52:3
    assert_eq!(array::variance(vec![f64::NAN, f64::NAN]), None); // d3-array/test/variance-test.js:53:3
}

// d3-array/test/deviation-test.js: deviation(array) returns the standard deviation of the specified numbers
#[test]
fn deviation_array_returns_the_standard_deviation_of_the_specified_numbers() {
    assert_eq!(array::deviation(vec![1.0, 1.0, 1.0, 1.0, 1.0]), Some(0.0)); // d3-array/test/deviation-test.js:5:3
    assert_eq!(
        array::deviation(vec![5.0, 1.0, 2.0, 3.0, 4.0]),
        Some(f64::sqrt(2.5))
    ); // d3-array/test/deviation-test.js:6:3
    assert_eq!(array::deviation(vec![20.0, 3.0]), Some(f64::sqrt(144.5))); // d3-array/test/deviation-test.js:7:3
    assert_eq!(array::deviation(vec![3.0, 20.0]), Some(f64::sqrt(144.5))); // d3-array/test/deviation-test.js:8:3
}

// d3-array/test/deviation-test.js: deviation(array) ignores null, undefined and NaN
#[test]
fn deviation_array_ignores_null_undefined_and_nan() {
    assert_eq!(
        array::deviation(vec![f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0]),
        Some(f64::sqrt(2.5))
    ); // d3-array/test/deviation-test.js:12:3
    assert_eq!(
        array::deviation(vec![1.0, 2.0, 3.0, 4.0, 5.0, f64::NAN]),
        Some(f64::sqrt(2.5))
    ); // d3-array/test/deviation-test.js:13:3
    assert_eq!(
        array::deviation(
            [
                Some(10.0),
                Option::<f64>::None,
                Some(3.0),
                Option::<f64>::None,
                Some(5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        Some(f64::sqrt(13.0))
    ); // d3-array/test/deviation-test.js:14:3
}

// d3-array/test/deviation-test.js: deviation(array) can handle large numbers without overflowing
#[test]
fn deviation_array_can_handle_large_numbers_without_overflowing() {
    assert_eq!(array::deviation(vec![f64::MAX, f64::MAX]), Some(0.0)); // d3-array/test/deviation-test.js:18:3
    assert_eq!(array::deviation(vec![-f64::MAX, -f64::MAX]), Some(0.0)); // d3-array/test/deviation-test.js:19:3
}

// d3-array/test/deviation-test.js: deviation(array) returns undefined if the array has fewer than two numbers
#[test]
fn deviation_array_returns_undefined_if_the_array_has_fewer_than_two_numbers() {
    assert_eq!(array::deviation(vec![1.0]), None); // d3-array/test/deviation-test.js:23:3
    assert_eq!(array::deviation(Vec::<f64>::new()), None); // d3-array/test/deviation-test.js:24:3
    assert_eq!(
        array::deviation(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/deviation-test.js:25:3
    assert_eq!(
        array::deviation(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/deviation-test.js:26:3
    assert_eq!(array::deviation(vec![f64::NAN]), None); // d3-array/test/deviation-test.js:27:3
    assert_eq!(array::deviation(vec![f64::NAN, f64::NAN]), None); // d3-array/test/deviation-test.js:28:3
}

// d3-array/test/deviation-test.js: deviation(array, f) returns the deviation of the specified numbers
#[test]
fn deviation_array_f_returns_the_deviation_of_the_specified_numbers() {
    assert_eq!(
        array::deviation(vec![5.0, 1.0, 2.0, 3.0, 4.0]),
        Some(f64::sqrt(2.5))
    ); // d3-array/test/deviation-test.js:32:3
    assert_eq!(array::deviation(vec![20.0, 3.0]), Some(f64::sqrt(144.5))); // d3-array/test/deviation-test.js:33:3
    assert_eq!(array::deviation(vec![3.0, 20.0]), Some(f64::sqrt(144.5))); // d3-array/test/deviation-test.js:34:3
}

// d3-array/test/deviation-test.js: deviation(array, f) ignores null, undefined and NaN
#[test]
fn deviation_array_f_ignores_null_undefined_and_nan() {
    assert_eq!(
        array::deviation(vec![f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0]),
        Some(f64::sqrt(2.5))
    ); // d3-array/test/deviation-test.js:38:3
    assert_eq!(
        array::deviation(vec![1.0, 2.0, 3.0, 4.0, 5.0, f64::NAN]),
        Some(f64::sqrt(2.5))
    ); // d3-array/test/deviation-test.js:39:3
    assert_eq!(
        array::deviation(
            [
                Some(10.0),
                Option::<f64>::None,
                Some(3.0),
                Option::<f64>::None,
                Some(5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        Some(f64::sqrt(13.0))
    ); // d3-array/test/deviation-test.js:40:3
}

// d3-array/test/deviation-test.js: deviation(array, f) can handle large numbers without overflowing
#[test]
fn deviation_array_f_can_handle_large_numbers_without_overflowing() {
    assert_eq!(array::deviation(vec![f64::MAX, f64::MAX]), Some(0.0)); // d3-array/test/deviation-test.js:44:3
    assert_eq!(array::deviation(vec![-f64::MAX, -f64::MAX]), Some(0.0)); // d3-array/test/deviation-test.js:45:3
}

// d3-array/test/deviation-test.js: deviation(array, f) returns undefined if the array has fewer than two numbers
#[test]
fn deviation_array_f_returns_undefined_if_the_array_has_fewer_than_two_numbers() {
    assert_eq!(array::deviation(vec![1.0]), None); // d3-array/test/deviation-test.js:49:3
    assert_eq!(array::deviation(Vec::<f64>::new()), None); // d3-array/test/deviation-test.js:50:3
    assert_eq!(
        array::deviation(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/deviation-test.js:51:3
    assert_eq!(
        array::deviation(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/deviation-test.js:52:3
    assert_eq!(array::deviation(vec![f64::NAN]), None); // d3-array/test/deviation-test.js:53:3
    assert_eq!(array::deviation(vec![f64::NAN, f64::NAN]), None); // d3-array/test/deviation-test.js:54:3
}

// d3-array/test/median-test.js: median(array) returns the median value for numbers
#[test]
fn median_array_returns_the_median_value_for_numbers() {
    assert_eq!(array::median(vec![1.0]), Some(1.0)); // d3-array/test/median-test.js:6:3
    assert_eq!(array::median(vec![5.0, 1.0, 2.0, 3.0]), Some(2.5)); // d3-array/test/median-test.js:7:3
    assert_eq!(array::median(vec![5.0, 1.0, 2.0, 3.0, 4.0]), Some(3.0)); // d3-array/test/median-test.js:8:3
    assert_eq!(array::median(vec![20.0, 3.0]), Some(11.5)); // d3-array/test/median-test.js:9:3
    assert_eq!(array::median(vec![3.0, 20.0]), Some(11.5)); // d3-array/test/median-test.js:10:3
}

// d3-array/test/median-test.js: median(array) ignores null, undefined and NaN
#[test]
fn median_array_ignores_null_undefined_and_nan() {
    assert_eq!(
        array::median(vec![f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0]),
        Some(3.0)
    ); // d3-array/test/median-test.js:14:3
    assert_eq!(
        array::median(vec![1.0, 2.0, 3.0, 4.0, 5.0, f64::NAN]),
        Some(3.0)
    ); // d3-array/test/median-test.js:15:3
    assert_eq!(
        array::median(
            [
                Some(10.0),
                Option::<f64>::None,
                Some(3.0),
                Option::<f64>::None,
                Some(5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        Some(5.0)
    ); // d3-array/test/median-test.js:16:3
}

// d3-array/test/median-test.js: median(array) can handle large numbers without overflowing
#[test]
fn median_array_can_handle_large_numbers_without_overflowing() {
    assert_eq!(array::median(vec![f64::MAX, f64::MAX]), Some(f64::MAX)); // d3-array/test/median-test.js:20:3
    assert_eq!(array::median(vec![-f64::MAX, -f64::MAX]), Some(-f64::MAX)); // d3-array/test/median-test.js:21:3
}

// d3-array/test/median-test.js: median(array) returns undefined if the array contains no observed values
#[test]
fn median_array_returns_undefined_if_the_array_contains_no_observed_values() {
    assert_eq!(array::median(Vec::<f64>::new()), None); // d3-array/test/median-test.js:25:3
    assert_eq!(
        array::median(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/median-test.js:26:3
    assert_eq!(
        array::median(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/median-test.js:27:3
    assert_eq!(array::median(vec![f64::NAN]), None); // d3-array/test/median-test.js:28:3
    assert_eq!(array::median(vec![f64::NAN, f64::NAN]), None); // d3-array/test/median-test.js:29:3
}

// d3-array/test/median-test.js: median(array, f) returns the median value for numbers
#[test]
fn median_array_f_returns_the_median_value_for_numbers() {
    assert_eq!(array::median(vec![1.0]), Some(1.0)); // d3-array/test/median-test.js:48:3
    assert_eq!(array::median(vec![5.0, 1.0, 2.0, 3.0, 4.0]), Some(3.0)); // d3-array/test/median-test.js:49:3
    assert_eq!(array::median(vec![20.0, 3.0]), Some(11.5)); // d3-array/test/median-test.js:50:3
    assert_eq!(array::median(vec![3.0, 20.0]), Some(11.5)); // d3-array/test/median-test.js:51:3
}

// d3-array/test/median-test.js: median(array, f) ignores null, undefined and NaN
#[test]
fn median_array_f_ignores_null_undefined_and_nan() {
    assert_eq!(
        array::median(vec![f64::NAN, 1.0, 2.0, 3.0, 4.0, 5.0]),
        Some(3.0)
    ); // d3-array/test/median-test.js:55:3
    assert_eq!(
        array::median(vec![1.0, 2.0, 3.0, 4.0, 5.0, f64::NAN]),
        Some(3.0)
    ); // d3-array/test/median-test.js:56:3
    assert_eq!(
        array::median(
            [
                Some(10.0),
                Option::<f64>::None,
                Some(3.0),
                Option::<f64>::None,
                Some(5.0),
                Some(f64::NAN)
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>()
        ),
        Some(5.0)
    ); // d3-array/test/median-test.js:57:3
}

// d3-array/test/median-test.js: median(array, f) can handle large numbers without overflowing
#[test]
fn median_array_f_can_handle_large_numbers_without_overflowing() {
    assert_eq!(array::median(vec![f64::MAX, f64::MAX]), Some(f64::MAX)); // d3-array/test/median-test.js:61:3
    assert_eq!(array::median(vec![-f64::MAX, -f64::MAX]), Some(-f64::MAX)); // d3-array/test/median-test.js:62:3
}

// d3-array/test/median-test.js: median(array, f) returns undefined if the array contains no observed values
#[test]
fn median_array_f_returns_undefined_if_the_array_contains_no_observed_values() {
    assert_eq!(array::median(Vec::<f64>::new()), None); // d3-array/test/median-test.js:66:3
    assert_eq!(
        array::median(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/median-test.js:67:3
    assert_eq!(
        array::median(
            [Option::<f64>::None]
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>()
        ),
        None
    ); // d3-array/test/median-test.js:68:3
    assert_eq!(array::median(vec![f64::NAN]), None); // d3-array/test/median-test.js:69:3
    assert_eq!(array::median(vec![f64::NAN, f64::NAN]), None); // d3-array/test/median-test.js:70:3
}

// d3-array/test/quantile-test.js: quantileSorted(array, p) requires sorted numeric input, quantile doesn't
#[test]
fn quantilesorted_array_p_requires_sorted_numeric_input_quantile_doesn_t() {
    assert_eq!(
        array::quantile_sorted(&[1.0, 2.0, 3.0, 4.0], 0.0),
        Some(1.0)
    ); // d3-array/test/quantile-test.js:5:3
    assert_eq!(
        array::quantile_sorted(&[1.0, 2.0, 3.0, 4.0], 1.0),
        Some(4.0)
    ); // d3-array/test/quantile-test.js:6:3
    assert_eq!(
        array::quantile_sorted(&[4.0, 3.0, 2.0, 1.0], 0.0),
        Some(4.0)
    ); // d3-array/test/quantile-test.js:7:3
    assert_eq!(
        array::quantile_sorted(&[4.0, 3.0, 2.0, 1.0], 1.0),
        Some(1.0)
    ); // d3-array/test/quantile-test.js:8:3
    assert_eq!(array::quantile(vec![1.0, 2.0, 3.0, 4.0], 0.0), Some(1.0)); // d3-array/test/quantile-test.js:9:3
    assert_eq!(array::quantile(vec![1.0, 2.0, 3.0, 4.0], 1.0), Some(4.0)); // d3-array/test/quantile-test.js:10:3
    assert_eq!(array::quantile(vec![4.0, 3.0, 2.0, 1.0], 0.0), Some(1.0)); // d3-array/test/quantile-test.js:11:3
    assert_eq!(array::quantile(vec![4.0, 3.0, 2.0, 1.0], 1.0), Some(4.0)); // d3-array/test/quantile-test.js:12:3
}

// d3-array/test/quantile-test.js: quantile(array, p) uses the R-7 method
#[test]
fn quantile_array_p_uses_the_r_7_method() {
    let even = vec![3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0];
    assert_eq!(array::quantile(even.clone(), 0.0), Some(3.0)); // d3-array/test/quantile-test.js:21:3
    assert_eq!(array::quantile(even.clone(), 0.25), Some(7.25)); // d3-array/test/quantile-test.js:22:3
    assert_eq!(array::quantile(even.clone(), 0.5), Some(9.0)); // d3-array/test/quantile-test.js:23:3
    assert_eq!(array::quantile(even.clone(), 0.75), Some(14.5)); // d3-array/test/quantile-test.js:24:3
    assert_eq!(array::quantile(even.clone(), 1.0), Some(20.0)); // d3-array/test/quantile-test.js:25:3
    let odd = vec![3.0, 6.0, 7.0, 8.0, 8.0, 9.0, 10.0, 13.0, 15.0, 16.0, 20.0];
    assert_eq!(array::quantile(odd.clone(), 0.0), Some(3.0)); // d3-array/test/quantile-test.js:27:3
    assert_eq!(array::quantile(odd.clone(), 0.25), Some(7.5)); // d3-array/test/quantile-test.js:28:3
    assert_eq!(array::quantile(odd.clone(), 0.5), Some(9.0)); // d3-array/test/quantile-test.js:29:3
    assert_eq!(array::quantile(odd.clone(), 0.75), Some(14.0)); // d3-array/test/quantile-test.js:30:3
    assert_eq!(array::quantile(odd.clone(), 1.0), Some(20.0)); // d3-array/test/quantile-test.js:31:3
}

// d3-array/test/quantile-test.js: quantile(array, p) returns an exact value for integer p-values
#[test]
fn quantile_array_p_returns_an_exact_value_for_integer_p_values() {
    let data = vec![1.0, 2.0, 3.0, 4.0];
    assert_eq!(array::quantile(data.clone(), 1.0 / 3.0), Some(2.0)); // d3-array/test/quantile-test.js:47:3
    assert_eq!(array::quantile(data.clone(), 2.0 / 3.0), Some(3.0)); // d3-array/test/quantile-test.js:48:3
}

// d3-array/test/quantile-test.js: quantile(array, p) returns the expected value for integer or fractional p
#[test]
fn quantile_array_p_returns_the_expected_value_for_integer_or_fractional_p() {
    let data = vec![3.0, 1.0, 2.0, 4.0, 0.0];
    assert_eq!(array::quantile(data.clone(), 0.0 / 4.0), Some(0.0)); // d3-array/test/quantile-test.js:53:3
    assert_eq!(array::quantile(data.clone(), 0.1 / 4.0), Some(0.1)); // d3-array/test/quantile-test.js:54:3
    assert_eq!(array::quantile(data.clone(), 1.0 / 4.0), Some(1.0)); // d3-array/test/quantile-test.js:55:3
    assert_eq!(array::quantile(data.clone(), 1.5 / 4.0), Some(1.5)); // d3-array/test/quantile-test.js:56:3
    assert_eq!(array::quantile(data.clone(), 2.0 / 4.0), Some(2.0)); // d3-array/test/quantile-test.js:57:3
    assert_eq!(array::quantile(data.clone(), 2.5 / 4.0), Some(2.5)); // d3-array/test/quantile-test.js:58:3
    assert_eq!(array::quantile(data.clone(), 3.0 / 4.0), Some(3.0)); // d3-array/test/quantile-test.js:59:3
    assert_eq!(array::quantile(data.clone(), 3.2 / 4.0), Some(3.2)); // d3-array/test/quantile-test.js:60:3
    assert_eq!(array::quantile(data.clone(), 4.0 / 4.0), Some(4.0)); // d3-array/test/quantile-test.js:61:3
}

// d3-array/test/quantile-test.js: quantile(array, p) returns the first value for p = 0
#[test]
fn quantile_array_p_returns_the_first_value_for_p_0() {
    let data = vec![1.0, 2.0, 3.0, 4.0];
    assert_eq!(array::quantile(data.clone(), 0.0), Some(1.0)); // d3-array/test/quantile-test.js:66:3
}

// d3-array/test/quantile-test.js: quantile(array, p) returns the last value for p = 1
#[test]
fn quantile_array_p_returns_the_last_value_for_p_1() {
    let data = vec![1.0, 2.0, 3.0, 4.0];
    assert_eq!(array::quantile(data.clone(), 1.0), Some(4.0)); // d3-array/test/quantile-test.js:71:3
}

// d3-array/test/quantile-test.js: quantile(array, p) returns undefined if p is not a number
#[test]
fn quantile_array_p_returns_undefined_if_p_is_not_a_number() {
    assert_eq!(array::quantile(vec![1.0, 2.0, 3.0], f64::NAN), None); // d3-array/test/quantile-test.js:75:3
    assert_eq!(array::quantile(vec![1.0, 2.0, 3.0], f64::NAN), None); // d3-array/test/quantile-test.js:77:3
}

// d3-array/test/quantile-test.js: quantile(array, p, f) observes the specified accessor
#[test]
fn quantile_array_p_f_observes_the_specified_accessor() {
    assert_eq!(array::quantile(vec![1.0, 2.0, 3.0, 4.0], 0.5), Some(2.5)); // d3-array/test/quantile-test.js:82:3
    assert_eq!(array::quantile(vec![1.0, 2.0, 3.0, 4.0], 0.0), Some(1.0)); // d3-array/test/quantile-test.js:83:3
    assert_eq!(array::quantile(vec![1.0, 2.0, 3.0, 4.0], 1.0), Some(4.0)); // d3-array/test/quantile-test.js:84:3
    assert_eq!(array::quantile(vec![2.0], 0.0), Some(2.0)); // d3-array/test/quantile-test.js:85:3
    assert_eq!(array::quantile(vec![2.0], 0.5), Some(2.0)); // d3-array/test/quantile-test.js:86:3
    assert_eq!(array::quantile(vec![2.0], 1.0), Some(2.0)); // d3-array/test/quantile-test.js:87:3
    assert_eq!(array::quantile(Vec::<f64>::new(), 0.0), None); // d3-array/test/quantile-test.js:88:3
    assert_eq!(array::quantile(Vec::<f64>::new(), 0.5), None); // d3-array/test/quantile-test.js:89:3
    assert_eq!(array::quantile(Vec::<f64>::new(), 1.0), None); // d3-array/test/quantile-test.js:90:3
}

// d3-array/test/range-test.js: range(stop) returns [0, 1, 2, … stop - 1]
#[test]
fn range_stop_returns_0_1_2_stop_1() {
    assert_eq!(array::range(0.0, 5.0, 1.0), vec![0.0, 1.0, 2.0, 3.0, 4.0]); // d3-array/test/range-test.js:5:3
    assert_eq!(array::range(0.0, 2.01, 1.0), vec![0.0, 1.0, 2.0]); // d3-array/test/range-test.js:6:3
    assert_eq!(array::range(0.0, 1.0, 1.0), vec![0.0]); // d3-array/test/range-test.js:7:3
    assert_eq!(array::range(0.0, 0.5, 1.0), vec![0.0]); // d3-array/test/range-test.js:8:3
}

// d3-array/test/range-test.js: range(stop) returns an empty array if stop <= 0
#[test]
fn range_stop_returns_an_empty_array_if_stop_0() {
    assert_eq!(array::range(0.0, 0.0, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:12:3
    assert_eq!(array::range(0.0, -0.5, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:13:3
    assert_eq!(array::range(0.0, -1.0, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:14:3
}

// d3-array/test/range-test.js: range(stop) returns an empty array if stop is NaN
#[test]
fn range_stop_returns_an_empty_array_if_stop_is_nan() {
    assert_eq!(array::range(0.0, f64::NAN, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:18:3
    assert_eq!(array::range(0.0, f64::NAN, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:19:3
}

// d3-array/test/range-test.js: range(start, stop) returns [start, start + 1, … stop - 1]
#[test]
fn range_start_stop_returns_start_start_1_stop_1() {
    assert_eq!(array::range(0.0, 5.0, 1.0), vec![0.0, 1.0, 2.0, 3.0, 4.0]); // d3-array/test/range-test.js:23:3
    assert_eq!(array::range(2.0, 5.0, 1.0), vec![2.0, 3.0, 4.0]); // d3-array/test/range-test.js:24:3
    assert_eq!(array::range(2.5, 5.0, 1.0), vec![2.5, 3.5, 4.5]); // d3-array/test/range-test.js:25:3
    assert_eq!(array::range(-1.0, 3.0, 1.0), vec![-1.0, 0.0, 1.0, 2.0]); // d3-array/test/range-test.js:26:3
}

// d3-array/test/range-test.js: range(start, stop) returns an empty array if start or stop is NaN
#[test]
fn range_start_stop_returns_an_empty_array_if_start_or_stop_is_nan() {
    assert_eq!(array::range(0.0, f64::NAN, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:30:3
    assert_eq!(array::range(1.0, f64::NAN, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:31:3
    assert_eq!(array::range(-1.0, f64::NAN, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:32:3
    assert_eq!(array::range(0.0, f64::NAN, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:33:3
    assert_eq!(array::range(1.0, f64::NAN, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:34:3
    assert_eq!(array::range(-1.0, f64::NAN, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:35:3
    assert_eq!(array::range(f64::NAN, 0.0, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:36:3
    assert_eq!(array::range(f64::NAN, 1.0, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:37:3
    assert_eq!(array::range(f64::NAN, -1.0, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:38:3
    assert_eq!(array::range(f64::NAN, 0.0, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:39:3
    assert_eq!(array::range(f64::NAN, 1.0, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:40:3
    assert_eq!(array::range(f64::NAN, -1.0, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:41:3
    assert_eq!(array::range(f64::NAN, f64::NAN, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:42:3
    assert_eq!(array::range(f64::NAN, f64::NAN, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:43:3
}

// d3-array/test/range-test.js: range(start, stop) returns an empty array if start >= stop
#[test]
fn range_start_stop_returns_an_empty_array_if_start_stop() {
    assert_eq!(array::range(0.0, 0.0, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:47:3
    assert_eq!(array::range(5.0, 5.0, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:48:3
    assert_eq!(array::range(6.0, 5.0, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:49:3
    assert_eq!(array::range(10.0, 10.0, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:50:3
    assert_eq!(array::range(20.0, 10.0, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:51:3
}

// d3-array/test/range-test.js: range(start, stop, step) returns [start, start + step, start + 2 * step, … stop - step]
#[test]
fn range_start_stop_step_returns_start_start_step_start_2_step_stop_step() {
    assert_eq!(array::range(0.0, 5.0, 1.0), vec![0.0, 1.0, 2.0, 3.0, 4.0]); // d3-array/test/range-test.js:55:3
    assert_eq!(array::range(0.0, 5.0, 2.0), vec![0.0, 2.0, 4.0]); // d3-array/test/range-test.js:56:3
    assert_eq!(array::range(2.0, 5.0, 2.0), vec![2.0, 4.0]); // d3-array/test/range-test.js:57:3
    assert_eq!(array::range(-1.0, 3.0, 2.0), vec![-1.0, 1.0]); // d3-array/test/range-test.js:58:3
}

// d3-array/test/range-test.js: range(start, stop, step) allows a negative step
#[test]
fn range_start_stop_step_allows_a_negative_step() {
    assert_eq!(array::range(5.0, 0.0, -1.0), vec![5.0, 4.0, 3.0, 2.0, 1.0]); // d3-array/test/range-test.js:62:3
    assert_eq!(array::range(5.0, 0.0, -2.0), vec![5.0, 3.0, 1.0]); // d3-array/test/range-test.js:63:3
    assert_eq!(array::range(5.0, 2.0, -2.0), vec![5.0, 3.0]); // d3-array/test/range-test.js:64:3
    assert_eq!(array::range(3.0, -1.0, -2.0), vec![3.0, 1.0]); // d3-array/test/range-test.js:65:3
}

// d3-array/test/range-test.js: range(start, stop, step) returns an empty array if start >= stop and step > 0
#[test]
fn range_start_stop_step_returns_an_empty_array_if_start_stop_and_step_0() {
    assert_eq!(array::range(5.0, 5.0, 2.0), Vec::<f64>::new()); // d3-array/test/range-test.js:69:3
    assert_eq!(array::range(6.0, 5.0, 2.0), Vec::<f64>::new()); // d3-array/test/range-test.js:70:3
    assert_eq!(array::range(10.0, 10.0, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:71:3
    assert_eq!(array::range(10.0, 10.0, 0.5), Vec::<f64>::new()); // d3-array/test/range-test.js:72:3
    assert_eq!(array::range(0.0, 0.0, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:73:3
    assert_eq!(array::range(0.0, 0.0, 0.5), Vec::<f64>::new()); // d3-array/test/range-test.js:74:3
    assert_eq!(array::range(20.0, 10.0, 2.0), Vec::<f64>::new()); // d3-array/test/range-test.js:75:3
    assert_eq!(array::range(20.0, 10.0, 1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:76:3
    assert_eq!(array::range(20.0, 10.0, 0.5), Vec::<f64>::new()); // d3-array/test/range-test.js:77:3
}

// d3-array/test/range-test.js: range(start, stop, step) returns an empty array if start >= stop and step < 0
#[test]
fn range_start_stop_step_returns_an_empty_array_if_start_stop_and_step_0_2() {
    assert_eq!(array::range(5.0, 5.0, -2.0), Vec::<f64>::new()); // d3-array/test/range-test.js:81:3
    assert_eq!(array::range(5.0, 6.0, -2.0), Vec::<f64>::new()); // d3-array/test/range-test.js:82:3
    assert_eq!(array::range(10.0, 10.0, -1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:83:3
    assert_eq!(array::range(10.0, 10.0, -0.5), Vec::<f64>::new()); // d3-array/test/range-test.js:84:3
    assert_eq!(array::range(0.0, 0.0, -1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:85:3
    assert_eq!(array::range(0.0, 0.0, -0.5), Vec::<f64>::new()); // d3-array/test/range-test.js:86:3
    assert_eq!(array::range(10.0, 20.0, -2.0), Vec::<f64>::new()); // d3-array/test/range-test.js:87:3
    assert_eq!(array::range(10.0, 20.0, -1.0), Vec::<f64>::new()); // d3-array/test/range-test.js:88:3
    assert_eq!(array::range(10.0, 20.0, -0.5), Vec::<f64>::new()); // d3-array/test/range-test.js:89:3
}

// d3-array/test/range-test.js: range(start, stop, step) returns an empty array if start, stop or step is NaN
#[test]
fn range_start_stop_step_returns_an_empty_array_if_start_stop_or_step_is_nan() {
    assert_eq!(array::range(f64::NAN, 3.0, 2.0), Vec::<f64>::new()); // d3-array/test/range-test.js:93:3
    assert_eq!(array::range(3.0, f64::NAN, 2.0), Vec::<f64>::new()); // d3-array/test/range-test.js:94:3
    assert_eq!(array::range(0.0, 5.0, f64::NAN), Vec::<f64>::new()); // d3-array/test/range-test.js:95:3
    assert_eq!(
        array::range(f64::NAN, f64::NAN, f64::NAN),
        Vec::<f64>::new()
    ); // d3-array/test/range-test.js:96:3
    assert_eq!(
        array::range(f64::NAN, f64::NAN, f64::NAN),
        Vec::<f64>::new()
    ); // d3-array/test/range-test.js:97:3
    assert_eq!(
        array::range(f64::NAN, f64::NAN, f64::NAN),
        Vec::<f64>::new()
    ); // d3-array/test/range-test.js:98:3
    assert_eq!(array::range(0.0, 10.0, f64::NAN), Vec::<f64>::new()); // d3-array/test/range-test.js:99:3
    assert_eq!(array::range(10.0, 0.0, f64::NAN), Vec::<f64>::new()); // d3-array/test/range-test.js:100:3
    assert_eq!(array::range(0.0, 10.0, f64::NAN), Vec::<f64>::new()); // d3-array/test/range-test.js:101:3
    assert_eq!(array::range(10.0, 0.0, f64::NAN), Vec::<f64>::new()); // d3-array/test/range-test.js:102:3
}

// d3-array/test/range-test.js: range(start, stop, step) returns an empty array if step is zero
#[test]
fn range_start_stop_step_returns_an_empty_array_if_step_is_zero() {
    assert_eq!(array::range(0.0, 5.0, 0.0), Vec::<f64>::new()); // d3-array/test/range-test.js:106:3
}

// d3-array/test/range-test.js: range(start, stop, step) returns exactly [start + step * i, …] for fractional steps
#[test]
fn range_start_stop_step_returns_exactly_start_step_i_for_fractional_steps() {
    assert_eq!(
        array::range(0.0, 0.5, 0.1),
        vec![
            (0.0 + (0.1 * 0.0)),
            (0.0 + (0.1 * 1.0)),
            (0.0 + (0.1 * 2.0)),
            (0.0 + (0.1 * 3.0)),
            (0.0 + (0.1 * 4.0))
        ]
    ); // d3-array/test/range-test.js:110:3
    assert_eq!(
        array::range(0.5, 0.0, -0.1),
        vec![
            (0.5 - (0.1 * 0.0)),
            (0.5 - (0.1 * 1.0)),
            (0.5 - (0.1 * 2.0)),
            (0.5 - (0.1 * 3.0)),
            (0.5 - (0.1 * 4.0))
        ]
    ); // d3-array/test/range-test.js:111:3
    assert_eq!(
        array::range(-2.0, -1.2, 0.1),
        vec![
            (-2.0 + (0.1 * 0.0)),
            (-2.0 + (0.1 * 1.0)),
            (-2.0 + (0.1 * 2.0)),
            (-2.0 + (0.1 * 3.0)),
            (-2.0 + (0.1 * 4.0)),
            (-2.0 + (0.1 * 5.0)),
            (-2.0 + (0.1 * 6.0)),
            (-2.0 + (0.1 * 7.0))
        ]
    ); // d3-array/test/range-test.js:112:3
    assert_eq!(
        array::range(-1.2, -2.0, -0.1),
        vec![
            (-1.2 - (0.1 * 0.0)),
            (-1.2 - (0.1 * 1.0)),
            (-1.2 - (0.1 * 2.0)),
            (-1.2 - (0.1 * 3.0)),
            (-1.2 - (0.1 * 4.0)),
            (-1.2 - (0.1 * 5.0)),
            (-1.2 - (0.1 * 6.0)),
            (-1.2 - (0.1 * 7.0))
        ]
    ); // d3-array/test/range-test.js:113:3
}

// d3-array/test/range-test.js: range(start, stop, step) returns exactly [start + step * i, …] for very small fractional steps
#[test]
fn range_start_stop_step_returns_exactly_start_step_i_for_very_small_fractional_steps() {
    assert_eq!(
        array::range(2.1e-31, 5e-31, 1.1e-31),
        vec![
            (2.1e-31 + (1.1e-31 * 0.0)),
            (2.1e-31 + (1.1e-31 * 1.0)),
            (2.1e-31 + (1.1e-31 * 2.0))
        ]
    ); // d3-array/test/range-test.js:117:3
    assert_eq!(
        array::range(5e-31, 2.1e-31, -1.1e-31),
        vec![
            (5e-31 - (1.1e-31 * 0.0)),
            (5e-31 - (1.1e-31 * 1.0)),
            (5e-31 - (1.1e-31 * 2.0))
        ]
    ); // d3-array/test/range-test.js:118:3
}

// d3-array/test/range-test.js: range(start, stop, step) returns exactly [start + step * i, …] for very large fractional steps
#[test]
fn range_start_stop_step_returns_exactly_start_step_i_for_very_large_fractional_steps() {
    assert_eq!(
        array::range(1e300, 2e300, 0.3e300),
        vec![
            (1e300 + (0.3e300 * 0.0)),
            (1e300 + (0.3e300 * 1.0)),
            (1e300 + (0.3e300 * 2.0)),
            (1e300 + (0.3e300 * 3.0))
        ]
    ); // d3-array/test/range-test.js:122:3
    assert_eq!(
        array::range(2e300, 1e300, -0.3e300),
        vec![
            (2e300 - (0.3e300 * 0.0)),
            (2e300 - (0.3e300 * 1.0)),
            (2e300 - (0.3e300 * 2.0)),
            (2e300 - (0.3e300 * 3.0))
        ]
    ); // d3-array/test/range-test.js:123:3
}

// d3-array/test/bisect-test.js: bisectLeft(array, value) returns the index of an exact match
#[test]
fn bisectleft_array_value_returns_the_index_of_an_exact_match() {
    let numbers = vec![1.0, 2.0, 3.0];
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 1.0)) as f64
        },
        0.0
    ); // d3-array/test/bisect-test.js:10:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 2.0)) as f64
        },
        1.0
    ); // d3-array/test/bisect-test.js:11:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 3.0)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:12:3
}

// d3-array/test/bisect-test.js: bisectLeft(array, value) returns the index of the first match
#[test]
fn bisectleft_array_value_returns_the_index_of_the_first_match() {
    let numbers = vec![1.0, 2.0, 2.0, 3.0];
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 1.0)) as f64
        },
        0.0
    ); // d3-array/test/bisect-test.js:17:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 2.0)) as f64
        },
        1.0
    ); // d3-array/test/bisect-test.js:18:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 3.0)) as f64
        },
        3.0
    ); // d3-array/test/bisect-test.js:19:3
}

// d3-array/test/bisect-test.js: bisectLeft(empty, value) returns zero
#[test]
fn bisectleft_empty_value_returns_zero() {
    assert_eq!(
        {
            let v_ = Vec::<f64>::new();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 1.0)) as f64
        },
        0.0
    ); // d3-array/test/bisect-test.js:23:3
}

// d3-array/test/bisect-test.js: bisectLeft(array, value) returns the insertion point of a non-exact match
#[test]
fn bisectleft_array_value_returns_the_insertion_point_of_a_non_exact_match() {
    let numbers = vec![1.0, 2.0, 3.0];
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 0.5)) as f64
        },
        0.0
    ); // d3-array/test/bisect-test.js:28:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 1.5)) as f64
        },
        1.0
    ); // d3-array/test/bisect-test.js:29:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 2.5)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:30:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 3.5)) as f64
        },
        3.0
    ); // d3-array/test/bisect-test.js:31:3
}

// d3-array/test/bisect-test.js: bisectLeft(array, value, lo) observes the specified lower bound
#[test]
fn bisectleft_array_value_lo_observes_the_specified_lower_bound() {
    let numbers = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 0.0)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:43:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 1.0)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:44:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 2.0)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:45:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 3.0)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:46:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 4.0)) as f64
        },
        3.0
    ); // d3-array/test/bisect-test.js:47:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 5.0)) as f64
        },
        4.0
    ); // d3-array/test/bisect-test.js:48:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 6.0)) as f64
        },
        5.0
    ); // d3-array/test/bisect-test.js:49:3
}

// d3-array/test/bisect-test.js: bisectLeft(array, value, lo, hi) observes the specified bounds
#[test]
fn bisectleft_array_value_lo_hi_observes_the_specified_bounds() {
    let numbers = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_left(&v_[lo_..(3.0) as usize], 0.0)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:54:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_left(&v_[lo_..(3.0) as usize], 1.0)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:55:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_left(&v_[lo_..(3.0) as usize], 2.0)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:56:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_left(&v_[lo_..(3.0) as usize], 3.0)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:57:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_left(&v_[lo_..(3.0) as usize], 4.0)) as f64
        },
        3.0
    ); // d3-array/test/bisect-test.js:58:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_left(&v_[lo_..(3.0) as usize], 5.0)) as f64
        },
        3.0
    ); // d3-array/test/bisect-test.js:59:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_left(&v_[lo_..(3.0) as usize], 6.0)) as f64
        },
        3.0
    ); // d3-array/test/bisect-test.js:60:3
}

// d3-array/test/bisect-test.js: bisectRight(array, value) returns the index after an exact match
#[test]
fn bisectright_array_value_returns_the_index_after_an_exact_match() {
    let numbers = vec![1.0, 2.0, 3.0];
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 1.0)) as f64
        },
        1.0
    ); // d3-array/test/bisect-test.js:82:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 2.0)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:83:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 3.0)) as f64
        },
        3.0
    ); // d3-array/test/bisect-test.js:84:3
}

// d3-array/test/bisect-test.js: bisectRight(array, value) returns the index after the last match
#[test]
fn bisectright_array_value_returns_the_index_after_the_last_match() {
    let numbers = vec![1.0, 2.0, 2.0, 3.0];
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 1.0)) as f64
        },
        1.0
    ); // d3-array/test/bisect-test.js:89:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 2.0)) as f64
        },
        3.0
    ); // d3-array/test/bisect-test.js:90:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 3.0)) as f64
        },
        4.0
    ); // d3-array/test/bisect-test.js:91:3
}

// d3-array/test/bisect-test.js: bisectRight(empty, value) returns zero
#[test]
fn bisectright_empty_value_returns_zero() {
    assert_eq!(
        {
            let v_ = Vec::<f64>::new();
            let lo_ = 0;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 1.0)) as f64
        },
        0.0
    ); // d3-array/test/bisect-test.js:95:3
}

// d3-array/test/bisect-test.js: bisectRight(array, value) returns the insertion point of a non-exact match
#[test]
fn bisectright_array_value_returns_the_insertion_point_of_a_non_exact_match() {
    let numbers = vec![1.0, 2.0, 3.0];
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 0.5)) as f64
        },
        0.0
    ); // d3-array/test/bisect-test.js:100:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 1.5)) as f64
        },
        1.0
    ); // d3-array/test/bisect-test.js:101:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 2.5)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:102:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = 0;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 3.5)) as f64
        },
        3.0
    ); // d3-array/test/bisect-test.js:103:3
}

// d3-array/test/bisect-test.js: bisectRight(array, value, lo) observes the specified lower bound
#[test]
fn bisectright_array_value_lo_observes_the_specified_lower_bound() {
    let numbers = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 0.0)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:108:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 1.0)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:109:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 2.0)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:110:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 3.0)) as f64
        },
        3.0
    ); // d3-array/test/bisect-test.js:111:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 4.0)) as f64
        },
        4.0
    ); // d3-array/test/bisect-test.js:112:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 5.0)) as f64
        },
        5.0
    ); // d3-array/test/bisect-test.js:113:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 6.0)) as f64
        },
        5.0
    ); // d3-array/test/bisect-test.js:114:3
}

// d3-array/test/bisect-test.js: bisectRight(array, value, lo, hi) observes the specified bounds
#[test]
fn bisectright_array_value_lo_hi_observes_the_specified_bounds() {
    let numbers = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_right(&v_[lo_..(3.0) as usize], 0.0)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:119:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_right(&v_[lo_..(3.0) as usize], 1.0)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:120:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_right(&v_[lo_..(3.0) as usize], 2.0)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:121:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_right(&v_[lo_..(3.0) as usize], 3.0)) as f64
        },
        3.0
    ); // d3-array/test/bisect-test.js:122:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_right(&v_[lo_..(3.0) as usize], 4.0)) as f64
        },
        3.0
    ); // d3-array/test/bisect-test.js:123:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_right(&v_[lo_..(3.0) as usize], 5.0)) as f64
        },
        3.0
    ); // d3-array/test/bisect-test.js:124:3
    assert_eq!(
        {
            let v_ = numbers.clone();
            let lo_ = (2.0) as usize;
            (lo_ + array::bisect_right(&v_[lo_..(3.0) as usize], 6.0)) as f64
        },
        3.0
    ); // d3-array/test/bisect-test.js:125:3
}

// d3-array/test/bisect-test.js: bisectLeft(array, value, lo, hi) keeps non-comparable values to the right
#[test]
fn bisectleft_array_value_lo_hi_keeps_non_comparable_values_to_the_right() {
    let values = [
        Some(1.0),
        Some(2.0),
        Option::<f64>::None,
        Option::<f64>::None,
        Some(f64::NAN),
    ];
    assert_eq!(
        {
            let v_ = values
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 1.0)) as f64
        },
        0.0
    ); // d3-array/test/bisect-test.js:147:3
    assert_eq!(
        {
            let v_ = values
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 2.0)) as f64
        },
        1.0
    ); // d3-array/test/bisect-test.js:148:3
    assert_eq!(
        {
            let v_ = values
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], f64::NAN)) as f64
        },
        5.0
    ); // d3-array/test/bisect-test.js:149:3
    assert_eq!(
        {
            let v_ = values
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], f64::NAN)) as f64
        },
        5.0
    ); // d3-array/test/bisect-test.js:150:3
    assert_eq!(
        {
            let v_ = values
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], f64::NAN)) as f64
        },
        5.0
    ); // d3-array/test/bisect-test.js:151:3
}

// d3-array/test/bisect-test.js: bisectLeft(array, value, lo, hi) keeps comparable values to the left
#[test]
fn bisectleft_array_value_lo_hi_keeps_comparable_values_to_the_left() {
    let values = [Option::<f64>::None, Option::<f64>::None, Some(f64::NAN)];
    assert_eq!(
        {
            let v_ = values
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 1.0)) as f64
        },
        0.0
    ); // d3-array/test/bisect-test.js:156:3
    assert_eq!(
        {
            let v_ = values
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>();
            let lo_ = 0;
            (lo_ + array::bisect_left(&v_[lo_..v_.len()], 2.0)) as f64
        },
        0.0
    ); // d3-array/test/bisect-test.js:157:3
}

// d3-array/test/bisect-test.js: bisectRight(array, value, lo, hi) keeps non-comparable values to the right
#[test]
fn bisectright_array_value_lo_hi_keeps_non_comparable_values_to_the_right() {
    let values = [
        Some(1.0),
        Some(2.0),
        Option::<f64>::None,
        Option::<f64>::None,
    ];
    assert_eq!(
        {
            let v_ = values
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>();
            let lo_ = 0;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 1.0)) as f64
        },
        1.0
    ); // d3-array/test/bisect-test.js:162:3
    assert_eq!(
        {
            let v_ = values
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>();
            let lo_ = 0;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], 2.0)) as f64
        },
        2.0
    ); // d3-array/test/bisect-test.js:163:3
    assert_eq!(
        {
            let v_ = values
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>();
            let lo_ = 0;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], f64::NAN)) as f64
        },
        4.0
    ); // d3-array/test/bisect-test.js:164:3
    assert_eq!(
        {
            let v_ = values
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>();
            let lo_ = 0;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], f64::NAN)) as f64
        },
        4.0
    ); // d3-array/test/bisect-test.js:165:3
    assert_eq!(
        {
            let v_ = values
                .iter()
                .map(|v| v.unwrap_or(f64::NAN))
                .collect::<Vec<f64>>();
            let lo_ = 0;
            (lo_ + array::bisect_right(&v_[lo_..v_.len()], f64::NAN)) as f64
        },
        4.0
    ); // d3-array/test/bisect-test.js:166:3
}

// d3-array/test/bin-test.js: bin(data) computes bins of the specified array of data
#[test]
fn bin_data_computes_bins_of_the_specified_array_of_data() {
    let h = array::bin();
    assert_eq!(
        {
            let d_: Vec<f64> = vec![0.0, 0.0, 0.0, 10.0, 20.0, 20.0];
            h.layout(&d_)
                .iter()
                .map(|b| {
                    (
                        (b.x0, b.x1),
                        b.indices.iter().map(|&i| d_[i]).collect::<Vec<f64>>(),
                    )
                })
                .collect::<Vec<_>>()
        },
        vec![
            ((0.0, 5.0), vec![0.0, 0.0, 0.0]),
            ((5.0, 10.0), Vec::<f64>::new()),
            ((10.0, 15.0), vec![10.0]),
            ((15.0, 20.0), Vec::<f64>::new()),
            ((20.0, 25.0), vec![20.0, 20.0])
        ]
    ); // d3-array/test/bin-test.js:19:3
}

// d3-array/test/bin-test.js: bin(data) does not bin null, undefined, or NaN
#[test]
fn bin_data_does_not_bin_null_undefined_or_nan() {
    let h = array::bin();
    assert_eq!(
        {
            let d_: Vec<f64> = [
                Some(0.0),
                Option::<f64>::None,
                Option::<f64>::None,
                Some(f64::NAN),
                Some(10.0),
                Some(20.0),
                Some(20.0),
            ]
            .iter()
            .map(|v| v.unwrap_or(f64::NAN))
            .collect::<Vec<f64>>();
            h.layout(&d_)
                .iter()
                .map(|b| {
                    (
                        (b.x0, b.x1),
                        b.indices.iter().map(|&i| d_[i]).collect::<Vec<f64>>(),
                    )
                })
                .collect::<Vec<_>>()
        },
        vec![
            ((0.0, 5.0), vec![0.0]),
            ((5.0, 10.0), Vec::<f64>::new()),
            ((10.0, 15.0), vec![10.0]),
            ((15.0, 20.0), Vec::<f64>::new()),
            ((20.0, 25.0), vec![20.0, 20.0])
        ]
    ); // d3-array/test/bin-test.js:48:3
}

// d3-array/test/bin-test.js: bin.domain(array) sets the domain
#[test]
fn bin_domain_array_sets_the_domain() {
    let h = array::bin().domain([0.0, 20.0]);
    assert_eq!(
        {
            let d_: Vec<f64> = vec![1.0, 2.0, 2.0, 10.0, 18.0, 18.0];
            h.layout(&d_)
                .iter()
                .map(|b| {
                    (
                        (b.x0, b.x1),
                        b.indices.iter().map(|&i| d_[i]).collect::<Vec<f64>>(),
                    )
                })
                .collect::<Vec<_>>()
        },
        vec![
            ((0.0, 5.0), vec![1.0, 2.0, 2.0]),
            ((5.0, 10.0), Vec::<f64>::new()),
            ((10.0, 15.0), vec![10.0]),
            ((15.0, 20.0), vec![18.0, 18.0])
        ]
    ); // d3-array/test/bin-test.js:74:3
}

// d3-array/test/bin-test.js: bin.thresholds(number) sets the approximate number of bin thresholds
#[test]
fn bin_thresholds_number_sets_the_approximate_number_of_bin_thresholds() {
    let h = array::bin().thresholds(array::bin::Thresholds::Count((3.0) as usize));
    assert_eq!(
        {
            let d_: Vec<f64> = vec![0.0, 0.0, 0.0, 10.0, 30.0, 30.0];
            h.layout(&d_)
                .iter()
                .map(|b| {
                    (
                        (b.x0, b.x1),
                        b.indices.iter().map(|&i| d_[i]).collect::<Vec<f64>>(),
                    )
                })
                .collect::<Vec<_>>()
        },
        vec![
            ((0.0, 10.0), vec![0.0, 0.0, 0.0]),
            ((10.0, 20.0), vec![10.0]),
            ((20.0, 30.0), Vec::<f64>::new()),
            ((30.0, 40.0), vec![30.0, 30.0])
        ]
    ); // d3-array/test/bin-test.js:99:3
}

// d3-array/test/bin-test.js: bin.thresholds(array) sets the bin thresholds
#[test]
fn bin_thresholds_array_sets_the_bin_thresholds() {
    let h = array::bin().thresholds(array::bin::Thresholds::Values(vec![10.0, 20.0]));
    assert_eq!(
        {
            let d_: Vec<f64> = vec![0.0, 0.0, 0.0, 10.0, 30.0, 30.0];
            h.layout(&d_)
                .iter()
                .map(|b| {
                    (
                        (b.x0, b.x1),
                        b.indices.iter().map(|&i| d_[i]).collect::<Vec<f64>>(),
                    )
                })
                .collect::<Vec<_>>()
        },
        vec![
            ((0.0, 10.0), vec![0.0, 0.0, 0.0]),
            ((10.0, 20.0), vec![10.0]),
            ((20.0, 30.0), vec![30.0, 30.0])
        ]
    ); // d3-array/test/bin-test.js:109:3
}

// d3-array/test/bin-test.js: bin.thresholds(array) ignores thresholds outside the domain
#[test]
fn bin_thresholds_array_ignores_thresholds_outside_the_domain() {
    let h = array::bin().thresholds(array::bin::Thresholds::Values(vec![
        0.0, 1.0, 2.0, 3.0, 4.0,
    ]));
    assert_eq!(
        {
            let d_: Vec<f64> = vec![0.0, 1.0, 2.0, 3.0];
            h.layout(&d_)
                .iter()
                .map(|b| {
                    (
                        (b.x0, b.x1),
                        b.indices.iter().map(|&i| d_[i]).collect::<Vec<f64>>(),
                    )
                })
                .collect::<Vec<_>>()
        },
        vec![
            ((0.0, 1.0), vec![0.0]),
            ((1.0, 2.0), vec![1.0]),
            ((2.0, 3.0), vec![2.0]),
            ((3.0, 3.0), vec![3.0])
        ]
    ); // d3-array/test/bin-test.js:118:3
}

// d3-array/test/bin-test.js: bin(data) uses nice thresholds
#[test]
fn bin_data_uses_nice_thresholds() {
    let h = array::bin()
        .domain([0.0, 1.0])
        .thresholds(array::bin::Thresholds::Count((5.0) as usize));
    assert_eq!(
        {
            let d_: Vec<f64> = Vec::<f64>::new();
            h.layout(&d_)
                .iter()
                .map(|b| {
                    (
                        (b.x0, b.x1),
                        b.indices.iter().map(|&i| d_[i]).collect::<Vec<f64>>(),
                    )
                })
                .collect::<Vec<_>>()
        }
        .iter()
        .map(|b| vec![b.0.0, b.0.1])
        .collect::<Vec<_>>(),
        vec![
            vec![0.0, 0.2],
            vec![0.2, 0.4],
            vec![0.4, 0.6],
            vec![0.6, 0.8],
            vec![0.8, 1.0]
        ]
    ); // d3-array/test/bin-test.js:149:3
}

// d3-array/test/bin-test.js: bin()() returns bins whose rightmost bin is not too wide
#[test]
fn bin_returns_bins_whose_rightmost_bin_is_not_too_wide() {
    let h = array::bin();
    assert_eq!(
        {
            let d_: Vec<f64> = vec![9.8, 10.0, 11.0, 12.0, 13.0, 13.2];
            h.layout(&d_)
                .iter()
                .map(|b| {
                    (
                        (b.x0, b.x1),
                        b.indices.iter().map(|&i| d_[i]).collect::<Vec<f64>>(),
                    )
                })
                .collect::<Vec<_>>()
        },
        vec![
            ((9.0, 10.0), vec![9.8]),
            ((10.0, 11.0), vec![10.0]),
            ((11.0, 12.0), vec![11.0]),
            ((12.0, 13.0), vec![12.0]),
            ((13.0, 14.0), vec![13.0, 13.2])
        ]
    ); // d3-array/test/bin-test.js:160:3
}

// d3-array/test/bin-test.js: bin(data) handles fractional step correctly
#[test]
fn bin_data_handles_fractional_step_correctly() {
    let h = array::bin().thresholds(array::bin::Thresholds::Count((10.0) as usize));
    assert_eq!(
        {
            let d_: Vec<f64> = vec![9.8, 10.0, 11.0, 12.0, 13.0, 13.2];
            h.layout(&d_)
                .iter()
                .map(|b| {
                    (
                        (b.x0, b.x1),
                        b.indices.iter().map(|&i| d_[i]).collect::<Vec<f64>>(),
                    )
                })
                .collect::<Vec<_>>()
        },
        vec![
            ((9.5, 10.0), vec![9.8]),
            ((10.0, 10.5), vec![10.0]),
            ((10.5, 11.0), Vec::<f64>::new()),
            ((11.0, 11.5), vec![11.0]),
            ((11.5, 12.0), Vec::<f64>::new()),
            ((12.0, 12.5), vec![12.0]),
            ((12.5, 13.0), Vec::<f64>::new()),
            ((13.0, 13.5), vec![13.0, 13.2])
        ]
    ); // d3-array/test/bin-test.js:171:3
}

// d3-array/test/bin-test.js: bin(data) handles fractional step correctly with a custom, non-aligned domain
#[test]
fn bin_data_handles_fractional_step_correctly_with_a_custom_non_aligned_domain() {
    let h = array::bin()
        .thresholds(array::bin::Thresholds::Count((10.0) as usize))
        .domain([9.7, 13.3]);
    assert_eq!(
        {
            let d_: Vec<f64> = vec![9.8, 10.0, 11.0, 12.0, 13.0, 13.2];
            h.layout(&d_)
                .iter()
                .map(|b| {
                    (
                        (b.x0, b.x1),
                        b.indices.iter().map(|&i| d_[i]).collect::<Vec<f64>>(),
                    )
                })
                .collect::<Vec<_>>()
        },
        vec![
            ((9.7, 10.0), vec![9.8]),
            ((10.0, 10.5), vec![10.0]),
            ((10.5, 11.0), Vec::<f64>::new()),
            ((11.0, 11.5), vec![11.0]),
            ((11.5, 12.0), Vec::<f64>::new()),
            ((12.0, 12.5), vec![12.0]),
            ((12.5, 13.0), Vec::<f64>::new()),
            ((13.0, 13.3), vec![13.0, 13.2])
        ]
    ); // d3-array/test/bin-test.js:185:3
}

// d3-array/test/bin-test.js: bin(data) handles fractional step correctly with a custom, aligned domain
#[test]
fn bin_data_handles_fractional_step_correctly_with_a_custom_aligned_domain() {
    let h = array::bin()
        .thresholds(array::bin::Thresholds::Count((10.0) as usize))
        .domain([9.5, 13.5]);
    assert_eq!(
        {
            let d_: Vec<f64> = vec![9.8, 10.0, 11.0, 12.0, 13.0, 13.2];
            h.layout(&d_)
                .iter()
                .map(|b| {
                    (
                        (b.x0, b.x1),
                        b.indices.iter().map(|&i| d_[i]).collect::<Vec<f64>>(),
                    )
                })
                .collect::<Vec<_>>()
        },
        vec![
            ((9.5, 10.0), vec![9.8]),
            ((10.0, 10.5), vec![10.0]),
            ((10.5, 11.0), Vec::<f64>::new()),
            ((11.0, 11.5), vec![11.0]),
            ((11.5, 12.0), Vec::<f64>::new()),
            ((12.0, 12.5), vec![12.0]),
            ((12.5, 13.0), Vec::<f64>::new()),
            ((13.0, 13.5), vec![13.0, 13.2])
        ]
    ); // d3-array/test/bin-test.js:199:3
}

// d3-array/test/bin-test.js: bin(data) assigns integer values to the correct bins
#[test]
fn bin_data_assigns_integer_values_to_the_correct_bins() {
    assert_eq!(
        {
            let d_: Vec<f64> = vec![5.0];
            array::bin()
                .domain([4.0, 5.0])
                .layout(&d_)
                .iter()
                .map(|b| {
                    (
                        (b.x0, b.x1),
                        b.indices.iter().map(|&i| d_[i]).collect::<Vec<f64>>(),
                    )
                })
                .collect::<Vec<_>>()
        },
        vec![((4.0, 5.0), vec![5.0])]
    ); // d3-array/test/bin-test.js:239:3
    let eights = vec![
        8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0,
    ];
    assert_eq!(
        {
            let d_: Vec<f64> = eights.clone();
            array::bin()
                .domain([3.0, 8.0])
                .layout(&d_)
                .iter()
                .map(|b| {
                    (
                        (b.x0, b.x1),
                        b.indices.iter().map(|&i| d_[i]).collect::<Vec<f64>>(),
                    )
                })
                .collect::<Vec<_>>()
        },
        vec![
            ((3.0, 4.0), Vec::<f64>::new()),
            ((4.0, 5.0), Vec::<f64>::new()),
            ((5.0, 6.0), Vec::<f64>::new()),
            ((6.0, 7.0), Vec::<f64>::new()),
            ((7.0, 8.0), eights.clone())
        ]
    ); // d3-array/test/bin-test.js:241:3
}

// d3-array/test/bin-test.js: bin(data) does not mutate user-supplied thresholds as an array
#[test]
fn bin_data_does_not_mutate_user_supplied_thresholds_as_an_array() {
    let thresholds = vec![3.0, 4.0, 5.0, 6.0];
    let b = array::bin()
        .domain([4.0, 5.0])
        .thresholds(array::bin::Thresholds::Values(thresholds.clone()));
    assert_eq!(
        {
            let d_: Vec<f64> = vec![5.0];
            b.layout(&d_)
                .iter()
                .map(|b| {
                    (
                        (b.x0, b.x1),
                        b.indices.iter().map(|&i| d_[i]).collect::<Vec<f64>>(),
                    )
                })
                .collect::<Vec<_>>()
        },
        vec![((4.0, 5.0), Vec::<f64>::new()), ((5.0, 5.0), vec![5.0])]
    ); // d3-array/test/bin-test.js:247:3
    assert_eq!(thresholds, vec![3.0, 4.0, 5.0, 6.0]); // d3-array/test/bin-test.js:248:3
}

// d3-array/test/threshold/sturges-test.js: thresholdSturges(values, min, max) returns the expected result
#[test]
fn thresholdsturges_values_min_max_returns_the_expected_result() {
    assert_eq!(
        array::bin::sturges(&[4.0, 3.0, 2.0, 1.0, f64::NAN]) as f64,
        3.0
    ); // d3-array/test/threshold/sturges-test.js:5:3
    assert_eq!(array::bin::sturges(&[1.0]) as f64, 1.0); // d3-array/test/threshold/sturges-test.js:6:3
}

// d3-array/test/threshold/sturges-test.js: thresholdSturges(values, min, max) handles empty arrays
#[test]
fn thresholdsturges_values_min_max_handles_empty_arrays() {
    assert_eq!(array::bin::sturges(&Vec::<f64>::new()) as f64, 1.0); // d3-array/test/threshold/sturges-test.js:10:3
}

// d3-array/test/threshold/scott-test.js: thresholdScott(values, min, max) returns the expected result
#[test]
fn thresholdscott_values_min_max_returns_the_expected_result() {
    assert_eq!(
        array::bin::scott(&[4.0, 3.0, 2.0, 1.0, f64::NAN], 1.0, 4.0) as f64,
        2.0
    ); // d3-array/test/threshold/scott-test.js:5:3
}

// d3-array/test/threshold/scott-test.js: thresholdScott(values, min, max) handles values with zero deviation
#[test]
fn thresholdscott_values_min_max_handles_values_with_zero_deviation() {
    assert_eq!(
        array::bin::scott(&[1.0, 1.0, 1.0, 1.0], 1.0, 4.0) as f64,
        1.0
    ); // d3-array/test/threshold/scott-test.js:9:3
}

// d3-array/test/threshold/scott-test.js: thresholdScott(values, min, max) handles single-value arrays
#[test]
fn thresholdscott_values_min_max_handles_single_value_arrays() {
    assert_eq!(array::bin::scott(&[1.0], 1.0, 4.0) as f64, 1.0); // d3-array/test/threshold/scott-test.js:13:3
}

// d3-array/test/threshold/scott-test.js: thresholdScott(values, min, max) handles empty arrays
#[test]
fn thresholdscott_values_min_max_handles_empty_arrays() {
    assert_eq!(array::bin::scott(&Vec::<f64>::new(), 1.0, 4.0) as f64, 1.0); // d3-array/test/threshold/scott-test.js:17:3
}

// d3-array/test/threshold/freedmanDiaconic-test.js: thresholdFreedmanDiaconis(values, min, max) returns the expected result
#[test]
fn thresholdfreedmandiaconis_values_min_max_returns_the_expected_result() {
    assert_eq!(
        array::bin::freedman_diaconis(&[4.0, 3.0, 2.0, 1.0, f64::NAN], 1.0, 4.0) as f64,
        2.0
    ); // d3-array/test/threshold/freedmanDiaconic-test.js:5:3
}

// d3-array/test/threshold/freedmanDiaconic-test.js: thresholdFreedmanDiaconis(values, min, max) handles values with zero deviation
#[test]
fn thresholdfreedmandiaconis_values_min_max_handles_values_with_zero_deviation() {
    assert_eq!(
        array::bin::freedman_diaconis(&[1.0, 1.0, 1.0, 1.0], 1.0, 4.0) as f64,
        1.0
    ); // d3-array/test/threshold/freedmanDiaconic-test.js:9:3
}

// d3-array/test/threshold/freedmanDiaconic-test.js: thresholdFreedmanDiaconis(values, min, max) handles single-value arrays
#[test]
fn thresholdfreedmandiaconis_values_min_max_handles_single_value_arrays() {
    assert_eq!(array::bin::freedman_diaconis(&[1.0], 1.0, 4.0) as f64, 1.0); // d3-array/test/threshold/freedmanDiaconic-test.js:13:3
}

// d3-array/test/threshold/freedmanDiaconic-test.js: thresholdFreedmanDiaconis(values, min, max) handles empty arrays
#[test]
fn thresholdfreedmandiaconis_values_min_max_handles_empty_arrays() {
    assert_eq!(
        array::bin::freedman_diaconis(&Vec::<f64>::new(), 1.0, 4.0) as f64,
        1.0
    ); // d3-array/test/threshold/freedmanDiaconic-test.js:17:3
}
