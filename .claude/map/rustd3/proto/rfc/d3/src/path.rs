//! The drawing target every generator writes into: d3's canvas path interface.

use std::f64::consts::{PI, TAU};
use std::fmt::{self, Write};

/// A sink for path commands: canvas-style, as in d3-path and `CanvasRenderingContext2D`.
///
/// Angles are in radians, clockwise from the positive x-axis (screen coordinates).
pub trait PathContext {
    fn move_to(&mut self, x: f64, y: f64);
    fn line_to(&mut self, x: f64, y: f64);
    fn quadratic_curve_to(&mut self, x1: f64, y1: f64, x: f64, y: f64);
    fn bezier_curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64);
    fn arc(&mut self, x: f64, y: f64, r: f64, start: f64, end: f64, anticlockwise: bool);
    fn close_path(&mut self);
}

/// SVG path data, as `d3.path()` produces it.
#[derive(Debug, Clone, Default)]
pub struct SvgPath {
    data: String,
    current: Option<(f64, f64)>,
    start: Option<(f64, f64)>,
}

impl SvgPath {
    pub fn new() -> Self {
        Self::default()
    }

    fn cmd(&mut self, letter: char, values: &[f64]) {
        self.data.push(letter);
        for (i, v) in values.iter().enumerate() {
            if i > 0 {
                self.data.push(',');
            }
            write!(self.data, "{v}").expect("writing to a String cannot fail");
        }
    }
}

impl PathContext for SvgPath {
    fn move_to(&mut self, x: f64, y: f64) {
        (self.start, self.current) = (Some((x, y)), Some((x, y)));
        self.cmd('M', &[x, y]);
    }
    fn line_to(&mut self, x: f64, y: f64) {
        self.current = Some((x, y));
        self.cmd('L', &[x, y]);
    }
    fn quadratic_curve_to(&mut self, x1: f64, y1: f64, x: f64, y: f64) {
        self.current = Some((x, y));
        self.cmd('Q', &[x1, y1, x, y]);
    }
    fn bezier_curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) {
        self.current = Some((x, y));
        self.cmd('C', &[x1, y1, x2, y2, x, y]);
    }
    fn arc(&mut self, x: f64, y: f64, r: f64, a0: f64, a1: f64, ccw: bool) {
        let (dx, dy) = (r * a0.cos(), r * a0.sin());
        let (x0, y0) = (x + dx, y + dy);
        match self.current {
            None => self.cmd('M', &[x0, y0]),
            Some((cx, cy)) if (cx - x0).abs() > 1e-6 || (cy - y0).abs() > 1e-6 => self.cmd('L', &[x0, y0]),
            Some(_) => {}
        }
        let sweep = if ccw { 0.0 } else { 1.0 };
        let mut da = if ccw { a0 - a1 } else { a1 - a0 };
        if r == 0.0 {
            return;
        }
        if da < 0.0 {
            da = da % TAU + TAU;
        }
        if da > TAU - 1e-6 {
            self.cmd('A', &[r, r, 0.0, 1.0, sweep, x - dx, y - dy]);
            self.cmd('A', &[r, r, 0.0, 1.0, sweep, x0, y0]);
            self.current = Some((x0, y0));
        } else if da > 1e-6 {
            let end = (x + r * a1.cos(), y + r * a1.sin());
            self.cmd('A', &[r, r, 0.0, if da >= PI { 1.0 } else { 0.0 }, sweep, end.0, end.1]);
            self.current = Some(end);
        }
    }
    fn close_path(&mut self) {
        if self.current.is_some() {
            self.current = self.start;
            self.data.push('Z');
        }
    }
}

impl fmt::Display for SvgPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.data)
    }
}

/// Renders anything that draws into a [`PathContext`] as SVG path data.
pub fn svg(draw: impl FnOnce(&mut SvgPath)) -> String {
    let mut path = SvgPath::new();
    draw(&mut path);
    path.to_string()
}
