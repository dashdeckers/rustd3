//! Densities: the kernel density estimate against scipy and seaborn
//! (`dev/fixtures/kde.json`, written by `kde.py`), and the marks that draw
//! it along y and along x.

use rustd3_core::scale;
use rustd3_core::scheme::COLORBLIND;
use rustd3_figure::kurbo::{BezPath, Rect, Shape};
use rustd3_figure::list::{Command, List};
use rustd3_figure::mark::{density_x, density_y};
use rustd3_figure::peniko::Brush;
use rustd3_figure::stats::Kde;
use rustd3_figure::{Figure, Held};
use rustd3_testkit::obs::{Obs, ToObs, Tolerance};
use serde_json::Value;

use super::{draw, marked, solid};
use crate::figure::resolve;

/// The fixture, read at compile time so the test needs no working directory.
const KDE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../dev/fixtures/kde.json"
));

/// The section's statistical data, n = 15.
const X: [f64; 15] = [
    1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 4.7, 5.1, 5.6, 6.2, 7.9, 9.5, 14.0,
];

/// `value` as an `Obs`.
fn obs(value: &Value) -> Obs {
    Obs::try_from(value).expect("numbers")
}

/// `value` as a number.
fn num(value: &Value) -> f64 {
    value.as_f64().expect("a number")
}

#[test]
fn the_kde_matches_scipys_gaussian_kde_and_seaborns_grid() {
    let fixture: Value = serde_json::from_str(KDE).expect("the fixture");
    let tolerance = Tolerance::relative(1e-9);
    assert!(X.to_obs().strict_eq(&obs(&fixture["values"])));
    let kde = Kde::new(X).expect("values that vary");
    let h = kde.bandwidth();
    let factor = 15f64.powf(-0.2);
    let sides = [h / factor, factor, h].to_obs();
    let expected = [
        &fixture["deviation"],
        &fixture["factor"],
        &fixture["bandwidth"],
    ]
    .map(num);
    assert!(sides.approx_eq(&expected.to_obs(), &tolerance), "{sides:?}");

    let points: Vec<f64> = fixture["points"]
        .as_array()
        .expect("points")
        .iter()
        .map(num)
        .collect();
    let density: Vec<f64> = points.iter().map(|&x| kde.eval(x)).collect();
    assert!(
        density
            .to_obs()
            .approx_eq(&obs(&fixture["density"]), &tolerance),
        "{density:?}"
    );

    let grid = &fixture["grid"];
    let ours = kde.grid(200, 3.0);
    let support: Vec<f64> = ours.iter().map(|(x, _)| *x).collect();
    let values: Vec<f64> = ours.iter().map(|(_, d)| *d).collect();
    assert!(
        support
            .to_obs()
            .approx_eq(&obs(&grid["support"]), &tolerance)
    );
    assert!(
        values
            .to_obs()
            .approx_eq(&obs(&grid["density"]), &tolerance)
    );
    let peak = ours
        .iter()
        .copied()
        .fold((f64::NAN, 0.0), |a, b| if b.1 > a.1 { b } else { a });
    assert!(
        [peak.0, peak.1]
            .to_obs()
            .approx_eq(&obs(&grid["peak"]), &tolerance)
    );
}

#[test]
fn the_kde_spot_rows_hold_and_the_prototypes_bandwidth_is_not_used() {
    let kde = Kde::new(X).expect("values that vary");
    let h = kde.bandwidth();
    let close = |a: f64, b: f64| (a - b).abs() < 5e-7;
    assert!(
        close(h / 15f64.powf(-0.2), 3.229654) && close(h, 1.879048),
        "{h}"
    );
    assert!((h - 1.924).abs() > 0.04, "Silverman's 1.06·σ_pop·n^−0.2");
    let rows = [
        (0.0, 0.034339),
        (2.0, 0.097612),
        (4.0, 0.138075),
        (4.4, 0.136582),
        (6.0, 0.103621),
        (10.0, 0.026567),
        (14.0, 0.015035),
    ];
    for (x, d) in rows {
        assert!(close(kde.eval(x), d), "eval({x}) = {}", kde.eval(x));
    }
    let grid = kde.grid(200, 3.0);
    assert!(close(grid[0].0, -4.437143) && close(grid[199].0, 19.637143));
    assert!(close(grid[1].0 - grid[0].0, 0.120976));
    let peak = grid
        .iter()
        .copied()
        .fold((0.0, 0.0), |a, b| if b.1 > a.1 { b } else { a });
    assert!(
        close(peak.0, 4.031199) && close(peak.1, 0.138086),
        "{peak:?}"
    );
}

