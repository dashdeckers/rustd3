//! d3's tests of d3-scale, as assertions on rustd3-core; each names d3's file, line and column.

use rustd3_core::jiff::civil::Weekday;
use rustd3_core::jiff::tz::TimeZone;
use rustd3_core::time::{self, Interval};

use super::support::{local, ts, tss, utc, zone};

// d3-scale/test/time-test.js: time.nice() is an alias for time.nice(10)
#[test]
fn time_nice_is_an_alias_for_time_nice_10() {
    let la = zone("America/Los_Angeles");
    let mut x = time::time(la.clone()).domain([
        local(&la, &[2009.0, 0.0, 1.0, 0.0, 17.0]),
        local(&la, &[2009.0, 0.0, 1.0, 23.0, 42.0]),
    ]);
    assert_eq!(
        tss(&({
            x = x.nice(10_usize);
            x.get_domain()
        })),
        tss(&[
            local(&la, &[2009.0, 0.0, 1.0]),
            local(&la, &[2009.0, 0.0, 2.0])
        ])
    ); // d3-scale/test/time-test.js:21:3
}

// d3-scale/test/time-test.js: time.nice() can nice sub-second domains
#[test]
fn time_nice_can_nice_sub_second_domains() {
    let la = zone("America/Los_Angeles");
    let mut x = time::time(la.clone()).domain([
        local(&la, &[2013.0, 0.0, 1.0, 12.0, 0.0, 0.0, 0.0]),
        local(&la, &[2013.0, 0.0, 1.0, 12.0, 0.0, 0.0, 128.0]),
    ]);
    assert_eq!(
        tss(&({
            x = x.nice(10_usize);
            x.get_domain()
        })),
        tss(&[
            local(&la, &[2013.0, 0.0, 1.0, 12.0, 0.0, 0.0, 0.0]),
            local(&la, &[2013.0, 0.0, 1.0, 12.0, 0.0, 0.0, 130.0])
        ])
    ); // d3-scale/test/time-test.js:26:3
}

// d3-scale/test/time-test.js: time.nice() can nice multi-year domains
#[test]
fn time_nice_can_nice_multi_year_domains() {
    let la = zone("America/Los_Angeles");
    let mut x = time::time(la.clone()).domain([
        local(&la, &[2001.0, 0.0, 1.0]),
        local(&la, &[2138.0, 0.0, 1.0]),
    ]);
    assert_eq!(
        tss(&({
            x = x.nice(10_usize);
            x.get_domain()
        })),
        tss(&[
            local(&la, &[2000.0, 0.0, 1.0]),
            local(&la, &[2140.0, 0.0, 1.0])
        ])
    ); // d3-scale/test/time-test.js:31:3
}

// d3-scale/test/time-test.js: time.nice() can nice empty domains
#[test]
fn time_nice_can_nice_empty_domains() {
    let la = zone("America/Los_Angeles");
    let mut x = time::time(la.clone()).domain([
        local(&la, &[2009.0, 0.0, 1.0, 0.0, 12.0]),
        local(&la, &[2009.0, 0.0, 1.0, 0.0, 12.0]),
    ]);
    assert_eq!(
        tss(&({
            x = x.nice(10_usize);
            x.get_domain()
        })),
        tss(&[
            local(&la, &[2009.0, 0.0, 1.0, 0.0, 12.0]),
            local(&la, &[2009.0, 0.0, 1.0, 0.0, 12.0])
        ])
    ); // d3-scale/test/time-test.js:36:3
}

// d3-scale/test/time-test.js: time.nice(count) nices using the specified tick count
#[test]
fn time_nice_count_nices_using_the_specified_tick_count() {
    let la = zone("America/Los_Angeles");
    let mut x = time::time(la.clone()).domain([
        local(&la, &[2009.0, 0.0, 1.0, 0.0, 17.0]),
        local(&la, &[2009.0, 0.0, 1.0, 23.0, 42.0]),
    ]);
    assert_eq!(
        tss(&({
            x = x.nice(100_usize);
            x.get_domain()
        })),
        tss(&[
            local(&la, &[2009.0, 0.0, 1.0, 0.0, 15.0]),
            local(&la, &[2009.0, 0.0, 1.0, 23.0, 45.0])
        ])
    ); // d3-scale/test/time-test.js:41:3
    assert_eq!(
        tss(&({
            x = x.nice(10_usize);
            x.get_domain()
        })),
        tss(&[
            local(&la, &[2009.0, 0.0, 1.0]),
            local(&la, &[2009.0, 0.0, 2.0])
        ])
    ); // d3-scale/test/time-test.js:42:3
}

// d3-scale/test/time-test.js: time.nice(interval) nices using the specified time interval
#[test]
fn time_nice_interval_nices_using_the_specified_time_interval() {
    let la = zone("America/Los_Angeles");
    let mut x = time::time(la.clone()).domain([
        local(&la, &[2009.0, 0.0, 1.0, 0.0, 12.0]),
        local(&la, &[2009.0, 0.0, 1.0, 23.0, 48.0]),
    ]);
    assert_eq!(
        tss(&({
            x = x.nice_with(Interval::Day(1));
            x.get_domain()
        })),
        tss(&[
            local(&la, &[2009.0, 0.0, 1.0]),
            local(&la, &[2009.0, 0.0, 2.0])
        ])
    ); // d3-scale/test/time-test.js:47:3
    assert_eq!(
        tss(&({
            x = x.nice_with(Interval::Week(Weekday::Sunday, 1));
            x.get_domain()
        })),
        tss(&[
            local(&la, &[2008.0, 11.0, 28.0]),
            local(&la, &[2009.0, 0.0, 4.0])
        ])
    ); // d3-scale/test/time-test.js:48:3
    assert_eq!(
        tss(&({
            x = x.nice_with(Interval::Month(1));
            x.get_domain()
        })),
        tss(&[
            local(&la, &[2008.0, 11.0, 1.0]),
            local(&la, &[2009.0, 1.0, 1.0])
        ])
    ); // d3-scale/test/time-test.js:49:3
    assert_eq!(
        tss(&({
            x = x.nice_with(Interval::Year(1));
            x.get_domain()
        })),
        tss(&[
            local(&la, &[2008.0, 0.0, 1.0]),
            local(&la, &[2010.0, 0.0, 1.0])
        ])
    ); // d3-scale/test/time-test.js:50:3
}

// d3-scale/test/time-test.js: time.nice(interval) can nice empty domains
#[test]
fn time_nice_interval_can_nice_empty_domains() {
    let la = zone("America/Los_Angeles");
    let mut x = time::time(la.clone()).domain([
        local(&la, &[2009.0, 0.0, 1.0, 0.0, 12.0]),
        local(&la, &[2009.0, 0.0, 1.0, 0.0, 12.0]),
    ]);
    assert_eq!(
        tss(&({
            x = x.nice_with(Interval::Day(1));
            x.get_domain()
        })),
        tss(&[
            local(&la, &[2009.0, 0.0, 1.0]),
            local(&la, &[2009.0, 0.0, 2.0])
        ])
    ); // d3-scale/test/time-test.js:55:3
}

