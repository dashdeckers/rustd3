//! d3's tests of d3-time, as assertions on rustd3-core; each names d3's file, line and column.

use rustd3_core::jiff::civil::Weekday;
use rustd3_core::jiff::tz::TimeZone;
use rustd3_core::time::{self, Interval};

use super::support::{assert_d3_eq, local, millis, ts, tss, utc, zone};

// d3-time/test/millisecond-test.js: timeMillisecond.every(step) returns every stepth millisecond, starting with the first millisecond of the second
#[test]
fn timemillisecond_every_step_returns_every_stepth_millisecond_starting_with_the_first_millisecond_of_the_second()
 {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Millisecond(1)
            .every(50_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                local(&la, &[2008.0, 11.0, 30.0, 12.0, 36.0, 0.0, 947.0]),
                local(&la, &[2008.0, 11.0, 30.0, 12.0, 36.0, 1.0, 157.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2008.0, 11.0, 30.0, 12.0, 36.0, 0.0, 950.0]),
            local(&la, &[2008.0, 11.0, 30.0, 12.0, 36.0, 1.0, 0.0]),
            local(&la, &[2008.0, 11.0, 30.0, 12.0, 36.0, 1.0, 50.0]),
            local(&la, &[2008.0, 11.0, 30.0, 12.0, 36.0, 1.0, 100.0]),
            local(&la, &[2008.0, 11.0, 30.0, 12.0, 36.0, 1.0, 150.0])
        ])
    ); // d3-time/test/millisecond-test.js:6:3
    assert_eq!(
        tss(&Interval::Millisecond(1)
            .every(100_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                local(&la, &[2008.0, 11.0, 30.0, 12.0, 36.0, 0.0, 947.0]),
                local(&la, &[2008.0, 11.0, 30.0, 12.0, 36.0, 1.0, 157.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2008.0, 11.0, 30.0, 12.0, 36.0, 1.0, 0.0]),
            local(&la, &[2008.0, 11.0, 30.0, 12.0, 36.0, 1.0, 100.0])
        ])
    ); // d3-time/test/millisecond-test.js:7:3
    assert_eq!(
        tss(&Interval::Millisecond(1)
            .every(50_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                utc(&[2008.0, 11.0, 30.0, 12.0, 36.0, 0.0, 947.0]),
                utc(&[2008.0, 11.0, 30.0, 12.0, 36.0, 1.0, 157.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2008.0, 11.0, 30.0, 12.0, 36.0, 0.0, 950.0]),
            utc(&[2008.0, 11.0, 30.0, 12.0, 36.0, 1.0, 0.0]),
            utc(&[2008.0, 11.0, 30.0, 12.0, 36.0, 1.0, 50.0]),
            utc(&[2008.0, 11.0, 30.0, 12.0, 36.0, 1.0, 100.0]),
            utc(&[2008.0, 11.0, 30.0, 12.0, 36.0, 1.0, 150.0])
        ])
    ); // d3-time/test/millisecond-test.js:8:3
    assert_eq!(
        tss(&Interval::Millisecond(1)
            .every(100_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                utc(&[2008.0, 11.0, 30.0, 12.0, 36.0, 0.0, 947.0]),
                utc(&[2008.0, 11.0, 30.0, 12.0, 36.0, 1.0, 157.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2008.0, 11.0, 30.0, 12.0, 36.0, 1.0, 0.0]),
            utc(&[2008.0, 11.0, 30.0, 12.0, 36.0, 1.0, 100.0])
        ])
    ); // d3-time/test/millisecond-test.js:9:3
}

// d3-time/test/utcMillisecond-test.js: utcMillisecond is an alias for timeMillisecond
#[test]
fn utcmillisecond_is_an_alias_for_timemillisecond() {
    assert_eq!(Interval::Millisecond(1), Interval::Millisecond(1)); // d3-time/test/utcMillisecond-test.js:5:3
}

// d3-time/test/second-test.js: timeSecond.floor(date) returns seconds
#[test]
fn timesecond_floor_date_returns_seconds() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Second(1).floor(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]))
    ); // d3-time/test/second-test.js:6:3
    assert_eq!(
        ts(Interval::Second(1).floor(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]))
    ); // d3-time/test/second-test.js:7:3
    assert_eq!(
        ts(Interval::Second(1).floor(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]))
    ); // d3-time/test/second-test.js:8:3
}

// d3-time/test/second-test.js: timeSecond.round(date) returns seconds
#[test]
fn timesecond_round_date_returns_seconds() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Second(1).round(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]))
    ); // d3-time/test/second-test.js:12:3
    assert_eq!(
        ts(Interval::Second(1).round(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0, 499.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]))
    ); // d3-time/test/second-test.js:13:3
    assert_eq!(
        ts(Interval::Second(1).round(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0, 500.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]))
    ); // d3-time/test/second-test.js:14:3
}

// d3-time/test/second-test.js: timeSecond.ceil(date) returns seconds
#[test]
fn timesecond_ceil_date_returns_seconds() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Second(1).ceil(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]))
    ); // d3-time/test/second-test.js:18:3
    assert_eq!(
        ts(Interval::Second(1).ceil(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]))
    ); // d3-time/test/second-test.js:19:3
    assert_eq!(
        ts(Interval::Second(1).ceil(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]))
    ); // d3-time/test/second-test.js:20:3
}

// d3-time/test/second-test.js: timeSecond.offset(date, step) does not modify the passed-in date
#[test]
fn timesecond_offset_date_step_does_not_modify_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    let d = local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Second(1).offset(d, 1_i64, &la);
    assert_eq!(
        ts(d),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/second-test.js:26:3
}

// d3-time/test/second-test.js: timeSecond.offset(date, step) does not round the passed-in-date
#[test]
fn timesecond_offset_date_step_does_not_round_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Second(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0, 999.0]))
    ); // d3-time/test/second-test.js:30:3
    assert_eq!(
        ts(Interval::Second(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 57.0, 456.0]))
    ); // d3-time/test/second-test.js:31:3
}

// d3-time/test/second-test.js: timeSecond.offset(date, step) allows negative offsets
#[test]
fn timesecond_offset_date_step_allows_negative_offsets() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Second(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]),
            -1_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0]))
    ); // d3-time/test/second-test.js:35:3
    assert_eq!(
        ts(Interval::Second(1).offset(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), -2_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0]))
    ); // d3-time/test/second-test.js:36:3
    assert_eq!(
        ts(Interval::Second(1).offset(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), -1_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]))
    ); // d3-time/test/second-test.js:37:3
}

// d3-time/test/second-test.js: timeSecond.offset(date, step) allows positive offsets
#[test]
fn timesecond_offset_date_step_allows_positive_offsets() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Second(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]))
    ); // d3-time/test/second-test.js:41:3
    assert_eq!(
        ts(Interval::Second(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0]),
            2_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]))
    ); // d3-time/test/second-test.js:42:3
    assert_eq!(
        ts(Interval::Second(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]))
    ); // d3-time/test/second-test.js:43:3
}

// d3-time/test/second-test.js: timeSecond.offset(date, step) allows zero offset
#[test]
fn timesecond_offset_date_step_allows_zero_offset() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Second(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/second-test.js:47:3
    assert_eq!(
        ts(Interval::Second(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/second-test.js:48:3
}

// d3-time/test/second-test.js: timeSecond.range(start, stop) returns seconds
#[test]
fn timesecond_range_start_stop_returns_seconds() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Second(1)
            .range(
                local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]),
                local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 2.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]),
            local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0])
        ])
    ); // d3-time/test/second-test.js:52:3
}

// d3-time/test/second-test.js: timeSecond.range(start, stop) has an inclusive lower bound
#[test]
fn timesecond_range_start_stop_has_an_inclusive_lower_bound() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Second(1)
            .range(
                local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]),
                local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 2.0]),
                &la
            )
            .collect::<Vec<_>>()[0_usize]),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]))
    ); // d3-time/test/second-test.js:60:3
}

// d3-time/test/second-test.js: timeSecond.range(start, stop) has an exclusive upper bound
#[test]
fn timesecond_range_start_stop_has_an_exclusive_upper_bound() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Second(1)
            .range(
                local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]),
                local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 2.0]),
                &la
            )
            .collect::<Vec<_>>()[2_usize]),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]))
    ); // d3-time/test/second-test.js:64:3
}

// d3-time/test/second-test.js: timeSecond.range(start, stop, step) can skip seconds
#[test]
fn timesecond_range_start_stop_step_can_skip_seconds() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Second(1)
            .range_step(
                local(&la, &[2011.0, 1.0, 1.0, 12.0, 0.0, 7.0]),
                local(&la, &[2011.0, 1.0, 1.0, 12.0, 1.0, 7.0]),
                15_u32,
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 1.0, 1.0, 12.0, 0.0, 7.0]),
            local(&la, &[2011.0, 1.0, 1.0, 12.0, 0.0, 22.0]),
            local(&la, &[2011.0, 1.0, 1.0, 12.0, 0.0, 37.0]),
            local(&la, &[2011.0, 1.0, 1.0, 12.0, 0.0, 52.0])
        ])
    ); // d3-time/test/second-test.js:68:3
}

// d3-time/test/second-test.js: timeSecond.range(start, stop) observes start of daylight savings time
#[test]
fn timesecond_range_start_stop_observes_start_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Second(1)
            .range(
                utc(&[2011.0, 2.0, 13.0, 9.0, 59.0, 59.0]),
                utc(&[2011.0, 2.0, 13.0, 10.0, 0.0, 2.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 2.0, 13.0, 9.0, 59.0, 59.0]),
            utc(&[2011.0, 2.0, 13.0, 10.0, 0.0, 0.0]),
            utc(&[2011.0, 2.0, 13.0, 10.0, 0.0, 1.0])
        ])
    ); // d3-time/test/second-test.js:77:3
}

// d3-time/test/second-test.js: timeSecond.range(start, stop) observes end of daylight savings time
#[test]
fn timesecond_range_start_stop_observes_end_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Second(1)
            .range(
                utc(&[2011.0, 10.0, 6.0, 8.0, 59.0, 59.0]),
                utc(&[2011.0, 10.0, 6.0, 9.0, 0.0, 2.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 10.0, 6.0, 8.0, 59.0, 59.0]),
            utc(&[2011.0, 10.0, 6.0, 9.0, 0.0, 0.0]),
            utc(&[2011.0, 10.0, 6.0, 9.0, 0.0, 1.0])
        ])
    ); // d3-time/test/second-test.js:85:3
}

// d3-time/test/second-test.js: timeSecond.every(step) returns every stepth second, starting with the first second of the minute
#[test]
fn timesecond_every_step_returns_every_stepth_second_starting_with_the_first_second_of_the_minute()
{
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Second(1)
            .every(15_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                local(&la, &[2008.0, 11.0, 30.0, 12.0, 36.0, 47.0]),
                local(&la, &[2008.0, 11.0, 30.0, 12.0, 37.0, 57.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2008.0, 11.0, 30.0, 12.0, 37.0, 0.0]),
            local(&la, &[2008.0, 11.0, 30.0, 12.0, 37.0, 15.0]),
            local(&la, &[2008.0, 11.0, 30.0, 12.0, 37.0, 30.0]),
            local(&la, &[2008.0, 11.0, 30.0, 12.0, 37.0, 45.0])
        ])
    ); // d3-time/test/second-test.js:93:3
    assert_eq!(
        tss(&Interval::Second(1)
            .every(30_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                local(&la, &[2008.0, 11.0, 30.0, 12.0, 36.0, 47.0]),
                local(&la, &[2008.0, 11.0, 30.0, 12.0, 37.0, 57.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2008.0, 11.0, 30.0, 12.0, 37.0, 0.0]),
            local(&la, &[2008.0, 11.0, 30.0, 12.0, 37.0, 30.0])
        ])
    ); // d3-time/test/second-test.js:94:3
}

// d3-time/test/second-test.js: timeSecond.range(start, stop) returns every second crossing the daylight savings boundary
#[test]
fn timesecond_range_start_stop_returns_every_second_crossing_the_daylight_savings_boundary() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Second(1)
            .range(
                millis(1478422800000.0 - (2.0 * 1e3)),
                millis(1478422800000.0 + (2.0 * 1e3)),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            millis(1478422798000.0),
            millis(1478422799000.0),
            millis(1478422800000.0),
            millis(1478422801000.0)
        ])
    ); // d3-time/test/second-test.js:98:3
}

// d3-time/test/utcSecond-test.js: utcSecond.floor(date) returns seconds
#[test]
fn utcsecond_floor_date_returns_seconds() {
    assert_eq!(
        ts(Interval::Second(1).floor(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]))
    ); // d3-time/test/utcSecond-test.js:6:3
    assert_eq!(
        ts(Interval::Second(1).floor(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]))
    ); // d3-time/test/utcSecond-test.js:7:3
    assert_eq!(
        ts(Interval::Second(1).floor(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]))
    ); // d3-time/test/utcSecond-test.js:8:3
}

// d3-time/test/utcSecond-test.js: utcSecond.round(date) returns seconds
#[test]
fn utcsecond_round_date_returns_seconds() {
    assert_eq!(
        ts(Interval::Second(1).round(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]))
    ); // d3-time/test/utcSecond-test.js:12:3
    assert_eq!(
        ts(Interval::Second(1).round(
            utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0, 499.0]),
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]))
    ); // d3-time/test/utcSecond-test.js:13:3
    assert_eq!(
        ts(Interval::Second(1).round(
            utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0, 500.0]),
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]))
    ); // d3-time/test/utcSecond-test.js:14:3
}

// d3-time/test/utcSecond-test.js: utcSecond.ceil(date) returns seconds
#[test]
fn utcsecond_ceil_date_returns_seconds() {
    assert_eq!(
        ts(Interval::Second(1).ceil(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]))
    ); // d3-time/test/utcSecond-test.js:18:3
    assert_eq!(
        ts(Interval::Second(1).ceil(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]))
    ); // d3-time/test/utcSecond-test.js:19:3
    assert_eq!(
        ts(Interval::Second(1).ceil(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]))
    ); // d3-time/test/utcSecond-test.js:20:3
}

// d3-time/test/utcSecond-test.js: utcSecond.offset(date, step) does not modify the passed-in date
#[test]
fn utcsecond_offset_date_step_does_not_modify_the_passed_in_date() {
    let d = utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Second(1).offset(d, 1_i64, &TimeZone::UTC);
    assert_eq!(
        ts(d),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcSecond-test.js:26:3
}

// d3-time/test/utcSecond-test.js: utcSecond.offset(date, step) does not round the passed-in-date
#[test]
fn utcsecond_offset_date_step_does_not_round_the_passed_in_date() {
    assert_eq!(
        ts(Interval::Second(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0, 999.0]))
    ); // d3-time/test/utcSecond-test.js:30:3
    assert_eq!(
        ts(Interval::Second(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 57.0, 456.0]))
    ); // d3-time/test/utcSecond-test.js:31:3
}

// d3-time/test/utcSecond-test.js: utcSecond.offset(date, step) allows negative offsets
#[test]
fn utcsecond_offset_date_step_allows_negative_offsets() {
    assert_eq!(
        ts(Interval::Second(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]),
            -1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0]))
    ); // d3-time/test/utcSecond-test.js:35:3
    assert_eq!(
        ts(Interval::Second(1).offset(
            utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]),
            -2_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0]))
    ); // d3-time/test/utcSecond-test.js:36:3
    assert_eq!(
        ts(Interval::Second(1).offset(
            utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]),
            -1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]))
    ); // d3-time/test/utcSecond-test.js:37:3
}

// d3-time/test/utcSecond-test.js: utcSecond.offset(date, step) allows positive offsets
#[test]
fn utcsecond_offset_date_step_allows_positive_offsets() {
    assert_eq!(
        ts(Interval::Second(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]))
    ); // d3-time/test/utcSecond-test.js:41:3
    assert_eq!(
        ts(Interval::Second(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0]),
            2_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]))
    ); // d3-time/test/utcSecond-test.js:42:3
    assert_eq!(
        ts(Interval::Second(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]))
    ); // d3-time/test/utcSecond-test.js:43:3
}

// d3-time/test/utcSecond-test.js: utcSecond.offset(date, step) allows zero offset
#[test]
fn utcsecond_offset_date_step_allows_zero_offset() {
    assert_eq!(
        ts(Interval::Second(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcSecond-test.js:47:3
    assert_eq!(
        ts(Interval::Second(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/utcSecond-test.js:48:3
}

// d3-time/test/utcSecond-test.js: utcSecond.range(start, stop) returns seconds
#[test]
fn utcsecond_range_start_stop_returns_seconds() {
    assert_eq!(
        tss(&Interval::Second(1)
            .range(
                utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]),
                utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 2.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]),
            utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0])
        ])
    ); // d3-time/test/utcSecond-test.js:52:3
}

// d3-time/test/utcSecond-test.js: utcSecond.range(start, stop) has an inclusive lower bound
#[test]
fn utcsecond_range_start_stop_has_an_inclusive_lower_bound() {
    assert_eq!(
        ts(Interval::Second(1)
            .range(
                utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]),
                utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 2.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()[0_usize]),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]))
    ); // d3-time/test/utcSecond-test.js:60:3
}

// d3-time/test/utcSecond-test.js: utcSecond.range(start, stop) has an exclusive upper bound
#[test]
fn utcsecond_range_start_stop_has_an_exclusive_upper_bound() {
    assert_eq!(
        ts(Interval::Second(1)
            .range(
                utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]),
                utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 2.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()[2_usize]),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]))
    ); // d3-time/test/utcSecond-test.js:64:3
}

// d3-time/test/utcSecond-test.js: utcSecond.range(start, stop, step) can skip seconds
#[test]
fn utcsecond_range_start_stop_step_can_skip_seconds() {
    assert_eq!(
        tss(&Interval::Second(1)
            .range_step(
                utc(&[2011.0, 1.0, 1.0, 12.0, 0.0, 7.0]),
                utc(&[2011.0, 1.0, 1.0, 12.0, 1.0, 7.0]),
                15_u32,
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 1.0, 1.0, 12.0, 0.0, 7.0]),
            utc(&[2011.0, 1.0, 1.0, 12.0, 0.0, 22.0]),
            utc(&[2011.0, 1.0, 1.0, 12.0, 0.0, 37.0]),
            utc(&[2011.0, 1.0, 1.0, 12.0, 0.0, 52.0])
        ])
    ); // d3-time/test/utcSecond-test.js:68:3
}

// d3-time/test/utcSecond-test.js: utcSecond.range(start, stop) observes start of daylight savings time
#[test]
fn utcsecond_range_start_stop_observes_start_of_daylight_savings_time() {
    assert_eq!(
        tss(&Interval::Second(1)
            .range(
                utc(&[2011.0, 2.0, 13.0, 9.0, 59.0, 59.0]),
                utc(&[2011.0, 2.0, 13.0, 10.0, 0.0, 2.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 2.0, 13.0, 9.0, 59.0, 59.0]),
            utc(&[2011.0, 2.0, 13.0, 10.0, 0.0, 0.0]),
            utc(&[2011.0, 2.0, 13.0, 10.0, 0.0, 1.0])
        ])
    ); // d3-time/test/utcSecond-test.js:77:3
}

// d3-time/test/utcSecond-test.js: utcSecond.range(start, stop) observes end of daylight savings time
#[test]
fn utcsecond_range_start_stop_observes_end_of_daylight_savings_time() {
    assert_eq!(
        tss(&Interval::Second(1)
            .range(
                utc(&[2011.0, 10.0, 6.0, 8.0, 59.0, 59.0]),
                utc(&[2011.0, 10.0, 6.0, 9.0, 0.0, 2.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 10.0, 6.0, 8.0, 59.0, 59.0]),
            utc(&[2011.0, 10.0, 6.0, 9.0, 0.0, 0.0]),
            utc(&[2011.0, 10.0, 6.0, 9.0, 0.0, 1.0])
        ])
    ); // d3-time/test/utcSecond-test.js:85:3
}

// d3-time/test/utcSecond-test.js: utcSecond.every(step) returns every stepth second, starting with the first second of the minute
#[test]
fn utcsecond_every_step_returns_every_stepth_second_starting_with_the_first_second_of_the_minute() {
    assert_eq!(
        tss(&Interval::Second(1)
            .every(15_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                utc(&[2008.0, 11.0, 30.0, 12.0, 36.0, 47.0]),
                utc(&[2008.0, 11.0, 30.0, 12.0, 37.0, 57.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2008.0, 11.0, 30.0, 12.0, 37.0, 0.0]),
            utc(&[2008.0, 11.0, 30.0, 12.0, 37.0, 15.0]),
            utc(&[2008.0, 11.0, 30.0, 12.0, 37.0, 30.0]),
            utc(&[2008.0, 11.0, 30.0, 12.0, 37.0, 45.0])
        ])
    ); // d3-time/test/utcSecond-test.js:93:3
    assert_eq!(
        tss(&Interval::Second(1)
            .every(30_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                utc(&[2008.0, 11.0, 30.0, 12.0, 36.0, 47.0]),
                utc(&[2008.0, 11.0, 30.0, 12.0, 37.0, 57.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2008.0, 11.0, 30.0, 12.0, 37.0, 0.0]),
            utc(&[2008.0, 11.0, 30.0, 12.0, 37.0, 30.0])
        ])
    ); // d3-time/test/utcSecond-test.js:94:3
}

// d3-time/test/minute-test.js: timeMinute.floor(date) returns minutes
#[test]
fn timeminute_floor_date_returns_minutes() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Minute(1).floor(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0]))
    ); // d3-time/test/minute-test.js:6:3
    assert_eq!(
        ts(Interval::Minute(1).floor(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0]))
    ); // d3-time/test/minute-test.js:7:3
    assert_eq!(
        ts(Interval::Minute(1).floor(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 59.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0]))
    ); // d3-time/test/minute-test.js:8:3
    assert_eq!(
        ts(Interval::Minute(1).floor(local(&la, &[2011.0, 0.0, 1.0, 0.0, 1.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 1.0]))
    ); // d3-time/test/minute-test.js:9:3
}

// d3-time/test/minute-test.js: timeMinute.ceil(date) returns minutes
#[test]
fn timeminute_ceil_date_returns_minutes() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Minute(1).ceil(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0]))
    ); // d3-time/test/minute-test.js:13:3
    assert_eq!(
        ts(Interval::Minute(1).ceil(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0]))
    ); // d3-time/test/minute-test.js:14:3
    assert_eq!(
        ts(Interval::Minute(1).ceil(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 59.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 1.0]))
    ); // d3-time/test/minute-test.js:15:3
    assert_eq!(
        ts(Interval::Minute(1).ceil(local(&la, &[2011.0, 0.0, 1.0, 0.0, 1.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 1.0]))
    ); // d3-time/test/minute-test.js:16:3
}

// d3-time/test/minute-test.js: timeMinute.offset(date) does not modify the passed-in date
#[test]
fn timeminute_offset_date_does_not_modify_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    let d = local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Minute(1).offset(d, 1_i64, &la);
    assert_eq!(
        ts(d),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/minute-test.js:22:3
}

// d3-time/test/minute-test.js: timeMinute.offset(date) does not round the passed-in-date
#[test]
fn timeminute_offset_date_does_not_round_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Minute(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 59.0, 999.0]))
    ); // d3-time/test/minute-test.js:26:3
    assert_eq!(
        ts(Interval::Minute(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 57.0, 59.0, 456.0]))
    ); // d3-time/test/minute-test.js:27:3
}

// d3-time/test/minute-test.js: timeMinute.offset(date) allows negative offsets
#[test]
fn timeminute_offset_date_allows_negative_offsets() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Minute(1).offset(local(&la, &[2010.0, 11.0, 31.0, 23.0, 12.0]), -1_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 11.0]))
    ); // d3-time/test/minute-test.js:31:3
    assert_eq!(
        ts(Interval::Minute(1).offset(local(&la, &[2011.0, 0.0, 1.0, 0.0, 1.0]), -2_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0]))
    ); // d3-time/test/minute-test.js:32:3
    assert_eq!(
        ts(Interval::Minute(1).offset(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0]), -1_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0]))
    ); // d3-time/test/minute-test.js:33:3
}

// d3-time/test/minute-test.js: timeMinute.offset(date) allows positive offsets
#[test]
fn timeminute_offset_date_allows_positive_offsets() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Minute(1).offset(local(&la, &[2010.0, 11.0, 31.0, 23.0, 11.0]), 1_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 12.0]))
    ); // d3-time/test/minute-test.js:37:3
    assert_eq!(
        ts(Interval::Minute(1).offset(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0]), 2_i64, &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 1.0]))
    ); // d3-time/test/minute-test.js:38:3
    assert_eq!(
        ts(Interval::Minute(1).offset(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0]), 1_i64, &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0]))
    ); // d3-time/test/minute-test.js:39:3
}

// d3-time/test/minute-test.js: timeMinute.offset(date) allows zero offset
#[test]
fn timeminute_offset_date_allows_zero_offset() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Minute(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/minute-test.js:43:3
    assert_eq!(
        ts(Interval::Minute(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/minute-test.js:44:3
}

// d3-time/test/minute-test.js: timeMinute.range(start, stop), returns minutes
#[test]
fn timeminute_range_start_stop_returns_minutes() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Minute(1)
            .range(
                local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0]),
                local(&la, &[2011.0, 0.0, 1.0, 0.0, 2.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0]),
            local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 0.0, 1.0])
        ])
    ); // d3-time/test/minute-test.js:48:3
}

// d3-time/test/minute-test.js: timeMinute.range(start, stop), has an inclusive lower bound
#[test]
fn timeminute_range_start_stop_has_an_inclusive_lower_bound() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Minute(1)
            .range(
                local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0]),
                local(&la, &[2011.0, 0.0, 1.0, 0.0, 2.0]),
                &la
            )
            .collect::<Vec<_>>()[0_usize]),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0]))
    ); // d3-time/test/minute-test.js:56:3
}

// d3-time/test/minute-test.js: timeMinute.range(start, stop), has an exclusive upper bound
#[test]
fn timeminute_range_start_stop_has_an_exclusive_upper_bound() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Minute(1)
            .range(
                local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0]),
                local(&la, &[2011.0, 0.0, 1.0, 0.0, 2.0]),
                &la
            )
            .collect::<Vec<_>>()[2_usize]),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 1.0]))
    ); // d3-time/test/minute-test.js:60:3
}

// d3-time/test/minute-test.js: timeMinute.range(start, stop), can skip minutes
#[test]
fn timeminute_range_start_stop_can_skip_minutes() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Minute(1)
            .range_step(
                local(&la, &[2011.0, 1.0, 1.0, 12.0, 7.0]),
                local(&la, &[2011.0, 1.0, 1.0, 13.0, 7.0]),
                15_u32,
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 1.0, 1.0, 12.0, 7.0]),
            local(&la, &[2011.0, 1.0, 1.0, 12.0, 22.0]),
            local(&la, &[2011.0, 1.0, 1.0, 12.0, 37.0]),
            local(&la, &[2011.0, 1.0, 1.0, 12.0, 52.0])
        ])
    ); // d3-time/test/minute-test.js:64:3
}

// d3-time/test/minute-test.js: timeMinute.range(start, stop), observes start of daylight savings time
#[test]
fn timeminute_range_start_stop_observes_start_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Minute(1)
            .range(
                utc(&[2011.0, 2.0, 13.0, 9.0, 59.0]),
                utc(&[2011.0, 2.0, 13.0, 10.0, 2.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 2.0, 13.0, 9.0, 59.0]),
            utc(&[2011.0, 2.0, 13.0, 10.0, 0.0]),
            utc(&[2011.0, 2.0, 13.0, 10.0, 1.0])
        ])
    ); // d3-time/test/minute-test.js:73:3
}

// d3-time/test/minute-test.js: timeMinute.range(start, stop), observes end of daylight savings time
#[test]
fn timeminute_range_start_stop_observes_end_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Minute(1)
            .range(
                utc(&[2011.0, 10.0, 6.0, 8.0, 59.0]),
                utc(&[2011.0, 10.0, 6.0, 9.0, 2.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 10.0, 6.0, 8.0, 59.0]),
            utc(&[2011.0, 10.0, 6.0, 9.0, 0.0]),
            utc(&[2011.0, 10.0, 6.0, 9.0, 1.0])
        ])
    ); // d3-time/test/minute-test.js:81:3
}

// d3-time/test/minute-test.js: timeMinute.every(step) returns every stepth minute, starting with the first minute of the hour
#[test]
fn timeminute_every_step_returns_every_stepth_minute_starting_with_the_first_minute_of_the_hour() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Minute(1)
            .every(15_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                local(&la, &[2008.0, 11.0, 30.0, 12.0, 47.0]),
                local(&la, &[2008.0, 11.0, 30.0, 13.0, 57.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2008.0, 11.0, 30.0, 13.0, 0.0]),
            local(&la, &[2008.0, 11.0, 30.0, 13.0, 15.0]),
            local(&la, &[2008.0, 11.0, 30.0, 13.0, 30.0]),
            local(&la, &[2008.0, 11.0, 30.0, 13.0, 45.0])
        ])
    ); // d3-time/test/minute-test.js:89:3
    assert_eq!(
        tss(&Interval::Minute(1)
            .every(30_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                local(&la, &[2008.0, 11.0, 30.0, 12.0, 47.0]),
                local(&la, &[2008.0, 11.0, 30.0, 13.0, 57.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2008.0, 11.0, 30.0, 13.0, 0.0]),
            local(&la, &[2008.0, 11.0, 30.0, 13.0, 30.0])
        ])
    ); // d3-time/test/minute-test.js:90:3
}

// d3-time/test/minute-test.js: timeMinute.range(start, stop) returns every minute crossing the daylight savings boundary
#[test]
fn timeminute_range_start_stop_returns_every_minute_crossing_the_daylight_savings_boundary() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Minute(1)
            .range(
                millis(1478422800000.0 - (2.0 * 6e4)),
                millis(1478422800000.0 + (2.0 * 6e4)),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            millis(1478422680000.0),
            millis(1478422740000.0),
            millis(1478422800000.0),
            millis(1478422860000.0)
        ])
    ); // d3-time/test/minute-test.js:94:3
}

// d3-time/test/utcMinute-test.js: utcMinute.floor(date) returns minutes
#[test]
fn utcminute_floor_date_returns_minutes() {
    assert_eq!(
        ts(Interval::Minute(1).floor(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0]))
    ); // d3-time/test/utcMinute-test.js:6:3
    assert_eq!(
        ts(Interval::Minute(1).floor(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0]))
    ); // d3-time/test/utcMinute-test.js:7:3
    assert_eq!(
        ts(Interval::Minute(1).floor(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0]))
    ); // d3-time/test/utcMinute-test.js:8:3
    assert_eq!(
        ts(Interval::Minute(1).floor(utc(&[2011.0, 0.0, 1.0, 0.0, 1.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 1.0]))
    ); // d3-time/test/utcMinute-test.js:9:3
}

// d3-time/test/utcMinute-test.js: utcMinute.ceil(date) returns minutes
#[test]
fn utcminute_ceil_date_returns_minutes() {
    assert_eq!(
        ts(Interval::Minute(1).ceil(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0]))
    ); // d3-time/test/utcMinute-test.js:13:3
    assert_eq!(
        ts(Interval::Minute(1).ceil(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0]))
    ); // d3-time/test/utcMinute-test.js:14:3
    assert_eq!(
        ts(Interval::Minute(1).ceil(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 1.0]))
    ); // d3-time/test/utcMinute-test.js:15:3
    assert_eq!(
        ts(Interval::Minute(1).ceil(utc(&[2011.0, 0.0, 1.0, 0.0, 1.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 1.0]))
    ); // d3-time/test/utcMinute-test.js:16:3
}

// d3-time/test/utcMinute-test.js: utcMinute.offset(date) does not modify the passed-in date
#[test]
fn utcminute_offset_date_does_not_modify_the_passed_in_date() {
    let d = utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Minute(1).offset(d, 1_i64, &TimeZone::UTC);
    assert_eq!(
        ts(d),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcMinute-test.js:22:3
}

// d3-time/test/utcMinute-test.js: utcMinute.offset(date) does not round the passed-in-date
#[test]
fn utcminute_offset_date_does_not_round_the_passed_in_date() {
    assert_eq!(
        ts(Interval::Minute(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 59.0, 999.0]))
    ); // d3-time/test/utcMinute-test.js:26:3
    assert_eq!(
        ts(Interval::Minute(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 57.0, 59.0, 456.0]))
    ); // d3-time/test/utcMinute-test.js:27:3
}

