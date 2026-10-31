//! [`Histogram`]: bars counting values; [`Binned`], what a histogram
//! counts on a scale, and [`Bins`], how it bins numbers.

use rustd3_core::array::{self, bin::Thresholds};
use rustd3_core::scale::{self, Band, Continuous, Point};

use crate::check::Issue;
use crate::figure::Fits;
use crate::hatch::Hatch;
use crate::hit;
use crate::legend::Swatch;
use crate::list::{self, Clip, List};
use crate::mark::bar::Across;
use crate::mark::color::Color;
use crate::mark::{Ctx, Mark, dodge, reach, rect, within};
use crate::scale::{Kind, Paint, Position, ScaleRef};

/// The most bins `Bins::Nice` asks d3's ticks for: they round a count
/// up to at most 1.59 times as many ticks, which stays under core's cap of
/// 10,000 ticks (past it, ticks give none and the histogram one bin).
const NICE: usize = 6_000;

/// The most bins `Bins::Count` makes.
const COUNT: usize = 10_000;

/// What a histogram counts on a position scale: implemented for band and
/// point scales, which count each key (the wheel figure's bars per speed
/// bin), and for continuous scales over numbers, which bin the values.
/// One function, [`histogram`], serves both.
pub trait Binned: Position {
    /// The bars of a histogram of `values`, in order: per bar the first
    /// and last domain value it spans and its count. Band and point scales
    /// count each key (both ends are the key), in the order keys first
    /// appear; continuous scales drop values that are not finite and bin
    /// the rest by `bins` (both ends are a bin's edges), by
    /// [`Bins::Nice`] with Sturges' count without.
    fn bars(values: &[Self::Domain], bins: Option<Bins>) -> Vec<([Self::Domain; 2], f64)>;
}

/// How a histogram bins numbers. Weights, densities, cumulative counts and
/// explicit edges are the helpers' (`helpers::stats`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bins {
    /// d3.bin's nice bins: thresholds from d3's `ticks(min, max, n)` over
    /// the values' extent niced to the outer ticks, the last bin closed.
    /// The default is Sturges' count, ⌈log₂ n⌉ + 1 for n values. At most
    /// 6,000 is asked for (d3's ticks round it to at most about 9,500
    /// bins), at least 1.
    Nice(usize),
    /// `n` equal-width bins over `[min, max]`, numpy's and matplotlib's
    /// rule (one unit wide around values that are all equal), the last
    /// bin closed; at most 10,000, at least 1.
    Count(usize),
}

/// Bars counting values: per key of a band or point x scale, or per bin
/// of numbers on a continuous one, each from 0 up to its count along y.
pub struct Histogram<X: Binned, Y: Position<Domain = f64>> {
    values: Vec<<X as Kind>::Domain>,
    bars: Vec<([<X as Kind>::Domain; 2], f64)>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    color: Color,
    hatch: Option<Hatch>,
    dodge: bool,
}

impl<X: Binned, Y: Position<Domain = f64>> std::fmt::Debug for Histogram<X, Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Histogram").finish_non_exhaustive()
    }
}