// d3-scale/test/time-test.js: time.nice(interval.every(step)) nices using the specified time interval and step
#[test]
fn time_nice_interval_every_step_nices_using_the_specified_time_interval_and_step() {
    let la = zone("America/Los_Angeles");
    let mut x = time::time(la.clone()).domain([
        local(&la, &[2009.0, 0.0, 1.0, 0.0, 12.0]),
        local(&la, &[2009.0, 0.0, 1.0, 23.0, 48.0]),
    ]);
    assert_eq!(
        tss(&({
            x = x.nice_with(
                Interval::Day(1)
                    .every(3_u32)
                    .expect("d3 test: a step of 1 or more"),
            );
            x.get_domain()
        })),
        tss(&[
            local(&la, &[2009.0, 0.0, 1.0]),
            local(&la, &[2009.0, 0.0, 4.0])
        ])
    ); // d3-scale/test/time-test.js:65:3
    assert_eq!(
        tss(&({
            x = x.nice_with(
                Interval::Week(Weekday::Sunday, 1)
                    .every(2_u32)
                    .expect("d3 test: a step of 1 or more"),
            );
            x.get_domain()
        })),
        tss(&[
            local(&la, &[2008.0, 11.0, 21.0]),
            local(&la, &[2009.0, 0.0, 4.0])
        ])
    ); // d3-scale/test/time-test.js:66:3
    assert_eq!(
        tss(&({
            x = x.nice_with(
                Interval::Month(1)
                    .every(3_u32)
                    .expect("d3 test: a step of 1 or more"),
            );
            x.get_domain()
        })),
        tss(&[
            local(&la, &[2008.0, 9.0, 1.0]),
            local(&la, &[2009.0, 3.0, 1.0])
        ])
    ); // d3-scale/test/time-test.js:67:3
    assert_eq!(
        tss(&({
            x = x.nice_with(
                Interval::Year(1)
                    .every(10_u32)
                    .expect("d3 test: a step of 1 or more"),
            );
            x.get_domain()
        })),
        tss(&[
            local(&la, &[2000.0, 0.0, 1.0]),
            local(&la, &[2010.0, 0.0, 1.0])
        ])
    ); // d3-scale/test/time-test.js:68:3
}

// d3-scale/test/time-test.js: time.copy() isolates changes to the domain
#[test]
fn time_copy_isolates_changes_to_the_domain() {
    let la = zone("America/Los_Angeles");
    let mut x = time::time(la.clone()).domain([
        local(&la, &[2009.0, 0.0, 1.0]),
        local(&la, &[2010.0, 0.0, 1.0]),
    ]);
    let mut y = x.clone();
    x = x.domain([
        local(&la, &[2010.0, 0.0, 1.0]),
        local(&la, &[2011.0, 0.0, 1.0]),
    ]);
    assert_eq!(
        tss(&y.get_domain()),
        tss(&[
            local(&la, &[2009.0, 0.0, 1.0]),
            local(&la, &[2010.0, 0.0, 1.0])
        ])
    ); // d3-scale/test/time-test.js:74:3
    assert_eq!(x.apply(local(&la, &[2010.0, 0.0, 1.0])), 0.0); // d3-scale/test/time-test.js:75:3
    assert_eq!(y.apply(local(&la, &[2010.0, 0.0, 1.0])), 1.0); // d3-scale/test/time-test.js:76:3
    y = y.domain([
        local(&la, &[2011.0, 0.0, 1.0]),
        local(&la, &[2012.0, 0.0, 1.0]),
    ]);
    assert_eq!(x.apply(local(&la, &[2011.0, 0.0, 1.0])), 1.0); // d3-scale/test/time-test.js:78:3
    assert_eq!(y.apply(local(&la, &[2011.0, 0.0, 1.0])), 0.0); // d3-scale/test/time-test.js:79:3
    assert_eq!(
        tss(&x.get_domain()),
        tss(&[
            local(&la, &[2010.0, 0.0, 1.0]),
            local(&la, &[2011.0, 0.0, 1.0])
        ])
    ); // d3-scale/test/time-test.js:80:3
    assert_eq!(
        tss(&y.get_domain()),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0]),
            local(&la, &[2012.0, 0.0, 1.0])
        ])
    ); // d3-scale/test/time-test.js:81:3
}

// d3-scale/test/time-test.js: time.copy() isolates changes to the range
#[test]
fn time_copy_isolates_changes_to_the_range() {
    let la = zone("America/Los_Angeles");
    let mut x = time::time(la.clone()).domain([
        local(&la, &[2009.0, 0.0, 1.0]),
        local(&la, &[2010.0, 0.0, 1.0]),
    ]);
    let mut y = x.clone();
    x = x.range([1.0, 2.0]);
    assert_eq!(ts(x.invert(1.0)), ts(local(&la, &[2009.0, 0.0, 1.0]))); // d3-scale/test/time-test.js:87:3
    assert_eq!(ts(y.invert(1.0)), ts(local(&la, &[2010.0, 0.0, 1.0]))); // d3-scale/test/time-test.js:88:3
    assert_eq!(y.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/time-test.js:89:3
    y = y.range([2.0, 3.0]);
    assert_eq!(ts(x.invert(2.0)), ts(local(&la, &[2010.0, 0.0, 1.0]))); // d3-scale/test/time-test.js:91:3
    assert_eq!(ts(y.invert(2.0)), ts(local(&la, &[2009.0, 0.0, 1.0]))); // d3-scale/test/time-test.js:92:3
    assert_eq!(x.get_range().to_vec(), vec![1.0, 2.0]); // d3-scale/test/time-test.js:93:3
    assert_eq!(y.get_range().to_vec(), vec![2.0, 3.0]); // d3-scale/test/time-test.js:94:3
}

// d3-scale/test/time-test.js: time.copy() isolates changes to clamping
#[test]
fn time_copy_isolates_changes_to_clamping() {
    let la = zone("America/Los_Angeles");
    let mut x = time::time(la.clone())
        .domain([
            local(&la, &[2009.0, 0.0, 1.0]),
            local(&la, &[2010.0, 0.0, 1.0]),
        ])
        .clamp(true);
    let mut y = x.clone();
    x = x.clamp(false);
    assert_eq!(x.apply(local(&la, &[2011.0, 0.0, 1.0])), 2.0); // d3-scale/test/time-test.js:110:3
    assert_eq!(y.apply(local(&la, &[2011.0, 0.0, 1.0])), 1.0); // d3-scale/test/time-test.js:111:3
    y = y.clamp(false);
    assert_eq!(x.apply(local(&la, &[2011.0, 0.0, 1.0])), 2.0); // d3-scale/test/time-test.js:114:3
    assert_eq!(y.apply(local(&la, &[2011.0, 0.0, 1.0])), 2.0); // d3-scale/test/time-test.js:115:3
}

// d3-scale/test/time-test.js: time.clamp(true).invert(value) never returns a value outside the domain
#[test]
fn time_clamp_true_invert_value_never_returns_a_value_outside_the_domain() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).clamp(true);
    assert_eq!(
        x.invert(-1.0).as_millisecond() as f64,
        x.get_domain()[0_usize].as_millisecond() as f64
    ); // d3-scale/test/time-test.js:123:3
    assert_eq!(
        x.invert(0.0).as_millisecond() as f64,
        x.get_domain()[0_usize].as_millisecond() as f64
    ); // d3-scale/test/time-test.js:124:3
    assert_eq!(
        x.invert(1.0).as_millisecond() as f64,
        x.get_domain()[1_usize].as_millisecond() as f64
    ); // d3-scale/test/time-test.js:125:3
    assert_eq!(
        x.invert(2.0).as_millisecond() as f64,
        x.get_domain()[1_usize].as_millisecond() as f64
    ); // d3-scale/test/time-test.js:126:3
}

