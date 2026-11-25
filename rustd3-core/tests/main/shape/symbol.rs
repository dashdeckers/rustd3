//! Symbols against d3-shape 3.2.0's `symbol(type, 64)()`, run in JavaScript and
//! printed by its default 3-digit path, and the commands they draw.

use rustd3_core::path::{PathContext, SvgPath};
use rustd3_core::shape::Symbol;

/// `symbol` at `size` as d3-shape prints it, at 3 digits.
fn drawn(symbol: Symbol, size: f64) -> String {
    let mut path = SvgPath::with_digits(3);
    symbol.draw(size, &mut path);
    path.to_string()
}

#[test]
fn the_fill_symbols_draw_d3s_paths() {
    let rows = [
        (
            Symbol::Circle,
            "M4.514,0A4.514,4.514,0,1,1,-4.514,0A4.514,4.514,0,1,1,4.514,0",
        ),
        (
            Symbol::Cross,
            "M-5.367,-1.789L-1.789,-1.789L-1.789,-5.367L1.789,-5.367L1.789,-1.789L5.367,-1.789\
             L5.367,1.789L1.789,1.789L1.789,5.367L-1.789,5.367L-1.789,1.789L-5.367,1.789Z",
        ),
        (Symbol::Diamond, "M0,-7.445L4.298,0L0,7.445L-4.298,0Z"),
        (Symbol::Square, "M-4,-4h8v8h-8Z"),
        (
            Symbol::Star,
            "M0,-7.551L1.695,-2.333L7.181,-2.333L2.743,0.891L4.438,6.109L0,2.884\
             L-4.438,6.109L-2.743,0.891L-7.181,-2.333L-1.695,-2.333Z",
        ),
        (Symbol::Triangle, "M0,-7.019L6.079,3.51L-6.079,3.51Z"),
        (
            Symbol::Wye,
            "M2.159,1.246L2.159,5.564L-2.159,5.564L-2.159,1.246L-5.898,-0.912\
             L-3.739,-4.652L0,-2.493L3.739,-4.652L5.898,-0.912Z",
        ),
    ];
    for (symbol, expected) in rows {
        assert_eq!(drawn(symbol, 64.0), expected, "{symbol:?}");
    }
}

#[test]
fn fill_lists_d3s_symbols_fill_in_order() {
    assert_eq!(
        Symbol::FILL,
        [
            Symbol::Circle,
            Symbol::Cross,
            Symbol::Diamond,
            Symbol::Square,
            Symbol::Star,
            Symbol::Triangle,
            Symbol::Wye
        ]
    );
}

#[test]
fn the_hexagon_draws_observable_plots_path_with_a_circles_half_width() {
    // Plot's symbolHexagon at size 64: rx = √(64 / π), ry = rx · 2 / √3.
    assert_eq!(
        drawn(Symbol::Hexagon, 64.0),
        "M0,5.212L4.514,2.606L4.514,-2.606L0,-5.212L-4.514,-2.606L-4.514,2.606Z"
    );
}

#[test]
fn only_plus_times_and_asterisk_are_open() {
    let open: Vec<Symbol> = Symbol::FILL
        .into_iter()
        .chain(Symbol::STROKE)
        .chain([Symbol::Hexagon])
        .filter(|s| s.is_open())
        .collect();
    assert_eq!(open, [Symbol::Plus, Symbol::Times, Symbol::Asterisk]);
}

/// Records each `PathContext` call by name, its own `rect` included.
#[derive(Default)]
struct Calls(Vec<&'static str>);

impl PathContext for Calls {
    fn move_to(&mut self, _x: f64, _y: f64) {
        self.0.push("move_to");
    }

    fn line_to(&mut self, _x: f64, _y: f64) {
        self.0.push("line_to");
    }

    fn quadratic_curve_to(&mut self, _x1: f64, _y1: f64, _x: f64, _y: f64) {
        self.0.push("quadratic_curve_to");
    }

    fn bezier_curve_to(&mut self, _x1: f64, _y1: f64, _x2: f64, _y2: f64, _x: f64, _y: f64) {
        self.0.push("bezier_curve_to");
    }

    fn arc(&mut self, _x: f64, _y: f64, _r: f64, _start: f64, _end: f64, _anticlockwise: bool) {
        self.0.push("arc");
    }

    fn close_path(&mut self) {
        self.0.push("close_path");
    }

    fn rect(&mut self, _x: f64, _y: f64, _w: f64, _h: f64) {
        self.0.push("rect");
    }
}

/// [`Calls`] without a `rect` of its own: the trait's provided one.
#[derive(Default)]
struct Provided(Calls);

impl PathContext for Provided {
    fn move_to(&mut self, x: f64, y: f64) {
        self.0.move_to(x, y);
    }

    fn line_to(&mut self, x: f64, y: f64) {
        self.0.line_to(x, y);
    }

    fn quadratic_curve_to(&mut self, x1: f64, y1: f64, x: f64, y: f64) {
        self.0.quadratic_curve_to(x1, y1, x, y);
    }

    fn bezier_curve_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) {
        self.0.bezier_curve_to(x1, y1, x2, y2, x, y);
    }

    fn arc(&mut self, x: f64, y: f64, r: f64, start: f64, end: f64, anticlockwise: bool) {
        self.0.arc(x, y, r, start, end, anticlockwise);
    }

    fn close_path(&mut self) {
        self.0.close_path();
    }
}

#[test]
fn a_circle_is_one_move_and_one_arc_call_and_a_square_one_rect() {
    let mut circle = Calls::default();
    Symbol::Circle.draw(64.0, &mut circle);
    assert_eq!(circle.0, ["move_to", "arc"]);
    let mut square = Calls::default();
    Symbol::Square.draw(64.0, &mut square);
    assert_eq!(square.0, ["rect"]);
}

#[test]
fn the_provided_rect_is_a_move_three_lines_and_a_close() {
    let mut square = Provided::default();
    Symbol::Square.draw(64.0, &mut square);
    assert_eq!(
        square.0.0,
        ["move_to", "line_to", "line_to", "line_to", "close_path"]
    );
    let mut path = SvgPath::new();
    PathContext::rect(&mut path, -4.0, -4.0, 8.0, 8.0);
    assert_eq!(path.to_string(), "M-4,-4h8v8h-8Z");
}

#[test]
fn a_zero_size_draws_at_the_origin() {
    assert_eq!(drawn(Symbol::Circle, 0.0), "M0,0");
    assert_eq!(drawn(Symbol::Square, 0.0), "M0,0h0v0h0Z");
}
