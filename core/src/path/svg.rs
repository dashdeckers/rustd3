//! SVG path data, as `d3.path()` and `d3.pathRound()` print it.

use std::f64::consts::{PI, TAU};
use std::fmt::{self, Write};

use crate::js;
use crate::path::{Error, PathContext};

/// d3's tolerance for coincident points and empty arcs.
const EPSILON: f64 = 1e-6;
/// An arc sweeping more than this is a full circle.
const TAU_EPSILON: f64 = TAU - EPSILON;
/// `10^digits` for the digits `with_digits` rounds to.
const POWERS: [f64; 16] = [
    1e0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15,
];

/// SVG path data, as `d3.path()` prints it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SvgPath {
    data: String,
    /// The start of the current subpath, where `close_path` returns.
    start: Option<(f64, f64)>,
    /// The end of the current subpath.
    current: Option<(f64, f64)>,
    /// `10^digits` when numbers are rounded.
    scale: Option<f64>,
}

impl fmt::Display for SvgPath {
    /// The path data.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.data)
    }
}

impl PathContext for SvgPath {
    fn move_to(&mut self, x: f64, y: f64) {
        self.start = Some((x, y));
        self.current = Some((x, y));
        self.push('M', &[x, y]);
    }

    fn line_to(&mut self, x: f64, y: f64) {
        self.current = Some((x, y));
        self.push('L', &[x, y]);
    }

    fn quadratic_curve_to(&mut self, x1: f64, y1: f64, x: f64, y: f64) {
        self.current = Some((x, y));
        self.push('Q', &[x1, y1, x, y]);
    }

    fn bezier_curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) {
        self.current = Some((x, y));
        self.push('C', &[x1, y1, x2, y2, x, y]);
    }

    /// d3's arc; a negative radius draws nothing ([`SvgPath::try_arc`]
    /// reports it).
    fn arc(&mut self, x: f64, y: f64, r: f64, start: f64, end: f64, anticlockwise: bool) {
        self.try_arc(x, y, r, start, end, anticlockwise)
            .unwrap_or_default();
    }

    fn close_path(&mut self) {
        if self.current.is_some() {
            self.current = self.start;
            self.data.push('Z');
        }
    }

    /// d3-path's rect: `M{x},{y}h{w}v{h}h{−w}Z` ([`SvgPath::rect`]).
    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        SvgPath::rect(self, x, y, w, h);
    }
}

impl SvgPath {
    /// An empty path that prints numbers in full: `d3.path()`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// d3.pathRound(digits): a path that rounds numbers to `digits` places,
    /// or leaves them unrounded when `digits` is above 15.
    #[must_use]
    pub fn with_digits(digits: usize) -> Self {
        let scale = POWERS.get(digits).copied();
        Self {
            scale,
            ..Self::default()
        }
    }

    /// Adds an arc as [`PathContext::arc`] does, but reports a negative
    /// radius (where d3 throws) instead of drawing nothing.
    ///
    /// # Errors
    ///
    /// [`Error::NegativeRadius`](crate::path::Error::NegativeRadius) for
    /// `r < 0`, with the path unchanged.
    pub fn try_arc(
        &mut self,
        x: f64,
        y: f64,
        r: f64,
        start: f64,
        end: f64,
        anticlockwise: bool,
    ) -> Result<(), Error> {
        if r < 0.0 {
            return Err(Error::NegativeRadius(r));
        }
        let (dx, dy) = (r * start.cos(), r * start.sin());
        let (x0, y0) = (x + dx, y + dy);
        let cw = if anticlockwise { 0.0 } else { 1.0 };
        let mut da = if anticlockwise {
            start - end
        } else {
            end - start
        };
        match self.current {
            None => self.push('M', &[x0, y0]),
            Some((x1, y1)) => {
                if (x1 - x0).abs() > EPSILON || (y1 - y0).abs() > EPSILON {
                    self.push('L', &[x0, y0]);
                }
            }
        }
        if r == 0.0 || r.is_nan() {
            return Ok(());
        }
        if da < 0.0 {
            da = da % TAU + TAU;
        }
        if da > TAU_EPSILON {
            // A full circle is two half-circle arcs; one arc to its own start draws nothing.
            self.push('A', &[r, r, 0.0, 1.0, cw, x - dx, y - dy]);
            self.push('A', &[r, r, 0.0, 1.0, cw, x0, y0]);
            self.current = Some((x0, y0));
        } else if da > EPSILON {
            let large = if da >= PI { 1.0 } else { 0.0 };
            let to = (x + r * end.cos(), y + r * end.sin());
            self.push('A', &[r, r, 0.0, large, cw, to.0, to.1]);
            self.current = Some(to);
        }
        Ok(())
    }