// d3-time/test/utcMinute-test.js: utcMinute.offset(date) allows negative offsets
#[test]
fn utcminute_offset_date_allows_negative_offsets() {
    assert_eq!(
        ts(Interval::Minute(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 12.0]),
            -1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 11.0]))
    ); // d3-time/test/utcMinute-test.js:31:3
    assert_eq!(
        ts(Interval::Minute(1).offset(utc(&[2011.0, 0.0, 1.0, 0.0, 1.0]), -2_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0]))
    ); // d3-time/test/utcMinute-test.js:32:3
    assert_eq!(
        ts(Interval::Minute(1).offset(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0]), -1_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0]))
    ); // d3-time/test/utcMinute-test.js:33:3
}

// d3-time/test/utcMinute-test.js: utcMinute.offset(date) allows positive offsets
#[test]
fn utcminute_offset_date_allows_positive_offsets() {
    assert_eq!(
        ts(Interval::Minute(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 11.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 12.0]))
    ); // d3-time/test/utcMinute-test.js:37:3
    assert_eq!(
        ts(Interval::Minute(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0]),
            2_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 1.0]))
    ); // d3-time/test/utcMinute-test.js:38:3
    assert_eq!(
        ts(Interval::Minute(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0]))
    ); // d3-time/test/utcMinute-test.js:39:3
}

// d3-time/test/utcMinute-test.js: utcMinute.offset(date) allows zero offset
#[test]
fn utcminute_offset_date_allows_zero_offset() {
    assert_eq!(
        ts(Interval::Minute(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcMinute-test.js:43:3
    assert_eq!(
        ts(Interval::Minute(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/utcMinute-test.js:44:3
}

// d3-time/test/utcMinute-test.js: utcMinute.range(start, stop), returns minutes
#[test]
fn utcminute_range_start_stop_returns_minutes() {
    assert_eq!(
        tss(&Interval::Minute(1)
            .range(
                utc(&[2010.0, 11.0, 31.0, 23.0, 59.0]),
                utc(&[2011.0, 0.0, 1.0, 0.0, 2.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0]),
            utc(&[2011.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 0.0, 1.0])
        ])
    ); // d3-time/test/utcMinute-test.js:48:3
}

// d3-time/test/utcMinute-test.js: utcMinute.range(start, stop), has an inclusive lower bound
#[test]
fn utcminute_range_start_stop_has_an_inclusive_lower_bound() {
    assert_eq!(
        ts(Interval::Minute(1)
            .range(
                utc(&[2010.0, 11.0, 31.0, 23.0, 59.0]),
                utc(&[2011.0, 0.0, 1.0, 0.0, 2.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()[0_usize]),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0]))
    ); // d3-time/test/utcMinute-test.js:56:3
}

// d3-time/test/utcMinute-test.js: utcMinute.range(start, stop), has an exclusive upper bound
#[test]
fn utcminute_range_start_stop_has_an_exclusive_upper_bound() {
    assert_eq!(
        ts(Interval::Minute(1)
            .range(
                utc(&[2010.0, 11.0, 31.0, 23.0, 59.0]),
                utc(&[2011.0, 0.0, 1.0, 0.0, 2.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()[2_usize]),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 1.0]))
    ); // d3-time/test/utcMinute-test.js:60:3
}

// d3-time/test/utcMinute-test.js: utcMinute.range(start, stop), can skip minutes
#[test]
fn utcminute_range_start_stop_can_skip_minutes() {
    assert_eq!(
        tss(&Interval::Minute(1)
            .range_step(
                utc(&[2011.0, 1.0, 1.0, 12.0, 7.0]),
                utc(&[2011.0, 1.0, 1.0, 13.0, 7.0]),
                15_u32,
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 1.0, 1.0, 12.0, 7.0]),
            utc(&[2011.0, 1.0, 1.0, 12.0, 22.0]),
            utc(&[2011.0, 1.0, 1.0, 12.0, 37.0]),
            utc(&[2011.0, 1.0, 1.0, 12.0, 52.0])
        ])
    ); // d3-time/test/utcMinute-test.js:64:3
}

// d3-time/test/utcMinute-test.js: utcMinute.range(start, stop), observes start of daylight savings time
#[test]
fn utcminute_range_start_stop_observes_start_of_daylight_savings_time() {
    assert_eq!(
        tss(&Interval::Minute(1)
            .range(
                utc(&[2011.0, 2.0, 13.0, 9.0, 59.0]),
                utc(&[2011.0, 2.0, 13.0, 10.0, 2.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 2.0, 13.0, 9.0, 59.0]),
            utc(&[2011.0, 2.0, 13.0, 10.0, 0.0]),
            utc(&[2011.0, 2.0, 13.0, 10.0, 1.0])
        ])
    ); // d3-time/test/utcMinute-test.js:73:3
}

// d3-time/test/utcMinute-test.js: utcMinute.range(start, stop), observes end of daylight savings time
#[test]
fn utcminute_range_start_stop_observes_end_of_daylight_savings_time() {
    assert_eq!(
        tss(&Interval::Minute(1)
            .range(
                utc(&[2011.0, 10.0, 6.0, 8.0, 59.0]),
                utc(&[2011.0, 10.0, 6.0, 9.0, 2.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 10.0, 6.0, 8.0, 59.0]),
            utc(&[2011.0, 10.0, 6.0, 9.0, 0.0]),
            utc(&[2011.0, 10.0, 6.0, 9.0, 1.0])
        ])
    ); // d3-time/test/utcMinute-test.js:81:3
}

// d3-time/test/utcMinute-test.js: utcMinute.every(step) returns every stepth minute, starting with the first minute of the hour
#[test]
fn utcminute_every_step_returns_every_stepth_minute_starting_with_the_first_minute_of_the_hour() {
    assert_eq!(
        tss(&Interval::Minute(1)
            .every(15_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                utc(&[2008.0, 11.0, 30.0, 12.0, 47.0]),
                utc(&[2008.0, 11.0, 30.0, 13.0, 57.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2008.0, 11.0, 30.0, 13.0, 0.0]),
            utc(&[2008.0, 11.0, 30.0, 13.0, 15.0]),
            utc(&[2008.0, 11.0, 30.0, 13.0, 30.0]),
            utc(&[2008.0, 11.0, 30.0, 13.0, 45.0])
        ])
    ); // d3-time/test/utcMinute-test.js:89:3
    assert_eq!(
        tss(&Interval::Minute(1)
            .every(30_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                utc(&[2008.0, 11.0, 30.0, 12.0, 47.0]),
                utc(&[2008.0, 11.0, 30.0, 13.0, 57.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2008.0, 11.0, 30.0, 13.0, 0.0]),
            utc(&[2008.0, 11.0, 30.0, 13.0, 30.0])
        ])
    ); // d3-time/test/utcMinute-test.js:90:3
}

// d3-time/test/hour-test.js: timeHour.floor(date) returns hours
#[test]
fn timehour_floor_date_returns_hours() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Hour(1).floor(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0]))
    ); // d3-time/test/hour-test.js:6:3
    assert_eq!(
        ts(Interval::Hour(1).floor(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0]))
    ); // d3-time/test/hour-test.js:7:3
    assert_eq!(
        ts(Interval::Hour(1).floor(local(&la, &[2011.0, 0.0, 1.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0]))
    ); // d3-time/test/hour-test.js:8:3
}

// d3-time/test/hour-test.js: timeHour.floor(date) observes start of daylight savings time
#[test]
fn timehour_floor_date_observes_start_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 2.0, 13.0, 8.0, 59.0]), &la)),
        ts(utc(&[2011.0, 2.0, 13.0, 8.0]))
    ); // d3-time/test/hour-test.js:12:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 2.0, 13.0, 9.0, 0.0]), &la)),
        ts(utc(&[2011.0, 2.0, 13.0, 9.0]))
    ); // d3-time/test/hour-test.js:13:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 2.0, 13.0, 9.0, 1.0]), &la)),
        ts(utc(&[2011.0, 2.0, 13.0, 9.0]))
    ); // d3-time/test/hour-test.js:14:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 2.0, 13.0, 9.0, 59.0]), &la)),
        ts(utc(&[2011.0, 2.0, 13.0, 9.0]))
    ); // d3-time/test/hour-test.js:15:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 2.0, 13.0, 10.0, 0.0]), &la)),
        ts(utc(&[2011.0, 2.0, 13.0, 10.0]))
    ); // d3-time/test/hour-test.js:16:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 2.0, 13.0, 10.0, 1.0]), &la)),
        ts(utc(&[2011.0, 2.0, 13.0, 10.0]))
    ); // d3-time/test/hour-test.js:17:3
}

// d3-time/test/hour-test.js: timeHour.floor(date) observes end of daylight savings time
#[test]
fn timehour_floor_date_observes_end_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 10.0, 6.0, 7.0, 59.0]), &la)),
        ts(utc(&[2011.0, 10.0, 6.0, 7.0]))
    ); // d3-time/test/hour-test.js:21:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 10.0, 6.0, 8.0, 0.0]), &la)),
        ts(utc(&[2011.0, 10.0, 6.0, 8.0]))
    ); // d3-time/test/hour-test.js:22:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 10.0, 6.0, 8.0, 1.0]), &la)),
        ts(utc(&[2011.0, 10.0, 6.0, 8.0]))
    ); // d3-time/test/hour-test.js:23:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 10.0, 6.0, 8.0, 59.0]), &la)),
        ts(utc(&[2011.0, 10.0, 6.0, 8.0]))
    ); // d3-time/test/hour-test.js:24:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 10.0, 6.0, 9.0, 0.0]), &la)),
        ts(utc(&[2011.0, 10.0, 6.0, 9.0]))
    ); // d3-time/test/hour-test.js:25:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 10.0, 6.0, 9.0, 1.0]), &la)),
        ts(utc(&[2011.0, 10.0, 6.0, 9.0]))
    ); // d3-time/test/hour-test.js:26:3
}

// d3-time/test/hour-test.js: timeHour.ceil(date) returns hours
#[test]
fn timehour_ceil_date_returns_hours() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Hour(1).ceil(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0]))
    ); // d3-time/test/hour-test.js:30:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0]))
    ); // d3-time/test/hour-test.js:31:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(local(&la, &[2011.0, 0.0, 1.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 1.0]))
    ); // d3-time/test/hour-test.js:32:3
}

// d3-time/test/hour-test.js: timeHour.ceil(date) observes start of daylight savings time
#[test]
fn timehour_ceil_date_observes_start_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 2.0, 13.0, 8.0, 59.0]), &la)),
        ts(utc(&[2011.0, 2.0, 13.0, 9.0]))
    ); // d3-time/test/hour-test.js:36:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 2.0, 13.0, 9.0, 0.0]), &la)),
        ts(utc(&[2011.0, 2.0, 13.0, 9.0]))
    ); // d3-time/test/hour-test.js:37:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 2.0, 13.0, 9.0, 1.0]), &la)),
        ts(utc(&[2011.0, 2.0, 13.0, 10.0]))
    ); // d3-time/test/hour-test.js:38:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 2.0, 13.0, 9.0, 59.0]), &la)),
        ts(utc(&[2011.0, 2.0, 13.0, 10.0]))
    ); // d3-time/test/hour-test.js:39:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 2.0, 13.0, 10.0, 0.0]), &la)),
        ts(utc(&[2011.0, 2.0, 13.0, 10.0]))
    ); // d3-time/test/hour-test.js:40:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 2.0, 13.0, 10.0, 1.0]), &la)),
        ts(utc(&[2011.0, 2.0, 13.0, 11.0]))
    ); // d3-time/test/hour-test.js:41:3
}

// d3-time/test/hour-test.js: timeHour.ceil(date) observes end of daylight savings time
#[test]
fn timehour_ceil_date_observes_end_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 10.0, 6.0, 7.0, 59.0]), &la)),
        ts(utc(&[2011.0, 10.0, 6.0, 8.0]))
    ); // d3-time/test/hour-test.js:45:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 10.0, 6.0, 8.0, 0.0]), &la)),
        ts(utc(&[2011.0, 10.0, 6.0, 8.0]))
    ); // d3-time/test/hour-test.js:46:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 10.0, 6.0, 8.0, 1.0]), &la)),
        ts(utc(&[2011.0, 10.0, 6.0, 9.0]))
    ); // d3-time/test/hour-test.js:47:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 10.0, 6.0, 8.0, 59.0]), &la)),
        ts(utc(&[2011.0, 10.0, 6.0, 9.0]))
    ); // d3-time/test/hour-test.js:48:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 10.0, 6.0, 9.0, 0.0]), &la)),
        ts(utc(&[2011.0, 10.0, 6.0, 9.0]))
    ); // d3-time/test/hour-test.js:49:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 10.0, 6.0, 9.0, 1.0]), &la)),
        ts(utc(&[2011.0, 10.0, 6.0, 10.0]))
    ); // d3-time/test/hour-test.js:50:3
}

// d3-time/test/hour-test.js: timeHour.offset(date) does not modify the passed-in date
#[test]
fn timehour_offset_date_does_not_modify_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    let d = local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Hour(1).offset(d, 1_i64, &la);
    assert_eq!(
        ts(d),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/hour-test.js:56:3
}

// d3-time/test/hour-test.js: timeHour.offset(date) does not round the passed-in-date
#[test]
fn timehour_offset_date_does_not_round_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Hour(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/hour-test.js:60:3
    assert_eq!(
        ts(Interval::Hour(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 21.0, 59.0, 59.0, 456.0]))
    ); // d3-time/test/hour-test.js:61:3
}

// d3-time/test/hour-test.js: timeHour.offset(date) allows negative offsets
#[test]
fn timehour_offset_date_allows_negative_offsets() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Hour(1).offset(local(&la, &[2010.0, 11.0, 31.0, 12.0]), -1_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0, 11.0]))
    ); // d3-time/test/hour-test.js:65:3
    assert_eq!(
        ts(Interval::Hour(1).offset(local(&la, &[2011.0, 0.0, 1.0, 1.0]), -2_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0]))
    ); // d3-time/test/hour-test.js:66:3
    assert_eq!(
        ts(Interval::Hour(1).offset(local(&la, &[2011.0, 0.0, 1.0, 0.0]), -1_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0]))
    ); // d3-time/test/hour-test.js:67:3
}

// d3-time/test/hour-test.js: timeHour.offset(date) allows positive offsets
#[test]
fn timehour_offset_date_allows_positive_offsets() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Hour(1).offset(local(&la, &[2010.0, 11.0, 31.0, 11.0]), 1_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0, 12.0]))
    ); // d3-time/test/hour-test.js:71:3
    assert_eq!(
        ts(Interval::Hour(1).offset(local(&la, &[2010.0, 11.0, 31.0, 23.0]), 2_i64, &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 1.0]))
    ); // d3-time/test/hour-test.js:72:3
    assert_eq!(
        ts(Interval::Hour(1).offset(local(&la, &[2010.0, 11.0, 31.0, 23.0]), 1_i64, &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0, 0.0]))
    ); // d3-time/test/hour-test.js:73:3
}

// d3-time/test/hour-test.js: timeHour.offset(date) allows zero offset
#[test]
fn timehour_offset_date_allows_zero_offset() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Hour(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/hour-test.js:77:3
    assert_eq!(
        ts(Interval::Hour(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/hour-test.js:78:3
}

// d3-time/test/hour-test.js: timeHour.range(start, stop) returns hours
#[test]
fn timehour_range_start_stop_returns_hours() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Hour(1)
            .range(
                local(&la, &[2010.0, 11.0, 31.0, 12.0, 30.0]),
                local(&la, &[2010.0, 11.0, 31.0, 15.0, 30.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2010.0, 11.0, 31.0, 13.0]),
            local(&la, &[2010.0, 11.0, 31.0, 14.0]),
            local(&la, &[2010.0, 11.0, 31.0, 15.0])
        ])
    ); // d3-time/test/hour-test.js:82:3
}

// d3-time/test/hour-test.js: timeHour.range(start, stop) has an inclusive lower bound
#[test]
fn timehour_range_start_stop_has_an_inclusive_lower_bound() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Hour(1)
            .range(
                local(&la, &[2010.0, 11.0, 31.0, 23.0]),
                local(&la, &[2011.0, 0.0, 1.0, 2.0]),
                &la
            )
            .collect::<Vec<_>>()[0_usize]),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0]))
    ); // d3-time/test/hour-test.js:90:3
}

// d3-time/test/hour-test.js: timeHour.range(start, stop) has an exclusive upper bound
#[test]
fn timehour_range_start_stop_has_an_exclusive_upper_bound() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Hour(1)
            .range(
                local(&la, &[2010.0, 11.0, 31.0, 23.0]),
                local(&la, &[2011.0, 0.0, 1.0, 2.0]),
                &la
            )
            .collect::<Vec<_>>()[2_usize]),
        ts(local(&la, &[2011.0, 0.0, 1.0, 1.0]))
    ); // d3-time/test/hour-test.js:94:3
}

// d3-time/test/hour-test.js: timeHour.range(start, stop) can skip hours
#[test]
fn timehour_range_start_stop_can_skip_hours() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Hour(1)
            .range_step(
                local(&la, &[2011.0, 1.0, 1.0, 1.0]),
                local(&la, &[2011.0, 1.0, 1.0, 13.0]),
                3_u32,
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 1.0, 1.0, 1.0]),
            local(&la, &[2011.0, 1.0, 1.0, 4.0]),
            local(&la, &[2011.0, 1.0, 1.0, 7.0]),
            local(&la, &[2011.0, 1.0, 1.0, 10.0])
        ])
    ); // d3-time/test/hour-test.js:98:3
}

// d3-time/test/hour-test.js: timeHour.range(start, stop) observes start of daylight savings time
#[test]
fn timehour_range_start_stop_observes_start_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Hour(1)
            .range(
                local(&la, &[2011.0, 2.0, 13.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 5.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 2.0, 13.0, 9.0]),
            utc(&[2011.0, 2.0, 13.0, 10.0]),
            utc(&[2011.0, 2.0, 13.0, 11.0])
        ])
    ); // d3-time/test/hour-test.js:107:3
}

// d3-time/test/hour-test.js: timeHour.range(start, stop) observes end of daylight savings time
#[test]
fn timehour_range_start_stop_observes_end_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Hour(1)
            .range(
                local(&la, &[2011.0, 10.0, 6.0, 0.0]),
                local(&la, &[2011.0, 10.0, 6.0, 2.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 10.0, 6.0, 7.0]),
            utc(&[2011.0, 10.0, 6.0, 8.0]),
            utc(&[2011.0, 10.0, 6.0, 9.0])
        ])
    ); // d3-time/test/hour-test.js:115:3
}

// d3-time/test/hour-test.js: timeHour.every(step) returns every stepth hour, starting with the first hour of the day
#[test]
fn timehour_every_step_returns_every_stepth_hour_starting_with_the_first_hour_of_the_day() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Hour(1)
            .every(4_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                local(&la, &[2008.0, 11.0, 30.0, 12.0, 47.0]),
                local(&la, &[2008.0, 11.0, 31.0, 13.0, 57.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2008.0, 11.0, 30.0, 16.0]),
            local(&la, &[2008.0, 11.0, 30.0, 20.0]),
            local(&la, &[2008.0, 11.0, 31.0, 0.0]),
            local(&la, &[2008.0, 11.0, 31.0, 4.0]),
            local(&la, &[2008.0, 11.0, 31.0, 8.0]),
            local(&la, &[2008.0, 11.0, 31.0, 12.0])
        ])
    ); // d3-time/test/hour-test.js:123:3
    assert_eq!(
        tss(&Interval::Hour(1)
            .every(12_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                local(&la, &[2008.0, 11.0, 30.0, 12.0, 47.0]),
                local(&la, &[2008.0, 11.0, 31.0, 13.0, 57.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2008.0, 11.0, 31.0, 0.0]),
            local(&la, &[2008.0, 11.0, 31.0, 12.0])
        ])
    ); // d3-time/test/hour-test.js:124:3
}

// d3-time/test/hour-test.js: timeHour.range(start, stop) returns every hour crossing the daylight savings boundary
#[test]
fn timehour_range_start_stop_returns_every_hour_crossing_the_daylight_savings_boundary() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Hour(1)
            .range(
                millis(1478422800000.0 - (2.0 * 36e5)),
                millis(1478422800000.0 + (2.0 * 36e5)),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            millis(1478415600000.0),
            millis(1478419200000.0),
            millis(1478422800000.0),
            millis(1478426400000.0)
        ])
    ); // d3-time/test/hour-test.js:128:3
}

// d3-time/test/utcHour-test.js: utcHour.floor(date) returns hours
#[test]
fn utchour_floor_date_returns_hours() {
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0]))
    ); // d3-time/test/utcHour-test.js:6:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0]))
    ); // d3-time/test/utcHour-test.js:7:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 0.0, 1.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0]))
    ); // d3-time/test/utcHour-test.js:8:3
}

// d3-time/test/utcHour-test.js: utcHour.floor(date) observes start of daylight savings time
#[test]
fn utchour_floor_date_observes_start_of_daylight_savings_time() {
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 2.0, 13.0, 8.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0, 8.0]))
    ); // d3-time/test/utcHour-test.js:12:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 2.0, 13.0, 9.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0, 9.0]))
    ); // d3-time/test/utcHour-test.js:13:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 2.0, 13.0, 9.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0, 9.0]))
    ); // d3-time/test/utcHour-test.js:14:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 2.0, 13.0, 9.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0, 9.0]))
    ); // d3-time/test/utcHour-test.js:15:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 2.0, 13.0, 10.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0, 10.0]))
    ); // d3-time/test/utcHour-test.js:16:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 2.0, 13.0, 10.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0, 10.0]))
    ); // d3-time/test/utcHour-test.js:17:3
}

// d3-time/test/utcHour-test.js: utcHour.floor(date) observes end of daylight savings time
#[test]
fn utchour_floor_date_observes_end_of_daylight_savings_time() {
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 10.0, 6.0, 7.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0, 7.0]))
    ); // d3-time/test/utcHour-test.js:21:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 10.0, 6.0, 8.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0, 8.0]))
    ); // d3-time/test/utcHour-test.js:22:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 10.0, 6.0, 8.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0, 8.0]))
    ); // d3-time/test/utcHour-test.js:23:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 10.0, 6.0, 8.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0, 8.0]))
    ); // d3-time/test/utcHour-test.js:24:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 10.0, 6.0, 9.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0, 9.0]))
    ); // d3-time/test/utcHour-test.js:25:3
    assert_eq!(
        ts(Interval::Hour(1).floor(utc(&[2011.0, 10.0, 6.0, 9.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0, 9.0]))
    ); // d3-time/test/utcHour-test.js:26:3
}

// d3-time/test/utcHour-test.js: utcHour.ceil(date) returns hours
#[test]
fn utchour_ceil_date_returns_hours() {
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0]))
    ); // d3-time/test/utcHour-test.js:31:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0]))
    ); // d3-time/test/utcHour-test.js:32:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 0.0, 1.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 1.0]))
    ); // d3-time/test/utcHour-test.js:33:3
}

// d3-time/test/utcHour-test.js: utcHour.ceil(date) observes start of daylight savings time
#[test]
fn utchour_ceil_date_observes_start_of_daylight_savings_time() {
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 2.0, 13.0, 8.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0, 9.0]))
    ); // d3-time/test/utcHour-test.js:37:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 2.0, 13.0, 9.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0, 9.0]))
    ); // d3-time/test/utcHour-test.js:38:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 2.0, 13.0, 9.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0, 10.0]))
    ); // d3-time/test/utcHour-test.js:39:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 2.0, 13.0, 9.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0, 10.0]))
    ); // d3-time/test/utcHour-test.js:40:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 2.0, 13.0, 10.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0, 10.0]))
    ); // d3-time/test/utcHour-test.js:41:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 2.0, 13.0, 10.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0, 11.0]))
    ); // d3-time/test/utcHour-test.js:42:3
}

// d3-time/test/utcHour-test.js: utcHour.ceil(date) observes end of daylight savings time
#[test]
fn utchour_ceil_date_observes_end_of_daylight_savings_time() {
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 10.0, 6.0, 7.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0, 8.0]))
    ); // d3-time/test/utcHour-test.js:46:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 10.0, 6.0, 8.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0, 8.0]))
    ); // d3-time/test/utcHour-test.js:47:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 10.0, 6.0, 8.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0, 9.0]))
    ); // d3-time/test/utcHour-test.js:48:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 10.0, 6.0, 8.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0, 9.0]))
    ); // d3-time/test/utcHour-test.js:49:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 10.0, 6.0, 9.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0, 9.0]))
    ); // d3-time/test/utcHour-test.js:50:3
    assert_eq!(
        ts(Interval::Hour(1).ceil(utc(&[2011.0, 10.0, 6.0, 9.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0, 10.0]))
    ); // d3-time/test/utcHour-test.js:51:3
}

// d3-time/test/utcHour-test.js: utcHour.offset(date) does not modify the passed-in date
#[test]
fn utchour_offset_date_does_not_modify_the_passed_in_date() {
    let d = utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Hour(1).offset(d, 1_i64, &TimeZone::UTC);
    assert_eq!(
        ts(d),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcHour-test.js:57:3
}

// d3-time/test/utcHour-test.js: utcHour.offset(date) does not round the passed-in-date
#[test]
fn utchour_offset_date_does_not_round_the_passed_in_date() {
    assert_eq!(
        ts(Interval::Hour(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcHour-test.js:61:3
    assert_eq!(
        ts(Interval::Hour(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 21.0, 59.0, 59.0, 456.0]))
    ); // d3-time/test/utcHour-test.js:62:3
}

// d3-time/test/utcHour-test.js: utcHour.offset(date) allows negative offsets
#[test]
fn utchour_offset_date_allows_negative_offsets() {
    assert_eq!(
        ts(Interval::Hour(1).offset(utc(&[2010.0, 11.0, 31.0, 12.0]), -1_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0, 11.0]))
    ); // d3-time/test/utcHour-test.js:66:3
    assert_eq!(
        ts(Interval::Hour(1).offset(utc(&[2011.0, 0.0, 1.0, 1.0]), -2_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0]))
    ); // d3-time/test/utcHour-test.js:67:3
    assert_eq!(
        ts(Interval::Hour(1).offset(utc(&[2011.0, 0.0, 1.0, 0.0]), -1_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0]))
    ); // d3-time/test/utcHour-test.js:68:3
}

// d3-time/test/utcHour-test.js: utcHour.offset(date) allows positive offsets
#[test]
fn utchour_offset_date_allows_positive_offsets() {
    assert_eq!(
        ts(Interval::Hour(1).offset(utc(&[2010.0, 11.0, 31.0, 11.0]), 1_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0, 12.0]))
    ); // d3-time/test/utcHour-test.js:72:3
    assert_eq!(
        ts(Interval::Hour(1).offset(utc(&[2010.0, 11.0, 31.0, 23.0]), 2_i64, &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 1.0]))
    ); // d3-time/test/utcHour-test.js:73:3
    assert_eq!(
        ts(Interval::Hour(1).offset(utc(&[2010.0, 11.0, 31.0, 23.0]), 1_i64, &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0, 0.0]))
    ); // d3-time/test/utcHour-test.js:74:3
}

// d3-time/test/utcHour-test.js: utcHour.offset(date) allows zero offset
#[test]
fn utchour_offset_date_allows_zero_offset() {
    assert_eq!(
        ts(Interval::Hour(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcHour-test.js:78:3
    assert_eq!(
        ts(Interval::Hour(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/utcHour-test.js:79:3
}

// d3-time/test/utcHour-test.js: utcHour.range(start, stop) returns hours
#[test]
fn utchour_range_start_stop_returns_hours() {
    assert_eq!(
        tss(&Interval::Hour(1)
            .range(
                utc(&[2010.0, 11.0, 31.0, 12.0, 30.0]),
                utc(&[2010.0, 11.0, 31.0, 15.0, 30.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2010.0, 11.0, 31.0, 13.0]),
            utc(&[2010.0, 11.0, 31.0, 14.0]),
            utc(&[2010.0, 11.0, 31.0, 15.0])
        ])
    ); // d3-time/test/utcHour-test.js:83:3
}

// d3-time/test/utcHour-test.js: utcHour.range(start, stop) has an inclusive lower bound
#[test]
fn utchour_range_start_stop_has_an_inclusive_lower_bound() {
    assert_eq!(
        ts(Interval::Hour(1)
            .range(
                utc(&[2010.0, 11.0, 31.0, 23.0]),
                utc(&[2011.0, 0.0, 1.0, 2.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()[0_usize]),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0]))
    ); // d3-time/test/utcHour-test.js:91:3
}

// d3-time/test/utcHour-test.js: utcHour.range(start, stop) has an exclusive upper bound
#[test]
fn utchour_range_start_stop_has_an_exclusive_upper_bound() {
    assert_eq!(
        ts(Interval::Hour(1)
            .range(
                utc(&[2010.0, 11.0, 31.0, 23.0]),
                utc(&[2011.0, 0.0, 1.0, 2.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()[2_usize]),
        ts(utc(&[2011.0, 0.0, 1.0, 1.0]))
    ); // d3-time/test/utcHour-test.js:95:3
}

// d3-time/test/utcHour-test.js: utcHour.range(start, stop) can skip hours
#[test]
fn utchour_range_start_stop_can_skip_hours() {
    assert_eq!(
        tss(&Interval::Hour(1)
            .range_step(
                utc(&[2011.0, 1.0, 1.0, 1.0]),
                utc(&[2011.0, 1.0, 1.0, 13.0]),
                3_u32,
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 1.0, 1.0, 1.0]),
            utc(&[2011.0, 1.0, 1.0, 4.0]),
            utc(&[2011.0, 1.0, 1.0, 7.0]),
            utc(&[2011.0, 1.0, 1.0, 10.0])
        ])
    ); // d3-time/test/utcHour-test.js:99:3
}

// d3-time/test/utcHour-test.js: utcHour.range(start, stop) does not observe the start of daylight savings time
#[test]
fn utchour_range_start_stop_does_not_observe_the_start_of_daylight_savings_time() {
    assert_eq!(
        tss(&Interval::Hour(1)
            .range(
                utc(&[2011.0, 2.0, 13.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 5.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 2.0, 13.0, 1.0]),
            utc(&[2011.0, 2.0, 13.0, 2.0]),
            utc(&[2011.0, 2.0, 13.0, 3.0]),
            utc(&[2011.0, 2.0, 13.0, 4.0])
        ])
    ); // d3-time/test/utcHour-test.js:108:3
}

// d3-time/test/utcHour-test.js: utcHour.range(start, stop) does not observe the end of daylight savings time
#[test]
fn utchour_range_start_stop_does_not_observe_the_end_of_daylight_savings_time() {
    assert_eq!(
        tss(&Interval::Hour(1)
            .range(
                utc(&[2011.0, 10.0, 6.0, 0.0]),
                utc(&[2011.0, 10.0, 6.0, 2.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 10.0, 6.0, 0.0]),
            utc(&[2011.0, 10.0, 6.0, 1.0])
        ])
    ); // d3-time/test/utcHour-test.js:117:3
}

// d3-time/test/utcHour-test.js: utcHour.every(step) returns every stepth hour, starting with the first hour of the day
#[test]
fn utchour_every_step_returns_every_stepth_hour_starting_with_the_first_hour_of_the_day() {
    assert_eq!(
        tss(&Interval::Hour(1)
            .every(4_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                utc(&[2008.0, 11.0, 30.0, 12.0, 47.0]),
                utc(&[2008.0, 11.0, 31.0, 13.0, 57.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2008.0, 11.0, 30.0, 16.0]),
            utc(&[2008.0, 11.0, 30.0, 20.0]),
            utc(&[2008.0, 11.0, 31.0, 0.0]),
            utc(&[2008.0, 11.0, 31.0, 4.0]),
            utc(&[2008.0, 11.0, 31.0, 8.0]),
            utc(&[2008.0, 11.0, 31.0, 12.0])
        ])
    ); // d3-time/test/utcHour-test.js:124:3
    assert_eq!(
        tss(&Interval::Hour(1)
            .every(12_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                utc(&[2008.0, 11.0, 30.0, 12.0, 47.0]),
                utc(&[2008.0, 11.0, 31.0, 13.0, 57.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2008.0, 11.0, 31.0, 0.0]),
            utc(&[2008.0, 11.0, 31.0, 12.0])
        ])
    ); // d3-time/test/utcHour-test.js:125:3
}

// d3-time/test/day-test.js: timeDay.floor(date) returns days
#[test]
fn timeday_floor_date_returns_days() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Day(1).floor(local(&la, &[2010.0, 11.0, 31.0, 23.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0]))
    ); // d3-time/test/day-test.js:20:3
    assert_eq!(
        ts(Interval::Day(1).floor(local(&la, &[2011.0, 0.0, 1.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/day-test.js:21:3
    assert_eq!(
        ts(Interval::Day(1).floor(local(&la, &[2011.0, 0.0, 1.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/day-test.js:22:3
}

// d3-time/test/day-test.js: timeDay.floor(date) observes daylight saving
#[test]
fn timeday_floor_date_observes_daylight_saving() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 2.0, 13.0, 7.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 12.0]))
    ); // d3-time/test/day-test.js:26:3
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 2.0, 13.0, 8.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 13.0]))
    ); // d3-time/test/day-test.js:27:3
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 2.0, 13.0, 9.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 13.0]))
    ); // d3-time/test/day-test.js:28:3
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 2.0, 13.0, 10.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 13.0]))
    ); // d3-time/test/day-test.js:29:3
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 10.0, 6.0, 7.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 6.0]))
    ); // d3-time/test/day-test.js:30:3
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 10.0, 6.0, 8.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 6.0]))
    ); // d3-time/test/day-test.js:31:3
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 10.0, 6.0, 9.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 6.0]))
    ); // d3-time/test/day-test.js:32:3
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 10.0, 6.0, 10.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 6.0]))
    ); // d3-time/test/day-test.js:33:3
}