// d3-scale/test/time-test.js: time.ticks(interval) observes the specified tick interval
#[test]
fn time_ticks_interval_observes_the_specified_tick_interval() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 1.0, 0.0]),
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 4.0, 4.0]),
    ]);
    assert_eq!(
        tss(&x.ticks_with(Interval::Minute(1))),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 1.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 2.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 3.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 4.0])
        ])
    ); // d3-scale/test/time-test.js:131:3
}

// d3-scale/test/time-test.js: time.ticks(interval.every(step)) observes the specified tick interval and step
#[test]
fn time_ticks_interval_every_step_observes_the_specified_tick_interval_and_step() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 33.0, 4.0]),
    ]);
    assert_eq!(
        tss(&x.ticks_with(
            Interval::Minute(1)
                .every(10_u32)
                .expect("d3 test: a step of 1 or more")
        )),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 10.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 20.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 30.0])
        ])
    ); // d3-scale/test/time-test.js:141:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate sub-second ticks
#[test]
fn time_ticks_count_can_generate_sub_second_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 1.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 200.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 400.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 600.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 800.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 1.0, 0.0])
        ])
    ); // d3-scale/test/time-test.js:151:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 1-second ticks
#[test]
fn time_ticks_count_can_generate_1_second_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 4.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 1.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 2.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 3.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 4.0])
        ])
    ); // d3-scale/test/time-test.js:163:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 5-second ticks
#[test]
fn time_ticks_count_can_generate_5_second_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 20.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 5.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 10.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 15.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 20.0])
        ])
    ); // d3-scale/test/time-test.js:174:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 15-second ticks
#[test]
fn time_ticks_count_can_generate_15_second_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 50.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 15.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 30.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 45.0])
        ])
    ); // d3-scale/test/time-test.js:185:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 30-second ticks
#[test]
fn time_ticks_count_can_generate_30_second_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 1.0, 50.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 30.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 1.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 1.0, 30.0])
        ])
    ); // d3-scale/test/time-test.js:195:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 1-minute ticks
#[test]
fn time_ticks_count_can_generate_1_minute_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 0.0, 27.0]),
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 4.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 1.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 2.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 3.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 4.0])
        ])
    ); // d3-scale/test/time-test.js:205:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 5-minute ticks
#[test]
fn time_ticks_count_can_generate_5_minute_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 3.0, 27.0]),
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 21.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 5.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 10.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 15.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 20.0])
        ])
    ); // d3-scale/test/time-test.js:215:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 15-minute ticks
#[test]
fn time_ticks_count_can_generate_15_minute_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 8.0, 27.0]),
        local(&la, &[2011.0, 0.0, 1.0, 13.0, 4.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 15.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 30.0]),
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 45.0]),
            local(&la, &[2011.0, 0.0, 1.0, 13.0, 0.0])
        ])
    ); // d3-scale/test/time-test.js:225:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 30-minute ticks
#[test]
fn time_ticks_count_can_generate_30_minute_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 28.0, 27.0]),
        local(&la, &[2011.0, 0.0, 1.0, 14.0, 4.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 12.0, 30.0]),
            local(&la, &[2011.0, 0.0, 1.0, 13.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 13.0, 30.0]),
            local(&la, &[2011.0, 0.0, 1.0, 14.0, 0.0])
        ])
    ); // d3-scale/test/time-test.js:235:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 1-hour ticks
#[test]
fn time_ticks_count_can_generate_1_hour_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 1.0, 12.0, 28.0, 27.0]),
        local(&la, &[2011.0, 0.0, 1.0, 16.0, 34.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 13.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 14.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 15.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 16.0, 0.0])
        ])
    ); // d3-scale/test/time-test.js:245:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 3-hour ticks
#[test]
fn time_ticks_count_can_generate_3_hour_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 1.0, 14.0, 28.0, 27.0]),
        local(&la, &[2011.0, 0.0, 2.0, 1.0, 34.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 15.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 18.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 21.0, 0.0]),
            local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0])
        ])
    ); // d3-scale/test/time-test.js:255:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 6-hour ticks
#[test]
fn time_ticks_count_can_generate_6_hour_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 1.0, 16.0, 28.0, 27.0]),
        local(&la, &[2011.0, 0.0, 2.0, 14.0, 34.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 18.0, 0.0]),
            local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 2.0, 6.0, 0.0]),
            local(&la, &[2011.0, 0.0, 2.0, 12.0, 0.0])
        ])
    ); // d3-scale/test/time-test.js:265:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 12-hour ticks
#[test]
fn time_ticks_count_can_generate_12_hour_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 1.0, 16.0, 28.0, 27.0]),
        local(&la, &[2011.0, 0.0, 3.0, 21.0, 34.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 2.0, 12.0, 0.0]),
            local(&la, &[2011.0, 0.0, 3.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 3.0, 12.0, 0.0])
        ])
    ); // d3-scale/test/time-test.js:275:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 1-day ticks
#[test]
fn time_ticks_count_can_generate_1_day_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 1.0, 16.0, 28.0, 27.0]),
        local(&la, &[2011.0, 0.0, 5.0, 21.0, 34.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 3.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 4.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 5.0, 0.0, 0.0])
        ])
    ); // d3-scale/test/time-test.js:285:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 2-day ticks
#[test]
fn time_ticks_count_can_generate_2_day_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 2.0, 16.0, 28.0, 27.0]),
        local(&la, &[2011.0, 0.0, 9.0, 21.0, 34.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 3.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 5.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 7.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 9.0, 0.0, 0.0])
        ])
    ); // d3-scale/test/time-test.js:295:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 1-week ticks
#[test]
fn time_ticks_count_can_generate_1_week_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 1.0, 16.0, 28.0, 27.0]),
        local(&la, &[2011.0, 0.0, 23.0, 21.0, 34.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 2.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 9.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 16.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 23.0, 0.0, 0.0])
        ])
    ); // d3-scale/test/time-test.js:305:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 1-month ticks
