//! d3's tests of d3-array, as assertions on rustd3-core; each names d3's file, line and column.

use rustd3_core::ticks;

use super::support::assert_d3_eq;

// d3-array/test/ticks-test.js: ticks(start, stop, count) returns the empty array if any argument is NaN
#[test]
fn ticks_start_stop_count_returns_the_empty_array_if_any_argument_is_nan() {
    assert_eq!(ticks::ticks(f64::NAN, 1.0, 1.0), Vec::<f64>::new()); // d3-array/test/ticks-test.js:5:3
    assert_eq!(ticks::ticks(0.0, f64::NAN, 1.0), Vec::<f64>::new()); // d3-array/test/ticks-test.js:6:3
    assert_eq!(ticks::ticks(0.0, 1.0, f64::NAN), Vec::<f64>::new()); // d3-array/test/ticks-test.js:7:3
    assert_eq!(ticks::ticks(f64::NAN, f64::NAN, 1.0), Vec::<f64>::new()); // d3-array/test/ticks-test.js:8:3
    assert_eq!(ticks::ticks(0.0, f64::NAN, f64::NAN), Vec::<f64>::new()); // d3-array/test/ticks-test.js:9:3
    assert_eq!(ticks::ticks(f64::NAN, 1.0, f64::NAN), Vec::<f64>::new()); // d3-array/test/ticks-test.js:10:3
    assert_eq!(
        ticks::ticks(f64::NAN, f64::NAN, f64::NAN),
        Vec::<f64>::new()
    ); // d3-array/test/ticks-test.js:11:3
}

// d3-array/test/ticks-test.js: ticks(start, stop, count) returns the empty array if start === stop and count is non-positive
#[test]
fn ticks_start_stop_count_returns_the_empty_array_if_start_stop_and_count_is_non_positive() {
    assert_eq!(ticks::ticks(1.0, 1.0, -1.0), Vec::<f64>::new()); // d3-array/test/ticks-test.js:15:3
    assert_eq!(ticks::ticks(1.0, 1.0, 0.0), Vec::<f64>::new()); // d3-array/test/ticks-test.js:16:3
    assert_eq!(ticks::ticks(1.0, 1.0, f64::NAN), Vec::<f64>::new()); // d3-array/test/ticks-test.js:17:3
}

// d3-array/test/ticks-test.js: ticks(start, stop, count) returns the empty array if start === stop and count is positive
#[test]
fn ticks_start_stop_count_returns_the_empty_array_if_start_stop_and_count_is_positive() {
    assert_eq!(ticks::ticks(1.0, 1.0, 1.0), vec![1.0]); // d3-array/test/ticks-test.js:21:3
    assert_eq!(ticks::ticks(1.0, 1.0, 10.0), vec![1.0]); // d3-array/test/ticks-test.js:22:3
}

// d3-array/test/ticks-test.js: ticks(start, stop, count) returns the empty array if count is not positive
#[test]
fn ticks_start_stop_count_returns_the_empty_array_if_count_is_not_positive() {
    assert_eq!(ticks::ticks(0.0, 1.0, 0.0), Vec::<f64>::new()); // d3-array/test/ticks-test.js:26:3
    assert_eq!(ticks::ticks(0.0, 1.0, -1.0), Vec::<f64>::new()); // d3-array/test/ticks-test.js:27:3
    assert_eq!(ticks::ticks(0.0, 1.0, f64::NAN), Vec::<f64>::new()); // d3-array/test/ticks-test.js:28:3
}

// d3-array/test/ticks-test.js: ticks(start, stop, count) returns the empty array if count is infinity
#[test]
fn ticks_start_stop_count_returns_the_empty_array_if_count_is_infinity() {
    assert_eq!(ticks::ticks(0.0, 1.0, f64::INFINITY), Vec::<f64>::new()); // d3-array/test/ticks-test.js:32:3
}

// d3-array/test/ticks-test.js: ticks(start, stop, count) does not include negative zero
#[test]
fn ticks_start_stop_count_does_not_include_negative_zero() {
    assert_eq!(
        1.0 / ticks::ticks(-1.0, 0.0, 5.0).pop().expect("d3 test pop"),
        f64::INFINITY
    ); // d3-array/test/ticks-test.js:36:3
}

// d3-array/test/ticks-test.js: ticks(start, stop, count) remains within the domain
#[test]
fn ticks_start_stop_count_remains_within_the_domain() {
    assert_eq!(ticks::ticks(0.0, 2.2, 3.0), vec![0.0, 1.0, 2.0]); // d3-array/test/ticks-test.js:40:3
}

