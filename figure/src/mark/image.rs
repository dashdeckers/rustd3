//! [`Image`]: a grid of values drawn as a raster through a color handle,
//! one pixel per cell; [`Grid`], its values and the rectangle they cover.

use std::sync::Arc;

use peniko::{Blob, ImageAlphaType, ImageData, ImageFormat};

use crate::figure::Fits;
use crate::list::{Command, List};
use crate::mark::{Ctx, Error, Mark};
use crate::scale::{Paint, Position, ScaleRef};
use crate::text;

/// A raster heatmap: one pixel per cell of a [`Grid`], colored through a
/// handle and stretched over the grid's extent with nearest sampling.
pub struct Image<X, Y, C>
where
    X: Position<Domain = f64>,
    Y: Position<Domain = f64>,
    C: Paint<Domain = f64>,
{
    grid: Grid,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    color: ScaleRef<C>,
}

/// Values in `cols × rows` cells, row-major with row 0 at the extent's
/// first y, and the data rectangle the cells cover, `[0, cols] × [0,
/// rows]` unless given.
#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    cols: usize,
    rows: usize,
    values: Vec<f64>,
    x: [f64; 2],
    y: [f64; 2],
}

/// Which cells of one axis are drawn and where: the cells between `first`
/// and `last` (counted from the axis' low px end) span `[lo, hi]` px, and
/// cell `k` there is the grid's `k` or, when the axis runs against px,
/// its `n − 1 − k`.
struct Span {
    first: usize,
    last: usize,
    lo: f64,
    hi: f64,
    reversed: bool,
    n: usize,
}

/// `grid` drawn on `x` and `y` through the color handle `color`: an image
/// of one pixel per cell, `cols × rows`, placed at the device rectangle of
/// the grid's extent with nearest sampling, so cells stay sharp as
/// colorbars need. Cells are opaque unless their color has alpha; a cell
/// whose value has no color (NaN, a value the scale does not take) is
/// transparent. Row 0 sits at the extent's first y, at the bottom when y
/// runs up. The extent's edges enter fitting and hold the margins, so the
/// grid fills the plot area as matplotlib's `imshow` does; the values fit
/// a fitted color handle. Only the cells within the guard band around the
/// plot area are drawn.
///
/// ```
/// use rustd3_core::{scale, scheme};
/// use rustd3_figure::Figure;
/// use rustd3_figure::mark::{Grid, image};
///
/// let values: Vec<f64> = (0..12).map(f64::from).collect();
/// let grid = Grid::new(4, 3, values)?.extent([0.0, 2.0], [-1.0, 1.0]);
/// let mut fig = Figure::new();
/// let (x, y) = (fig.fit(scale::linear()), fig.fit(scale::linear()));
/// let heat = fig.fit(scale::sequential([0.0, 1.0], scheme::viridis));
/// fig.panel(0, 0).mark(image(grid, x, y, heat));
/// # Ok::<(), rustd3_figure::mark::Error>(())
/// ```
#[must_use]
pub fn image<X, Y, C>(
    grid: Grid,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
    color: ScaleRef<C>,
) -> Image<X, Y, C>
where
    X: Position<Domain = f64>,
    Y: Position<Domain = f64>,
    C: Paint<Domain = f64>,
{
    Image { grid, x, y, color }
}

impl Grid {
    /// `cols × rows` cells holding `values`, row-major, row 0 at `y[0]`;
    /// [`Error::GridLength`] with the lengths when the values do not fill
    /// the cells exactly.
    ///
    /// ```
    /// use rustd3_figure::mark::{Error, Grid};
    ///
    /// assert!(Grid::new(2, 2, vec![0.0; 4]).is_ok());
    /// assert_eq!(
    ///     Grid::new(2, 3, vec![0.0; 4]),
    ///     Err(Error::GridLength { cols: 2, rows: 3, len: 4 })
    /// );
    /// ```
    pub fn new(cols: usize, rows: usize, values: Vec<f64>) -> Result<Self, Error> {
        if cols.checked_mul(rows) != Some(values.len()) {
            return Err(Error::GridLength {
                cols,
                rows,
                len: values.len(),
            });
        }
        Ok(Grid {
            cols,
            rows,
            values,
            x: [0.0, cols as f64],
            y: [0.0, rows as f64],
        })
    }