#[test]
fn time_ticks_count_can_generate_1_month_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2011.0, 0.0, 18.0]),
        local(&la, &[2011.0, 4.0, 2.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 1.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 2.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 3.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 4.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-scale/test/time-test.js:315:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 3-month ticks
#[test]
fn time_ticks_count_can_generate_3_month_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2010.0, 11.0, 18.0]),
        local(&la, &[2011.0, 10.0, 2.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 3.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 6.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 9.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-scale/test/time-test.js:325:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate 1-year ticks
#[test]
fn time_ticks_count_can_generate_1_year_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2010.0, 11.0, 18.0]),
        local(&la, &[2014.0, 2.0, 2.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[2012.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[2013.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[2014.0, 0.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-scale/test/time-test.js:335:3
}

// d3-scale/test/time-test.js: time.ticks(count) can generate multi-year ticks
#[test]
fn time_ticks_count_can_generate_multi_year_ticks() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[0.0, 11.0, 18.0]),
        local(&la, &[2014.0, 2.0, 2.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(6_usize)),
        tss(&[
            local(&la, &[500.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[1000.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[1500.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[2000.0, 0.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-scale/test/time-test.js:345:3
}

// d3-scale/test/time-test.js: time.ticks(count) returns one tick for an empty domain
#[test]
fn time_ticks_count_returns_one_tick_for_an_empty_domain() {
    let la = zone("America/Los_Angeles");
    let x = time::time(la.clone()).domain([
        local(&la, &[2014.0, 2.0, 2.0]),
        local(&la, &[2014.0, 2.0, 2.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(6_usize)),
        tss(&[local(&la, &[2014.0, 2.0, 2.0])])
    ); // d3-scale/test/time-test.js:355:3
}

// d3-scale/test/time-test.js: time.ticks() returns descending ticks for a descending domain
#[test]
fn time_ticks_returns_descending_ticks_for_a_descending_domain() {
    let la = zone("America/Los_Angeles");
    let mut x = time::time(la.clone());
    assert_eq!(
        tss(&({
            x = x.domain([
                local(&la, &[2014.0, 2.0, 2.0]),
                local(&la, &[2010.0, 11.0, 18.0]),
            ]);
            x.ticks(4_usize)
        })),
        tss(&[
            local(&la, &[2014.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[2013.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[2012.0, 0.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-scale/test/time-test.js:360:3
    assert_eq!(
        tss(&({
            x = x.domain([
                local(&la, &[2011.0, 10.0, 2.0]),
                local(&la, &[2010.0, 11.0, 18.0]),
            ]);
            x.ticks(4_usize)
        })),
        tss(&[
            local(&la, &[2011.0, 9.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 6.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 3.0, 1.0, 0.0, 0.0]),
            local(&la, &[2011.0, 0.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-scale/test/time-test.js:361:3
}

// d3-scale/test/time-test.js: time.tickFormat()(date) formats year on New Year's
#[test]
fn time_tickformat_date_formats_year_on_new_year_s() {
    let la = zone("America/Los_Angeles");
    let f = time::time(la.clone());
    assert_eq!(
        f.tick_label(local(&la, &[2011.0, 0.0, 1.0]), &time::Locale::EN_US),
        "2011"
    ); // d3-scale/test/time-test.js:366:3
    assert_eq!(
        f.tick_label(local(&la, &[2012.0, 0.0, 1.0]), &time::Locale::EN_US),
        "2012"
    ); // d3-scale/test/time-test.js:367:3
    assert_eq!(
        f.tick_label(local(&la, &[2013.0, 0.0, 1.0]), &time::Locale::EN_US),
        "2013"
    ); // d3-scale/test/time-test.js:368:3
}

// d3-scale/test/time-test.js: time.tickFormat()(date) formats month on the 1st of each month
#[test]
fn time_tickformat_date_formats_month_on_the_1st_of_each_month() {
    let la = zone("America/Los_Angeles");
    let f = time::time(la.clone());
    assert_eq!(
        f.tick_label(local(&la, &[2011.0, 1.0, 1.0]), &time::Locale::EN_US),
        "February"
    ); // d3-scale/test/time-test.js:373:3
    assert_eq!(
        f.tick_label(local(&la, &[2011.0, 2.0, 1.0]), &time::Locale::EN_US),
        "March"
    ); // d3-scale/test/time-test.js:374:3
    assert_eq!(
        f.tick_label(local(&la, &[2011.0, 3.0, 1.0]), &time::Locale::EN_US),
        "April"
    ); // d3-scale/test/time-test.js:375:3
}

// d3-scale/test/time-test.js: time.tickFormat()(date) formats week on Sunday midnight
#[test]
fn time_tickformat_date_formats_week_on_sunday_midnight() {
    let la = zone("America/Los_Angeles");
    let f = time::time(la.clone());
    assert_eq!(
        f.tick_label(local(&la, &[2011.0, 1.0, 6.0]), &time::Locale::EN_US),
        "Feb 06"
    ); // d3-scale/test/time-test.js:380:3
    assert_eq!(
        f.tick_label(local(&la, &[2011.0, 1.0, 13.0]), &time::Locale::EN_US),
        "Feb 13"
    ); // d3-scale/test/time-test.js:381:3
    assert_eq!(
        f.tick_label(local(&la, &[2011.0, 1.0, 20.0]), &time::Locale::EN_US),
        "Feb 20"
    ); // d3-scale/test/time-test.js:382:3
}

// d3-scale/test/time-test.js: time.tickFormat()(date) formats date on midnight
#[test]
fn time_tickformat_date_formats_date_on_midnight() {
    let la = zone("America/Los_Angeles");
    let f = time::time(la.clone());
    assert_eq!(
        f.tick_label(local(&la, &[2011.0, 1.0, 2.0]), &time::Locale::EN_US),
        "Wed 02"
    ); // d3-scale/test/time-test.js:387:3
    assert_eq!(
        f.tick_label(local(&la, &[2011.0, 1.0, 3.0]), &time::Locale::EN_US),
        "Thu 03"
    ); // d3-scale/test/time-test.js:388:3
    assert_eq!(
        f.tick_label(local(&la, &[2011.0, 1.0, 4.0]), &time::Locale::EN_US),
        "Fri 04"
    ); // d3-scale/test/time-test.js:389:3
}

// d3-scale/test/time-test.js: time.tickFormat()(date) formats hour on minute zero
#[test]
fn time_tickformat_date_formats_hour_on_minute_zero() {
    let la = zone("America/Los_Angeles");
    let f = time::time(la.clone());
    assert_eq!(
        f.tick_label(local(&la, &[2011.0, 1.0, 2.0, 11.0]), &time::Locale::EN_US),
        "11 AM"
    ); // d3-scale/test/time-test.js:394:3
    assert_eq!(
        f.tick_label(local(&la, &[2011.0, 1.0, 2.0, 12.0]), &time::Locale::EN_US),
        "12 PM"
    ); // d3-scale/test/time-test.js:395:3
    assert_eq!(
        f.tick_label(local(&la, &[2011.0, 1.0, 2.0, 13.0]), &time::Locale::EN_US),
        "01 PM"
    ); // d3-scale/test/time-test.js:396:3
}

// d3-scale/test/time-test.js: time.tickFormat()(date) formats minute on second zero
#[test]
fn time_tickformat_date_formats_minute_on_second_zero() {
    let la = zone("America/Los_Angeles");
    let f = time::time(la.clone());
    assert_eq!(
        f.tick_label(
            local(&la, &[2011.0, 1.0, 2.0, 11.0, 59.0]),
            &time::Locale::EN_US
        ),
        "11:59"
    ); // d3-scale/test/time-test.js:401:3
    assert_eq!(
        f.tick_label(
            local(&la, &[2011.0, 1.0, 2.0, 12.0, 1.0]),
            &time::Locale::EN_US
        ),
        "12:01"
    ); // d3-scale/test/time-test.js:402:3
    assert_eq!(
        f.tick_label(
            local(&la, &[2011.0, 1.0, 2.0, 12.0, 2.0]),
            &time::Locale::EN_US
        ),
        "12:02"
    ); // d3-scale/test/time-test.js:403:3
}

// d3-scale/test/time-test.js: time.tickFormat()(date) otherwise, formats second
#[test]
fn time_tickformat_date_otherwise_formats_second() {
    let la = zone("America/Los_Angeles");
    let f = time::time(la.clone());
    assert_eq!(
        f.tick_label(
            local(&la, &[2011.0, 1.0, 2.0, 12.0, 1.0, 9.0]),
            &time::Locale::EN_US
        ),
        ":09"
    ); // d3-scale/test/time-test.js:408:3
    assert_eq!(
        f.tick_label(
            local(&la, &[2011.0, 1.0, 2.0, 12.0, 1.0, 10.0]),
            &time::Locale::EN_US
        ),
        ":10"
    ); // d3-scale/test/time-test.js:409:3
    assert_eq!(
        f.tick_label(
            local(&la, &[2011.0, 1.0, 2.0, 12.0, 1.0, 11.0]),
            &time::Locale::EN_US
        ),
        ":11"
    ); // d3-scale/test/time-test.js:410:3
}

// d3-scale/test/time-test.js: time.tickFormat(count, specifier) returns a time format for the specified specifier
#[test]
fn time_tickformat_count_specifier_returns_a_time_format_for_the_specified_specifier() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%c");
    assert_eq!(
        f.format(local(&la, &[2011.0, 1.0, 2.0, 12.0]), &la),
        "2/2/2011, 12:00:00 PM"
    ); // d3-scale/test/time-test.js:415:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.nice() is an alias for scaleUtc.nice(10)
#[test]
fn scaleutc_nice_is_an_alias_for_scaleutc_nice_10() {
    let mut x = time::utc().domain([
        utc(&[2009.0, 0.0, 1.0, 0.0, 17.0]),
        utc(&[2009.0, 0.0, 1.0, 23.0, 42.0]),
    ]);
    assert_eq!(
        tss(&({
            x = x.nice(10_usize);
            x.get_domain()
        })),
        tss(&[utc(&[2009.0, 0.0, 1.0]), utc(&[2009.0, 0.0, 2.0])])
    ); // d3-scale/test/utcTime-test.js:9:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.nice() can nice sub-second domains
#[test]
fn scaleutc_nice_can_nice_sub_second_domains() {
    let mut x = time::utc().domain([
        utc(&[2013.0, 0.0, 1.0, 12.0, 0.0, 0.0, 0.0]),
        utc(&[2013.0, 0.0, 1.0, 12.0, 0.0, 0.0, 128.0]),
    ]);
    assert_eq!(
        tss(&({
            x = x.nice(10_usize);
            x.get_domain()
        })),
        tss(&[
            utc(&[2013.0, 0.0, 1.0, 12.0, 0.0, 0.0, 0.0]),
            utc(&[2013.0, 0.0, 1.0, 12.0, 0.0, 0.0, 130.0])
        ])
    ); // d3-scale/test/utcTime-test.js:14:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.nice() can nice multi-year domains
#[test]
fn scaleutc_nice_can_nice_multi_year_domains() {
    let mut x = time::utc().domain([utc(&[2001.0, 0.0, 1.0]), utc(&[2138.0, 0.0, 1.0])]);
    assert_eq!(
        tss(&({
            x = x.nice(10_usize);
            x.get_domain()
        })),
        tss(&[utc(&[2000.0, 0.0, 1.0]), utc(&[2140.0, 0.0, 1.0])])
    ); // d3-scale/test/utcTime-test.js:19:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.nice() can nice empty domains
#[test]
fn scaleutc_nice_can_nice_empty_domains() {
    let mut x = time::utc().domain([
        utc(&[2009.0, 0.0, 1.0, 0.0, 12.0]),
        utc(&[2009.0, 0.0, 1.0, 0.0, 12.0]),
    ]);
    assert_eq!(
        tss(&({
            x = x.nice(10_usize);
            x.get_domain()
        })),
        tss(&[
            utc(&[2009.0, 0.0, 1.0, 0.0, 12.0]),
            utc(&[2009.0, 0.0, 1.0, 0.0, 12.0])
        ])
    ); // d3-scale/test/utcTime-test.js:24:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.nice(count) nices using the specified tick count
#[test]
fn scaleutc_nice_count_nices_using_the_specified_tick_count() {
    let mut x = time::utc().domain([
        utc(&[2009.0, 0.0, 1.0, 0.0, 17.0]),
        utc(&[2009.0, 0.0, 1.0, 23.0, 42.0]),
    ]);
    assert_eq!(
        tss(&({
            x = x.nice(100_usize);
            x.get_domain()
        })),
        tss(&[
            utc(&[2009.0, 0.0, 1.0, 0.0, 15.0]),
            utc(&[2009.0, 0.0, 1.0, 23.0, 45.0])
        ])
    ); // d3-scale/test/utcTime-test.js:29:3
    assert_eq!(
        tss(&({
            x = x.nice(10_usize);
            x.get_domain()
        })),
        tss(&[utc(&[2009.0, 0.0, 1.0]), utc(&[2009.0, 0.0, 2.0])])
    ); // d3-scale/test/utcTime-test.js:30:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.nice(interval) nices using the specified time interval
#[test]
fn scaleutc_nice_interval_nices_using_the_specified_time_interval() {
    let mut x = time::utc().domain([
        utc(&[2009.0, 0.0, 1.0, 0.0, 12.0]),
        utc(&[2009.0, 0.0, 1.0, 23.0, 48.0]),
    ]);
    assert_eq!(
        tss(&({
            x = x.nice_with(Interval::Day(1));
            x.get_domain()
        })),
        tss(&[utc(&[2009.0, 0.0, 1.0]), utc(&[2009.0, 0.0, 2.0])])
    ); // d3-scale/test/utcTime-test.js:35:3
    assert_eq!(
        tss(&({
            x = x.nice_with(Interval::Week(Weekday::Sunday, 1));
            x.get_domain()
        })),
        tss(&[utc(&[2008.0, 11.0, 28.0]), utc(&[2009.0, 0.0, 4.0])])
    ); // d3-scale/test/utcTime-test.js:36:3
    assert_eq!(
        tss(&({
            x = x.nice_with(Interval::Month(1));
            x.get_domain()
        })),
        tss(&[utc(&[2008.0, 11.0, 1.0]), utc(&[2009.0, 1.0, 1.0])])
    ); // d3-scale/test/utcTime-test.js:37:3
    assert_eq!(
        tss(&({
            x = x.nice_with(Interval::Year(1));
            x.get_domain()
        })),
        tss(&[utc(&[2008.0, 0.0, 1.0]), utc(&[2010.0, 0.0, 1.0])])
    ); // d3-scale/test/utcTime-test.js:38:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.nice(interval) can nice empty domains
#[test]
fn scaleutc_nice_interval_can_nice_empty_domains() {
    let mut x = time::utc().domain([
        utc(&[2009.0, 0.0, 1.0, 0.0, 12.0]),
        utc(&[2009.0, 0.0, 1.0, 0.0, 12.0]),
    ]);
    assert_eq!(
        tss(&({
            x = x.nice_with(Interval::Day(1));
            x.get_domain()
        })),
        tss(&[utc(&[2009.0, 0.0, 1.0]), utc(&[2009.0, 0.0, 2.0])])
    ); // d3-scale/test/utcTime-test.js:43:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.nice(interval.every(step)) nices using the specified time interval and step
#[test]
fn scaleutc_nice_interval_every_step_nices_using_the_specified_time_interval_and_step() {
    let mut x = time::utc().domain([
        utc(&[2009.0, 0.0, 1.0, 0.0, 12.0]),
        utc(&[2009.0, 0.0, 1.0, 23.0, 48.0]),
    ]);
    assert_eq!(
        tss(&({
            x = x.nice_with(
                Interval::Day(1)
                    .every(3_u32)
                    .expect("d3 test: a step of 1 or more"),
            );
            x.get_domain()
        })),
        tss(&[utc(&[2009.0, 0.0, 1.0]), utc(&[2009.0, 0.0, 4.0])])
    ); // d3-scale/test/utcTime-test.js:53:3
    assert_eq!(
        tss(&({
            x = x.nice_with(
                Interval::Week(Weekday::Sunday, 1)
                    .every(2_u32)
                    .expect("d3 test: a step of 1 or more"),
            );
            x.get_domain()
        })),
        tss(&[utc(&[2008.0, 11.0, 21.0]), utc(&[2009.0, 0.0, 4.0])])
    ); // d3-scale/test/utcTime-test.js:54:3
    assert_eq!(
        tss(&({
            x = x.nice_with(
                Interval::Month(1)
                    .every(3_u32)
                    .expect("d3 test: a step of 1 or more"),
            );
            x.get_domain()
        })),
        tss(&[utc(&[2008.0, 9.0, 1.0]), utc(&[2009.0, 3.0, 1.0])])
    ); // d3-scale/test/utcTime-test.js:55:3
    assert_eq!(
        tss(&({
            x = x.nice_with(
                Interval::Year(1)
                    .every(10_u32)
                    .expect("d3 test: a step of 1 or more"),
            );
            x.get_domain()
        })),
        tss(&[utc(&[2000.0, 0.0, 1.0]), utc(&[2010.0, 0.0, 1.0])])
    ); // d3-scale/test/utcTime-test.js:56:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.copy() isolates changes to the domain
#[test]
fn scaleutc_copy_isolates_changes_to_the_domain() {
    let mut x = time::utc().domain([utc(&[2009.0, 0.0, 1.0]), utc(&[2010.0, 0.0, 1.0])]);
    let mut y = x.clone();
    x = x.domain([utc(&[2010.0, 0.0, 1.0]), utc(&[2011.0, 0.0, 1.0])]);
    assert_eq!(
        tss(&y.get_domain()),
        tss(&[utc(&[2009.0, 0.0, 1.0]), utc(&[2010.0, 0.0, 1.0])])
    ); // d3-scale/test/utcTime-test.js:62:3
    assert_eq!(x.apply(utc(&[2010.0, 0.0, 1.0])), 0.0); // d3-scale/test/utcTime-test.js:63:3
    assert_eq!(y.apply(utc(&[2010.0, 0.0, 1.0])), 1.0); // d3-scale/test/utcTime-test.js:64:3
    y = y.domain([utc(&[2011.0, 0.0, 1.0]), utc(&[2012.0, 0.0, 1.0])]);
    assert_eq!(x.apply(utc(&[2011.0, 0.0, 1.0])), 1.0); // d3-scale/test/utcTime-test.js:66:3
    assert_eq!(y.apply(utc(&[2011.0, 0.0, 1.0])), 0.0); // d3-scale/test/utcTime-test.js:67:3
    assert_eq!(
        tss(&x.get_domain()),
        tss(&[utc(&[2010.0, 0.0, 1.0]), utc(&[2011.0, 0.0, 1.0])])
    ); // d3-scale/test/utcTime-test.js:68:3
    assert_eq!(
        tss(&y.get_domain()),
        tss(&[utc(&[2011.0, 0.0, 1.0]), utc(&[2012.0, 0.0, 1.0])])
    ); // d3-scale/test/utcTime-test.js:69:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.copy() isolates changes to the range
#[test]
fn scaleutc_copy_isolates_changes_to_the_range() {
    let mut x = time::utc().domain([utc(&[2009.0, 0.0, 1.0]), utc(&[2010.0, 0.0, 1.0])]);
    let mut y = x.clone();
    x = x.range([1.0, 2.0]);
    assert_eq!(ts(x.invert(1.0)), ts(utc(&[2009.0, 0.0, 1.0]))); // d3-scale/test/utcTime-test.js:75:3
    assert_eq!(ts(y.invert(1.0)), ts(utc(&[2010.0, 0.0, 1.0]))); // d3-scale/test/utcTime-test.js:76:3
    assert_eq!(y.get_range().to_vec(), vec![0.0, 1.0]); // d3-scale/test/utcTime-test.js:77:3
    y = y.range([2.0, 3.0]);
    assert_eq!(ts(x.invert(2.0)), ts(utc(&[2010.0, 0.0, 1.0]))); // d3-scale/test/utcTime-test.js:79:3
    assert_eq!(ts(y.invert(2.0)), ts(utc(&[2009.0, 0.0, 1.0]))); // d3-scale/test/utcTime-test.js:80:3
    assert_eq!(x.get_range().to_vec(), vec![1.0, 2.0]); // d3-scale/test/utcTime-test.js:81:3
    assert_eq!(y.get_range().to_vec(), vec![2.0, 3.0]); // d3-scale/test/utcTime-test.js:82:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.copy() isolates changes to clamping
#[test]
fn scaleutc_copy_isolates_changes_to_clamping() {
    let mut x = time::utc()
        .domain([utc(&[2009.0, 0.0, 1.0]), utc(&[2010.0, 0.0, 1.0])])
        .clamp(true);
    let mut y = x.clone();
    x = x.clamp(false);
    assert_eq!(x.apply(utc(&[2011.0, 0.0, 1.0])), 2.0); // d3-scale/test/utcTime-test.js:98:3
    assert_eq!(y.apply(utc(&[2011.0, 0.0, 1.0])), 1.0); // d3-scale/test/utcTime-test.js:99:3
    y = y.clamp(false);
    assert_eq!(x.apply(utc(&[2011.0, 0.0, 1.0])), 2.0); // d3-scale/test/utcTime-test.js:102:3
    assert_eq!(y.apply(utc(&[2011.0, 0.0, 1.0])), 2.0); // d3-scale/test/utcTime-test.js:103:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(interval) observes the specified tick interval
#[test]
fn scaleutc_ticks_interval_observes_the_specified_tick_interval() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 12.0, 1.0, 0.0]),
        utc(&[2011.0, 0.0, 1.0, 12.0, 4.0, 4.0]),
    ]);
    assert_eq!(
        tss(&x.ticks_with(Interval::Minute(1))),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 1.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 2.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 3.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 4.0])
        ])
    ); // d3-scale/test/utcTime-test.js:109:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(interval) observes the specified named tick interval
#[test]
fn scaleutc_ticks_interval_observes_the_specified_named_tick_interval() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 12.0, 1.0, 0.0]),
        utc(&[2011.0, 0.0, 1.0, 12.0, 4.0, 4.0]),
    ]);
    assert_eq!(
        tss(&x.ticks_with(Interval::Minute(1))),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 1.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 2.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 3.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 4.0])
        ])
    ); // d3-scale/test/utcTime-test.js:119:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(interval.every(step)) observes the specified tick interval and step
#[test]
fn scaleutc_ticks_interval_every_step_observes_the_specified_tick_interval_and_step() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
        utc(&[2011.0, 0.0, 1.0, 12.0, 33.0, 4.0]),
    ]);
    assert_eq!(
        tss(&x.ticks_with(
            Interval::Minute(1)
                .every(10_u32)
                .expect("d3 test: a step of 1 or more")
        )),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 10.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 20.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 30.0])
        ])
    ); // d3-scale/test/utcTime-test.js:129:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate sub-second ticks
#[test]
fn scaleutc_ticks_count_can_generate_sub_second_ticks() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
        utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 1.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 200.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 400.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 600.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0, 800.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 1.0, 0.0])
        ])
    ); // d3-scale/test/utcTime-test.js:139:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate 1-second ticks
#[test]
fn scaleutc_ticks_count_can_generate_1_second_ticks() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
        utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 4.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 1.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 2.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 3.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 4.0])
        ])
    ); // d3-scale/test/utcTime-test.js:151:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate 5-second ticks
#[test]
fn scaleutc_ticks_count_can_generate_5_second_ticks() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
        utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 20.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 5.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 10.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 15.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 20.0])
        ])
    ); // d3-scale/test/utcTime-test.js:162:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate 15-second ticks
