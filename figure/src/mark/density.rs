//! [`DensityY`] and [`DensityX`]: a Gaussian kernel density estimate drawn
//! along y or along x.

use kurbo::{BezPath, Point};

use crate::check::Issue;
use crate::figure::Fits;
use crate::list::{self, Command, List};
use crate::mark::color::Color;
use crate::mark::{Ctx, Error, Mark, guard, polygon, positive, reach, stroke};
use crate::scale::{Paint, Position, ScaleRef};
use crate::stats::Kde;

/// Points of the curve: seaborn's `gridsize`.
const POINTS: usize = 200;

/// How far past the data the curve runs, in bandwidths: seaborn's `cut`.
const CUT: f64 = 3.0;

/// The fill's opacity.
const FILL: f64 = 0.35;

/// A kernel density estimate along y: the curve over the data's values on
/// `y`, its width on `w` peak-normalized to 1, filled from `w = 0` at 0.35
/// opacity and outlined 0.75 pt wide.
pub struct DensityY<Y: Position<Domain = f64>, W: Position<Domain = f64>> {
    density: Density,
    y: ScaleRef<Y>,
    w: ScaleRef<W>,
}

/// A kernel density estimate along x: the curve over the data's values on
/// `x`, its height on `h` peak-normalized to 1, filled from `h = 0` at 0.35
/// opacity and outlined 0.75 pt wide.
pub struct DensityX<X: Position<Domain = f64>, H: Position<Domain = f64>> {
    density: Density,
    x: ScaleRef<X>,
    h: ScaleRef<H>,
}

/// What both orientations hold: the values, their estimate and its curve.
struct Density {
    /// The finite values, collected when the mark is built.
    values: Vec<f64>,
    /// An explicit bandwidth, instead of Scott's.
    bandwidth: Option<f64>,
    /// What the bandwidth is multiplied by.
    adjust: f64,
    /// The curve: positions along the value axis and the density there
    /// over its peak; empty when there is no estimate.
    curve: Vec<(f64, f64)>,
    color: Color,
}

/// A kernel density estimate of `values` along `y`, Gaussian with Scott's
/// bandwidth as scipy and seaborn compute it, over 200 points from 3
/// bandwidths below the least value to 3 above the greatest (seaborn's
/// `kdeplot`). The curve is over the data, not the view, so zooming does
/// not reshape it; its width on `w` is peak-normalized to 1. The values
/// enter fitting on `y` (the curve's tails do not); `w` spans 0 to 1, held
/// at 0. Fewer than two finite values, or values that are all equal, draw
/// nothing, and [`Figure::check`](crate::Figure::check) reports
/// [`Error::Density`].
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::Figure;
/// use rustd3_figure::mark::density_y;
///
/// let mut fig = Figure::new();
/// let y = fig.fit(scale::linear());
/// let width = fig.scale(scale::linear().domain([0.0, 1.0]));
/// fig.panel(0, 0).mark(density_y([1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0], y, width));
/// ```
pub fn density_y<Y, W>(
    values: impl IntoIterator<Item = f64>,
    y: ScaleRef<Y>,
    w: ScaleRef<W>,
) -> DensityY<Y, W>
where
    Y: Position<Domain = f64>,
    W: Position<Domain = f64>,
{
    DensityY {
        density: Density::new(values),
        y,
        w,
    }
}

/// A kernel density estimate of `values` along `x`, its height on `h`:
/// [`density_y`] turned on its side.
pub fn density_x<X, H>(
    values: impl IntoIterator<Item = f64>,
    x: ScaleRef<X>,
    h: ScaleRef<H>,
) -> DensityX<X, H>
where
    X: Position<Domain = f64>,
    H: Position<Domain = f64>,
{
    DensityX {
        density: Density::new(values),
        x,
        h,
    }
}

impl Density {
    /// The finite `values` and their curve with Scott's bandwidth.
    fn new(values: impl IntoIterator<Item = f64>) -> Density {
        Density {
            values: values.into_iter().filter(|v| v.is_finite()).collect(),
            bandwidth: None,
            adjust: 1.0,
            curve: Vec::new(),
            color: Color::Theme,
        }
        .traced()
    }

    /// The same with its curve computed for its bandwidth.
    fn traced(self) -> Density {
        let curve = self.estimate().map_or_else(Vec::new, |kde| {
            let grid = kde.grid(POINTS, CUT);
            let peak = grid.iter().map(|(_, d)| *d).fold(0.0, f64::max);
            grid.into_iter().map(|(at, d)| (at, d / peak)).collect()
        });
        Density { curve, ..self }
    }

    /// The estimate at the mark's bandwidth; `None` for fewer than two
    /// values or values that are all equal.
    fn estimate(&self) -> Option<Kde> {
        Some(Kde::new(self.values.iter().copied())?.rescaled(self.bandwidth, self.adjust))
    }

    /// Fits the values on the value axis and 0 to 1, held at 0, on the
    /// other; nothing when there is no estimate.
    fn fit<V: Position<Domain = f64>, S: Position<Domain = f64>>(
        &self,
        along: ScaleRef<V>,
        size: ScaleRef<S>,
        fits: &mut Fits,
    ) {
        if self.curve.is_empty() {
            return;
        }
        for v in &self.values {
            fits.add(along, v);
        }
        fits.add(size, &0.0);
        fits.add(size, &1.0);
        fits.stick(size, 0.0);
    }