/// A histogram of `values` on `x` with its counts on `y`, in the theme
/// palette's first color: one bar per key on band and point scales, per
/// bin of the finite values ([`Bins::Nice`] with Sturges' count by
/// default) on continuous ones. The bars stand on 0, which is held: the y
/// margin stops there. The values are collected when the mark is built.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::Figure;
/// use rustd3_figure::mark::{Bins, histogram};
///
/// let mut fig = Figure::new();
/// let (x, count) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
/// let values = [1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 4.7, 5.1];
/// fig.panel(0, 0).mark(histogram(values, x, count).bins(Bins::Count(4)));
/// ```
///
/// Counts go on a position scale: a palette as y does not compile
/// (E0277, `Ordinal<Source, Rgb>: Position` is not satisfied).
///
/// ```compile_fail,E0277
/// # use rustd3_figure::{Figure, mark::histogram};
/// # use rustd3_core::{scale, scheme};
/// #[derive(Clone, PartialEq)]
/// struct Bin(u8);
/// #[derive(Clone, PartialEq)]
/// enum Source { Hub, Bearing }
///
/// let mut fig = Figure::new();
/// let speed = fig.scale(scale::band((0..20).map(Bin)));
/// let color = fig.scale(scale::ordinal([Source::Hub, Source::Bearing], scheme::COLORBLIND));
/// let bins = [Bin(3), Bin(4), Bin(4)];
/// fig.panel(0, 0).mark(histogram(bins, speed, color)); // E0277
/// ```
/// ```
/// # use rustd3_figure::{Figure, mark::histogram};
/// # use rustd3_core::{scale, scheme};
/// #[derive(Clone, PartialEq)]
/// struct Bin(u8);
/// #[derive(Clone, PartialEq)]
/// enum Source { Hub, Bearing }
///
/// let mut fig = Figure::new();
/// let speed = fig.scale(scale::band((0..20).map(Bin)));
/// let count = fig.fit(scale::linear());
/// let color = fig.scale(scale::ordinal([Source::Hub, Source::Bearing], scheme::COLORBLIND));
/// let bins = [Bin(3), Bin(4), Bin(4)];
/// fig.panel(0, 0).mark(histogram(bins, speed, count).fill(color, Source::Hub));
/// ```
pub fn histogram<X, Y>(
    values: impl IntoIterator<Item = <X as Kind>::Domain>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> Histogram<X, Y>
where
    X: Position + Binned,
    Y: Position<Domain = f64>,
{
    let values: Vec<_> = values.into_iter().collect();
    Histogram {
        bars: X::bars(&values, None),
        values,
        x,
        y,
        color: Color::Theme,
        hatch: None,
        dodge: false,
    }
}

impl<X: Binned, Y: Position<Domain = f64>> Histogram<X, Y> {
    /// How numbers are binned; band and point scales count keys and ignore
    /// it.
    #[must_use]
    pub fn bins(self, b: Bins) -> Self {
        Self {
            bars: X::bars(&self.values, Some(b)),
            ..self
        }
    }

    /// Fills the bars with `palette`'s color for `key`, looked up when the
    /// figure draws; a key the palette lacks draws the theme's gray.
    #[must_use]
    pub fn fill<C: Paint>(self, palette: ScaleRef<C>, key: C::Domain) -> Self {
        Self {
            color: Color::keyed(palette, key),
            ..self
        }
    }

    /// Hatches every bar with `h`, after its fill; legend swatches of the
    /// fill key repeat it.
    #[must_use]
    pub fn hatch(self, h: Hatch) -> Self {
        Self {
            hatch: Some(h),
            ..self
        }
    }

    /// Places the bars in their fill key's slot of each band or bin, split
    /// into one part per palette key in the palette's order, so that the
    /// bars of several histograms stand side by side.
    #[must_use]
    pub fn dodge(self) -> Self {
        Self {
            dodge: true,
            ..self
        }
    }
}

impl<'a, X: Binned, Y: Position<Domain = f64>> Mark<'a> for Histogram<X, Y> {
    fn fit(&self, fits: &mut Fits) {
        fits.across(self.x);
        for (ends, count) in &self.bars {
            for end in ends {
                fits.add(self.x, end);
            }
            fits.add(self.y, count);
        }
        fits.add(self.y, &0.0);
        fits.stick(self.y, 0.0);
        self.color.report(Swatch::Square, self.hatch.as_ref(), fits);
    }

    /// One `Fill` command per bar with area, through the rectangle code of
    /// [`rect()`](crate::mark::rect): across its key's band (a width
    /// centred on a point key) or its bin's edges, dodged when asked, from
    /// 0 up to its count; each followed by its hatch when set.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let paint = list::brush(self.color.resolve(ctx));
        for (_, [x0, x1], [base, top]) in self.placed(ctx) {
            let bar = kurbo::Rect::new(x0, top, x1, base);
            if let Some(r) = rect::fill(list, ctx, bar, &paint) {
                super::hatch(self.hatch.as_ref(), Clip::Rect(r), r, ctx, list);
            }
        }
    }

    /// Per bar, the centre of its top when it lies in the plot area, with
    /// the bar's index (in the order of [`Binned::bars`]).
    fn hits(&self, ctx: &Ctx<'_>, sink: &mut hit::Sink) {
        let area = ctx.get_rect().with_origin(kurbo::Point::ORIGIN);
        for (i, [x0, x1], [_, top]) in self.placed(ctx) {
            let p = kurbo::Point::new((x0 + x1) / 2.0, top);
            if within(area, p) {
                sink.push(i, p);
            }
        }
    }

    /// A fill key the palette lacks; values outside a band or point domain.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        super::unknown(&self.color, ctx, issues);
        let Some(sx) = ctx.scale(self.x) else {
            return;
        };
        let outside: f64 = self
            .bars
            .iter()
            .filter(|([k, _], _)| sx.position(k).is_none())
            .map(|(_, count)| count)
            .sum();
        super::outside_count(self.x.get_index(), outside as usize, issues);
    }
}

