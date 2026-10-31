//! [`Pie`]: labels at the centroids of pie slices.

use std::fmt::Display;
use std::marker::PhantomData;

use kurbo::{Point, Vec2};
use rustd3_core::format::Format;
use rustd3_core::locale::Locale;
use rustd3_core::shape;

use crate::figure::Fits;
use crate::helpers::labels::{Edge, key, place};
use crate::list::List;
use crate::mark::{Ctx, Mark};

/// The labels of a pie's slices (see [`pie`]).
pub struct Pie<K> {
    /// Each slice's key as written and its value, in data order.
    slices: Vec<(String, f64)>,
    inner: f64,
    outer: f64,
    /// The ring labels sit on, when not the slices' own middle.
    ring: Option<f64>,
    percent: bool,
    keys: PhantomData<fn(&K)>,
}

impl<K> std::fmt::Debug for Pie<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pie").finish_non_exhaustive()
    }
}

/// One label per slice of a pie of `slices` between radii `inner` and
/// `outer` (px; `inner` 0 for a pie, above it for a donut), centred on the
/// pie's slice's arc centroid (d3's `arc.centroid`: the middle of its
/// angle, halfway between the radii) about the middle of the plot area:
/// d3's `pie` angles, 0 at 12 o'clock and clockwise, sorted descending by
/// value, the labels in data order. A label is its key as `Display`
/// writes it, or its share of the total with [`percent`](Pie::percent);
/// slices whose value is not positive span no angle and are not labelled.
/// The labels sit inside the slices, so a pie reserves no room for them.
///
/// ```
/// use rustd3_figure::Figure;
/// use rustd3_figure::helpers::labels;
///
/// let shares = [("apples", 3.0), ("bananas", 1.0), ("cherries", 2.0)];
/// let mut fig = Figure::new();
/// fig.panel(0, 0)
///     .mark(labels::pie(&shares, 40.0, 80.0))
///     .mark(labels::pie(&shares, 40.0, 80.0).percent(true).radius(95.0));
/// ```
pub fn pie<K: Display>(slices: &[(K, f64)], inner: f64, outer: f64) -> Pie<K> {
    Pie {
        slices: slices.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        inner,
        outer,
        ring: None,
        percent: false,
        keys: PhantomData,
    }
}

impl<K> Pie<K> {
    /// Each label as its slice's share of the total of the positive values,
    /// written `.0%` in the figure's locale ("50%"), in place of its key.
    #[must_use]
    pub fn percent(self, on: bool) -> Self {
        Self {
            percent: on,
            ..self
        }
    }

    /// Centres the labels on the ring of radius `r` px, at each slice's
    /// middle angle, in place of halfway between the radii. A radius that is
    /// not between 0 and the outer radius leaves the ring as it was: labels
    /// outside the pie are not in v0.1.
    #[must_use]
    pub fn radius(self, r: f64) -> Self {
        let inside = (0.0..=self.outer).contains(&r);
        Self {
            ring: if inside { Some(r) } else { self.ring },
            ..self
        }
    }

    /// Each slice's label under `locale`, in data order; `None` for a slice
    /// whose value is not positive.
    fn texts(&self, locale: &Locale) -> Vec<Option<String>> {
        let total: f64 = self.positive().sum();
        let share = Format::new(
            locale.get_number(),
            ".0%"
                .parse()
                .expect("invariant: `.0%` is a valid specifier"),
        );
        self.slices
            .iter()
            .map(|(text, v)| {
                (*v > 0.0).then(|| {
                    if self.percent {
                        share.format(v / total)
                    } else {
                        text.clone()
                    }
                })
            })
            .collect()
    }

    /// The values that span an angle.
    fn positive(&self) -> impl Iterator<Item = f64> + '_ {
        self.slices.iter().map(|(_, v)| *v).filter(|v| *v > 0.0)
    }
}

impl<'a, K: 'a> Mark<'a> for Pie<K> {
    /// Reports the labels to be shaped; no values.
    fn fit(&self, fits: &mut Fits) {
        for text in self.texts(fits.locale()).into_iter().flatten() {
            let label = key(text, fits.theme());
            fits.label(label);
        }
    }

    /// Each positive slice's label centred on its centroid.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let rect = ctx.rect();
        let centre = Point::new(rect.width() / 2.0, rect.height() / 2.0);
        let (inner, outer) = self.ring.map_or((self.inner, self.outer), |r| (r, r));
        let arc = shape::arc().inner_radius(inner).outer_radius(outer);
        let slices = shape::pie().layout(self.slices.iter().map(|(_, v)| *v));
        let texts = self.texts(ctx.locale());
        for (slice, text) in slices.iter().zip(texts) {
            let Some(text) = text else { continue };
            let (dx, dy) = arc.centroid(slice);
            let at = centre + Vec2::new(dx, dy);
            if at.is_finite() {
                place(&key(text, ctx.theme()), at, Edge::Middle, ctx, list);
            }
        }
    }
}
