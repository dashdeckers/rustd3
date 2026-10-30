//! [`hash`]: what a part of a drawing list looks like apart from where it
//! draws, so that a plot area whose content moved by whole device pixels
//! and changed nothing else keeps its translated tiles.
//!
//! A pan changes every position in a plot area's layer, so its part hashes
//! all change; the style hash leaves positions out and keeps the
//! paints, stroke styles (width, caps, joins, the dash pattern without its
//! offset, which follows the column grid), marker shapes and paint tables,
//! fonts, sizes and glyph ids, images, and the commands' kinds and groups.
//! Per-point arrays (marker centres, sizes and paint indices) are left out
//! with the positions: points entering and leaving the overscan change
//! them on every pan. A command whose style equals the one before it is
//! folded once, so a thinned line that a pan splits into more or fewer
//! runs (NaN gaps entering the view) keeps its hash.

use std::fmt::Write as _;

use rustd3_figure::kurbo::{BezPath, PathEl, Stroke};
use rustd3_figure::list::{Clip, Command, List, Part};
use rustd3_figure::peniko::Brush;

use crate::plot::layer::{FNV, fnv};

/// The style hash of `part`, a part of `list`: FNV-1a 64 over its
/// commands' looks in order, positions left out, equal consecutive
/// commands folded once.
pub(crate) fn hash(list: &List, part: &Part) -> u64 {
    let mut hash = FNV;
    let mut last = None;
    for command in list.commands.get(part.commands.clone()).unwrap_or_default() {
        let one = look(list, command);
        if last != Some(one) {
            hash = fnv(hash, one);
            last = Some(one);
        }
    }
    hash
}

/// One command's look: its kind and what it draws with.
fn look(list: &List, command: &Command) -> u64 {
    let mut h = Words(FNV);
    match command {
        Command::Fill { paint, .. } => {
            h.tag(1);
            h.brush(paint);
        }
        Command::Stroke { style, paint, .. } => {
            h.tag(2);
            h.stroke(style);
            h.brush(paint);
        }
        Command::Markers {
            shape,
            paint,
            sizes,
            paints,
            ..
        } => {
            h.tag(3);
            h.path(shape);
            h.tag(u64::from(sizes.is_some()));
            match paints {
                Some(paints) => {
                    h.tag(paints.table.len() as u64);
                    for brush in &paints.table {
                        h.brush(brush);
                    }
                }
                None => h.brush(paint),
            }
        }
        Command::Glyphs(run) => {
            h.tag(4);
            match list.fonts.get(run.font) {
                Some(font) => {
                    h.tag(font.data.id());
                    h.tag(u64::from(font.index));
                }
                None => h.tag(run.font as u64),
            }
            h.tag(u64::from(run.size.to_bits()));
            h.brush(&run.fill);
            for glyph in &run.glyphs {
                h.tag(u64::from(glyph.id));
            }
        }
        Command::Image { image, smooth, .. } => {
            h.tag(5);
            h.tag(image.data.id());
            h.tag(u64::from(image.width) << 32 | u64::from(image.height));
            h.tag(u64::from(*smooth));
        }
        Command::Push { clip, .. } => {
            h.tag(6);
            h.tag(match clip {
                None => 0,
                Some(Clip::Rect(_)) => 1,
                Some(Clip::Path(_)) => 2,
            });
        }
        Command::Pop => h.tag(7),
        // `Command` is #[non_exhaustive] in rustd3-figure.
        _ => h.tag(8),
    }
    h.0
}

/// An FNV-1a 64 hash being written a word at a time.
struct Words(u64);

impl Words {
    /// Folds `word` in.
    fn tag(&mut self, word: u64) {
        self.0 = fnv(self.0, word);
    }

    /// A paint: a solid color's components; a gradient's stops and
    /// interpolation, not its geometry; an image brush's pixels and
    /// sampler.
    fn brush(&mut self, brush: &Brush) {
        match brush {
            Brush::Solid(color) => {
                self.tag(1);
                for c in color.components {
                    self.tag(u64::from(c.to_bits()));
                }
            }
            Brush::Gradient(gradient) => {
                self.tag(2);
                let mut text = String::new();
                let _ = write!(
                    text,
                    "{:?}{:?}{:?}{:?}{:?}",
                    gradient.extend,
                    gradient.interpolation_cs,
                    gradient.hue_direction,
                    gradient.interpolation_alpha_space,
                    gradient.stops
                );
                self.text(&text);
            }
            Brush::Image(image) => {
                self.tag(3);
                self.tag(image.image.data.id());
                self.text(&format!("{:?}", image.sampler));
            }
        }
    }

