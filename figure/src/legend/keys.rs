//! A legend of keys laid out as a [`Block`]: a row for top and bottom
//! slots, a column for side slots and insets, sized from matplotlib
//! 3.11.2's legend rc in em of the legend's text (the tick-label size); and
//! drawing it.

use kurbo::{Point, Rect, Shape, Size, Vec2};
use rustd3_core::color::Rgb;

use crate::figure::Resolved;
use crate::hatch::Hatch;
use crate::legend::{Draw, Swatch, swatch};
use crate::list::{self, Command, List};
use crate::text::{self, Anchor, Key, Shaped, Text, anchor};
use crate::theme::Theme;

/// A swatch's height, em: `legend.handleheight`.
const SWATCH: f64 = swatch::SIDE;

/// From a swatch to its label, and from a row's title to its first
/// swatch, em: `legend.handletextpad`.
const LABEL: f64 = 0.8;

/// Between the entries of a row, em: `legend.columnspacing`.
const ENTRIES: f64 = 2.0;

/// Between the rows of a column, its title's included, em:
/// `legend.labelspacing`.
const ROWS: f64 = 0.5;

/// Inside an inset's frame, em: `legend.borderpad`.
pub(crate) const PAD: f64 = 0.4;

/// The frame's edge, px.
const EDGE: f64 = 1.0;

/// Tolerance of a rectangle's path: exact, rectangles have no curves.
const TOLERANCE: f64 = 0.1;

/// A legend's content for one resolve: its title's key and its entries in
/// order.
pub(crate) struct Content {
    title: Option<Key>,
    entries: Vec<Keyed>,
}

/// An entry for one resolve: its swatch's color, its label's key, what its
/// swatch shows, the hatch of its marks and its dot's diameter, px
/// (`None`: the theme's marker size).
struct Keyed {
    color: Rgb,
    key: Key,
    glyph: Swatch,
    hatch: Option<Hatch>,
    size: Option<f64>,
}

/// A legend laid out, relative to its top-left corner: its size, whether
/// it is framed, its swatches (boxes, colors, what they show, hatches and
/// dot diameters), and where its labels' first baselines start (or, for a
/// centred title, are centred).
pub(crate) struct Block {
    size: Size,
    framed: bool,
    swatches: Vec<Swatched>,
    labels: Vec<(Key, Point, anchor::Horizontal)>,
}

/// A swatch: its box, color, what it shows, hatch and dot diameter.
type Swatched = (Rect, Rgb, Swatch, Option<Hatch>, Option<f64>);

/// A block being laid out: how far its labels and swatches reach above and
/// below the baseline at y = 0, and what it holds so far.
#[derive(Default)]
struct Rows {
    top: f64,
    bottom: f64,
    swatches: Vec<Swatched>,
    labels: Vec<(Key, Point, anchor::Horizontal)>,
}

impl Content {
    /// `legend`'s title and entries over the resolved scales, in `theme`'s
    /// legend styles; entries without a color take the theme's gray.
    pub(crate) fn new(legend: &dyn Draw, resolved: &Resolved, theme: &Theme) -> Content {
        let title = legend.title().map(|t| Key::new(t, &theme.legend_title()));
        let style = theme.legend_label();
        let entries = legend
            .entries(resolved, theme)
            .into_iter()
            .map(|entry| Keyed {
                color: entry.color.unwrap_or(theme.get_gray()),
                key: Key::new(&Text::from(entry.label), &style),
                glyph: entry.glyph,
                hatch: entry.hatch,
                size: entry.size,
            })
            .collect();
        Content { title, entries }
    }

    /// Every key, for the text cache's batch.
    pub(crate) fn keys(&self) -> impl Iterator<Item = &Key> {
        self.title
            .iter()
            .chain(self.entries.iter().map(|entry| &entry.key))
    }

