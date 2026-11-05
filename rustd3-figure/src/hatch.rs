//! [`Hatch`]: parallel lines over area-like marks and their legend
//! swatches, built as line geometry clipped to each shape.

use kurbo::{BezPath, Point, Rect, Vec2};
use rustd3_core::color::Rgb;

use crate::list::{self, Clip, Command, List};
use crate::theme::Theme;

/// The most lines of one family over one shape; a denser family draws none.
const MOST: f64 = 4096.0;

/// Parallel lines over a filled shape, clipped to it: one family of lines
/// at an angle, or two crossing at right angles. Angles are degrees
/// counter-clockwise on screen, matplotlib's sense (45° looks like `/`).
/// The lines lie `k·s` from the figure's origin for whole `k`, so they
/// continue across neighboring bars, as matplotlib's do; the spacing `s`
/// and the line width are points times the theme's context (4/3 px per
/// pt). Legend swatches of a hatched mark's key repeat its hatch.
///
/// Area-like marks take it with `.hatch(h)`: [`area`](crate::mark::area()),
/// [`ribbon`](crate::mark::ribbon), the bars of [`bar_y`](crate::mark::bar_y),
/// [`bar_x`](crate::mark::bar_x) and their span forms,
/// [`histogram`](crate::mark::histogram()), [`density_y`](crate::mark::density_y)
/// and [`density_x`](crate::mark::density_x).
///
/// ```
/// use rustd3_core::scale;
/// use rustd3_figure::Figure;
/// use rustd3_figure::hatch::Hatch;
/// use rustd3_figure::mark::bar_y;
///
/// let mut fig = Figure::new();
/// let x = fig.scale(scale::band(["a", "b"]));
/// let y = fig.fit(scale::linear());
/// fig.panel(0, 0)
///     .mark(bar_y([("a", 3.0), ("b", 1.5)], x, y).hatch(Hatch::cross(45.0).spacing(4.0)));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Hatch {
    /// Each family's angle, degrees counter-clockwise.
    angles: Vec<f64>,
    /// The distance between neighboring lines of a family, pt.
    spacing: f64,
    /// The lines' width, pt.
    width: f64,
    /// The lines' color; the theme's ink when unset.
    color: Option<Rgb>,
}

impl Hatch {
    /// One family of lines at `deg`, 6 pt apart and 0.8 pt wide in the
    /// theme's ink. An angle that is not finite hatches nothing.
    #[must_use]
    pub fn lines(deg: f64) -> Self {
        Hatch {
            angles: vec![deg],
            spacing: 6.0,
            width: 0.8,
            color: None,
        }
    }

    /// Two families, at `deg` and `deg + 90`, otherwise as
    /// [`lines`](Hatch::lines).
    #[must_use]
    pub fn cross(deg: f64) -> Self {
        Hatch {
            angles: vec![deg, deg + 90.0],
            ..Hatch::lines(deg)
        }
    }

    /// `pt` between neighboring lines of a family, 6 by default; a
    /// spacing that is not finite and positive leaves it as it was.
    #[must_use]
    pub fn spacing(self, pt: f64) -> Self {
        Hatch {
            spacing: if pt.is_finite() && pt > 0.0 {
                pt
            } else {
                self.spacing
            },
            ..self
        }
    }

    /// Lines `pt` wide, 0.8 by default; a width that is not finite and
    /// positive leaves it as it was.
    #[must_use]
    pub fn width(self, pt: f64) -> Self {
        Hatch {
            width: if pt.is_finite() && pt > 0.0 {
                pt
            } else {
                self.width
            },
            ..self
        }
    }

    /// Lines in `c`, the theme's ink by default; a color with a channel
    /// that is not finite leaves it as it was.
    #[must_use]
    pub fn color(self, c: Rgb) -> Self {
        let finite = [c.r, c.g, c.b, c.opacity].iter().all(|v| v.is_finite());
        Hatch {
            color: if finite { Some(c) } else { self.color },
            ..self
        }
    }

    /// The lines over `bounds`, a box in a group whose origin lies at
    /// `origin` in the figure's coordinates, as one path in the group's
    /// coordinates. Each family at angle θ is the lines `n·p = k·s` in the
    /// figure's y-down coordinates, with normal `n = (sin θ, cos θ)` and
    /// spacing `s` px under `theme`. `k` runs over the whole numbers from
    /// `⌈min n·c / s⌉` to `⌊max n·c / s⌋` over the box's corners `c`, and
    /// each line runs across the box. Families of more than 4,096 lines are
    /// left out.
    pub(crate) fn path(&self, bounds: Rect, origin: Vec2, theme: &Theme) -> BezPath {
        let s = theme.pt(self.spacing);
        let corners = [
            Point::new(bounds.x0, bounds.y0),
            Point::new(bounds.x1, bounds.y0),
            Point::new(bounds.x0, bounds.y1),
            Point::new(bounds.x1, bounds.y1),
        ]
        .map(|c| (c + origin).to_vec2());
        let mut path = BezPath::new();
        for deg in self.angles.iter().filter(|a| a.is_finite()) {
            let (sin, cos) = deg.to_radians().sin_cos();
            let n = Vec2::new(sin, cos);
            let along = Vec2::new(cos, -sin);
            let range = |v: Vec2| {
                corners
                    .iter()
                    .map(|c| c.dot(v))
                    .fold([f64::INFINITY, f64::NEG_INFINITY], |[lo, hi], x| {
                        [lo.min(x), hi.max(x)]
                    })
            };
            let [lo, hi] = range(n);
            let [first, last] = [(lo / s).ceil(), (hi / s).floor()];
            let count = last - first + 1.0;
            if count > MOST || count.is_nan() {
                continue;
            }
            let [t0, t1] = range(along);
            let mut k = first;
            while k <= last {
                let at = n * (k * s) - origin;
                path.move_to((at + along * t0).to_point());
                path.line_to((at + along * t1).to_point());
                k += 1.0;
            }
        }
        path
    }

