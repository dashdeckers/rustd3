//! Port of d3-path: an SVG path-data builder with the canvas path API.

use std::f64::consts::{PI, TAU};
use std::fmt::{self, Write};

const EPSILON: f64 = 1e-6;

/// Why a path operation was rejected.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    NegativeRadius(f64),
    InvalidDigits(f64),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NegativeRadius(r) => write!(f, "negative radius: {r}"),
            Error::InvalidDigits(d) => write!(f, "invalid digits: {d}"),
        }
    }
}

impl std::error::Error for Error {}

/// Builds SVG path data from canvas-style drawing commands.
#[derive(Debug, Clone, Default)]
pub struct Path {
    start: Option<(f64, f64)>,
    current: Option<(f64, f64)>,
    data: String,
    scale: Option<f64>,
}

impl Path {
    /// A path that prints numbers at full precision.
    pub fn new() -> Self {
        Self::default()
    }

    /// A path that rounds every number to `digits` decimal places (none above 15).
    pub fn with_digits(digits: f64) -> Result<Self, Error> {
        let d = digits.floor();
        if !(d >= 0.0) {
            return Err(Error::InvalidDigits(digits));
        }
        let scale = (d <= 15.0).then(|| 10f64.powi(d as i32));
        Ok(Self { scale, ..Self::default() })
    }

    fn num(&mut self, v: f64) {
        let v = self.scale.map_or(v, |k| (v * k).round() / k);
        write!(self.data, "{v}").expect("writing to a String cannot fail");
    }

    fn command(&mut self, letter: char, values: &[f64]) {
        self.data.push(letter);
        for (i, &v) in values.iter().enumerate() {
            if i > 0 {
                self.data.push(',');
            }
            self.num(v);
        }
    }

    pub fn move_to(&mut self, x: f64, y: f64) {
        self.start = Some((x, y));
        self.current = Some((x, y));
        self.command('M', &[x, y]);
    }

    pub fn close_path(&mut self) {
        if self.current.is_some() {
            self.current = self.start;
            self.data.push('Z');
        }
    }

    pub fn line_to(&mut self, x: f64, y: f64) {
        self.current = Some((x, y));
        self.command('L', &[x, y]);
    }

    pub fn quadratic_curve_to(&mut self, x1: f64, y1: f64, x: f64, y: f64) {
        self.current = Some((x, y));
        self.command('Q', &[x1, y1, x, y]);
    }

    pub fn bezier_curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) {
        self.current = Some((x, y));
        self.command('C', &[x1, y1, x2, y2, x, y]);
    }

    pub fn arc_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, r: f64) -> Result<(), Error> {
        if r < 0.0 {
            return Err(Error::NegativeRadius(r));
        }
        let Some((x0, y0)) = self.current else {
            self.current = Some((x1, y1));
            self.command('M', &[x1, y1]);
            return Ok(());
        };
        let (x21, y21, x01, y01) = (x2 - x1, y2 - y1, x0 - x1, y0 - y1);
        let l01_2 = x01 * x01 + y01 * y01;
        if !(l01_2 > EPSILON) {
            return Ok(());
        }
        if !((y01 * x21 - y21 * x01).abs() > EPSILON) || r == 0.0 {
            self.current = Some((x1, y1));
            self.command('L', &[x1, y1]);
            return Ok(());
        }
        let (x20, y20) = (x2 - x0, y2 - y0);
        let l21_2 = x21 * x21 + y21 * y21;
        let l20_2 = x20 * x20 + y20 * y20;
        let (l21, l01) = (l21_2.sqrt(), l01_2.sqrt());
        let l = r * ((PI - ((l21_2 + l01_2 - l20_2) / (2.0 * l21 * l01)).acos()) / 2.0).tan();
        let (t01, t21) = (l / l01, l / l21);
        if (t01 - 1.0).abs() > EPSILON {
            self.command('L', &[x1 + t01 * x01, y1 + t01 * y01]);
        }
        let sweep = if y01 * x20 > x01 * y20 { 1.0 } else { 0.0 };
        let end = (x1 + t21 * x21, y1 + t21 * y21);
        self.current = Some(end);
        self.command('A', &[r, r, 0.0, 0.0, sweep, end.0, end.1]);
        Ok(())
    }

    pub fn arc(&mut self, x: f64, y: f64, r: f64, a0: f64, a1: f64, ccw: bool) -> Result<(), Error> {
        if r < 0.0 {
            return Err(Error::NegativeRadius(r));
        }
        let (dx, dy) = (r * a0.cos(), r * a0.sin());
        let (x0, y0) = (x + dx, y + dy);
        let cw = if ccw { 0.0 } else { 1.0 };
        let mut da = if ccw { a0 - a1 } else { a1 - a0 };
        match self.current {
            None => self.command('M', &[x0, y0]),
            Some((cx, cy)) if (cx - x0).abs() > EPSILON || (cy - y0).abs() > EPSILON => {
                self.command('L', &[x0, y0])
            }
            Some(_) => {}
        }
        if r == 0.0 {
            return Ok(());
        }
        if da < 0.0 {
            da = da % TAU + TAU;
        }
        if da > TAU - EPSILON {
            self.command('A', &[r, r, 0.0, 1.0, cw, x - dx, y - dy]);
            self.command('A', &[r, r, 0.0, 1.0, cw, x0, y0]);
            self.current = Some((x0, y0));
        } else if da > EPSILON {
            let large = if da >= PI { 1.0 } else { 0.0 };
            let end = (x + r * a1.cos(), y + r * a1.sin());
            self.command('A', &[r, r, 0.0, large, cw, end.0, end.1]);
            self.current = Some(end);
        }
        Ok(())
    }

    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.start = Some((x, y));
        self.current = Some((x, y));
        self.command('M', &[x, y]);
        self.command('h', &[w]);
        self.command('v', &[h]);
        self.command('h', &[-w]);
        self.data.push('Z');
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.data)
    }
}
