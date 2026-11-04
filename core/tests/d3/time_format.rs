//! d3's tests of d3-time-format, as assertions on rustd3-core; each names d3's file, line and column.

use rustd3_core::jiff::tz::TimeZone;
use rustd3_core::time;

use super::support::{local, millis, ts, utc, zone};

// d3-time-format/test/format-test.js: timeFormat("%a")(date) formats abbreviated weekdays
#[test]
fn timeformat_a_date_formats_abbreviated_weekdays() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%a");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0]), &la), "Mon"); // d3-time-format/test/format-test.js:38:3
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 2.0]), &la), "Tue"); // d3-time-format/test/format-test.js:39:3
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 3.0]), &la), "Wed"); // d3-time-format/test/format-test.js:40:3
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 4.0]), &la), "Thu"); // d3-time-format/test/format-test.js:41:3
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 5.0]), &la), "Fri"); // d3-time-format/test/format-test.js:42:3
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 6.0]), &la), "Sat"); // d3-time-format/test/format-test.js:43:3
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 7.0]), &la), "Sun"); // d3-time-format/test/format-test.js:44:3
}

// d3-time-format/test/format-test.js: timeFormat("%A")(date) formats weekdays
#[test]
fn timeformat_a_date_formats_weekdays() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%A");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0]), &la), "Monday"); // d3-time-format/test/format-test.js:49:3
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 2.0]), &la), "Tuesday"); // d3-time-format/test/format-test.js:50:3
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 3.0]), &la), "Wednesday"); // d3-time-format/test/format-test.js:51:3
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 4.0]), &la), "Thursday"); // d3-time-format/test/format-test.js:52:3
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 5.0]), &la), "Friday"); // d3-time-format/test/format-test.js:53:3
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 6.0]), &la), "Saturday"); // d3-time-format/test/format-test.js:54:3
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 7.0]), &la), "Sunday"); // d3-time-format/test/format-test.js:55:3
}

// d3-time-format/test/format-test.js: timeFormat("%b")(date) formats abbreviated months
#[test]
fn timeformat_b_date_formats_abbreviated_months() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%b");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0]), &la), "Jan"); // d3-time-format/test/format-test.js:60:3
    assert_eq!(f.format(local(&la, &[1990.0, 1.0, 1.0]), &la), "Feb"); // d3-time-format/test/format-test.js:61:3
    assert_eq!(f.format(local(&la, &[1990.0, 2.0, 1.0]), &la), "Mar"); // d3-time-format/test/format-test.js:62:3
    assert_eq!(f.format(local(&la, &[1990.0, 3.0, 1.0]), &la), "Apr"); // d3-time-format/test/format-test.js:63:3
    assert_eq!(f.format(local(&la, &[1990.0, 4.0, 1.0]), &la), "May"); // d3-time-format/test/format-test.js:64:3
    assert_eq!(f.format(local(&la, &[1990.0, 5.0, 1.0]), &la), "Jun"); // d3-time-format/test/format-test.js:65:3
    assert_eq!(f.format(local(&la, &[1990.0, 6.0, 1.0]), &la), "Jul"); // d3-time-format/test/format-test.js:66:3
    assert_eq!(f.format(local(&la, &[1990.0, 7.0, 1.0]), &la), "Aug"); // d3-time-format/test/format-test.js:67:3
    assert_eq!(f.format(local(&la, &[1990.0, 8.0, 1.0]), &la), "Sep"); // d3-time-format/test/format-test.js:68:3
    assert_eq!(f.format(local(&la, &[1990.0, 9.0, 1.0]), &la), "Oct"); // d3-time-format/test/format-test.js:69:3
    assert_eq!(f.format(local(&la, &[1990.0, 10.0, 1.0]), &la), "Nov"); // d3-time-format/test/format-test.js:70:3
    assert_eq!(f.format(local(&la, &[1990.0, 11.0, 1.0]), &la), "Dec"); // d3-time-format/test/format-test.js:71:3
}

// d3-time-format/test/format-test.js: timeFormat("%B")(date) formats months
#[test]
fn timeformat_b_date_formats_months() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%B");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0]), &la), "January"); // d3-time-format/test/format-test.js:76:3
    assert_eq!(f.format(local(&la, &[1990.0, 1.0, 1.0]), &la), "February"); // d3-time-format/test/format-test.js:77:3
    assert_eq!(f.format(local(&la, &[1990.0, 2.0, 1.0]), &la), "March"); // d3-time-format/test/format-test.js:78:3
    assert_eq!(f.format(local(&la, &[1990.0, 3.0, 1.0]), &la), "April"); // d3-time-format/test/format-test.js:79:3
    assert_eq!(f.format(local(&la, &[1990.0, 4.0, 1.0]), &la), "May"); // d3-time-format/test/format-test.js:80:3
    assert_eq!(f.format(local(&la, &[1990.0, 5.0, 1.0]), &la), "June"); // d3-time-format/test/format-test.js:81:3
    assert_eq!(f.format(local(&la, &[1990.0, 6.0, 1.0]), &la), "July"); // d3-time-format/test/format-test.js:82:3
    assert_eq!(f.format(local(&la, &[1990.0, 7.0, 1.0]), &la), "August"); // d3-time-format/test/format-test.js:83:3
    assert_eq!(f.format(local(&la, &[1990.0, 8.0, 1.0]), &la), "September"); // d3-time-format/test/format-test.js:84:3
    assert_eq!(f.format(local(&la, &[1990.0, 9.0, 1.0]), &la), "October"); // d3-time-format/test/format-test.js:85:3
    assert_eq!(f.format(local(&la, &[1990.0, 10.0, 1.0]), &la), "November"); // d3-time-format/test/format-test.js:86:3
    assert_eq!(f.format(local(&la, &[1990.0, 11.0, 1.0]), &la), "December"); // d3-time-format/test/format-test.js:87:3
}

// d3-time-format/test/format-test.js: timeFormat("%c")(date) formats localized dates and times
#[test]
fn timeformat_c_date_formats_localized_dates_and_times() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%c");
    assert_eq!(
        f.format(local(&la, &[1990.0, 0.0, 1.0]), &la),
        "1/1/1990, 12:00:00 AM"
    ); // d3-time-format/test/format-test.js:92:3
}

// d3-time-format/test/format-test.js: timeFormat("%d")(date) formats zero-padded dates
#[test]
fn timeformat_d_date_formats_zero_padded_dates() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%d");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0]), &la), "01"); // d3-time-format/test/format-test.js:97:3
}

// d3-time-format/test/format-test.js: timeFormat("%e")(date) formats space-padded dates
#[test]
fn timeformat_e_date_formats_space_padded_dates() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%e");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0]), &la), " 1"); // d3-time-format/test/format-test.js:102:3
}

// d3-time-format/test/format-test.js: timeFormat("%g")(date) formats zero-padded two-digit ISO 8601 years
#[test]
fn timeformat_g_date_formats_zero_padded_two_digit_iso_8601_years() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%g");
    assert_eq!(f.format(local(&la, &[2018.0, 11.0, 30.0, 0.0]), &la), "18"); // d3-time-format/test/format-test.js:107:3
    assert_eq!(f.format(local(&la, &[2018.0, 11.0, 31.0, 0.0]), &la), "19"); // d3-time-format/test/format-test.js:108:3
    assert_eq!(f.format(local(&la, &[2019.0, 0.0, 1.0, 0.0]), &la), "19"); // d3-time-format/test/format-test.js:109:3
}

// d3-time-format/test/format-test.js: timeFormat("%G")(date) formats zero-padded four-digit ISO 8601 years
#[test]
fn timeformat_g_date_formats_zero_padded_four_digit_iso_8601_years() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%G");
    assert_eq!(
        f.format(local(&la, &[2018.0, 11.0, 30.0, 0.0]), &la),
        "2018"
    ); // d3-time-format/test/format-test.js:114:3
    assert_eq!(
        f.format(local(&la, &[2018.0, 11.0, 31.0, 0.0]), &la),
        "2019"
    ); // d3-time-format/test/format-test.js:115:3
    assert_eq!(f.format(local(&la, &[2019.0, 0.0, 1.0, 0.0]), &la), "2019"); // d3-time-format/test/format-test.js:116:3
}

// d3-time-format/test/format-test.js: timeFormat("%H")(date) formats zero-padded hours (24)
#[test]
fn timeformat_h_date_formats_zero_padded_hours_24() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%H");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0]), &la), "00"); // d3-time-format/test/format-test.js:121:3
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0, 13.0]), &la), "13"); // d3-time-format/test/format-test.js:122:3
}

// d3-time-format/test/format-test.js: timeFormat("%I")(date) formats zero-padded hours (12)
#[test]
fn timeformat_i_date_formats_zero_padded_hours_12() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%I");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0]), &la), "12"); // d3-time-format/test/format-test.js:127:3
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0, 13.0]), &la), "01"); // d3-time-format/test/format-test.js:128:3
}

// d3-time-format/test/format-test.js: timeFormat("%j")(date) formats zero-padded day of year numbers
#[test]
fn timeformat_j_date_formats_zero_padded_day_of_year_numbers() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%j");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0]), &la), "001"); // d3-time-format/test/format-test.js:133:3
    assert_eq!(f.format(local(&la, &[1990.0, 5.0, 1.0]), &la), "152"); // d3-time-format/test/format-test.js:134:3
    assert_eq!(f.format(local(&la, &[2010.0, 2.0, 13.0]), &la), "072"); // d3-time-format/test/format-test.js:135:3
    assert_eq!(f.format(local(&la, &[2010.0, 2.0, 14.0]), &la), "073"); // d3-time-format/test/format-test.js:136:3
    assert_eq!(f.format(local(&la, &[2010.0, 2.0, 15.0]), &la), "074"); // d3-time-format/test/format-test.js:137:3
    assert_eq!(f.format(local(&la, &[2010.0, 10.0, 6.0]), &la), "310"); // d3-time-format/test/format-test.js:138:3
    assert_eq!(f.format(local(&la, &[2010.0, 10.0, 7.0]), &la), "311"); // d3-time-format/test/format-test.js:139:3
    assert_eq!(f.format(local(&la, &[2010.0, 10.0, 8.0]), &la), "312"); // d3-time-format/test/format-test.js:140:3
}

// d3-time-format/test/format-test.js: timeFormat("%m")(date) formats zero-padded months
#[test]
fn timeformat_m_date_formats_zero_padded_months() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%m");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0]), &la), "01"); // d3-time-format/test/format-test.js:145:3
    assert_eq!(f.format(local(&la, &[1990.0, 9.0, 1.0]), &la), "10"); // d3-time-format/test/format-test.js:146:3
}

// d3-time-format/test/format-test.js: timeFormat("%M")(date) formats zero-padded minutes
#[test]
fn timeformat_m_date_formats_zero_padded_minutes() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%M");
    assert_eq!(
        f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0, 0.0]), &la),
        "00"
    ); // d3-time-format/test/format-test.js:151:3
    assert_eq!(
        f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0, 32.0]), &la),
        "32"
    ); // d3-time-format/test/format-test.js:152:3
}

// d3-time-format/test/format-test.js: timeFormat("%p")(date) formats AM or PM
#[test]
fn timeformat_p_date_formats_am_or_pm() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%p");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0]), &la), "AM"); // d3-time-format/test/format-test.js:157:3
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0, 13.0]), &la), "PM"); // d3-time-format/test/format-test.js:158:3
}

// d3-time-format/test/format-test.js: timeFormat("%q")(date) formats quarters
#[test]
fn timeformat_q_date_formats_quarters() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%q");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0]), &la), "1"); // d3-time-format/test/format-test.js:163:3
    assert_eq!(f.format(local(&la, &[1990.0, 3.0, 1.0]), &la), "2"); // d3-time-format/test/format-test.js:164:3
    assert_eq!(f.format(local(&la, &[1990.0, 6.0, 1.0]), &la), "3"); // d3-time-format/test/format-test.js:165:3
    assert_eq!(f.format(local(&la, &[1990.0, 9.0, 1.0]), &la), "4"); // d3-time-format/test/format-test.js:166:3
}

