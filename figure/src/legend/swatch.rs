//! [`Swatch`]: what a legend draws beside each label.

use kurbo::{BezPath, Point, Rect, Shape, Vec2};
use rustd3_core::color::Rgb;

use crate::hatch::Hatch;
use crate::list::{self, Clip, Command, List};
use crate::mark;
use crate::theme::Theme;

/// A square's and a dot's box, em: matplotlib's `legend.handleheight`.
pub(crate) const SIDE: f64 = 0.7;

/// A line's length, em: matplotlib's `legend.handlelength`.
const LENGTH: f64 = 2.0;

/// How closely a dot's circle follows the true circle, in px.
const TOLERANCE: f64 = 1e-3;

/// What a legend draws beside each label, in the key's color, standing on
/// the label's baseline in a box 0.7 em tall (matplotlib's
/// `legend.handleheight`). A legend of a palette draws each key's as its
/// marks report it ([`Legend`](crate::legend::Legend)); a square of a key
/// whose marks are hatched repeats their [`Hatch`](crate::hatch::Hatch),
/// and one of a translucent area, ribbon or density is drawn at the
/// opacity it fills at, as matplotlib's legend handles are; a key filled
/// by several marks takes the most opaque fill, so a square of opaque
/// bars stays opaque beside a translucent density.
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_core::scheme::COLORBLIND;
/// use rustd3_figure::Figure;
/// use rustd3_figure::legend::{Slot, Swatch, legend};
///
/// let mut fig = Figure::new();
/// let color = fig.scale(scale::ordinal(["fit", "data"], COLORBLIND));
/// fig.legend(Slot::Bottom, legend(color).glyph(Swatch::Line { dash: Some([6.0, 3.0]) }));
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Swatch {
    /// A filled square 0.7 em on a side: for bars, areas, ribbons and
    /// densities, and for keys no mark uses.
    #[default]
    Square,
    /// A filled dot of the theme's marker diameter (at most the square's
    /// side), centred where the square would be: for scatters.
    Dot,
    /// A line 2 em long (matplotlib's `legend.handlelength`) at the height
    /// of the square's centre, the theme's line width, solid or dashed by
    /// `dash`, its on and off lengths in px as a line mark's `.dash`: for
    /// lines. A dash that is not finite and non-negative with a positive
    /// sum draws solid.
    Line {
        /// The dash's on and off lengths, px; `None` for a solid line.
        dash: Option<[f64; 2]>,
    },
}

impl Swatch {
    /// Its box's width for a legend em of `em` px.
    pub(crate) fn width(self, em: f64) -> f64 {
        match self {
            Swatch::Square | Swatch::Dot => SIDE * em,
            Swatch::Line { .. } => LENGTH * em,
        }
    }

    /// Appends the swatch in `color` to `list`, in its box `rect` in the
    /// figure's coordinates, sized by `theme`; a square, as large as the
    /// box's shorter side and centred in it, is hatched with `hatch`, anchored to the figure
    /// as its marks' hatches are, and a dot is `size` px across when given
    /// (a size legend's), else the theme's marker diameter at most the
    /// box's height.
    pub(crate) fn draw(
        self,
        rect: Rect,
        color: Rgb,
        hatch: Option<&Hatch>,
        size: Option<f64>,
        theme: &Theme,
        list: &mut List,
    ) {
        let paint = list::brush(color);
        let command = match self {
            Swatch::Square => {
                let side = rect.height().min(rect.width());
                let square = Rect::from_center_size(rect.center(), (side, side));
                list.commands.push(Command::Fill {
                    path: square.to_path(TOLERANCE),
                    paint,
                });
                if let Some(hatch) = hatch {
                    hatch.draw(Clip::Rect(square), square, Vec2::ZERO, theme, list);
                }
                return;
            }
            Swatch::Dot => {
                let d = size.unwrap_or_else(|| theme.get_marker().min(rect.height()));
                let r = d / 2.0;
                Command::Fill {
                    path: kurbo::Circle::new(rect.center(), r).to_path(TOLERANCE),
                    paint,
                }
            }
            Swatch::Line { dash } => {
                let y = rect.center().y;
                let mut path = BezPath::new();
                path.move_to(Point::new(rect.x0, y));
                path.line_to(Point::new(rect.x1, y));
                let dash = dash.and_then(mark::dashes).unwrap_or_default();
                Command::Stroke {
                    path,
                    style: list::stroke(theme.get_line_width()).with_dashes(0.0, dash),
                    paint,
                }
            }
        };
        list.commands.push(command);
    }
}
