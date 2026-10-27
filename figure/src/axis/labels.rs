//! [`Labels`]: one axis's tick labels and title as text-cache keys, and
//! the boxes they take beside the axis.

use rustd3_core::axis::Orient;
use rustd3_core::color::Rgb;

use crate::axis::{Draw, across};
use crate::figure::Resolved;
use crate::text::{self, Anchor, Key, Shaped, Text, anchor};
use crate::theme::Theme;

/// One axis's labels for a resolve: its side, offset and color, the keys of
/// its tick labels (none when it is not labelled) and of its title, set in
/// that color.
pub(crate) struct Labels {
    pub(super) orient: Orient,
    pub(super) outward: f64,
    pub(super) ink: Rgb,
    pub(super) tinted: bool,
    pub(super) ticks: Vec<Key>,
    pub(super) title: Option<Key>,
}

impl Labels {
    /// The labels of `axis` over the resolved scales, in `theme`'s styles.
    pub(crate) fn new<'a>(
        axis: &(dyn Draw<'a> + 'a),
        resolved: &Resolved,
        theme: &Theme,
    ) -> Labels {
        let orient = axis.orient();
        let tint = axis.tint(resolved);
        let ink = tint.unwrap_or(theme.ink());
        let labeled = axis.labeled();
        let style = theme.tick_label(anchor(orient).h, ink);
        let ticks = if labeled {
            let texts = axis.texts(resolved);
            texts
                .into_iter()
                .map(|t| Key::new(&Text::from(t), &style))
                .collect()
        } else {
            Vec::new()
        };
        let title = axis
            .title()
            .filter(|_| labeled)
            .map(|t| Key::new(t, &theme.axis_label(ink)));
        Labels {
            orient,
            outward: axis.outward(),
            ink,
            tinted: tint.is_some(),
            ticks,
            title,
        }
    }

    /// The side of the panel the axis is on.
    pub(crate) fn orient(&self) -> Orient {
        self.orient
    }

    /// Every key, for the text cache's one batch.
    pub(crate) fn keys(&self) -> impl Iterator<Item = &Key> {
        self.ticks.iter().chain(&self.title)
    }

    /// How far the axis reaches past the plot area: its offset and tick,
    /// and, when labelled, the tick pad, its labels' extent across the axis
    /// (the widest advance beside a vertical axis, the tallest ink under a
    /// horizontal one) and, with a title, the label pad and the title's ink
    /// height.
    pub(crate) fn reach(&self, text: &text::Cache, theme: &Theme) -> f64 {
        let mut reach = self.outward + theme.tick();
        if self.ticks.is_empty() && self.title.is_none() {
            return reach;
        }
        reach += theme.tick_pad() + self.extent(text);
        if let Some(title) = self.title.as_ref().and_then(|key| get(text, key)) {
            reach += theme.label_pad() + ink(title);
        }
        reach
    }

    /// The tick labels' extent across the axis: the widest advance beside
    /// a vertical axis, the tallest ink under a horizontal one; 0 without
    /// labels.
    pub(super) fn extent(&self, text: &text::Cache) -> f64 {
        let across = across(self.orient);
        shaped(text, &self.ticks)
            .flatten()
            .map(|s| if across { ink(s) } else { s.width() })
            .fold(0.0, f64::max)
    }

    /// How far the tick labels at `positions` (aligned with the ticks)
    /// reach past the two ends of a panel `length` px long, along the
    /// axis: [before its start, after its end] (left and right of a
    /// horizontal axis, above and below a vertical one).
    pub(crate) fn overhang(
        &self,
        positions: &[Option<f64>],
        length: f64,
        text: &text::Cache,
    ) -> [f64; 2] {
        let across = across(self.orient);
        let anchor = anchor(self.orient);
        shaped(text, &self.ticks)
            .zip(positions)
            .filter_map(|(s, p)| Some((s?, (*p)?)))
            .fold([0.0, 0.0], |[before, after], (s, p)| {
                let b = s.bounds(anchor);
                let (low, high) = if across { (b.x0, b.x1) } else { (b.y0, b.y1) };
                [before.max(-(p + low)), after.max(p + high - length)]
            })
    }
}

/// Where a tick label's box sits against the end of its tick plus the tick
/// pad: centred under or over a horizontal axis, right- or left-aligned
/// and centred beside a vertical one.
pub(crate) fn anchor(orient: Orient) -> Anchor {
    let (h, v) = match orient {
        Orient::Bottom => (anchor::Horizontal::Middle, anchor::Vertical::Top),
        Orient::Top => (anchor::Horizontal::Middle, anchor::Vertical::Bottom),
        Orient::Left => (anchor::Horizontal::End, anchor::Vertical::Middle),
        Orient::Right => (anchor::Horizontal::Start, anchor::Vertical::Middle),
    };
    Anchor { h, v }
}

/// The ink height of a shaped label.
fn ink(shaped: &Shaped) -> f64 {
    shaped.ascent() + shaped.descent()
}

/// Each key's shaped label; `None` for one typst rejected or not cached.
fn shaped<'c>(text: &'c text::Cache, keys: &'c [Key]) -> impl Iterator<Item = Option<&'c Shaped>> {
    keys.iter().map(|key| get(text, key))
}

/// `key`'s shaped label; `None` when typst rejected it or it is not
/// cached.
pub(super) fn get<'c>(text: &'c text::Cache, key: &Key) -> Option<&'c Shaped> {
    text.get(key)?.ok()
}