// d3-time-format/test/format-test.js: timeFormat("%S")(date) formats zero-padded seconds
#[test]
fn timeformat_s_date_formats_zero_padded_seconds() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%S");
    assert_eq!(
        f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &la),
        "00"
    ); // d3-time-format/test/format-test.js:171:3
    assert_eq!(
        f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0, 0.0, 32.0]), &la),
        "32"
    ); // d3-time-format/test/format-test.js:172:3
    let f2 = time::Locale::EN_US.format("%0S");
    assert_eq!(
        f2.format(local(&la, &[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &la),
        "00"
    ); // d3-time-format/test/format-test.js:174:3
    assert_eq!(
        f2.format(local(&la, &[1990.0, 0.0, 1.0, 0.0, 0.0, 32.0]), &la),
        "32"
    ); // d3-time-format/test/format-test.js:175:3
}

// d3-time-format/test/format-test.js: timeFormat("%_S")(date) formats space-padded seconds
#[test]
fn timeformat_s_date_formats_space_padded_seconds() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%_S");
    assert_eq!(
        f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &la),
        " 0"
    ); // d3-time-format/test/format-test.js:180:3
    assert_eq!(
        f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0, 0.0, 3.0]), &la),
        " 3"
    ); // d3-time-format/test/format-test.js:181:3
    assert_eq!(
        f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0, 0.0, 32.0]), &la),
        "32"
    ); // d3-time-format/test/format-test.js:182:3
}

// d3-time-format/test/format-test.js: timeFormat("-S")(date) formats no-padded seconds
#[test]
fn timeformat_s_date_formats_no_padded_seconds() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%-S");
    assert_eq!(
        f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &la),
        "0"
    ); // d3-time-format/test/format-test.js:187:3
    assert_eq!(
        f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0, 0.0, 3.0]), &la),
        "3"
    ); // d3-time-format/test/format-test.js:188:3
    assert_eq!(
        f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0, 0.0, 32.0]), &la),
        "32"
    ); // d3-time-format/test/format-test.js:189:3
}

// d3-time-format/test/format-test.js: timeFormat("%L")(date) formats zero-padded milliseconds
#[test]
fn timeformat_l_date_formats_zero_padded_milliseconds() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%L");
    assert_eq!(
        f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]), &la),
        "000"
    ); // d3-time-format/test/format-test.js:194:3
    assert_eq!(
        f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0, 432.0]), &la),
        "432"
    ); // d3-time-format/test/format-test.js:195:3
}

// d3-time-format/test/format-test.js: timeFormat("%u")(date) formats week day numbers
#[test]
fn timeformat_u_date_formats_week_day_numbers() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%u");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0]), &la), "1"); // d3-time-format/test/format-test.js:200:3
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 7.0, 0.0]), &la), "7"); // d3-time-format/test/format-test.js:201:3
    assert_eq!(f.format(local(&la, &[2010.0, 2.0, 13.0, 23.0]), &la), "6"); // d3-time-format/test/format-test.js:202:3
}

// d3-time-format/test/format-test.js: timeFormat("%f")(date) formats zero-padded microseconds
#[test]
fn timeformat_f_date_formats_zero_padded_microseconds() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%f");
    assert_eq!(
        f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]), &la),
        "000000"
    ); // d3-time-format/test/format-test.js:207:3
    assert_eq!(
        f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0, 432.0]), &la),
        "432000"
    ); // d3-time-format/test/format-test.js:208:3
}

// d3-time-format/test/format-test.js: timeFormat("%U")(date) formats zero-padded week numbers
#[test]
fn timeformat_u_date_formats_zero_padded_week_numbers() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%U");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0]), &la), "00"); // d3-time-format/test/format-test.js:213:3
    assert_eq!(f.format(local(&la, &[1990.0, 5.0, 1.0, 0.0]), &la), "21"); // d3-time-format/test/format-test.js:214:3
    assert_eq!(f.format(local(&la, &[2010.0, 2.0, 13.0, 23.0]), &la), "10"); // d3-time-format/test/format-test.js:215:3
    assert_eq!(f.format(local(&la, &[2010.0, 2.0, 14.0, 0.0]), &la), "11"); // d3-time-format/test/format-test.js:216:3
    assert_eq!(f.format(local(&la, &[2010.0, 2.0, 15.0, 0.0]), &la), "11"); // d3-time-format/test/format-test.js:217:3
    assert_eq!(f.format(local(&la, &[2010.0, 10.0, 6.0, 23.0]), &la), "44"); // d3-time-format/test/format-test.js:218:3
    assert_eq!(f.format(local(&la, &[2010.0, 10.0, 7.0, 0.0]), &la), "45"); // d3-time-format/test/format-test.js:219:3
    assert_eq!(f.format(local(&la, &[2010.0, 10.0, 8.0, 0.0]), &la), "45"); // d3-time-format/test/format-test.js:220:3
    assert_eq!(f.format(local(&la, &[2012.0, 0.0, 1.0, 0.0]), &la), "01"); // d3-time-format/test/format-test.js:221:3
}

// d3-time-format/test/format-test.js: timeFormat("%W")(date) formats zero-padded week numbers
#[test]
fn timeformat_w_date_formats_zero_padded_week_numbers() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%W");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0]), &la), "01"); // d3-time-format/test/format-test.js:226:3
    assert_eq!(f.format(local(&la, &[1990.0, 5.0, 1.0, 0.0]), &la), "22"); // d3-time-format/test/format-test.js:227:3
    assert_eq!(f.format(local(&la, &[2010.0, 2.0, 15.0, 0.0]), &la), "11"); // d3-time-format/test/format-test.js:228:3
    assert_eq!(f.format(local(&la, &[2010.0, 10.0, 8.0, 0.0]), &la), "45"); // d3-time-format/test/format-test.js:229:3
}

// d3-time-format/test/format-test.js: timeFormat("%V")(date) formats zero-padded ISO 8601 week numbers
#[test]
fn timeformat_v_date_formats_zero_padded_iso_8601_week_numbers() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%V");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0]), &la), "01"); // d3-time-format/test/format-test.js:234:3
    assert_eq!(f.format(local(&la, &[1990.0, 5.0, 1.0, 0.0]), &la), "22"); // d3-time-format/test/format-test.js:235:3
    assert_eq!(f.format(local(&la, &[2010.0, 2.0, 13.0, 23.0]), &la), "10"); // d3-time-format/test/format-test.js:236:3
    assert_eq!(f.format(local(&la, &[2010.0, 2.0, 14.0, 0.0]), &la), "10"); // d3-time-format/test/format-test.js:237:3
    assert_eq!(f.format(local(&la, &[2010.0, 2.0, 15.0, 0.0]), &la), "11"); // d3-time-format/test/format-test.js:238:3
    assert_eq!(f.format(local(&la, &[2010.0, 10.0, 6.0, 23.0]), &la), "44"); // d3-time-format/test/format-test.js:239:3
    assert_eq!(f.format(local(&la, &[2010.0, 10.0, 7.0, 0.0]), &la), "44"); // d3-time-format/test/format-test.js:240:3
    assert_eq!(f.format(local(&la, &[2010.0, 10.0, 8.0, 0.0]), &la), "45"); // d3-time-format/test/format-test.js:241:3
    assert_eq!(f.format(local(&la, &[2015.0, 11.0, 31.0, 0.0]), &la), "53"); // d3-time-format/test/format-test.js:242:3
    assert_eq!(f.format(local(&la, &[2016.0, 0.0, 1.0, 0.0]), &la), "53"); // d3-time-format/test/format-test.js:243:3
}

// d3-time-format/test/format-test.js: timeFormat("%x")(date) formats localized dates
#[test]
fn timeformat_x_date_formats_localized_dates() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%x");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0]), &la), "1/1/1990"); // d3-time-format/test/format-test.js:248:3
    assert_eq!(f.format(local(&la, &[2010.0, 5.0, 1.0]), &la), "6/1/2010"); // d3-time-format/test/format-test.js:249:3
}

// d3-time-format/test/format-test.js: timeFormat("%X")(date) formats localized times
#[test]
fn timeformat_x_date_formats_localized_times() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%X");
    assert_eq!(
        f.format(local(&la, &[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &la),
        "12:00:00 AM"
    ); // d3-time-format/test/format-test.js:254:3
    assert_eq!(
        f.format(local(&la, &[1990.0, 0.0, 1.0, 13.0, 34.0, 59.0]), &la),
        "1:34:59 PM"
    ); // d3-time-format/test/format-test.js:255:3
}

// d3-time-format/test/format-test.js: timeFormat("%y")(date) formats zero-padded two-digit years
#[test]
fn timeformat_y_date_formats_zero_padded_two_digit_years() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%y");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0]), &la), "90"); // d3-time-format/test/format-test.js:260:3
    assert_eq!(f.format(local(&la, &[2002.0, 0.0, 1.0]), &la), "02"); // d3-time-format/test/format-test.js:261:3
    assert_eq!(f.format(local(&la, &[-2.0, 0.0, 1.0]), &la), "-02"); // d3-time-format/test/format-test.js:262:3
}

// d3-time-format/test/format-test.js: timeFormat("%Y")(date) formats zero-padded four-digit years
#[test]
fn timeformat_y_date_formats_zero_padded_four_digit_years() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%Y");
    assert_eq!(f.format(local(&la, &[123.0, 0.0, 1.0]), &la), "0123"); // d3-time-format/test/format-test.js:267:3
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0]), &la), "1990"); // d3-time-format/test/format-test.js:268:3
    assert_eq!(f.format(local(&la, &[2002.0, 0.0, 1.0]), &la), "2002"); // d3-time-format/test/format-test.js:269:3
    assert_eq!(f.format(local(&la, &[-2.0, 0.0, 1.0]), &la), "-0002"); // d3-time-format/test/format-test.js:271:3
}

// d3-time-format/test/format-test.js: timeFormat("%Z")(date) formats time zones
#[test]
fn timeformat_z_date_formats_time_zones() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%Z");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0]), &la), "-0800"); // d3-time-format/test/format-test.js:276:3
}

// d3-time-format/test/format-test.js: timeFormat("%%")(date) formats literal percent signs
#[test]
fn timeformat_date_formats_literal_percent_signs() {
    let la = zone("America/Los_Angeles");
    let f = time::Locale::EN_US.format("%%");
    assert_eq!(f.format(local(&la, &[1990.0, 0.0, 1.0]), &la), "%"); // d3-time-format/test/format-test.js:281:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%a")(date) formats abbreviated weekdays
#[test]
fn utcformat_a_date_formats_abbreviated_weekdays() {
    let f = time::Locale::EN_US.format("%a");
    assert_eq!(f.format(utc(&[1990.0, 0.0, 1.0]), &TimeZone::UTC), "Mon"); // d3-time-format/test/utcFormat-test.js:27:3
    assert_eq!(f.format(utc(&[1990.0, 0.0, 2.0]), &TimeZone::UTC), "Tue"); // d3-time-format/test/utcFormat-test.js:28:3
    assert_eq!(f.format(utc(&[1990.0, 0.0, 3.0]), &TimeZone::UTC), "Wed"); // d3-time-format/test/utcFormat-test.js:29:3
    assert_eq!(f.format(utc(&[1990.0, 0.0, 4.0]), &TimeZone::UTC), "Thu"); // d3-time-format/test/utcFormat-test.js:30:3
    assert_eq!(f.format(utc(&[1990.0, 0.0, 5.0]), &TimeZone::UTC), "Fri"); // d3-time-format/test/utcFormat-test.js:31:3
    assert_eq!(f.format(utc(&[1990.0, 0.0, 6.0]), &TimeZone::UTC), "Sat"); // d3-time-format/test/utcFormat-test.js:32:3
    assert_eq!(f.format(utc(&[1990.0, 0.0, 7.0]), &TimeZone::UTC), "Sun"); // d3-time-format/test/utcFormat-test.js:33:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%A")(date) formats weekdays
#[test]
fn utcformat_a_date_formats_weekdays() {
    let f = time::Locale::EN_US.format("%A");
    assert_eq!(f.format(utc(&[1990.0, 0.0, 1.0]), &TimeZone::UTC), "Monday"); // d3-time-format/test/utcFormat-test.js:38:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 2.0]), &TimeZone::UTC),
        "Tuesday"
    ); // d3-time-format/test/utcFormat-test.js:39:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 3.0]), &TimeZone::UTC),
        "Wednesday"
    ); // d3-time-format/test/utcFormat-test.js:40:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 4.0]), &TimeZone::UTC),
        "Thursday"
    ); // d3-time-format/test/utcFormat-test.js:41:3
    assert_eq!(f.format(utc(&[1990.0, 0.0, 5.0]), &TimeZone::UTC), "Friday"); // d3-time-format/test/utcFormat-test.js:42:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 6.0]), &TimeZone::UTC),
        "Saturday"
    ); // d3-time-format/test/utcFormat-test.js:43:3
    assert_eq!(f.format(utc(&[1990.0, 0.0, 7.0]), &TimeZone::UTC), "Sunday"); // d3-time-format/test/utcFormat-test.js:44:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%b")(date) formats abbreviated months