impl<X: Binned, Y: Position<Domain = f64>> Histogram<X, Y> {
    /// Each bar placed, with its index: across its key's band (a width
    /// centred on a point key) or its bin's edges, dodged when asked, and
    /// from 0 up to its count along y, clamped to the guard band; bars
    /// that cannot be placed are skipped, all of them when 0 cannot be.
    fn placed(&self, ctx: &Ctx<'_>) -> Vec<(usize, [f64; 2], [f64; 2])> {
        let (Some(sx), Some(sy)) = (ctx.x(self.x), ctx.y(self.y)) else {
            return Vec::new();
        };
        let w = ctx.get_rect().width();
        let band = ctx.band();
        let Some(base) = reach(&sy, 0.0, [band.y0, band.y1]) else {
            return Vec::new();
        };
        let across = Across::new(&sx, self.bars.iter().map(|([k, _], _)| k), None, w);
        let slot = self.color.slot(ctx).filter(|_| self.dodge);
        self.bars
            .iter()
            .enumerate()
            .filter_map(|(i, ([first, last], count))| {
                let start = sx.position(first).filter(|p| p.is_finite())?;
                let end = sx.position(last).filter(|p| p.is_finite())? + sx.bandwidth();
                let span = if end == start {
                    across.band(&sx, first)?
                } else {
                    (start, end - start)
                };
                let top = reach(&sy, *count, [band.y0, band.y1])?;
                let (start, width) = dodge(span, slot);
                Some((i, [start, start + width], [base, top]))
            })
            .collect()
    }
}

impl<K: PartialEq + Clone + 'static> Binned for Band<K> {
    /// One bar per key.
    fn bars(values: &[K], _bins: Option<Bins>) -> Vec<([K; 2], f64)> {
        keys(values)
    }
}

impl<K: PartialEq + Clone + 'static> Binned for Point<K> {
    /// One bar per key.
    fn bars(values: &[K], _bins: Option<Bins>) -> Vec<([K; 2], f64)> {
        keys(values)
    }
}

impl<T: scale::Transform + 'static, const N: usize> Binned for Continuous<T, f64, N> {
    /// One bar per bin of the finite values, in the data's units whatever
    /// the transform.
    fn bars(values: &[f64], bins: Option<Bins>) -> Vec<([f64; 2], f64)> {
        let finite: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
        if finite.is_empty() {
            return Vec::new();
        }
        let thresholds = match bins {
            None => Thresholds::Sturges,
            Some(Bins::Nice(n)) => Thresholds::Count(n.clamp(1, NICE)),
            Some(Bins::Count(n)) => return equal(&finite, n.clamp(1, COUNT)),
        };
        array::bin()
            .thresholds(thresholds)
            .layout(&finite)
            .into_iter()
            .map(|b| ([b.x0, b.x1], b.indices.len() as f64))
            .collect()
    }
}

/// Each key of `values` once, in the order it first appears, with how
/// often it appears.
fn keys<K: PartialEq + Clone>(values: &[K]) -> Vec<([K; 2], f64)> {
    let mut bars: Vec<([K; 2], f64)> = Vec::new();
    for v in values {
        match bars.iter_mut().find(|([k, _], _)| k == v) {
            Some((_, count)) => *count += 1.0,
            None => bars.push(([v.clone(), v.clone()], 1.0)),
        }
    }
    bars
}