/// The fills and strokes a list's marks drew, in order.
fn shapes(list: &List) -> Vec<(&BezPath, &Brush, Option<f64>)> {
    marked(list)
        .into_iter()
        .filter_map(|c| {
            if let Command::Fill { path, paint } = c {
                Some((path, paint, None))
            } else if let Command::Stroke { path, style, paint } = c {
                Some((path, paint, Some(style.width)))
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn a_vertical_density_fills_from_zero_width_out_to_its_peak() {
    let mut fig = Figure::new();
    let y = fig.fit(scale::linear());
    let width = fig.scale(scale::linear().domain([0.0, 1.0]));
    let color = fig.scale(scale::ordinal(["a", "b"], COLORBLIND));
    fig.panel(0, 0)
        .mark(density_y(X, y, width).fill(color, "b"));
    let resolved = resolve(&fig, &Held::default());
    let dy = resolved.scale(y).unwrap().get_domain();
    assert!(
        (dy[0] - 0.56).abs() < 1e-12 && (dy[1] - 14.64).abs() < 1e-12,
        "{dy:?}"
    );
    let area = resolved.plot_areas()[0];
    let list = draw(&fig);
    let drawn = shapes(&list);
    assert_eq!(drawn.len(), 2);
    let (fill, paint, _) = drawn[0];
    assert_eq!(*paint, solid(COLORBLIND[1].with_opacity(0.35)));
    let (outline, ink, width) = drawn[1];
    assert_eq!((outline, ink), (fill, &solid(COLORBLIND[1])));
    assert!((width.expect("a stroke") - 1.0).abs() < 1e-12, "0.75 pt");
    let bounds: Rect = fill.bounding_box();
    assert!(
        bounds.x0.abs() < 1e-9 && (bounds.x1 - area.width()).abs() < 1e-9,
        "{bounds:?}"
    );
    assert!(
        bounds.y0 < 0.0 && bounds.y1 > area.height(),
        "the tails run past the data"
    );
}

#[test]
fn bandwidth_and_adjust_reshape_the_curve() {
    let extent = |adjust: Option<f64>, h: Option<f64>| {
        let mut fig = Figure::new();
        let x = fig.scale(scale::linear().domain([-20.0, 40.0]));
        let height = fig.scale(scale::linear().domain([0.0, 1.0]));
        let mut mark = density_x(X, x, height);
        if let Some(h) = h {
            mark = mark.bandwidth(h).bandwidth(f64::NAN);
        }
        if let Some(f) = adjust {
            mark = mark.adjust(f).adjust(0.0);
        }
        fig.panel(0, 0).mark(mark);
        let w = resolve(&fig, &Held::default()).plot_areas()[0].width();
        let list = draw(&fig);
        let b = shapes(&list)[0].0.bounding_box();
        [b.x0 / w * 60.0 - 20.0, b.x1 / w * 60.0 - 20.0]
    };
    let scott = Kde::new(X).expect("values that vary").bandwidth();
    let close = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9;
    let span = |h: f64| [1.2 - 3.0 * h, 14.0 + 3.0 * h];
    assert!(close(extent(None, None), span(scott)));
    assert!(close(extent(Some(2.0), None), span(2.0 * scott)));
    assert!(close(extent(None, Some(0.5)), span(0.5)));
    assert!(close(extent(Some(2.0), Some(0.5)), span(1.0)));
}

#[test]
fn a_density_of_one_value_draws_nothing() {
    let mut fig = Figure::new();
    let y = fig.fit(scale::linear());
    let width = fig.scale(scale::linear().domain([0.0, 1.0]));
    fig.panel(0, 0)
        .mark(density_y([3.0, f64::NAN], y, width))
        .mark(density_y([2.0, 2.0], y, width));
    assert!(shapes(&draw(&fig)).is_empty());
    #[cfg(feature = "fonts")]
    {
        use rustd3_figure::check::{Issue, size};
        use rustd3_figure::mark::Error;
        let issue = |mark, values| Issue::Mark {
            panel: 0,
            mark,
            error: Error::Density { values },
        };
        assert_eq!(fig.check(size(640.0, 480.0)), [issue(0, 1), issue(1, 2)]);
    }
}