#[test]
fn utcformat_b_date_formats_abbreviated_months() {
    let f = time::Locale::EN_US.format("%b");
    assert_eq!(f.format(utc(&[1990.0, 0.0, 1.0]), &TimeZone::UTC), "Jan"); // d3-time-format/test/utcFormat-test.js:49:3
    assert_eq!(f.format(utc(&[1990.0, 1.0, 1.0]), &TimeZone::UTC), "Feb"); // d3-time-format/test/utcFormat-test.js:50:3
    assert_eq!(f.format(utc(&[1990.0, 2.0, 1.0]), &TimeZone::UTC), "Mar"); // d3-time-format/test/utcFormat-test.js:51:3
    assert_eq!(f.format(utc(&[1990.0, 3.0, 1.0]), &TimeZone::UTC), "Apr"); // d3-time-format/test/utcFormat-test.js:52:3
    assert_eq!(f.format(utc(&[1990.0, 4.0, 1.0]), &TimeZone::UTC), "May"); // d3-time-format/test/utcFormat-test.js:53:3
    assert_eq!(f.format(utc(&[1990.0, 5.0, 1.0]), &TimeZone::UTC), "Jun"); // d3-time-format/test/utcFormat-test.js:54:3
    assert_eq!(f.format(utc(&[1990.0, 6.0, 1.0]), &TimeZone::UTC), "Jul"); // d3-time-format/test/utcFormat-test.js:55:3
    assert_eq!(f.format(utc(&[1990.0, 7.0, 1.0]), &TimeZone::UTC), "Aug"); // d3-time-format/test/utcFormat-test.js:56:3
    assert_eq!(f.format(utc(&[1990.0, 8.0, 1.0]), &TimeZone::UTC), "Sep"); // d3-time-format/test/utcFormat-test.js:57:3
    assert_eq!(f.format(utc(&[1990.0, 9.0, 1.0]), &TimeZone::UTC), "Oct"); // d3-time-format/test/utcFormat-test.js:58:3
    assert_eq!(f.format(utc(&[1990.0, 10.0, 1.0]), &TimeZone::UTC), "Nov"); // d3-time-format/test/utcFormat-test.js:59:3
    assert_eq!(f.format(utc(&[1990.0, 11.0, 1.0]), &TimeZone::UTC), "Dec"); // d3-time-format/test/utcFormat-test.js:60:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%B")(date) formats months
#[test]
fn utcformat_b_date_formats_months() {
    let f = time::Locale::EN_US.format("%B");
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0]), &TimeZone::UTC),
        "January"
    ); // d3-time-format/test/utcFormat-test.js:65:3
    assert_eq!(
        f.format(utc(&[1990.0, 1.0, 1.0]), &TimeZone::UTC),
        "February"
    ); // d3-time-format/test/utcFormat-test.js:66:3
    assert_eq!(f.format(utc(&[1990.0, 2.0, 1.0]), &TimeZone::UTC), "March"); // d3-time-format/test/utcFormat-test.js:67:3
    assert_eq!(f.format(utc(&[1990.0, 3.0, 1.0]), &TimeZone::UTC), "April"); // d3-time-format/test/utcFormat-test.js:68:3
    assert_eq!(f.format(utc(&[1990.0, 4.0, 1.0]), &TimeZone::UTC), "May"); // d3-time-format/test/utcFormat-test.js:69:3
    assert_eq!(f.format(utc(&[1990.0, 5.0, 1.0]), &TimeZone::UTC), "June"); // d3-time-format/test/utcFormat-test.js:70:3
    assert_eq!(f.format(utc(&[1990.0, 6.0, 1.0]), &TimeZone::UTC), "July"); // d3-time-format/test/utcFormat-test.js:71:3
    assert_eq!(f.format(utc(&[1990.0, 7.0, 1.0]), &TimeZone::UTC), "August"); // d3-time-format/test/utcFormat-test.js:72:3
    assert_eq!(
        f.format(utc(&[1990.0, 8.0, 1.0]), &TimeZone::UTC),
        "September"
    ); // d3-time-format/test/utcFormat-test.js:73:3
    assert_eq!(
        f.format(utc(&[1990.0, 9.0, 1.0]), &TimeZone::UTC),
        "October"
    ); // d3-time-format/test/utcFormat-test.js:74:3
    assert_eq!(
        f.format(utc(&[1990.0, 10.0, 1.0]), &TimeZone::UTC),
        "November"
    ); // d3-time-format/test/utcFormat-test.js:75:3
    assert_eq!(
        f.format(utc(&[1990.0, 11.0, 1.0]), &TimeZone::UTC),
        "December"
    ); // d3-time-format/test/utcFormat-test.js:76:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%c")(date) formats localized dates and times
#[test]
fn utcformat_c_date_formats_localized_dates_and_times() {
    let f = time::Locale::EN_US.format("%c");
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0]), &TimeZone::UTC),
        "1/1/1990, 12:00:00 AM"
    ); // d3-time-format/test/utcFormat-test.js:81:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%d")(date) formats zero-padded dates
#[test]
fn utcformat_d_date_formats_zero_padded_dates() {
    let f = time::Locale::EN_US.format("%d");
    assert_eq!(f.format(utc(&[1990.0, 0.0, 1.0]), &TimeZone::UTC), "01"); // d3-time-format/test/utcFormat-test.js:86:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%e")(date) formats space-padded dates
#[test]
fn utcformat_e_date_formats_space_padded_dates() {
    let f = time::Locale::EN_US.format("%e");
    assert_eq!(f.format(utc(&[1990.0, 0.0, 1.0]), &TimeZone::UTC), " 1"); // d3-time-format/test/utcFormat-test.js:91:3
}

// d3-time-format/test/utcFormat-test.js: timeFormat("%g")(date) formats zero-padded two-digit ISO 8601 years
#[test]
fn timeformat_g_date_formats_zero_padded_two_digit_iso_8601_years_2() {
    let f = time::Locale::EN_US.format("%g");
    assert_eq!(
        f.format(utc(&[2018.0, 11.0, 30.0, 0.0]), &TimeZone::UTC),
        "18"
    ); // d3-time-format/test/utcFormat-test.js:96:3
    assert_eq!(
        f.format(utc(&[2018.0, 11.0, 31.0, 0.0]), &TimeZone::UTC),
        "19"
    ); // d3-time-format/test/utcFormat-test.js:97:3
    assert_eq!(
        f.format(utc(&[2019.0, 0.0, 1.0, 0.0]), &TimeZone::UTC),
        "19"
    ); // d3-time-format/test/utcFormat-test.js:98:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%G")(date) formats zero-padded four-digit ISO 8601 years
#[test]
fn utcformat_g_date_formats_zero_padded_four_digit_iso_8601_years() {
    let f = time::Locale::EN_US.format("%G");
    assert_eq!(
        f.format(utc(&[2018.0, 11.0, 30.0, 0.0]), &TimeZone::UTC),
        "2018"
    ); // d3-time-format/test/utcFormat-test.js:103:3
    assert_eq!(
        f.format(utc(&[2018.0, 11.0, 31.0, 0.0]), &TimeZone::UTC),
        "2019"
    ); // d3-time-format/test/utcFormat-test.js:104:3
    assert_eq!(
        f.format(utc(&[2019.0, 0.0, 1.0, 0.0]), &TimeZone::UTC),
        "2019"
    ); // d3-time-format/test/utcFormat-test.js:105:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%H")(date) formats zero-padded hours (24)
#[test]
fn utcformat_h_date_formats_zero_padded_hours_24() {
    let f = time::Locale::EN_US.format("%H");
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0]), &TimeZone::UTC),
        "00"
    ); // d3-time-format/test/utcFormat-test.js:110:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 13.0]), &TimeZone::UTC),
        "13"
    ); // d3-time-format/test/utcFormat-test.js:111:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%I")(date) formats zero-padded hours (12)
#[test]
fn utcformat_i_date_formats_zero_padded_hours_12() {
    let f = time::Locale::EN_US.format("%I");
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0]), &TimeZone::UTC),
        "12"
    ); // d3-time-format/test/utcFormat-test.js:116:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 13.0]), &TimeZone::UTC),
        "01"
    ); // d3-time-format/test/utcFormat-test.js:117:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%j")(date) formats zero-padded day of year numbers
#[test]
fn utcformat_j_date_formats_zero_padded_day_of_year_numbers() {
    let f = time::Locale::EN_US.format("%j");
    assert_eq!(f.format(utc(&[1990.0, 0.0, 1.0]), &TimeZone::UTC), "001"); // d3-time-format/test/utcFormat-test.js:122:3
    assert_eq!(f.format(utc(&[1990.0, 5.0, 1.0]), &TimeZone::UTC), "152"); // d3-time-format/test/utcFormat-test.js:123:3
    assert_eq!(f.format(utc(&[2010.0, 2.0, 13.0]), &TimeZone::UTC), "072"); // d3-time-format/test/utcFormat-test.js:124:3
    assert_eq!(f.format(utc(&[2010.0, 2.0, 14.0]), &TimeZone::UTC), "073"); // d3-time-format/test/utcFormat-test.js:125:3
    assert_eq!(f.format(utc(&[2010.0, 2.0, 15.0]), &TimeZone::UTC), "074"); // d3-time-format/test/utcFormat-test.js:126:3
    assert_eq!(f.format(utc(&[2010.0, 10.0, 6.0]), &TimeZone::UTC), "310"); // d3-time-format/test/utcFormat-test.js:127:3
    assert_eq!(f.format(utc(&[2010.0, 10.0, 7.0]), &TimeZone::UTC), "311"); // d3-time-format/test/utcFormat-test.js:128:3
    assert_eq!(f.format(utc(&[2010.0, 10.0, 8.0]), &TimeZone::UTC), "312"); // d3-time-format/test/utcFormat-test.js:129:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%m")(date) formats zero-padded months
#[test]
fn utcformat_m_date_formats_zero_padded_months() {
    let f = time::Locale::EN_US.format("%m");
    assert_eq!(f.format(utc(&[1990.0, 0.0, 1.0]), &TimeZone::UTC), "01"); // d3-time-format/test/utcFormat-test.js:134:3
    assert_eq!(f.format(utc(&[1990.0, 9.0, 1.0]), &TimeZone::UTC), "10"); // d3-time-format/test/utcFormat-test.js:135:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%M")(date) formats zero-padded minutes
#[test]
fn utcformat_m_date_formats_zero_padded_minutes() {
    let f = time::Locale::EN_US.format("%M");
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0]), &TimeZone::UTC),
        "00"
    ); // d3-time-format/test/utcFormat-test.js:140:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0, 32.0]), &TimeZone::UTC),
        "32"
    ); // d3-time-format/test/utcFormat-test.js:141:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%p")(date) formats AM or PM
#[test]
fn utcformat_p_date_formats_am_or_pm() {
    let f = time::Locale::EN_US.format("%p");
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0]), &TimeZone::UTC),
        "AM"
    ); // d3-time-format/test/utcFormat-test.js:146:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 13.0]), &TimeZone::UTC),
        "PM"
    ); // d3-time-format/test/utcFormat-test.js:147:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%q")(date) formats quarters
