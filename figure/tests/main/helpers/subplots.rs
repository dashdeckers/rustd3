//! `subplots`: a grid of panels with x shared per column and y per row.

use rustd3_figure::Held;
use rustd3_figure::helpers::subplots;
use rustd3_figure::mark::dots;
use rustd3_testkit::obs::{ToObs, Tolerance};

use crate::figure::resolve;

#[test]
fn a_two_by_three_grid_labels_only_its_outer_axes() {
    let mut grid = subplots(2, 3);
    let xs: Vec<_> = (0..3).map(|c| grid.x(c).unwrap()).collect();
    let ys: Vec<_> = (0..2).map(|r| grid.y(r).unwrap()).collect();
    let mut handles: Vec<usize> = xs.iter().chain(&ys).map(|h| h.index()).collect();
    handles.sort_unstable();
    handles.dedup();
    assert_eq!(handles.len(), 5, "x(0..3) and y(0..2) are distinct handles");
    for (row, &y) in ys.iter().enumerate() {
        for (col, &x) in xs.iter().enumerate() {
            let k = (10 * row + col) as f64;
            grid.panel(row, col)
                .unwrap()
                .mark(dots([(k, k), (k + 1.0, k + 2.0)], x, y));
        }
    }
    let fig = grid.into_figure();
    let r = resolve(&fig, &Held::default());
    assert_eq!(r.plot_areas().len(), 6);
    for panel in 0..6 {
        let (row, col) = r.panel_cell(panel).expect("a panel of the grid");
        let bottom = r.plan(panel, 0).expect("a bottom axis");
        let left = r.plan(panel, 1).expect("a left axis");
        assert!(bottom.shown && left.shown, "panel {panel}");
        assert_eq!(bottom.labeled, row == 1, "x labels of ({row}, {col})");
        assert_eq!(left.labeled, col == 0, "y labels of ({row}, {col})");
    }
}

#[test]
fn x_is_shared_per_column_and_y_per_row() {
    let mut grid = subplots(2, 2);
    let [x0, x1] = [0, 1].map(|c| grid.x(c).unwrap());
    let [y0, y1] = [0, 1].map(|r| grid.y(r).unwrap());
    grid.panel(0, 0).unwrap().mark(dots([(0.0, 0.0)], x0, y0));
    grid.panel(1, 0)
        .unwrap()
        .mark(dots([(10.0, 100.0)], x0, y1));
    grid.panel(0, 1).unwrap().mark(dots([(-5.0, 1.0)], x1, y0));
    grid.panel(1, 1).unwrap().mark(dots([(5.0, 50.0)], x1, y1));
    let fig = grid.into_figure();
    let r = resolve(&fig, &Held::default());
    // Fitted with the theme's 5% margins.
    let domains = [x0, x1, y0, y1].map(|h| r.scale(h).unwrap().get_domain());
    let expected = [[-0.5, 10.5], [-5.5, 5.5], [-0.05, 1.05], [47.5, 102.5]];
    assert!(
        domains
            .to_obs()
            .approx_eq(&expected.to_obs(), &Tolerance::default()),
        "{domains:?}"
    );
}

#[test]
fn a_column_row_or_cell_past_the_grid_is_none() {
    let mut grid = subplots(1, 2);
    assert!(grid.x(1).is_some() && grid.x(2).is_none());
    assert!(grid.y(0).is_some() && grid.y(1).is_none());
    assert!(grid.panel(0, 1).is_some());
    assert!(grid.panel(1, 0).is_none() && grid.panel(0, 2).is_none());
    assert_eq!(
        resolve(&grid.into_figure(), &Held::default())
            .plot_areas()
            .len(),
        2
    );
}
