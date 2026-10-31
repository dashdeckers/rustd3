//! Every line, area and curve generator, arcs, pies, stacks and symbols on
//! random point sets with NaN, infinities, repeated points, one point or
//! none. Nothing panics, and the SVG path data is well formed: each
//! command letter followed by its number of arguments, starting with a
//! move.

use rustd3_core::path::{self, PathContext};
use rustd3_core::shape::stack::{Offset, Order};
use rustd3_core::shape::{self, Curve, Symbol};
use rustd3_testkit::hostile::{self, Rng};

const CASES: usize = 500;

/// Whether `d` is SVG path data as `SvgPath` writes it: command letters
/// with comma-separated JavaScript numbers (`NaN` and `Infinity`
/// included), each command with its arity, the first a move.
fn well_formed(d: &str) -> Result<(), String> {
    let arity = |c: char| match c {
        'M' | 'L' => Some(2),
        'Q' => Some(4),
        'C' => Some(6),
        'A' => Some(7),
        'h' | 'v' => Some(1),
        'Z' => Some(0),
        _ => None,
    };
    let mut rest = d;
    let mut first = true;
    while let Some(c) = rest.chars().next() {
        let n = arity(c).ok_or_else(|| format!("unknown command {c:?} in {d}"))?;
        if first && c != 'M' {
            return Err(format!("starts with {c:?}: {d}"));
        }
        first = false;
        rest = &rest[c.len_utf8()..];
        let end = rest
            .find(|ch: char| arity(ch).is_some())
            .unwrap_or(rest.len());
        let args = &rest[..end];
        let count = if args.is_empty() {
            0
        } else {
            args.split(',').count()
        };
        if count != n {
            return Err(format!("{c} with {count} arguments ({args}) in {d}"));
        }
        if args
            .split(',')
            .filter(|a| !a.is_empty())
            .any(|a| a.parse::<f64>().is_err() && !matches!(a, "NaN" | "Infinity" | "-Infinity"))
        {
            return Err(format!("{c} with a malformed number ({args})"));
        }
        rest = &rest[end..];
    }
    Ok(())
}

/// A path context that records whether anything drew before a move.
#[derive(Default)]
struct Moves {
    moved: bool,
    stray: Option<&'static str>,
}

impl Moves {
    fn draw(&mut self, what: &'static str) {
        if !self.moved && self.stray.is_none() {
            self.stray = Some(what);
        }
    }
}

impl PathContext for Moves {
    fn move_to(&mut self, _: f64, _: f64) {
        self.moved = true;
    }

    fn line_to(&mut self, _: f64, _: f64) {
        self.draw("line_to");
    }

    fn quadratic_curve_to(&mut self, _: f64, _: f64, _: f64, _: f64) {
        self.draw("quadratic_curve_to");
    }

    fn bezier_curve_to(&mut self, _: f64, _: f64, _: f64, _: f64, _: f64, _: f64) {
        self.draw("bezier_curve_to");
    }

    fn arc(&mut self, _: f64, _: f64, _: f64, _: f64, _: f64, _: bool) {
        // An arc starts its own subpath when none is open.
        self.moved = true;
    }

    fn close_path(&mut self) {}
}

fn curve(rng: &mut Rng) -> Curve {
    let p = if rng.chance(0.5) {
        rng.f64()
    } else {
        rng.unit()
    };
    match rng.below(21) {
        0 => Curve::Linear,
        1 => Curve::LinearClosed,
        2 => Curve::Step,
        3 => Curve::StepBefore,
        4 => Curve::StepAfter,
        5 => Curve::Basis,
        6 => Curve::BasisClosed,
        7 => Curve::BasisOpen,
        8 => Curve::BumpX,
        9 => Curve::BumpY,
        10 => Curve::Bundle { beta: p },
        11 => Curve::Cardinal { tension: p },
        12 => Curve::CardinalClosed { tension: p },
        13 => Curve::CardinalOpen { tension: p },
        14 => Curve::CatmullRom { alpha: p },
        15 => Curve::CatmullRomClosed { alpha: p },
        16 => Curve::CatmullRomOpen { alpha: p },
        17 => Curve::MonotoneX,
        18 => Curve::MonotoneY,
        _ => Curve::Natural,
    }
}

fn points(rng: &mut Rng) -> Vec<Option<(f64, f64)>> {
    let xs = rng.values(12);
    let repeat = rng.chance(0.2);
    let mut last = (0.0, 0.0);
    xs.into_iter()
        .map(|x| {
            if rng.chance(0.1) {
                return None;
            }
            if repeat && rng.chance(0.5) {
                return Some(last);
            }
            last = (
                x,
                if rng.chance(0.3) {
                    rng.f64()
                } else {
                    rng.plain()
                },
            );
            Some(last)
        })
        .collect()
}