    /// A `Fill` of the region between the curve and size 0 at 0.35 opacity
    /// and a `Stroke` of its outline, 0.75 pt, placed by `at` from a
    /// position along the value axis and a size; nothing without an
    /// estimate or when the region falls outside the guard band.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List, at: impl Fn(f64, f64) -> Option<Point>) {
        let placed: Vec<(f64, Point)> = self
            .curve
            .iter()
            .filter_map(|&(v, d)| Some((v, at(v, d)?)))
            .collect();
        let (Some(&(first, _)), Some(&(last, _))) = (placed.first(), placed.last()) else {
            return;
        };
        let (Some(start), Some(end)) = (at(first, 0.0), at(last, 0.0)) else {
            return;
        };
        let curve = placed.into_iter().map(|(_, p)| p);
        let outline: Vec<Point> = [start].into_iter().chain(curve).chain([end]).collect();
        let mut path = BezPath::new();
        let band = guard::band(ctx.rect().width(), ctx.rect().height());
        polygon(&mut path, &outline, band);
        if path.elements().is_empty() {
            return;
        }
        let color = self.color.resolve(ctx);
        list.commands.push(Command::Fill {
            path: path.clone(),
            paint: list::brush(color.with_opacity(color.opacity * FILL)),
        });
        list.commands.push(Command::Stroke {
            path,
            style: stroke(ctx.theme().density_width(), &[]),
            paint: list::brush(color),
        });
    }

    /// Fewer than two finite values or values that are all equal; a fill
    /// key the palette lacks.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        if self.curve.is_empty() {
            issues.push(Issue::Mark {
                panel: ctx.panel(),
                mark: ctx.mark(),
                error: Error::Density {
                    values: self.values.len(),
                },
            });
        }
        super::unknown(&self.color, ctx, issues);
    }
}

/// The builders both orientations share, for `$density` with the
/// generics `$g`.
macro_rules! builders {
    ($density:ident, [$($g:tt)*]) => {
        impl<$($g)*> $density<A, B> {
            /// Fills the region with `palette`'s color for `key` (at 0.35
            /// opacity, the outline opaque), looked up when the figure
            /// draws; a key the palette lacks draws the theme's gray.
            pub fn fill<C: Paint>(mut self, palette: ScaleRef<C>, key: C::Domain) -> Self {
                self.density.color = Color::keyed(palette, key);
                self
            }

            /// An explicit bandwidth `h`, the kernel's standard deviation in
            /// the values' units, instead of Scott's; [`adjust`](Self::adjust)
            /// still multiplies it. A bandwidth that is not finite and
            /// positive leaves it as it was.
            pub fn bandwidth(mut self, h: f64) -> Self {
                self.density.bandwidth = positive(h).or(self.density.bandwidth);
                self.density = self.density.traced();
                self
            }

            /// Multiplies the bandwidth by `f`, seaborn's `bw_adjust`: above 1
            /// smoother, below 1 more detailed. A factor that is not finite and
            /// positive leaves it as it was.
            pub fn adjust(mut self, f: f64) -> Self {
                self.density.adjust = positive(f).unwrap_or(self.density.adjust);
                self.density = self.density.traced();
                self
            }
        }
    };
}

builders!(DensityY, [A: Position<Domain = f64>, B: Position<Domain = f64>]);
builders!(DensityX, [A: Position<Domain = f64>, B: Position<Domain = f64>]);

impl<'a, Y: Position<Domain = f64>, W: Position<Domain = f64>> Mark<'a> for DensityY<Y, W> {
    fn fit(&self, fits: &mut Fits) {
        fits.across(self.w);
        self.density.fit(self.y, self.w, fits);
    }

    /// A `Fill` at 0.35 opacity and a `Stroke` of the region between the
    /// curve and `w = 0`, clipped to the guard band.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let (sy, sw) = (ctx.y(self.y), ctx.x(self.w));
        let band = guard::band(ctx.rect().width(), ctx.rect().height());
        let at = |v: f64, d: f64| {
            let x = reach(&sw, d, [band.x0, band.x1])?;
            let y = sy.position(&v).filter(|p| p.is_finite())?;
            Some(Point::new(x, y))
        };
        self.density.draw(ctx, list, at);
    }

    /// Fewer than two finite values or values that are all equal; a fill
    /// key the palette lacks.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        self.density.issues(ctx, issues);
    }
}

impl<'a, X: Position<Domain = f64>, H: Position<Domain = f64>> Mark<'a> for DensityX<X, H> {
    fn fit(&self, fits: &mut Fits) {
        fits.across(self.x);
        self.density.fit(self.x, self.h, fits);
    }

    /// A `Fill` at 0.35 opacity and a `Stroke` of the region between the
    /// curve and `h = 0`, clipped to the guard band.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let (sx, sh) = (ctx.x(self.x), ctx.y(self.h));
        let band = guard::band(ctx.rect().width(), ctx.rect().height());
        let at = |v: f64, d: f64| {
            let x = sx.position(&v).filter(|p| p.is_finite())?;
            let y = reach(&sh, d, [band.y0, band.y1])?;
            Some(Point::new(x, y))
        };
        self.density.draw(ctx, list, at);
    }

    /// Fewer than two finite values or values that are all equal; a fill
    /// key the palette lacks.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        self.density.issues(ctx, issues);
    }
}