#[test]
fn scaleutc_ticks_count_can_generate_15_second_ticks() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
        utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 50.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 15.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 30.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 45.0])
        ])
    ); // d3-scale/test/utcTime-test.js:173:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate 30-second ticks
#[test]
fn scaleutc_ticks_count_can_generate_30_second_ticks() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
        utc(&[2011.0, 0.0, 1.0, 12.0, 1.0, 50.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 30.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 1.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 1.0, 30.0])
        ])
    ); // d3-scale/test/utcTime-test.js:183:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate 1-minute ticks
#[test]
fn scaleutc_ticks_count_can_generate_1_minute_ticks() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 12.0, 0.0, 27.0]),
        utc(&[2011.0, 0.0, 1.0, 12.0, 4.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 1.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 2.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 3.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 4.0])
        ])
    ); // d3-scale/test/utcTime-test.js:193:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate 5-minute ticks
#[test]
fn scaleutc_ticks_count_can_generate_5_minute_ticks() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 12.0, 3.0, 27.0]),
        utc(&[2011.0, 0.0, 1.0, 12.0, 21.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 5.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 10.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 15.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 20.0])
        ])
    ); // d3-scale/test/utcTime-test.js:203:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate 15-minute ticks
#[test]
fn scaleutc_ticks_count_can_generate_15_minute_ticks() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 12.0, 8.0, 27.0]),
        utc(&[2011.0, 0.0, 1.0, 13.0, 4.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 15.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 30.0]),
            utc(&[2011.0, 0.0, 1.0, 12.0, 45.0]),
            utc(&[2011.0, 0.0, 1.0, 13.0, 0.0])
        ])
    ); // d3-scale/test/utcTime-test.js:213:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate 30-minute ticks
