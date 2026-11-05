//! d3's quickselect (Floyd–Rivest, after mourner/quickselect, ISC license,
//! Copyright 2018 Vladimir Agafonkin), on values without NaN.

/// Reorders `values` so that `values[k]` is the value that sorting would
/// put there, with none greater before it and none less after it.
pub(crate) fn quickselect(values: &mut [f64], k: usize) {
    if let Some(right) = values.len().checked_sub(1) {
        select(values, k as isize, 0, right as isize);
    }
}

/// quickselect.js's loop over `values[left..=right]`. Indices are signed
/// as in d3, whose `j - 1` may reach −1 and end the loop.
fn select(values: &mut [f64], k: isize, mut left: isize, mut right: isize) {
    if !(left <= k && k <= right) {
        return;
    }
    let at = |i: isize| i as usize;
    while right > left {
        if right - left > 600 {
            let n = (right - left + 1) as f64;
            let m = (k - left + 1) as f64;
            let z = n.ln();
            let s = 0.5 * (2.0 * z / 3.0).exp();
            let sign = if m - n / 2.0 < 0.0 { -1.0 } else { 1.0 };
            let sd = 0.5 * (z * s * (n - s) / n).sqrt() * sign;
            let kf = k as f64;
            let new_left = left.max((kf - m * s / n + sd).floor() as isize);
            let new_right = right.min((kf + (n - m) * s / n + sd).floor() as isize);
            select(values, k, new_left, new_right);
        }
        let t = values[at(k)];
        let (mut i, mut j) = (left, right);
        values.swap(at(left), at(k));
        if values[at(right)] > t {
            values.swap(at(left), at(right));
        }
        while i < j {
            values.swap(at(i), at(j));
            i += 1;
            j -= 1;
            while values[at(i)] < t {
                i += 1;
            }
            while values[at(j)] > t {
                j -= 1;
            }
        }
        if values[at(left)] == t {
            values.swap(at(left), at(j));
        } else {
            j += 1;
            values.swap(at(j), at(right));
        }
        if j <= k {
            left = j + 1;
        }
        if k <= j {
            right = j - 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_position_of_a_large_shuffle_selects_its_sorted_value() {
        // A linear congruential shuffle of 0..1500 with repeats, past the
        // 600-element threshold of the sampling step.
        let shuffled: Vec<f64> = (0..1500u64)
            .map(|i| ((i * 7919 + 13) % 1009) as f64)
            .collect();
        let mut sorted = shuffled.clone();
        sorted.sort_by(f64::total_cmp);
        for k in [0, 1, 499, 750, 1001, 1498, 1499] {
            let mut values = shuffled.clone();
            quickselect(&mut values, k);
            assert_eq!(values[k], sorted[k]);
            assert!(values[..k].iter().all(|&v| v <= sorted[k]));
            assert!(values[k + 1..].iter().all(|&v| v >= sorted[k]));
        }
    }
}
