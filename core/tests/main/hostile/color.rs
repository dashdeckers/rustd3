//! Colour parsers on random strings, colour spaces and interpolators at t
//! outside [0, 1] and at NaN, schemes at any t, and d3-array's statistics,
//! bins and bisection over hostile values. Nothing panics.

use rustd3_core::array::{self, bin::Thresholds};
use rustd3_core::color::{Cubehelix, Hcl, Hsl, Lab, Rgb};
use rustd3_core::ease::{Ease, Mode};
use rustd3_core::{interpolate, scheme};
use rustd3_testkit::hostile::{self, Rng};

const CASES: usize = 600;

/// Characters of CSS colours.
const CSS: &str = "#0123456789abcdefABCDEF(),.%+- rgbhslatrednvy";

fn t(rng: &mut Rng) -> f64 {
    if rng.chance(0.5) {
        rng.f64()
    } else {
        rng.unit() * 3.0 - 1.0
    }
}

fn rgb(rng: &mut Rng) -> Rgb {
    Rgb::new(rng.f64(), rng.f64(), rng.f64()).with_opacity(rng.f64())
}

#[test]
fn colours_parse_any_text_and_convert_any_value() {
    hostile::run("colours", 0xC0_1E, CASES, |rng| {
        let text = rng.string(CSS, 24);
        let _ = text.parse::<Rgb>();
        let _ = text.parse::<Hsl>();
        let _ = text.parse::<Lab>();
        let _ = text.parse::<Hcl>();
        let _ = text.parse::<Cubehelix>();
        let c = rgb(rng);
        let k = rng.f64();
        let _ = c.brighter(k).darker(k).clamp();
        c.format_hex();
        let _ = c.format_hex8();
        let _ = c.to_string();
        let _ = c.displayable();
        let hsl = Hsl::from(c);
        let _ = hsl.brighter(k).clamp().format_hsl();
        let _ = Rgb::from(hsl);
        let lab = Lab::from(c);
        let _ = Rgb::from(lab.brighter(k));
        let hcl = Hcl::from(c);
        let _ = Rgb::from(hcl.darker(k));
        let cube = Cubehelix::from(c);
        let _ = Rgb::from(cube.brighter(k));
    });
}

#[test]
fn interpolators_take_any_t() {
    hostile::run("interpolators", 0x17_E2, CASES, |rng| {
        let (a, b) = (rgb(rng), rgb(rng));
        let x = t(rng);
        interpolate::number(rng.f64(), rng.f64())(x);
        interpolate::round(rng.f64(), rng.f64())(x);
        interpolate::hue(rng.f64(), rng.f64())(x);
        interpolate::rgb(a, b)(x);
        interpolate::rgb_gamma(rng.f64(), a, b)(x);
        interpolate::hsl(Hsl::from(a), Hsl::from(b))(x);
        interpolate::hsl_long(Hsl::from(a), Hsl::from(b))(x);
        interpolate::lab(Lab::from(a), Lab::from(b))(x);
        interpolate::hcl(Hcl::from(a), Hcl::from(b))(x);
        interpolate::hcl_long(Hcl::from(a), Hcl::from(b))(x);
        interpolate::cubehelix(rng.f64(), Cubehelix::from(a), Cubehelix::from(b))(x);
        interpolate::cubehelix_long(rng.f64(), Cubehelix::from(a), Cubehelix::from(b))(x);
        let values = rng.values(6);
        interpolate::basis(&values)(x);
        let colors: Vec<Rgb> = (0..rng.small(5)).map(|_| rgb(rng)).collect();
        interpolate::rgb_basis(&colors)(x);
        interpolate::quantize(
            |v| v,
            if rng.chance(0.5) {
                rng.small(50)
            } else {
                *rng.pick(&[1 << 31, usize::MAX / 2, usize::MAX])
            },
        );
        let f = interpolate::piecewise(&[rng.f64(), rng.f64(), rng.f64()], |a, b| {
            interpolate::number(*a, *b)
        });
        f(x);
        let _ = scheme::viridis(x);
        let _ = scheme::magma(x);
        let _ = scheme::turbo(x);
        let _ = scheme::rainbow(x);
        let _ = scheme::sinebow(x);
        scheme::blues(x);
        scheme::rd_bu(x);
        let mode = *rng.pick(&[Mode::In, Mode::Out, Mode::InOut]);
        for e in [
            Ease::Linear,
            Ease::Quad(mode),
            Ease::Cubic(mode),
            Ease::Sin(mode),
            Ease::Exp(mode),
            Ease::Circle(mode),
            Ease::Bounce(mode),
            Ease::Poly {
                mode,
                exponent: rng.f64(),
            },
            Ease::Back {
                mode,
                overshoot: rng.f64(),
            },
            Ease::Elastic {
                mode,
                amplitude: rng.f64(),
                period: rng.f64(),
            },
        ] {
            let _ = e.apply(x);
        }
    });
}

#[test]
fn array_statistics_and_bins_take_any_values() {
    hostile::run("arrays", 0xA7_7A, CASES, |rng| {
        let values = rng.values(40);
        let it = || values.iter().copied();
        array::extent(it());
        array::min(it());
        array::max(it());
        array::sum(it());
        array::mean(it());
        array::median(it());
        array::variance(it());
        array::deviation(it());
        array::count(it());
        array::quantile(it(), rng.f64());
        let mut sorted = values.clone();
        sorted.sort_by(f64::total_cmp);
        let _ = array::quantile_sorted(&sorted, rng.f64());
        let _ = array::bisect_left(&values, rng.f64());
        let _ = array::bisect_right(&values, rng.f64());
        let thresholds = match rng.below(5) {
            0 => Thresholds::Sturges,
            1 => Thresholds::Scott,
            2 => Thresholds::FreedmanDiaconis,
            3 => Thresholds::Count(rng.count()),
            _ => Thresholds::Values(rng.values(8)),
        };
        let mut bin = array::bin().thresholds(thresholds);
        if rng.chance(0.4) {
            bin = bin.domain([rng.f64(), rng.f64()]);
        }
        let buckets = bin.layout(&values);
        let placed: usize = buckets.iter().map(|b| b.indices.len()).sum();
        assert!(
            placed <= values.len(),
            "{placed} placed of {}",
            values.len()
        );
        let (start, step) = (rng.f64(), rng.f64());
        let stop = if rng.chance(0.5) {
            start + step * rng.below(100) as f64
        } else {
            rng.f64()
        };
        let n = ((stop - start) / step).ceil();
        if n.is_nan() || n <= 1e6 {
            let _ = array::range(start, stop, step);
        }
        let _ = array::bin::sturges(&values);
        let [lo, hi] = [rng.f64(), rng.f64()];
        let _ = array::bin::scott(&values, lo, hi);
        let _ = array::bin::freedman_diaconis(&values, lo, hi);
    });
}
