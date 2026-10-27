//! [`Notation`]: offset and multiplier notation, matplotlib 3.11.2's
//! `ScalarFormatter` offset text and order of magnitude for a continuous
//! axis whose domain sits far from zero, or is tiny or huge.

use rustd3_core::format;

/// Domains whose centre is more than this many spans from zero get an
/// offset.
const FAR: f64 = 1e4;

/// Largest magnitudes at or below 10 to this power get a multiplier
/// (matplotlib's `axes.formatter.limits`).
const TINY: i32 = -5;

/// Largest magnitudes at or above 10 to this power get a multiplier.
const HUGE: i32 = 6;

/// How an axis writes its tick labels when its domain is far from zero, or
/// tiny or huge: as distances from an offset, or in units of a power of
/// ten, with the offset or the power at the axis's end.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Notation {
    /// Labels show `t − o`.
    Offset(f64),
    /// Labels show `t / 10^m`.
    Multiplier(i32),
}

impl Notation {
    /// The notation of a continuous domain `[a, b]`, or `None`.
    ///
    /// An offset when `|a + b| / 2 / |b − a| > 10⁴`: with `p` the smallest
    /// integer for which `10^p ≥ |b − a|`, the multiple of `10^p` in
    /// `[min(a, b) − 10^p, max(a, b)]` with the most trailing zeros (ties:
    /// nearest the centre). Otherwise a multiplier `10^m`, `m =
    /// floor(log10(max(|a|, |b|)))`, when `m ≤ −5` or `m ≥ 6`.
    pub(crate) fn of([a, b]: [f64; 2]) -> Option<Notation> {
        let span = (b - a).abs();
        let centre = (a + b) / 2.0;
        if !(span > 0.0 && span.is_finite() && centre.is_finite()) {
            return None;
        }
        if centre.abs() / span > FAR {
            return offset(a.min(b), a.max(b), centre, span).map(Notation::Offset);
        }
        let largest = a.abs().max(b.abs());
        let m = magnitude(largest);
        (m <= TINY || m >= HUGE).then_some(Notation::Multiplier(m))
    }

    /// What tick value `t` reads as.
    pub(crate) fn apply(self, t: f64) -> f64 {
        match self {
            Notation::Offset(o) => t - o,
            // Powers of ten up to 10²² are exact, so the label is one
            // rounding from the value.
            Notation::Multiplier(m) if m < 0 => t * 10f64.powi(-m),
            Notation::Multiplier(m) => t / 10f64.powi(m),
        }
    }

    /// The label at the axis's end, written in `locale` as tick labels are
    /// (its decimal mark, minus sign and numerals): `+` or the minus and the
    /// offset's shortest mantissa and exponent (`+1e9`, `+1.2345e4`,
    /// `−1e9`), or the multiplier (`1e−7`, `1e7`).
    pub(crate) fn label(self, locale: &format::Locale) -> String {
        let (mantissa, digits, exponent, sign) = match self {
            Notation::Offset(o) => {
                let text = format!("{o:e}");
                let (mantissa, exponent) =
                    text.split_once('e').expect("invariant: {:e} writes an e");
                let digits = mantissa.bytes().filter(u8::is_ascii_digit).count();
                let mantissa = mantissa.parse().expect("invariant: {:e} writes a decimal");
                let exponent = exponent.parse().expect("invariant: {:e} writes an integer");
                (mantissa, digits, exponent, "+")
            }
            Notation::Multiplier(m) => (1.0, 1, m, ""),
        };
        let written = |specifier: &str, value: f64| {
            let format = locale.format(specifier);
            format
                .expect("invariant: an r or d specifier parses")
                .format(value)
        };
        let mantissa = written(&format!("{sign}.{digits}r"), mantissa);
        format!("{mantissa}e{}", written("d", f64::from(exponent)))
    }
}

/// The offset of a domain from `low` to `high` around `centre`, `span`
/// wide; `None` when its candidates cannot be counted.
fn offset(low: f64, high: f64, centre: f64, span: f64) -> Option<f64> {
    let mut p = span.log10().ceil() as i32;
    while power(p - 1) >= span {
        p -= 1;
    }
    while power(p) < span {
        p += 1;
    }
    let unit = power(p);
    // A multiple k·10^p, computed as d3 computes ticks: k / 10^−p below 1.
    let at = |k: f64| if p < 0 { k / power(-p) } else { k * unit };
    let first = ((low - unit) / unit).ceil();
    let last = (high / unit).floor();
    if !(first.is_finite() && last.is_finite() && last - first <= 100.0) {
        return None;
    }
    let mut best: Option<(u32, f64)> = None;
    let mut k = first;
    while k <= last {
        let candidate = at(k);
        let zeros = zeros(k);
        let better = best.is_none_or(|(z, o)| {
            zeros > z || (zeros == z && (candidate - centre).abs() < (o - centre).abs())
        });
        if better {
            best = Some((zeros, candidate));
        }
        k += 1.0;
    }
    best.map(|(_, o)| o)
}