// d3-time/test/day-test.js: timeDay.floor(date) handles years in the first century
#[test]
fn timeday_floor_date_handles_years_in_the_first_century() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Day(1).floor(local(&la, &[9.0, 10.0, 6.0, 7.0]), &la)),
        ts(local(&la, &[9.0, 10.0, 6.0]))
    ); // d3-time/test/day-test.js:37:3
}

// d3-time/test/day-test.js: timeDay.round(date) returns days
#[test]
fn timeday_round_date_returns_days() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Day(1).round(local(&la, &[2010.0, 11.0, 30.0, 13.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0]))
    ); // d3-time/test/day-test.js:41:3
    assert_eq!(
        ts(Interval::Day(1).round(local(&la, &[2010.0, 11.0, 30.0, 11.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 30.0]))
    ); // d3-time/test/day-test.js:42:3
}

// d3-time/test/day-test.js: timeDay.round(date) observes daylight saving
#[test]
fn timeday_round_date_observes_daylight_saving() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Day(1).round(utc(&[2011.0, 2.0, 13.0, 7.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 13.0]))
    ); // d3-time/test/day-test.js:46:3
    assert_eq!(
        ts(Interval::Day(1).round(utc(&[2011.0, 2.0, 13.0, 8.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 13.0]))
    ); // d3-time/test/day-test.js:47:3
    assert_eq!(
        ts(Interval::Day(1).round(utc(&[2011.0, 2.0, 13.0, 9.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 13.0]))
    ); // d3-time/test/day-test.js:48:3
    assert_eq!(
        ts(Interval::Day(1).round(utc(&[2011.0, 2.0, 13.0, 20.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 14.0]))
    ); // d3-time/test/day-test.js:49:3
    assert_eq!(
        ts(Interval::Day(1).round(utc(&[2011.0, 10.0, 6.0, 7.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 6.0]))
    ); // d3-time/test/day-test.js:50:3
    assert_eq!(
        ts(Interval::Day(1).round(utc(&[2011.0, 10.0, 6.0, 8.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 6.0]))
    ); // d3-time/test/day-test.js:51:3
    assert_eq!(
        ts(Interval::Day(1).round(utc(&[2011.0, 10.0, 6.0, 9.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 6.0]))
    ); // d3-time/test/day-test.js:52:3
    assert_eq!(
        ts(Interval::Day(1).round(utc(&[2011.0, 10.0, 6.0, 20.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 7.0]))
    ); // d3-time/test/day-test.js:53:3
}

// d3-time/test/day-test.js: timeDay.round(date) handles midnight in leap years
#[test]
fn timeday_round_date_handles_midnight_in_leap_years() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Day(1).round(utc(&[2012.0, 2.0, 1.0, 0.0]), &la)),
        ts(local(&la, &[2012.0, 2.0, 1.0]))
    ); // d3-time/test/day-test.js:57:3
    assert_eq!(
        ts(Interval::Day(1).round(utc(&[2012.0, 2.0, 1.0, 0.0]), &la)),
        ts(local(&la, &[2012.0, 2.0, 1.0]))
    ); // d3-time/test/day-test.js:58:3
}

// d3-time/test/day-test.js: timeDay.ceil(date) returns days
#[test]
fn timeday_ceil_date_returns_days() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Day(1).ceil(local(&la, &[2010.0, 11.0, 30.0, 23.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0]))
    ); // d3-time/test/day-test.js:62:3
    assert_eq!(
        ts(Interval::Day(1).ceil(local(&la, &[2010.0, 11.0, 31.0, 0.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0]))
    ); // d3-time/test/day-test.js:63:3
    assert_eq!(
        ts(Interval::Day(1).ceil(local(&la, &[2010.0, 11.0, 31.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/day-test.js:64:3
}

// d3-time/test/day-test.js: timeDay.ceil(date) observes start of daylight saving
#[test]
fn timeday_ceil_date_observes_start_of_daylight_saving() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2011.0, 2.0, 13.0, 7.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 13.0]))
    ); // d3-time/test/day-test.js:68:3
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2011.0, 2.0, 13.0, 8.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 13.0]))
    ); // d3-time/test/day-test.js:69:3
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2011.0, 2.0, 13.0, 9.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 14.0]))
    ); // d3-time/test/day-test.js:70:3
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2011.0, 2.0, 13.0, 10.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 14.0]))
    ); // d3-time/test/day-test.js:71:3
}

// d3-time/test/day-test.js: timeDay.ceil(date) observes end of daylight saving
#[test]
fn timeday_ceil_date_observes_end_of_daylight_saving() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2011.0, 10.0, 6.0, 7.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 6.0]))
    ); // d3-time/test/day-test.js:75:3
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2011.0, 10.0, 6.0, 8.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 7.0]))
    ); // d3-time/test/day-test.js:76:3
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2011.0, 10.0, 6.0, 9.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 7.0]))
    ); // d3-time/test/day-test.js:77:3
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2011.0, 10.0, 6.0, 10.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 7.0]))
    ); // d3-time/test/day-test.js:78:3
}

// d3-time/test/day-test.js: timeDay.ceil(date) handles midnight for leap years
#[test]
fn timeday_ceil_date_handles_midnight_for_leap_years() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2012.0, 2.0, 1.0, 0.0]), &la)),
        ts(local(&la, &[2012.0, 2.0, 1.0]))
    ); // d3-time/test/day-test.js:82:3
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2012.0, 2.0, 1.0, 0.0]), &la)),
        ts(local(&la, &[2012.0, 2.0, 1.0]))
    ); // d3-time/test/day-test.js:83:3
}

// d3-time/test/day-test.js: timeDay.offset(date) is an alias for timeDay.offset(date, 1)
#[test]
fn timeday_offset_date_is_an_alias_for_timeday_offset_date_1() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Day(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 1.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/day-test.js:87:3
}

// d3-time/test/day-test.js: timeDay.offset(date, step) does not modify the passed-in date
#[test]
fn timeday_offset_date_step_does_not_modify_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    let d = local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Day(1).offset(d, 1_i64, &la);
    assert_eq!(
        ts(d),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/day-test.js:93:3
}

// d3-time/test/day-test.js: timeDay.offset(date, step) does not round the passed-in date
#[test]
fn timeday_offset_date_step_does_not_round_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Day(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 1.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/day-test.js:97:3
    assert_eq!(
        ts(Interval::Day(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 29.0, 23.0, 59.0, 59.0, 456.0]))
    ); // d3-time/test/day-test.js:98:3
}

// d3-time/test/day-test.js: timeDay.offset(date, step) allows step to be negative
#[test]
fn timeday_offset_date_step_allows_step_to_be_negative() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Day(1).offset(local(&la, &[2010.0, 11.0, 31.0]), -1_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 30.0]))
    ); // d3-time/test/day-test.js:102:3
    assert_eq!(
        ts(Interval::Day(1).offset(local(&la, &[2011.0, 0.0, 1.0]), -2_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 30.0]))
    ); // d3-time/test/day-test.js:103:3
    assert_eq!(
        ts(Interval::Day(1).offset(local(&la, &[2011.0, 0.0, 1.0]), -1_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0]))
    ); // d3-time/test/day-test.js:104:3
}

// d3-time/test/day-test.js: timeDay.offset(date, step) allows step to be positive
#[test]
fn timeday_offset_date_step_allows_step_to_be_positive() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Day(1).offset(local(&la, &[2010.0, 11.0, 31.0]), 1_i64, &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/day-test.js:108:3
    assert_eq!(
        ts(Interval::Day(1).offset(local(&la, &[2010.0, 11.0, 30.0]), 2_i64, &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/day-test.js:109:3
    assert_eq!(
        ts(Interval::Day(1).offset(local(&la, &[2010.0, 11.0, 30.0]), 1_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0]))
    ); // d3-time/test/day-test.js:110:3
}

// d3-time/test/day-test.js: timeDay.offset(date, step) allows step to be zero
#[test]
fn timeday_offset_date_step_allows_step_to_be_zero() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Day(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/day-test.js:114:3
    assert_eq!(
        ts(Interval::Day(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/day-test.js:115:3
}

// d3-time/test/day-test.js: timeDay.range(start, stop) returns days between start (inclusive) and stop (exclusive)
#[test]
fn timeday_range_start_stop_returns_days_between_start_inclusive_and_stop_exclusive() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Day(1)
            .range(
                local(&la, &[2011.0, 10.0, 4.0]),
                local(&la, &[2011.0, 10.0, 10.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 10.0, 4.0]),
            local(&la, &[2011.0, 10.0, 5.0]),
            local(&la, &[2011.0, 10.0, 6.0]),
            local(&la, &[2011.0, 10.0, 7.0]),
            local(&la, &[2011.0, 10.0, 8.0]),
            local(&la, &[2011.0, 10.0, 9.0])
        ])
    ); // d3-time/test/day-test.js:119:3
}

// d3-time/test/day-test.js: timeDay.range(start, stop) returns days
#[test]
fn timeday_range_start_stop_returns_days() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Day(1)
            .range(
                local(&la, &[2011.0, 10.0, 4.0, 2.0]),
                local(&la, &[2011.0, 10.0, 10.0, 13.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 10.0, 5.0]),
            local(&la, &[2011.0, 10.0, 6.0]),
            local(&la, &[2011.0, 10.0, 7.0]),
            local(&la, &[2011.0, 10.0, 8.0]),
            local(&la, &[2011.0, 10.0, 9.0]),
            local(&la, &[2011.0, 10.0, 10.0])
        ])
    ); // d3-time/test/day-test.js:130:3
}

// d3-time/test/day-test.js: timeDay.range(start, stop) returns the empty array if start >= stop
#[test]
fn timeday_range_start_stop_returns_the_empty_array_if_start_stop() {
    let la = zone("America/Los_Angeles");
    assert_d3_eq(
        &tss(&Interval::Day(1)
            .range(
                local(&la, &[2011.0, 10.0, 10.0]),
                local(&la, &[2011.0, 10.0, 4.0]),
                &la,
            )
            .collect::<Vec<_>>()),
        &Vec::<f64>::new(),
    ); // d3-time/test/day-test.js:153:3
    assert_d3_eq(
        &tss(&Interval::Day(1)
            .range(
                local(&la, &[2011.0, 10.0, 10.0]),
                local(&la, &[2011.0, 10.0, 10.0]),
                &la,
            )
            .collect::<Vec<_>>()),
        &Vec::<f64>::new(),
    ); // d3-time/test/day-test.js:154:3
}

// d3-time/test/day-test.js: timeDay.range(start, stop, step) returns every step day
#[test]
fn timeday_range_start_stop_step_returns_every_step_day() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Day(1)
            .range_step(
                local(&la, &[2011.0, 10.0, 4.0, 2.0]),
                local(&la, &[2011.0, 10.0, 14.0, 13.0]),
                3_u32,
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 10.0, 5.0]),
            local(&la, &[2011.0, 10.0, 8.0]),
            local(&la, &[2011.0, 10.0, 11.0]),
            local(&la, &[2011.0, 10.0, 14.0])
        ])
    ); // d3-time/test/day-test.js:158:3
}

// d3-time/test/day-test.js: timeDay.range(start, stop, step) returns the empty array if step is zero, negative or NaN
#[test]
fn timeday_range_start_stop_step_returns_the_empty_array_if_step_is_zero_negative_or_nan() {
    let la = zone("America/Los_Angeles");
    assert_d3_eq(
        &tss(&Interval::Day(1)
            .range_step(
                local(&la, &[2011.0, 0.0, 1.0, 0.0]),
                local(&la, &[2011.0, 4.0, 9.0, 0.0]),
                0_u32,
                &la,
            )
            .collect::<Vec<_>>()),
        &Vec::<f64>::new(),
    ); // d3-time/test/day-test.js:167:3
}

// d3-time/test/day-test.js: timeDay.count(start, end) counts days after start (exclusive) and before end (inclusive)
#[test]
fn timeday_count_start_end_counts_days_after_start_exclusive_and_before_end_inclusive() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0, 0.0]),
                local(&la, &[2011.0, 4.0, 9.0, 0.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        128.0
    ); // d3-time/test/day-test.js:174:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0, 1.0]),
                local(&la, &[2011.0, 4.0, 9.0, 0.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        128.0
    ); // d3-time/test/day-test.js:175:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2010.0, 11.0, 31.0, 23.0]),
                local(&la, &[2011.0, 4.0, 9.0, 0.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        129.0
    ); // d3-time/test/day-test.js:176:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0, 0.0]),
                local(&la, &[2011.0, 4.0, 8.0, 23.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        127.0
    ); // d3-time/test/day-test.js:177:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0, 0.0]),
                local(&la, &[2011.0, 4.0, 9.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        128.0
    ); // d3-time/test/day-test.js:178:3
}

// d3-time/test/day-test.js: timeDay.count(start, end) observes daylight saving
#[test]
fn timeday_count_start_end_observes_daylight_saving() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        71.0
    ); // d3-time/test/day-test.js:182:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 3.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        71.0
    ); // d3-time/test/day-test.js:183:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 4.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        71.0
    ); // d3-time/test/day-test.js:184:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 0.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        309.0
    ); // d3-time/test/day-test.js:185:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        309.0
    ); // d3-time/test/day-test.js:186:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 2.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        309.0
    ); // d3-time/test/day-test.js:187:3
}

// d3-time/test/day-test.js: timeDay.count(start, stop) does not exhibit floating-point rounding error
#[test]
fn timeday_count_start_stop_does_not_exhibit_floating_point_rounding_error() {
    let la = zone("America/Los_Angeles");
    let date = local(&la, &[2011.0, 4.0, 9.0]);
    assert_eq!(
        Interval::Day(1)
            .count(
                Interval::Year(1)
                    .floor(date, &la)
                    .expect("d3 test: a valid date"),
                date,
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        128.0
    ); // d3-time/test/day-test.js:192:3
}

// d3-time/test/day-test.js: timeDay.count(start, end) returns 364 or 365 for a full year
#[test]
fn timeday_count_start_end_returns_364_or_365_for_a_full_year() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[1999.0, 0.0, 1.0]),
                local(&la, &[1999.0, 11.0, 31.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/day-test.js:196:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2000.0, 0.0, 1.0]),
                local(&la, &[2000.0, 11.0, 31.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        365.0
    ); // d3-time/test/day-test.js:197:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2001.0, 0.0, 1.0]),
                local(&la, &[2001.0, 11.0, 31.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/day-test.js:198:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2002.0, 0.0, 1.0]),
                local(&la, &[2002.0, 11.0, 31.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/day-test.js:199:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2003.0, 0.0, 1.0]),
                local(&la, &[2003.0, 11.0, 31.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/day-test.js:200:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2004.0, 0.0, 1.0]),
                local(&la, &[2004.0, 11.0, 31.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        365.0
    ); // d3-time/test/day-test.js:201:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2005.0, 0.0, 1.0]),
                local(&la, &[2005.0, 11.0, 31.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/day-test.js:202:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2006.0, 0.0, 1.0]),
                local(&la, &[2006.0, 11.0, 31.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/day-test.js:203:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2007.0, 0.0, 1.0]),
                local(&la, &[2007.0, 11.0, 31.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/day-test.js:204:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2008.0, 0.0, 1.0]),
                local(&la, &[2008.0, 11.0, 31.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        365.0
    ); // d3-time/test/day-test.js:205:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2009.0, 0.0, 1.0]),
                local(&la, &[2009.0, 11.0, 31.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/day-test.js:206:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2010.0, 0.0, 1.0]),
                local(&la, &[2010.0, 11.0, 31.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/day-test.js:207:3
    assert_eq!(
        Interval::Day(1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 11.0, 31.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/day-test.js:208:3
}

// d3-time/test/day-test.js: timeDay.every(step) returns every stepth day, starting with the first day of the month
#[test]
fn timeday_every_step_returns_every_stepth_day_starting_with_the_first_day_of_the_month() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Day(1)
            .every(3_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                local(&la, &[2008.0, 11.0, 30.0, 0.0, 12.0]),
                local(&la, &[2009.0, 0.0, 5.0, 23.0, 48.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2008.0, 11.0, 31.0]),
            local(&la, &[2009.0, 0.0, 1.0]),
            local(&la, &[2009.0, 0.0, 4.0])
        ])
    ); // d3-time/test/day-test.js:212:3
    assert_eq!(
        tss(&Interval::Day(1)
            .every(5_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                local(&la, &[2008.0, 11.0, 30.0, 0.0, 12.0]),
                local(&la, &[2009.0, 0.0, 6.0, 23.0, 48.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2008.0, 11.0, 31.0]),
            local(&la, &[2009.0, 0.0, 1.0]),
            local(&la, &[2009.0, 0.0, 6.0])
        ])
    ); // d3-time/test/day-test.js:213:3
    assert_eq!(
        tss(&Interval::Day(1)
            .every(7_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                local(&la, &[2008.0, 11.0, 30.0, 0.0, 12.0]),
                local(&la, &[2009.0, 0.0, 8.0, 23.0, 48.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2009.0, 0.0, 1.0]),
            local(&la, &[2009.0, 0.0, 8.0])
        ])
    ); // d3-time/test/day-test.js:214:3
}

// d3-time/test/utcDay-test.js: utcDay.floor(date) returns days
#[test]
fn utcday_floor_date_returns_days() {
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2010.0, 11.0, 31.0, 23.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0]))
    ); // d3-time/test/utcDay-test.js:10:3
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 0.0, 1.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcDay-test.js:11:3
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 0.0, 1.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcDay-test.js:12:3
}

// d3-time/test/utcDay-test.js: utcDay.floor(date) does not observe daylight saving
#[test]
fn utcday_floor_date_does_not_observe_daylight_saving() {
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 2.0, 13.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0]))
    ); // d3-time/test/utcDay-test.js:16:3
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 2.0, 13.0, 8.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0]))
    ); // d3-time/test/utcDay-test.js:17:3
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 2.0, 13.0, 9.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0]))
    ); // d3-time/test/utcDay-test.js:18:3
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 2.0, 13.0, 10.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0]))
    ); // d3-time/test/utcDay-test.js:19:3
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 10.0, 6.0, 5.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0]))
    ); // d3-time/test/utcDay-test.js:20:3
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 10.0, 6.0, 6.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0]))
    ); // d3-time/test/utcDay-test.js:21:3
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 10.0, 6.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0]))
    ); // d3-time/test/utcDay-test.js:22:3
    assert_eq!(
        ts(Interval::Day(1).floor(utc(&[2011.0, 10.0, 6.0, 8.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0]))
    ); // d3-time/test/utcDay-test.js:23:3
}

// d3-time/test/utcDay-test.js: utcDay.round(date) returns days
#[test]
fn utcday_round_date_returns_days() {
    assert_eq!(
        ts(Interval::Day(1).round(utc(&[2010.0, 11.0, 30.0, 13.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0]))
    ); // d3-time/test/utcDay-test.js:27:3
    assert_eq!(
        ts(Interval::Day(1).round(utc(&[2010.0, 11.0, 30.0, 11.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 30.0]))
    ); // d3-time/test/utcDay-test.js:28:3
}

// d3-time/test/utcDay-test.js: utcDay.ceil(date) returns days
#[test]
fn utcday_ceil_date_returns_days() {
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2010.0, 11.0, 30.0, 23.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0]))
    ); // d3-time/test/utcDay-test.js:32:3
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2010.0, 11.0, 31.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0]))
    ); // d3-time/test/utcDay-test.js:33:3
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2010.0, 11.0, 31.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcDay-test.js:34:3
}

// d3-time/test/utcDay-test.js: utcDay.ceil(date) does not observe daylight saving
#[test]
fn utcday_ceil_date_does_not_observe_daylight_saving() {
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2011.0, 2.0, 13.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 14.0]))
    ); // d3-time/test/utcDay-test.js:38:3
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2011.0, 2.0, 13.0, 8.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 14.0]))
    ); // d3-time/test/utcDay-test.js:39:3
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2011.0, 2.0, 13.0, 9.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 14.0]))
    ); // d3-time/test/utcDay-test.js:40:3
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2011.0, 2.0, 13.0, 10.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 14.0]))
    ); // d3-time/test/utcDay-test.js:41:3
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2011.0, 10.0, 6.0, 5.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 7.0]))
    ); // d3-time/test/utcDay-test.js:42:3
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2011.0, 10.0, 6.0, 6.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 7.0]))
    ); // d3-time/test/utcDay-test.js:43:3
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2011.0, 10.0, 6.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 7.0]))
    ); // d3-time/test/utcDay-test.js:44:3
    assert_eq!(
        ts(Interval::Day(1).ceil(utc(&[2011.0, 10.0, 6.0, 8.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 7.0]))
    ); // d3-time/test/utcDay-test.js:45:3
}