    /// The legend as one row on a shared baseline: the title, 0.8 em, then
    /// each entry (its swatch's box, 0.7 em tall standing on the baseline
    /// and 0.7 em wide, 2 em for a line, or as wide and tall as a larger
    /// dot ([`swatch`]), then 0.8 em and its label) 2 em after the last.
    /// Its height is the line boxes of its labels (the text face's
    /// ascender and descender at their size) united with their ink and the
    /// swatches.
    pub(crate) fn row(&self, text: &text::Cache, theme: &Theme) -> Block {
        let em = theme.em();
        let entry = text.fonts().line(theme.tick_text());
        let mut rows = Rows {
            top: -SWATCH * em,
            ..Rows::default()
        };
        let mut x = 0.0;
        if let Some(title) = &self.title {
            let line = text.fonts().line(theme.label_text());
            x += rows.label(
                text,
                title,
                line,
                Point::new(x, 0.0),
                anchor::Horizontal::Start,
            );
            if !self.entries.is_empty() {
                x += LABEL * em;
            }
        }
        for (i, e) in self.entries.iter().enumerate() {
            if i > 0 {
                x += ENTRIES * em;
            }
            let width = e.width(em);
            let r = swatch(x, width, em, e.size);
            rows.top = rows.top.min(r.y0);
            rows.bottom = rows.bottom.max(r.y1);
            rows.swatches
                .push((r, e.color, e.glyph, e.hatch.clone(), e.size));
            x += width + LABEL * em;
            x += rows.label(
                text,
                &e.key,
                entry,
                Point::new(x, 0.0),
                anchor::Horizontal::Start,
            );
        }
        let size = Size::new(x, rows.bottom - rows.top);
        let by = Vec2::new(0.0, -rows.top);
        rows.block(size, by)
    }

    /// The legend as a column: the title centred on top, then one entry per
    /// row (its swatch's box standing on the row's baseline as in a
    /// [`row`](Content::row), every box as wide as the widest, 0.8 em, its
    /// label), each row the height of its label's line box united with its
    /// ink and swatch, rows 0.5 em apart. Squares and dots are centred in
    /// boxes wider than they are.
    pub(crate) fn column(&self, text: &text::Cache, theme: &Theme) -> Block {
        let em = theme.em();
        let side = self.entries.iter().map(|e| e.width(em)).fold(0.0, f64::max);
        let entry = text.fonts().line(theme.tick_text());
        let width = |key: &Key| get(text, key).map_or(0.0, Shaped::get_width);
        let labels = self
            .entries
            .iter()
            .map(|e| width(&e.key))
            .fold(0.0, f64::max);
        let entries = if self.entries.is_empty() {
            0.0
        } else {
            side + LABEL * em + labels
        };
        let w = self.title.as_ref().map_or(0.0, width).max(entries);
        let mut rows = Rows::default();
        let mut y = 0.0;
        let title = self
            .title
            .as_ref()
            .and_then(|key| Some((key, get(text, key)?)));
        if let Some((key, shaped)) = title {
            let [top, bottom] = extent(shaped, text.fonts().line(theme.label_text()));
            let at = Point::new(w / 2.0, y - top);
            rows.labels
                .push((key.clone(), at, anchor::Horizontal::Middle));
            y += bottom - top;
        }
        for (i, e) in self.entries.iter().enumerate() {
            if i > 0 || title.is_some() {
                y += ROWS * em;
            }
            let shaped = get(text, &e.key);
            let [top, bottom] = shaped.map_or([0.0, 0.0], |s| extent(s, entry));
            let r = swatch(0.0, side, em, e.size);
            let (top, bottom) = (top.min(r.y0), bottom.max(r.y1));
            let baseline = y - top;
            let r = r + Vec2::new(0.0, baseline);
            rows.swatches
                .push((r, e.color, e.glyph, e.hatch.clone(), e.size));
            if shaped.is_some() {
                let at = Point::new(side + LABEL * em, baseline);
                rows.labels
                    .push((e.key.clone(), at, anchor::Horizontal::Start));
            }
            y += bottom - top;
        }
        rows.block(Size::new(w, y), Vec2::ZERO)
    }
}

impl Keyed {
    /// Its swatch's box width for a legend em of `em` px: its glyph's, or
    /// a larger dot's diameter.
    fn width(&self, em: f64) -> f64 {
        self.glyph.width(em).max(self.size.unwrap_or(0.0))
    }
}

impl Rows {
    /// Adds `key`'s label with its first baseline at `at`, aligned by `h`,
    /// widening the reach above and below the baseline by its line box
    /// (`line`: ascender and descender at its size) united with its ink;
    /// its advance width, 0 when typst rejected it.
    fn label(
        &mut self,
        text: &text::Cache,
        key: &Key,
        line: [f64; 2],
        at: Point,
        h: anchor::Horizontal,
    ) -> f64 {
        let Some(shaped) = get(text, key) else {
            return 0.0;
        };
        let [top, bottom] = extent(shaped, line);
        self.top = self.top.min(top);
        self.bottom = self.bottom.max(bottom);
        self.labels.push((key.clone(), at, h));
        shaped.get_width()
    }

