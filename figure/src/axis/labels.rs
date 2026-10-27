//! [`Labels`]: one axis's tick labels and title as text-cache keys, and
//! the boxes they take beside the axis.

use kurbo::Rect;
use rustd3_core::axis::Orient;
use rustd3_core::color::Rgb;

use crate::axis::{Draw, across};
use crate::figure::Resolved;
use crate::text::{self, Anchor, Key, Shaped, Text, anchor};
use crate::theme::Theme;

/// One axis's labels for a layout pass: its side and color, the keys of its
/// tick labels (none when it is not labelled; `None` for an empty label)
/// with their ticks' absolute indices, the line box its tick labels share,
/// and the keys of its offset or multiplier label and of its title, set in
/// that color.
pub(crate) struct Labels {
    pub(super) orient: Orient,
    pub(super) ink: Rgb,
    pub(super) tinted: bool,
    pub(super) ticks: Vec<Option<Key>>,
    pub(super) indices: Vec<i64>,
    /// The text face's ascender and descender at the tick-label size.
    line: [f64; 2],
    pub(super) notation: Option<Key>,
    pub(super) title: Option<Key>,
}

impl Labels {
    /// The labels of `axis` over the resolved scales for about `count`
    /// ticks, in `theme`'s styles, with the line box of `text`'s fonts;
    /// none when not `labeled`.
    pub(crate) fn new<'a>(
        axis: &(dyn Draw<'a> + 'a),
        resolved: &Resolved,
        theme: &Theme,
        text: &text::Cache,
        labeled: bool,
        count: usize,
    ) -> Labels {
        let orient = axis.orient();
        let tint = axis.tint(resolved);
        let ink = tint.unwrap_or(theme.ink());
        let style = theme.tick_label(align(orient), ink);
        let (ticks, indices) = if labeled {
            let texts = axis.texts(resolved, count);
            let keys = texts
                .into_iter()
                .map(|t| (!t.is_empty()).then(|| Key::new(&Text::from(t), &style)))
                .collect();
            (keys, axis.indices(resolved, count))
        } else {
            (Vec::new(), Vec::new())
        };
        let notation = axis
            .notation(resolved, count)
            .filter(|_| labeled)
            .map(|t| Key::new(&Text::from(t), &theme.tick_label(notation(orient).h, ink)));
        let title = axis
            .title()
            .filter(|_| labeled)
            .map(|t| Key::new(t, &theme.axis_label(ink)));
        Labels {
            orient,
            ink,
            tinted: tint.is_some(),
            ticks,
            indices,
            line: text.fonts().line(theme.tick_text()),
            notation,
            title,
        }
    }

    /// Each tick label's tick's absolute index.
    pub(crate) fn indices(&self) -> &[i64] {
        &self.indices
    }

    /// Every key, for the text cache's batch.
    pub(crate) fn keys(&self) -> impl Iterator<Item = &Key> {
        self.ticks
            .iter()
            .flatten()
            .chain(&self.notation)
            .chain(&self.title)
    }

    /// How far the axis reaches past its offset: its tick, and, when
    /// labelled, the tick pad, its labels' extent across the axis (the
    /// widest advance beside a vertical axis, the tallest box under a
    /// horizontal one), on a horizontal axis its offset or multiplier label
    /// a tick pad further, and, with a title, the label pad and the title's
    /// ink height.
    pub(crate) fn thickness(&self, text: &text::Cache, theme: &Theme) -> f64 {
        let mut thickness = theme.tick();
        if self.ticks.is_empty() && self.title.is_none() {
            return thickness;
        }
        thickness += theme.tick_pad() + self.extent(text);
        if across(self.orient) {
            thickness += self.raise(text, theme);
        }
        if let Some(title) = self.title.as_ref().and_then(|key| get(text, key)) {
            thickness += theme.label_pad() + ink(title);
        }
        thickness
    }

    /// The room the offset or multiplier label takes: a tick pad and its
    /// ink height (beyond the tick labels of a horizontal axis, above the
    /// top end of a vertical one); 0 without one.
    pub(crate) fn raise(&self, text: &text::Cache, theme: &Theme) -> f64 {
        self.notation
            .as_ref()
            .and_then(|key| get(text, key))
            .map_or(0.0, |s| theme.tick_pad() + ink(s))
    }

    /// The tick labels' extent across the axis: the widest advance beside
    /// a vertical axis, the tallest box under a horizontal one; 0 without
    /// labels.
    pub(super) fn extent(&self, text: &text::Cache) -> f64 {
        let across = across(self.orient);
        shaped(text, &self.ticks)
            .flatten()
            .map(|s| {
                let b = self.bounds(s);
                if across { b.height() } else { b.width() }
            })
            .fold(0.0, f64::max)
    }

    /// Where tick label `shaped` has its first baseline, px below its
    /// anchor point (the end of its tick plus the tick pad; above it when
    /// negative). Rows share baselines through the text face's line box,
    /// as matplotlib places tick labels: under a bottom axis the first
    /// baseline lies the ascender below the point; over a top axis the last
    /// baseline lies the descender above it; beside a vertical axis the box
    /// from the first line's ascender to the last line's descender is
    /// centred on the point.
    pub(super) fn baseline(&self, shaped: &Shaped) -> f64 {
        let [ascender, descender] = self.line;
        match self.orient {
            Orient::Bottom => ascender,
            Orient::Top => -descender - shaped.last(),
            Orient::Left | Orient::Right => (ascender - descender - shaped.last()) / 2.0,
        }
    }

    /// Tick label `shaped`'s box relative to its anchor point: its advance
    /// width aligned by its side, by its line box (the text face's ascender
    /// above its first baseline to its descender below the last) united
    /// with its ink.
    pub(crate) fn bounds(&self, shaped: &Shaped) -> Rect {
        let [ascender, descender] = self.line;
        let width = shaped.width();
        let x0 = match align(self.orient) {
            anchor::Horizontal::Start => 0.0,
            anchor::Horizontal::Middle => -width / 2.0,
            anchor::Horizontal::End => -width,
        };
        let b = self.baseline(shaped);
        let top = (b - ascender).min(b - shaped.ascent());
        let bottom = (b + shaped.last() + descender).max(b + shaped.descent());
        Rect::new(x0, top, x0 + width, bottom)
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
        shaped(text, &self.ticks)
            .zip(positions)
            .filter_map(|(s, p)| Some((s?, (*p)?)))
            .fold([0.0, 0.0], |[before, after], (s, p)| {
                let b = self.bounds(s);
                let (low, high) = if across { (b.x0, b.x1) } else { (b.y0, b.y1) };
                [before.max(-(p + low)), after.max(p + high - length)]
            })
    }
}

