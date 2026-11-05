//! [`BarY`] and [`BarX`]: bars from a base or over explicit spans, along y
//! or along x.

use kurbo::Point;

use crate::check::Issue;
use crate::figure::Fits;
use crate::hatch::Hatch;
use crate::hit;
use crate::legend::Swatch;
use crate::list::{self, Clip, List};
use crate::mark::color::{Alpha, Color};
use crate::mark::{Ctx, Mark, dodge, positive, reach, rect, within};
use crate::scale::{Kind, Paint, Position, ScaleRef};

/// The share of the gap between neighboring keys a bar takes on a scale
/// without bands: matplotlib's default bar width of 0.8.
const SHARE: f64 = 0.8;

/// Vertical bars: one filled rectangle per row, across its x key's band
/// and from its base (or its span's first end) to its value along y.
pub struct BarY<X: Position, Y: Position<Domain = f64>> {
    bars: Bars<<X as Kind>::Domain>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
}

impl<X: Position, Y: Position<Domain = f64>> std::fmt::Debug for BarY<X, Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BarY").finish_non_exhaustive()
    }
}

/// Horizontal bars: one filled rectangle per row, across its y key's band
/// and from its base (or its span's first end) to its value along x.
pub struct BarX<X: Position<Domain = f64>, Y: Position> {
    bars: Bars<<Y as Kind>::Domain>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
}

impl<X: Position<Domain = f64>, Y: Position> std::fmt::Debug for BarX<X, Y> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BarX").finish_non_exhaustive()
    }
}

/// What both orientations hold: rows of a key and a value or a span, and
/// how they are filled, hatched and placed.
struct Bars<K> {
    rows: Rows<K>,
    color: Color,
    hatch: Option<Hatch>,
    dodge: bool,
    width: Option<f64>,
}

/// A bar mark's rows.
enum Rows<K> {
    /// `bar_y`, `bar_x`: values from a held base.
    Values { rows: Vec<(K, f64)>, base: f64 },
    /// `bar_y_spans`, `bar_x_spans`: both ends given, only an end at 0 held.
    Spans(Vec<(K, [f64; 2])>),
}

/// Where bars sit across their key axis: in each key's band on band
/// scales, and elsewhere in a width centered on the key.
pub(super) struct Across {
    /// The bars' width in px.
    width: f64,
    /// Whether the scale has bands, whose start `position` gives.
    banded: bool,
}

