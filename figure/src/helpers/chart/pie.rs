//! [`Pie`]: a pie or donut chart.

use std::fmt::Display;

use kurbo::{Affine, Vec2};
use rustd3_core::color::Rgb;
use rustd3_core::scale::Ordinal;
use rustd3_core::shape;

use crate::figure::Fits;
use crate::helpers::chart::setup::Setup;
use crate::helpers::labels;
use crate::list::{self, Command, List};
use crate::mark::{Ctx, Mark};
use crate::scale::{Paint, ScaleRef};
use crate::{Figure, path};

/// A pie chart: one wedge per slice around the middle of the plot area,
/// labelled inside (see [`pie`]).
pub struct Pie<K> {
    /// Each slice's key and value, in data order.
    slices: Vec<(K, f64)>,
    /// The inner radius as a share of the outer one: 0 for a pie.
    inner: f64,
    setup: Setup,
}

impl<K> std::fmt::Debug for Pie<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pie").finish_non_exhaustive()
    }
}

/// The wedges of a pie, sized by the plot area they are drawn in, and
/// their labels.
struct Wedges<K> {
    slices: Vec<(K, f64)>,
    inner: f64,
    color: ScaleRef<Ordinal<K, Rgb>>,
}

/// A pie chart of `slices`, keys and values: d3's `pie` angles (0 at 12
/// o'clock, clockwise, the largest value first) and `arc` wedges centred in
/// the plot area, of outer radius half the plot area's smaller side less
/// half an em, each in its key's color (the palette in data order), and
/// each key written at its wedge's centroid by
/// [`labels::pie`](crate::helpers::labels::pie), so the pie reserves no
/// room for labels and needs no legend. Slices whose value is not positive
/// span no angle and are not drawn. A pie has no axes: `.x_label`,
/// `.y_label` and `.minor` leave it as it is.
///
/// ```
/// use rustd3_figure::helpers::chart;
///
/// let fig = chart::pie([("rent", 1200.0), ("food", 450.0), ("travel", 300.0)])
///     .title("spending")
///     .figure();
/// ```
pub fn pie<K>(slices: impl IntoIterator<Item = (K, f64)>) -> Pie<K> {
    Pie {
        slices: slices.into_iter().collect(),
        inner: 0.0,
        setup: Setup::default(),
    }
}

impl<K> Pie<K> {
    /// A donut: wedges from `share` of the outer radius out, the labels
    /// halfway across the ring. A share outside [0, 1) leaves the chart as
    /// it was.
    ///
    /// ```
    /// use rustd3_figure::helpers::chart;
    ///
    /// let fig = chart::pie([("a", 3.0), ("b", 1.0), ("c", 2.0)]).inner(0.5).figure();
    /// ```
    #[must_use]
    pub fn inner(self, share: f64) -> Self {
        let inner = if (0.0..1.0).contains(&share) {
            share
        } else {
            self.inner
        };
        Pie { inner, ..self }
    }
}

impl<K: PartialEq + Clone + Display + 'static> Pie<K> {
    /// The figure: one panel with the wedges and their labels, and its
    /// title.
    #[must_use]
    pub fn figure<'a>(self) -> Figure<'a> {
        let Pie {
            slices,
            inner,
            setup,
        } = self;
        let mut fig = setup.figure();
        let color = fig.scale(setup.palette(slices.iter().map(|(k, _)| k.clone())));
        let panel = fig.panel(0, 0);
        panel.mark(Wedges {
            slices,
            inner,
            color,
        });
        setup.titled(panel);
        fig
    }
}

finish!(impl[K: PartialEq + Clone + Display + 'static] Pie<K>);

impl<K: Display> Wedges<K> {
    /// The labels of the wedges between radii `inner` and `outer`.
    fn labels(&self, inner: f64, outer: f64) -> labels::Pie<K> {
        labels::pie(&self.slices, inner, outer)
    }
}

impl<'a, K: PartialEq + Clone + Display + 'static> Mark<'a> for Wedges<K> {
    /// Reports the labels to be shaped; a pie fits no scale.
    fn fit(&self, fits: &mut Fits) {
        self.labels(0.0, 0.0).fit(fits);
    }

    /// One `Fill` command per wedge that spans an angle, in its key's color
    /// (the theme's gray for a key the palette lacks), then the labels.
    fn draw(&self, ctx: &Ctx<'_>, list: &mut List) {
        let area = ctx.rect();
        let outer = 0.5 * area.width().min(area.height()) - 0.5 * ctx.theme().em();
        if outer <= 0.0 || !outer.is_finite() {
            return;
        }
        let inner = self.inner * outer;
        let centre = Affine::translate(Vec2::new(area.width() / 2.0, area.height() / 2.0));
        let arc = shape::arc().inner_radius(inner).outer_radius(outer);
        let Some(palette) = ctx.raw(self.color) else {
            return;
        };
        let values = self.slices.iter().map(|(_, v)| *v);
        for slice in shape::pie().layout(values) {
            if slice.end_angle <= slice.start_angle {
                continue;
            }
            let key = &self.slices[slice.datum].0;
            let color = palette.paint(key).unwrap_or_else(|| ctx.theme().get_gray());
            list.commands.push(Command::Fill {
                path: centre * path::bez(|p| arc.draw(&slice, p)),
                paint: list::brush(color),
            });
        }
        self.labels(inner, outer).draw(ctx, list);
    }
}