// d3-array/test/ticks-test.js: ticks(start, stop, count) returns approximately count + 1 ticks when start < stop
#[test]
fn ticks_start_stop_count_returns_approximately_count_1_ticks_when_start_stop() {
    assert_eq!(
        ticks::ticks(0.0, 1.0, 10.0),
        vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    ); // d3-array/test/ticks-test.js:44:3
    assert_eq!(
        ticks::ticks(0.0, 1.0, 9.0),
        vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    ); // d3-array/test/ticks-test.js:45:3
    assert_eq!(
        ticks::ticks(0.0, 1.0, 8.0),
        vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    ); // d3-array/test/ticks-test.js:46:3
    assert_eq!(
        ticks::ticks(0.0, 1.0, 7.0),
        vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
    ); // d3-array/test/ticks-test.js:47:3
    assert_eq!(
        ticks::ticks(0.0, 1.0, 6.0),
        vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
    ); // d3-array/test/ticks-test.js:48:3
    assert_eq!(
        ticks::ticks(0.0, 1.0, 5.0),
        vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
    ); // d3-array/test/ticks-test.js:49:3
    assert_eq!(
        ticks::ticks(0.0, 1.0, 4.0),
        vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
    ); // d3-array/test/ticks-test.js:50:3
    assert_eq!(ticks::ticks(0.0, 1.0, 3.0), vec![0.0, 0.5, 1.0]); // d3-array/test/ticks-test.js:51:3
    assert_eq!(ticks::ticks(0.0, 1.0, 2.0), vec![0.0, 0.5, 1.0]); // d3-array/test/ticks-test.js:52:3
    assert_eq!(ticks::ticks(0.0, 1.0, 1.0), vec![0.0, 1.0]); // d3-array/test/ticks-test.js:53:3
    assert_eq!(
        ticks::ticks(0.0, 10.0, 10.0),
        vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
    ); // d3-array/test/ticks-test.js:54:3
    assert_eq!(
        ticks::ticks(0.0, 10.0, 9.0),
        vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
    ); // d3-array/test/ticks-test.js:55:3
    assert_eq!(
        ticks::ticks(0.0, 10.0, 8.0),
        vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
    ); // d3-array/test/ticks-test.js:56:3
    assert_eq!(
        ticks::ticks(0.0, 10.0, 7.0),
        vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]
    ); // d3-array/test/ticks-test.js:57:3
    assert_eq!(
        ticks::ticks(0.0, 10.0, 6.0),
        vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]
    ); // d3-array/test/ticks-test.js:58:3
    assert_eq!(
        ticks::ticks(0.0, 10.0, 5.0),
        vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]
    ); // d3-array/test/ticks-test.js:59:3
    assert_eq!(
        ticks::ticks(0.0, 10.0, 4.0),
        vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]
    ); // d3-array/test/ticks-test.js:60:3
    assert_eq!(ticks::ticks(0.0, 10.0, 3.0), vec![0.0, 5.0, 10.0]); // d3-array/test/ticks-test.js:61:3
    assert_eq!(ticks::ticks(0.0, 10.0, 2.0), vec![0.0, 5.0, 10.0]); // d3-array/test/ticks-test.js:62:3
    assert_eq!(ticks::ticks(0.0, 10.0, 1.0), vec![0.0, 10.0]); // d3-array/test/ticks-test.js:63:3
    assert_eq!(
        ticks::ticks(-10.0, 10.0, 10.0),
        vec![-10.0, -8.0, -6.0, -4.0, -2.0, 0.0, 2.0, 4.0, 6.0, 8.0, 10.0]
    ); // d3-array/test/ticks-test.js:64:3
    assert_eq!(
        ticks::ticks(-10.0, 10.0, 9.0),
        vec![-10.0, -8.0, -6.0, -4.0, -2.0, 0.0, 2.0, 4.0, 6.0, 8.0, 10.0]
    ); // d3-array/test/ticks-test.js:65:3
    assert_eq!(
        ticks::ticks(-10.0, 10.0, 8.0),
        vec![-10.0, -8.0, -6.0, -4.0, -2.0, 0.0, 2.0, 4.0, 6.0, 8.0, 10.0]
    ); // d3-array/test/ticks-test.js:66:3
    assert_eq!(
        ticks::ticks(-10.0, 10.0, 7.0),
        vec![-10.0, -8.0, -6.0, -4.0, -2.0, 0.0, 2.0, 4.0, 6.0, 8.0, 10.0]
    ); // d3-array/test/ticks-test.js:67:3
    assert_eq!(
        ticks::ticks(-10.0, 10.0, 6.0),
        vec![-10.0, -5.0, 0.0, 5.0, 10.0]
    ); // d3-array/test/ticks-test.js:68:3
    assert_eq!(
        ticks::ticks(-10.0, 10.0, 5.0),
        vec![-10.0, -5.0, 0.0, 5.0, 10.0]
    ); // d3-array/test/ticks-test.js:69:3
    assert_eq!(
        ticks::ticks(-10.0, 10.0, 4.0),
        vec![-10.0, -5.0, 0.0, 5.0, 10.0]
    ); // d3-array/test/ticks-test.js:70:3
    assert_eq!(
        ticks::ticks(-10.0, 10.0, 3.0),
        vec![-10.0, -5.0, 0.0, 5.0, 10.0]
    ); // d3-array/test/ticks-test.js:71:3
    assert_eq!(ticks::ticks(-10.0, 10.0, 2.0), vec![-10.0, 0.0, 10.0]); // d3-array/test/ticks-test.js:72:3
    assert_eq!(ticks::ticks(-10.0, 10.0, 1.0), vec![0.0]); // d3-array/test/ticks-test.js:73:3
}