/// Vertical bars from the base y = 0 to each value of `data`, pairs of an
/// x key and a value, in the theme palette's first color. The base is
/// held, so the y margin stops there.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::Figure;
/// use rustd3_figure::mark::bar_y;
///
/// let mut fig = Figure::new();
/// let x = fig.scale(scale::band(["a", "b", "c"]).padding(0.1));
/// let y = fig.fit(scale::linear());
/// fig.panel(0, 0).mark(bar_y([("a", 3.0), ("b", 1.5), ("c", 2.0)], x, y));
/// ```
pub fn bar_y<X, Y>(
    data: impl IntoIterator<Item = (<X as Kind>::Domain, f64)>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> BarY<X, Y>
where
    X: Position,
    Y: Position<Domain = f64>,
{
    BarY {
        bars: Bars::new(Rows::Values {
            rows: data.into_iter().collect(),
            base: 0.0,
        }),
        x,
        y,
    }
}

/// Horizontal bars from the base x = 0 to each value of `data`, pairs of
/// a value and a y key, in the theme palette's first color. The base is
/// held, so the x margin stops there.
pub fn bar_x<X, Y>(
    data: impl IntoIterator<Item = (f64, <Y as Kind>::Domain)>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> BarX<X, Y>
where
    X: Position<Domain = f64>,
    Y: Position,
{
    let rows = data.into_iter().map(|(v, k)| (k, v)).collect();
    BarX {
        bars: Bars::new(Rows::Values { rows, base: 0.0 }),
        x,
        y,
    }
}

/// One vertical bar from y0 to y1 per row `(x, [y0, y1])`, so
/// `shape::stack`'s spans plug in directly. Both ends enter fitting. An
/// end exactly 0 is held (matplotlib's sticky edge), so stacked bars stand
/// on the axis; no other end is held.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::Figure;
/// use rustd3_figure::mark::bar_y_spans;
///
/// let mut fig = Figure::new();
/// let x = fig.scale(scale::band(["a", "b"]).padding(0.1));
/// let y = fig.fit(scale::linear());
/// fig.panel(0, 0).mark(bar_y_spans([("a", [1.0, 3.0]), ("b", [-1.0, 0.5])], x, y));
/// ```
pub fn bar_y_spans<X, Y>(
    rows: impl IntoIterator<Item = (<X as Kind>::Domain, [f64; 2])>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> BarY<X, Y>
where
    X: Position,
    Y: Position<Domain = f64>,
{
    BarY {
        bars: Bars::new(Rows::Spans(rows.into_iter().collect())),
        x,
        y,
    }
}

/// One horizontal bar from x0 to x1 per row `([x0, x1], y)`. Both ends
/// enter fitting; an end exactly 0 is held, and no other end is.
pub fn bar_x_spans<X, Y>(
    rows: impl IntoIterator<Item = ([f64; 2], <Y as Kind>::Domain)>,
    x: ScaleRef<X>,
    y: ScaleRef<Y>,
) -> BarX<X, Y>
where
    X: Position<Domain = f64>,
    Y: Position,
{
    let rows = rows.into_iter().map(|(span, k)| (k, span)).collect();
    BarX {
        bars: Bars::new(Rows::Spans(rows)),
        x,
        y,
    }
}

impl<K> Bars<K> {
    /// `rows`, in the theme palette's first color, not dodged.
    fn new(rows: Rows<K>) -> Self {
        Bars {
            rows,
            color: Color::Theme,
            hatch: None,
            dodge: false,
            width: None,
        }
    }

    /// Each row's key and the two ends of its bar along the value axis.
    fn rows(&self) -> Box<dyn Iterator<Item = (&K, [f64; 2])> + '_> {
        match &self.rows {
            Rows::Values { rows, base } => Box::new(rows.iter().map(|(k, v)| (k, [*base, *v]))),
            Rows::Spans(rows) => Box::new(rows.iter().map(|(k, span)| (k, *span))),
        }
    }

    /// Fits the keys on `key` and both ends on `value`, holding the base of
    /// rows from a base and 0 when a span ends exactly there.
    fn fit<S: Position<Domain = K>, V: Position<Domain = f64>>(
        &self,
        key: ScaleRef<S>,
        value: ScaleRef<V>,
        fits: &mut Fits,
    ) {
        for (k, ends) in self.rows() {
            fits.add(key, k);
            for v in ends {
                fits.add(value, &v);
            }
        }
        match &self.rows {
            Rows::Values { base, .. } => {
                fits.add(value, base);
                fits.stick(value, *base);
            }
            Rows::Spans(rows) => {
                if rows.iter().flat_map(|(_, span)| span).any(|v| *v == 0.0) {
                    fits.stick(value, 0.0);
                }
            }
        }
        self.color
            .report(Swatch::Square, self.hatch.as_ref(), Alpha::OPAQUE, fits);
    }

    /// One `Fill` command per bar `rects` lists (panel coordinates), each
    /// followed by its hatch's clip group when hatched.
    fn fill(&self, ctx: &Ctx<'_>, rects: impl Iterator<Item = kurbo::Rect>, list: &mut List) {
        let paint = list::brush(self.color.resolve(ctx));
        for r in rects {
            if let Some(r) = rect::fill(list, ctx, r, &paint) {
                super::hatch(self.hatch.as_ref(), Clip::Rect(r), r, ctx, list);
            }
        }
    }

    /// Each bar placed, with its row's index: its extent across the key
    /// axis of length `length` (dodged into the fill key's sub-band when
    /// asked), and its two ends along the value axis (the base or the
    /// span's first end, then the value or the span's second end), clamped
    /// to `guard`. Rows whose key or ends cannot be placed are skipped.
    fn placed<S: Position<Domain = K>, V: Position<Domain = f64>>(
        &self,
        ctx: &Ctx<'_>,
        (sk, sv): (&S, &V),
        length: f64,
        guard: [f64; 2],
    ) -> Vec<(usize, [f64; 2], [f64; 2])> {
        let across = Across::new(sk, self.rows().map(|(k, _)| k), self.width, length);
        let slot = self.color.slot(ctx).filter(|_| self.dodge);
        self.rows()
            .enumerate()
            .filter_map(|(i, (k, [v0, v1]))| {
                let (start, width) = dodge(across.band(sk, k)?, slot);
                let ends = [reach(sv, v0, guard)?, reach(sv, v1, guard)?];
                Some((i, [start, start + width], ends))
            })
            .collect()
    }

    /// The rows whose key `scale` cannot place: keys outside a band or
    /// point domain.
    fn outside<S: Position<Domain = K>>(&self, scale: &S) -> usize {
        self.rows()
            .filter(|(k, _)| scale.position(k).is_none())
            .count()
    }
}

impl Across {
    /// How bars over `keys` sit across `scale`, an axis `length` px long.
    /// On band scales they sit in each key's band. Elsewhere they are
    /// `width` px wide, or 0.8 of the smallest gap between the keys'
    /// distinct positions (or of `length` for keys at one position).
    pub(super) fn new<'k, S: Position>(
        scale: &S,
        keys: impl Iterator<Item = &'k S::Domain>,
        width: Option<f64>,
        length: f64,
    ) -> Across {
        let bandwidth = scale.bandwidth();
        if bandwidth > 0.0 {
            return Across {
                width: bandwidth,
                banded: true,
            };
        }
        let width = width.unwrap_or_else(|| {
            let mut at: Vec<f64> = keys
                .filter_map(|k| scale.position(k))
                .filter(|p| p.is_finite())
                .collect();
            at.sort_by(f64::total_cmp);
            let gap = at
                .windows(2)
                .map(|w| w[1] - w[0])
                .filter(|d| *d > 0.0)
                .fold(f64::INFINITY, f64::min);
            SHARE * if gap.is_finite() { gap } else { length }
        });
        Across {
            width,
            banded: false,
        }
    }

    /// The band `(start, width)` in px of the bars over `key`; `None` for
    /// a key outside the domain or a position that is not finite.
    pub(super) fn band<S: Position>(&self, scale: &S, key: &S::Domain) -> Option<(f64, f64)> {
        let p = scale.position(key).filter(|p| p.is_finite())?;
        Some(if self.banded {
            (p, self.width)
        } else {
            (p - self.width / 2.0, self.width)
        })
    }
}

/// The builders both orientations share, for `$bar` with the generics
/// `$g` and its keys along `$axis`.
macro_rules! builders {
    ($bar:ident, [$($g:tt)*], $axis:literal) => {
        impl<$($g)*> $bar<X, Y> {
            /// Fills the bars with `palette`'s color for `key`, looked up
            /// when the figure draws; a key the palette lacks draws the
            /// theme's gray.
            #[must_use]
            pub fn fill<C: Paint>(mut self, palette: ScaleRef<C>, key: C::Domain) -> Self {
                self.bars.color = Color::keyed(palette, key);
                self
            }

            /// Hatches every bar with `h`, after its fill; legend swatches
            /// of the fill key repeat it.
            #[must_use]
            pub fn hatch(mut self, h: Hatch) -> Self {
                self.bars.hatch = Some(h);
                self
            }

            /// Places the bars in their fill key's slot of the band. The
            /// band is split into one sub-band per palette key, in the
            /// palette's order, so that bars of several marks stand side by
            /// side.
            #[must_use]
            pub fn dodge(mut self) -> Self {
                self.bars.dodge = true;
                self
            }

            #[doc = concat!("The bars' width in px on a ", $axis, " key scale without bands (continuous or")]
            /// point scales); by default 0.8 of the smallest gap between
            /// the keys' positions. Band scales give their bandwidth and
            /// ignore it. A width that is not finite and positive leaves it
            /// as it was.
            #[must_use]
            pub fn width(mut self, px: f64) -> Self {
                self.bars.width = positive(px).or(self.bars.width);
                self
            }

            /// The base the bars reach from, 0 by default. It is held, so
            /// the margin stops there. It applies only to rows of values,
            /// since span rows carry both ends and ignore it. A base that is
            /// not finite leaves it as it was.
            #[must_use]
            pub fn base(mut self, v: f64) -> Self {
                if let Rows::Values { base, .. } = &mut self.bars.rows
                    && v.is_finite()
                {
                    *base = v;
                }
                self
            }
        }
    };
}

builders!(BarY, [X: Position, Y: Position<Domain = f64>], "x");
builders!(BarX, [X: Position<Domain = f64>, Y: Position], "y");

impl<'a, X: Position, Y: Position<Domain = f64>> Mark<'a> for BarY<X, Y> {
    fn fit(&self, fits: &mut Fits) {
        fits.across(self.x);
        self.bars.fit(self.x, self.y, fits);
    }

    /// One `Fill` command per bar with area, clipped to the guard band, its
    /// edges on device pixels on raster targets, and its hatch when set.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let (Some(sx), Some(sy)) = (ctx.x(self.x), ctx.y(self.y)) else {
            return;
        };
        let w = ctx.get_rect().width();
        let band = ctx.band();
        let placed = self.bars.placed(ctx, (&sx, &sy), w, [band.y0, band.y1]);
        let rects = placed
            .into_iter()
            .map(|(_, [x0, x1], [y0, y1])| kurbo::Rect::new(x0, y0, x1, y1));
        self.bars.fill(ctx, rects, list);
    }

    /// Per bar, the center of its value end (the top of a bar from a base
    /// below its value; a span's second end) when it lies in the plot
    /// area, with its row's index.
    fn hits(&self, ctx: &Ctx<'_>, sink: &mut hit::Sink) {
        let (Some(sx), Some(sy)) = (ctx.x(self.x), ctx.y(self.y)) else {
            return;
        };
        let w = ctx.get_rect().width();
        let band = ctx.band();
        let area = ctx.get_rect().with_origin(Point::ORIGIN);
        for (i, [x0, x1], [_, y]) in self.bars.placed(ctx, (&sx, &sy), w, [band.y0, band.y1]) {
            let p = Point::new((x0 + x1) / 2.0, y);
            if within(area, p) {
                sink.push(i, p);
            }
        }
    }

    /// A fill key the palette lacks; keys outside a band or point domain.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        super::unknown(&self.bars.color, ctx, issues);
        if let Some(sx) = ctx.scale(self.x) {
            let count = self.bars.outside(sx);
            super::outside_count(self.x.get_index(), count, issues);
        }
    }
}