#[test]
fn scaleutc_ticks_count_can_generate_30_minute_ticks() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 12.0, 28.0, 27.0]),
        utc(&[2011.0, 0.0, 1.0, 14.0, 4.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 12.0, 30.0]),
            utc(&[2011.0, 0.0, 1.0, 13.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 13.0, 30.0]),
            utc(&[2011.0, 0.0, 1.0, 14.0, 0.0])
        ])
    ); // d3-scale/test/utcTime-test.js:223:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate 1-hour ticks
#[test]
fn scaleutc_ticks_count_can_generate_1_hour_ticks() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 12.0, 28.0, 27.0]),
        utc(&[2011.0, 0.0, 1.0, 16.0, 34.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 13.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 14.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 15.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 16.0, 0.0])
        ])
    ); // d3-scale/test/utcTime-test.js:233:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate 3-hour ticks
#[test]
fn scaleutc_ticks_count_can_generate_3_hour_ticks() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 14.0, 28.0, 27.0]),
        utc(&[2011.0, 0.0, 2.0, 1.0, 34.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 15.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 18.0, 0.0]),
            utc(&[2011.0, 0.0, 1.0, 21.0, 0.0]),
            utc(&[2011.0, 0.0, 2.0, 0.0, 0.0])
        ])
    ); // d3-scale/test/utcTime-test.js:243:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate 6-hour ticks