/// How a tick label aligns on the end of its tick plus the tick pad:
/// centred under or over a horizontal axis, ending left of a left axis and
/// starting right of a right one.
pub(crate) fn align(orient: Orient) -> anchor::Horizontal {
    match orient {
        Orient::Bottom | Orient::Top => anchor::Horizontal::Middle,
        Orient::Left => anchor::Horizontal::End,
        Orient::Right => anchor::Horizontal::Start,
    }
}

/// Where an axis's offset or multiplier label sits: at the right end of a
/// horizontal axis, beyond its tick labels; above the top end of a vertical
/// one, starting at a left axis's line and ending at a right one's, over
/// the plot area.
pub(crate) fn notation(orient: Orient) -> Anchor {
    let (h, v) = match orient {
        Orient::Bottom => (anchor::Horizontal::End, anchor::Vertical::Top),
        Orient::Top | Orient::Right => (anchor::Horizontal::End, anchor::Vertical::Bottom),
        Orient::Left => (anchor::Horizontal::Start, anchor::Vertical::Bottom),
    };
    Anchor { h, v }
}

/// The ink height of a shaped label.
fn ink(shaped: &Shaped) -> f64 {
    shaped.ascent() + shaped.descent()
}

/// Each key's shaped label; `None` for an empty label, or one typst
/// rejected or not cached.
fn shaped<'c>(
    text: &'c text::Cache,
    keys: &'c [Option<Key>],
) -> impl Iterator<Item = Option<&'c Shaped>> {
    keys.iter().map(|key| get(text, key.as_ref()?))
}

/// `key`'s shaped label; `None` when typst rejected it or it is not
/// cached.
pub(super) fn get<'c>(text: &'c text::Cache, key: &Key) -> Option<&'c Shaped> {
    text.get(key)?.ok()
}