// d3-time/test/utcDay-test.js: utcDay.offset(date) is an alias for utcDay.offset(date, 1)
#[test]
fn utcday_offset_date_is_an_alias_for_utcday_offset_date_1() {
    assert_eq!(
        ts(Interval::Day(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcDay-test.js:49:3
}

// d3-time/test/utcDay-test.js: utcDay.offset(date, step) does not modify the passed-in date
#[test]
fn utcday_offset_date_step_does_not_modify_the_passed_in_date() {
    let d = utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Day(1).offset(d, 1_i64, &TimeZone::UTC);
    assert_eq!(
        ts(d),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcDay-test.js:55:3
}

// d3-time/test/utcDay-test.js: utcDay.offset(date, step) does not round the passed-in date
#[test]
fn utcday_offset_date_step_does_not_round_the_passed_in_date() {
    assert_eq!(
        ts(Interval::Day(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcDay-test.js:59:3
    assert_eq!(
        ts(Interval::Day(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 29.0, 23.0, 59.0, 59.0, 456.0]))
    ); // d3-time/test/utcDay-test.js:60:3
}

// d3-time/test/utcDay-test.js: utcDay.offset(date, step) allows step to be negative
#[test]
fn utcday_offset_date_step_allows_step_to_be_negative() {
    assert_eq!(
        ts(Interval::Day(1).offset(utc(&[2010.0, 11.0, 31.0]), -1_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 30.0]))
    ); // d3-time/test/utcDay-test.js:64:3
    assert_eq!(
        ts(Interval::Day(1).offset(utc(&[2011.0, 0.0, 1.0]), -2_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 30.0]))
    ); // d3-time/test/utcDay-test.js:65:3
    assert_eq!(
        ts(Interval::Day(1).offset(utc(&[2011.0, 0.0, 1.0]), -1_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0]))
    ); // d3-time/test/utcDay-test.js:66:3
}

// d3-time/test/utcDay-test.js: utcDay.offset(date, step) allows step to be positive
#[test]
fn utcday_offset_date_step_allows_step_to_be_positive() {
    assert_eq!(
        ts(Interval::Day(1).offset(utc(&[2010.0, 11.0, 31.0]), 1_i64, &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcDay-test.js:70:3
    assert_eq!(
        ts(Interval::Day(1).offset(utc(&[2010.0, 11.0, 30.0]), 2_i64, &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcDay-test.js:71:3
    assert_eq!(
        ts(Interval::Day(1).offset(utc(&[2010.0, 11.0, 30.0]), 1_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0]))
    ); // d3-time/test/utcDay-test.js:72:3
}

// d3-time/test/utcDay-test.js: utcDay.offset(date, step) allows step to be zero
#[test]
fn utcday_offset_date_step_allows_step_to_be_zero() {
    assert_eq!(
        ts(Interval::Day(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcDay-test.js:76:3
    assert_eq!(
        ts(Interval::Day(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/utcDay-test.js:77:3
}

// d3-time/test/utcDay-test.js: utcDay.count(start, end) counts days after start (exclusive) and before end (inclusive)
#[test]
fn utcday_count_start_end_counts_days_after_start_exclusive_and_before_end_inclusive() {
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2011.0, 0.0, 1.0, 0.0]),
                utc(&[2011.0, 4.0, 9.0, 0.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        128.0
    ); // d3-time/test/utcDay-test.js:81:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2011.0, 0.0, 1.0, 1.0]),
                utc(&[2011.0, 4.0, 9.0, 0.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        128.0
    ); // d3-time/test/utcDay-test.js:82:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2010.0, 11.0, 31.0, 23.0]),
                utc(&[2011.0, 4.0, 9.0, 0.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        129.0
    ); // d3-time/test/utcDay-test.js:83:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2011.0, 0.0, 1.0, 0.0]),
                utc(&[2011.0, 4.0, 8.0, 23.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        127.0
    ); // d3-time/test/utcDay-test.js:84:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2011.0, 0.0, 1.0, 0.0]),
                utc(&[2011.0, 4.0, 9.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        128.0
    ); // d3-time/test/utcDay-test.js:85:3
}

// d3-time/test/utcDay-test.js: utcDay.count(start, end) does not observe daylight saving
#[test]
fn utcday_count_start_end_does_not_observe_daylight_saving() {
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        71.0
    ); // d3-time/test/utcDay-test.js:89:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 3.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        71.0
    ); // d3-time/test/utcDay-test.js:90:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 4.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        71.0
    ); // d3-time/test/utcDay-test.js:91:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 0.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        309.0
    ); // d3-time/test/utcDay-test.js:92:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        309.0
    ); // d3-time/test/utcDay-test.js:93:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 2.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        309.0
    ); // d3-time/test/utcDay-test.js:94:3
}

// d3-time/test/utcDay-test.js: utcDay.count(start, end) returns 364 or 365 for a full year
#[test]
fn utcday_count_start_end_returns_364_or_365_for_a_full_year() {
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[1999.0, 0.0, 1.0]),
                utc(&[1999.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/utcDay-test.js:98:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2000.0, 0.0, 1.0]),
                utc(&[2000.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        365.0
    ); // d3-time/test/utcDay-test.js:99:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2001.0, 0.0, 1.0]),
                utc(&[2001.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/utcDay-test.js:100:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2002.0, 0.0, 1.0]),
                utc(&[2002.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/utcDay-test.js:101:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2003.0, 0.0, 1.0]),
                utc(&[2003.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/utcDay-test.js:102:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2004.0, 0.0, 1.0]),
                utc(&[2004.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        365.0
    ); // d3-time/test/utcDay-test.js:103:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2005.0, 0.0, 1.0]),
                utc(&[2005.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/utcDay-test.js:104:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2006.0, 0.0, 1.0]),
                utc(&[2006.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/utcDay-test.js:105:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2007.0, 0.0, 1.0]),
                utc(&[2007.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/utcDay-test.js:106:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2008.0, 0.0, 1.0]),
                utc(&[2008.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        365.0
    ); // d3-time/test/utcDay-test.js:107:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2009.0, 0.0, 1.0]),
                utc(&[2009.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/utcDay-test.js:108:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2010.0, 0.0, 1.0]),
                utc(&[2010.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/utcDay-test.js:109:3
    assert_eq!(
        Interval::Day(1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/utcDay-test.js:110:3
}

// d3-time/test/utcDay-test.js: utcDay.every(step) returns every stepth day, starting with the first day of the month
#[test]
fn utcday_every_step_returns_every_stepth_day_starting_with_the_first_day_of_the_month() {
    assert_eq!(
        tss(&Interval::Day(1)
            .every(3_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                utc(&[2008.0, 11.0, 30.0, 0.0, 12.0]),
                utc(&[2009.0, 0.0, 5.0, 23.0, 48.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2008.0, 11.0, 31.0]),
            utc(&[2009.0, 0.0, 1.0]),
            utc(&[2009.0, 0.0, 4.0])
        ])
    ); // d3-time/test/utcDay-test.js:114:3
    assert_eq!(
        tss(&Interval::Day(1)
            .every(5_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                utc(&[2008.0, 11.0, 30.0, 0.0, 12.0]),
                utc(&[2009.0, 0.0, 6.0, 23.0, 48.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2008.0, 11.0, 31.0]),
            utc(&[2009.0, 0.0, 1.0]),
            utc(&[2009.0, 0.0, 6.0])
        ])
    ); // d3-time/test/utcDay-test.js:115:3
    assert_eq!(
        tss(&Interval::Day(1)
            .every(7_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                utc(&[2008.0, 11.0, 30.0, 0.0, 12.0]),
                utc(&[2009.0, 0.0, 8.0, 23.0, 48.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[utc(&[2009.0, 0.0, 1.0]), utc(&[2009.0, 0.0, 8.0])])
    ); // d3-time/test/utcDay-test.js:116:3
}

// d3-time/test/unixDay-test.js: unixDay.floor(date) returns days
#[test]
fn unixday_floor_date_returns_days() {
    assert_eq!(
        ts(Interval::UnixDay(1).floor(utc(&[2010.0, 11.0, 31.0, 23.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0]))
    ); // d3-time/test/unixDay-test.js:10:3
    assert_eq!(
        ts(Interval::UnixDay(1).floor(utc(&[2011.0, 0.0, 1.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/unixDay-test.js:11:3
    assert_eq!(
        ts(Interval::UnixDay(1).floor(utc(&[2011.0, 0.0, 1.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/unixDay-test.js:12:3
}

// d3-time/test/unixDay-test.js: unixDay.floor(date) does not observe daylight saving
#[test]
fn unixday_floor_date_does_not_observe_daylight_saving() {
    assert_eq!(
        ts(Interval::UnixDay(1).floor(utc(&[2011.0, 2.0, 13.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0]))
    ); // d3-time/test/unixDay-test.js:16:3
    assert_eq!(
        ts(Interval::UnixDay(1).floor(utc(&[2011.0, 2.0, 13.0, 8.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0]))
    ); // d3-time/test/unixDay-test.js:17:3
    assert_eq!(
        ts(Interval::UnixDay(1).floor(utc(&[2011.0, 2.0, 13.0, 9.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0]))
    ); // d3-time/test/unixDay-test.js:18:3
    assert_eq!(
        ts(Interval::UnixDay(1).floor(utc(&[2011.0, 2.0, 13.0, 10.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0]))
    ); // d3-time/test/unixDay-test.js:19:3
    assert_eq!(
        ts(Interval::UnixDay(1).floor(utc(&[2011.0, 10.0, 6.0, 5.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0]))
    ); // d3-time/test/unixDay-test.js:20:3
    assert_eq!(
        ts(Interval::UnixDay(1).floor(utc(&[2011.0, 10.0, 6.0, 6.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0]))
    ); // d3-time/test/unixDay-test.js:21:3
    assert_eq!(
        ts(Interval::UnixDay(1).floor(utc(&[2011.0, 10.0, 6.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0]))
    ); // d3-time/test/unixDay-test.js:22:3
    assert_eq!(
        ts(Interval::UnixDay(1).floor(utc(&[2011.0, 10.0, 6.0, 8.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0]))
    ); // d3-time/test/unixDay-test.js:23:3
}

// d3-time/test/unixDay-test.js: unixDay.round(date) returns days
#[test]
fn unixday_round_date_returns_days() {
    assert_eq!(
        ts(Interval::UnixDay(1).round(utc(&[2010.0, 11.0, 30.0, 13.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0]))
    ); // d3-time/test/unixDay-test.js:27:3
    assert_eq!(
        ts(Interval::UnixDay(1).round(utc(&[2010.0, 11.0, 30.0, 11.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 30.0]))
    ); // d3-time/test/unixDay-test.js:28:3
}

// d3-time/test/unixDay-test.js: unixDay.ceil(date) returns days
#[test]
fn unixday_ceil_date_returns_days() {
    assert_eq!(
        ts(Interval::UnixDay(1).ceil(utc(&[2010.0, 11.0, 30.0, 23.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0]))
    ); // d3-time/test/unixDay-test.js:32:3
    assert_eq!(
        ts(Interval::UnixDay(1).ceil(utc(&[2010.0, 11.0, 31.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0]))
    ); // d3-time/test/unixDay-test.js:33:3
    assert_eq!(
        ts(Interval::UnixDay(1).ceil(utc(&[2010.0, 11.0, 31.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/unixDay-test.js:34:3
}

// d3-time/test/unixDay-test.js: unixDay.ceil(date) does not observe daylight saving
#[test]
fn unixday_ceil_date_does_not_observe_daylight_saving() {
    assert_eq!(
        ts(Interval::UnixDay(1).ceil(utc(&[2011.0, 2.0, 13.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 14.0]))
    ); // d3-time/test/unixDay-test.js:38:3
    assert_eq!(
        ts(Interval::UnixDay(1).ceil(utc(&[2011.0, 2.0, 13.0, 8.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 14.0]))
    ); // d3-time/test/unixDay-test.js:39:3
    assert_eq!(
        ts(Interval::UnixDay(1).ceil(utc(&[2011.0, 2.0, 13.0, 9.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 14.0]))
    ); // d3-time/test/unixDay-test.js:40:3
    assert_eq!(
        ts(Interval::UnixDay(1).ceil(utc(&[2011.0, 2.0, 13.0, 10.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 14.0]))
    ); // d3-time/test/unixDay-test.js:41:3
    assert_eq!(
        ts(Interval::UnixDay(1).ceil(utc(&[2011.0, 10.0, 6.0, 5.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 7.0]))
    ); // d3-time/test/unixDay-test.js:42:3
    assert_eq!(
        ts(Interval::UnixDay(1).ceil(utc(&[2011.0, 10.0, 6.0, 6.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 7.0]))
    ); // d3-time/test/unixDay-test.js:43:3
    assert_eq!(
        ts(Interval::UnixDay(1).ceil(utc(&[2011.0, 10.0, 6.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 7.0]))
    ); // d3-time/test/unixDay-test.js:44:3
    assert_eq!(
        ts(Interval::UnixDay(1).ceil(utc(&[2011.0, 10.0, 6.0, 8.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 7.0]))
    ); // d3-time/test/unixDay-test.js:45:3
}

// d3-time/test/unixDay-test.js: unixDay.offset(date) is an alias for unixDay.offset(date, 1)
#[test]
fn unixday_offset_date_is_an_alias_for_unixday_offset_date_1() {
    assert_eq!(
        ts(Interval::UnixDay(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/unixDay-test.js:49:3
}

// d3-time/test/unixDay-test.js: unixDay.offset(date, step) does not modify the passed-in date
#[test]
fn unixday_offset_date_step_does_not_modify_the_passed_in_date() {
    let d = utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::UnixDay(1).offset(d, 1_i64, &TimeZone::UTC);
    assert_eq!(
        ts(d),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/unixDay-test.js:55:3
}

// d3-time/test/unixDay-test.js: unixDay.offset(date, step) does not round the passed-in date
#[test]
fn unixday_offset_date_step_does_not_round_the_passed_in_date() {
    assert_eq!(
        ts(Interval::UnixDay(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/unixDay-test.js:59:3
    assert_eq!(
        ts(Interval::UnixDay(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 29.0, 23.0, 59.0, 59.0, 456.0]))
    ); // d3-time/test/unixDay-test.js:60:3
}

// d3-time/test/unixDay-test.js: unixDay.offset(date, step) allows step to be negative
#[test]
fn unixday_offset_date_step_allows_step_to_be_negative() {
    assert_eq!(
        ts(Interval::UnixDay(1).offset(utc(&[2010.0, 11.0, 31.0]), -1_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 30.0]))
    ); // d3-time/test/unixDay-test.js:64:3
    assert_eq!(
        ts(Interval::UnixDay(1).offset(utc(&[2011.0, 0.0, 1.0]), -2_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 30.0]))
    ); // d3-time/test/unixDay-test.js:65:3
    assert_eq!(
        ts(Interval::UnixDay(1).offset(utc(&[2011.0, 0.0, 1.0]), -1_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0]))
    ); // d3-time/test/unixDay-test.js:66:3
}

// d3-time/test/unixDay-test.js: unixDay.offset(date, step) allows step to be positive
#[test]
fn unixday_offset_date_step_allows_step_to_be_positive() {
    assert_eq!(
        ts(Interval::UnixDay(1).offset(utc(&[2010.0, 11.0, 31.0]), 1_i64, &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/unixDay-test.js:70:3
    assert_eq!(
        ts(Interval::UnixDay(1).offset(utc(&[2010.0, 11.0, 30.0]), 2_i64, &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/unixDay-test.js:71:3
    assert_eq!(
        ts(Interval::UnixDay(1).offset(utc(&[2010.0, 11.0, 30.0]), 1_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0]))
    ); // d3-time/test/unixDay-test.js:72:3
}

// d3-time/test/unixDay-test.js: unixDay.offset(date, step) allows step to be zero
#[test]
fn unixday_offset_date_step_allows_step_to_be_zero() {
    assert_eq!(
        ts(Interval::UnixDay(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/unixDay-test.js:76:3
    assert_eq!(
        ts(Interval::UnixDay(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/unixDay-test.js:77:3
}

// d3-time/test/unixDay-test.js: unixDay.count(start, end) counts days after start (exclusive) and before end (inclusive)
#[test]
fn unixday_count_start_end_counts_days_after_start_exclusive_and_before_end_inclusive() {
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2011.0, 0.0, 1.0, 0.0]),
                utc(&[2011.0, 4.0, 9.0, 0.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        128.0
    ); // d3-time/test/unixDay-test.js:81:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2011.0, 0.0, 1.0, 1.0]),
                utc(&[2011.0, 4.0, 9.0, 0.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        128.0
    ); // d3-time/test/unixDay-test.js:82:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2010.0, 11.0, 31.0, 23.0]),
                utc(&[2011.0, 4.0, 9.0, 0.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        129.0
    ); // d3-time/test/unixDay-test.js:83:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2011.0, 0.0, 1.0, 0.0]),
                utc(&[2011.0, 4.0, 8.0, 23.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        127.0
    ); // d3-time/test/unixDay-test.js:84:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2011.0, 0.0, 1.0, 0.0]),
                utc(&[2011.0, 4.0, 9.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        128.0
    ); // d3-time/test/unixDay-test.js:85:3
}

// d3-time/test/unixDay-test.js: unixDay.count(start, end) does not observe daylight saving
#[test]
fn unixday_count_start_end_does_not_observe_daylight_saving() {
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        71.0
    ); // d3-time/test/unixDay-test.js:89:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 3.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        71.0
    ); // d3-time/test/unixDay-test.js:90:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 4.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        71.0
    ); // d3-time/test/unixDay-test.js:91:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 0.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        309.0
    ); // d3-time/test/unixDay-test.js:92:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        309.0
    ); // d3-time/test/unixDay-test.js:93:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 2.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        309.0
    ); // d3-time/test/unixDay-test.js:94:3
}

// d3-time/test/unixDay-test.js: unixDay.count(start, end) returns 364 or 365 for a full year
#[test]
fn unixday_count_start_end_returns_364_or_365_for_a_full_year() {
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[1999.0, 0.0, 1.0]),
                utc(&[1999.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/unixDay-test.js:98:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2000.0, 0.0, 1.0]),
                utc(&[2000.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        365.0
    ); // d3-time/test/unixDay-test.js:99:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2001.0, 0.0, 1.0]),
                utc(&[2001.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/unixDay-test.js:100:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2002.0, 0.0, 1.0]),
                utc(&[2002.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/unixDay-test.js:101:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2003.0, 0.0, 1.0]),
                utc(&[2003.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/unixDay-test.js:102:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2004.0, 0.0, 1.0]),
                utc(&[2004.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        365.0
    ); // d3-time/test/unixDay-test.js:103:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2005.0, 0.0, 1.0]),
                utc(&[2005.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/unixDay-test.js:104:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2006.0, 0.0, 1.0]),
                utc(&[2006.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/unixDay-test.js:105:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2007.0, 0.0, 1.0]),
                utc(&[2007.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/unixDay-test.js:106:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2008.0, 0.0, 1.0]),
                utc(&[2008.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        365.0
    ); // d3-time/test/unixDay-test.js:107:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2009.0, 0.0, 1.0]),
                utc(&[2009.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/unixDay-test.js:108:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2010.0, 0.0, 1.0]),
                utc(&[2010.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/unixDay-test.js:109:3
    assert_eq!(
        Interval::UnixDay(1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        364.0
    ); // d3-time/test/unixDay-test.js:110:3
}

// d3-time/test/unixDay-test.js: unixDay.every(step) returns every stepth day
#[test]
fn unixday_every_step_returns_every_stepth_day() {
    assert_eq!(
        tss(&Interval::UnixDay(1)
            .every(3_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                utc(&[2008.0, 11.0, 30.0, 0.0, 12.0]),
                utc(&[2009.0, 0.0, 5.0, 23.0, 48.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[utc(&[2008.0, 11.0, 31.0]), utc(&[2009.0, 0.0, 3.0])])
    ); // d3-time/test/unixDay-test.js:114:3
    assert_eq!(
        tss(&Interval::UnixDay(1)
            .every(5_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                utc(&[2008.0, 11.0, 30.0, 0.0, 12.0]),
                utc(&[2009.0, 0.0, 6.0, 23.0, 48.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[utc(&[2009.0, 0.0, 1.0]), utc(&[2009.0, 0.0, 6.0])])
    ); // d3-time/test/unixDay-test.js:115:3
    assert_eq!(
        tss(&Interval::UnixDay(1)
            .every(7_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                utc(&[2008.0, 11.0, 30.0, 0.0, 12.0]),
                utc(&[2009.0, 0.0, 8.0, 23.0, 48.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[utc(&[2009.0, 0.0, 1.0]), utc(&[2009.0, 0.0, 8.0])])
    ); // d3-time/test/unixDay-test.js:116:3
}

// d3-time/test/week-test.js: timeWeek.floor(date) returns sundays
#[test]
fn timeweek_floor_date_returns_sundays() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 26.0]))
    ); // d3-time/test/week-test.js:6:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 26.0]))
    ); // d3-time/test/week-test.js:7:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 26.0]))
    ); // d3-time/test/week-test.js:8:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(local(&la, &[2011.0, 0.0, 1.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 26.0]))
    ); // d3-time/test/week-test.js:9:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 2.0]))
    ); // d3-time/test/week-test.js:10:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 2.0]))
    ); // d3-time/test/week-test.js:11:3
}

// d3-time/test/week-test.js: timeWeek.floor(date) observes the start of daylight savings time
#[test]
fn timeweek_floor_date_observes_the_start_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).floor(local(&la, &[2011.0, 2.0, 13.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 13.0]))
    ); // d3-time/test/week-test.js:15:3
}

// d3-time/test/week-test.js: timeWeek.floor(date) observes the end of the daylight savings time
#[test]
fn timeweek_floor_date_observes_the_end_of_the_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).floor(local(&la, &[2011.0, 10.0, 6.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 6.0]))
    ); // d3-time/test/week-test.js:19:3
}

// d3-time/test/week-test.js: timeWeek.floor(date) correctly handles years in the first century
#[test]
fn timeweek_floor_date_correctly_handles_years_in_the_first_century() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).floor(local(&la, &[9.0, 10.0, 6.0, 7.0]), &la)),
        ts(local(&la, &[9.0, 10.0, 1.0]))
    ); // d3-time/test/week-test.js:23:3
}

// d3-time/test/week-test.js: timeWeek.ceil(date) returns sundays
#[test]
fn timeweek_ceil_date_returns_sundays() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 2.0]))
    ); // d3-time/test/week-test.js:27:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 2.0]))
    ); // d3-time/test/week-test.js:28:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 2.0]))
    ); // d3-time/test/week-test.js:29:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(local(&la, &[2011.0, 0.0, 1.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 2.0]))
    ); // d3-time/test/week-test.js:30:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 2.0]))
    ); // d3-time/test/week-test.js:31:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 9.0]))
    ); // d3-time/test/week-test.js:32:3
}

// d3-time/test/week-test.js: timeWeek.ceil(date) observes the start of daylight savings time
#[test]
fn timeweek_ceil_date_observes_the_start_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).ceil(local(&la, &[2011.0, 2.0, 13.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 20.0]))
    ); // d3-time/test/week-test.js:36:3
}

// d3-time/test/week-test.js: timeWeek.ceil(date) observes the end of the daylight savings time
#[test]
fn timeweek_ceil_date_observes_the_end_of_the_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).ceil(local(&la, &[2011.0, 10.0, 6.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 13.0]))
    ); // d3-time/test/week-test.js:40:3
}

// d3-time/test/week-test.js: timeWeek.offset(date, step) does not modify the passed-in date
#[test]
fn timeweek_offset_date_step_does_not_modify_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    let d = local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Week(Weekday::Sunday, 1).offset(d, 1_i64, &la);
    assert_eq!(
        ts(d),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/week-test.js:46:3
}

// d3-time/test/week-test.js: timeWeek.offset(date, step) does not round the passed-in-date
#[test]
fn timeweek_offset_date_step_does_not_round_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 7.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/week-test.js:50:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 17.0, 23.0, 59.0, 59.0, 456.0]))
    ); // d3-time/test/week-test.js:51:3
}

// d3-time/test/week-test.js: timeWeek.offset(date, step) allows negative offsets
#[test]
fn timeweek_offset_date_step_allows_negative_offsets() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            local(&la, &[2010.0, 11.0, 1.0]),
            -1_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 10.0, 24.0]))
    ); // d3-time/test/week-test.js:55:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(local(&la, &[2011.0, 0.0, 1.0]), -2_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 18.0]))
    ); // d3-time/test/week-test.js:56:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(local(&la, &[2011.0, 0.0, 1.0]), -1_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 25.0]))
    ); // d3-time/test/week-test.js:57:3
}

// d3-time/test/week-test.js: timeWeek.offset(date, step) allows positive offsets
#[test]
fn timeweek_offset_date_step_allows_positive_offsets() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            local(&la, &[2010.0, 10.0, 24.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 1.0]))
    ); // d3-time/test/week-test.js:61:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            local(&la, &[2010.0, 11.0, 18.0]),
            2_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/week-test.js:62:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            local(&la, &[2010.0, 11.0, 25.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/week-test.js:63:3
}

// d3-time/test/week-test.js: timeWeek.offset(date, step) allows zero offset
#[test]
fn timeweek_offset_date_step_allows_zero_offset() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/week-test.js:67:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/week-test.js:68:3
}

// d3-time/test/week-test.js: timeWeek.range(start, stop) returns sundays
#[test]
fn timeweek_range_start_stop_returns_sundays() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Week(Weekday::Sunday, 1)
            .range(
                local(&la, &[2010.0, 11.0, 21.0]),
                local(&la, &[2011.0, 0.0, 12.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2010.0, 11.0, 26.0]),
            local(&la, &[2011.0, 0.0, 2.0]),
            local(&la, &[2011.0, 0.0, 9.0])
        ])
    ); // d3-time/test/week-test.js:72:3
}

// d3-time/test/week-test.js: timeWeek.range(start, stop) has an inclusive lower bound
#[test]
fn timeweek_range_start_stop_has_an_inclusive_lower_bound() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .range(
                local(&la, &[2010.0, 11.0, 21.0]),
                local(&la, &[2011.0, 0.0, 12.0]),
                &la
            )
            .collect::<Vec<_>>()[0_usize]),
        ts(local(&la, &[2010.0, 11.0, 26.0]))
    ); // d3-time/test/week-test.js:80:3
}

// d3-time/test/week-test.js: timeWeek.range(start, stop) has an exclusive upper bound
#[test]
fn timeweek_range_start_stop_has_an_exclusive_upper_bound() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .range(
                local(&la, &[2010.0, 11.0, 21.0]),
                local(&la, &[2011.0, 0.0, 12.0]),
                &la
            )
            .collect::<Vec<_>>()[2_usize]),
        ts(local(&la, &[2011.0, 0.0, 9.0]))
    ); // d3-time/test/week-test.js:84:3
}

// d3-time/test/week-test.js: timeWeek.range(start, stop) can skip weeks
#[test]
fn timeweek_range_start_stop_can_skip_weeks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Week(Weekday::Sunday, 1)
            .range_step(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 3.0, 1.0]),
                4_u32,
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 0.0, 2.0]),
            local(&la, &[2011.0, 0.0, 30.0]),
            local(&la, &[2011.0, 1.0, 27.0]),
            local(&la, &[2011.0, 2.0, 27.0])
        ])
    ); // d3-time/test/week-test.js:88:3
}

// d3-time/test/week-test.js: timeWeek.range(start, stop) observes start of daylight savings time
#[test]
fn timeweek_range_start_stop_observes_start_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Week(Weekday::Sunday, 1)
            .range(
                local(&la, &[2011.0, 2.0, 1.0]),
                local(&la, &[2011.0, 2.0, 28.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 2.0, 6.0]),
            local(&la, &[2011.0, 2.0, 13.0]),
            local(&la, &[2011.0, 2.0, 20.0]),
            local(&la, &[2011.0, 2.0, 27.0])
        ])
    ); // d3-time/test/week-test.js:97:3
}

// d3-time/test/week-test.js: timeWeek.range(start, stop) observes end of daylight savings time
#[test]
fn timeweek_range_start_stop_observes_end_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Week(Weekday::Sunday, 1)
            .range(
                local(&la, &[2011.0, 10.0, 1.0]),
                local(&la, &[2011.0, 10.0, 30.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 10.0, 6.0]),
            local(&la, &[2011.0, 10.0, 13.0]),
            local(&la, &[2011.0, 10.0, 20.0]),
            local(&la, &[2011.0, 10.0, 27.0])
        ])
    ); // d3-time/test/week-test.js:106:3
}

// d3-time/test/week-test.js: timeWeek is an alias for timeSunday
#[test]
fn timeweek_is_an_alias_for_timesunday() {
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1),
        Interval::Week(Weekday::Sunday, 1)
    ); // d3-time/test/week-test.js:115:3
}

// d3-time/test/week-test.js: timeWeek.every(step) returns every stepth Sunday, starting with the first Sunday of the month
#[test]
fn timeweek_every_step_returns_every_stepth_sunday_starting_with_the_first_sunday_of_the_month() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Week(Weekday::Sunday, 1)
            .every(2_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                local(&la, &[2008.0, 11.0, 3.0]),
                local(&la, &[2009.0, 1.0, 5.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2008.0, 11.0, 7.0]),
            local(&la, &[2008.0, 11.0, 21.0]),
            local(&la, &[2009.0, 0.0, 4.0]),
            local(&la, &[2009.0, 0.0, 18.0]),
            local(&la, &[2009.0, 1.0, 1.0])
        ])
    ); // d3-time/test/week-test.js:119:3
}

// d3-time/test/utcWeek-test.js: utcWeek.floor(date) returns sundays
#[test]
fn utcweek_floor_date_returns_sundays() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 26.0]))
    ); // d3-time/test/utcWeek-test.js:6:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 26.0]))
    ); // d3-time/test/utcWeek-test.js:7:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 26.0]))
    ); // d3-time/test/utcWeek-test.js:8:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(utc(&[2011.0, 0.0, 1.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 26.0]))
    ); // d3-time/test/utcWeek-test.js:9:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(utc(&[2011.0, 0.0, 2.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 2.0]))
    ); // d3-time/test/utcWeek-test.js:10:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(utc(&[2011.0, 0.0, 2.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 2.0]))
    ); // d3-time/test/utcWeek-test.js:11:3
}

// d3-time/test/utcWeek-test.js: utcWeek.floor(date) observes the start of daylight savings time
#[test]
fn utcweek_floor_date_observes_the_start_of_daylight_savings_time() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(utc(&[2011.0, 2.0, 13.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0]))
    ); // d3-time/test/utcWeek-test.js:15:3
}

// d3-time/test/utcWeek-test.js: utcWeek.floor(date) observes the end of the daylight savings time
#[test]
fn utcweek_floor_date_observes_the_end_of_the_daylight_savings_time() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(utc(&[2011.0, 10.0, 6.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0]))
    ); // d3-time/test/utcWeek-test.js:19:3
}

// d3-time/test/utcWeek-test.js: utcWeek.floor(date) correctly handles years in the first century
#[test]
fn utcweek_floor_date_correctly_handles_years_in_the_first_century() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).floor(utc(&[9.0, 10.0, 6.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[9.0, 10.0, 1.0]))
    ); // d3-time/test/utcWeek-test.js:23:3
}

// d3-time/test/utcWeek-test.js: utcWeek.ceil(date) returns sundays
#[test]
fn utcweek_ceil_date_returns_sundays() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 2.0]))
    ); // d3-time/test/utcWeek-test.js:27:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 2.0]))
    ); // d3-time/test/utcWeek-test.js:28:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 2.0]))
    ); // d3-time/test/utcWeek-test.js:29:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(utc(&[2011.0, 0.0, 1.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 2.0]))
    ); // d3-time/test/utcWeek-test.js:30:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(utc(&[2011.0, 0.0, 2.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 2.0]))
    ); // d3-time/test/utcWeek-test.js:31:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(utc(&[2011.0, 0.0, 2.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 9.0]))
    ); // d3-time/test/utcWeek-test.js:32:3
}

// d3-time/test/utcWeek-test.js: utcWeek.ceil(date) does not observe the start of daylight savings time
#[test]
fn utcweek_ceil_date_does_not_observe_the_start_of_daylight_savings_time() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).ceil(utc(&[2011.0, 2.0, 13.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 20.0]))
    ); // d3-time/test/utcWeek-test.js:36:3
}

// d3-time/test/utcWeek-test.js: utcWeek.ceil(date) does not observe the end of the daylight savings time
#[test]
fn utcweek_ceil_date_does_not_observe_the_end_of_the_daylight_savings_time() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).ceil(utc(&[2011.0, 10.0, 6.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 13.0]))
    ); // d3-time/test/utcWeek-test.js:40:3
}

// d3-time/test/utcWeek-test.js: utcWeek.offset(date, step) does not modify the passed-in date
#[test]
fn utcweek_offset_date_step_does_not_modify_the_passed_in_date() {
    let d = utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Week(Weekday::Sunday, 1).offset(d, 1_i64, &TimeZone::UTC);
    assert_eq!(
        ts(d),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcWeek-test.js:46:3
}

// d3-time/test/utcWeek-test.js: utcWeek.offset(date, step) does not round the passed-in-date
#[test]
fn utcweek_offset_date_step_does_not_round_the_passed_in_date() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 7.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcWeek-test.js:50:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 17.0, 23.0, 59.0, 59.0, 456.0]))
    ); // d3-time/test/utcWeek-test.js:51:3
}

// d3-time/test/utcWeek-test.js: utcWeek.offset(date, step) allows negative offsets
#[test]
fn utcweek_offset_date_step_allows_negative_offsets() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2010.0, 11.0, 1.0]),
            -1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 10.0, 24.0]))
    ); // d3-time/test/utcWeek-test.js:55:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2011.0, 0.0, 1.0]),
            -2_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 18.0]))
    ); // d3-time/test/utcWeek-test.js:56:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2011.0, 0.0, 1.0]),
            -1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 25.0]))
    ); // d3-time/test/utcWeek-test.js:57:3
}

// d3-time/test/utcWeek-test.js: utcWeek.offset(date, step) allows positive offsets
#[test]
fn utcweek_offset_date_step_allows_positive_offsets() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2010.0, 10.0, 24.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 1.0]))
    ); // d3-time/test/utcWeek-test.js:61:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2010.0, 11.0, 18.0]),
            2_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcWeek-test.js:62:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2010.0, 11.0, 25.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcWeek-test.js:63:3
}

// d3-time/test/utcWeek-test.js: utcWeek.offset(date, step) allows zero offset
#[test]
fn utcweek_offset_date_step_allows_zero_offset() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcWeek-test.js:67:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/utcWeek-test.js:68:3
}

// d3-time/test/utcWeek-test.js: utcWeek.range(start, stop) returns sundays
#[test]
fn utcweek_range_start_stop_returns_sundays() {
    assert_eq!(
        tss(&Interval::Week(Weekday::Sunday, 1)
            .range(
                utc(&[2010.0, 11.0, 21.0]),
                utc(&[2011.0, 0.0, 12.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2010.0, 11.0, 26.0]),
            utc(&[2011.0, 0.0, 2.0]),
            utc(&[2011.0, 0.0, 9.0])
        ])
    ); // d3-time/test/utcWeek-test.js:72:3
}

// d3-time/test/utcWeek-test.js: utcWeek.range(start, stop) has an inclusive lower bound
#[test]
fn utcweek_range_start_stop_has_an_inclusive_lower_bound() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .range(
                utc(&[2010.0, 11.0, 21.0]),
                utc(&[2011.0, 0.0, 12.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()[0_usize]),
        ts(utc(&[2010.0, 11.0, 26.0]))
    ); // d3-time/test/utcWeek-test.js:80:3
}

// d3-time/test/utcWeek-test.js: utcWeek.range(start, stop) has an exclusive upper bound
#[test]
fn utcweek_range_start_stop_has_an_exclusive_upper_bound() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .range(
                utc(&[2010.0, 11.0, 21.0]),
                utc(&[2011.0, 0.0, 12.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()[2_usize]),
        ts(utc(&[2011.0, 0.0, 9.0]))
    ); // d3-time/test/utcWeek-test.js:84:3
}

// d3-time/test/utcWeek-test.js: utcWeek.range(start, stop) can skip weeks
#[test]
fn utcweek_range_start_stop_can_skip_weeks() {
    assert_eq!(
        tss(&Interval::Week(Weekday::Sunday, 1)
            .range_step(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 3.0, 1.0]),
                4_u32,
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 0.0, 2.0]),
            utc(&[2011.0, 0.0, 30.0]),
            utc(&[2011.0, 1.0, 27.0]),
            utc(&[2011.0, 2.0, 27.0])
        ])
    ); // d3-time/test/utcWeek-test.js:88:3
}

// d3-time/test/utcWeek-test.js: utcWeek.range(start, stop) does not observe start of daylight savings time
#[test]
fn utcweek_range_start_stop_does_not_observe_start_of_daylight_savings_time() {
    assert_eq!(
        tss(&Interval::Week(Weekday::Sunday, 1)
            .range(
                utc(&[2011.0, 2.0, 1.0]),
                utc(&[2011.0, 2.0, 28.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 2.0, 6.0]),
            utc(&[2011.0, 2.0, 13.0]),
            utc(&[2011.0, 2.0, 20.0]),
            utc(&[2011.0, 2.0, 27.0])
        ])
    ); // d3-time/test/utcWeek-test.js:97:3
}

// d3-time/test/utcWeek-test.js: utcWeek.range(start, stop) does not observe end of daylight savings time
#[test]
fn utcweek_range_start_stop_does_not_observe_end_of_daylight_savings_time() {
    assert_eq!(
        tss(&Interval::Week(Weekday::Sunday, 1)
            .range(
                utc(&[2011.0, 10.0, 1.0]),
                utc(&[2011.0, 10.0, 30.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 10.0, 6.0]),
            utc(&[2011.0, 10.0, 13.0]),
            utc(&[2011.0, 10.0, 20.0]),
            utc(&[2011.0, 10.0, 27.0])
        ])
    ); // d3-time/test/utcWeek-test.js:106:3
}

// d3-time/test/utcWeek-test.js: utcWeek is an alias for utcSunday
#[test]
fn utcweek_is_an_alias_for_utcsunday() {
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1),
        Interval::Week(Weekday::Sunday, 1)
    ); // d3-time/test/utcWeek-test.js:115:3
}

// d3-time/test/utcWeek-test.js: utcWeek.every(step) returns every stepth Sunday, starting with the first Sunday of the month
#[test]
fn utcweek_every_step_returns_every_stepth_sunday_starting_with_the_first_sunday_of_the_month() {
    assert_eq!(
        tss(&Interval::Week(Weekday::Sunday, 1)
            .every(2_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                utc(&[2008.0, 11.0, 3.0]),
                utc(&[2009.0, 1.0, 5.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2008.0, 11.0, 7.0]),
            utc(&[2008.0, 11.0, 21.0]),
            utc(&[2009.0, 0.0, 4.0]),
            utc(&[2009.0, 0.0, 18.0]),
            utc(&[2009.0, 1.0, 1.0])
        ])
    ); // d3-time/test/utcWeek-test.js:119:3
}

// d3-time/test/sunday-test.js: timeSunday.floor(date) returns Sundays
#[test]
fn timesunday_floor_date_returns_sundays() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 26.0]))
    ); // d3-time/test/sunday-test.js:10:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 26.0]))
    ); // d3-time/test/sunday-test.js:11:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 26.0]))
    ); // d3-time/test/sunday-test.js:12:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(local(&la, &[2011.0, 0.0, 1.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 26.0]))
    ); // d3-time/test/sunday-test.js:13:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 2.0]))
    ); // d3-time/test/sunday-test.js:14:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 2.0]))
    ); // d3-time/test/sunday-test.js:15:3
}

// d3-time/test/sunday-test.js: timeSunday.floor(date) observes daylight saving
#[test]
fn timesunday_floor_date_observes_daylight_saving() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).floor(local(&la, &[2011.0, 2.0, 13.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 13.0]))
    ); // d3-time/test/sunday-test.js:19:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).floor(local(&la, &[2011.0, 10.0, 6.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 6.0]))
    ); // d3-time/test/sunday-test.js:20:3
}

// d3-time/test/sunday-test.js: timeSunday.floor(date) handles years in the first century
#[test]
fn timesunday_floor_date_handles_years_in_the_first_century() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).floor(local(&la, &[9.0, 10.0, 6.0, 7.0]), &la)),
        ts(local(&la, &[9.0, 10.0, 1.0]))
    ); // d3-time/test/sunday-test.js:24:3
}

// d3-time/test/sunday-test.js: timeSunday.ceil(date) returns Sundays
#[test]
fn timesunday_ceil_date_returns_sundays() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 2.0]))
    ); // d3-time/test/sunday-test.js:28:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 2.0]))
    ); // d3-time/test/sunday-test.js:29:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 2.0]))
    ); // d3-time/test/sunday-test.js:30:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(local(&la, &[2011.0, 0.0, 1.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 2.0]))
    ); // d3-time/test/sunday-test.js:31:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 2.0]))
    ); // d3-time/test/sunday-test.js:32:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 9.0]))
    ); // d3-time/test/sunday-test.js:33:3
}

// d3-time/test/sunday-test.js: timeSunday.ceil(date) observes daylight saving
#[test]
fn timesunday_ceil_date_observes_daylight_saving() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).ceil(local(&la, &[2011.0, 2.0, 13.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 20.0]))
    ); // d3-time/test/sunday-test.js:37:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).ceil(local(&la, &[2011.0, 10.0, 6.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 13.0]))
    ); // d3-time/test/sunday-test.js:38:3
}

// d3-time/test/sunday-test.js: timeSunday.offset(date) is an alias for timeSunday.offset(date, 1)
#[test]
fn timesunday_offset_date_is_an_alias_for_timesunday_offset_date_1() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 7.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/sunday-test.js:42:3
}

// d3-time/test/sunday-test.js: timeSunday.offset(date, step) does not modify the passed-in date
#[test]
fn timesunday_offset_date_step_does_not_modify_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    let d = local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Week(Weekday::Sunday, 1).offset(d, 1_i64, &la);
    assert_eq!(
        ts(d),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/sunday-test.js:48:3
}

// d3-time/test/sunday-test.js: timeSunday.offset(date, step) does not round the passed-in date
#[test]
fn timesunday_offset_date_step_does_not_round_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 7.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/sunday-test.js:52:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 17.0, 23.0, 59.0, 59.0, 456.0]))
    ); // d3-time/test/sunday-test.js:53:3
}

