//! [`Setup`]: what every preset is finished with.

use kurbo::Size;
use rustd3_core::color::Rgb;
use rustd3_core::scale::{self, Ordinal, Ticks};

use crate::axis::{self, Axis};
use crate::check::Issue;
use crate::scale::{Kind, Position, ScaleRef};
use crate::text::Text;
use crate::{Figure, Panel, Theme};

/// A preset's default size, px: matplotlib's 6.4 × 4.8 in at 100 dpi.
const SIZE: Size = Size::new(640.0, 480.0);

/// The finishing builders' settings: the panel's title, the axes' labels,
/// the size files are written at, the theme and whether axes have minor
/// ticks.
pub(super) struct Setup {
    pub(super) title: Option<Text>,
    pub(super) x_label: Option<Text>,
    pub(super) y_label: Option<Text>,
    pub(super) size: Size,
    pub(super) theme: Theme,
    pub(super) minor: bool,
}

impl Default for Setup {
    /// No title or labels, 640 × 480 px, the default theme, minor ticks on.
    fn default() -> Self {
        Setup {
            title: None,
            x_label: None,
            y_label: None,
            size: SIZE,
            theme: Theme::default(),
            minor: true,
        }
    }
}

impl Setup {
    /// Files are written at `w × h` px; a side that is not finite and
    /// positive leaves the size as it was.
    pub(super) fn size(&mut self, w: f64, h: f64) {
        if [w, h].iter().all(|v| v.is_finite() && *v > 0.0) {
            self.size = Size::new(w, h);
        }
    }

    /// An empty figure in the theme.
    pub(super) fn figure<'a>(&self) -> Figure<'a> {
        let mut fig = Figure::new();
        fig.theme(self.theme.clone());
        fig
    }

    /// A palette of the theme's colors over `keys`, in their order (the
    /// colors repeat past the palette's length, as d3's ordinal scales
    /// cycle their range).
    pub(super) fn palette<K: PartialEq>(
        &self,
        keys: impl IntoIterator<Item = K>,
    ) -> Ordinal<K, Rgb> {
        scale::ordinal(keys, self.theme.get_palette().to_vec())
    }

    /// Places on `panel` its title, an axis of `x` below and one of `y` on
    /// the left, each with its label and, when on, minor ticks.
    pub(super) fn axes<'a, X, Y>(&self, panel: &mut Panel<'a>, x: ScaleRef<X>, y: ScaleRef<Y>)
    where
        X: Position + Ticks<Domain = <X as Kind>::Domain>,
        Y: Position + Ticks<Domain = <Y as Kind>::Domain>,
    {
        let bottom = labelled(axis::bottom(x), self.x_label.as_ref());
        let left = labelled(axis::left(y), self.y_label.as_ref());
        panel.axis(self.minors(bottom)).axis(self.minors(left));
        self.titled(panel);
    }

    /// Gives `panel` the title, when there is one.
    pub(super) fn titled(&self, panel: &mut Panel<'_>) {
        if let Some(t) = &self.title {
            panel.title(t.clone());
        }
    }

    /// `axis` with its minor ticks on when the preset's are.
    fn minors<'a, S: Kind>(&self, axis: Axis<'a, S>) -> Axis<'a, S> {
        axis.minor(self.minor)
    }
}

/// Notes on `fig` an [`Issue::DuplicateKey`] for each key that `keys`
/// holds more than once, in the order of its second occurrence.
pub(super) fn duplicates<'k>(fig: &mut Figure<'_>, keys: impl IntoIterator<Item = &'k String>) {
    let mut seen: Vec<&String> = Vec::new();
    let mut twice: Vec<&String> = Vec::new();
    for key in keys {
        if !seen.contains(&key) {
            seen.push(key);
        } else if !twice.contains(&key) {
            twice.push(key);
            fig.note(Issue::DuplicateKey { key: key.clone() });
        }
    }
}

/// `axis` with `label`, when there is one.
fn labelled<'a, S: Kind>(axis: Axis<'a, S>, label: Option<&Text>) -> Axis<'a, S> {
    match label {
        Some(t) => axis.label(t.clone()),
        None => axis,
    }
}
