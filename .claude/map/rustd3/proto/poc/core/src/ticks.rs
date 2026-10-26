//! Port of d3-array's ticks, tickIncrement, tickStep and nice, written with plain `std` math.

const E10: f64 = 7.0710678118654755; // sqrt(50)
const E5: f64 = 3.1622776601683795; // sqrt(10)
const E2: f64 = std::f64::consts::SQRT_2;

/// First and last tick index, and the increment (negative means "divide by its inverse").
fn tick_spec(start: f64, stop: f64, count: f64) -> (f64, f64, f64) {
    // JS's Math.max propagates NaN; f64::max would silently pick 0.
    let step = (stop - start) / if count.is_nan() { count } else { count.max(0.0) };
    let power = step.log10().floor();
    let error = step / 10f64.powf(power);
    let factor = if error >= E10 {
        10.0
    } else if error >= E5 {
        5.0
    } else if error >= E2 {
        2.0
    } else {
        1.0
    };
    let (mut i1, mut i2, inc);
    if power < 0.0 {
        let inv = 10f64.powf(-power) / factor;
        i1 = (start * inv).round();
        i2 = (stop * inv).round();
        if i1 / inv < start {
            i1 += 1.0;
        }
        if i2 / inv > stop {
            i2 -= 1.0;
        }
        inc = -inv;
    } else {
        inc = 10f64.powf(power) * factor;
        i1 = (start / inc).round();
        i2 = (stop / inc).round();
        if i1 * inc < start {
            i1 += 1.0;
        }
        if i2 * inc > stop {
            i2 -= 1.0;
        }
    }
    if i2 < i1 && (0.5..2.0).contains(&count) {
        return tick_spec(start, stop, count * 2.0);
    }
    (i1, i2, inc)
}

/// About `count` evenly spaced, nicely rounded values between `start` and `stop` (inclusive).
pub fn ticks(start: f64, stop: f64, count: f64) -> Vec<f64> {
    if !(count > 0.0) {
        return Vec::new();
    }
    if start == stop {
        return vec![start];
    }
    let reverse = stop < start;
    let (i1, i2, inc) = if reverse { tick_spec(stop, start, count) } else { tick_spec(start, stop, count) };
    if !(i2 >= i1) {
        return Vec::new();
    }
    let n = (i2 - i1 + 1.0) as usize;
    (0..n)
        .map(|i| {
            let k = if reverse { i2 - i as f64 } else { i1 + i as f64 };
            if inc < 0.0 { k / -inc } else { k * inc }
        })
        .collect()
}

/// The tick increment; negative values are the inverse of a fractional step.
pub fn tick_increment(start: f64, stop: f64, count: f64) -> f64 {
    tick_spec(start, stop, count).2
}

/// The signed difference between adjacent ticks.
pub fn tick_step(start: f64, stop: f64, count: f64) -> f64 {
    let reverse = stop < start;
    let inc = if reverse { tick_increment(stop, start, count) } else { tick_increment(start, stop, count) };
    let sign = if reverse { -1.0 } else { 1.0 };
    sign * if inc < 0.0 { 1.0 / -inc } else { inc }
}

/// Extends `[start, stop]` to align with tick boundaries.
pub fn nice(mut start: f64, mut stop: f64, count: f64) -> [f64; 2] {
    let mut prestep = None;
    loop {
        let step = tick_increment(start, stop, count);
        if prestep == Some(step) || step == 0.0 || !step.is_finite() {
            return [start, stop];
        } else if step > 0.0 {
            start = (start / step).floor() * step;
            stop = (stop / step).ceil() * step;
        } else if step < 0.0 {
            start = (start * step).ceil() / step;
            stop = (stop * step).floor() / step;
        }
        prestep = Some(step);
    }
}