    /// The block of `size` holding what was added, moved by `by`.
    fn block(self, size: Size, by: Vec2) -> Block {
        Block {
            size,
            framed: false,
            swatches: self
                .swatches
                .into_iter()
                .map(|(r, c, g, h, d)| (r + by, c, g, h, d))
                .collect(),
            labels: self
                .labels
                .into_iter()
                .map(|(key, p, h)| (key, p + by, h))
                .collect(),
        }
    }
}

impl Block {
    /// Its size, px.
    pub(crate) fn size(&self) -> Size {
        self.size
    }

    /// The block in a frame 0.4 em larger on every side, as an inset draws
    /// it.
    pub(crate) fn framed(self, em: f64) -> Block {
        let pad = PAD * em;
        let by = Vec2::new(pad, pad);
        Block {
            size: Size::new(self.size.width + 2.0 * pad, self.size.height + 2.0 * pad),
            framed: true,
            swatches: self
                .swatches
                .into_iter()
                .map(|(r, c, g, h, d)| (r + by, c, g, h, d))
                .collect(),
            labels: self
                .labels
                .into_iter()
                .map(|(key, p, h)| (key, p + by, h))
                .collect(),
        }
    }

    /// Appends the block with its top-left corner at `at`: the frame (the
    /// theme's legend face, an edge of 1 px), each swatch in its box, then
    /// each label.
    pub(crate) fn draw(&self, at: Point, text: &text::Cache, theme: &Theme, list: &mut List) {
        let by = at.to_vec2();
        if self.framed {
            frame(self.size.to_rect() + by, theme, list);
        }
        for (rect, color, glyph, hatch, size) in &self.swatches {
            glyph.draw(*rect + by, *color, hatch.as_ref(), *size, theme, list);
        }
        for (key, p, h) in &self.labels {
            if let Some(shaped) = get(text, key) {
                let anchor = Anchor {
                    h: *h,
                    v: anchor::Vertical::Baseline,
                };
                shaped.place(list, *p + by, anchor, 0.0);
            }
        }
    }
}

/// Appends an inset's frame around `rect`: the theme's legend face, an
/// edge of 1 px in its legend edge color.
pub(crate) fn frame(rect: Rect, theme: &Theme, list: &mut List) {
    list.commands.push(Command::Fill {
        path: rect.to_path(TOLERANCE),
        paint: list::brush(theme.legend_face()),
    });
    list.commands.push(Command::Stroke {
        path: rect.to_path(TOLERANCE),
        style: list::stroke(EDGE),
        paint: list::brush(theme.legend_edge()),
    });
}

/// An entry's swatch box `width` px wide from `x`, relative to its row's
/// baseline at y = 0: 0.7 em tall standing on the baseline, or, for a dot
/// whose diameter `size` exceeds that, a square of that side centred where
/// the 0.7-em box's centre is, so rows grow to hold it.
fn swatch(x: f64, width: f64, em: f64, size: Option<f64>) -> Rect {
    let h = SWATCH * em;
    match size {
        Some(d) if d > h => {
            let (cx, cy) = (x + width / 2.0, -h / 2.0);
            Rect::new(cx - d / 2.0, cy - d / 2.0, cx + d / 2.0, cy + d / 2.0)
        }
        Some(_) | None => Rect::new(x, -h, x + width, 0.0),
    }
}

/// How far `shaped` reaches above its first baseline (negative) and below
/// it: its line box (`line`, the ascender above the first baseline and the
/// descender below the last) united with its ink.
fn extent(shaped: &Shaped, [ascender, descender]: [f64; 2]) -> [f64; 2] {
    [
        (-ascender).min(-shaped.get_ascent()),
        (shaped.get_last() + descender).max(shaped.get_descent()),
    ]
}

/// `key`'s shaped label; `None` when typst rejected it or it is not
/// cached.
fn get<'c>(text: &'c text::Cache, key: &Key) -> Option<&'c Shaped> {
    text.get(key)?.ok()
}