// d3-time/test/sunday-test.js: timeSunday.offset(date, step) allows step to be negative
#[test]
fn timesunday_offset_date_step_allows_step_to_be_negative() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            local(&la, &[2010.0, 11.0, 1.0]),
            -1_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 10.0, 24.0]))
    ); // d3-time/test/sunday-test.js:57:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(local(&la, &[2011.0, 0.0, 1.0]), -2_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 18.0]))
    ); // d3-time/test/sunday-test.js:58:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(local(&la, &[2011.0, 0.0, 1.0]), -1_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 25.0]))
    ); // d3-time/test/sunday-test.js:59:3
}

// d3-time/test/sunday-test.js: timeSunday.offset(date, step) allows step to be positive
#[test]
fn timesunday_offset_date_step_allows_step_to_be_positive() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            local(&la, &[2010.0, 10.0, 24.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 1.0]))
    ); // d3-time/test/sunday-test.js:63:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            local(&la, &[2010.0, 11.0, 18.0]),
            2_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/sunday-test.js:64:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            local(&la, &[2010.0, 11.0, 25.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/sunday-test.js:65:3
}

// d3-time/test/sunday-test.js: timeSunday.offset(date, step) allows step to be zero
#[test]
fn timesunday_offset_date_step_allows_step_to_be_zero() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/sunday-test.js:69:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/sunday-test.js:70:3
}

// d3-time/test/sunday-test.js: timeSunday.range(start, stop) returns Sundays between start (inclusive) and stop (exclusive)
#[test]
fn timesunday_range_start_stop_returns_sundays_between_start_inclusive_and_stop_exclusive() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Week(Weekday::Sunday, 1)
            .range(
                local(&la, &[2011.0, 11.0, 1.0]),
                local(&la, &[2012.0, 0.0, 15.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 11.0, 4.0]),
            local(&la, &[2011.0, 11.0, 11.0]),
            local(&la, &[2011.0, 11.0, 18.0]),
            local(&la, &[2011.0, 11.0, 25.0]),
            local(&la, &[2012.0, 0.0, 1.0]),
            local(&la, &[2012.0, 0.0, 8.0])
        ])
    ); // d3-time/test/sunday-test.js:74:3
}

// d3-time/test/sunday-test.js: timeSunday.range(start, stop) returns Sundays
#[test]
fn timesunday_range_start_stop_returns_sundays() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Week(Weekday::Sunday, 1)
            .range(
                local(&la, &[2011.0, 11.0, 1.0, 12.0, 23.0]),
                local(&la, &[2012.0, 0.0, 14.0, 12.0, 23.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 11.0, 4.0]),
            local(&la, &[2011.0, 11.0, 11.0]),
            local(&la, &[2011.0, 11.0, 18.0]),
            local(&la, &[2011.0, 11.0, 25.0]),
            local(&la, &[2012.0, 0.0, 1.0]),
            local(&la, &[2012.0, 0.0, 8.0])
        ])
    ); // d3-time/test/sunday-test.js:85:3
}

// d3-time/test/sunday-test.js: timeSunday.range(start, stop) returns the empty array if start >= stop
#[test]
fn timesunday_range_start_stop_returns_the_empty_array_if_start_stop() {
    let la = zone("America/Los_Angeles");
    assert_d3_eq(
        &tss(&Interval::Week(Weekday::Sunday, 1)
            .range(
                local(&la, &[2011.0, 11.0, 10.0]),
                local(&la, &[2011.0, 10.0, 4.0]),
                &la,
            )
            .collect::<Vec<_>>()),
        &Vec::<f64>::new(),
    ); // d3-time/test/sunday-test.js:111:3
    assert_d3_eq(
        &tss(&Interval::Week(Weekday::Sunday, 1)
            .range(
                local(&la, &[2011.0, 10.0, 1.0]),
                local(&la, &[2011.0, 10.0, 1.0]),
                &la,
            )
            .collect::<Vec<_>>()),
        &Vec::<f64>::new(),
    ); // d3-time/test/sunday-test.js:112:3
}

// d3-time/test/sunday-test.js: timeSunday.range(start, stop, step) returns every step Sunday
#[test]
fn timesunday_range_start_stop_step_returns_every_step_sunday() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Week(Weekday::Sunday, 1)
            .range_step(
                local(&la, &[2011.0, 11.0, 1.0]),
                local(&la, &[2012.0, 0.0, 15.0]),
                2_u32,
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 11.0, 4.0]),
            local(&la, &[2011.0, 11.0, 18.0]),
            local(&la, &[2012.0, 0.0, 1.0])
        ])
    ); // d3-time/test/sunday-test.js:116:3
}

// d3-time/test/sunday-test.js: timeSunday.count(start, end) counts Sundays after start (exclusive) and before end (inclusive)
#[test]
fn timesunday_count_start_end_counts_sundays_after_start_exclusive_and_before_end_inclusive() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                local(&la, &[2014.0, 0.0, 1.0]),
                local(&la, &[2014.0, 0.0, 4.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/sunday-test.js:131:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                local(&la, &[2014.0, 0.0, 1.0]),
                local(&la, &[2014.0, 0.0, 5.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/sunday-test.js:132:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                local(&la, &[2014.0, 0.0, 1.0]),
                local(&la, &[2014.0, 0.0, 6.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/sunday-test.js:133:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                local(&la, &[2014.0, 0.0, 1.0]),
                local(&la, &[2014.0, 0.0, 12.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        2.0
    ); // d3-time/test/sunday-test.js:134:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 7.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/sunday-test.js:143:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 8.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/sunday-test.js:144:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 9.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/sunday-test.js:145:3
}

// d3-time/test/sunday-test.js: timeSunday.count(start, end) observes daylight saving
#[test]
fn timesunday_count_start_end_observes_daylight_saving() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        11.0
    ); // d3-time/test/sunday-test.js:149:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 3.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        11.0
    ); // d3-time/test/sunday-test.js:150:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 4.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        11.0
    ); // d3-time/test/sunday-test.js:151:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 0.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        45.0
    ); // d3-time/test/sunday-test.js:152:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        45.0
    ); // d3-time/test/sunday-test.js:153:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 2.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        45.0
    ); // d3-time/test/sunday-test.js:154:3
}

// d3-time/test/utcSunday-test.js: utcSunday.floor(date) returns Sundays
#[test]
fn utcsunday_floor_date_returns_sundays() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 26.0]))
    ); // d3-time/test/utcSunday-test.js:10:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 26.0]))
    ); // d3-time/test/utcSunday-test.js:11:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 26.0]))
    ); // d3-time/test/utcSunday-test.js:12:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(utc(&[2011.0, 0.0, 1.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 26.0]))
    ); // d3-time/test/utcSunday-test.js:13:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(utc(&[2011.0, 0.0, 2.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 2.0]))
    ); // d3-time/test/utcSunday-test.js:14:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(utc(&[2011.0, 0.0, 2.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 2.0]))
    ); // d3-time/test/utcSunday-test.js:15:3
}

// d3-time/test/utcSunday-test.js: utcSunday.floor(date) observes daylight saving
#[test]
fn utcsunday_floor_date_observes_daylight_saving() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(utc(&[2011.0, 2.0, 13.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 13.0]))
    ); // d3-time/test/utcSunday-test.js:19:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .floor(utc(&[2011.0, 10.0, 6.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 6.0]))
    ); // d3-time/test/utcSunday-test.js:20:3
}

// d3-time/test/utcSunday-test.js: utcSunday.floor(date) handles years in the first century
#[test]
fn utcsunday_floor_date_handles_years_in_the_first_century() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).floor(utc(&[9.0, 10.0, 6.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[9.0, 10.0, 1.0]))
    ); // d3-time/test/utcSunday-test.js:24:3
}

// d3-time/test/utcSunday-test.js: utcSunday.ceil(date) returns Sundays
#[test]
fn utcsunday_ceil_date_returns_sundays() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 2.0]))
    ); // d3-time/test/utcSunday-test.js:28:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 2.0]))
    ); // d3-time/test/utcSunday-test.js:29:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 2.0]))
    ); // d3-time/test/utcSunday-test.js:30:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(utc(&[2011.0, 0.0, 1.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 2.0]))
    ); // d3-time/test/utcSunday-test.js:31:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(utc(&[2011.0, 0.0, 2.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 2.0]))
    ); // d3-time/test/utcSunday-test.js:32:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1)
            .ceil(utc(&[2011.0, 0.0, 2.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 9.0]))
    ); // d3-time/test/utcSunday-test.js:33:3
}

// d3-time/test/utcSunday-test.js: utcSunday.ceil(date) observes daylight saving
#[test]
fn utcsunday_ceil_date_observes_daylight_saving() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).ceil(utc(&[2011.0, 2.0, 13.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 20.0]))
    ); // d3-time/test/utcSunday-test.js:37:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).ceil(utc(&[2011.0, 10.0, 6.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 13.0]))
    ); // d3-time/test/utcSunday-test.js:38:3
}

// d3-time/test/utcSunday-test.js: utcSunday.offset(date) is an alias for utcSunday.offset(date, 1)
#[test]
fn utcsunday_offset_date_is_an_alias_for_utcsunday_offset_date_1() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 7.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcSunday-test.js:42:3
}

// d3-time/test/utcSunday-test.js: utcSunday.offset(date, step) does not modify the passed-in date
#[test]
fn utcsunday_offset_date_step_does_not_modify_the_passed_in_date() {
    let d = utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Week(Weekday::Sunday, 1).offset(d, 1_i64, &TimeZone::UTC);
    assert_eq!(
        ts(d),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcSunday-test.js:48:3
}

// d3-time/test/utcSunday-test.js: utcSunday.offset(date, step) does not round the passed-in date
#[test]
fn utcsunday_offset_date_step_does_not_round_the_passed_in_date() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 7.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcSunday-test.js:52:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 17.0, 23.0, 59.0, 59.0, 456.0]))
    ); // d3-time/test/utcSunday-test.js:53:3
}

// d3-time/test/utcSunday-test.js: utcSunday.offset(date, step) allows step to be negative
#[test]
fn utcsunday_offset_date_step_allows_step_to_be_negative() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2010.0, 11.0, 1.0]),
            -1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 10.0, 24.0]))
    ); // d3-time/test/utcSunday-test.js:57:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2011.0, 0.0, 1.0]),
            -2_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 18.0]))
    ); // d3-time/test/utcSunday-test.js:58:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2011.0, 0.0, 1.0]),
            -1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 25.0]))
    ); // d3-time/test/utcSunday-test.js:59:3
}

// d3-time/test/utcSunday-test.js: utcSunday.offset(date, step) allows step to be positive
#[test]
fn utcsunday_offset_date_step_allows_step_to_be_positive() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2010.0, 10.0, 24.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 1.0]))
    ); // d3-time/test/utcSunday-test.js:63:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2010.0, 11.0, 18.0]),
            2_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcSunday-test.js:64:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2010.0, 11.0, 25.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcSunday-test.js:65:3
}

// d3-time/test/utcSunday-test.js: utcSunday.offset(date, step) allows step to be zero
#[test]
fn utcsunday_offset_date_step_allows_step_to_be_zero() {
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcSunday-test.js:69:3
    assert_eq!(
        ts(Interval::Week(Weekday::Sunday, 1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/utcSunday-test.js:70:3
}

// d3-time/test/utcSunday-test.js: utcSunday.range(start, stop) returns Sundays between start (inclusive) and stop (exclusive)
#[test]
fn utcsunday_range_start_stop_returns_sundays_between_start_inclusive_and_stop_exclusive() {
    assert_eq!(
        tss(&Interval::Week(Weekday::Sunday, 1)
            .range(
                utc(&[2011.0, 11.0, 1.0]),
                utc(&[2012.0, 0.0, 15.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 11.0, 4.0]),
            utc(&[2011.0, 11.0, 11.0]),
            utc(&[2011.0, 11.0, 18.0]),
            utc(&[2011.0, 11.0, 25.0]),
            utc(&[2012.0, 0.0, 1.0]),
            utc(&[2012.0, 0.0, 8.0])
        ])
    ); // d3-time/test/utcSunday-test.js:74:3
}

// d3-time/test/utcSunday-test.js: utcSunday.range(start, stop) returns Sundays
#[test]
fn utcsunday_range_start_stop_returns_sundays() {
    assert_eq!(
        tss(&Interval::Week(Weekday::Sunday, 1)
            .range(
                utc(&[2011.0, 11.0, 1.0, 12.0, 23.0]),
                utc(&[2012.0, 0.0, 14.0, 12.0, 23.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 11.0, 4.0]),
            utc(&[2011.0, 11.0, 11.0]),
            utc(&[2011.0, 11.0, 18.0]),
            utc(&[2011.0, 11.0, 25.0]),
            utc(&[2012.0, 0.0, 1.0]),
            utc(&[2012.0, 0.0, 8.0])
        ])
    ); // d3-time/test/utcSunday-test.js:85:3
}

// d3-time/test/utcSunday-test.js: utcSunday.range(start, stop) returns the empty array if start >= stop
#[test]
fn utcsunday_range_start_stop_returns_the_empty_array_if_start_stop() {
    assert_d3_eq(
        &tss(&Interval::Week(Weekday::Sunday, 1)
            .range(
                utc(&[2011.0, 11.0, 10.0]),
                utc(&[2011.0, 10.0, 4.0]),
                &TimeZone::UTC,
            )
            .collect::<Vec<_>>()),
        &Vec::<f64>::new(),
    ); // d3-time/test/utcSunday-test.js:111:3
    assert_d3_eq(
        &tss(&Interval::Week(Weekday::Sunday, 1)
            .range(
                utc(&[2011.0, 10.0, 1.0]),
                utc(&[2011.0, 10.0, 1.0]),
                &TimeZone::UTC,
            )
            .collect::<Vec<_>>()),
        &Vec::<f64>::new(),
    ); // d3-time/test/utcSunday-test.js:112:3
}

// d3-time/test/utcSunday-test.js: utcSunday.range(start, stop, step) returns every step Sunday
#[test]
fn utcsunday_range_start_stop_step_returns_every_step_sunday() {
    assert_eq!(
        tss(&Interval::Week(Weekday::Sunday, 1)
            .range_step(
                utc(&[2011.0, 11.0, 1.0]),
                utc(&[2012.0, 0.0, 15.0]),
                2_u32,
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 11.0, 4.0]),
            utc(&[2011.0, 11.0, 18.0]),
            utc(&[2012.0, 0.0, 1.0])
        ])
    ); // d3-time/test/utcSunday-test.js:116:3
}

// d3-time/test/utcSunday-test.js: utcSunday.count(start, end) counts Sundays after start (exclusive) and before end (inclusive)
#[test]
fn utcsunday_count_start_end_counts_sundays_after_start_exclusive_and_before_end_inclusive() {
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                utc(&[2014.0, 0.0, 1.0]),
                utc(&[2014.0, 0.0, 4.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/utcSunday-test.js:131:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                utc(&[2014.0, 0.0, 1.0]),
                utc(&[2014.0, 0.0, 5.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcSunday-test.js:132:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                utc(&[2014.0, 0.0, 1.0]),
                utc(&[2014.0, 0.0, 6.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcSunday-test.js:133:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                utc(&[2014.0, 0.0, 1.0]),
                utc(&[2014.0, 0.0, 12.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        2.0
    ); // d3-time/test/utcSunday-test.js:134:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 7.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/utcSunday-test.js:143:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 8.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcSunday-test.js:144:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 9.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcSunday-test.js:145:3
}

// d3-time/test/utcSunday-test.js: utcSunday.count(start, end) does not observe daylight saving
#[test]
fn utcsunday_count_start_end_does_not_observe_daylight_saving() {
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        11.0
    ); // d3-time/test/utcSunday-test.js:149:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 3.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        11.0
    ); // d3-time/test/utcSunday-test.js:150:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 4.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        11.0
    ); // d3-time/test/utcSunday-test.js:151:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 0.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        45.0
    ); // d3-time/test/utcSunday-test.js:152:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        45.0
    ); // d3-time/test/utcSunday-test.js:153:3
    assert_eq!(
        Interval::Week(Weekday::Sunday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 2.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        45.0
    ); // d3-time/test/utcSunday-test.js:154:3
}

// d3-time/test/monday-test.js: timeMonday.floor(date) returns Mondays
#[test]
fn timemonday_floor_date_returns_mondays() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Monday, 1)
            .floor(local(&la, &[2011.0, 0.0, 1.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 27.0]))
    ); // d3-time/test/monday-test.js:10:3
    assert_eq!(
        ts(Interval::Week(Weekday::Monday, 1)
            .floor(local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 27.0]))
    ); // d3-time/test/monday-test.js:11:3
    assert_eq!(
        ts(Interval::Week(Weekday::Monday, 1)
            .floor(local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 27.0]))
    ); // d3-time/test/monday-test.js:12:3
    assert_eq!(
        ts(Interval::Week(Weekday::Monday, 1)
            .floor(local(&la, &[2011.0, 0.0, 2.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 27.0]))
    ); // d3-time/test/monday-test.js:13:3
    assert_eq!(
        ts(Interval::Week(Weekday::Monday, 1)
            .floor(local(&la, &[2011.0, 0.0, 3.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 3.0]))
    ); // d3-time/test/monday-test.js:14:3
    assert_eq!(
        ts(Interval::Week(Weekday::Monday, 1)
            .floor(local(&la, &[2011.0, 0.0, 3.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 3.0]))
    ); // d3-time/test/monday-test.js:15:3
}

// d3-time/test/monday-test.js: timeMonday.range(start, stop, step) returns every step Monday
#[test]
fn timemonday_range_start_stop_step_returns_every_step_monday() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Week(Weekday::Monday, 1)
            .range_step(
                local(&la, &[2011.0, 11.0, 1.0]),
                local(&la, &[2012.0, 0.0, 15.0]),
                2_u32,
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 11.0, 5.0]),
            local(&la, &[2011.0, 11.0, 19.0]),
            local(&la, &[2012.0, 0.0, 2.0])
        ])
    ); // d3-time/test/monday-test.js:19:3
}

