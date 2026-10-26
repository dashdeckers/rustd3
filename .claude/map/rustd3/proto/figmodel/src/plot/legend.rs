//! A legend is to a color scale what an axis is to a
//! position scale: a value, placed in a figure-level slot
//! or inside a panel.

use super::draw::{Label, List};
use super::text::Text;
use super::{Resolved, ScaleRef};
use d3::color::Rgb;
use d3::scale::Ordinal;
use kurbo::{Affine, Rect};
use std::fmt::Display;

/// Figure-level slots, outside the grid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Slot {
    Top,
    Bottom,
}

/// Corners inside a panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Inset {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub struct Legend<K> {
    color: ScaleRef<Ordinal<K, Rgb>>,
    title: Option<Text>,
}

impl<K> Clone for Legend<K> {
    fn clone(&self) -> Self {
        let title = self.title.clone();
        Self { color: self.color, title }
    }
}

/// One swatch per key of an ordinal color scale.
pub fn legend<K>(color: ScaleRef<Ordinal<K, Rgb>>) -> Legend<K>
where
    K: PartialEq + Clone + Display + 'static,
{
    Legend { color, title: None }
}

impl<K> Legend<K> {
    pub fn title(self, t: impl Into<Text>) -> Self {
        Self { title: Some(t.into()), ..self }
    }
}

/// Approximate text width until typst measures here.
pub(crate) fn width(r: &Resolved, t: &Text, bold: bool) -> f64 {
    r.measure.0.width(t, r.theme.tick_text(), bold)
}

pub(crate) trait Draw {
    fn entries(&self, r: &Resolved) -> Vec<(Rgb, String)>;
    fn title(&self) -> Option<&Text>;

    /// A row, centered on `cx`, baseline at `y`.
    fn row(&self, r: &Resolved, cx: f64, y: f64, o: &mut List) {
        let entries = self.entries(r);
        let title = self.title();
        let th = &r.theme;
        let (s, k) = (th.tick_text(), th.tick_text() / 10.0);
        let title_w = title.map_or(0.0, |t| width(r, t, true) + 12.0 * k);
        let total = title_w
            + entries
                .iter()
                .map(|(_, l)| 14.0 * k + width(r, &Text::from(l.as_str()), false) + 18.0 * k)
                .sum::<f64>()
            - 18.0 * k;
        let mut x = cx - total / 2.0;
        if let Some(t) = title {
            o.label(Label::new(t.clone(), x, y).size(s as f32).fill(th.ink()).bold());
            x += title_w;
        }
        for (c, l) in entries {
            o.fill(Rect::new(x, y - 0.8 * s, x + 0.8 * s, y), c);
            o.label(Label::new(l.as_str(), x + 14.0 * k, y).size(s as f32).fill(th.ink()));
            x += 14.0 * k + width(r, &Text::from(l.as_str()), false) + 18.0 * k;
        }
    }

    /// A boxed column in a corner of a panel `w` × `h`.
    fn inset(&self, r: &Resolved, at: Inset, w: f64, h: f64, o: &mut List) {
        let entries = self.entries(r);
        let title = self.title();
        let rows = entries.len() + usize::from(title.is_some());
        let label_w = entries
            .iter()
            .map(|(_, l)| width(r, &Text::from(l.as_str()), false))
            .fold(0.0, f64::max);
        let title_w = title.map_or(0.0, |t| width(r, t, true));
        let th = &r.theme;
        let (s, k) = (th.tick_text(), th.tick_text() / 10.0);
        let bw = 22.0 * k + label_w.max(title_w);
        let bh = 8.0 * k + 14.0 * k * rows as f64;
        let pad = 6.0 * k;
        let (x, y) = match at {
            Inset::TopLeft => (pad, pad),
            Inset::TopRight => (w - bw - pad, pad),
            Inset::BottomLeft => (pad, h - bh - pad),
            Inset::BottomRight => (w - bw - pad, h - bh - pad),
        };
        o.push(Affine::translate((x, y)));
        let frame = Rect::new(0.0, 0.0, bw, bh);
        o.fill(frame, Rgb::hex(0xffffff).with_opacity(0.85));
        o.stroke(frame, 1.0, None, Rgb::hex(0x999999));
        let mut ty = 15.0 * k;
        if let Some(t) = title {
            o.label(Label::new(t.clone(), 6.0 * k, ty).size(s as f32).fill(th.ink()).bold());
            ty += 14.0 * k;
        }
        for (c, l) in entries {
            let swatch = Rect::new(6.0 * k, ty - 0.8 * s, 6.0 * k + 0.8 * s, ty);
            o.fill(swatch, c);
            o.label(Label::new(l.as_str(), 20.0 * k, ty).size(s as f32).fill(th.ink()));
            ty += 14.0 * k;
        }
        o.pop();
    }
}

impl<K> Draw for Legend<K>
where
    K: PartialEq + Clone + Display + 'static,
{
    fn entries(&self, r: &Resolved) -> Vec<(Rgb, String)> {
        let s = r.raw(self.color);
        s.keys()
            .iter()
            .map(|k| {
                let c = s.apply(k).copied();
                (c.unwrap_or(super::GRAY), k.to_string())
            })
            .collect()
    }

    fn title(&self) -> Option<&Text> {
        self.title.as_ref()
    }
}