    /// d3's arcTo: a line toward (`x1`, `y1`), then an arc of radius `r`
    /// tangent to the lines through (`x1`, `y1`) and (`x2`, `y2`).
    ///
    /// # Errors
    ///
    /// [`Error::NegativeRadius`](crate::path::Error::NegativeRadius) for
    /// `r < 0`, with the path unchanged.
    pub fn arc_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, r: f64) -> Result<(), Error> {
        if r < 0.0 {
            return Err(Error::NegativeRadius(r));
        }
        let Some((x0, y0)) = self.current else {
            self.current = Some((x1, y1));
            self.push('M', &[x1, y1]);
            return Ok(());
        };
        let (x21, y21) = (x2 - x1, y2 - y1);
        let (x01, y01) = (x0 - x1, y0 - y1);
        let l01_2 = x01 * x01 + y01 * y01;
        // (x1, y1) coincides with the current point: nothing to draw.
        if l01_2.is_nan() || l01_2 <= EPSILON {
            return Ok(());
        }
        // Collinear points, (x1, y1) on (x2, y2), or no radius: a line.
        let cross = (y01 * x21 - y21 * x01).abs();
        if cross.is_nan() || cross <= EPSILON || r == 0.0 || r.is_nan() {
            self.current = Some((x1, y1));
            self.push('L', &[x1, y1]);
            return Ok(());
        }
        let (x20, y20) = (x2 - x0, y2 - y0);
        let l21_2 = x21 * x21 + y21 * y21;
        let l20_2 = x20 * x20 + y20 * y20;
        let (l21, l01) = (l21_2.sqrt(), l01_2.sqrt());
        let l = r * ((PI - ((l21_2 + l01_2 - l20_2) / (2.0 * l21 * l01)).acos()) / 2.0).tan();
        let (t01, t21) = (l / l01, l / l21);
        if (t01 - 1.0).abs() > EPSILON {
            self.push('L', &[x1 + t01 * x01, y1 + t01 * y01]);
        }
        let sweep = if y01 * x20 > x01 * y20 { 1.0 } else { 0.0 };
        let to = (x1 + t21 * x21, y1 + t21 * y21);
        self.current = Some(to);
        self.push('A', &[r, r, 0.0, 0.0, sweep, to.0, to.1]);
        Ok(())
    }

    /// d3's rect: `M{x},{y}h{w}v{h}h{−w}Z`.
    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.start = Some((x, y));
        self.current = Some((x, y));
        self.push('M', &[x, y]);
        self.push('h', &[w]);
        self.push('v', &[h]);
        self.push('h', &[-w]);
        self.data.push('Z');
    }

    /// Appends `command` and its comma-separated numbers.
    fn push(&mut self, command: char, values: &[f64]) {
        self.data.push(command);
        for (i, &value) in values.iter().enumerate() {
            if i > 0 {
                self.data.push(',');
            }
            let value = self.scale.map_or(value, |k| js::round(value * k) / k);
            write!(self.data, "{}", js::Number(value))
                .expect("invariant: writing to a String cannot fail");
        }
    }
}