    /// Appends the hatch of a shape to `list`. The shape is clipped by
    /// `clip` and has the bounding box `bounds`, in a group whose origin
    /// lies at `origin` in the figure's coordinates. The hatch is a group
    /// under `clip` holding one stroke of all its lines, with butt caps and
    /// miter joins, in its color (the theme's ink by default). Nothing is
    /// appended when no line crosses the box.
    pub(crate) fn draw(
        &self,
        clip: Clip,
        bounds: Rect,
        origin: Vec2,
        theme: &Theme,
        list: &mut List,
    ) {
        let path = self.path(bounds, origin, theme);
        if path.elements().is_empty() {
            return;
        }
        list.commands.push(Command::Push {
            transform: kurbo::Affine::IDENTITY,
            clip: Some(clip),
        });
        list.commands.push(Command::Stroke {
            path,
            style: list::stroke(theme.pt(self.width)),
            paint: list::brush(self.color.unwrap_or(theme.get_ink())),
        });
        list.commands.push(Command::Pop);
    }
}

#[cfg(test)]
mod tests {
    use kurbo::PathEl;

    use super::*;

    /// Each line's `k` along normal `n` at spacing `s`: `n·p / s` of its
    /// start, in the figure's coordinates.
    fn offsets(path: &BezPath, n: Vec2, s: f64, origin: Vec2) -> Vec<f64> {
        path.elements()
            .iter()
            .filter_map(|el| match el {
                PathEl::MoveTo(p) => Some((p.to_vec2() + origin).dot(n) / s),
                PathEl::LineTo(_)
                | PathEl::QuadTo(..)
                | PathEl::CurveTo(..)
                | PathEl::ClosePath => None,
            })
            .collect()
    }

    #[test]
    fn a_45_degree_family_over_a_bar_draws_18_lines_8_px_apart() {
        let theme = Theme::default();
        let bar = Rect::new(100.0, 300.0, 200.0, 400.0);
        let path = Hatch::lines(45.0).path(bar, Vec2::ZERO, &theme);
        let n = Vec2::new(0.5_f64.sqrt(), 0.5_f64.sqrt());
        let ks = offsets(&path, n, 8.0, Vec2::ZERO);
        assert_eq!(ks.len(), 18);
        for (k, expected) in ks.iter().zip(36..=53) {
            assert!((k - f64::from(expected)).abs() < 1e-9, "{k}");
        }
        // 45° looks like `/`: up and to the right on screen.
        let PathEl::MoveTo(a) = path.elements()[0] else {
            panic!("a line starts with a move");
        };
        let PathEl::LineTo(b) = path.elements()[1] else {
            panic!("a line is one segment");
        };
        assert!(b.x > a.x && b.y < a.y);
        assert!((theme.pt(0.8) - 1.0667).abs() < 1e-4);
    }

    #[test]
    fn neighbouring_shapes_share_their_lines_across_the_edge() {
        let theme = Theme::default();
        let origin = Vec2::new(40.0, 20.0);
        let n = Vec2::new(0.5_f64.sqrt(), 0.5_f64.sqrt());
        let left = Hatch::lines(45.0).path(Rect::new(60.0, 280.0, 160.0, 380.0), origin, &theme);
        let right = Hatch::lines(45.0).path(Rect::new(160.0, 280.0, 260.0, 380.0), origin, &theme);
        let (a, b) = (
            offsets(&left, n, 8.0, origin),
            offsets(&right, n, 8.0, origin),
        );
        for k in a.iter().chain(&b) {
            assert!((k - k.round()).abs() < 1e-9, "{k}");
        }
        // The lines crossing the shared edge x = 200 in the figure: k from
        // 36 + 13 to 53 on the left are 45 … 53 on the right.
        assert_eq!(a.last().map(|k| k.round()), Some(53.0));
        assert_eq!(b.first().map(|k| k.round()), Some(45.0));
    }

    #[test]
    fn a_cross_adds_the_perpendicular_family() {
        let theme = Theme::default();
        let bar = Rect::new(0.0, 0.0, 16.0, 16.0);
        let lines = Hatch::lines(0.0).path(bar, Vec2::ZERO, &theme);
        let cross = Hatch::cross(0.0).path(bar, Vec2::ZERO, &theme);
        // At 0° the lines are horizontal: y = 0, 8, 16.
        assert_eq!(lines.elements().len(), 6);
        assert_eq!(cross.elements().len(), 12);
    }

    #[test]
    fn rejected_input_leaves_a_hatch_as_it_was() {
        let h = Hatch::lines(45.0);
        assert_eq!(h.clone().spacing(f64::NAN).spacing(-1.0).width(0.0), h);
        assert_eq!(h.clone().color(Rgb::new(f64::NAN, 0.0, 0.0)), h);
        let none = Hatch::lines(f64::INFINITY).path(
            Rect::new(0.0, 0.0, 9.0, 9.0),
            Vec2::ZERO,
            &Theme::default(),
        );
        assert!(none.elements().is_empty());
        let dense = Hatch::lines(45.0).spacing(1e-6);
        assert!(
            dense
                .path(Rect::new(0.0, 0.0, 9.0, 9.0), Vec2::ZERO, &Theme::default())
                .elements()
                .is_empty()
        );
    }
}