#[test]
fn scaleutc_ticks_count_can_generate_6_hour_ticks() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 16.0, 28.0, 27.0]),
        utc(&[2011.0, 0.0, 2.0, 14.0, 34.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 18.0, 0.0]),
            utc(&[2011.0, 0.0, 2.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 2.0, 6.0, 0.0]),
            utc(&[2011.0, 0.0, 2.0, 12.0, 0.0])
        ])
    ); // d3-scale/test/utcTime-test.js:253:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate 12-hour ticks
#[test]
fn scaleutc_ticks_count_can_generate_12_hour_ticks() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 16.0, 28.0, 27.0]),
        utc(&[2011.0, 0.0, 3.0, 21.0, 34.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 0.0, 2.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 2.0, 12.0, 0.0]),
            utc(&[2011.0, 0.0, 3.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 3.0, 12.0, 0.0])
        ])
    ); // d3-scale/test/utcTime-test.js:263:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate 1-day ticks
#[test]
fn scaleutc_ticks_count_can_generate_1_day_ticks() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 16.0, 28.0, 27.0]),
        utc(&[2011.0, 0.0, 5.0, 21.0, 34.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 0.0, 2.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 3.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 4.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 5.0, 0.0, 0.0])
        ])
    ); // d3-scale/test/utcTime-test.js:273:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate 1-week ticks
#[test]
fn scaleutc_ticks_count_can_generate_1_week_ticks() {
    let x = time::utc().domain([
        utc(&[2011.0, 0.0, 1.0, 16.0, 28.0, 27.0]),
        utc(&[2011.0, 0.0, 23.0, 21.0, 34.0, 12.0]),
    ]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 0.0, 2.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 9.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 16.0, 0.0, 0.0]),
            utc(&[2011.0, 0.0, 23.0, 0.0, 0.0])
        ])
    ); // d3-scale/test/utcTime-test.js:293:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate 1-month ticks