    /// A stroke's width, joins, caps and dash pattern; not the dash
    /// offset, which a pan moves with the column grid.
    fn stroke(&mut self, style: &Stroke) {
        self.tag(style.width.to_bits());
        self.tag(style.miter_limit.to_bits());
        self.text(&format!(
            "{:?}{:?}{:?}",
            style.join, style.start_cap, style.end_cap
        ));
        self.tag(style.dash_pattern.len() as u64);
        for v in &style.dash_pattern {
            self.tag(v.to_bits());
        }
    }

    /// A marker's shape, centred on the origin: its elements and points.
    fn path(&mut self, path: &BezPath) {
        for element in path.elements() {
            let points: &[_] = match element {
                PathEl::MoveTo(p) => {
                    self.tag(1);
                    &[*p]
                }
                PathEl::LineTo(p) => {
                    self.tag(2);
                    &[*p]
                }
                PathEl::QuadTo(a, b) => {
                    self.tag(3);
                    &[*a, *b]
                }
                PathEl::CurveTo(a, b, c) => {
                    self.tag(4);
                    &[*a, *b, *c]
                }
                PathEl::ClosePath => {
                    self.tag(5);
                    &[]
                }
            };
            for p in points {
                self.tag(p.x.to_bits());
                self.tag(p.y.to_bits());
            }
        }
    }

    /// The bytes of `text`, eight at a time.
    fn text(&mut self, text: &str) {
        for chunk in text.as_bytes().chunks(8) {
            let mut word = [0; 8];
            word[..chunk.len()].copy_from_slice(chunk);
            self.tag(u64::from_le_bytes(word));
        }
    }
}

#[cfg(test)]
mod tests {
    use rustd3_core::color::Rgb;
    use rustd3_core::scale;
    use rustd3_figure::kurbo::Size;
    use rustd3_figure::list::Part;
    use rustd3_figure::mark::{dots, line};
    use rustd3_figure::{Figure, Held, Target};

    use super::hash;
    use crate::plot::state;

    /// The mark parts of a curve and dots over x `x`, the curve in
    /// `color` and `width` px, drawn at 480 × 320 at 2: each part's hash
    /// and style hash.
    fn marks(x: [f64; 2], color: Rgb, width: f64) -> Vec<(u64, u64)> {
        let curve: Vec<(f64, f64)> = (0..=200)
            .map(|i| {
                let t = f64::from(i) * 0.5;
                (t, (t * 0.1).sin())
            })
            .collect();
        let mut fig = Figure::new();
        let hx = fig.scale(scale::linear().domain(x));
        let hy = fig.scale(scale::linear().domain([-1.2, 1.2]));
        fig.panel(0, 0)
            .mark(line(curve.clone(), hx, hy).color(color).width(width))
            .mark(dots(curve, hx, hy));
        let list = fig.draw_with(
            &mut state::cache(),
            Size::new(480.0, 320.0),
            Target::Screen { scale: 2.0 },
            &Held::default(),
        );
        let marks: Vec<&Part> = list
            .parts
            .iter()
            .filter(|part| part.id >= rustd3_figure::list::part::Id::new(6 << 56))
            .filter(|part| part.id < rustd3_figure::list::part::Id::new(7 << 56))
            .collect();
        marks
            .into_iter()
            .map(|part| (part.hash, hash(&list, part)))
            .collect()
    }

    #[test]
    fn a_pan_changes_part_hashes_and_keeps_style_hashes() {
        let blue = Rgb::new(30.0, 60.0, 140.0);
        let before = marks([0.0, 100.0], blue, 1.5);
        let after = marks([3.0, 103.0], blue, 1.5);
        assert_eq!(before.len(), 2);
        for ((part, style), (moved, kept)) in before.iter().zip(&after) {
            assert_ne!(part, moved, "positions changed");
            assert_eq!(style, kept, "the looks did not");
        }
    }

    #[test]
    fn a_new_color_or_width_changes_the_style_hash() {
        let blue = Rgb::new(30.0, 60.0, 140.0);
        let red = Rgb::new(200.0, 30.0, 30.0);
        let base = marks([0.0, 100.0], blue, 1.5);
        let recolored = marks([3.0, 103.0], red, 1.5);
        let wider = marks([3.0, 103.0], blue, 2.0);
        assert_ne!(base[0].1, recolored[0].1);
        assert_ne!(base[0].1, wider[0].1);
        // The dots' look did not change.
        assert_eq!(base[1].1, recolored[1].1);
    }
}