// d3-time/test/monday-test.js: timeMonday.count(start, end) counts Mondays after start (exclusive) and before end (inclusive)
#[test]
fn timemonday_count_start_end_counts_mondays_after_start_exclusive_and_before_end_inclusive() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                local(&la, &[2014.0, 0.0, 1.0]),
                local(&la, &[2014.0, 0.0, 5.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/monday-test.js:34:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                local(&la, &[2014.0, 0.0, 1.0]),
                local(&la, &[2014.0, 0.0, 6.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/monday-test.js:35:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                local(&la, &[2014.0, 0.0, 1.0]),
                local(&la, &[2014.0, 0.0, 7.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/monday-test.js:36:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                local(&la, &[2014.0, 0.0, 1.0]),
                local(&la, &[2014.0, 0.0, 13.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        2.0
    ); // d3-time/test/monday-test.js:37:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                local(&la, &[2018.0, 0.0, 1.0]),
                local(&la, &[2018.0, 0.0, 7.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/monday-test.js:46:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                local(&la, &[2018.0, 0.0, 1.0]),
                local(&la, &[2018.0, 0.0, 8.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/monday-test.js:47:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                local(&la, &[2018.0, 0.0, 1.0]),
                local(&la, &[2018.0, 0.0, 9.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/monday-test.js:48:3
}

// d3-time/test/monday-test.js: timeMonday.count(start, end) observes daylight saving
#[test]
fn timemonday_count_start_end_observes_daylight_saving() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/monday-test.js:52:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 3.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/monday-test.js:53:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 4.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/monday-test.js:54:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 0.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/monday-test.js:55:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/monday-test.js:56:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 2.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/monday-test.js:57:3
}

// d3-time/test/utcMonday-test.js: utcMonday.floor(date) returns Mondays
#[test]
fn utcmonday_floor_date_returns_mondays() {
    assert_eq!(
        ts(Interval::Week(Weekday::Monday, 1)
            .floor(utc(&[2011.0, 0.0, 1.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 27.0]))
    ); // d3-time/test/utcMonday-test.js:10:3
    assert_eq!(
        ts(Interval::Week(Weekday::Monday, 1)
            .floor(utc(&[2011.0, 0.0, 2.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 27.0]))
    ); // d3-time/test/utcMonday-test.js:11:3
    assert_eq!(
        ts(Interval::Week(Weekday::Monday, 1)
            .floor(utc(&[2011.0, 0.0, 2.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 27.0]))
    ); // d3-time/test/utcMonday-test.js:12:3
    assert_eq!(
        ts(Interval::Week(Weekday::Monday, 1)
            .floor(utc(&[2011.0, 0.0, 2.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 27.0]))
    ); // d3-time/test/utcMonday-test.js:13:3
    assert_eq!(
        ts(Interval::Week(Weekday::Monday, 1)
            .floor(utc(&[2011.0, 0.0, 3.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 3.0]))
    ); // d3-time/test/utcMonday-test.js:14:3
    assert_eq!(
        ts(Interval::Week(Weekday::Monday, 1)
            .floor(utc(&[2011.0, 0.0, 3.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 3.0]))
    ); // d3-time/test/utcMonday-test.js:15:3
}

// d3-time/test/utcMonday-test.js: utcMonday.range(start, stop, step) returns every step Monday
#[test]
fn utcmonday_range_start_stop_step_returns_every_step_monday() {
    assert_eq!(
        tss(&Interval::Week(Weekday::Monday, 1)
            .range_step(
                utc(&[2011.0, 11.0, 1.0]),
                utc(&[2012.0, 0.0, 15.0]),
                2_u32,
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 11.0, 5.0]),
            utc(&[2011.0, 11.0, 19.0]),
            utc(&[2012.0, 0.0, 2.0])
        ])
    ); // d3-time/test/utcMonday-test.js:19:3
}

// d3-time/test/utcMonday-test.js: utcMonday.count(start, end) counts Mondays after start (exclusive) and before end (inclusive)
#[test]
fn utcmonday_count_start_end_counts_mondays_after_start_exclusive_and_before_end_inclusive() {
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                utc(&[2014.0, 0.0, 1.0]),
                utc(&[2014.0, 0.0, 5.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/utcMonday-test.js:34:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                utc(&[2014.0, 0.0, 1.0]),
                utc(&[2014.0, 0.0, 6.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcMonday-test.js:35:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                utc(&[2014.0, 0.0, 1.0]),
                utc(&[2014.0, 0.0, 7.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcMonday-test.js:36:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                utc(&[2014.0, 0.0, 1.0]),
                utc(&[2014.0, 0.0, 13.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        2.0
    ); // d3-time/test/utcMonday-test.js:37:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                utc(&[2018.0, 0.0, 1.0]),
                utc(&[2018.0, 0.0, 7.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/utcMonday-test.js:46:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                utc(&[2018.0, 0.0, 1.0]),
                utc(&[2018.0, 0.0, 8.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcMonday-test.js:47:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                utc(&[2018.0, 0.0, 1.0]),
                utc(&[2018.0, 0.0, 9.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcMonday-test.js:48:3
}

// d3-time/test/utcMonday-test.js: utcMonday.count(start, end) does not observe daylight saving
#[test]
fn utcmonday_count_start_end_does_not_observe_daylight_saving() {
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcMonday-test.js:52:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 3.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcMonday-test.js:53:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 4.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcMonday-test.js:54:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 0.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcMonday-test.js:55:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcMonday-test.js:56:3
    assert_eq!(
        Interval::Week(Weekday::Monday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 2.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcMonday-test.js:57:3
}

// d3-time/test/tuesday-test.js: timeTuesday.floor(date) returns Tuesdays
#[test]
fn timetuesday_floor_date_returns_tuesdays() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Tuesday, 1)
            .floor(local(&la, &[2011.0, 0.0, 2.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 28.0]))
    ); // d3-time/test/tuesday-test.js:10:3
    assert_eq!(
        ts(Interval::Week(Weekday::Tuesday, 1)
            .floor(local(&la, &[2011.0, 0.0, 3.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 28.0]))
    ); // d3-time/test/tuesday-test.js:11:3
    assert_eq!(
        ts(Interval::Week(Weekday::Tuesday, 1)
            .floor(local(&la, &[2011.0, 0.0, 3.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 28.0]))
    ); // d3-time/test/tuesday-test.js:12:3
    assert_eq!(
        ts(Interval::Week(Weekday::Tuesday, 1)
            .floor(local(&la, &[2011.0, 0.0, 3.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 28.0]))
    ); // d3-time/test/tuesday-test.js:13:3
    assert_eq!(
        ts(Interval::Week(Weekday::Tuesday, 1)
            .floor(local(&la, &[2011.0, 0.0, 4.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 4.0]))
    ); // d3-time/test/tuesday-test.js:14:3
    assert_eq!(
        ts(Interval::Week(Weekday::Tuesday, 1)
            .floor(local(&la, &[2011.0, 0.0, 4.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 4.0]))
    ); // d3-time/test/tuesday-test.js:15:3
}

// d3-time/test/tuesday-test.js: timeTuesday.count(start, end) counts Tuesdays after start (exclusive) and before end (inclusive)
#[test]
fn timetuesday_count_start_end_counts_tuesdays_after_start_exclusive_and_before_end_inclusive() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                local(&la, &[2014.0, 0.0, 1.0]),
                local(&la, &[2014.0, 0.0, 6.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/tuesday-test.js:26:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                local(&la, &[2014.0, 0.0, 1.0]),
                local(&la, &[2014.0, 0.0, 7.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/tuesday-test.js:27:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                local(&la, &[2014.0, 0.0, 1.0]),
                local(&la, &[2014.0, 0.0, 8.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/tuesday-test.js:28:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                local(&la, &[2014.0, 0.0, 1.0]),
                local(&la, &[2014.0, 0.0, 14.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        2.0
    ); // d3-time/test/tuesday-test.js:29:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                local(&la, &[2013.0, 0.0, 1.0]),
                local(&la, &[2013.0, 0.0, 7.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/tuesday-test.js:38:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                local(&la, &[2013.0, 0.0, 1.0]),
                local(&la, &[2013.0, 0.0, 8.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/tuesday-test.js:39:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                local(&la, &[2013.0, 0.0, 1.0]),
                local(&la, &[2013.0, 0.0, 9.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/tuesday-test.js:40:3
}

// d3-time/test/tuesday-test.js: timeTuesday.count(start, end) observes daylight saving
#[test]
fn timetuesday_count_start_end_observes_daylight_saving() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/tuesday-test.js:44:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 3.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/tuesday-test.js:45:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 4.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/tuesday-test.js:46:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 0.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/tuesday-test.js:47:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/tuesday-test.js:48:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 2.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/tuesday-test.js:49:3
}

// d3-time/test/utcTuesday-test.js: utcTuesday.floor(date) returns Tuesdays
#[test]
fn utctuesday_floor_date_returns_tuesdays() {
    assert_eq!(
        ts(Interval::Week(Weekday::Tuesday, 1)
            .floor(utc(&[2011.0, 0.0, 2.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 28.0]))
    ); // d3-time/test/utcTuesday-test.js:10:3
    assert_eq!(
        ts(Interval::Week(Weekday::Tuesday, 1)
            .floor(utc(&[2011.0, 0.0, 3.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 28.0]))
    ); // d3-time/test/utcTuesday-test.js:11:3
    assert_eq!(
        ts(Interval::Week(Weekday::Tuesday, 1)
            .floor(utc(&[2011.0, 0.0, 3.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 28.0]))
    ); // d3-time/test/utcTuesday-test.js:12:3
    assert_eq!(
        ts(Interval::Week(Weekday::Tuesday, 1)
            .floor(utc(&[2011.0, 0.0, 3.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 28.0]))
    ); // d3-time/test/utcTuesday-test.js:13:3
    assert_eq!(
        ts(Interval::Week(Weekday::Tuesday, 1)
            .floor(utc(&[2011.0, 0.0, 4.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 4.0]))
    ); // d3-time/test/utcTuesday-test.js:14:3
    assert_eq!(
        ts(Interval::Week(Weekday::Tuesday, 1)
            .floor(utc(&[2011.0, 0.0, 4.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 4.0]))
    ); // d3-time/test/utcTuesday-test.js:15:3
}

// d3-time/test/utcTuesday-test.js: utcTuesday.count(start, end) counts Tuesdays after start (exclusive) and before end (inclusive)
#[test]
fn utctuesday_count_start_end_counts_tuesdays_after_start_exclusive_and_before_end_inclusive() {
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                utc(&[2014.0, 0.0, 1.0]),
                utc(&[2014.0, 0.0, 6.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/utcTuesday-test.js:26:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                utc(&[2014.0, 0.0, 1.0]),
                utc(&[2014.0, 0.0, 7.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcTuesday-test.js:27:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                utc(&[2014.0, 0.0, 1.0]),
                utc(&[2014.0, 0.0, 8.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcTuesday-test.js:28:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                utc(&[2014.0, 0.0, 1.0]),
                utc(&[2014.0, 0.0, 14.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        2.0
    ); // d3-time/test/utcTuesday-test.js:29:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                utc(&[2013.0, 0.0, 1.0]),
                utc(&[2013.0, 0.0, 7.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/utcTuesday-test.js:38:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                utc(&[2013.0, 0.0, 1.0]),
                utc(&[2013.0, 0.0, 8.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcTuesday-test.js:39:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                utc(&[2013.0, 0.0, 1.0]),
                utc(&[2013.0, 0.0, 9.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcTuesday-test.js:40:3
}

// d3-time/test/utcTuesday-test.js: utcTuesday.count(start, end) does not observe daylight saving
#[test]
fn utctuesday_count_start_end_does_not_observe_daylight_saving() {
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcTuesday-test.js:44:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 3.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcTuesday-test.js:45:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 4.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcTuesday-test.js:46:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 0.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcTuesday-test.js:47:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcTuesday-test.js:48:3
    assert_eq!(
        Interval::Week(Weekday::Tuesday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 2.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcTuesday-test.js:49:3
}

// d3-time/test/wednesday-test.js: timeWednesday.floor(date) returns Wednesdays
#[test]
fn timewednesday_floor_date_returns_wednesdays() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Wednesday, 1)
            .floor(local(&la, &[2011.0, 0.0, 3.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 29.0]))
    ); // d3-time/test/wednesday-test.js:10:3
    assert_eq!(
        ts(Interval::Week(Weekday::Wednesday, 1)
            .floor(local(&la, &[2011.0, 0.0, 4.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 29.0]))
    ); // d3-time/test/wednesday-test.js:11:3
    assert_eq!(
        ts(Interval::Week(Weekday::Wednesday, 1)
            .floor(local(&la, &[2011.0, 0.0, 4.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 29.0]))
    ); // d3-time/test/wednesday-test.js:12:3
    assert_eq!(
        ts(Interval::Week(Weekday::Wednesday, 1)
            .floor(local(&la, &[2011.0, 0.0, 4.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 29.0]))
    ); // d3-time/test/wednesday-test.js:13:3
    assert_eq!(
        ts(Interval::Week(Weekday::Wednesday, 1)
            .floor(local(&la, &[2011.0, 0.0, 5.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 5.0]))
    ); // d3-time/test/wednesday-test.js:14:3
    assert_eq!(
        ts(Interval::Week(Weekday::Wednesday, 1)
            .floor(local(&la, &[2011.0, 0.0, 5.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 5.0]))
    ); // d3-time/test/wednesday-test.js:15:3
}

// d3-time/test/wednesday-test.js: timeWednesday.count(start, end) counts Wednesdays after start (exclusive) and before end (inclusive)
#[test]
fn timewednesday_count_start_end_counts_wednesdays_after_start_exclusive_and_before_end_inclusive()
{
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 3.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/wednesday-test.js:26:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 4.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/wednesday-test.js:27:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 5.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/wednesday-test.js:28:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 11.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        2.0
    ); // d3-time/test/wednesday-test.js:29:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                local(&la, &[2014.0, 0.0, 1.0]),
                local(&la, &[2014.0, 0.0, 7.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/wednesday-test.js:38:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                local(&la, &[2014.0, 0.0, 1.0]),
                local(&la, &[2014.0, 0.0, 8.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/wednesday-test.js:39:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                local(&la, &[2014.0, 0.0, 1.0]),
                local(&la, &[2014.0, 0.0, 9.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/wednesday-test.js:40:3
}

// d3-time/test/wednesday-test.js: timeWednesday.count(start, end) observes daylight saving
#[test]
fn timewednesday_count_start_end_observes_daylight_saving() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/wednesday-test.js:44:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 3.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/wednesday-test.js:45:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 4.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/wednesday-test.js:46:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 0.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/wednesday-test.js:47:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/wednesday-test.js:48:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 2.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/wednesday-test.js:49:3
}

// d3-time/test/utcWednesday-test.js: utcWednesday.floor(date) returns Wednesdays
#[test]
fn utcwednesday_floor_date_returns_wednesdays() {
    assert_eq!(
        ts(Interval::Week(Weekday::Wednesday, 1)
            .floor(utc(&[2011.0, 0.0, 3.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 29.0]))
    ); // d3-time/test/utcWednesday-test.js:10:3
    assert_eq!(
        ts(Interval::Week(Weekday::Wednesday, 1)
            .floor(utc(&[2011.0, 0.0, 4.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 29.0]))
    ); // d3-time/test/utcWednesday-test.js:11:3
    assert_eq!(
        ts(Interval::Week(Weekday::Wednesday, 1)
            .floor(utc(&[2011.0, 0.0, 4.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 29.0]))
    ); // d3-time/test/utcWednesday-test.js:12:3
    assert_eq!(
        ts(Interval::Week(Weekday::Wednesday, 1)
            .floor(utc(&[2011.0, 0.0, 4.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 29.0]))
    ); // d3-time/test/utcWednesday-test.js:13:3
    assert_eq!(
        ts(Interval::Week(Weekday::Wednesday, 1)
            .floor(utc(&[2011.0, 0.0, 5.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 5.0]))
    ); // d3-time/test/utcWednesday-test.js:14:3
    assert_eq!(
        ts(Interval::Week(Weekday::Wednesday, 1)
            .floor(utc(&[2011.0, 0.0, 5.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 5.0]))
    ); // d3-time/test/utcWednesday-test.js:15:3
}

// d3-time/test/utcWednesday-test.js: utcWednesday.count(start, end) counts Wednesdays after start (exclusive) and before end (inclusive)
#[test]
fn utcwednesday_count_start_end_counts_wednesdays_after_start_exclusive_and_before_end_inclusive() {
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 3.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/utcWednesday-test.js:26:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 4.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcWednesday-test.js:27:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 5.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcWednesday-test.js:28:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 11.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        2.0
    ); // d3-time/test/utcWednesday-test.js:29:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                utc(&[2014.0, 0.0, 1.0]),
                utc(&[2014.0, 0.0, 7.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/utcWednesday-test.js:38:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                utc(&[2014.0, 0.0, 1.0]),
                utc(&[2014.0, 0.0, 8.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcWednesday-test.js:39:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                utc(&[2014.0, 0.0, 1.0]),
                utc(&[2014.0, 0.0, 9.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcWednesday-test.js:40:3
}

// d3-time/test/utcWednesday-test.js: utcWednesday.count(start, end) does not observe daylight saving
#[test]
fn utcwednesday_count_start_end_does_not_observe_daylight_saving() {
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcWednesday-test.js:44:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 3.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcWednesday-test.js:45:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 4.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcWednesday-test.js:46:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 0.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcWednesday-test.js:47:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcWednesday-test.js:48:3
    assert_eq!(
        Interval::Week(Weekday::Wednesday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 2.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcWednesday-test.js:49:3
}

// d3-time/test/thursday-test.js: timeThursday.floor(date) returns Thursdays
#[test]
fn timethursday_floor_date_returns_thursdays() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Thursday, 1)
            .floor(local(&la, &[2011.0, 0.0, 4.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 30.0]))
    ); // d3-time/test/thursday-test.js:10:3
    assert_eq!(
        ts(Interval::Week(Weekday::Thursday, 1)
            .floor(local(&la, &[2011.0, 0.0, 5.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 30.0]))
    ); // d3-time/test/thursday-test.js:11:3
    assert_eq!(
        ts(Interval::Week(Weekday::Thursday, 1)
            .floor(local(&la, &[2011.0, 0.0, 5.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 30.0]))
    ); // d3-time/test/thursday-test.js:12:3
    assert_eq!(
        ts(Interval::Week(Weekday::Thursday, 1)
            .floor(local(&la, &[2011.0, 0.0, 5.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 30.0]))
    ); // d3-time/test/thursday-test.js:13:3
    assert_eq!(
        ts(Interval::Week(Weekday::Thursday, 1)
            .floor(local(&la, &[2011.0, 0.0, 6.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 6.0]))
    ); // d3-time/test/thursday-test.js:14:3
    assert_eq!(
        ts(Interval::Week(Weekday::Thursday, 1)
            .floor(local(&la, &[2011.0, 0.0, 6.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 6.0]))
    ); // d3-time/test/thursday-test.js:15:3
}

// d3-time/test/thursday-test.js: timeThursday.count(start, end) counts Thursdays after start (exclusive) and before end (inclusive)
#[test]
fn timethursday_count_start_end_counts_thursdays_after_start_exclusive_and_before_end_inclusive() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 4.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/thursday-test.js:26:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 5.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/thursday-test.js:27:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 6.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/thursday-test.js:28:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 12.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        2.0
    ); // d3-time/test/thursday-test.js:29:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                local(&la, &[2015.0, 0.0, 1.0]),
                local(&la, &[2015.0, 0.0, 7.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/thursday-test.js:38:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                local(&la, &[2015.0, 0.0, 1.0]),
                local(&la, &[2015.0, 0.0, 8.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/thursday-test.js:39:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                local(&la, &[2015.0, 0.0, 1.0]),
                local(&la, &[2015.0, 0.0, 9.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/thursday-test.js:40:3
}

// d3-time/test/thursday-test.js: timeThursday.count(start, end) observes daylight saving
#[test]
fn timethursday_count_start_end_observes_daylight_saving() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/thursday-test.js:44:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 3.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/thursday-test.js:45:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 4.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/thursday-test.js:46:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 0.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/thursday-test.js:47:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/thursday-test.js:48:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 2.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/thursday-test.js:49:3
}

// d3-time/test/utcThursday-test.js: utcThursday.floor(date) returns Thursdays
#[test]
fn utcthursday_floor_date_returns_thursdays() {
    assert_eq!(
        ts(Interval::Week(Weekday::Thursday, 1)
            .floor(utc(&[2011.0, 0.0, 4.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 30.0]))
    ); // d3-time/test/utcThursday-test.js:10:3
    assert_eq!(
        ts(Interval::Week(Weekday::Thursday, 1)
            .floor(utc(&[2011.0, 0.0, 5.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 30.0]))
    ); // d3-time/test/utcThursday-test.js:11:3
    assert_eq!(
        ts(Interval::Week(Weekday::Thursday, 1)
            .floor(utc(&[2011.0, 0.0, 5.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 30.0]))
    ); // d3-time/test/utcThursday-test.js:12:3
    assert_eq!(
        ts(Interval::Week(Weekday::Thursday, 1)
            .floor(utc(&[2011.0, 0.0, 5.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 30.0]))
    ); // d3-time/test/utcThursday-test.js:13:3
    assert_eq!(
        ts(Interval::Week(Weekday::Thursday, 1)
            .floor(utc(&[2011.0, 0.0, 6.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 6.0]))
    ); // d3-time/test/utcThursday-test.js:14:3
    assert_eq!(
        ts(Interval::Week(Weekday::Thursday, 1)
            .floor(utc(&[2011.0, 0.0, 6.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 6.0]))
    ); // d3-time/test/utcThursday-test.js:15:3
}

// d3-time/test/utcThursday-test.js: utcThursday.count(start, end) counts Thursdays after start (exclusive) and before end (inclusive)
#[test]
fn utcthursday_count_start_end_counts_thursdays_after_start_exclusive_and_before_end_inclusive() {
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 4.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/utcThursday-test.js:26:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 5.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcThursday-test.js:27:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 6.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcThursday-test.js:28:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 12.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        2.0
    ); // d3-time/test/utcThursday-test.js:29:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                utc(&[2015.0, 0.0, 1.0]),
                utc(&[2015.0, 0.0, 7.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/utcThursday-test.js:38:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                utc(&[2015.0, 0.0, 1.0]),
                utc(&[2015.0, 0.0, 8.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcThursday-test.js:39:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                utc(&[2015.0, 0.0, 1.0]),
                utc(&[2015.0, 0.0, 9.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcThursday-test.js:40:3
}

// d3-time/test/utcThursday-test.js: utcThursday.count(start, end) does not observe daylight saving
#[test]
fn utcthursday_count_start_end_does_not_observe_daylight_saving() {
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcThursday-test.js:44:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 3.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcThursday-test.js:45:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 4.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcThursday-test.js:46:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 0.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcThursday-test.js:47:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcThursday-test.js:48:3
    assert_eq!(
        Interval::Week(Weekday::Thursday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 2.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcThursday-test.js:49:3
}

// d3-time/test/friday-test.js: timeFriday.floor(date) returns Fridays
#[test]
fn timefriday_floor_date_returns_fridays() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Friday, 1)
            .floor(local(&la, &[2011.0, 0.0, 5.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0]))
    ); // d3-time/test/friday-test.js:10:3
    assert_eq!(
        ts(Interval::Week(Weekday::Friday, 1)
            .floor(local(&la, &[2011.0, 0.0, 6.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0]))
    ); // d3-time/test/friday-test.js:11:3
    assert_eq!(
        ts(Interval::Week(Weekday::Friday, 1)
            .floor(local(&la, &[2011.0, 0.0, 6.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0]))
    ); // d3-time/test/friday-test.js:12:3
    assert_eq!(
        ts(Interval::Week(Weekday::Friday, 1)
            .floor(local(&la, &[2011.0, 0.0, 6.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 31.0]))
    ); // d3-time/test/friday-test.js:13:3
    assert_eq!(
        ts(Interval::Week(Weekday::Friday, 1)
            .floor(local(&la, &[2011.0, 0.0, 7.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 7.0]))
    ); // d3-time/test/friday-test.js:14:3
    assert_eq!(
        ts(Interval::Week(Weekday::Friday, 1)
            .floor(local(&la, &[2011.0, 0.0, 7.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 7.0]))
    ); // d3-time/test/friday-test.js:15:3
}

// d3-time/test/friday-test.js: timeFriday.count(start, end) counts Fridays after start (exclusive) and before end (inclusive)
#[test]
fn timefriday_count_start_end_counts_fridays_after_start_exclusive_and_before_end_inclusive() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 5.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/friday-test.js:26:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 6.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/friday-test.js:27:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 7.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/friday-test.js:28:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 13.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        2.0
    ); // d3-time/test/friday-test.js:29:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                local(&la, &[2010.0, 0.0, 1.0]),
                local(&la, &[2010.0, 0.0, 7.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/friday-test.js:39:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                local(&la, &[2010.0, 0.0, 1.0]),
                local(&la, &[2010.0, 0.0, 8.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/friday-test.js:40:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                local(&la, &[2010.0, 0.0, 1.0]),
                local(&la, &[2010.0, 0.0, 9.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/friday-test.js:41:3
}

// d3-time/test/friday-test.js: timeFriday.count(start, end) observes daylight saving
#[test]
fn timefriday_count_start_end_observes_daylight_saving() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/friday-test.js:45:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 3.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/friday-test.js:46:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 4.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/friday-test.js:47:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 0.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/friday-test.js:48:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/friday-test.js:49:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 2.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/friday-test.js:50:3
}

// d3-time/test/utcFriday-test.js: utcFriday.floor(date) returns Fridays
#[test]
fn utcfriday_floor_date_returns_fridays() {
    assert_eq!(
        ts(Interval::Week(Weekday::Friday, 1)
            .floor(utc(&[2011.0, 0.0, 5.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0]))
    ); // d3-time/test/utcFriday-test.js:10:3
    assert_eq!(
        ts(Interval::Week(Weekday::Friday, 1)
            .floor(utc(&[2011.0, 0.0, 6.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0]))
    ); // d3-time/test/utcFriday-test.js:11:3
    assert_eq!(
        ts(Interval::Week(Weekday::Friday, 1)
            .floor(utc(&[2011.0, 0.0, 6.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0]))
    ); // d3-time/test/utcFriday-test.js:12:3
    assert_eq!(
        ts(Interval::Week(Weekday::Friday, 1)
            .floor(utc(&[2011.0, 0.0, 6.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 31.0]))
    ); // d3-time/test/utcFriday-test.js:13:3
    assert_eq!(
        ts(Interval::Week(Weekday::Friday, 1)
            .floor(utc(&[2011.0, 0.0, 7.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 7.0]))
    ); // d3-time/test/utcFriday-test.js:14:3
    assert_eq!(
        ts(Interval::Week(Weekday::Friday, 1)
            .floor(utc(&[2011.0, 0.0, 7.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 7.0]))
    ); // d3-time/test/utcFriday-test.js:15:3
}

// d3-time/test/utcFriday-test.js: utcFriday.count(start, end) counts Fridays after start (exclusive) and before end (inclusive)
#[test]
fn utcfriday_count_start_end_counts_fridays_after_start_exclusive_and_before_end_inclusive() {
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 5.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/utcFriday-test.js:26:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 6.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcFriday-test.js:27:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 7.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcFriday-test.js:28:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 13.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        2.0
    ); // d3-time/test/utcFriday-test.js:29:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                utc(&[2010.0, 0.0, 1.0]),
                utc(&[2010.0, 0.0, 7.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/utcFriday-test.js:39:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                utc(&[2010.0, 0.0, 1.0]),
                utc(&[2010.0, 0.0, 8.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcFriday-test.js:40:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                utc(&[2010.0, 0.0, 1.0]),
                utc(&[2010.0, 0.0, 9.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcFriday-test.js:41:3
}

// d3-time/test/utcFriday-test.js: utcFriday.count(start, end) does not observe daylight saving
#[test]
fn utcfriday_count_start_end_does_not_observe_daylight_saving() {
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcFriday-test.js:45:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 3.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcFriday-test.js:46:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 4.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcFriday-test.js:47:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 0.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcFriday-test.js:48:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcFriday-test.js:49:3
    assert_eq!(
        Interval::Week(Weekday::Friday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 2.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcFriday-test.js:50:3
}

// d3-time/test/saturday-test.js: timeSaturday.floor(date) returns Saturdays
#[test]
fn timesaturday_floor_date_returns_saturdays() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Week(Weekday::Saturday, 1)
            .floor(local(&la, &[2011.0, 0.0, 6.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/saturday-test.js:10:3
    assert_eq!(
        ts(Interval::Week(Weekday::Saturday, 1)
            .floor(local(&la, &[2011.0, 0.0, 7.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/saturday-test.js:11:3
    assert_eq!(
        ts(Interval::Week(Weekday::Saturday, 1)
            .floor(local(&la, &[2011.0, 0.0, 7.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/saturday-test.js:12:3
    assert_eq!(
        ts(Interval::Week(Weekday::Saturday, 1)
            .floor(local(&la, &[2011.0, 0.0, 7.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/saturday-test.js:13:3
    assert_eq!(
        ts(Interval::Week(Weekday::Saturday, 1)
            .floor(local(&la, &[2011.0, 0.0, 8.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 8.0]))
    ); // d3-time/test/saturday-test.js:14:3
    assert_eq!(
        ts(Interval::Week(Weekday::Saturday, 1)
            .floor(local(&la, &[2011.0, 0.0, 8.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 8.0]))
    ); // d3-time/test/saturday-test.js:15:3
}

// d3-time/test/saturday-test.js: timeSaturday.count(start, end) counts Saturdays after start (exclusive) and before end (inclusive)
#[test]
fn timesaturday_count_start_end_counts_saturdays_after_start_exclusive_and_before_end_inclusive() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 6.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/saturday-test.js:26:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 7.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/saturday-test.js:27:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 8.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/saturday-test.js:28:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                local(&la, &[2012.0, 0.0, 1.0]),
                local(&la, &[2012.0, 0.0, 14.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        2.0
    ); // d3-time/test/saturday-test.js:29:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 0.0, 7.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/saturday-test.js:39:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 0.0, 8.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/saturday-test.js:40:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 0.0, 9.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/saturday-test.js:41:3
}

// d3-time/test/saturday-test.js: timeSaturday.count(start, end) observes daylight saving
#[test]
fn timesaturday_count_start_end_observes_daylight_saving() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/saturday-test.js:45:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 3.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/saturday-test.js:46:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 2.0, 13.0, 4.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/saturday-test.js:47:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 0.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/saturday-test.js:48:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/saturday-test.js:49:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 10.0, 6.0, 2.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/saturday-test.js:50:3
}

// d3-time/test/utcSaturday-test.js: utcSaturday.floor(date) returns Saturdays
#[test]
fn utcsaturday_floor_date_returns_saturdays() {
    assert_eq!(
        ts(Interval::Week(Weekday::Saturday, 1)
            .floor(utc(&[2011.0, 0.0, 6.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcSaturday-test.js:10:3
    assert_eq!(
        ts(Interval::Week(Weekday::Saturday, 1)
            .floor(utc(&[2011.0, 0.0, 7.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcSaturday-test.js:11:3
    assert_eq!(
        ts(Interval::Week(Weekday::Saturday, 1)
            .floor(utc(&[2011.0, 0.0, 7.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcSaturday-test.js:12:3
    assert_eq!(
        ts(Interval::Week(Weekday::Saturday, 1)
            .floor(utc(&[2011.0, 0.0, 7.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcSaturday-test.js:13:3
    assert_eq!(
        ts(Interval::Week(Weekday::Saturday, 1)
            .floor(utc(&[2011.0, 0.0, 8.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 8.0]))
    ); // d3-time/test/utcSaturday-test.js:14:3
    assert_eq!(
        ts(Interval::Week(Weekday::Saturday, 1)
            .floor(utc(&[2011.0, 0.0, 8.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 8.0]))
    ); // d3-time/test/utcSaturday-test.js:15:3
}

// d3-time/test/utcSaturday-test.js: utcSaturday.count(start, end) counts Saturdays after start (exclusive) and before end (inclusive)
#[test]
fn utcsaturday_count_start_end_counts_saturdays_after_start_exclusive_and_before_end_inclusive() {
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 6.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/utcSaturday-test.js:26:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 7.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcSaturday-test.js:27:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 8.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcSaturday-test.js:28:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                utc(&[2012.0, 0.0, 1.0]),
                utc(&[2012.0, 0.0, 14.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        2.0
    ); // d3-time/test/utcSaturday-test.js:29:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 0.0, 7.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        0.0
    ); // d3-time/test/utcSaturday-test.js:39:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 0.0, 8.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcSaturday-test.js:40:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 0.0, 9.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        1.0
    ); // d3-time/test/utcSaturday-test.js:41:3
}

// d3-time/test/utcSaturday-test.js: utcSaturday.count(start, end) does not observe daylight saving
#[test]
fn utcsaturday_count_start_end_does_not_observe_daylight_saving() {
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcSaturday-test.js:45:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 3.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcSaturday-test.js:46:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 2.0, 13.0, 4.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        10.0
    ); // d3-time/test/utcSaturday-test.js:47:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 0.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcSaturday-test.js:48:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcSaturday-test.js:49:3
    assert_eq!(
        Interval::Week(Weekday::Saturday, 1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 10.0, 6.0, 2.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        44.0
    ); // d3-time/test/utcSaturday-test.js:50:3
}

// d3-time/test/month-test.js: timeMonth.floor(date) returns months
#[test]
fn timemonth_floor_date_returns_months() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).floor(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 1.0]))
    ); // d3-time/test/month-test.js:10:3
    assert_eq!(
        ts(Interval::Month(1).floor(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/month-test.js:11:3
    assert_eq!(
        ts(Interval::Month(1).floor(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/month-test.js:12:3
}

// d3-time/test/month-test.js: timeMonth.floor(date) observes the start of daylight savings time
#[test]
fn timemonth_floor_date_observes_the_start_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).floor(local(&la, &[2011.0, 2.0, 13.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 1.0]))
    ); // d3-time/test/month-test.js:16:3
}

// d3-time/test/month-test.js: timeMonth.floor(date) observes the end of the daylight savings time
#[test]
fn timemonth_floor_date_observes_the_end_of_the_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).floor(local(&la, &[2011.0, 10.0, 6.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 1.0]))
    ); // d3-time/test/month-test.js:20:3
}

// d3-time/test/month-test.js: timeMonth.floor(date) correctly handles years in the first century
#[test]
fn timemonth_floor_date_correctly_handles_years_in_the_first_century() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).floor(local(&la, &[9.0, 10.0, 6.0, 7.0]), &la)),
        ts(local(&la, &[9.0, 10.0, 1.0]))
    ); // d3-time/test/month-test.js:24:3
}

// d3-time/test/month-test.js: timeMonth.ceil(date) returns months
#[test]
fn timemonth_ceil_date_returns_months() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).ceil(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/month-test.js:28:3
    assert_eq!(
        ts(Interval::Month(1).ceil(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/month-test.js:29:3
    assert_eq!(
        ts(Interval::Month(1).ceil(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 1.0, 1.0]))
    ); // d3-time/test/month-test.js:30:3
}

// d3-time/test/month-test.js: timeMonth.ceil(date) observes the start of daylight savings time
#[test]
fn timemonth_ceil_date_observes_the_start_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).ceil(local(&la, &[2011.0, 2.0, 13.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 3.0, 1.0]))
    ); // d3-time/test/month-test.js:34:3
}

// d3-time/test/month-test.js: timeMonth.ceil(date) observes the end of the daylight savings time
#[test]
fn timemonth_ceil_date_observes_the_end_of_the_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).ceil(local(&la, &[2011.0, 10.0, 6.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 11.0, 1.0]))
    ); // d3-time/test/month-test.js:38:3
}

// d3-time/test/month-test.js: timeMonth.offset(date) does not modify the passed-in date
#[test]
fn timemonth_offset_date_does_not_modify_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    let d = local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Month(1).offset(d, 1_i64, &la);
    assert_eq!(
        ts(d),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/month-test.js:44:3
}

// d3-time/test/month-test.js: timeMonth.offset(date) does not round the passed-in-date
#[test]
fn timemonth_offset_date_does_not_round_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/month-test.js:48:3
    assert_eq!(
        ts(Interval::Month(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 9.0, 31.0, 23.0, 59.0, 59.0, 456.0]))
    ); // d3-time/test/month-test.js:49:3
}

// d3-time/test/month-test.js: timeMonth.offset(date) allows negative offsets
#[test]
fn timemonth_offset_date_allows_negative_offsets() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).offset(local(&la, &[2010.0, 11.0, 1.0]), -1_i64, &la)),
        ts(local(&la, &[2010.0, 10.0, 1.0]))
    ); // d3-time/test/month-test.js:53:3
    assert_eq!(
        ts(Interval::Month(1).offset(local(&la, &[2011.0, 0.0, 1.0]), -2_i64, &la)),
        ts(local(&la, &[2010.0, 10.0, 1.0]))
    ); // d3-time/test/month-test.js:54:3
    assert_eq!(
        ts(Interval::Month(1).offset(local(&la, &[2011.0, 0.0, 1.0]), -1_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 1.0]))
    ); // d3-time/test/month-test.js:55:3
}

// d3-time/test/month-test.js: timeMonth.offset(date) allows positive offsets
#[test]
fn timemonth_offset_date_allows_positive_offsets() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).offset(local(&la, &[2010.0, 10.0, 1.0]), 1_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 1.0]))
    ); // d3-time/test/month-test.js:59:3
    assert_eq!(
        ts(Interval::Month(1).offset(local(&la, &[2010.0, 10.0, 1.0]), 2_i64, &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/month-test.js:60:3
    assert_eq!(
        ts(Interval::Month(1).offset(local(&la, &[2010.0, 11.0, 1.0]), 1_i64, &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/month-test.js:61:3
}

// d3-time/test/month-test.js: timeMonth.offset(date) allows zero offset
#[test]
fn timemonth_offset_date_allows_zero_offset() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/month-test.js:65:3
    assert_eq!(
        ts(Interval::Month(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/month-test.js:66:3
}

// d3-time/test/month-test.js: timeMonth.floor(date) returns months
#[test]
fn timemonth_floor_date_returns_months_2() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).floor(local(&la, &[2010.0, 11.0, 31.0, 23.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 1.0]))
    ); // d3-time/test/month-test.js:74:3
    assert_eq!(
        ts(Interval::Month(1).floor(local(&la, &[2011.0, 0.0, 1.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/month-test.js:75:3
    assert_eq!(
        ts(Interval::Month(1).floor(local(&la, &[2011.0, 0.0, 1.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/month-test.js:76:3
}

// d3-time/test/month-test.js: timeMonth.floor(date) observes daylight saving
#[test]
fn timemonth_floor_date_observes_daylight_saving() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 2.0, 13.0, 7.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 1.0]))
    ); // d3-time/test/month-test.js:80:3
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 2.0, 13.0, 8.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 1.0]))
    ); // d3-time/test/month-test.js:81:3
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 2.0, 13.0, 9.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 1.0]))
    ); // d3-time/test/month-test.js:82:3
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 2.0, 13.0, 10.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 1.0]))
    ); // d3-time/test/month-test.js:83:3
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 10.0, 6.0, 7.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 1.0]))
    ); // d3-time/test/month-test.js:84:3
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 10.0, 6.0, 8.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 1.0]))
    ); // d3-time/test/month-test.js:85:3
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 10.0, 6.0, 9.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 1.0]))
    ); // d3-time/test/month-test.js:86:3
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 10.0, 6.0, 10.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 1.0]))
    ); // d3-time/test/month-test.js:87:3
}

// d3-time/test/month-test.js: timeMonth.floor(date) handles years in the first century
#[test]
fn timemonth_floor_date_handles_years_in_the_first_century() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).floor(local(&la, &[9.0, 10.0, 6.0, 7.0]), &la)),
        ts(local(&la, &[9.0, 10.0, 1.0]))
    ); // d3-time/test/month-test.js:91:3
}

// d3-time/test/month-test.js: timeMonth.round(date) returns months
#[test]
fn timemonth_round_date_returns_months() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).round(local(&la, &[2010.0, 11.0, 16.0, 12.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/month-test.js:95:3
    assert_eq!(
        ts(Interval::Month(1).round(local(&la, &[2010.0, 11.0, 16.0, 11.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 1.0]))
    ); // d3-time/test/month-test.js:96:3
}

// d3-time/test/month-test.js: timeMonth.round(date) observes daylight saving
#[test]
fn timemonth_round_date_observes_daylight_saving() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2011.0, 2.0, 13.0, 7.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 1.0]))
    ); // d3-time/test/month-test.js:100:3
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2011.0, 2.0, 13.0, 8.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 1.0]))
    ); // d3-time/test/month-test.js:101:3
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2011.0, 2.0, 13.0, 9.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 1.0]))
    ); // d3-time/test/month-test.js:102:3
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2011.0, 2.0, 13.0, 20.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 1.0]))
    ); // d3-time/test/month-test.js:103:3
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2011.0, 10.0, 6.0, 7.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 1.0]))
    ); // d3-time/test/month-test.js:104:3
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2011.0, 10.0, 6.0, 8.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 1.0]))
    ); // d3-time/test/month-test.js:105:3
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2011.0, 10.0, 6.0, 9.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 1.0]))
    ); // d3-time/test/month-test.js:106:3
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2011.0, 10.0, 6.0, 20.0]), &la)),
        ts(local(&la, &[2011.0, 10.0, 1.0]))
    ); // d3-time/test/month-test.js:107:3
}

// d3-time/test/month-test.js: timeMonth.round(date) handles midnight for leap years
#[test]
fn timemonth_round_date_handles_midnight_for_leap_years() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2012.0, 2.0, 1.0, 0.0]), &la)),
        ts(local(&la, &[2012.0, 2.0, 1.0]))
    ); // d3-time/test/month-test.js:111:3
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2012.0, 2.0, 1.0, 0.0]), &la)),
        ts(local(&la, &[2012.0, 2.0, 1.0]))
    ); // d3-time/test/month-test.js:112:3
}

// d3-time/test/month-test.js: timeMonth.ceil(date) returns months
#[test]
fn timemonth_ceil_date_returns_months_2() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).ceil(local(&la, &[2010.0, 10.0, 30.0, 23.0]), &la)),
        ts(local(&la, &[2010.0, 11.0, 1.0]))
    ); // d3-time/test/month-test.js:116:3
    assert_eq!(
        ts(Interval::Month(1).ceil(local(&la, &[2010.0, 11.0, 1.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/month-test.js:117:3
    assert_eq!(
        ts(Interval::Month(1).ceil(local(&la, &[2011.0, 1.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 1.0, 1.0]))
    ); // d3-time/test/month-test.js:118:3
    assert_eq!(
        ts(Interval::Month(1).ceil(local(&la, &[2011.0, 2.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 2.0, 1.0]))
    ); // d3-time/test/month-test.js:119:3
    assert_eq!(
        ts(Interval::Month(1).ceil(local(&la, &[2011.0, 3.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 3.0, 1.0]))
    ); // d3-time/test/month-test.js:120:3
}

// d3-time/test/month-test.js: timeMonth.ceil(date) observes daylight saving
#[test]
fn timemonth_ceil_date_observes_daylight_saving() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2011.0, 2.0, 13.0, 7.0]), &la)),
        ts(local(&la, &[2011.0, 3.0, 1.0]))
    ); // d3-time/test/month-test.js:124:3
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2011.0, 2.0, 13.0, 8.0]), &la)),
        ts(local(&la, &[2011.0, 3.0, 1.0]))
    ); // d3-time/test/month-test.js:125:3
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2011.0, 2.0, 13.0, 9.0]), &la)),
        ts(local(&la, &[2011.0, 3.0, 1.0]))
    ); // d3-time/test/month-test.js:126:3
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2011.0, 2.0, 13.0, 10.0]), &la)),
        ts(local(&la, &[2011.0, 3.0, 1.0]))
    ); // d3-time/test/month-test.js:127:3
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2011.0, 10.0, 6.0, 7.0]), &la)),
        ts(local(&la, &[2011.0, 11.0, 1.0]))
    ); // d3-time/test/month-test.js:128:3
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2011.0, 10.0, 6.0, 8.0]), &la)),
        ts(local(&la, &[2011.0, 11.0, 1.0]))
    ); // d3-time/test/month-test.js:129:3
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2011.0, 10.0, 6.0, 9.0]), &la)),
        ts(local(&la, &[2011.0, 11.0, 1.0]))
    ); // d3-time/test/month-test.js:130:3
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2011.0, 10.0, 6.0, 10.0]), &la)),
        ts(local(&la, &[2011.0, 11.0, 1.0]))
    ); // d3-time/test/month-test.js:131:3
}

// d3-time/test/month-test.js: timeMonth.ceil(date) handles midnight for leap years
#[test]
fn timemonth_ceil_date_handles_midnight_for_leap_years() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2012.0, 2.0, 1.0, 0.0]), &la)),
        ts(local(&la, &[2012.0, 2.0, 1.0]))
    ); // d3-time/test/month-test.js:135:3
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2012.0, 2.0, 1.0, 0.0]), &la)),
        ts(local(&la, &[2012.0, 2.0, 1.0]))
    ); // d3-time/test/month-test.js:136:3
}

// d3-time/test/month-test.js: timeMonth.offset(date) is an alias for timeMonth.offset(date, 1)
#[test]
fn timemonth_offset_date_is_an_alias_for_timemonth_offset_date_1() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/month-test.js:140:3
}

// d3-time/test/month-test.js: timeMonth.offset(date, step) does not modify the passed-in date
#[test]
fn timemonth_offset_date_step_does_not_modify_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    let d = local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Month(1).offset(d, 1_i64, &la);
    assert_eq!(
        ts(d),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/month-test.js:146:3
}

// d3-time/test/month-test.js: timeMonth.offset(date, step) does not round the passed-in date
#[test]
fn timemonth_offset_date_step_does_not_round_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 0.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/month-test.js:150:3
    assert_eq!(
        ts(Interval::Month(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 9.0, 31.0, 23.0, 59.0, 59.0, 456.0]))
    ); // d3-time/test/month-test.js:151:3
}

// d3-time/test/month-test.js: timeMonth.offset(date, step) allows step to be negative
#[test]
fn timemonth_offset_date_step_allows_step_to_be_negative() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).offset(local(&la, &[2010.0, 11.0, 31.0]), -1_i64, &la)),
        ts(local(&la, &[2010.0, 10.0, 31.0]))
    ); // d3-time/test/month-test.js:155:3
    assert_eq!(
        ts(Interval::Month(1).offset(local(&la, &[2011.0, 0.0, 1.0]), -2_i64, &la)),
        ts(local(&la, &[2010.0, 10.0, 1.0]))
    ); // d3-time/test/month-test.js:156:3
    assert_eq!(
        ts(Interval::Month(1).offset(local(&la, &[2011.0, 0.0, 1.0]), -1_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 1.0]))
    ); // d3-time/test/month-test.js:157:3
}

// d3-time/test/month-test.js: timeMonth.offset(date, step) allows step to be positive
#[test]
fn timemonth_offset_date_step_allows_step_to_be_positive() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).offset(local(&la, &[2010.0, 11.0, 31.0]), 1_i64, &la)),
        ts(local(&la, &[2011.0, 0.0, 31.0]))
    ); // d3-time/test/month-test.js:161:3
    assert_eq!(
        ts(Interval::Month(1).offset(local(&la, &[2010.0, 11.0, 30.0]), 2_i64, &la)),
        ts(local(&la, &[2011.0, 1.0, 30.0]))
    ); // d3-time/test/month-test.js:162:3
    assert_eq!(
        ts(Interval::Month(1).offset(local(&la, &[2010.0, 11.0, 30.0]), 1_i64, &la)),
        ts(local(&la, &[2011.0, 0.0, 30.0]))
    ); // d3-time/test/month-test.js:163:3
}

// d3-time/test/month-test.js: timeMonth.offset(date, step) allows step to be zero
#[test]
fn timemonth_offset_date_step_allows_step_to_be_zero() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/month-test.js:167:3
    assert_eq!(
        ts(Interval::Month(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/month-test.js:168:3
}

// d3-time/test/month-test.js: timeMonth.range(start, stop) returns months between start (inclusive) and stop (exclusive)
#[test]
fn timemonth_range_start_stop_returns_months_between_start_inclusive_and_stop_exclusive() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Month(1)
            .range(
                local(&la, &[2011.0, 11.0, 1.0]),
                local(&la, &[2012.0, 5.0, 1.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 11.0, 1.0]),
            local(&la, &[2012.0, 0.0, 1.0]),
            local(&la, &[2012.0, 1.0, 1.0]),
            local(&la, &[2012.0, 2.0, 1.0]),
            local(&la, &[2012.0, 3.0, 1.0]),
            local(&la, &[2012.0, 4.0, 1.0])
        ])
    ); // d3-time/test/month-test.js:172:3
}

// d3-time/test/month-test.js: timeMonth.range(start, stop) returns months
#[test]
fn timemonth_range_start_stop_returns_months() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Month(1)
            .range(
                local(&la, &[2011.0, 10.0, 4.0, 2.0]),
                local(&la, &[2012.0, 4.0, 10.0, 13.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 11.0, 1.0]),
            local(&la, &[2012.0, 0.0, 1.0]),
            local(&la, &[2012.0, 1.0, 1.0]),
            local(&la, &[2012.0, 2.0, 1.0]),
            local(&la, &[2012.0, 3.0, 1.0]),
            local(&la, &[2012.0, 4.0, 1.0])
        ])
    ); // d3-time/test/month-test.js:183:3
}

// d3-time/test/month-test.js: timeMonth.range(start, stop) returns the empty array if start >= stop
#[test]
fn timemonth_range_start_stop_returns_the_empty_array_if_start_stop() {
    let la = zone("America/Los_Angeles");
    assert_d3_eq(
        &tss(&Interval::Month(1)
            .range(
                local(&la, &[2011.0, 11.0, 10.0]),
                local(&la, &[2011.0, 10.0, 4.0]),
                &la,
            )
            .collect::<Vec<_>>()),
        &Vec::<f64>::new(),
    ); // d3-time/test/month-test.js:206:3
    assert_d3_eq(
        &tss(&Interval::Month(1)
            .range(
                local(&la, &[2011.0, 10.0, 1.0]),
                local(&la, &[2011.0, 10.0, 1.0]),
                &la,
            )
            .collect::<Vec<_>>()),
        &Vec::<f64>::new(),
    ); // d3-time/test/month-test.js:207:3
}

// d3-time/test/month-test.js: timeMonth.range(start, stop) returns months
#[test]
fn timemonth_range_start_stop_returns_months_2() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Month(1)
            .range(
                local(&la, &[2010.0, 10.0, 31.0]),
                local(&la, &[2011.0, 2.0, 1.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2010.0, 11.0, 1.0]),
            local(&la, &[2011.0, 0.0, 1.0]),
            local(&la, &[2011.0, 1.0, 1.0])
        ])
    ); // d3-time/test/month-test.js:211:3
}