#[test]
fn utcformat_q_date_formats_quarters() {
    let f = time::Locale::EN_US.format("%q");
    assert_eq!(f.format(utc(&[1990.0, 0.0, 1.0]), &TimeZone::UTC), "1"); // d3-time-format/test/utcFormat-test.js:152:3
    assert_eq!(f.format(utc(&[1990.0, 3.0, 1.0]), &TimeZone::UTC), "2"); // d3-time-format/test/utcFormat-test.js:153:3
    assert_eq!(f.format(utc(&[1990.0, 6.0, 1.0]), &TimeZone::UTC), "3"); // d3-time-format/test/utcFormat-test.js:154:3
    assert_eq!(f.format(utc(&[1990.0, 9.0, 1.0]), &TimeZone::UTC), "4"); // d3-time-format/test/utcFormat-test.js:155:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%Q")(date) formats UNIX timestamps
#[test]
fn utcformat_q_date_formats_unix_timestamps() {
    let f = time::Locale::EN_US.format("%Q");
    assert_eq!(
        f.format(utc(&[1970.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC),
        "0"
    ); // d3-time-format/test/utcFormat-test.js:160:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC),
        "631152000000"
    ); // d3-time-format/test/utcFormat-test.js:161:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 12.0, 34.0, 56.0]), &TimeZone::UTC),
        "631197296000"
    ); // d3-time-format/test/utcFormat-test.js:162:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%s")(date) formats UNIX timetamps in seconds
#[test]
fn utcformat_s_date_formats_unix_timetamps_in_seconds() {
    let f = time::Locale::EN_US.format("%s");
    assert_eq!(
        f.format(utc(&[1970.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC),
        "0"
    ); // d3-time-format/test/utcFormat-test.js:167:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC),
        "631152000"
    ); // d3-time-format/test/utcFormat-test.js:168:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 12.0, 34.0, 56.0]), &TimeZone::UTC),
        "631197296"
    ); // d3-time-format/test/utcFormat-test.js:169:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%s.%L")(date) formats UNIX timetamps in seconds and milliseconds
#[test]
fn utcformat_s_l_date_formats_unix_timetamps_in_seconds_and_milliseconds() {
    let f = time::Locale::EN_US.format("%s.%L");
    assert_eq!(
        f.format(
            utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0, 123.0]),
            &TimeZone::UTC
        ),
        "631152000.123"
    ); // d3-time-format/test/utcFormat-test.js:174:3
    assert_eq!(
        f.format(
            utc(&[1990.0, 0.0, 1.0, 12.0, 34.0, 56.0, 789.0]),
            &TimeZone::UTC
        ),
        "631197296.789"
    ); // d3-time-format/test/utcFormat-test.js:175:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%s.%f")(date) formats UNIX timetamps in seconds and microseconds
#[test]
fn utcformat_s_f_date_formats_unix_timetamps_in_seconds_and_microseconds() {
    let f = time::Locale::EN_US.format("%s.%f");
    assert_eq!(
        f.format(
            utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0, 123.0]),
            &TimeZone::UTC
        ),
        "631152000.123000"
    ); // d3-time-format/test/utcFormat-test.js:180:3
    assert_eq!(
        f.format(
            utc(&[1990.0, 0.0, 1.0, 12.0, 34.0, 56.0, 789.0]),
            &TimeZone::UTC
        ),
        "631197296.789000"
    ); // d3-time-format/test/utcFormat-test.js:181:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%S")(date) formats zero-padded seconds
#[test]
fn utcformat_s_date_formats_zero_padded_seconds() {
    let f = time::Locale::EN_US.format("%S");
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC),
        "00"
    ); // d3-time-format/test/utcFormat-test.js:186:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 32.0]), &TimeZone::UTC),
        "32"
    ); // d3-time-format/test/utcFormat-test.js:187:3
    let f2 = time::Locale::EN_US.format("%0S");
    assert_eq!(
        f2.format(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC),
        "00"
    ); // d3-time-format/test/utcFormat-test.js:189:3
    assert_eq!(
        f2.format(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 32.0]), &TimeZone::UTC),
        "32"
    ); // d3-time-format/test/utcFormat-test.js:190:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%_S")(date) formats space-padded seconds
#[test]
fn utcformat_s_date_formats_space_padded_seconds() {
    let f = time::Locale::EN_US.format("%_S");
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC),
        " 0"
    ); // d3-time-format/test/utcFormat-test.js:195:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 3.0]), &TimeZone::UTC),
        " 3"
    ); // d3-time-format/test/utcFormat-test.js:196:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 32.0]), &TimeZone::UTC),
        "32"
    ); // d3-time-format/test/utcFormat-test.js:197:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("-S")(date) formats no-padded seconds
#[test]
fn utcformat_s_date_formats_no_padded_seconds() {
    let f = time::Locale::EN_US.format("%-S");
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC),
        "0"
    ); // d3-time-format/test/utcFormat-test.js:202:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 3.0]), &TimeZone::UTC),
        "3"
    ); // d3-time-format/test/utcFormat-test.js:203:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 32.0]), &TimeZone::UTC),
        "32"
    ); // d3-time-format/test/utcFormat-test.js:204:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%L")(date) formats zero-padded milliseconds
#[test]
fn utcformat_l_date_formats_zero_padded_milliseconds() {
    let f = time::Locale::EN_US.format("%L");
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]), &TimeZone::UTC),
        "000"
    ); // d3-time-format/test/utcFormat-test.js:209:3
    assert_eq!(
        f.format(
            utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0, 432.0]),
            &TimeZone::UTC
        ),
        "432"
    ); // d3-time-format/test/utcFormat-test.js:210:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%u")(date) formats week day numbers
#[test]
fn utcformat_u_date_formats_week_day_numbers() {
    let f = time::Locale::EN_US.format("%u");
    assert_eq!(f.format(utc(&[1990.0, 0.0, 1.0, 0.0]), &TimeZone::UTC), "1"); // d3-time-format/test/utcFormat-test.js:215:3
    assert_eq!(f.format(utc(&[1990.0, 0.0, 7.0, 0.0]), &TimeZone::UTC), "7"); // d3-time-format/test/utcFormat-test.js:216:3
    assert_eq!(
        f.format(utc(&[2010.0, 2.0, 13.0, 23.0]), &TimeZone::UTC),
        "6"
    ); // d3-time-format/test/utcFormat-test.js:217:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%f")(date) formats zero-padded microseconds
#[test]
fn utcformat_f_date_formats_zero_padded_microseconds() {
    let f = time::Locale::EN_US.format("%f");
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]), &TimeZone::UTC),
        "000000"
    ); // d3-time-format/test/utcFormat-test.js:222:3
    assert_eq!(
        f.format(
            utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0, 432.0]),
            &TimeZone::UTC
        ),
        "432000"
    ); // d3-time-format/test/utcFormat-test.js:223:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%U")(date) formats zero-padded week numbers
#[test]
fn utcformat_u_date_formats_zero_padded_week_numbers() {
    let f = time::Locale::EN_US.format("%U");
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0]), &TimeZone::UTC),
        "00"
    ); // d3-time-format/test/utcFormat-test.js:228:3
    assert_eq!(
        f.format(utc(&[1990.0, 5.0, 1.0, 0.0]), &TimeZone::UTC),
        "21"
    ); // d3-time-format/test/utcFormat-test.js:229:3
    assert_eq!(
        f.format(utc(&[2010.0, 2.0, 13.0, 23.0]), &TimeZone::UTC),
        "10"
    ); // d3-time-format/test/utcFormat-test.js:230:3
    assert_eq!(
        f.format(utc(&[2010.0, 2.0, 14.0, 0.0]), &TimeZone::UTC),
        "11"
    ); // d3-time-format/test/utcFormat-test.js:231:3
    assert_eq!(
        f.format(utc(&[2010.0, 2.0, 15.0, 0.0]), &TimeZone::UTC),
        "11"
    ); // d3-time-format/test/utcFormat-test.js:232:3
    assert_eq!(
        f.format(utc(&[2010.0, 10.0, 6.0, 23.0]), &TimeZone::UTC),
        "44"
    ); // d3-time-format/test/utcFormat-test.js:233:3
    assert_eq!(
        f.format(utc(&[2010.0, 10.0, 7.0, 0.0]), &TimeZone::UTC),
        "45"
    ); // d3-time-format/test/utcFormat-test.js:234:3
    assert_eq!(
        f.format(utc(&[2010.0, 10.0, 8.0, 0.0]), &TimeZone::UTC),
        "45"
    ); // d3-time-format/test/utcFormat-test.js:235:3
    assert_eq!(
        f.format(utc(&[2012.0, 0.0, 1.0, 0.0]), &TimeZone::UTC),
        "01"
    ); // d3-time-format/test/utcFormat-test.js:236:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%W")(date) formats zero-padded week numbers
#[test]
fn utcformat_w_date_formats_zero_padded_week_numbers() {
    let f = time::Locale::EN_US.format("%W");
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0]), &TimeZone::UTC),
        "01"
    ); // d3-time-format/test/utcFormat-test.js:241:3
    assert_eq!(
        f.format(utc(&[1990.0, 5.0, 1.0, 0.0]), &TimeZone::UTC),
        "22"
    ); // d3-time-format/test/utcFormat-test.js:242:3
    assert_eq!(
        f.format(utc(&[2010.0, 2.0, 15.0, 0.0]), &TimeZone::UTC),
        "11"
    ); // d3-time-format/test/utcFormat-test.js:243:3
    assert_eq!(
        f.format(utc(&[2010.0, 10.0, 8.0, 0.0]), &TimeZone::UTC),
        "45"
    ); // d3-time-format/test/utcFormat-test.js:244:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%V")(date) formats zero-padded ISO 8601 week numbers
#[test]
fn utcformat_v_date_formats_zero_padded_iso_8601_week_numbers() {
    let f = time::Locale::EN_US.format("%V");
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0]), &TimeZone::UTC),
        "01"
    ); // d3-time-format/test/utcFormat-test.js:249:3
    assert_eq!(
        f.format(utc(&[1990.0, 5.0, 1.0, 0.0]), &TimeZone::UTC),
        "22"
    ); // d3-time-format/test/utcFormat-test.js:250:3
    assert_eq!(
        f.format(utc(&[2010.0, 2.0, 13.0, 23.0]), &TimeZone::UTC),
        "10"
    ); // d3-time-format/test/utcFormat-test.js:251:3
    assert_eq!(
        f.format(utc(&[2010.0, 2.0, 14.0, 0.0]), &TimeZone::UTC),
        "10"
    ); // d3-time-format/test/utcFormat-test.js:252:3
    assert_eq!(
        f.format(utc(&[2010.0, 2.0, 15.0, 0.0]), &TimeZone::UTC),
        "11"
    ); // d3-time-format/test/utcFormat-test.js:253:3
    assert_eq!(
        f.format(utc(&[2010.0, 10.0, 6.0, 23.0]), &TimeZone::UTC),
        "44"
    ); // d3-time-format/test/utcFormat-test.js:254:3
    assert_eq!(
        f.format(utc(&[2010.0, 10.0, 7.0, 0.0]), &TimeZone::UTC),
        "44"
    ); // d3-time-format/test/utcFormat-test.js:255:3
    assert_eq!(
        f.format(utc(&[2010.0, 10.0, 8.0, 0.0]), &TimeZone::UTC),
        "45"
    ); // d3-time-format/test/utcFormat-test.js:256:3
    assert_eq!(
        f.format(utc(&[2015.0, 11.0, 31.0, 0.0]), &TimeZone::UTC),
        "53"
    ); // d3-time-format/test/utcFormat-test.js:257:3
    assert_eq!(
        f.format(utc(&[2016.0, 0.0, 1.0, 0.0]), &TimeZone::UTC),
        "53"
    ); // d3-time-format/test/utcFormat-test.js:258:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%x")(date) formats localized dates
#[test]
fn utcformat_x_date_formats_localized_dates() {
    let f = time::Locale::EN_US.format("%x");
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0]), &TimeZone::UTC),
        "1/1/1990"
    ); // d3-time-format/test/utcFormat-test.js:263:3
    assert_eq!(
        f.format(utc(&[2010.0, 5.0, 1.0]), &TimeZone::UTC),
        "6/1/2010"
    ); // d3-time-format/test/utcFormat-test.js:264:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%X")(date) formats localized times