#[test]
fn lines_and_areas_draw_well_formed_paths_from_any_points() {
    hostile::run("shapes", 0x5A_9E, CASES, |rng| {
        let c = curve(rng);
        let pts = points(rng);
        let line = shape::line().curve(c);
        let d = path::svg(|p| line.draw(pts.iter().copied(), p));
        if let Err(e) = well_formed(&d) {
            panic!("line {c:?} over {pts:?}: {e}");
        }
        let mut order = Moves::default();
        line.draw(pts.iter().copied(), &mut order);
        assert_eq!(
            order.stray, None,
            "line {c:?} drew before a move over {pts:?}"
        );

        let area = shape::area().curve(c).y0(rng.f64());
        let d = path::svg(|p| area.draw(pts.iter().copied(), p));
        if let Err(e) = well_formed(&d) {
            panic!("area {c:?} over {pts:?}: {e}");
        }
        let spans: Vec<_> = pts
            .iter()
            .map(|p| p.map(|(x, y)| (x, y, rng.f64())))
            .collect();
        let d = path::svg(|p| area.draw(spans.iter().copied(), p));
        if let Err(e) = well_formed(&d) {
            panic!("area {c:?} over {spans:?}: {e}");
        }
        let mut order = Moves::default();
        area.draw(spans.iter().copied(), &mut order);
        assert_eq!(order.stray, None, "area {c:?} drew before a move");

        let digits = u32::try_from(rng.below(40)).unwrap_or(0);
        let mut fixed = path::SvgPath::with_digits(digits);
        line.draw(pts.iter().copied(), &mut fixed);
        if let Err(e) = well_formed(&fixed.to_string()) {
            panic!("line {c:?} at {digits} digits: {e}");
        }
    });
}

#[test]
fn arcs_pies_stacks_and_symbols_never_panic() {
    hostile::run("arcs", 0xA2_C5, CASES, |rng| {
        let values = rng.values(10);
        let pie = shape::pie()
            .start_angle(rng.f64())
            .end_angle(rng.f64())
            .pad_angle(rng.f64());
        let slices = pie.layout(values.iter().copied());
        let arc = shape::arc()
            .inner_radius(rng.f64())
            .outer_radius(rng.f64())
            .corner_radius(rng.f64())
            .pad_radius(rng.chance(0.5).then(|| rng.f64()))
            .pad_angle(rng.chance(0.5).then(|| rng.f64()));
        for slice in &slices {
            let d = path::svg(|p| arc.draw(slice, p));
            if let Err(e) = well_formed(&d) {
                panic!("arc of {slice:?}: {e}");
            }
            let _ = arc.centroid(slice);
        }
        let mut svg = path::SvgPath::new();
        let _ = svg.try_arc(
            rng.f64(),
            rng.f64(),
            rng.f64(),
            rng.f64(),
            rng.f64(),
            rng.chance(0.5),
        );
        let _ = svg.arc_to(rng.f64(), rng.f64(), rng.f64(), rng.f64(), rng.f64());
        svg.rect(rng.f64(), rng.f64(), rng.f64(), rng.f64());
        if let Err(e) = well_formed(&svg.to_string()) {
            panic!("arcs: {e}");
        }

        let columns: Vec<Vec<f64>> = (0..rng.small(4)).map(|_| rng.values(6)).collect();
        let refs: Vec<&[f64]> = columns.iter().map(Vec::as_slice).collect();
        let _ = shape::stack(&refs);
        let order = *rng.pick(&[
            Order::None,
            Order::Ascending,
            Order::Descending,
            Order::InsideOut,
            Order::Appearance,
            Order::Reverse,
        ]);
        let offset = *rng.pick(&[
            Offset::None,
            Offset::Expand,
            Offset::Diverging,
            Offset::Silhouette,
            Offset::Wiggle,
        ]);
        let stack = shape::Stack::default().order(order).offset(offset);
        let _ = stack.layout(&refs);
        let _ = stack.stacking_order(&refs);
        let symbol = *rng.pick(&[
            Symbol::Circle,
            Symbol::Cross,
            Symbol::Diamond,
            Symbol::Square,
            Symbol::Star,
            Symbol::Triangle,
            Symbol::Wye,
        ]);
        let d = path::svg(|p| symbol.draw(rng.f64(), p));
        if let Err(e) = well_formed(&d) {
            panic!("{symbol:?}: {e}");
        }
    });
}