// d3-time/test/month-test.js: timeMonth.range(start, stop) has an inclusive lower bound
#[test]
fn timemonth_range_start_stop_has_an_inclusive_lower_bound() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1)
            .range(
                local(&la, &[2010.0, 10.0, 31.0]),
                local(&la, &[2011.0, 2.0, 1.0]),
                &la
            )
            .collect::<Vec<_>>()[0_usize]),
        ts(local(&la, &[2010.0, 11.0, 1.0]))
    ); // d3-time/test/month-test.js:219:3
}

// d3-time/test/month-test.js: timeMonth.range(start, stop) has an exclusive upper bound
#[test]
fn timemonth_range_start_stop_has_an_exclusive_upper_bound() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Month(1)
            .range(
                local(&la, &[2010.0, 10.0, 31.0]),
                local(&la, &[2011.0, 2.0, 1.0]),
                &la
            )
            .collect::<Vec<_>>()[2_usize]),
        ts(local(&la, &[2011.0, 1.0, 1.0]))
    ); // d3-time/test/month-test.js:223:3
}

// d3-time/test/month-test.js: timeMonth.range(start, stop) can skip months
#[test]
fn timemonth_range_start_stop_can_skip_months() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Month(1)
            .range_step(
                local(&la, &[2011.0, 1.0, 1.0]),
                local(&la, &[2012.0, 1.0, 1.0]),
                3_u32,
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 1.0, 1.0]),
            local(&la, &[2011.0, 4.0, 1.0]),
            local(&la, &[2011.0, 7.0, 1.0]),
            local(&la, &[2011.0, 10.0, 1.0])
        ])
    ); // d3-time/test/month-test.js:227:3
}

// d3-time/test/month-test.js: timeMonth.range(start, stop) observes start of daylight savings time
#[test]
fn timemonth_range_start_stop_observes_start_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Month(1)
            .range(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 4.0, 1.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0]),
            local(&la, &[2011.0, 1.0, 1.0]),
            local(&la, &[2011.0, 2.0, 1.0]),
            local(&la, &[2011.0, 3.0, 1.0])
        ])
    ); // d3-time/test/month-test.js:236:3
}

// d3-time/test/month-test.js: timeMonth.range(start, stop) observes end of daylight savings time
#[test]
fn timemonth_range_start_stop_observes_end_of_daylight_savings_time() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Month(1)
            .range(
                local(&la, &[2011.0, 9.0, 1.0]),
                local(&la, &[2012.0, 1.0, 1.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2011.0, 9.0, 1.0]),
            local(&la, &[2011.0, 10.0, 1.0]),
            local(&la, &[2011.0, 11.0, 1.0]),
            local(&la, &[2012.0, 0.0, 1.0])
        ])
    ); // d3-time/test/month-test.js:245:3
}

// d3-time/test/month-test.js: timeMonth.count(start, end) counts months after start (exclusive) and before end (inclusive)
#[test]
fn timemonth_count_start_end_counts_months_after_start_exclusive_and_before_end_inclusive() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        Interval::Month(1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 4.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        4.0
    ); // d3-time/test/month-test.js:254:3
    assert_eq!(
        Interval::Month(1)
            .count(
                local(&la, &[2011.0, 0.0, 1.0]),
                local(&la, &[2011.0, 3.0, 30.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        3.0
    ); // d3-time/test/month-test.js:255:3
    assert_eq!(
        Interval::Month(1)
            .count(
                local(&la, &[2010.0, 11.0, 31.0]),
                local(&la, &[2011.0, 3.0, 30.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        4.0
    ); // d3-time/test/month-test.js:256:3
    assert_eq!(
        Interval::Month(1)
            .count(
                local(&la, &[2010.0, 11.0, 31.0]),
                local(&la, &[2011.0, 4.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        5.0
    ); // d3-time/test/month-test.js:257:3
    assert_eq!(
        Interval::Month(1)
            .count(
                local(&la, &[2009.0, 11.0, 31.0]),
                local(&la, &[2012.0, 4.0, 1.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        29.0
    ); // d3-time/test/month-test.js:258:3
    assert_eq!(
        Interval::Month(1)
            .count(
                local(&la, &[2012.0, 4.0, 1.0]),
                local(&la, &[2009.0, 11.0, 31.0]),
                &la
            )
            .map_or(f64::NAN, |n| n as f64),
        -29.0
    ); // d3-time/test/month-test.js:259:3
}

// d3-time/test/month-test.js: timeMonth.every(step) returns every stepth month, starting with the first month of the year
#[test]
fn timemonth_every_step_returns_every_stepth_month_starting_with_the_first_month_of_the_year() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Month(1)
            .every(3_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                local(&la, &[2008.0, 11.0, 3.0]),
                local(&la, &[2010.0, 6.0, 5.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2009.0, 0.0, 1.0]),
            local(&la, &[2009.0, 3.0, 1.0]),
            local(&la, &[2009.0, 6.0, 1.0]),
            local(&la, &[2009.0, 9.0, 1.0]),
            local(&la, &[2010.0, 0.0, 1.0]),
            local(&la, &[2010.0, 3.0, 1.0]),
            local(&la, &[2010.0, 6.0, 1.0])
        ])
    ); // d3-time/test/month-test.js:263:3
}

// d3-time/test/utcMonth-test.js: utcMonth.floor(date) returns months
#[test]
fn utcmonth_floor_date_returns_months() {
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2010.0, 11.0, 31.0, 23.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:10:3
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 0.0, 1.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:11:3
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 0.0, 1.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:12:3
}

// d3-time/test/utcMonth-test.js: utcMonth.floor(date) observes daylight saving
#[test]
fn utcmonth_floor_date_observes_daylight_saving() {
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 2.0, 13.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:16:3
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 2.0, 13.0, 8.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:17:3
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 2.0, 13.0, 9.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:18:3
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 2.0, 13.0, 10.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:19:3
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 10.0, 6.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:20:3
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 10.0, 6.0, 8.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:21:3
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 10.0, 6.0, 9.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:22:3
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[2011.0, 10.0, 6.0, 10.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:23:3
}

// d3-time/test/utcMonth-test.js: utcMonth.floor(date) handles years in the first century
#[test]
fn utcmonth_floor_date_handles_years_in_the_first_century() {
    assert_eq!(
        ts(Interval::Month(1).floor(utc(&[9.0, 10.0, 6.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[9.0, 10.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:27:3
}

// d3-time/test/utcMonth-test.js: utcMonth.round(date) returns months
#[test]
fn utcmonth_round_date_returns_months() {
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2010.0, 11.0, 16.0, 12.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:31:3
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2010.0, 11.0, 16.0, 11.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:32:3
}

// d3-time/test/utcMonth-test.js: utcMonth.round(date) observes daylight saving
#[test]
fn utcmonth_round_date_observes_daylight_saving() {
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2011.0, 2.0, 13.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:36:3
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2011.0, 2.0, 13.0, 8.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:37:3
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2011.0, 2.0, 13.0, 9.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:38:3
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2011.0, 2.0, 13.0, 20.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 2.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:39:3
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2011.0, 10.0, 6.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:40:3
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2011.0, 10.0, 6.0, 8.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:41:3
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2011.0, 10.0, 6.0, 9.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:42:3
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2011.0, 10.0, 6.0, 20.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 10.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:43:3
}

// d3-time/test/utcMonth-test.js: utcMonth.round(date) handles midnight for leap years
#[test]
fn utcmonth_round_date_handles_midnight_for_leap_years() {
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2012.0, 2.0, 1.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2012.0, 2.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:47:3
    assert_eq!(
        ts(Interval::Month(1).round(utc(&[2012.0, 2.0, 1.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2012.0, 2.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:48:3
}

// d3-time/test/utcMonth-test.js: utcMonth.ceil(date) returns months
#[test]
fn utcmonth_ceil_date_returns_months() {
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2010.0, 10.0, 30.0, 23.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:52:3
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2010.0, 11.0, 1.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:53:3
}

// d3-time/test/utcMonth-test.js: utcMonth.ceil(date) observes daylight saving
#[test]
fn utcmonth_ceil_date_observes_daylight_saving() {
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2011.0, 2.0, 13.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 3.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:57:3
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2011.0, 2.0, 13.0, 8.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 3.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:58:3
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2011.0, 2.0, 13.0, 9.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 3.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:59:3
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2011.0, 2.0, 13.0, 10.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 3.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:60:3
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2011.0, 10.0, 6.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 11.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:61:3
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2011.0, 10.0, 6.0, 8.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 11.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:62:3
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2011.0, 10.0, 6.0, 9.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 11.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:63:3
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2011.0, 10.0, 6.0, 10.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 11.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:64:3
}

// d3-time/test/utcMonth-test.js: utcMonth.ceil(date) handles midnight for leap years
#[test]
fn utcmonth_ceil_date_handles_midnight_for_leap_years() {
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2012.0, 2.0, 1.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2012.0, 2.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:68:3
    assert_eq!(
        ts(Interval::Month(1).ceil(utc(&[2012.0, 2.0, 1.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2012.0, 2.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:69:3
}

// d3-time/test/utcMonth-test.js: utcMonth.offset(date) is an alias for utcMonth.offset(date, 1)
#[test]
fn utcmonth_offset_date_is_an_alias_for_utcmonth_offset_date_1() {
    assert_eq!(
        ts(Interval::Month(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcMonth-test.js:73:3
}

// d3-time/test/utcMonth-test.js: utcMonth.offset(date, step) does not modify the passed-in date
#[test]
fn utcmonth_offset_date_step_does_not_modify_the_passed_in_date() {
    let d = utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Month(1).offset(d, 1_i64, &TimeZone::UTC);
    assert_eq!(
        ts(d),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcMonth-test.js:79:3
}

// d3-time/test/utcMonth-test.js: utcMonth.offset(date, step) does not round the passed-in date
#[test]
fn utcmonth_offset_date_step_does_not_round_the_passed_in_date() {
    assert_eq!(
        ts(Interval::Month(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 0.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcMonth-test.js:83:3
    assert_eq!(
        ts(Interval::Month(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 9.0, 31.0, 23.0, 59.0, 59.0, 456.0]))
    ); // d3-time/test/utcMonth-test.js:84:3
}

// d3-time/test/utcMonth-test.js: utcMonth.offset(date, step) allows step to be negative
#[test]
fn utcmonth_offset_date_step_allows_step_to_be_negative() {
    assert_eq!(
        ts(Interval::Month(1).offset(utc(&[2010.0, 11.0, 31.0]), -1_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 10.0, 31.0]))
    ); // d3-time/test/utcMonth-test.js:88:3
    assert_eq!(
        ts(Interval::Month(1).offset(utc(&[2011.0, 0.0, 1.0]), -2_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 10.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:89:3
    assert_eq!(
        ts(Interval::Month(1).offset(utc(&[2011.0, 0.0, 1.0]), -1_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:90:3
}

// d3-time/test/utcMonth-test.js: utcMonth.offset(date, step) allows step to be positive
#[test]
fn utcmonth_offset_date_step_allows_step_to_be_positive() {
    assert_eq!(
        ts(Interval::Month(1).offset(utc(&[2010.0, 11.0, 31.0]), 1_i64, &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 31.0]))
    ); // d3-time/test/utcMonth-test.js:94:3
    assert_eq!(
        ts(Interval::Month(1).offset(utc(&[2010.0, 11.0, 30.0]), 2_i64, &TimeZone::UTC)),
        ts(utc(&[2011.0, 1.0, 30.0]))
    ); // d3-time/test/utcMonth-test.js:95:3
    assert_eq!(
        ts(Interval::Month(1).offset(utc(&[2010.0, 11.0, 30.0]), 1_i64, &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 30.0]))
    ); // d3-time/test/utcMonth-test.js:96:3
}

// d3-time/test/utcMonth-test.js: utcMonth.offset(date, step) allows step to be zero
#[test]
fn utcmonth_offset_date_step_allows_step_to_be_zero() {
    assert_eq!(
        ts(Interval::Month(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcMonth-test.js:100:3
    assert_eq!(
        ts(Interval::Month(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/utcMonth-test.js:101:3
}

// d3-time/test/utcMonth-test.js: utcMonth.range(start, stop) returns months between start (inclusive) and stop (exclusive)
#[test]
fn utcmonth_range_start_stop_returns_months_between_start_inclusive_and_stop_exclusive() {
    assert_eq!(
        tss(&Interval::Month(1)
            .range(
                utc(&[2011.0, 11.0, 1.0]),
                utc(&[2012.0, 5.0, 1.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 11.0, 1.0]),
            utc(&[2012.0, 0.0, 1.0]),
            utc(&[2012.0, 1.0, 1.0]),
            utc(&[2012.0, 2.0, 1.0]),
            utc(&[2012.0, 3.0, 1.0]),
            utc(&[2012.0, 4.0, 1.0])
        ])
    ); // d3-time/test/utcMonth-test.js:105:3
}

// d3-time/test/utcMonth-test.js: utcMonth.range(start, stop) returns months
#[test]
fn utcmonth_range_start_stop_returns_months() {
    assert_eq!(
        tss(&Interval::Month(1)
            .range(
                utc(&[2011.0, 10.0, 4.0, 2.0]),
                utc(&[2012.0, 4.0, 10.0, 13.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 11.0, 1.0]),
            utc(&[2012.0, 0.0, 1.0]),
            utc(&[2012.0, 1.0, 1.0]),
            utc(&[2012.0, 2.0, 1.0]),
            utc(&[2012.0, 3.0, 1.0]),
            utc(&[2012.0, 4.0, 1.0])
        ])
    ); // d3-time/test/utcMonth-test.js:116:3
}

// d3-time/test/utcMonth-test.js: utcMonth.range(start, stop) returns the empty array if start >= stop
#[test]
fn utcmonth_range_start_stop_returns_the_empty_array_if_start_stop() {
    assert_d3_eq(
        &tss(&Interval::Month(1)
            .range(
                utc(&[2011.0, 11.0, 10.0]),
                utc(&[2011.0, 10.0, 4.0]),
                &TimeZone::UTC,
            )
            .collect::<Vec<_>>()),
        &Vec::<f64>::new(),
    ); // d3-time/test/utcMonth-test.js:139:3
    assert_d3_eq(
        &tss(&Interval::Month(1)
            .range(
                utc(&[2011.0, 10.0, 1.0]),
                utc(&[2011.0, 10.0, 1.0]),
                &TimeZone::UTC,
            )
            .collect::<Vec<_>>()),
        &Vec::<f64>::new(),
    ); // d3-time/test/utcMonth-test.js:140:3
}

// d3-time/test/utcMonth-test.js: utcMonth.range(start, stop) returns months
#[test]
fn utcmonth_range_start_stop_returns_months_2() {
    assert_eq!(
        tss(&Interval::Month(1)
            .range(
                utc(&[2010.0, 10.0, 31.0]),
                utc(&[2011.0, 2.0, 1.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2010.0, 11.0, 1.0]),
            utc(&[2011.0, 0.0, 1.0]),
            utc(&[2011.0, 1.0, 1.0])
        ])
    ); // d3-time/test/utcMonth-test.js:144:3
}

// d3-time/test/utcMonth-test.js: utcMonth.range(start, stop) has an inclusive lower bound
#[test]
fn utcmonth_range_start_stop_has_an_inclusive_lower_bound() {
    assert_eq!(
        ts(Interval::Month(1)
            .range(
                utc(&[2010.0, 10.0, 31.0]),
                utc(&[2011.0, 2.0, 1.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()[0_usize]),
        ts(utc(&[2010.0, 11.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:152:3
}

// d3-time/test/utcMonth-test.js: utcMonth.range(start, stop) has an exclusive upper bound
#[test]
fn utcmonth_range_start_stop_has_an_exclusive_upper_bound() {
    assert_eq!(
        ts(Interval::Month(1)
            .range(
                utc(&[2010.0, 10.0, 31.0]),
                utc(&[2011.0, 2.0, 1.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()[2_usize]),
        ts(utc(&[2011.0, 1.0, 1.0]))
    ); // d3-time/test/utcMonth-test.js:156:3
}

// d3-time/test/utcMonth-test.js: utcMonth.range(start, stop) can skip months
#[test]
fn utcmonth_range_start_stop_can_skip_months() {
    assert_eq!(
        tss(&Interval::Month(1)
            .range_step(
                utc(&[2011.0, 1.0, 1.0]),
                utc(&[2012.0, 1.0, 1.0]),
                3_u32,
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 1.0, 1.0]),
            utc(&[2011.0, 4.0, 1.0]),
            utc(&[2011.0, 7.0, 1.0]),
            utc(&[2011.0, 10.0, 1.0])
        ])
    ); // d3-time/test/utcMonth-test.js:160:3
}

// d3-time/test/utcMonth-test.js: utcMonth.range(start, stop) observes start of daylight savings time
#[test]
fn utcmonth_range_start_stop_observes_start_of_daylight_savings_time() {
    assert_eq!(
        tss(&Interval::Month(1)
            .range(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 4.0, 1.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 0.0, 1.0]),
            utc(&[2011.0, 1.0, 1.0]),
            utc(&[2011.0, 2.0, 1.0]),
            utc(&[2011.0, 3.0, 1.0])
        ])
    ); // d3-time/test/utcMonth-test.js:169:3
}

// d3-time/test/utcMonth-test.js: utcMonth.range(start, stop) observes end of daylight savings time
#[test]
fn utcmonth_range_start_stop_observes_end_of_daylight_savings_time() {
    assert_eq!(
        tss(&Interval::Month(1)
            .range(
                utc(&[2011.0, 9.0, 1.0]),
                utc(&[2012.0, 1.0, 1.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2011.0, 9.0, 1.0]),
            utc(&[2011.0, 10.0, 1.0]),
            utc(&[2011.0, 11.0, 1.0]),
            utc(&[2012.0, 0.0, 1.0])
        ])
    ); // d3-time/test/utcMonth-test.js:178:3
}

// d3-time/test/utcMonth-test.js: utcMonth.count(start, end) counts months after start (exclusive) and before end (inclusive)
#[test]
fn utcmonth_count_start_end_counts_months_after_start_exclusive_and_before_end_inclusive() {
    assert_eq!(
        Interval::Month(1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 4.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        4.0
    ); // d3-time/test/utcMonth-test.js:187:3
    assert_eq!(
        Interval::Month(1)
            .count(
                utc(&[2011.0, 0.0, 1.0]),
                utc(&[2011.0, 3.0, 30.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        3.0
    ); // d3-time/test/utcMonth-test.js:188:3
    assert_eq!(
        Interval::Month(1)
            .count(
                utc(&[2010.0, 11.0, 31.0]),
                utc(&[2011.0, 3.0, 30.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        4.0
    ); // d3-time/test/utcMonth-test.js:189:3
    assert_eq!(
        Interval::Month(1)
            .count(
                utc(&[2010.0, 11.0, 31.0]),
                utc(&[2011.0, 4.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        5.0
    ); // d3-time/test/utcMonth-test.js:190:3
    assert_eq!(
        Interval::Month(1)
            .count(
                utc(&[2009.0, 11.0, 31.0]),
                utc(&[2012.0, 4.0, 1.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        29.0
    ); // d3-time/test/utcMonth-test.js:191:3
    assert_eq!(
        Interval::Month(1)
            .count(
                utc(&[2012.0, 4.0, 1.0]),
                utc(&[2009.0, 11.0, 31.0]),
                &TimeZone::UTC
            )
            .map_or(f64::NAN, |n| n as f64),
        -29.0
    ); // d3-time/test/utcMonth-test.js:192:3
}

// d3-time/test/utcMonth-test.js: utcMonth.every(step) returns every stepth month, starting with the first month of the year
#[test]
fn utcmonth_every_step_returns_every_stepth_month_starting_with_the_first_month_of_the_year() {
    assert_eq!(
        tss(&Interval::Month(1)
            .every(3_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                utc(&[2008.0, 11.0, 3.0]),
                utc(&[2010.0, 6.0, 5.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2009.0, 0.0, 1.0]),
            utc(&[2009.0, 3.0, 1.0]),
            utc(&[2009.0, 6.0, 1.0]),
            utc(&[2009.0, 9.0, 1.0]),
            utc(&[2010.0, 0.0, 1.0]),
            utc(&[2010.0, 3.0, 1.0]),
            utc(&[2010.0, 6.0, 1.0])
        ])
    ); // d3-time/test/utcMonth-test.js:196:3
}

// d3-time/test/year-test.js: timeYear.floor(date) returns years
#[test]
fn timeyear_floor_date_returns_years() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Year(1).floor(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2010.0, 0.0, 1.0]))
    ); // d3-time/test/year-test.js:6:3
    assert_eq!(
        ts(Interval::Year(1).floor(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/year-test.js:7:3
    assert_eq!(
        ts(Interval::Year(1).floor(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/year-test.js:8:3
}

// d3-time/test/year-test.js: timeYear.floor(date) does not modify the specified date
#[test]
fn timeyear_floor_date_does_not_modify_the_specified_date() {
    let la = zone("America/Los_Angeles");
    let d = local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]);
    assert_eq!(
        ts(Interval::Year(1).floor(d, &la)),
        ts(local(&la, &[2010.0, 0.0, 1.0]))
    ); // d3-time/test/year-test.js:13:3
    assert_eq!(
        ts(d),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]))
    ); // d3-time/test/year-test.js:14:3
}

// d3-time/test/year-test.js: timeYear.floor(date) correctly handles years in the first century
#[test]
fn timeyear_floor_date_correctly_handles_years_in_the_first_century() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Year(1).floor(local(&la, &[9.0, 10.0, 6.0, 7.0]), &la)),
        ts(local(&la, &[9.0, 0.0, 1.0]))
    ); // d3-time/test/year-test.js:18:3
}

// d3-time/test/year-test.js: timeYear.ceil(date) returns years
#[test]
fn timeyear_ceil_date_returns_years() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Year(1).ceil(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/year-test.js:22:3
    assert_eq!(
        ts(Interval::Year(1).ceil(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/year-test.js:23:3
    assert_eq!(
        ts(Interval::Year(1).ceil(local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2012.0, 0.0, 1.0]))
    ); // d3-time/test/year-test.js:24:3
}

// d3-time/test/year-test.js: timeYear.offset(date, count) does not modify the passed-in date
#[test]
fn timeyear_offset_date_count_does_not_modify_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    let d = local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Year(1).offset(d, 1_i64, &la);
    assert_eq!(
        ts(d),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/year-test.js:30:3
}

// d3-time/test/year-test.js: timeYear.offset(date, count) does not round the passed-in-date
#[test]
fn timeyear_offset_date_count_does_not_round_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Year(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &la
        )),
        ts(local(&la, &[2011.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/year-test.js:34:3
    assert_eq!(
        ts(Interval::Year(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &la
        )),
        ts(local(&la, &[2008.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]))
    ); // d3-time/test/year-test.js:35:3
}

// d3-time/test/year-test.js: timeYear.offset(date, count) allows negative offsets
#[test]
fn timeyear_offset_date_count_allows_negative_offsets() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Year(1).offset(local(&la, &[2010.0, 11.0, 1.0]), -1_i64, &la)),
        ts(local(&la, &[2009.0, 11.0, 1.0]))
    ); // d3-time/test/year-test.js:39:3
    assert_eq!(
        ts(Interval::Year(1).offset(local(&la, &[2011.0, 0.0, 1.0]), -2_i64, &la)),
        ts(local(&la, &[2009.0, 0.0, 1.0]))
    ); // d3-time/test/year-test.js:40:3
    assert_eq!(
        ts(Interval::Year(1).offset(local(&la, &[2011.0, 0.0, 1.0]), -1_i64, &la)),
        ts(local(&la, &[2010.0, 0.0, 1.0]))
    ); // d3-time/test/year-test.js:41:3
}

// d3-time/test/year-test.js: timeYear.offset(date, count) allows positive offsets
#[test]
fn timeyear_offset_date_count_allows_positive_offsets() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Year(1).offset(local(&la, &[2009.0, 11.0, 1.0]), 1_i64, &la)),
        ts(local(&la, &[2010.0, 11.0, 1.0]))
    ); // d3-time/test/year-test.js:45:3
    assert_eq!(
        ts(Interval::Year(1).offset(local(&la, &[2009.0, 0.0, 1.0]), 2_i64, &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/year-test.js:46:3
    assert_eq!(
        ts(Interval::Year(1).offset(local(&la, &[2010.0, 0.0, 1.0]), 1_i64, &la)),
        ts(local(&la, &[2011.0, 0.0, 1.0]))
    ); // d3-time/test/year-test.js:47:3
}

// d3-time/test/year-test.js: timeYear.offset(date, count) allows zero offset
#[test]
fn timeyear_offset_date_count_allows_zero_offset() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Year(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/year-test.js:51:3
    assert_eq!(
        ts(Interval::Year(1).offset(
            local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &la
        )),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/year-test.js:52:3
}

// d3-time/test/year-test.js: timeYear.every(step) returns every stepth year, starting with year zero
#[test]
fn timeyear_every_step_returns_every_stepth_year_starting_with_year_zero() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Year(1)
            .every(5_u32)
            .expect("d3 test: a step of 1 or more")
            .range(local(&la, &[2008.0]), local(&la, &[2023.0]), &la)
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2010.0]),
            local(&la, &[2015.0]),
            local(&la, &[2020.0])
        ])
    ); // d3-time/test/year-test.js:56:3
}

// d3-time/test/year-test.js: timeYear.range(start, stop) returns years
#[test]
fn timeyear_range_start_stop_returns_years() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Year(1)
            .range(
                local(&la, &[2010.0, 0.0, 1.0]),
                local(&la, &[2013.0, 0.0, 1.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2010.0, 0.0, 1.0]),
            local(&la, &[2011.0, 0.0, 1.0]),
            local(&la, &[2012.0, 0.0, 1.0])
        ])
    ); // d3-time/test/year-test.js:60:3
}

// d3-time/test/year-test.js: timeYear.range(start, stop) has an inclusive lower bound
#[test]
fn timeyear_range_start_stop_has_an_inclusive_lower_bound() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Year(1)
            .range(
                local(&la, &[2010.0, 0.0, 1.0]),
                local(&la, &[2013.0, 0.0, 1.0]),
                &la
            )
            .collect::<Vec<_>>()[0_usize]),
        ts(local(&la, &[2010.0, 0.0, 1.0]))
    ); // d3-time/test/year-test.js:68:3
}

// d3-time/test/year-test.js: timeYear.range(start, stop) has an exclusive upper bound
#[test]
fn timeyear_range_start_stop_has_an_exclusive_upper_bound() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Year(1)
            .range(
                local(&la, &[2010.0, 0.0, 1.0]),
                local(&la, &[2013.0, 0.0, 1.0]),
                &la
            )
            .collect::<Vec<_>>()[2_usize]),
        ts(local(&la, &[2012.0, 0.0, 1.0]))
    ); // d3-time/test/year-test.js:72:3
}

// d3-time/test/year-test.js: timeYear.range(start, stop, step) can skip years
#[test]
fn timeyear_range_start_stop_step_can_skip_years() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Year(1)
            .range_step(
                local(&la, &[2009.0, 0.0, 1.0]),
                local(&la, &[2029.0, 0.0, 1.0]),
                5_u32,
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2009.0, 0.0, 1.0]),
            local(&la, &[2014.0, 0.0, 1.0]),
            local(&la, &[2019.0, 0.0, 1.0]),
            local(&la, &[2024.0, 0.0, 1.0])
        ])
    ); // d3-time/test/year-test.js:76:3
}

// d3-time/test/utcYear-test.js: utcYear.floor(date) returns years
#[test]
fn utcyear_floor_date_returns_years() {
    assert_eq!(
        ts(Interval::Year(1).floor(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 0.0, 1.0]))
    ); // d3-time/test/utcYear-test.js:6:3
    assert_eq!(
        ts(Interval::Year(1).floor(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcYear-test.js:7:3
    assert_eq!(
        ts(Interval::Year(1).floor(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcYear-test.js:8:3
}

// d3-time/test/utcYear-test.js: utcYear.floor(date) does not modify the specified date
#[test]
fn utcyear_floor_date_does_not_modify_the_specified_date() {
    let d = utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]);
    assert_eq!(
        ts(Interval::Year(1).floor(d, &TimeZone::UTC)),
        ts(utc(&[2010.0, 0.0, 1.0]))
    ); // d3-time/test/utcYear-test.js:13:3
    assert_eq!(ts(d), ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]))); // d3-time/test/utcYear-test.js:14:3
}

// d3-time/test/utcYear-test.js: utcYear.floor(date) correctly handles years in the first century
#[test]
fn utcyear_floor_date_correctly_handles_years_in_the_first_century() {
    assert_eq!(
        ts(Interval::Year(1).floor(utc(&[9.0, 10.0, 6.0, 7.0]), &TimeZone::UTC)),
        ts(utc(&[9.0, 0.0, 1.0]))
    ); // d3-time/test/utcYear-test.js:18:3
}

// d3-time/test/utcYear-test.js: utcYear.ceil(date) returns years
#[test]
fn utcyear_ceil_date_returns_years() {
    assert_eq!(
        ts(Interval::Year(1).ceil(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcYear-test.js:22:3
    assert_eq!(
        ts(Interval::Year(1).ceil(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcYear-test.js:23:3
    assert_eq!(
        ts(Interval::Year(1).ceil(utc(&[2011.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2012.0, 0.0, 1.0]))
    ); // d3-time/test/utcYear-test.js:24:3
}

// d3-time/test/utcYear-test.js: utcYear.offset(date, count) does not modify the passed-in date
#[test]
fn utcyear_offset_date_count_does_not_modify_the_passed_in_date() {
    let d = utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Year(1).offset(d, 1_i64, &TimeZone::UTC);
    assert_eq!(
        ts(d),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcYear-test.js:30:3
}

// d3-time/test/utcYear-test.js: utcYear.offset(date, count) does not round the passed-in-date
#[test]
fn utcyear_offset_date_count_does_not_round_the_passed_in_date() {
    assert_eq!(
        ts(Interval::Year(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            1_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2011.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcYear-test.js:34:3
    assert_eq!(
        ts(Interval::Year(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
            -2_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2008.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]))
    ); // d3-time/test/utcYear-test.js:35:3
}

// d3-time/test/utcYear-test.js: utcYear.offset(date, count) allows negative offsets
#[test]
fn utcyear_offset_date_count_allows_negative_offsets() {
    assert_eq!(
        ts(Interval::Year(1).offset(utc(&[2010.0, 11.0, 1.0]), -1_i64, &TimeZone::UTC)),
        ts(utc(&[2009.0, 11.0, 1.0]))
    ); // d3-time/test/utcYear-test.js:39:3
    assert_eq!(
        ts(Interval::Year(1).offset(utc(&[2011.0, 0.0, 1.0]), -2_i64, &TimeZone::UTC)),
        ts(utc(&[2009.0, 0.0, 1.0]))
    ); // d3-time/test/utcYear-test.js:40:3
    assert_eq!(
        ts(Interval::Year(1).offset(utc(&[2011.0, 0.0, 1.0]), -1_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 0.0, 1.0]))
    ); // d3-time/test/utcYear-test.js:41:3
}

// d3-time/test/utcYear-test.js: utcYear.offset(date, count) allows positive offsets
#[test]
fn utcyear_offset_date_count_allows_positive_offsets() {
    assert_eq!(
        ts(Interval::Year(1).offset(utc(&[2009.0, 11.0, 1.0]), 1_i64, &TimeZone::UTC)),
        ts(utc(&[2010.0, 11.0, 1.0]))
    ); // d3-time/test/utcYear-test.js:45:3
    assert_eq!(
        ts(Interval::Year(1).offset(utc(&[2009.0, 0.0, 1.0]), 2_i64, &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcYear-test.js:46:3
    assert_eq!(
        ts(Interval::Year(1).offset(utc(&[2010.0, 0.0, 1.0]), 1_i64, &TimeZone::UTC)),
        ts(utc(&[2011.0, 0.0, 1.0]))
    ); // d3-time/test/utcYear-test.js:47:3
}

// d3-time/test/utcYear-test.js: utcYear.offset(date, count) allows zero offset
#[test]
fn utcyear_offset_date_count_allows_zero_offset() {
    assert_eq!(
        ts(Interval::Year(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcYear-test.js:51:3
    assert_eq!(
        ts(Interval::Year(1).offset(
            utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]),
            0_i64,
            &TimeZone::UTC
        )),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 58.0, 0.0]))
    ); // d3-time/test/utcYear-test.js:52:3
}

// d3-time/test/utcYear-test.js: utcYear.every(step) returns every stepth year, starting with year zero
#[test]
fn utcyear_every_step_returns_every_stepth_year_starting_with_year_zero() {
    assert_eq!(
        tss(&Interval::Year(1)
            .every(5_u32)
            .expect("d3 test: a step of 1 or more")
            .range(utc(&[2008.0]), utc(&[2023.0]), &TimeZone::UTC)
            .collect::<Vec<_>>()),
        tss(&[utc(&[2010.0]), utc(&[2015.0]), utc(&[2020.0])])
    ); // d3-time/test/utcYear-test.js:56:3
}

// d3-time/test/utcYear-test.js: utcYear.range(start, stop) returns years
#[test]
fn utcyear_range_start_stop_returns_years() {
    assert_eq!(
        tss(&Interval::Year(1)
            .range(
                utc(&[2010.0, 0.0, 1.0]),
                utc(&[2013.0, 0.0, 1.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2010.0, 0.0, 1.0]),
            utc(&[2011.0, 0.0, 1.0]),
            utc(&[2012.0, 0.0, 1.0])
        ])
    ); // d3-time/test/utcYear-test.js:60:3
}

// d3-time/test/utcYear-test.js: utcYear.range(start, stop) has an inclusive lower bound
#[test]
fn utcyear_range_start_stop_has_an_inclusive_lower_bound() {
    assert_eq!(
        ts(Interval::Year(1)
            .range(
                utc(&[2010.0, 0.0, 1.0]),
                utc(&[2013.0, 0.0, 1.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()[0_usize]),
        ts(utc(&[2010.0, 0.0, 1.0]))
    ); // d3-time/test/utcYear-test.js:68:3
}

// d3-time/test/utcYear-test.js: utcYear.range(start, stop) has an exclusive upper bound
#[test]
fn utcyear_range_start_stop_has_an_exclusive_upper_bound() {
    assert_eq!(
        ts(Interval::Year(1)
            .range(
                utc(&[2010.0, 0.0, 1.0]),
                utc(&[2013.0, 0.0, 1.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()[2_usize]),
        ts(utc(&[2012.0, 0.0, 1.0]))
    ); // d3-time/test/utcYear-test.js:72:3
}

// d3-time/test/utcYear-test.js: utcYear.range(start, stop, step) can skip years
#[test]
fn utcyear_range_start_stop_step_can_skip_years() {
    assert_eq!(
        tss(&Interval::Year(1)
            .range_step(
                utc(&[2009.0, 0.0, 1.0]),
                utc(&[2029.0, 0.0, 1.0]),
                5_u32,
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2009.0, 0.0, 1.0]),
            utc(&[2014.0, 0.0, 1.0]),
            utc(&[2019.0, 0.0, 1.0]),
            utc(&[2024.0, 0.0, 1.0])
        ])
    ); // d3-time/test/utcYear-test.js:76:3
}

// d3-time/test/multiYear-test.js: timeYear.every(n).floor(date) returns integer multiples of n years
#[test]
fn timeyear_every_n_floor_date_returns_integer_multiples_of_n_years() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Year(1)
            .every(10_u32)
            .expect("d3 test: a step of 1 or more")
            .floor(local(&la, &[2009.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2000.0, 0.0, 1.0]))
    ); // d3-time/test/multiYear-test.js:6:3
    assert_eq!(
        ts(Interval::Year(1)
            .every(10_u32)
            .expect("d3 test: a step of 1 or more")
            .floor(local(&la, &[2010.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2010.0, 0.0, 1.0]))
    ); // d3-time/test/multiYear-test.js:7:3
    assert_eq!(
        ts(Interval::Year(1)
            .every(10_u32)
            .expect("d3 test: a step of 1 or more")
            .floor(local(&la, &[2010.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2010.0, 0.0, 1.0]))
    ); // d3-time/test/multiYear-test.js:8:3
}

// d3-time/test/multiYear-test.js: timeYear.every(n).ceil(date) returns integer multiples of n years
#[test]
fn timeyear_every_n_ceil_date_returns_integer_multiples_of_n_years() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Year(1)
            .every(100_u32)
            .expect("d3 test: a step of 1 or more")
            .ceil(local(&la, &[1999.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &la)),
        ts(local(&la, &[2000.0, 0.0, 1.0]))
    ); // d3-time/test/multiYear-test.js:12:3
    assert_eq!(
        ts(Interval::Year(1)
            .every(100_u32)
            .expect("d3 test: a step of 1 or more")
            .ceil(local(&la, &[2000.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &la)),
        ts(local(&la, &[2000.0, 0.0, 1.0]))
    ); // d3-time/test/multiYear-test.js:13:3
    assert_eq!(
        ts(Interval::Year(1)
            .every(100_u32)
            .expect("d3 test: a step of 1 or more")
            .ceil(local(&la, &[2000.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &la)),
        ts(local(&la, &[2100.0, 0.0, 1.0]))
    ); // d3-time/test/multiYear-test.js:14:3
}

// d3-time/test/multiYear-test.js: timeYear.every(n).offset(date, count) does not modify the passed-in date
#[test]
fn timeyear_every_n_offset_date_count_does_not_modify_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    let d = local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Year(1)
        .every(5_u32)
        .expect("d3 test: a step of 1 or more")
        .offset(d, 1_i64, &la);
    assert_eq!(
        ts(d),
        ts(local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/multiYear-test.js:20:3
}

// d3-time/test/multiYear-test.js: timeYear.every(n).offset(date, count) does not round the passed-in-date
#[test]
fn timeyear_every_n_offset_date_count_does_not_round_the_passed_in_date() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        ts(Interval::Year(1)
            .every(5_u32)
            .expect("d3 test: a step of 1 or more")
            .offset(
                local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
                1_i64,
                &la
            )),
        ts(local(&la, &[2015.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/multiYear-test.js:24:3
    assert_eq!(
        ts(Interval::Year(1)
            .every(5_u32)
            .expect("d3 test: a step of 1 or more")
            .offset(
                local(&la, &[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
                -2_i64,
                &la
            )),
        ts(local(&la, &[2000.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]))
    ); // d3-time/test/multiYear-test.js:25:3
}

// d3-time/test/multiYear-test.js: timeYear.every(n).range(start, stop) returns multiples of n years
#[test]
fn timeyear_every_n_range_start_stop_returns_multiples_of_n_years() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&Interval::Year(1)
            .every(10_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                local(&la, &[2010.0, 0.0, 1.0]),
                local(&la, &[2031.0, 0.0, 1.0]),
                &la
            )
            .collect::<Vec<_>>()),
        tss(&[
            local(&la, &[2010.0, 0.0, 1.0]),
            local(&la, &[2020.0, 0.0, 1.0]),
            local(&la, &[2030.0, 0.0, 1.0])
        ])
    ); // d3-time/test/multiYear-test.js:35:3
}

// d3-time/test/utcMultiYear-test.js: utcYear.every(n).floor(date) returns integer multiples of n years
#[test]
fn utcyear_every_n_floor_date_returns_integer_multiples_of_n_years() {
    assert_eq!(
        ts(Interval::Year(1)
            .every(10_u32)
            .expect("d3 test: a step of 1 or more")
            .floor(utc(&[2009.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2000.0, 0.0, 1.0]))
    ); // d3-time/test/utcMultiYear-test.js:6:3
    assert_eq!(
        ts(Interval::Year(1)
            .every(10_u32)
            .expect("d3 test: a step of 1 or more")
            .floor(utc(&[2010.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 0.0, 1.0]))
    ); // d3-time/test/utcMultiYear-test.js:7:3
    assert_eq!(
        ts(Interval::Year(1)
            .every(10_u32)
            .expect("d3 test: a step of 1 or more")
            .floor(utc(&[2010.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2010.0, 0.0, 1.0]))
    ); // d3-time/test/utcMultiYear-test.js:8:3
}

// d3-time/test/utcMultiYear-test.js: utcYear.every(n).ceil(date) returns integer multiples of n years
#[test]
fn utcyear_every_n_ceil_date_returns_integer_multiples_of_n_years() {
    assert_eq!(
        ts(Interval::Year(1)
            .every(100_u32)
            .expect("d3 test: a step of 1 or more")
            .ceil(utc(&[1999.0, 11.0, 31.0, 23.0, 59.0, 59.0]), &TimeZone::UTC)),
        ts(utc(&[2000.0, 0.0, 1.0]))
    ); // d3-time/test/utcMultiYear-test.js:12:3
    assert_eq!(
        ts(Interval::Year(1)
            .every(100_u32)
            .expect("d3 test: a step of 1 or more")
            .ceil(utc(&[2000.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC)),
        ts(utc(&[2000.0, 0.0, 1.0]))
    ); // d3-time/test/utcMultiYear-test.js:13:3
    assert_eq!(
        ts(Interval::Year(1)
            .every(100_u32)
            .expect("d3 test: a step of 1 or more")
            .ceil(utc(&[2000.0, 0.0, 1.0, 0.0, 0.0, 1.0]), &TimeZone::UTC)),
        ts(utc(&[2100.0, 0.0, 1.0]))
    ); // d3-time/test/utcMultiYear-test.js:14:3
}

// d3-time/test/utcMultiYear-test.js: utcYear.every(n).offset(date, count) does not modify the passed-in date
#[test]
fn utcyear_every_n_offset_date_count_does_not_modify_the_passed_in_date() {
    let d = utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]);
    let _ = Interval::Year(1)
        .every(5_u32)
        .expect("d3 test: a step of 1 or more")
        .offset(d, 1_i64, &TimeZone::UTC);
    assert_eq!(
        ts(d),
        ts(utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcMultiYear-test.js:20:3
}

// d3-time/test/utcMultiYear-test.js: utcYear.every(n).offset(date, count) does not round the passed-in-date
#[test]
fn utcyear_every_n_offset_date_count_does_not_round_the_passed_in_date() {
    assert_eq!(
        ts(Interval::Year(1)
            .every(5_u32)
            .expect("d3 test: a step of 1 or more")
            .offset(
                utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]),
                1_i64,
                &TimeZone::UTC
            )),
        ts(utc(&[2015.0, 11.0, 31.0, 23.0, 59.0, 59.0, 999.0]))
    ); // d3-time/test/utcMultiYear-test.js:24:3
    assert_eq!(
        ts(Interval::Year(1)
            .every(5_u32)
            .expect("d3 test: a step of 1 or more")
            .offset(
                utc(&[2010.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]),
                -2_i64,
                &TimeZone::UTC
            )),
        ts(utc(&[2000.0, 11.0, 31.0, 23.0, 59.0, 59.0, 456.0]))
    ); // d3-time/test/utcMultiYear-test.js:25:3
}

// d3-time/test/utcMultiYear-test.js: utcYear.every(n).range(start, stop) returns multiples of n years
#[test]
fn utcyear_every_n_range_start_stop_returns_multiples_of_n_years() {
    assert_eq!(
        tss(&Interval::Year(1)
            .every(10_u32)
            .expect("d3 test: a step of 1 or more")
            .range(
                utc(&[2010.0, 0.0, 1.0]),
                utc(&[2031.0, 0.0, 1.0]),
                &TimeZone::UTC
            )
            .collect::<Vec<_>>()),
        tss(&[
            utc(&[2010.0, 0.0, 1.0]),
            utc(&[2020.0, 0.0, 1.0]),
            utc(&[2030.0, 0.0, 1.0])
        ])
    ); // d3-time/test/utcMultiYear-test.js:35:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, interval) respects the specified interval
#[test]
fn timeticks_start_stop_interval_respects_the_specified_interval() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks_with(
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 1.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 4.0, 4.0]),
            Interval::Minute(1),
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 1.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 2.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 3.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 4.0])
        ])
    ); // d3-time/test/ticks-test.js:6:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, interval.every(step)) observes the specified tick interval and step
#[test]
fn timeticks_start_stop_interval_every_step_observes_the_specified_tick_interval_and_step() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks_with(
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 33.0, 4.0]),
            Interval::Minute(1)
                .every(10_u32)
                .expect("d3 test: a step of 1 or more"),
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 10.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 20.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 30.0])
        ])
    ); // d3-time/test/ticks-test.js:19:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate sub-second ticks
#[test]
fn timeticks_start_stop_count_can_generate_sub_second_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 1.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 200.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 400.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 600.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 800.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 1.0, 0.0])
        ])
    ); // d3-time/test/ticks-test.js:28:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 1-second ticks
#[test]
fn timeticks_start_stop_count_can_generate_1_second_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 4.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 1.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 2.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 3.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 4.0])
        ])
    ); // d3-time/test/ticks-test.js:39:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 5-second ticks
#[test]
fn timeticks_start_stop_count_can_generate_5_second_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 20.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 5.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 10.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 15.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 20.0])
        ])
    ); // d3-time/test/ticks-test.js:49:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 15-second ticks
#[test]
fn timeticks_start_stop_count_can_generate_15_second_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 50.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 15.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 30.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 45.0])
        ])
    ); // d3-time/test/ticks-test.js:59:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 30-second ticks
#[test]
fn timeticks_start_stop_count_can_generate_30_second_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 1.0, 50.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 30.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 1.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 1.0, 30.0])
        ])
    ); // d3-time/test/ticks-test.js:68:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 1-minute ticks
#[test]
fn timeticks_start_stop_count_can_generate_1_minute_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 27.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 4.0, 12.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 1.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 2.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 3.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 4.0])
        ])
    ); // d3-time/test/ticks-test.js:77:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 5-minute ticks
#[test]
fn timeticks_start_stop_count_can_generate_5_minute_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 3.0, 27.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 21.0, 12.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 5.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 10.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 15.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 20.0])
        ])
    ); // d3-time/test/ticks-test.js:86:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 15-minute ticks
#[test]
fn timeticks_start_stop_count_can_generate_15_minute_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 8.0, 27.0]),
            local(&la, &[2011.0, 0.0, 1.0, 13.0, 4.0, 12.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 15.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 30.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 45.0]),
            local(&la, &[2011.0, 0.0, 1.0, 13.0, 0.0])
        ])
    ); // d3-time/test/ticks-test.js:95:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 30-minute ticks
#[test]
fn timeticks_start_stop_count_can_generate_30_minute_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 28.0, 27.0]),
            local(&la, &[2011.0, 0.0, 1.0, 14.0, 4.0, 12.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 30.0]),
            local(&la, &[2011.0, 0.0, 1.0, 13.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 13.0, 30.0]),
            local(&la, &[2011.0, 0.0, 1.0, 14.0, 0.0])
        ])
    ); // d3-time/test/ticks-test.js:104:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 1-hour ticks