#[test]
fn utcformat_x_date_formats_localized_times() {
    let f = time::Locale::EN_US.format("%X");
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0]), &TimeZone::UTC),
        "12:00:00 AM"
    ); // d3-time-format/test/utcFormat-test.js:269:3
    assert_eq!(
        f.format(utc(&[1990.0, 0.0, 1.0, 13.0, 34.0, 59.0]), &TimeZone::UTC),
        "1:34:59 PM"
    ); // d3-time-format/test/utcFormat-test.js:270:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%y")(date) formats zero-padded two-digit years
#[test]
fn utcformat_y_date_formats_zero_padded_two_digit_years() {
    let f = time::Locale::EN_US.format("%y");
    assert_eq!(f.format(utc(&[1990.0, 0.0, 1.0]), &TimeZone::UTC), "90"); // d3-time-format/test/utcFormat-test.js:275:3
    assert_eq!(f.format(utc(&[2002.0, 0.0, 1.0]), &TimeZone::UTC), "02"); // d3-time-format/test/utcFormat-test.js:276:3
    assert_eq!(f.format(utc(&[-2.0, 0.0, 1.0]), &TimeZone::UTC), "-02"); // d3-time-format/test/utcFormat-test.js:277:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%Y")(date) formats zero-padded four-digit years
#[test]
fn utcformat_y_date_formats_zero_padded_four_digit_years() {
    let f = time::Locale::EN_US.format("%Y");
    assert_eq!(f.format(utc(&[123.0, 0.0, 1.0]), &TimeZone::UTC), "0123"); // d3-time-format/test/utcFormat-test.js:282:3
    assert_eq!(f.format(utc(&[1990.0, 0.0, 1.0]), &TimeZone::UTC), "1990"); // d3-time-format/test/utcFormat-test.js:283:3
    assert_eq!(f.format(utc(&[2002.0, 0.0, 1.0]), &TimeZone::UTC), "2002"); // d3-time-format/test/utcFormat-test.js:284:3
    assert_eq!(f.format(utc(&[-2.0, 0.0, 1.0]), &TimeZone::UTC), "-0002"); // d3-time-format/test/utcFormat-test.js:286:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%Z")(date) formats time zones
#[test]
fn utcformat_z_date_formats_time_zones() {
    let f = time::Locale::EN_US.format("%Z");
    assert_eq!(f.format(utc(&[1990.0, 0.0, 1.0]), &TimeZone::UTC), "+0000"); // d3-time-format/test/utcFormat-test.js:291:3
}

// d3-time-format/test/utcFormat-test.js: utcFormat("%%")(date) formats literal percent signs
#[test]
fn utcformat_date_formats_literal_percent_signs() {
    let f = time::Locale::EN_US.format("%%");
    assert_eq!(f.format(utc(&[1990.0, 0.0, 1.0]), &TimeZone::UTC), "%"); // d3-time-format/test/utcFormat-test.js:296:3
}

// d3-time-format/test/isoFormat-test.js: isoFormat(date) returns an ISO 8601 UTC string
#[test]
fn isoformat_date_returns_an_iso_8601_utc_string() {
    assert_eq!(
        time::iso_format(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0])),
        "1990-01-01T00:00:00.000Z"
    ); // d3-time-format/test/isoFormat-test.js:6:3
    assert_eq!(
        time::iso_format(utc(&[2011.0, 11.0, 31.0, 23.0, 59.0, 59.0])),
        "2011-12-31T23:59:59.000Z"
    ); // d3-time-format/test/isoFormat-test.js:7:3
}

// d3-time-format/test/isoParse-test.js: isoParse as ISO 8601
#[test]
fn isoparse_as_iso_8601() {
    assert_eq!(
        ts(time::iso_parse("1990-01-01T00:00:00.000Z").expect("d3 test")),
        ts(utc(&[1990.0, 0.0, 1.0, 0.0, 0.0, 0.0]))
    ); // d3-time-format/test/isoParse-test.js:6:3
    assert_eq!(
        ts(time::iso_parse("2011-12-31T23:59:59.000Z").expect("d3 test")),
        ts(utc(&[2011.0, 11.0, 31.0, 23.0, 59.0, 59.0]))
    ); // d3-time-format/test/isoParse-test.js:7:3
    assert!(time::iso_parse("1990-01-01T00:00:00.000X").is_err()); // d3-time-format/test/isoParse-test.js:8:3
}