    /// The data rectangle the cells cover: column 0 at `x[0]`, row 0 at
    /// `y[0]`; `[0, cols] × [0, rows]` by default. An end that is not
    /// finite leaves the extent as it was.
    #[must_use]
    pub fn extent(self, x: [f64; 2], y: [f64; 2]) -> Self {
        if x.iter().chain(&y).all(|v| v.is_finite()) {
            Grid { x, y, ..self }
        } else {
            self
        }
    }
}

impl<'a, X, Y, C> Mark<'a> for Image<X, Y, C>
where
    X: Position<Domain = f64>,
    Y: Position<Domain = f64>,
    C: Paint<Domain = f64>,
{
    fn fit(&self, fits: &mut Fits) {
        fits.across(self.x);
        for v in self.grid.x {
            fits.add(self.x, &v);
            fits.stick(self.x, v);
        }
        for v in self.grid.y {
            fits.add(self.y, &v);
            fits.stick(self.y, v);
        }
        for v in &self.grid.values {
            fits.add(self.color, v);
        }
    }

    /// One `Image` command of the cells within the guard band, unsmoothed,
    /// its edges on device pixels on raster targets; nothing when the
    /// extent cannot be placed or has no area.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let (Some(sx), Some(sy)) = (ctx.x(self.x), ctx.y(self.y)) else {
            return;
        };
        let band = ctx.band();
        let across =
            ends(&sx, self.grid.x).and_then(|p| Span::new(p, self.grid.cols, [band.x0, band.x1]));
        let up =
            ends(&sy, self.grid.y).and_then(|p| Span::new(p, self.grid.rows, [band.y0, band.y1]));
        let (Some(across), Some(up)) = (across, up) else {
            return;
        };
        let rect = ctx
            .snap()
            .rect(kurbo::Rect::new(across.lo, up.lo, across.hi, up.hi));
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return;
        }
        let Some(palette) = ctx.scale(self.color) else {
            return;
        };
        let (width, height) = (across.last - across.first, up.last - up.first);
        let mut rgba = Vec::with_capacity(width * height * 4);
        for k in up.first..up.last {
            let row = up.cell(k);
            for j in across.first..across.last {
                let v = self.grid.values[row * self.grid.cols + across.cell(j)];
                rgba.extend(palette.paint(&v).map_or([0; 4], text::channels));
            }
        }
        let side = |n: usize| {
            u32::try_from(n).expect("invariant: an image side within the guard band fits u32")
        };
        list.commands.push(Command::Image {
            rect,
            image: ImageData {
                data: Blob::new(Arc::new(rgba)),
                format: ImageFormat::Rgba8,
                alpha_type: ImageAlphaType::Alpha,
                width: side(width),
                height: side(height),
            },
            smooth: false,
        });
    }
}

impl Span {
    /// The cells of an axis of `n` cells whose ends lie at `[p0, p1]` px
    /// that reach into `[lo, hi]`; `None` when none do or the axis has no
    /// length.
    fn new([p0, p1]: [f64; 2], n: usize, [lo, hi]: [f64; 2]) -> Option<Span> {
        let (start, end) = (p0.min(p1), p0.max(p1));
        let step = (end - start) / n as f64;
        if n == 0 || step <= 0.0 || !step.is_finite() || end <= lo || start >= hi {
            return None;
        }
        let first = ((lo - start) / step).floor().clamp(0.0, n as f64) as usize;
        let last = ((hi - start) / step).ceil().clamp(0.0, n as f64) as usize;
        (first < last).then_some(Span {
            first,
            last,
            lo: start + first as f64 * step,
            hi: start + last as f64 * step,
            reversed: p0 > p1,
            n,
        })
    }

    /// The grid's cell at position `k` from the low px end.
    fn cell(&self, k: usize) -> usize {
        if self.reversed { self.n - 1 - k } else { k }
    }
}

/// Where both ends of `[a, b]` lie on `scale`, when both are finite.
fn ends<S: Position<Domain = f64>>(scale: &S, [a, b]: [f64; 2]) -> Option<[f64; 2]> {
    Some([scale.position(&a)?, scale.position(&b)?]).filter(|p| p.iter().all(|v| v.is_finite()))
}
