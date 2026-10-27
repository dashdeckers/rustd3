//! [`Shaped`]: a label as typst laid it out, and placing it into a list.

use std::sync::Arc;

use kurbo::{Affine, BezPath, Point, Rect};
use peniko::{Brush, FontData};

use crate::list::{Command, GlyphRun, List};
use crate::text::{Anchor, Fonts, anchor};

/// A label shaped by typst: its glyph runs and filled shapes (math rules)
/// in frame order, relative to the first line's baseline at the box's left
/// edge (y down), and its box. The box is as wide as the advances and as
/// tall as the ink.
#[derive(Debug)]
pub struct Shaped {
    fonts: Arc<Fonts>,
    items: Vec<Item>,
    width: f64,
    ascent: f64,
    descent: f64,
    last: f64,
    missing: usize,
}

/// One drawn piece of a label.
#[derive(Debug)]
pub(crate) enum Item {
    /// A glyph run; its font is a face index in the label's [`Fonts`].
    Run(GlyphRun),
    /// A filled shape: a rule, or a stroke's outline.
    Fill {
        /// The shape.
        path: BezPath,
        /// Its paint.
        paint: Brush,
    },
}

impl Shaped {
    /// A shaped label from its parts; see the fields.
    pub(crate) fn new(
        fonts: Arc<Fonts>,
        items: Vec<Item>,
        [width, ascent, descent, last]: [f64; 4],
        missing: usize,
    ) -> Shaped {
        Shaped {
            fonts,
            items,
            width,
            ascent,
            descent,
            last,
            missing,
        }
    }

    /// The advance width of the widest line, px.
    pub fn width(&self) -> f64 {
        self.width
    }

    /// From the top of the ink to the first baseline, px.
    pub fn ascent(&self) -> f64 {
        self.ascent
    }

    /// From the first baseline to the bottom of the ink, px.
    pub fn descent(&self) -> f64 {
        self.descent
    }

    /// From the first baseline down to the last line's, px: 0 for one
    /// line. Read from the runs set at the label's largest size, so
    /// scripts do not count as lines.
    pub fn last(&self) -> f64 {
        self.last
    }

    /// Glyphs typst drew as .notdef because no font has them (fig.check
    /// reports them).
    pub fn missing(&self) -> usize {
        self.missing
    }

    /// The box relative to the anchor point, before rotation.
    pub fn bounds(&self, anchor: Anchor) -> Rect {
        let (dx, dy) = self.offset(anchor);
        Rect::new(dx, dy - self.ascent, dx + self.width, dy + self.descent)
    }

    /// Appends the label's runs and fills to `list`, its `anchor` at `at`,
    /// rotated by `angle` radians (clockwise on screen) about it: the
    /// placement is `translate(at) × rotate(angle) × translate(offset)`.
    /// Each run's face is found in `list.fonts` by blob id and index, or
    /// appended to it.
    pub fn place(&self, list: &mut List, at: Point, anchor: Anchor, angle: f64) {
        let placement = Affine::translate(at.to_vec2())
            * Affine::rotate(angle)
            * Affine::translate(self.offset(anchor));
        for item in &self.items {
            let command = match item {
                Item::Run(run) => Command::Glyphs(GlyphRun {
                    font: font(list, self.fonts.data(run.font)),
                    transform: placement * run.transform,
                    ..run.clone()
                }),
                Item::Fill { path, paint } => Command::Fill {
                    path: placement * path.clone(),
                    paint: paint.clone(),
                },
            };
            list.commands.push(command);
        }
    }

    /// The anchor's offset from the first baseline's left end.
    fn offset(&self, anchor: Anchor) -> (f64, f64) {
        let dx = match anchor.h {
            anchor::Horizontal::Start => 0.0,
            anchor::Horizontal::Middle => -self.width / 2.0,
            anchor::Horizontal::End => -self.width,
        };
        let dy = match anchor.v {
            anchor::Vertical::Baseline => 0.0,
            anchor::Vertical::Top => self.ascent,
            anchor::Vertical::Middle => (self.ascent - self.descent) / 2.0,
            anchor::Vertical::Bottom => -self.descent,
        };
        (dx, dy)
    }
}

/// `face`'s index in `list.fonts`, appended when the list lacks it.
fn font(list: &mut List, face: &FontData) -> usize {
    let found = list
        .fonts
        .iter()
        .position(|f| f.data.id() == face.data.id() && f.index == face.index);
    found.unwrap_or_else(|| {
        list.fonts.push(face.clone());
        list.fonts.len() - 1
    })
}