// d3-time-format/test/locale-test.js: locale data is valid
#[test]
fn locale_data_is_valid() {
    let tables: [(&[&str], time::Definition); 35] = [
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%x, %X".into(),
                date: "%-d/%-m/%Y".into(),
                time: "%-I:%M:%S %p".into(),
                periods: ["ص".into(), "م".into()],
                days: [
                    "الأحد".into(),
                    "الإثنين".into(),
                    "الثلاثاء".into(),
                    "الأربعاء".into(),
                    "الخميس".into(),
                    "الجمعة".into(),
                    "السبت".into(),
                ],
                short_days: [
                    "أحد".into(),
                    "إثنين".into(),
                    "ثلاثاء".into(),
                    "أربعاء".into(),
                    "خميس".into(),
                    "جمعة".into(),
                    "سبت".into(),
                ],
                months: [
                    "يناير".into(),
                    "فبراير".into(),
                    "مارس".into(),
                    "أبريل".into(),
                    "مايو".into(),
                    "يونيو".into(),
                    "يوليو".into(),
                    "أغسطس".into(),
                    "سبتمبر".into(),
                    "أكتوبر".into(),
                    "نوفمبر".into(),
                    "ديسمبر".into(),
                ],
                short_months: [
                    "يناير".into(),
                    "فبراير".into(),
                    "مارس".into(),
                    "أبريل".into(),
                    "مايو".into(),
                    "يونيو".into(),
                    "يوليو".into(),
                    "أغسطس".into(),
                    "سبتمبر".into(),
                    "أكتوبر".into(),
                    "نوفمبر".into(),
                    "ديسمبر".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%x, %X".into(),
                date: "%-d/%-m/%Y".into(),
                time: "%-I:%M:%S %p".into(),
                periods: ["ص".into(), "م".into()],
                days: [
                    "الأحد".into(),
                    "الإثنين".into(),
                    "الثلاثاء".into(),
                    "الأربعاء".into(),
                    "الخميس".into(),
                    "الجمعة".into(),
                    "السبت".into(),
                ],
                short_days: [
                    "أحد".into(),
                    "إثنين".into(),
                    "ثلاثاء".into(),
                    "أربعاء".into(),
                    "خميس".into(),
                    "جمعة".into(),
                    "سبت".into(),
                ],
                months: [
                    "كانون الثاني".into(),
                    "شباط".into(),
                    "آذار".into(),
                    "نيسان".into(),
                    "أيار".into(),
                    "حزيران".into(),
                    "تموز".into(),
                    "آب".into(),
                    "أيلول".into(),
                    "تشرين الأول".into(),
                    "تشرين الثاني".into(),
                    "كانون الأول".into(),
                ],
                short_months: [
                    "ك٢".into(),
                    "شباط".into(),
                    "آذار".into(),
                    "نيسان".into(),
                    "أيار".into(),
                    "حزيران".into(),
                    "تموز".into(),
                    "آب".into(),
                    "أيلول".into(),
                    "ت١".into(),
                    "ت٢".into(),
                    "ك١".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A, %e de %B de %Y, %X".into(),
                date: "%d/%m/%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "diumenge".into(),
                    "dilluns".into(),
                    "dimarts".into(),
                    "dimecres".into(),
                    "dijous".into(),
                    "divendres".into(),
                    "dissabte".into(),
                ],
                short_days: [
                    "dg.".into(),
                    "dl.".into(),
                    "dt.".into(),
                    "dc.".into(),
                    "dj.".into(),
                    "dv.".into(),
                    "ds.".into(),
                ],
                months: [
                    "gener".into(),
                    "febrer".into(),
                    "març".into(),
                    "abril".into(),
                    "maig".into(),
                    "juny".into(),
                    "juliol".into(),
                    "agost".into(),
                    "setembre".into(),
                    "octubre".into(),
                    "novembre".into(),
                    "desembre".into(),
                ],
                short_months: [
                    "gen.".into(),
                    "febr.".into(),
                    "març".into(),
                    "abr.".into(),
                    "maig".into(),
                    "juny".into(),
                    "jul.".into(),
                    "ag.".into(),
                    "set.".into(),
                    "oct.".into(),
                    "nov.".into(),
                    "des.".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A,%e.%B %Y, %X".into(),
                date: "%-d.%-m.%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "neděle".into(),
                    "pondělí".into(),
                    "úterý".into(),
                    "středa".into(),
                    "čvrtek".into(),
                    "pátek".into(),
                    "sobota".into(),
                ],
                short_days: [
                    "ne.".into(),
                    "po.".into(),
                    "út.".into(),
                    "st.".into(),
                    "čt.".into(),
                    "pá.".into(),
                    "so.".into(),
                ],
                months: [
                    "leden".into(),
                    "únor".into(),
                    "březen".into(),
                    "duben".into(),
                    "květen".into(),
                    "červen".into(),
                    "červenec".into(),
                    "srpen".into(),
                    "září".into(),
                    "říjen".into(),
                    "listopad".into(),
                    "prosinec".into(),
                ],
                short_months: [
                    "led".into(),
                    "úno".into(),
                    "břez".into(),
                    "dub".into(),
                    "kvě".into(),
                    "čer".into(),
                    "červ".into(),
                    "srp".into(),
                    "zář".into(),
                    "říj".into(),
                    "list".into(),
                    "pros".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A den %d %B %Y %X".into(),
                date: "%d-%m-%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "søndag".into(),
                    "mandag".into(),
                    "tirsdag".into(),
                    "onsdag".into(),
                    "torsdag".into(),
                    "fredag".into(),
                    "lørdag".into(),
                ],
                short_days: [
                    "søn".into(),
                    "man".into(),
                    "tir".into(),
                    "ons".into(),
                    "tor".into(),
                    "fre".into(),
                    "lør".into(),
                ],
                months: [
                    "januar".into(),
                    "februar".into(),
                    "marts".into(),
                    "april".into(),
                    "maj".into(),
                    "juni".into(),
                    "juli".into(),
                    "august".into(),
                    "september".into(),
                    "oktober".into(),
                    "november".into(),
                    "december".into(),
                ],
                short_months: [
                    "jan".into(),
                    "feb".into(),
                    "mar".into(),
                    "apr".into(),
                    "maj".into(),
                    "jun".into(),
                    "jul".into(),
                    "aug".into(),
                    "sep".into(),
                    "okt".into(),
                    "nov".into(),
                    "dec".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A, der %e. %B %Y, %X".into(),
                date: "%d.%m.%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "Sonntag".into(),
                    "Montag".into(),
                    "Dienstag".into(),
                    "Mittwoch".into(),
                    "Donnerstag".into(),
                    "Freitag".into(),
                    "Samstag".into(),
                ],
                short_days: [
                    "So".into(),
                    "Mo".into(),
                    "Di".into(),
                    "Mi".into(),
                    "Do".into(),
                    "Fr".into(),
                    "Sa".into(),
                ],
                months: [
                    "Januar".into(),
                    "Februar".into(),
                    "März".into(),
                    "April".into(),
                    "Mai".into(),
                    "Juni".into(),
                    "Juli".into(),
                    "August".into(),
                    "September".into(),
                    "Oktober".into(),
                    "November".into(),
                    "Dezember".into(),
                ],
                short_months: [
                    "Jan".into(),
                    "Feb".into(),
                    "Mrz".into(),
                    "Apr".into(),
                    "Mai".into(),
                    "Jun".into(),
                    "Jul".into(),
                    "Aug".into(),
                    "Sep".into(),
                    "Okt".into(),
                    "Nov".into(),
                    "Dez".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A, der %e. %B %Y, %X".into(),
                date: "%d.%m.%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "Sonntag".into(),
                    "Montag".into(),
                    "Dienstag".into(),
                    "Mittwoch".into(),
                    "Donnerstag".into(),
                    "Freitag".into(),
                    "Samstag".into(),
                ],
                short_days: [
                    "So".into(),
                    "Mo".into(),
                    "Di".into(),
                    "Mi".into(),
                    "Do".into(),
                    "Fr".into(),
                    "Sa".into(),
                ],
                months: [
                    "Januar".into(),
                    "Februar".into(),
                    "März".into(),
                    "April".into(),
                    "Mai".into(),
                    "Juni".into(),
                    "Juli".into(),
                    "August".into(),
                    "September".into(),
                    "Oktober".into(),
                    "November".into(),
                    "Dezember".into(),
                ],
                short_months: [
                    "Jan".into(),
                    "Feb".into(),
                    "Mrz".into(),
                    "Apr".into(),
                    "Mai".into(),
                    "Jun".into(),
                    "Jul".into(),
                    "Aug".into(),
                    "Sep".into(),
                    "Okt".into(),
                    "Nov".into(),
                    "Dez".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%a %b %e %X %Y".into(),
                date: "%Y-%m-%d".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "Sunday".into(),
                    "Monday".into(),
                    "Tuesday".into(),
                    "Wednesday".into(),
                    "Thursday".into(),
                    "Friday".into(),
                    "Saturday".into(),
                ],
                short_days: [
                    "Sun".into(),
                    "Mon".into(),
                    "Tue".into(),
                    "Wed".into(),
                    "Thu".into(),
                    "Fri".into(),
                    "Sat".into(),
                ],
                months: [
                    "January".into(),
                    "February".into(),
                    "March".into(),
                    "April".into(),
                    "May".into(),
                    "June".into(),
                    "July".into(),
                    "August".into(),
                    "September".into(),
                    "October".into(),
                    "November".into(),
                    "December".into(),
                ],
                short_months: [
                    "Jan".into(),
                    "Feb".into(),
                    "Mar".into(),
                    "Apr".into(),
                    "May".into(),
                    "Jun".into(),
                    "Jul".into(),
                    "Aug".into(),
                    "Sep".into(),
                    "Oct".into(),
                    "Nov".into(),
                    "Dec".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%a %e %b %X %Y".into(),
                date: "%d/%m/%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "Sunday".into(),
                    "Monday".into(),
                    "Tuesday".into(),
                    "Wednesday".into(),
                    "Thursday".into(),
                    "Friday".into(),
                    "Saturday".into(),
                ],
                short_days: [
                    "Sun".into(),
                    "Mon".into(),
                    "Tue".into(),
                    "Wed".into(),
                    "Thu".into(),
                    "Fri".into(),
                    "Sat".into(),
                ],
                months: [
                    "January".into(),
                    "February".into(),
                    "March".into(),
                    "April".into(),
                    "May".into(),
                    "June".into(),
                    "July".into(),
                    "August".into(),
                    "September".into(),
                    "October".into(),
                    "November".into(),
                    "December".into(),
                ],
                short_months: [
                    "Jan".into(),
                    "Feb".into(),
                    "Mar".into(),
                    "Apr".into(),
                    "May".into(),
                    "Jun".into(),
                    "Jul".into(),
                    "Aug".into(),
                    "Sep".into(),
                    "Oct".into(),
                    "Nov".into(),
                    "Dec".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%x, %X".into(),
                date: "%-m/%-d/%Y".into(),
                time: "%-I:%M:%S %p".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "Sunday".into(),
                    "Monday".into(),
                    "Tuesday".into(),
                    "Wednesday".into(),
                    "Thursday".into(),
                    "Friday".into(),
                    "Saturday".into(),
                ],
                short_days: [
                    "Sun".into(),
                    "Mon".into(),
                    "Tue".into(),
                    "Wed".into(),
                    "Thu".into(),
                    "Fri".into(),
                    "Sat".into(),
                ],
                months: [
                    "January".into(),
                    "February".into(),
                    "March".into(),
                    "April".into(),
                    "May".into(),
                    "June".into(),
                    "July".into(),
                    "August".into(),
                    "September".into(),
                    "October".into(),
                    "November".into(),
                    "December".into(),
                ],
                short_months: [
                    "Jan".into(),
                    "Feb".into(),
                    "Mar".into(),
                    "Apr".into(),
                    "May".into(),
                    "Jun".into(),
                    "Jul".into(),
                    "Aug".into(),
                    "Sep".into(),
                    "Oct".into(),
                    "Nov".into(),
                    "Dec".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A, %e de %B de %Y, %X".into(),
                date: "%d/%m/%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "domingo".into(),
                    "lunes".into(),
                    "martes".into(),
                    "miércoles".into(),
                    "jueves".into(),
                    "viernes".into(),
                    "sábado".into(),
                ],
                short_days: [
                    "dom".into(),
                    "lun".into(),
                    "mar".into(),
                    "mié".into(),
                    "jue".into(),
                    "vie".into(),
                    "sáb".into(),
                ],
                months: [
                    "enero".into(),
                    "febrero".into(),
                    "marzo".into(),
                    "abril".into(),
                    "mayo".into(),
                    "junio".into(),
                    "julio".into(),
                    "agosto".into(),
                    "septiembre".into(),
                    "octubre".into(),
                    "noviembre".into(),
                    "diciembre".into(),
                ],
                short_months: [
                    "ene".into(),
                    "feb".into(),
                    "mar".into(),
                    "abr".into(),
                    "may".into(),
                    "jun".into(),
                    "jul".into(),
                    "ago".into(),
                    "sep".into(),
                    "oct".into(),
                    "nov".into(),
                    "dic".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%x, %X".into(),
                date: "%d/%m/%Y".into(),
                time: "%-I:%M:%S %p".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "domingo".into(),
                    "lunes".into(),
                    "martes".into(),
                    "miércoles".into(),
                    "jueves".into(),
                    "viernes".into(),
                    "sábado".into(),
                ],
                short_days: [
                    "dom".into(),
                    "lun".into(),
                    "mar".into(),
                    "mié".into(),
                    "jue".into(),
                    "vie".into(),
                    "sáb".into(),
                ],
                months: [
                    "enero".into(),
                    "febrero".into(),
                    "marzo".into(),
                    "abril".into(),
                    "mayo".into(),
                    "junio".into(),
                    "julio".into(),
                    "agosto".into(),
                    "septiembre".into(),
                    "octubre".into(),
                    "noviembre".into(),
                    "diciembre".into(),
                ],
                short_months: [
                    "ene".into(),
                    "feb".into(),
                    "mar".into(),
                    "abr".into(),
                    "may".into(),
                    "jun".into(),
                    "jul".into(),
                    "ago".into(),
                    "sep".into(),
                    "oct".into(),
                    "nov".into(),
                    "dic".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%x, %X".into(),
                date: "%-d/%-m/%Y".into(),
                time: "%-I:%M:%S %p".into(),
                periods: ["صبح".into(), "عصر".into()],
                days: [
                    "یکشنبه".into(),
                    "دوشنبه".into(),
                    "سه شنبه".into(),
                    "چهارشنبه".into(),
                    "پنجشنبه".into(),
                    "جمعه".into(),
                    "شنبه".into(),
                ],
                short_days: [
                    "یکشنبه".into(),
                    "دوشنبه".into(),
                    "سه شنبه".into(),
                    "چهارشنبه".into(),
                    "پنجشنبه".into(),
                    "جمعه".into(),
                    "شنبه".into(),
                ],
                months: [
                    "ژانویه".into(),
                    "فوریه".into(),
                    "مارس".into(),
                    "آوریل".into(),
                    "مه".into(),
                    "ژوئن".into(),
                    "ژوئیه".into(),
                    "اوت".into(),
                    "سپتامبر".into(),
                    "اکتبر".into(),
                    "نوامبر".into(),
                    "دسامبر".into(),
                ],
                short_months: [
                    "ژانویه".into(),
                    "فوریه".into(),
                    "مارس".into(),
                    "آوریل".into(),
                    "مه".into(),
                    "ژوئن".into(),
                    "ژوئیه".into(),
                    "اوت".into(),
                    "سپتامبر".into(),
                    "اکتبر".into(),
                    "نوامبر".into(),
                    "دسامبر".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A, %-d. %Bta %Y klo %X".into(),
                date: "%-d.%-m.%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["a.m.".into(), "p.m.".into()],
                days: [
                    "sunnuntai".into(),
                    "maanantai".into(),
                    "tiistai".into(),
                    "keskiviikko".into(),
                    "torstai".into(),
                    "perjantai".into(),
                    "lauantai".into(),
                ],
                short_days: [
                    "Su".into(),
                    "Ma".into(),
                    "Ti".into(),
                    "Ke".into(),
                    "To".into(),
                    "Pe".into(),
                    "La".into(),
                ],
                months: [
                    "tammikuu".into(),
                    "helmikuu".into(),
                    "maaliskuu".into(),
                    "huhtikuu".into(),
                    "toukokuu".into(),
                    "kesäkuu".into(),
                    "heinäkuu".into(),
                    "elokuu".into(),
                    "syyskuu".into(),
                    "lokakuu".into(),
                    "marraskuu".into(),
                    "joulukuu".into(),
                ],
                short_months: [
                    "Tammi".into(),
                    "Helmi".into(),
                    "Maalis".into(),
                    "Huhti".into(),
                    "Touko".into(),
                    "Kesä".into(),
                    "Heinä".into(),
                    "Elo".into(),
                    "Syys".into(),
                    "Loka".into(),
                    "Marras".into(),
                    "Joulu".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%a %e %b %Y %X".into(),
                date: "%Y-%m-%d".into(),
                time: "%H:%M:%S".into(),
                periods: ["".into(), "".into()],
                days: [
                    "dimanche".into(),
                    "lundi".into(),
                    "mardi".into(),
                    "mercredi".into(),
                    "jeudi".into(),
                    "vendredi".into(),
                    "samedi".into(),
                ],
                short_days: [
                    "dim".into(),
                    "lun".into(),
                    "mar".into(),
                    "mer".into(),
                    "jeu".into(),
                    "ven".into(),
                    "sam".into(),
                ],
                months: [
                    "janvier".into(),
                    "février".into(),
                    "mars".into(),
                    "avril".into(),
                    "mai".into(),
                    "juin".into(),
                    "juillet".into(),
                    "août".into(),
                    "septembre".into(),
                    "octobre".into(),
                    "novembre".into(),
                    "décembre".into(),
                ],
                short_months: [
                    "jan".into(),
                    "fév".into(),
                    "mar".into(),
                    "avr".into(),
                    "mai".into(),
                    "jui".into(),
                    "jul".into(),
                    "aoû".into(),
                    "sep".into(),
                    "oct".into(),
                    "nov".into(),
                    "déc".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A %e %B %Y à %X".into(),
                date: "%d/%m/%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "dimanche".into(),
                    "lundi".into(),
                    "mardi".into(),
                    "mercredi".into(),
                    "jeudi".into(),
                    "vendredi".into(),
                    "samedi".into(),
                ],
                short_days: [
                    "dim.".into(),
                    "lun.".into(),
                    "mar.".into(),
                    "mer.".into(),
                    "jeu.".into(),
                    "ven.".into(),
                    "sam.".into(),
                ],
                months: [
                    "janvier".into(),
                    "février".into(),
                    "mars".into(),
                    "avril".into(),
                    "mai".into(),
                    "juin".into(),
                    "juillet".into(),
                    "août".into(),
                    "septembre".into(),
                    "octobre".into(),
                    "novembre".into(),
                    "décembre".into(),
                ],
                short_months: [
                    "janv.".into(),
                    "févr.".into(),
                    "mars".into(),
                    "avr.".into(),
                    "mai".into(),
                    "juin".into(),
                    "juil.".into(),
                    "août".into(),
                    "sept.".into(),
                    "oct.".into(),
                    "nov.".into(),
                    "déc.".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A, %e ב%B %Y %X".into(),
                date: "%d.%m.%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "ראשון".into(),
                    "שני".into(),
                    "שלישי".into(),
                    "רביעי".into(),
                    "חמישי".into(),
                    "שישי".into(),
                    "שבת".into(),
                ],
                short_days: [
                    "א׳".into(),
                    "ב׳".into(),
                    "ג׳".into(),
                    "ד׳".into(),
                    "ה׳".into(),
                    "ו׳".into(),
                    "ש׳".into(),
                ],
                months: [
                    "ינואר".into(),
                    "פברואר".into(),
                    "מרץ".into(),
                    "אפריל".into(),
                    "מאי".into(),
                    "יוני".into(),
                    "יולי".into(),
                    "אוגוסט".into(),
                    "ספטמבר".into(),
                    "אוקטובר".into(),
                    "נובמבר".into(),
                    "דצמבר".into(),
                ],
                short_months: [
                    "ינו׳".into(),
                    "פבר׳".into(),
                    "מרץ".into(),
                    "אפר׳".into(),
                    "מאי".into(),
                    "יוני".into(),
                    "יולי".into(),
                    "אוג׳".into(),
                    "ספט׳".into(),
                    "אוק׳".into(),
                    "נוב׳".into(),
                    "דצמ׳".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A, %e. %B %Y., %X".into(),
                date: "%d. %m. %Y.".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "Nedjelja".into(),
                    "Ponedjeljak".into(),
                    "Utorak".into(),
                    "Srijeda".into(),
                    "Četvtrak".into(),
                    "Petak".into(),
                    "Subota".into(),
                ],
                short_days: [
                    "Ne".into(),
                    "Po".into(),
                    "Ut".into(),
                    "Sr".into(),
                    "Če".into(),
                    "Pe".into(),
                    "Su".into(),
                ],
                months: [
                    "Siječanj".into(),
                    "Veljača".into(),
                    "Ožujak".into(),
                    "Travanj".into(),
                    "Svibanj".into(),
                    "Lipanj".into(),
                    "Srpanj".into(),
                    "Kolovoz".into(),
                    "Rujan".into(),
                    "Listopad".into(),
                    "Studeni".into(),
                    "Prosinac".into(),
                ],
                short_months: [
                    "Sij".into(),
                    "Velj".into(),
                    "Ožu".into(),
                    "Tra".into(),
                    "Svi".into(),
                    "Lip".into(),
                    "Srp".into(),
                    "Kol".into(),
                    "Ruj".into(),
                    "Lis".into(),
                    "Stu".into(),
                    "Pro".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%Y. %B %-e., %A %X".into(),
                date: "%Y. %m. %d.".into(),
                time: "%H:%M:%S".into(),
                periods: ["de.".into(), "du.".into()],
                days: [
                    "vasárnap".into(),
                    "hétfő".into(),
                    "kedd".into(),
                    "szerda".into(),
                    "csütörtök".into(),
                    "péntek".into(),
                    "szombat".into(),
                ],
                short_days: [
                    "V".into(),
                    "H".into(),
                    "K".into(),
                    "Sze".into(),
                    "Cs".into(),
                    "P".into(),
                    "Szo".into(),
                ],
                months: [
                    "január".into(),
                    "február".into(),
                    "március".into(),
                    "április".into(),
                    "május".into(),
                    "június".into(),
                    "július".into(),
                    "augusztus".into(),
                    "szeptember".into(),
                    "október".into(),
                    "november".into(),
                    "december".into(),
                ],
                short_months: [
                    "jan.".into(),
                    "feb.".into(),
                    "már.".into(),
                    "ápr.".into(),
                    "máj.".into(),
                    "jún.".into(),
                    "júl.".into(),
                    "aug.".into(),
                    "szept.".into(),
                    "okt.".into(),
                    "nov.".into(),
                    "dec.".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A %e %B %Y, %X".into(),
                date: "%d/%m/%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "Domenica".into(),
                    "Lunedì".into(),
                    "Martedì".into(),
                    "Mercoledì".into(),
                    "Giovedì".into(),
                    "Venerdì".into(),
                    "Sabato".into(),
                ],
                short_days: [
                    "Dom".into(),
                    "Lun".into(),
                    "Mar".into(),
                    "Mer".into(),
                    "Gio".into(),
                    "Ven".into(),
                    "Sab".into(),
                ],
                months: [
                    "Gennaio".into(),
                    "Febbraio".into(),
                    "Marzo".into(),
                    "Aprile".into(),
                    "Maggio".into(),
                    "Giugno".into(),
                    "Luglio".into(),
                    "Agosto".into(),
                    "Settembre".into(),
                    "Ottobre".into(),
                    "Novembre".into(),
                    "Dicembre".into(),
                ],
                short_months: [
                    "Gen".into(),
                    "Feb".into(),
                    "Mar".into(),
                    "Apr".into(),
                    "Mag".into(),
                    "Giu".into(),
                    "Lug".into(),
                    "Ago".into(),
                    "Set".into(),
                    "Ott".into(),
                    "Nov".into(),
                    "Dic".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%x %a %X".into(),
                date: "%Y/%m/%d".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "日曜日".into(),
                    "月曜日".into(),
                    "火曜日".into(),
                    "水曜日".into(),
                    "木曜日".into(),
                    "金曜日".into(),
                    "土曜日".into(),
                ],
                short_days: [
                    "日".into(),
                    "月".into(),
                    "火".into(),
                    "水".into(),
                    "木".into(),
                    "金".into(),
                    "土".into(),
                ],
                months: [
                    "1月".into(),
                    "2月".into(),
                    "3月".into(),
                    "4月".into(),
                    "5月".into(),
                    "6月".into(),
                    "7月".into(),
                    "8月".into(),
                    "9月".into(),
                    "10月".into(),
                    "11月".into(),
                    "12月".into(),
                ],
                short_months: [
                    "1月".into(),
                    "2月".into(),
                    "3月".into(),
                    "4月".into(),
                    "5月".into(),
                    "6月".into(),
                    "7月".into(),
                    "8月".into(),
                    "9月".into(),
                    "10月".into(),
                    "11月".into(),
                    "12月".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%Y/%m/%d %a %X".into(),
                date: "%Y/%m/%d".into(),
                time: "%H:%M:%S".into(),
                periods: ["오전".into(), "오후".into()],
                days: [
                    "일요일".into(),
                    "월요일".into(),
                    "화요일".into(),
                    "수요일".into(),
                    "목요일".into(),
                    "금요일".into(),
                    "토요일".into(),
                ],
                short_days: [
                    "일".into(),
                    "월".into(),
                    "화".into(),
                    "수".into(),
                    "목".into(),
                    "금".into(),
                    "토".into(),
                ],
                months: [
                    "1월".into(),
                    "2월".into(),
                    "3월".into(),
                    "4월".into(),
                    "5월".into(),
                    "6월".into(),
                    "7월".into(),
                    "8월".into(),
                    "9월".into(),
                    "10월".into(),
                    "11월".into(),
                    "12월".into(),
                ],
                short_months: [
                    "1월".into(),
                    "2월".into(),
                    "3월".into(),
                    "4월".into(),
                    "5월".into(),
                    "6월".into(),
                    "7월".into(),
                    "8월".into(),
                    "9월".into(),
                    "10월".into(),
                    "11월".into(),
                    "12월".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A, %e %B %Y г. %X".into(),
                date: "%d.%m.%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "недела".into(),
                    "понеделник".into(),
                    "вторник".into(),
                    "среда".into(),
                    "четврток".into(),
                    "петок".into(),
                    "сабота".into(),
                ],
                short_days: [
                    "нед".into(),
                    "пон".into(),
                    "вто".into(),
                    "сре".into(),
                    "чет".into(),
                    "пет".into(),
                    "саб".into(),
                ],
                months: [
                    "јануари".into(),
                    "февруари".into(),
                    "март".into(),
                    "април".into(),
                    "мај".into(),
                    "јуни".into(),
                    "јули".into(),
                    "август".into(),
                    "септември".into(),
                    "октомври".into(),
                    "ноември".into(),
                    "декември".into(),
                ],
                short_months: [
                    "јан".into(),
                    "фев".into(),
                    "мар".into(),
                    "апр".into(),
                    "мај".into(),
                    "јун".into(),
                    "јул".into(),
                    "авг".into(),
                    "сеп".into(),
                    "окт".into(),
                    "ное".into(),
                    "дек".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A den %d. %B %Y %X".into(),
                date: "%d.%m.%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "søndag".into(),
                    "mandag".into(),
                    "tirsdag".into(),
                    "onsdag".into(),
                    "torsdag".into(),
                    "fredag".into(),
                    "lørdag".into(),
                ],
                short_days: [
                    "søn".into(),
                    "man".into(),
                    "tir".into(),
                    "ons".into(),
                    "tor".into(),
                    "fre".into(),
                    "lør".into(),
                ],
                months: [
                    "januar".into(),
                    "februar".into(),
                    "mars".into(),
                    "april".into(),
                    "mai".into(),
                    "juni".into(),
                    "juli".into(),
                    "august".into(),
                    "september".into(),
                    "oktober".into(),
                    "november".into(),
                    "desember".into(),
                ],
                short_months: [
                    "jan".into(),
                    "feb".into(),
                    "mars".into(),
                    "apr".into(),
                    "mai".into(),
                    "juni".into(),
                    "juli".into(),
                    "aug".into(),
                    "sep".into(),
                    "okt".into(),
                    "nov".into(),
                    "des".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%a %e %B %Y %X".into(),
                date: "%d/%m/%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "zondag".into(),
                    "maandag".into(),
                    "dinsdag".into(),
                    "woensdag".into(),
                    "donderdag".into(),
                    "vrijdag".into(),
                    "zaterdag".into(),
                ],
                short_days: [
                    "zo".into(),
                    "ma".into(),
                    "di".into(),
                    "wo".into(),
                    "do".into(),
                    "vr".into(),
                    "za".into(),
                ],
                months: [
                    "januari".into(),
                    "februari".into(),
                    "maart".into(),
                    "april".into(),
                    "mei".into(),
                    "juni".into(),
                    "juli".into(),
                    "augustus".into(),
                    "september".into(),
                    "oktober".into(),
                    "november".into(),
                    "december".into(),
                ],
                short_months: [
                    "jan".into(),
                    "feb".into(),
                    "mrt".into(),
                    "apr".into(),
                    "mei".into(),
                    "jun".into(),
                    "jul".into(),
                    "aug".into(),
                    "sep".into(),
                    "okt".into(),
                    "nov".into(),
                    "dec".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%a %e %B %Y %X".into(),
                date: "%d-%m-%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "zondag".into(),
                    "maandag".into(),
                    "dinsdag".into(),
                    "woensdag".into(),
                    "donderdag".into(),
                    "vrijdag".into(),
                    "zaterdag".into(),
                ],
                short_days: [
                    "zo".into(),
                    "ma".into(),
                    "di".into(),
                    "wo".into(),
                    "do".into(),
                    "vr".into(),
                    "za".into(),
                ],
                months: [
                    "januari".into(),
                    "februari".into(),
                    "maart".into(),
                    "april".into(),
                    "mei".into(),
                    "juni".into(),
                    "juli".into(),
                    "augustus".into(),
                    "september".into(),
                    "oktober".into(),
                    "november".into(),
                    "december".into(),
                ],
                short_months: [
                    "jan".into(),
                    "feb".into(),
                    "mrt".into(),
                    "apr".into(),
                    "mei".into(),
                    "jun".into(),
                    "jul".into(),
                    "aug".into(),
                    "sep".into(),
                    "okt".into(),
                    "nov".into(),
                    "dec".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A, %e %B %Y, %X".into(),
                date: "%d/%m/%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "Niedziela".into(),
                    "Poniedziałek".into(),
                    "Wtorek".into(),
                    "Środa".into(),
                    "Czwartek".into(),
                    "Piątek".into(),
                    "Sobota".into(),
                ],
                short_days: [
                    "Niedz.".into(),
                    "Pon.".into(),
                    "Wt.".into(),
                    "Śr.".into(),
                    "Czw.".into(),
                    "Pt.".into(),
                    "Sob.".into(),
                ],
                months: [
                    "Styczeń".into(),
                    "Luty".into(),
                    "Marzec".into(),
                    "Kwiecień".into(),
                    "Maj".into(),
                    "Czerwiec".into(),
                    "Lipiec".into(),
                    "Sierpień".into(),
                    "Wrzesień".into(),
                    "Październik".into(),
                    "Listopad".into(),
                    "Grudzień".into(),
                ],
                short_months: [
                    "Stycz.".into(),
                    "Luty".into(),
                    "Marz.".into(),
                    "Kwie.".into(),
                    "Maj".into(),
                    "Czerw.".into(),
                    "Lipc.".into(),
                    "Sierp.".into(),
                    "Wrz.".into(),
                    "Paźdz.".into(),
                    "Listop.".into(),
                    "Grudz.".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A, %e de %B de %Y. %X".into(),
                date: "%d/%m/%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "Domingo".into(),
                    "Segunda".into(),
                    "Terça".into(),
                    "Quarta".into(),
                    "Quinta".into(),
                    "Sexta".into(),
                    "Sábado".into(),
                ],
                short_days: [
                    "Dom".into(),
                    "Seg".into(),
                    "Ter".into(),
                    "Qua".into(),
                    "Qui".into(),
                    "Sex".into(),
                    "Sáb".into(),
                ],
                months: [
                    "Janeiro".into(),
                    "Fevereiro".into(),
                    "Março".into(),
                    "Abril".into(),
                    "Maio".into(),
                    "Junho".into(),
                    "Julho".into(),
                    "Agosto".into(),
                    "Setembro".into(),
                    "Outubro".into(),
                    "Novembro".into(),
                    "Dezembro".into(),
                ],
                short_months: [
                    "Jan".into(),
                    "Fev".into(),
                    "Mar".into(),
                    "Abr".into(),
                    "Mai".into(),
                    "Jun".into(),
                    "Jul".into(),
                    "Ago".into(),
                    "Set".into(),
                    "Out".into(),
                    "Nov".into(),
                    "Dez".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A, %e %B %Y г. %X".into(),
                date: "%d.%m.%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "воскресенье".into(),
                    "понедельник".into(),
                    "вторник".into(),
                    "среда".into(),
                    "четверг".into(),
                    "пятница".into(),
                    "суббота".into(),
                ],
                short_days: [
                    "вс".into(),
                    "пн".into(),
                    "вт".into(),
                    "ср".into(),
                    "чт".into(),
                    "пт".into(),
                    "сб".into(),
                ],
                months: [
                    "января".into(),
                    "февраля".into(),
                    "марта".into(),
                    "апреля".into(),
                    "мая".into(),
                    "июня".into(),
                    "июля".into(),
                    "августа".into(),
                    "сентября".into(),
                    "октября".into(),
                    "ноября".into(),
                    "декабря".into(),
                ],
                short_months: [
                    "янв".into(),
                    "фев".into(),
                    "мар".into(),
                    "апр".into(),
                    "май".into(),
                    "июн".into(),
                    "июл".into(),
                    "авг".into(),
                    "сен".into(),
                    "окт".into(),
                    "ноя".into(),
                    "дек".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A den %d %B %Y %X".into(),
                date: "%Y-%m-%d".into(),
                time: "%H:%M:%S".into(),
                periods: ["fm".into(), "em".into()],
                days: [
                    "Söndag".into(),
                    "Måndag".into(),
                    "Tisdag".into(),
                    "Onsdag".into(),
                    "Torsdag".into(),
                    "Fredag".into(),
                    "Lördag".into(),
                ],
                short_days: [
                    "Sön".into(),
                    "Mån".into(),
                    "Tis".into(),
                    "Ons".into(),
                    "Tor".into(),
                    "Fre".into(),
                    "Lör".into(),
                ],
                months: [
                    "Januari".into(),
                    "Februari".into(),
                    "Mars".into(),
                    "April".into(),
                    "Maj".into(),
                    "Juni".into(),
                    "Juli".into(),
                    "Augusti".into(),
                    "September".into(),
                    "Oktober".into(),
                    "November".into(),
                    "December".into(),
                ],
                short_months: [
                    "Jan".into(),
                    "Feb".into(),
                    "Mar".into(),
                    "Apr".into(),
                    "Maj".into(),
                    "Jun".into(),
                    "Jul".into(),
                    "Aug".into(),
                    "Sep".into(),
                    "Okt".into(),
                    "Nov".into(),
                    "Dec".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%a %e %b %X %Y".into(),
                date: "%d/%m/%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "Pazar".into(),
                    "Pazartesi".into(),
                    "Salı".into(),
                    "Çarşamba".into(),
                    "Perşembe".into(),
                    "Cuma".into(),
                    "Cumartesi".into(),
                ],
                short_days: [
                    "Paz".into(),
                    "Pzt".into(),
                    "Sal".into(),
                    "Çar".into(),
                    "Per".into(),
                    "Cum".into(),
                    "Cmt".into(),
                ],
                months: [
                    "Ocak".into(),
                    "Şubat".into(),
                    "Mart".into(),
                    "Nisan".into(),
                    "Mayıs".into(),
                    "Haziran".into(),
                    "Temmuz".into(),
                    "Ağustos".into(),
                    "Eylül".into(),
                    "Ekim".into(),
                    "Kasım".into(),
                    "Aralık".into(),
                ],
                short_months: [
                    "Oca".into(),
                    "Şub".into(),
                    "Mar".into(),
                    "Nis".into(),
                    "May".into(),
                    "Haz".into(),
                    "Tem".into(),
                    "Ağu".into(),
                    "Eyl".into(),
                    "Eki".into(),
                    "Kas".into(),
                    "Ara".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%A, %e %B %Y р. %X".into(),
                date: "%d.%m.%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["дп".into(), "пп".into()],
                days: [
                    "неділя".into(),
                    "понеділок".into(),
                    "вівторок".into(),
                    "середа".into(),
                    "четвер".into(),
                    "п'ятниця".into(),
                    "субота".into(),
                ],
                short_days: [
                    "нд".into(),
                    "пн".into(),
                    "вт".into(),
                    "ср".into(),
                    "чт".into(),
                    "пт".into(),
                    "сб".into(),
                ],
                months: [
                    "січня".into(),
                    "лютого".into(),
                    "березня".into(),
                    "квітня".into(),
                    "травня".into(),
                    "червня".into(),
                    "липня".into(),
                    "серпня".into(),
                    "вересня".into(),
                    "жовтня".into(),
                    "листопада".into(),
                    "грудня".into(),
                ],
                short_months: [
                    "січ.".into(),
                    "лют.".into(),
                    "бер.".into(),
                    "квіт.".into(),
                    "трав.".into(),
                    "черв.".into(),
                    "лип.".into(),
                    "серп.".into(),
                    "вер.".into(),
                    "жовт.".into(),
                    "лист.".into(),
                    "груд.".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%a, %e %b, %Y %H:%M".into(),
                date: "%d/%m/%Y".into(),
                time: "%H:%M:%S".into(),
                periods: ["AM".into(), "PM".into()],
                days: [
                    "Chủ nhật".into(),
                    "Thứ hai".into(),
                    "Thứ ba".into(),
                    "Thứ tư".into(),
                    "Thứ năm".into(),
                    "Thứ sáu".into(),
                    "Thứ bảy".into(),
                ],
                short_days: [
                    "CN".into(),
                    "Thứ 2".into(),
                    "Thứ 3".into(),
                    "Thứ 4".into(),
                    "Thứ 5".into(),
                    "Thứ 6".into(),
                    "Thứ 7".into(),
                ],
                months: [
                    "Tháng một".into(),
                    "Tháng hai".into(),
                    "Tháng ba".into(),
                    "Tháng tư".into(),
                    "Tháng năm".into(),
                    "Tháng sáu".into(),
                    "Tháng bảy".into(),
                    "Tháng tám".into(),
                    "Tháng chín".into(),
                    "Tháng mười".into(),
                    "Tháng mười một".into(),
                    "Tháng mười hai".into(),
                ],
                short_months: [
                    "Tháng 1".into(),
                    "Tháng 2".into(),
                    "Tháng 3".into(),
                    "Tháng 4".into(),
                    "Tháng 5".into(),
                    "Tháng 6".into(),
                    "Tháng 7".into(),
                    "Tháng 8".into(),
                    "Tháng 9".into(),
                    "Tháng 10".into(),
                    "Tháng 11".into(),
                    "Tháng 12".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%x %A %X".into(),
                date: "%Y年%-m月%-d日".into(),
                time: "%H:%M:%S".into(),
                periods: ["上午".into(), "下午".into()],
                days: [
                    "星期日".into(),
                    "星期一".into(),
                    "星期二".into(),
                    "星期三".into(),
                    "星期四".into(),
                    "星期五".into(),
                    "星期六".into(),
                ],
                short_days: [
                    "周日".into(),
                    "周一".into(),
                    "周二".into(),
                    "周三".into(),
                    "周四".into(),
                    "周五".into(),
                    "周六".into(),
                ],
                months: [
                    "一月".into(),
                    "二月".into(),
                    "三月".into(),
                    "四月".into(),
                    "五月".into(),
                    "六月".into(),
                    "七月".into(),
                    "八月".into(),
                    "九月".into(),
                    "十月".into(),
                    "十一月".into(),
                    "十二月".into(),
                ],
                short_months: [
                    "一月".into(),
                    "二月".into(),
                    "三月".into(),
                    "四月".into(),
                    "五月".into(),
                    "六月".into(),
                    "七月".into(),
                    "八月".into(),
                    "九月".into(),
                    "十月".into(),
                    "十一月".into(),
                    "十二月".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
        (
            &[
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time",
            ],
            time::Definition {
                date_time: "%x %A %X".into(),
                date: "%Y年%-m月%-d日".into(),
                time: "%H:%M:%S".into(),
                periods: ["上午".into(), "下午".into()],
                days: [
                    "星期日".into(),
                    "星期一".into(),
                    "星期二".into(),
                    "星期三".into(),
                    "星期四".into(),
                    "星期五".into(),
                    "星期六".into(),
                ],
                short_days: [
                    "日".into(),
                    "一".into(),
                    "二".into(),
                    "三".into(),
                    "四".into(),
                    "五".into(),
                    "六".into(),
                ],
                months: [
                    "一月".into(),
                    "二月".into(),
                    "三月".into(),
                    "四月".into(),
                    "五月".into(),
                    "六月".into(),
                    "七月".into(),
                    "八月".into(),
                    "九月".into(),
                    "十月".into(),
                    "十一月".into(),
                    "十二月".into(),
                ],
                short_months: [
                    "1月".into(),
                    "2月".into(),
                    "3月".into(),
                    "4月".into(),
                    "5月".into(),
                    "6月".into(),
                    "7月".into(),
                    "8月".into(),
                    "9月".into(),
                    "10月".into(),
                    "11月".into(),
                    "12月".into(),
                ],
                ladder: time::Ladder::D3,
            },
        ),
    ];
    for (keys, definition) in tables {
        assert_eq!(
            keys,
            vec![
                "date",
                "dateTime",
                "days",
                "months",
                "periods",
                "shortDays",
                "shortMonths",
                "time"
            ]
        ); // d3-time-format/test/locale-test.js:10:5
        let _ = time::Locale::new(definition);
    }
}

// d3-time-format/test/defaultLocale-test.js: timeFormat(specifier) defaults to en-US
#[test]
fn timeformat_specifier_defaults_to_en_us() {
    let la = zone("America/Los_Angeles");
    assert_eq!(
        time::Locale::EN_US
            .format("%c")
            .format(local(&la, &[2000.0, 0.0, 1.0]), &la),
        "1/1/2000, 12:00:00 AM"
    ); // d3-time-format/test/defaultLocale-test.js:9:3
}

// d3-time-format/test/defaultLocale-test.js: utcFormat(specifier) defaults to en-US
#[test]
fn utcformat_specifier_defaults_to_en_us() {
    assert_eq!(
        time::Locale::EN_US.format("%c").format(
            millis(utc(&[2000.0, 0.0, 1.0]).as_millisecond() as f64),
            &TimeZone::UTC
        ),
        "1/1/2000, 12:00:00 AM"
    ); // d3-time-format/test/defaultLocale-test.js:17:3
}