// d3-array/test/ticks-test.js: ticks(start, stop, count) returns the reverse of ticks(stop, start, count)
#[test]
fn ticks_start_stop_count_returns_the_reverse_of_ticks_stop_start_count() {
    assert_eq!(ticks::ticks(1.0, 0.0, 10.0), {
        let mut v = ticks::ticks(0.0, 1.0, 10.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:77:3
    assert_eq!(ticks::ticks(1.0, 0.0, 9.0), {
        let mut v = ticks::ticks(0.0, 1.0, 9.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:78:3
    assert_eq!(ticks::ticks(1.0, 0.0, 8.0), {
        let mut v = ticks::ticks(0.0, 1.0, 8.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:79:3
    assert_eq!(ticks::ticks(1.0, 0.0, 7.0), {
        let mut v = ticks::ticks(0.0, 1.0, 7.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:80:3
    assert_eq!(ticks::ticks(1.0, 0.0, 6.0), {
        let mut v = ticks::ticks(0.0, 1.0, 6.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:81:3
    assert_eq!(ticks::ticks(1.0, 0.0, 5.0), {
        let mut v = ticks::ticks(0.0, 1.0, 5.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:82:3
    assert_eq!(ticks::ticks(1.0, 0.0, 4.0), {
        let mut v = ticks::ticks(0.0, 1.0, 4.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:83:3
    assert_eq!(ticks::ticks(1.0, 0.0, 3.0), {
        let mut v = ticks::ticks(0.0, 1.0, 3.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:84:3
    assert_eq!(ticks::ticks(1.0, 0.0, 2.0), {
        let mut v = ticks::ticks(0.0, 1.0, 2.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:85:3
    assert_eq!(ticks::ticks(1.0, 0.0, 1.0), {
        let mut v = ticks::ticks(0.0, 1.0, 1.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:86:3
    assert_eq!(ticks::ticks(10.0, 0.0, 10.0), {
        let mut v = ticks::ticks(0.0, 10.0, 10.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:87:3
    assert_eq!(ticks::ticks(10.0, 0.0, 9.0), {
        let mut v = ticks::ticks(0.0, 10.0, 9.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:88:3
    assert_eq!(ticks::ticks(10.0, 0.0, 8.0), {
        let mut v = ticks::ticks(0.0, 10.0, 8.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:89:3
    assert_eq!(ticks::ticks(10.0, 0.0, 7.0), {
        let mut v = ticks::ticks(0.0, 10.0, 7.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:90:3
    assert_eq!(ticks::ticks(10.0, 0.0, 6.0), {
        let mut v = ticks::ticks(0.0, 10.0, 6.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:91:3
    assert_eq!(ticks::ticks(10.0, 0.0, 5.0), {
        let mut v = ticks::ticks(0.0, 10.0, 5.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:92:3
    assert_eq!(ticks::ticks(10.0, 0.0, 4.0), {
        let mut v = ticks::ticks(0.0, 10.0, 4.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:93:3
    assert_eq!(ticks::ticks(10.0, 0.0, 3.0), {
        let mut v = ticks::ticks(0.0, 10.0, 3.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:94:3
    assert_eq!(ticks::ticks(10.0, 0.0, 2.0), {
        let mut v = ticks::ticks(0.0, 10.0, 2.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:95:3
    assert_eq!(ticks::ticks(10.0, 0.0, 1.0), {
        let mut v = ticks::ticks(0.0, 10.0, 1.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:96:3
    assert_eq!(ticks::ticks(10.0, -10.0, 10.0), {
        let mut v = ticks::ticks(-10.0, 10.0, 10.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:97:3
    assert_eq!(ticks::ticks(10.0, -10.0, 9.0), {
        let mut v = ticks::ticks(-10.0, 10.0, 9.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:98:3
    assert_eq!(ticks::ticks(10.0, -10.0, 8.0), {
        let mut v = ticks::ticks(-10.0, 10.0, 8.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:99:3
    assert_eq!(ticks::ticks(10.0, -10.0, 7.0), {
        let mut v = ticks::ticks(-10.0, 10.0, 7.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:100:3
    assert_eq!(ticks::ticks(10.0, -10.0, 6.0), {
        let mut v = ticks::ticks(-10.0, 10.0, 6.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:101:3
    assert_eq!(ticks::ticks(10.0, -10.0, 5.0), {
        let mut v = ticks::ticks(-10.0, 10.0, 5.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:102:3
    assert_eq!(ticks::ticks(10.0, -10.0, 4.0), {
        let mut v = ticks::ticks(-10.0, 10.0, 4.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:103:3
    assert_eq!(ticks::ticks(10.0, -10.0, 3.0), {
        let mut v = ticks::ticks(-10.0, 10.0, 3.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:104:3
    assert_eq!(ticks::ticks(10.0, -10.0, 2.0), {
        let mut v = ticks::ticks(-10.0, 10.0, 2.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:105:3
    assert_eq!(ticks::ticks(10.0, -10.0, 1.0), {
        let mut v = ticks::ticks(-10.0, 10.0, 1.0);
        v.reverse();
        v
    }); // d3-array/test/ticks-test.js:106:3
}

// d3-array/test/ticks-test.js: ticks(start, stop, count) handles precision problems
#[test]
fn ticks_start_stop_count_handles_precision_problems() {
    assert_eq!(
        ticks::ticks(0.98, 1.14, 10.0),
        vec![0.98, 1.0, 1.02, 1.04, 1.06, 1.08, 1.1, 1.12, 1.14]
    ); // d3-array/test/ticks-test.js:110:3
}

// d3-array/test/ticks-test.js: ticks(start, stop, count) tries to return at least one tick if count >= 0.5
#[test]
fn ticks_start_stop_count_tries_to_return_at_least_one_tick_if_count_0_5() {
    assert_eq!(ticks::ticks(1.0, 364.0, 0.1), Vec::<f64>::new()); // d3-array/test/ticks-test.js:114:3
    assert_eq!(ticks::ticks(1.0, 364.0, 0.499), Vec::<f64>::new()); // d3-array/test/ticks-test.js:115:3
    assert_eq!(ticks::ticks(1.0, 364.0, 0.5), vec![200.0]); // d3-array/test/ticks-test.js:116:3
    assert_eq!(ticks::ticks(1.0, 364.0, 1.0), vec![200.0]); // d3-array/test/ticks-test.js:117:3
    assert_eq!(ticks::ticks(1.0, 364.0, 1.5), vec![200.0]); // d3-array/test/ticks-test.js:118:3
    assert_eq!(ticks::ticks(1.0, 499.0, 1.0), vec![200.0, 400.0]); // d3-array/test/ticks-test.js:119:3
    assert_eq!(ticks::ticks(364.0, 1.0, 0.5), vec![200.0]); // d3-array/test/ticks-test.js:120:3
    assert_eq!(ticks::ticks(0.001, 0.364, 0.5), vec![0.2]); // d3-array/test/ticks-test.js:121:3
    assert_eq!(ticks::ticks(0.364, 0.001, 0.5), vec![0.2]); // d3-array/test/ticks-test.js:122:3
    assert_eq!(ticks::ticks(-1.0, -364.0, 0.5), vec![-200.0]); // d3-array/test/ticks-test.js:123:3
    assert_eq!(ticks::ticks(-364.0, -1.0, 0.5), vec![-200.0]); // d3-array/test/ticks-test.js:124:3
    assert_eq!(ticks::ticks(-0.001, -0.364, 0.5), vec![-0.2]); // d3-array/test/ticks-test.js:125:3
    assert_eq!(ticks::ticks(-0.364, -0.001, 0.5), vec![-0.2]); // d3-array/test/ticks-test.js:126:3
}

// d3-array/test/tickIncrement-test.js: tickIncrement(start, stop, count) returns NaN if any argument is NaN
#[test]
fn tickincrement_start_stop_count_returns_nan_if_any_argument_is_nan() {
    assert!((ticks::tick_increment(f64::NAN, 1.0, 1.0)).is_nan()); // d3-array/test/tickIncrement-test.js:5:3
    assert!((ticks::tick_increment(0.0, f64::NAN, 1.0)).is_nan()); // d3-array/test/tickIncrement-test.js:6:3
    assert!((ticks::tick_increment(0.0, 1.0, f64::NAN)).is_nan()); // d3-array/test/tickIncrement-test.js:7:3
    assert!((ticks::tick_increment(f64::NAN, f64::NAN, 1.0)).is_nan()); // d3-array/test/tickIncrement-test.js:8:3
    assert!((ticks::tick_increment(0.0, f64::NAN, f64::NAN)).is_nan()); // d3-array/test/tickIncrement-test.js:9:3
    assert!((ticks::tick_increment(f64::NAN, 1.0, f64::NAN)).is_nan()); // d3-array/test/tickIncrement-test.js:10:3
    assert!((ticks::tick_increment(f64::NAN, f64::NAN, f64::NAN)).is_nan()); // d3-array/test/tickIncrement-test.js:11:3
}

// d3-array/test/tickIncrement-test.js: tickIncrement(start, stop, count) returns NaN or -Infinity if start === stop
#[test]
fn tickincrement_start_stop_count_returns_nan_or_infinity_if_start_stop() {
    assert!((ticks::tick_increment(1.0, 1.0, -1.0)).is_nan()); // d3-array/test/tickIncrement-test.js:15:3
    assert!((ticks::tick_increment(1.0, 1.0, 0.0)).is_nan()); // d3-array/test/tickIncrement-test.js:16:3
    assert!((ticks::tick_increment(1.0, 1.0, f64::NAN)).is_nan()); // d3-array/test/tickIncrement-test.js:17:3
    assert_eq!(ticks::tick_increment(1.0, 1.0, 1.0), -f64::INFINITY); // d3-array/test/tickIncrement-test.js:18:3
    assert_eq!(ticks::tick_increment(1.0, 1.0, 10.0), -f64::INFINITY); // d3-array/test/tickIncrement-test.js:19:3
}

// d3-array/test/tickIncrement-test.js: tickIncrement(start, stop, count) returns 0 or Infinity if count is not positive
#[test]
fn tickincrement_start_stop_count_returns_0_or_infinity_if_count_is_not_positive() {
    assert_eq!(ticks::tick_increment(0.0, 1.0, -1.0), f64::INFINITY); // d3-array/test/tickIncrement-test.js:23:3
    assert_eq!(ticks::tick_increment(0.0, 1.0, 0.0), f64::INFINITY); // d3-array/test/tickIncrement-test.js:24:3
}

// d3-array/test/tickIncrement-test.js: tickIncrement(start, stop, count) returns -Infinity if count is infinity
#[test]
fn tickincrement_start_stop_count_returns_infinity_if_count_is_infinity() {
    assert_eq!(
        ticks::tick_increment(0.0, 1.0, f64::INFINITY),
        -f64::INFINITY
    ); // d3-array/test/tickIncrement-test.js:28:3
}

// d3-array/test/tickIncrement-test.js: tickIncrement(start, stop, count) returns approximately count + 1 tickIncrement when start < stop
#[test]
fn tickincrement_start_stop_count_returns_approximately_count_1_tickincrement_when_start_stop() {
    assert_eq!(ticks::tick_increment(0.0, 1.0, 10.0), -10.0); // d3-array/test/tickIncrement-test.js:32:3
    assert_eq!(ticks::tick_increment(0.0, 1.0, 9.0), -10.0); // d3-array/test/tickIncrement-test.js:33:3
    assert_eq!(ticks::tick_increment(0.0, 1.0, 8.0), -10.0); // d3-array/test/tickIncrement-test.js:34:3
    assert_eq!(ticks::tick_increment(0.0, 1.0, 7.0), -5.0); // d3-array/test/tickIncrement-test.js:35:3
    assert_eq!(ticks::tick_increment(0.0, 1.0, 6.0), -5.0); // d3-array/test/tickIncrement-test.js:36:3
    assert_eq!(ticks::tick_increment(0.0, 1.0, 5.0), -5.0); // d3-array/test/tickIncrement-test.js:37:3
    assert_eq!(ticks::tick_increment(0.0, 1.0, 4.0), -5.0); // d3-array/test/tickIncrement-test.js:38:3
    assert_eq!(ticks::tick_increment(0.0, 1.0, 3.0), -2.0); // d3-array/test/tickIncrement-test.js:39:3
    assert_eq!(ticks::tick_increment(0.0, 1.0, 2.0), -2.0); // d3-array/test/tickIncrement-test.js:40:3
    assert_eq!(ticks::tick_increment(0.0, 1.0, 1.0), 1.0); // d3-array/test/tickIncrement-test.js:41:3
    assert_eq!(ticks::tick_increment(0.0, 10.0, 10.0), 1.0); // d3-array/test/tickIncrement-test.js:42:3
    assert_eq!(ticks::tick_increment(0.0, 10.0, 9.0), 1.0); // d3-array/test/tickIncrement-test.js:43:3
    assert_eq!(ticks::tick_increment(0.0, 10.0, 8.0), 1.0); // d3-array/test/tickIncrement-test.js:44:3
    assert_eq!(ticks::tick_increment(0.0, 10.0, 7.0), 2.0); // d3-array/test/tickIncrement-test.js:45:3
    assert_eq!(ticks::tick_increment(0.0, 10.0, 6.0), 2.0); // d3-array/test/tickIncrement-test.js:46:3
    assert_eq!(ticks::tick_increment(0.0, 10.0, 5.0), 2.0); // d3-array/test/tickIncrement-test.js:47:3
    assert_eq!(ticks::tick_increment(0.0, 10.0, 4.0), 2.0); // d3-array/test/tickIncrement-test.js:48:3
    assert_eq!(ticks::tick_increment(0.0, 10.0, 3.0), 5.0); // d3-array/test/tickIncrement-test.js:49:3
    assert_eq!(ticks::tick_increment(0.0, 10.0, 2.0), 5.0); // d3-array/test/tickIncrement-test.js:50:3
    assert_eq!(ticks::tick_increment(0.0, 10.0, 1.0), 10.0); // d3-array/test/tickIncrement-test.js:51:3
    assert_eq!(ticks::tick_increment(-10.0, 10.0, 10.0), 2.0); // d3-array/test/tickIncrement-test.js:52:3
    assert_eq!(ticks::tick_increment(-10.0, 10.0, 9.0), 2.0); // d3-array/test/tickIncrement-test.js:53:3
    assert_eq!(ticks::tick_increment(-10.0, 10.0, 8.0), 2.0); // d3-array/test/tickIncrement-test.js:54:3
    assert_eq!(ticks::tick_increment(-10.0, 10.0, 7.0), 2.0); // d3-array/test/tickIncrement-test.js:55:3
    assert_eq!(ticks::tick_increment(-10.0, 10.0, 6.0), 5.0); // d3-array/test/tickIncrement-test.js:56:3
    assert_eq!(ticks::tick_increment(-10.0, 10.0, 5.0), 5.0); // d3-array/test/tickIncrement-test.js:57:3
    assert_eq!(ticks::tick_increment(-10.0, 10.0, 4.0), 5.0); // d3-array/test/tickIncrement-test.js:58:3
    assert_eq!(ticks::tick_increment(-10.0, 10.0, 3.0), 5.0); // d3-array/test/tickIncrement-test.js:59:3
    assert_eq!(ticks::tick_increment(-10.0, 10.0, 2.0), 10.0); // d3-array/test/tickIncrement-test.js:60:3
    assert_eq!(ticks::tick_increment(-10.0, 10.0, 1.0), 20.0); // d3-array/test/tickIncrement-test.js:61:3
}

// d3-array/test/tickStep-test.js: tickStep(start, stop, count) returns NaN if any argument is NaN
#[test]
fn tickstep_start_stop_count_returns_nan_if_any_argument_is_nan() {
    assert!((ticks::tick_step(f64::NAN, 1.0, 1.0)).is_nan()); // d3-array/test/tickStep-test.js:5:3
    assert!((ticks::tick_step(0.0, f64::NAN, 1.0)).is_nan()); // d3-array/test/tickStep-test.js:6:3
    assert!((ticks::tick_step(0.0, 1.0, f64::NAN)).is_nan()); // d3-array/test/tickStep-test.js:7:3
    assert!((ticks::tick_step(f64::NAN, f64::NAN, 1.0)).is_nan()); // d3-array/test/tickStep-test.js:8:3
    assert!((ticks::tick_step(0.0, f64::NAN, f64::NAN)).is_nan()); // d3-array/test/tickStep-test.js:9:3
    assert!((ticks::tick_step(f64::NAN, 1.0, f64::NAN)).is_nan()); // d3-array/test/tickStep-test.js:10:3
    assert!((ticks::tick_step(f64::NAN, f64::NAN, f64::NAN)).is_nan()); // d3-array/test/tickStep-test.js:11:3
}

// d3-array/test/tickStep-test.js: tickStep(start, stop, count) returns NaN or 0 if start === stop
#[test]
fn tickstep_start_stop_count_returns_nan_or_0_if_start_stop() {
    assert!((ticks::tick_step(1.0, 1.0, -1.0)).is_nan()); // d3-array/test/tickStep-test.js:15:3
    assert!((ticks::tick_step(1.0, 1.0, 0.0)).is_nan()); // d3-array/test/tickStep-test.js:16:3
    assert!((ticks::tick_step(1.0, 1.0, f64::NAN)).is_nan()); // d3-array/test/tickStep-test.js:17:3
    assert_eq!(ticks::tick_step(1.0, 1.0, 1.0), 0.0); // d3-array/test/tickStep-test.js:18:3
    assert_eq!(ticks::tick_step(1.0, 1.0, 10.0), 0.0); // d3-array/test/tickStep-test.js:19:3
}

// d3-array/test/tickStep-test.js: tickStep(start, stop, count) returns 0 or Infinity if count is not positive
#[test]
fn tickstep_start_stop_count_returns_0_or_infinity_if_count_is_not_positive() {
    assert_eq!(ticks::tick_step(0.0, 1.0, -1.0), f64::INFINITY); // d3-array/test/tickStep-test.js:23:3
    assert_eq!(ticks::tick_step(0.0, 1.0, 0.0), f64::INFINITY); // d3-array/test/tickStep-test.js:24:3
}

// d3-array/test/tickStep-test.js: tickStep(start, stop, count) returns 0 if count is infinity
#[test]
fn tickstep_start_stop_count_returns_0_if_count_is_infinity() {
    assert_eq!(ticks::tick_step(0.0, 1.0, f64::INFINITY), 0.0); // d3-array/test/tickStep-test.js:28:3
}

// d3-array/test/tickStep-test.js: tickStep(start, stop, count) returns approximately count + 1 tickStep when start < stop
#[test]
fn tickstep_start_stop_count_returns_approximately_count_1_tickstep_when_start_stop() {
    assert_eq!(ticks::tick_step(0.0, 1.0, 10.0), 0.1); // d3-array/test/tickStep-test.js:32:3
    assert_eq!(ticks::tick_step(0.0, 1.0, 9.0), 0.1); // d3-array/test/tickStep-test.js:33:3
    assert_eq!(ticks::tick_step(0.0, 1.0, 8.0), 0.1); // d3-array/test/tickStep-test.js:34:3
    assert_eq!(ticks::tick_step(0.0, 1.0, 7.0), 0.2); // d3-array/test/tickStep-test.js:35:3
    assert_eq!(ticks::tick_step(0.0, 1.0, 6.0), 0.2); // d3-array/test/tickStep-test.js:36:3
    assert_eq!(ticks::tick_step(0.0, 1.0, 5.0), 0.2); // d3-array/test/tickStep-test.js:37:3
    assert_eq!(ticks::tick_step(0.0, 1.0, 4.0), 0.2); // d3-array/test/tickStep-test.js:38:3
    assert_eq!(ticks::tick_step(0.0, 1.0, 3.0), 0.5); // d3-array/test/tickStep-test.js:39:3
    assert_eq!(ticks::tick_step(0.0, 1.0, 2.0), 0.5); // d3-array/test/tickStep-test.js:40:3
    assert_eq!(ticks::tick_step(0.0, 1.0, 1.0), 1.0); // d3-array/test/tickStep-test.js:41:3
    assert_eq!(ticks::tick_step(0.0, 10.0, 10.0), 1.0); // d3-array/test/tickStep-test.js:42:3
    assert_eq!(ticks::tick_step(0.0, 10.0, 9.0), 1.0); // d3-array/test/tickStep-test.js:43:3
    assert_eq!(ticks::tick_step(0.0, 10.0, 8.0), 1.0); // d3-array/test/tickStep-test.js:44:3
    assert_eq!(ticks::tick_step(0.0, 10.0, 7.0), 2.0); // d3-array/test/tickStep-test.js:45:3
    assert_eq!(ticks::tick_step(0.0, 10.0, 6.0), 2.0); // d3-array/test/tickStep-test.js:46:3
    assert_eq!(ticks::tick_step(0.0, 10.0, 5.0), 2.0); // d3-array/test/tickStep-test.js:47:3
    assert_eq!(ticks::tick_step(0.0, 10.0, 4.0), 2.0); // d3-array/test/tickStep-test.js:48:3
    assert_eq!(ticks::tick_step(0.0, 10.0, 3.0), 5.0); // d3-array/test/tickStep-test.js:49:3
    assert_eq!(ticks::tick_step(0.0, 10.0, 2.0), 5.0); // d3-array/test/tickStep-test.js:50:3
    assert_eq!(ticks::tick_step(0.0, 10.0, 1.0), 10.0); // d3-array/test/tickStep-test.js:51:3
    assert_eq!(ticks::tick_step(-10.0, 10.0, 10.0), 2.0); // d3-array/test/tickStep-test.js:52:3
    assert_eq!(ticks::tick_step(-10.0, 10.0, 9.0), 2.0); // d3-array/test/tickStep-test.js:53:3
    assert_eq!(ticks::tick_step(-10.0, 10.0, 8.0), 2.0); // d3-array/test/tickStep-test.js:54:3
    assert_eq!(ticks::tick_step(-10.0, 10.0, 7.0), 2.0); // d3-array/test/tickStep-test.js:55:3
    assert_eq!(ticks::tick_step(-10.0, 10.0, 6.0), 5.0); // d3-array/test/tickStep-test.js:56:3
    assert_eq!(ticks::tick_step(-10.0, 10.0, 5.0), 5.0); // d3-array/test/tickStep-test.js:57:3
    assert_eq!(ticks::tick_step(-10.0, 10.0, 4.0), 5.0); // d3-array/test/tickStep-test.js:58:3
    assert_eq!(ticks::tick_step(-10.0, 10.0, 3.0), 5.0); // d3-array/test/tickStep-test.js:59:3
    assert_eq!(ticks::tick_step(-10.0, 10.0, 2.0), 10.0); // d3-array/test/tickStep-test.js:60:3
    assert_eq!(ticks::tick_step(-10.0, 10.0, 1.0), 20.0); // d3-array/test/tickStep-test.js:61:3
}

// d3-array/test/tickStep-test.js: tickStep(start, stop, count) returns -tickStep(stop, start, count)
#[test]
fn tickstep_start_stop_count_returns_tickstep_stop_start_count() {
    assert_eq!(
        ticks::tick_step(0.0, 1.0, 10.0),
        -ticks::tick_step(1.0, 0.0, 10.0)
    ); // d3-array/test/tickStep-test.js:65:3
    assert_eq!(
        ticks::tick_step(0.0, 1.0, 9.0),
        -ticks::tick_step(1.0, 0.0, 9.0)
    ); // d3-array/test/tickStep-test.js:66:3
    assert_eq!(
        ticks::tick_step(0.0, 1.0, 8.0),
        -ticks::tick_step(1.0, 0.0, 8.0)
    ); // d3-array/test/tickStep-test.js:67:3
    assert_eq!(
        ticks::tick_step(0.0, 1.0, 7.0),
        -ticks::tick_step(1.0, 0.0, 7.0)
    ); // d3-array/test/tickStep-test.js:68:3
    assert_eq!(
        ticks::tick_step(0.0, 1.0, 6.0),
        -ticks::tick_step(1.0, 0.0, 6.0)
    ); // d3-array/test/tickStep-test.js:69:3
    assert_eq!(
        ticks::tick_step(0.0, 1.0, 5.0),
        -ticks::tick_step(1.0, 0.0, 5.0)
    ); // d3-array/test/tickStep-test.js:70:3
    assert_eq!(
        ticks::tick_step(0.0, 1.0, 4.0),
        -ticks::tick_step(1.0, 0.0, 4.0)
    ); // d3-array/test/tickStep-test.js:71:3
    assert_eq!(
        ticks::tick_step(0.0, 1.0, 3.0),
        -ticks::tick_step(1.0, 0.0, 3.0)
    ); // d3-array/test/tickStep-test.js:72:3
    assert_eq!(
        ticks::tick_step(0.0, 1.0, 2.0),
        -ticks::tick_step(1.0, 0.0, 2.0)
    ); // d3-array/test/tickStep-test.js:73:3
    assert_eq!(
        ticks::tick_step(0.0, 1.0, 1.0),
        -ticks::tick_step(1.0, 0.0, 1.0)
    ); // d3-array/test/tickStep-test.js:74:3
    assert_eq!(
        ticks::tick_step(0.0, 10.0, 10.0),
        -ticks::tick_step(10.0, 0.0, 10.0)
    ); // d3-array/test/tickStep-test.js:75:3
    assert_eq!(
        ticks::tick_step(0.0, 10.0, 9.0),
        -ticks::tick_step(10.0, 0.0, 9.0)
    ); // d3-array/test/tickStep-test.js:76:3
    assert_eq!(
        ticks::tick_step(0.0, 10.0, 8.0),
        -ticks::tick_step(10.0, 0.0, 8.0)
    ); // d3-array/test/tickStep-test.js:77:3
    assert_eq!(
        ticks::tick_step(0.0, 10.0, 7.0),
        -ticks::tick_step(10.0, 0.0, 7.0)
    ); // d3-array/test/tickStep-test.js:78:3
    assert_eq!(
        ticks::tick_step(0.0, 10.0, 6.0),
        -ticks::tick_step(10.0, 0.0, 6.0)
    ); // d3-array/test/tickStep-test.js:79:3
    assert_eq!(
        ticks::tick_step(0.0, 10.0, 5.0),
        -ticks::tick_step(10.0, 0.0, 5.0)
    ); // d3-array/test/tickStep-test.js:80:3
    assert_eq!(
        ticks::tick_step(0.0, 10.0, 4.0),
        -ticks::tick_step(10.0, 0.0, 4.0)
    ); // d3-array/test/tickStep-test.js:81:3
    assert_eq!(
        ticks::tick_step(0.0, 10.0, 3.0),
        -ticks::tick_step(10.0, 0.0, 3.0)
    ); // d3-array/test/tickStep-test.js:82:3
    assert_eq!(
        ticks::tick_step(0.0, 10.0, 2.0),
        -ticks::tick_step(10.0, 0.0, 2.0)
    ); // d3-array/test/tickStep-test.js:83:3
    assert_eq!(
        ticks::tick_step(0.0, 10.0, 1.0),
        -ticks::tick_step(10.0, 0.0, 1.0)
    ); // d3-array/test/tickStep-test.js:84:3
    assert_eq!(
        ticks::tick_step(-10.0, 10.0, 10.0),
        -ticks::tick_step(10.0, -10.0, 10.0)
    ); // d3-array/test/tickStep-test.js:85:3
    assert_eq!(
        ticks::tick_step(-10.0, 10.0, 9.0),
        -ticks::tick_step(10.0, -10.0, 9.0)
    ); // d3-array/test/tickStep-test.js:86:3
    assert_eq!(
        ticks::tick_step(-10.0, 10.0, 8.0),
        -ticks::tick_step(10.0, -10.0, 8.0)
    ); // d3-array/test/tickStep-test.js:87:3
    assert_eq!(
        ticks::tick_step(-10.0, 10.0, 7.0),
        -ticks::tick_step(10.0, -10.0, 7.0)
    ); // d3-array/test/tickStep-test.js:88:3
    assert_eq!(
        ticks::tick_step(-10.0, 10.0, 6.0),
        -ticks::tick_step(10.0, -10.0, 6.0)
    ); // d3-array/test/tickStep-test.js:89:3
    assert_eq!(
        ticks::tick_step(-10.0, 10.0, 5.0),
        -ticks::tick_step(10.0, -10.0, 5.0)
    ); // d3-array/test/tickStep-test.js:90:3
    assert_eq!(
        ticks::tick_step(-10.0, 10.0, 4.0),
        -ticks::tick_step(10.0, -10.0, 4.0)
    ); // d3-array/test/tickStep-test.js:91:3
    assert_eq!(
        ticks::tick_step(-10.0, 10.0, 3.0),
        -ticks::tick_step(10.0, -10.0, 3.0)
    ); // d3-array/test/tickStep-test.js:92:3
    assert_eq!(
        ticks::tick_step(-10.0, 10.0, 2.0),
        -ticks::tick_step(10.0, -10.0, 2.0)
    ); // d3-array/test/tickStep-test.js:93:3
    assert_eq!(
        ticks::tick_step(-10.0, 10.0, 1.0),
        -ticks::tick_step(10.0, -10.0, 1.0)
    ); // d3-array/test/tickStep-test.js:94:3
}

// d3-array/test/nice-test.js: nice(start, stop, count) returns [start, stop] if any argument is NaN
#[test]
fn nice_start_stop_count_returns_start_stop_if_any_argument_is_nan() {
    assert_d3_eq(&ticks::nice(f64::NAN, 1.0, 1.0), &vec![f64::NAN, 1.0]); // d3-array/test/nice-test.js:5:3
    assert_d3_eq(&ticks::nice(0.0, f64::NAN, 1.0), &vec![0.0, f64::NAN]); // d3-array/test/nice-test.js:6:3
    assert_d3_eq(&ticks::nice(0.0, 1.0, f64::NAN), &vec![0.0, 1.0]); // d3-array/test/nice-test.js:7:3
    assert_d3_eq(
        &ticks::nice(f64::NAN, f64::NAN, 1.0),
        &vec![f64::NAN, f64::NAN],
    ); // d3-array/test/nice-test.js:8:3
    assert_d3_eq(&ticks::nice(0.0, f64::NAN, f64::NAN), &vec![0.0, f64::NAN]); // d3-array/test/nice-test.js:9:3
    assert_d3_eq(&ticks::nice(f64::NAN, 1.0, f64::NAN), &vec![f64::NAN, 1.0]); // d3-array/test/nice-test.js:10:3
    assert_d3_eq(
        &ticks::nice(f64::NAN, f64::NAN, f64::NAN),
        &vec![f64::NAN, f64::NAN],
    ); // d3-array/test/nice-test.js:11:3
}

// d3-array/test/nice-test.js: nice(start, stop, count) returns [start, stop] if start === stop
#[test]
fn nice_start_stop_count_returns_start_stop_if_start_stop() {
    assert_d3_eq(&ticks::nice(1.0, 1.0, -1.0), &vec![1.0, 1.0]); // d3-array/test/nice-test.js:15:3
    assert_d3_eq(&ticks::nice(1.0, 1.0, 0.0), &vec![1.0, 1.0]); // d3-array/test/nice-test.js:16:3
    assert_d3_eq(&ticks::nice(1.0, 1.0, f64::NAN), &vec![1.0, 1.0]); // d3-array/test/nice-test.js:17:3
    assert_d3_eq(&ticks::nice(1.0, 1.0, 1.0), &vec![1.0, 1.0]); // d3-array/test/nice-test.js:18:3
    assert_d3_eq(&ticks::nice(1.0, 1.0, 10.0), &vec![1.0, 1.0]); // d3-array/test/nice-test.js:19:3
}

// d3-array/test/nice-test.js: nice(start, stop, count) returns [start, stop] if count is not positive
#[test]
fn nice_start_stop_count_returns_start_stop_if_count_is_not_positive() {
    assert_d3_eq(&ticks::nice(0.0, 1.0, -1.0), &vec![0.0, 1.0]); // d3-array/test/nice-test.js:23:3
    assert_d3_eq(&ticks::nice(0.0, 1.0, 0.0), &vec![0.0, 1.0]); // d3-array/test/nice-test.js:24:3
}

// d3-array/test/nice-test.js: nice(start, stop, count) returns [start, stop] if count is infinity
#[test]
fn nice_start_stop_count_returns_start_stop_if_count_is_infinity() {
    assert_d3_eq(&ticks::nice(0.0, 1.0, f64::INFINITY), &vec![0.0, 1.0]); // d3-array/test/nice-test.js:28:3
}

// d3-array/test/nice-test.js: nice(start, stop, count) returns the expected values
#[test]
fn nice_start_stop_count_returns_the_expected_values() {
    assert_d3_eq(&ticks::nice(0.132, 0.876, 1000.0), &vec![0.132, 0.876]); // d3-array/test/nice-test.js:32:3
    assert_d3_eq(&ticks::nice(0.132, 0.876, 100.0), &vec![0.13, 0.88]); // d3-array/test/nice-test.js:33:3
    assert_d3_eq(&ticks::nice(0.132, 0.876, 30.0), &vec![0.12, 0.88]); // d3-array/test/nice-test.js:34:3
    assert_d3_eq(&ticks::nice(0.132, 0.876, 10.0), &vec![0.1, 0.9]); // d3-array/test/nice-test.js:35:3
    assert_d3_eq(&ticks::nice(0.132, 0.876, 6.0), &vec![0.1, 0.9]); // d3-array/test/nice-test.js:36:3
    assert_d3_eq(&ticks::nice(0.132, 0.876, 5.0), &vec![0.0, 1.0]); // d3-array/test/nice-test.js:37:3
    assert_d3_eq(&ticks::nice(0.132, 0.876, 1.0), &vec![0.0, 1.0]); // d3-array/test/nice-test.js:38:3
    assert_d3_eq(&ticks::nice(132.0, 876.0, 1000.0), &vec![132.0, 876.0]); // d3-array/test/nice-test.js:39:3
    assert_d3_eq(&ticks::nice(132.0, 876.0, 100.0), &vec![130.0, 880.0]); // d3-array/test/nice-test.js:40:3
    assert_d3_eq(&ticks::nice(132.0, 876.0, 30.0), &vec![120.0, 880.0]); // d3-array/test/nice-test.js:41:3
    assert_d3_eq(&ticks::nice(132.0, 876.0, 10.0), &vec![100.0, 900.0]); // d3-array/test/nice-test.js:42:3
    assert_d3_eq(&ticks::nice(132.0, 876.0, 6.0), &vec![100.0, 900.0]); // d3-array/test/nice-test.js:43:3
    assert_d3_eq(&ticks::nice(132.0, 876.0, 5.0), &vec![0.0, 1000.0]); // d3-array/test/nice-test.js:44:3
    assert_d3_eq(&ticks::nice(132.0, 876.0, 1.0), &vec![0.0, 1000.0]); // d3-array/test/nice-test.js:45:3
}
