//! The labels of a binned colorbar's edges: as many decimal places as the
//! edges need to read as themselves, never so few that two read alike.

use rustd3_core::locale::Locale;

/// The most places d3-format writes.
const MOST: usize = 20;

/// The labels of `edges` (a binned scale's tick set), `,f` under `locale`
/// at the precision matplotlib's `ScalarFormatter` gives tick labels: the
/// fewest places, from 3 − ⌊log₁₀ range⌋ down, at which every edge lies
/// within a thousandth of the range's power of ten of its label (so
/// 3, 7.25, 9, 14.5, 20 read "3.00", "7.25", … and 0, 20, … 100 need none);
/// then more places while two labels read alike, up to 20.
pub(crate) fn labels(edges: &[f64], locale: &Locale) -> Vec<String> {
    let finite = || edges.iter().copied().filter(|v| v.is_finite());
    let (low, high) = finite().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), v| {
        (a.min(v), b.max(v))
    });
    let spread = high - low;
    let largest = finite().fold(0.0, |m: f64, v| m.max(v.abs()));
    let range = if spread > 0.0 {
        spread
    } else if largest > 0.0 {
        largest
    } else {
        1.0
    };
    let power = range.log10().floor();
    let near = 1e-3 * 10f64.powf(power);
    let start = (3.0 - power).clamp(0.0, MOST as f64) as usize;
    let rounds = |places: usize| {
        let k = 10f64.powi(places as i32);
        finite().all(|v| (v - (v * k).round() / k).abs() < near)
    };
    let mut places = start;
    while places > 0 && rounds(places - 1) {
        places -= 1;
    }
    loop {
        let format = locale
            .number()
            .format(&format!(",.{places}f"))
            .expect("invariant: `,.Nf` parses for N up to 20");
        let texts: Vec<String> = edges.iter().map(|v| format.format(*v)).collect();
        let distinct = texts.windows(2).all(|w| w[0] != w[1]);
        if distinct || places >= MOST {
            return texts;
        }
        places += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(edges: &[f64]) -> Vec<String> {
        labels(edges, &Locale::EN_US)
    }

    #[test]
    fn uneven_edges_keep_their_own_digits() {
        // D25.3 (8): d3's `,f` at the smallest gap read 7.25 as "7".
        assert_eq!(
            read(&[3.0, 7.25, 9.0, 14.5, 20.0]),
            ["3.00", "7.25", "9.00", "14.50", "20.00"]
        );
        assert_eq!(
            read(&[0.001, 0.0025, 0.004]),
            ["0.0010", "0.0025", "0.0040"]
        );
    }

    #[test]
    fn even_and_whole_edges_need_no_more_places_than_matplotlib_gives() {
        assert_eq!(
            read(&[0.0, 20.0, 40.0, 60.0, 80.0, 100.0]),
            ["0", "20", "40", "60", "80", "100"]
        );
        assert_eq!(
            read(&[0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0]),
            ["0.000", "0.333", "0.667", "1.000"]
        );
        assert_eq!(read(&[0.0, 1.0]), ["0", "1"]);
        assert_eq!(read(&[0.0, 12_500.0, 25_000.0]), ["0", "12,500", "25,000"]);
    }

    #[test]
    fn edges_closer_than_the_precision_get_more_places() {
        // matplotlib reads 1 and 1.0004 both "1.0".
        assert_eq!(
            read(&[1.0, 1.0004, 1.5, 2.0]),
            ["1.0000", "1.0004", "1.5000", "2.0000"]
        );
        assert_eq!(read(&[5.0]), ["5"]);
        assert!(read(&[]).is_empty());
    }
}