#[test]
fn scaleutc_ticks_count_can_generate_1_month_ticks() {
    let x = time::utc().domain([utc(&[2011.0, 0.0, 18.0]), utc(&[2011.0, 4.0, 2.0])]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 1.0, 1.0, 0.0, 0.0]),
            utc(&[2011.0, 2.0, 1.0, 0.0, 0.0]),
            utc(&[2011.0, 3.0, 1.0, 0.0, 0.0]),
            utc(&[2011.0, 4.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-scale/test/utcTime-test.js:303:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate 3-month ticks
#[test]
fn scaleutc_ticks_count_can_generate_3_month_ticks() {
    let x = time::utc().domain([utc(&[2010.0, 11.0, 18.0]), utc(&[2011.0, 10.0, 2.0])]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[2011.0, 3.0, 1.0, 0.0, 0.0]),
            utc(&[2011.0, 6.0, 1.0, 0.0, 0.0]),
            utc(&[2011.0, 9.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-scale/test/utcTime-test.js:313:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate 1-year ticks
#[test]
fn scaleutc_ticks_count_can_generate_1_year_ticks() {
    let x = time::utc().domain([utc(&[2010.0, 11.0, 18.0]), utc(&[2014.0, 2.0, 2.0])]);
    assert_eq!(
        tss(&x.ticks(4_usize)),
        tss(&[
            utc(&[2011.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[2012.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[2013.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[2014.0, 0.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-scale/test/utcTime-test.js:323:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) can generate multi-year ticks
#[test]
fn scaleutc_ticks_count_can_generate_multi_year_ticks() {
    let x = time::utc().domain([utc(&[0.0, 11.0, 18.0]), utc(&[2014.0, 2.0, 2.0])]);
    assert_eq!(
        tss(&x.ticks(6_usize)),
        tss(&[
            utc(&[500.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[1000.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[1500.0, 0.0, 1.0, 0.0, 0.0]),
            utc(&[2000.0, 0.0, 1.0, 0.0, 0.0])
        ])
    ); // d3-scale/test/utcTime-test.js:333:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.ticks(count) returns one tick for an empty domain
#[test]
fn scaleutc_ticks_count_returns_one_tick_for_an_empty_domain() {
    let x = time::utc().domain([utc(&[2014.0, 2.0, 2.0]), utc(&[2014.0, 2.0, 2.0])]);
    assert_eq!(tss(&x.ticks(6_usize)), tss(&[utc(&[2014.0, 2.0, 2.0])])); // d3-scale/test/utcTime-test.js:343:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.tickFormat()(date) formats year on New Year's
#[test]
fn scaleutc_tickformat_date_formats_year_on_new_year_s() {
    let f = time::utc();
    assert_eq!(
        f.tick_label(utc(&[2011.0, 0.0, 1.0]), &time::Locale::EN_US),
        "2011"
    ); // d3-scale/test/utcTime-test.js:348:3
    assert_eq!(
        f.tick_label(utc(&[2012.0, 0.0, 1.0]), &time::Locale::EN_US),
        "2012"
    ); // d3-scale/test/utcTime-test.js:349:3
    assert_eq!(
        f.tick_label(utc(&[2013.0, 0.0, 1.0]), &time::Locale::EN_US),
        "2013"
    ); // d3-scale/test/utcTime-test.js:350:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.tickFormat()(date) formats month on the 1st of each month
#[test]
fn scaleutc_tickformat_date_formats_month_on_the_1st_of_each_month() {
    let f = time::utc();
    assert_eq!(
        f.tick_label(utc(&[2011.0, 1.0, 1.0]), &time::Locale::EN_US),
        "February"
    ); // d3-scale/test/utcTime-test.js:355:3
    assert_eq!(
        f.tick_label(utc(&[2011.0, 2.0, 1.0]), &time::Locale::EN_US),
        "March"
    ); // d3-scale/test/utcTime-test.js:356:3
    assert_eq!(
        f.tick_label(utc(&[2011.0, 3.0, 1.0]), &time::Locale::EN_US),
        "April"
    ); // d3-scale/test/utcTime-test.js:357:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.tickFormat()(date) formats week on Sunday midnight
#[test]
fn scaleutc_tickformat_date_formats_week_on_sunday_midnight() {
    let f = time::utc();
    assert_eq!(
        f.tick_label(utc(&[2011.0, 1.0, 6.0]), &time::Locale::EN_US),
        "Feb 06"
    ); // d3-scale/test/utcTime-test.js:362:3
    assert_eq!(
        f.tick_label(utc(&[2011.0, 1.0, 13.0]), &time::Locale::EN_US),
        "Feb 13"
    ); // d3-scale/test/utcTime-test.js:363:3
    assert_eq!(
        f.tick_label(utc(&[2011.0, 1.0, 20.0]), &time::Locale::EN_US),
        "Feb 20"
    ); // d3-scale/test/utcTime-test.js:364:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.tickFormat()(date) formats date on midnight
#[test]
fn scaleutc_tickformat_date_formats_date_on_midnight() {
    let f = time::utc();
    assert_eq!(
        f.tick_label(utc(&[2011.0, 1.0, 2.0]), &time::Locale::EN_US),
        "Wed 02"
    ); // d3-scale/test/utcTime-test.js:369:3
    assert_eq!(
        f.tick_label(utc(&[2011.0, 1.0, 3.0]), &time::Locale::EN_US),
        "Thu 03"
    ); // d3-scale/test/utcTime-test.js:370:3
    assert_eq!(
        f.tick_label(utc(&[2011.0, 1.0, 4.0]), &time::Locale::EN_US),
        "Fri 04"
    ); // d3-scale/test/utcTime-test.js:371:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.tickFormat()(date) formats hour on minute zero
#[test]
fn scaleutc_tickformat_date_formats_hour_on_minute_zero() {
    let f = time::utc();
    assert_eq!(
        f.tick_label(utc(&[2011.0, 1.0, 2.0, 11.0]), &time::Locale::EN_US),
        "11 AM"
    ); // d3-scale/test/utcTime-test.js:376:3
    assert_eq!(
        f.tick_label(utc(&[2011.0, 1.0, 2.0, 12.0]), &time::Locale::EN_US),
        "12 PM"
    ); // d3-scale/test/utcTime-test.js:377:3
    assert_eq!(
        f.tick_label(utc(&[2011.0, 1.0, 2.0, 13.0]), &time::Locale::EN_US),
        "01 PM"
    ); // d3-scale/test/utcTime-test.js:378:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.tickFormat()(date) formats minute on second zero
#[test]
fn scaleutc_tickformat_date_formats_minute_on_second_zero() {
    let f = time::utc();
    assert_eq!(
        f.tick_label(utc(&[2011.0, 1.0, 2.0, 11.0, 59.0]), &time::Locale::EN_US),
        "11:59"
    ); // d3-scale/test/utcTime-test.js:383:3
    assert_eq!(
        f.tick_label(utc(&[2011.0, 1.0, 2.0, 12.0, 1.0]), &time::Locale::EN_US),
        "12:01"
    ); // d3-scale/test/utcTime-test.js:384:3
    assert_eq!(
        f.tick_label(utc(&[2011.0, 1.0, 2.0, 12.0, 2.0]), &time::Locale::EN_US),
        "12:02"
    ); // d3-scale/test/utcTime-test.js:385:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.tickFormat()(date) otherwise, formats second
#[test]
fn scaleutc_tickformat_date_otherwise_formats_second() {
    let f = time::utc();
    assert_eq!(
        f.tick_label(
            utc(&[2011.0, 1.0, 2.0, 12.0, 1.0, 9.0]),
            &time::Locale::EN_US
        ),
        ":09"
    ); // d3-scale/test/utcTime-test.js:390:3
    assert_eq!(
        f.tick_label(
            utc(&[2011.0, 1.0, 2.0, 12.0, 1.0, 10.0]),
            &time::Locale::EN_US
        ),
        ":10"
    ); // d3-scale/test/utcTime-test.js:391:3
    assert_eq!(
        f.tick_label(
            utc(&[2011.0, 1.0, 2.0, 12.0, 1.0, 11.0]),
            &time::Locale::EN_US
        ),
        ":11"
    ); // d3-scale/test/utcTime-test.js:392:3
}

// d3-scale/test/utcTime-test.js: scaleUtc.tickFormat(count, specifier) returns a time format for the specified specifier
#[test]
fn scaleutc_tickformat_count_specifier_returns_a_time_format_for_the_specified_specifier() {
    let f = time::Locale::EN_US.format("%c");
    assert_eq!(
        f.format(utc(&[2011.0, 1.0, 2.0, 12.0]), &TimeZone::UTC),
        "2/2/2011, 12:00:00 PM"
    ); // d3-scale/test/utcTime-test.js:397:3
}