#[test]
fn timeticks_start_stop_count_can_generate_1_hour_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 28.0, 27.0]),
            local(&la, &[2011.0, 0.0, 1.0, 16.0, 34.0, 12.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 13.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 14.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 15.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 16.0, 0.0])
        ])
    ); // d3-time/test/ticks-test.js:113:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 3-hour ticks
#[test]
fn timeticks_start_stop_count_can_generate_3_hour_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 0.0, 1.0, 14.0, 28.0, 27.0]),
            local(&la, &[2011.0, 0.0, 2.0, 1.0, 34.0, 12.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 15.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 18.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 21.0, 0.0]),
            local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0])
        ])
    ); // d3-time/test/ticks-test.js:122:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 6-hour ticks
#[test]
fn timeticks_start_stop_count_can_generate_6_hour_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 0.0, 1.0, 16.0, 28.0, 27.0]),
            local(&la, &[2011.0, 0.0, 2.0, 14.0, 34.0, 12.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 18.0, 0.0]),
            local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 2.0, 6.0, 0.0]),
            local(&la, &[2011.0, 0.0, 2.0, 12.0, 0.0])
        ])
    ); // d3-time/test/ticks-test.js:131:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 12-hour ticks
#[test]
fn timeticks_start_stop_count_can_generate_12_hour_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 0.0, 1.0, 16.0, 28.0, 27.0]),
            local(&la, &[2011.0, 0.0, 3.0, 21.0, 34.0, 12.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 2.0, 12.0, 0.0]),
            local(&la, &[2011.0, 0.0, 3.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 3.0, 12.0, 0.0])
        ])
    ); // d3-time/test/ticks-test.js:140:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 1-day ticks
#[test]
fn timeticks_start_stop_count_can_generate_1_day_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 0.0, 1.0, 16.0, 28.0, 27.0]),
            local(&la, &[2011.0, 0.0, 5.0, 21.0, 34.0, 12.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 3.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 4.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 5.0, 0.0, 0.0])
        ])
    ); // d3-time/test/ticks-test.js:149:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 2-day ticks
#[test]
fn timeticks_start_stop_count_can_generate_2_day_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 0.0, 2.0, 16.0, 28.0, 27.0]),
            local(&la, &[2011.0, 0.0, 9.0, 21.0, 34.0, 12.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 3.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 5.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 7.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 9.0, 0.0, 0.0])
        ])
    ); // d3-time/test/ticks-test.js:158:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 1-week ticks
#[test]
fn timeticks_start_stop_count_can_generate_1_week_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 0.0, 1.0, 16.0, 28.0, 27.0]),
            local(&la, &[2011.0, 0.0, 23.0, 21.0, 34.0, 12.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 9.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 16.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 23.0, 0.0, 0.0])
        ])
    ); // d3-time/test/ticks-test.js:167:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 1-month ticks
#[test]
fn timeticks_start_stop_count_can_generate_1_month_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 0.0, 18.0]),
            local(&la, &[2011.0, 4.0, 2.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 1.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 2.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 3.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 4.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-time/test/ticks-test.js:176:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 3-month ticks
#[test]
fn timeticks_start_stop_count_can_generate_3_month_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2010.0, 11.0, 18.0]),
            local(&la, &[2011.0, 10.0, 2.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 3.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 6.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 9.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-time/test/ticks-test.js:185:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate 1-year ticks
#[test]
fn timeticks_start_stop_count_can_generate_1_year_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2010.0, 11.0, 18.0]),
            local(&la, &[2014.0, 2.0, 2.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[2012.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[2013.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[2014.0, 0.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-time/test/ticks-test.js:194:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) can generate multi-year ticks
#[test]
fn timeticks_start_stop_count_can_generate_multi_year_ticks() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[0.0, 11.0, 18.0]),
            local(&la, &[2014.0, 2.0, 2.0]),
            6_usize,
            &la
        )),
        tss(&[
            local(&la, &[500.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[1000.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[1500.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[2000.0, 0.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-time/test/ticks-test.js:203:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) returns one tick for an empty domain
#[test]
fn timeticks_start_stop_count_returns_one_tick_for_an_empty_domain() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2014.0, 2.0, 2.0]),
            local(&la, &[2014.0, 2.0, 2.0]),
            6_usize,
            &la
        )),
        tss(&[local(&la, &[2014.0, 2.0, 2.0])])
    ); // d3-time/test/ticks-test.js:212:3
}

// d3-time/test/ticks-test.js: timeTicks(start, stop, count) returns descending ticks for a descending domain
#[test]
fn timeticks_start_stop_count_returns_descending_ticks_for_a_descending_domain() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2014.0, 2.0, 2.0]),
            local(&la, &[2010.0, 11.0, 18.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2014.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[2013.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[2012.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-time/test/ticks-test.js:216:3
    assert_eq!(
        tss(&time::ticks(
            local(&la, &[2011.0, 10.0, 2.0]),
            local(&la, &[2010.0, 11.0, 18.0]),
            4_usize,
            &la
        )),
        tss(&[
            local(&la, &[2011.0, 9.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 6.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 3.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-time/test/ticks-test.js:217:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, interval) respects the specified interval
#[test]
fn utcticks_start_stop_interval_respects_the_specified_interval() {
    assert_eq!(
        tss(&time::ticks_with(
            utc(&[2011.0, 0.0, 1.0, 12.0, 1.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 4.0, 4.0]),
            Interval::Minute(1),
            &TimeZone::UTC
        )),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 1.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 2.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 3.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 4.0])
        ])
    ); // d3-time/test/utcTicks-test.js:6:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, interval.every(step)) observes the specified tick interval and step
#[test]
fn utcticks_start_stop_interval_every_step_observes_the_specified_tick_interval_and_step() {
    assert_eq!(
        tss(&time::ticks_with(
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 33.0, 4.0]),
            Interval::Minute(1)
                .every(10_u32)
                .expect("d3 test: a step of 1 or more"),
            &TimeZone::UTC
        )),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 10.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 20.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 30.0])
        ])
    ); // d3-time/test/utcTicks-test.js:15:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate sub-second ticks
#[test]
fn utcticks_start_stop_count_can_generate_sub_second_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 1.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 200.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 400.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 600.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 800.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 1.0, 0.0])
        ])
    ); // d3-time/test/utcTicks-test.js:24:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 1-second ticks
#[test]
fn utcticks_start_stop_count_can_generate_1_second_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 4.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 1.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 2.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 3.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 4.0])
        ])
    ); // d3-time/test/utcTicks-test.js:35:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 5-second ticks
#[test]
fn utcticks_start_stop_count_can_generate_5_second_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 20.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 5.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 10.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 15.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 20.0])
        ])
    ); // d3-time/test/utcTicks-test.js:45:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 15-second ticks
#[test]
fn utcticks_start_stop_count_can_generate_15_second_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 50.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 15.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 30.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 45.0])
        ])
    ); // d3-time/test/utcTicks-test.js:55:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 30-second ticks
#[test]
fn utcticks_start_stop_count_can_generate_30_second_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 1.0, 50.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 30.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 1.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 1.0, 30.0])
        ])
    ); // d3-time/test/utcTicks-test.js:64:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 1-minute ticks
#[test]
fn utcticks_start_stop_count_can_generate_1_minute_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 27.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 4.0, 12.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 1.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 2.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 3.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 4.0])
        ])
    ); // d3-time/test/utcTicks-test.js:73:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 5-minute ticks
#[test]
fn utcticks_start_stop_count_can_generate_5_minute_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 0.0, 1.0, 12.0, 3.0, 27.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 21.0, 12.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 5.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 10.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 15.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 20.0])
        ])
    ); // d3-time/test/utcTicks-test.js:82:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 15-minute ticks
#[test]
fn utcticks_start_stop_count_can_generate_15_minute_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 0.0, 1.0, 12.0, 8.0, 27.0]),
            utc(&[2011.0, 0.0, 1.0, 13.0, 4.0, 12.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 15.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 30.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 45.0]),
            utc(&[2011.0, 0.0, 1.0, 13.0, 0.0])
        ])
    ); // d3-time/test/utcTicks-test.js:91:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 30-minute ticks
#[test]
fn utcticks_start_stop_count_can_generate_30_minute_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 0.0, 1.0, 12.0, 28.0, 27.0]),
            utc(&[2011.0, 0.0, 1.0, 14.0, 4.0, 12.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 30.0]),
            utc(&[2011.0, 0.0, 1.0, 13.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 13.0, 30.0]),
            utc(&[2011.0, 0.0, 1.0, 14.0, 0.0])
        ])
    ); // d3-time/test/utcTicks-test.js:100:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 1-hour ticks
#[test]
fn utcticks_start_stop_count_can_generate_1_hour_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 0.0, 1.0, 12.0, 28.0, 27.0]),
            utc(&[2011.0, 0.0, 1.0, 16.0, 34.0, 12.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 13.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 14.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 15.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 16.0, 0.0])
        ])
    ); // d3-time/test/utcTicks-test.js:109:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 3-hour ticks
#[test]
fn utcticks_start_stop_count_can_generate_3_hour_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 0.0, 1.0, 14.0, 28.0, 27.0]),
            utc(&[2011.0, 0.0, 2.0, 1.0, 34.0, 12.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 15.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 18.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 21.0, 0.0]),
            utc(&[2011.0, 0.0, 2.0, 0.0, 0.0])
        ])
    ); // d3-time/test/utcTicks-test.js:118:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 6-hour ticks
#[test]
fn utcticks_start_stop_count_can_generate_6_hour_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 0.0, 1.0, 16.0, 28.0, 27.0]),
            utc(&[2011.0, 0.0, 2.0, 14.0, 34.0, 12.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 18.0, 0.0]),
            utc(&[2011.0, 0.0, 2.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 2.0, 6.0, 0.0]),
            utc(&[2011.0, 0.0, 2.0, 12.0, 0.0])
        ])
    ); // d3-time/test/utcTicks-test.js:127:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 12-hour ticks
#[test]
fn utcticks_start_stop_count_can_generate_12_hour_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 0.0, 1.0, 16.0, 28.0, 27.0]),
            utc(&[2011.0, 0.0, 3.0, 21.0, 34.0, 12.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 2.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 2.0, 12.0, 0.0]),
            utc(&[2011.0, 0.0, 3.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 3.0, 12.0, 0.0])
        ])
    ); // d3-time/test/utcTicks-test.js:136:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 1-day ticks
#[test]
fn utcticks_start_stop_count_can_generate_1_day_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 0.0, 1.0, 16.0, 28.0, 27.0]),
            utc(&[2011.0, 0.0, 5.0, 21.0, 34.0, 12.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 2.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 3.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 4.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 5.0, 0.0, 0.0])
        ])
    ); // d3-time/test/utcTicks-test.js:145:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 2-day ticks
#[test]
fn utcticks_start_stop_count_can_generate_2_day_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 0.0, 2.0, 16.0, 28.0, 27.0]),
            utc(&[2011.0, 0.0, 9.0, 21.0, 34.0, 12.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 4.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 6.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 8.0, 0.0, 0.0])
        ])
    ); // d3-time/test/utcTicks-test.js:154:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 1-week ticks
#[test]
fn utcticks_start_stop_count_can_generate_1_week_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 0.0, 1.0, 16.0, 28.0, 27.0]),
            utc(&[2011.0, 0.0, 23.0, 21.0, 34.0, 12.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 2.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 9.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 16.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 23.0, 0.0, 0.0])
        ])
    ); // d3-time/test/utcTicks-test.js:162:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 1-month ticks
#[test]
fn utcticks_start_stop_count_can_generate_1_month_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 0.0, 18.0]),
            utc(&[2011.0, 4.0, 2.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 1.0, 1.0, 0.0, 0.0]),
            utc(&[2011.0, 2.0, 1.0, 0.0, 0.0]),
            utc(&[2011.0, 3.0, 1.0, 0.0, 0.0]),
            utc(&[2011.0, 4.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-time/test/utcTicks-test.js:171:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 3-month ticks
#[test]
fn utcticks_start_stop_count_can_generate_3_month_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2010.0, 11.0, 18.0]),
            utc(&[2011.0, 10.0, 2.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[2011.0, 3.0, 1.0, 0.0, 0.0]),
            utc(&[2011.0, 6.0, 1.0, 0.0, 0.0]),
            utc(&[2011.0, 9.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-time/test/utcTicks-test.js:180:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate 1-year ticks
#[test]
fn utcticks_start_stop_count_can_generate_1_year_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2010.0, 11.0, 18.0]),
            utc(&[2014.0, 2.0, 2.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[2012.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[2013.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[2014.0, 0.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-time/test/utcTicks-test.js:189:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) can generate multi-year ticks
#[test]
fn utcticks_start_stop_count_can_generate_multi_year_ticks() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[0.0, 11.0, 18.0]),
            utc(&[2014.0, 2.0, 2.0]),
            6_usize
        )),
        tss(&[
            utc(&[500.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[1000.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[1500.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[2000.0, 0.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-time/test/utcTicks-test.js:198:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) returns one tick for an empty domain
#[test]
fn utcticks_start_stop_count_returns_one_tick_for_an_empty_domain() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2014.0, 2.0, 2.0]),
            utc(&[2014.0, 2.0, 2.0]),
            6_usize
        )),
        tss(&[utc(&[2014.0, 2.0, 2.0])])
    ); // d3-time/test/utcTicks-test.js:207:3
}

// d3-time/test/utcTicks-test.js: utcTicks(start, stop, count) returns descending ticks for a descending domain
#[test]
fn utcticks_start_stop_count_returns_descending_ticks_for_a_descending_domain() {
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2014.0, 2.0, 2.0]),
            utc(&[2010.0, 11.0, 18.0]),
            4_usize
        )),
        tss(&[
            utc(&[2014.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[2013.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[2012.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-time/test/utcTicks-test.js:211:3
    assert_eq!(
        tss(&time::utc_ticks(
            utc(&[2011.0, 10.0, 2.0]),
            utc(&[2010.0, 11.0, 18.0]),
            4_usize
        )),
        tss(&[
            utc(&[2011.0, 9.0, 1.0, 0.0, 0.0]),
            utc(&[2011.0, 6.0, 1.0, 0.0, 0.0]),
            utc(&[2011.0, 3.0, 1.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-time/test/utcTicks-test.js:212:3
}
