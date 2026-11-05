//! [`Ramp`]: how a color scale becomes a colorbar's strip, and [`Samples`],
//! the strip's colors.

use rustd3_core::color::Rgb;
use rustd3_core::scale::{
    self,
    Continuous,
    Diverging,
    Quantile,
    Quantize,
    Sequential,
    Threshold,
    Ticks,
    transform,
};

/// The rows of an interpolator's strip: d3's table schemes have 256
/// entries and pick `table[⌊256 t⌋]`, so each entry gets one row.
const ROWS: usize = 256;

/// The color of a bin a threshold scale's range has none for:
/// transparent, as a mark skips a value its color scale cannot paint.
const NONE: Rgb = Rgb {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    opacity: 0.0,
};

/// How a color scale becomes a colorbar's strip, from its low end
/// (position 0 of its [`Ticks::position`]) to its high end (position 1).
/// Implemented for [`Sequential`] and [`Diverging`] over colors under any
/// transform, continuous scales over colors (`Linear<Rgb>`, piecewise or
/// under a transform), and [`Quantize`], [`Quantile`] and [`Threshold`]
/// over colors. Ordinal palettes are not ramps: their keys get a
/// [`legend`](crate::legend::legend).
pub trait Ramp: Ticks<Domain = f64, Step = f64> {
    /// The strip's colors.
    fn samples(&self) -> Samples;
}

/// A colorbar strip's colors, from the scale's low end to its high end.
/// Plain data.
#[derive(Debug, Clone, PartialEq)]
pub enum Samples {
    /// Equal rows in transformed space: row `i` of `n` is the scale's
    /// color `(i + 0.5)/n` of the way from its low end. Interpolators give
    /// 256 rows, so d3's 256-entry table schemes give each entry one row.
    Even(Vec<Rgb>),
    /// One row per bin, from the low end, at the bins' widths along the
    /// bar (their edges at the scale's tick positions).
    Bins {
        /// Each bin's color.
        colors: Vec<Rgb>,
        /// The edges between neighbouring bins in domain units, one fewer
        /// than the bins: thresholds or quantiles. The first and last bins
        /// end at the bar's ends.
        edges: Vec<f64>,
    },
}

/// The interpolator at each of 256 row centres.
impl<T: scale::Transform> Ramp for Sequential<Rgb, T> {
    fn samples(&self) -> Samples {
        let unit = self
            .clone()
            .transform(transform::Identity)
            .domain([0.0, 1.0]);
        Samples::Even(centres().map(|t| unit.apply(t)).collect())
    }
}

/// The interpolator at each of 256 row centres, the middle stop at the
/// strip's middle.
impl<T: scale::Transform> Ramp for Diverging<Rgb, T> {
    fn samples(&self) -> Samples {
        let unit = self
            .clone()
            .transform(transform::Identity)
            .domain([0.0, 0.5, 1.0]);
        Samples::Even(centres().map(|t| unit.apply(t)).collect())
    }
}

/// The colors at each of 256 row centres between the end stops in
/// transformed space, inner stops at their positions.
impl<T: scale::Transform, const N: usize> Ramp for Continuous<T, Rgb, N> {
    fn samples(&self) -> Samples {
        let stops = self
            .get_domain()
            .map(|d| Ticks::position(self, &d, 0.0).unwrap_or(f64::NAN));
        // The same stops over their positions along the strip: inverting
        // a row centre gives the value there.
        let unit = self.clone().range(stops);
        Samples::Even(centres().map(|t| self.apply(unit.invert(t))).collect())
    }
}

/// Its colors between its thresholds.
impl Ramp for Quantize<Rgb> {
    fn samples(&self) -> Samples {
        Samples::Bins {
            colors: self.get_range().to_vec(),
            edges: self.thresholds(),
        }
    }
}

/// Its colors between the sample's quantiles.
impl Ramp for Quantile<Rgb> {
    fn samples(&self) -> Samples {
        Samples::Bins {
            colors: self.get_range().to_vec(),
            edges: self.quantiles().to_vec(),
        }
    }
}