impl<'a, X: Position<Domain = f64>, Y: Position> Mark<'a> for BarX<X, Y> {
    fn fit(&self, fits: &mut Fits) {
        fits.across(self.x);
        self.bars.fit(self.y, self.x, fits);
    }

    /// One `Fill` command per bar with area, clipped to the guard band, its
    /// edges on device pixels on raster targets, and its hatch when set.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let (Some(sx), Some(sy)) = (ctx.x(self.x), ctx.y(self.y)) else {
            return;
        };
        let h = ctx.get_rect().height();
        let band = ctx.band();
        let placed = self.bars.placed(ctx, (&sy, &sx), h, [band.x0, band.x1]);
        let rects = placed
            .into_iter()
            .map(|(_, [y0, y1], [x0, x1])| kurbo::Rect::new(x0, y0, x1, y1));
        self.bars.fill(ctx, rects, list);
    }

    /// Per bar, the center of its value end (the right end of a bar from a
    /// base left of its value; a span's second end) when it lies in the
    /// plot area, with its row's index.
    fn hits(&self, ctx: &Ctx<'_>, sink: &mut hit::Sink) {
        let (Some(sx), Some(sy)) = (ctx.x(self.x), ctx.y(self.y)) else {
            return;
        };
        let h = ctx.get_rect().height();
        let band = ctx.band();
        let area = ctx.get_rect().with_origin(Point::ORIGIN);
        for (i, [y0, y1], [_, x]) in self.bars.placed(ctx, (&sy, &sx), h, [band.x0, band.x1]) {
            let p = Point::new(x, (y0 + y1) / 2.0);
            if within(area, p) {
                sink.push(i, p);
            }
        }
    }

    /// A fill key the palette lacks; keys outside a band or point domain.
    fn issues(&self, ctx: &Ctx<'_>, issues: &mut Vec<Issue>) {
        super::unknown(&self.bars.color, ctx, issues);
        if let Some(sy) = ctx.scale(self.y) {
            let count = self.bars.outside(sy);
            super::outside_count(self.y.get_index(), count, issues);
        }
    }
}