/// `10^p` as one correctly rounded operation where it can be.
fn power(p: i32) -> f64 {
    10f64.powi(p)
}

/// The integer `m` with `10^m ≤ x < 10^(m+1)`, for `x > 0`; `i32::MIN` for
/// zero.
fn magnitude(x: f64) -> i32 {
    if x <= 0.0 || !x.is_finite() {
        return i32::MIN;
    }
    let mut m = x.log10().floor() as i32;
    while power(m) > x {
        m -= 1;
    }
    while power(m + 1) <= x {
        m += 1;
    }
    m
}

/// The trailing zeros of integer `k` in decimal; zero has as many as any
/// candidate can.
fn zeros(k: f64) -> u32 {
    if k == 0.0 {
        return u32::MAX;
    }
    let mut k = k;
    let mut n = 0;
    while k % 10.0 == 0.0 && n < 400 {
        k /= 10.0;
        n += 1;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domains_far_from_zero_take_the_offset_with_the_most_trailing_zeros() {
        let rows = [
            ([1e9, 1e9 + 1.0], "+1e9"),
            ([12345.0, 12345.3], "+1.2345e4"),
            ([-1e9 - 1.0, -1e9], "−1e9"),
        ];
        for (domain, label) in rows {
            let notation = Notation::of(domain).expect("an offset");
            assert!(matches!(notation, Notation::Offset(_)), "{notation:?}");
            assert_eq!(notation.label(&format::Locale::NEUTRAL), label);
        }
        assert_eq!(
            Notation::of([12345.0, 12345.3]),
            Some(Notation::Offset(12345.0)),
            "12344 and 12345 tie; 12345 is nearer the centre"
        );
    }

    #[test]
    fn domains_near_zero_and_moderate_ones_keep_their_labels() {
        assert_eq!(Notation::of([1000.0, 1000.5]), None);
        assert_eq!(Notation::of([0.0, 12000.0]), None);
        assert_eq!(Notation::of([-100.0, 100.0]), None);
        assert_eq!(Notation::of([5.0, 5.0]), None);
        assert_eq!(Notation::of([0.0, f64::INFINITY]), None);
    }

    #[test]
    fn tiny_and_huge_domains_take_a_multiplier() {
        let tiny = Notation::of([0.0, 3e-7]).expect("a multiplier");
        assert_eq!(tiny, Notation::Multiplier(-7));
        assert_eq!(tiny.label(&format::Locale::NEUTRAL), "1e−7");
        assert_eq!(tiny.apply(3e-7), 3.0);
        let huge = Notation::of([0.0, 2.5e7]).expect("a multiplier");
        assert_eq!(
            (huge, huge.label(&format::Locale::NEUTRAL)),
            (Notation::Multiplier(7), "1e7".into())
        );
        assert_eq!(Notation::of([0.0, 9.9e5]), None);
        assert_eq!(Notation::of([0.0, 1e-4]), None);
        assert_eq!(Notation::of([0.0, 9.9e-5]), Some(Notation::Multiplier(-5)));
    }

    #[test]
    fn labels_take_the_locales_decimal_mark_minus_and_numerals() {
        let de = format::Locale::DE_DE;
        let offset = Notation::of([12345.0, 12345.3]).expect("an offset");
        assert_eq!(offset.label(&de), "+1,2345e4");
        let hyphen = format::Locale::new(format::Definition {
            minus: "-".into(),
            ..format::Definition::default()
        });
        let below = Notation::of([-1e9 - 1.0, -1e9]).expect("an offset");
        assert_eq!(below.label(&hyphen), "-1e9");
        assert_eq!(Notation::Multiplier(-7).label(&hyphen), "1e-7");
        assert_eq!(Notation::Multiplier(-7).label(&de), "1e\u{2212}7");
        let arabic = format::Locale::new(format::Definition {
            decimal: "\u{66b}".into(),
            numerals: Some(["٠", "١", "٢", "٣", "٤", "٥", "٦", "٧", "٨", "٩"].map(Into::into)),
            ..format::Definition::default()
        });
        assert_eq!(offset.label(&arabic), "+١٫٢٣٤٥e٤");
    }

    #[test]
    fn labels_keep_the_offsets_shortest_digits() {
        let neutral = format::Locale::NEUTRAL;
        for (o, label) in [
            (1.5e9, "+1.5e9"),
            (-2.0e-3, "−2e−3"),
            (0.1 + 0.2, "+3.0000000000000004e−1"),
        ] {
            assert_eq!(Notation::Offset(o).label(&neutral), label);
        }
    }

    #[test]
    fn magnitudes_are_exact_at_powers_of_ten() {
        assert_eq!([1e6, 999_999.999_999_9, 1e-5].map(magnitude), [6, 5, -5]);
    }
}