/// One bin more than thresholds, each in its range color; a bin the range
/// has no color for is transparent.
impl Ramp for Threshold<Rgb> {
    fn samples(&self) -> Samples {
        let edges = self.get_domain().to_vec();
        let range = self.get_range();
        let colors = (0..=edges.len())
            .map(|i| range.get(i).copied().unwrap_or(NONE))
            .collect();
        Samples::Bins { colors, edges }
    }
}

/// The centres of 256 equal rows in [0, 1].
fn centres() -> impl Iterator<Item = f64> {
    (0..ROWS).map(|i| (i as f64 + 0.5) / ROWS as f64)
}

#[cfg(test)]
mod tests {
    use rustd3_core::scheme;

    use super::*;

    fn even(samples: Samples) -> Vec<Rgb> {
        match samples {
            Samples::Even(colors) => colors,
            Samples::Bins { .. } => panic!("bins"),
        }
    }

    #[test]
    fn a_table_scheme_gives_each_entry_one_row() {
        let s = scale::sequential([0.0, 80.0], scheme::viridis);
        let rows = even(s.samples());
        assert_eq!(rows.len(), 256);
        assert_eq!(rows[0].format_hex(), "#440154");
        assert_eq!(rows[128].format_hex(), "#21918c");
        assert_eq!(rows[255].format_hex(), "#fde725");
        for (i, row) in rows.iter().enumerate() {
            assert_eq!(*row, scheme::viridis(i as f64 / 256.0), "row {i}");
        }
    }

    #[test]
    fn transforms_and_reversed_domains_leave_the_strip_to_the_interpolator() {
        let log =
            scale::sequential([1.0, 1000.0], scheme::magma).transform(transform::Log::new(10.0));
        let reversed = scale::sequential([80.0, 0.0], scheme::magma);
        assert_eq!(even(log.samples()), even(reversed.samples()));
        let d = scale::diverging([-1.0, 0.0, 4.0], scheme::rd_bu);
        let rows = even(d.samples());
        assert_eq!(rows[128], scheme::rd_bu(128.5 / 256.0));
        assert_eq!(rows[0], scheme::rd_bu(0.5 / 256.0));
    }

    #[test]
    fn a_color_range_samples_between_its_stops() {
        let red = Rgb::new(255.0, 0.0, 0.0);
        let white = Rgb::new(255.0, 255.0, 255.0);
        let blue = Rgb::new(0.0, 0.0, 255.0);
        let l = scale::piecewise([0.0, 25.0, 100.0], [red, white, blue]);
        let rows = even(l.samples());
        for (i, row) in rows.iter().enumerate() {
            let at = l.apply((i as f64 + 0.5) / 256.0 * 100.0);
            let close = [(row.r, at.r), (row.g, at.g), (row.b, at.b)];
            assert!(close.iter().all(|(a, b)| (a - b).abs() < 1e-9), "row {i}");
        }
        // Rows 63 and 64 lie either side of white, at 24.8 and 25.2.
        assert!(rows[63].r == 255.0 && rows[63].b < 255.0);
        assert!(rows[64].b == 255.0 && rows[64].r < 255.0);
    }

    #[test]
    fn binned_scales_give_one_row_per_bin_and_their_inner_edges() {
        let blues5 = ["#eff3ff", "#bdd7e7", "#6baed6", "#3182bd", "#08519c"]
            .map(|c| c.parse::<Rgb>().expect("a color"));
        let q = scale::quantize([0.0, 100.0], blues5);
        assert_eq!(
            q.samples(),
            Samples::Bins {
                colors: blues5.to_vec(),
                edges: vec![20.0, 40.0, 60.0, 80.0]
            }
        );
        let red = Rgb::new(255.0, 0.0, 0.0);
        let white = Rgb::new(255.0, 255.0, 255.0);
        let t = scale::threshold([0.0, 1.0], [red, white]);
        assert_eq!(
            t.samples(),
            Samples::Bins {
                colors: vec![red, white, NONE],
                edges: vec![0.0, 1.0]
            }
        );
        let quantile = scale::quantile(
            [3.0, 6.0, 7.0, 8.0, 8.0, 10.0, 13.0, 15.0, 16.0, 20.0],
            [red, white, red, white],
        );
        let Samples::Bins { edges, .. } = quantile.samples() else {
            panic!("bins");
        };
        assert_eq!(edges, [7.25, 9.0, 14.5]);
    }
}