/// `n` equal-width bins over the extent of `values` (finite, not empty),
/// as numpy's `histogram` makes them: edges `min + i·step` with the last at
/// `max`, a value's bin from its share of the extent, corrected by a step
/// where that lands across an edge, the last bin closed.
fn equal(values: &[f64], n: usize) -> Vec<([f64; 2], f64)> {
    let [lo, hi] = array::extent(values.iter().copied()).expect("invariant: finite values");
    let (lo, hi) = if lo == hi {
        (lo - 0.5, hi + 0.5)
    } else {
        (lo, hi)
    };
    let step = (hi - lo) / n as f64;
    let edges: Vec<f64> = (0..=n)
        .map(|i| if i == n { hi } else { i as f64 * step + lo })
        .collect();
    let mut counts = vec![0.0; n];
    for &v in values {
        let mut i = (((v - lo) / (hi - lo)) * n as f64) as usize;
        i = i.min(n - 1);
        if v < edges[i] {
            i -= 1;
        } else if i != n - 1 && v >= edges[i + 1] {
            i += 1;
        }
        counts[i] += 1.0;
    }
    edges
        .windows(2)
        .zip(counts)
        .map(|(e, count)| ([e[0], e[1]], count))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A linear scale's bars.
    fn linear(values: &[f64], bins: Option<Bins>) -> Vec<([f64; 2], f64)> {
        <scale::Linear as Binned>::bars(values, bins)
    }

    /// The section's statistical data, n = 15.
    const X: [f64; 15] = [
        1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 4.7, 5.1, 5.6, 6.2, 7.9, 9.5, 14.0,
    ];

    #[test]
    fn default_bins_are_d3s_nice_sturges_bins() {
        let bars = linear(&X, None);
        let edges: Vec<[f64; 2]> = bars.iter().map(|(e, _)| *e).collect();
        let counts: Vec<f64> = bars.iter().map(|(_, c)| *c).collect();
        let expected: Vec<[f64; 2]> = (0..8)
            .map(|i| [2.0 * i as f64, 2.0 * i as f64 + 2.0])
            .collect();
        assert_eq!(edges, expected);
        assert_eq!(counts, [1.0, 5.0, 5.0, 2.0, 1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn count_bins_are_numpys_equal_width_bins() {
        let bars = linear(&X, Some(Bins::Count(10)));
        let counts: Vec<f64> = bars.iter().map(|(_, c)| *c).collect();
        assert_eq!(counts, [2.0, 3.0, 4.0, 3.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0]);
        assert_eq!(bars[0].0[0], 1.2);
        assert!((bars[0].0[1] - 2.48).abs() < 1e-12);
        assert_eq!(bars[9].0[1], 14.0);
        let flat = linear(&[3.0, 3.0], Some(Bins::Count(2)));
        assert_eq!(flat, [([2.5, 3.0], 0.0), ([3.0, 3.5], 2.0)]);
    }

    #[test]
    fn nice_bins_never_collapse_past_the_tick_cap() {
        let values: Vec<f64> = (0..1000).map(|i| f64::from(i) * 0.37).collect();
        for n in [1, 10, 6_000, 9_000, 50_000, usize::MAX] {
            let bars = linear(&values, Some(Bins::Nice(n)));
            let total: f64 = bars.iter().map(|(_, c)| c).sum();
            assert_eq!(total, 1000.0, "Nice({n})");
            if n >= NICE {
                assert!(bars.len() > 1_000, "Nice({n}) gave {} bins", bars.len());
            }
            assert!(bars.len() <= 10_000, "Nice({n}) gave {} bins", bars.len());
        }
        let most = linear(&values, Some(Bins::Nice(usize::MAX)));
        let capped = linear(&values, Some(Bins::Nice(NICE)));
        assert_eq!(most, capped);
        let count = linear(&values, Some(Bins::Count(usize::MAX)));
        assert_eq!(count.len(), 10_000);
        assert_eq!(linear(&values, Some(Bins::Count(0))).len(), 1);
    }

    #[test]
    fn values_that_are_not_finite_are_dropped() {
        let bars = linear(&[1.0, f64::NAN, f64::INFINITY, 2.0], Some(Bins::Count(1)));
        assert_eq!(bars, [([1.0, 2.0], 2.0)]);
        assert!(linear(&[f64::NAN], None).is_empty());
    }

    #[test]
    fn keys_are_counted_in_the_order_they_first_appear() {
        let bars = <Band<&str> as Binned>::bars(&["b", "a", "b", "c", "b"], Some(Bins::Count(2)));
        assert_eq!(
            bars,
            [(["b", "b"], 3.0), (["a", "a"], 1.0), (["c", "c"], 1.0)]
        );
        assert_eq!(<Point<i32> as Binned>::bars(&[1, 1], None), [([1, 1], 2.0)]);
    }
}
