//! d3-path: the canvas path interface every generator draws into, and SVG
//! path data as `d3.path()` prints it.
//!
//! ```
//! use rustd3_core::path::{self, PathContext};
//!
//! let d = path::svg(|p| {
//!     p.move_to(150.0, 50.0);
//!     p.line_to(200.0, 100.0);
//! });
//! assert_eq!(d, "M150,50L200,100");
//! ```

mod error;
mod svg;

pub use error::Error;
pub use svg::SvgPath;

/// d3's canvas path interface: the one drawing seam between generators and
/// backends. Coordinates are in px, y pointing down.
pub trait PathContext {
    /// Starts a new subpath at (`x`, `y`).
    fn move_to(&mut self, x: f64, y: f64);
    /// Adds a straight line from the current point to (`x`, `y`).
    fn line_to(&mut self, x: f64, y: f64);
    /// Adds a quadratic Bézier curve through control point (`x1`, `y1`) to
    /// (`x`, `y`).
    fn quadratic_curve_to(&mut self, x1: f64, y1: f64, x: f64, y: f64);
    /// Adds a cubic Bézier curve through control points (`x1`, `y1`) and
    /// (`x2`, `y2`) to (`x`, `y`).
    fn bezier_curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64);
    /// Adds a circular arc around (`x`, `y`) from angle `start` to `end`,
    /// joined to the current point by a line. Angles are in radians,
    /// clockwise from +x in screen coordinates. `r` must be ≥ 0.
    fn arc(&mut self, x: f64, y: f64, r: f64, start: f64, end: f64, anticlockwise: bool);
    /// Closes the current subpath with a line back to its start.
    fn close_path(&mut self);
    /// d3's rect: adds a closed rectangle from (`x`, `y`), `w` wide and `h`
    /// high, as a move, three lines and a close. [`SvgPath`] prints it as d3-path
    /// does, `M{x},{y}h{w}v{h}h{−w}Z`.
    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.move_to(x, y);
        self.line_to(x + w, y);
        self.line_to(x + w, y + h);
        self.line_to(x, y + h);
        self.close_path();
    }
}

/// Draws with `draw` into a fresh [`SvgPath`] and returns its data.
pub fn svg(draw: impl FnOnce(&mut SvgPath)) -> String {
    let mut path = SvgPath::new();
    draw(&mut path);
    path.to_string()
}
