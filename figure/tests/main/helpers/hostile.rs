//! The chart presets and statistical helpers over hostile data and
//! parameters: empty, single and repeated values with NaN, infinities and
//! 1e±300; hostile bins, whiskers, cuts, point counts and errors. Each
//! figure is checked and drawn; the statistics are pure functions called
//! directly. Nothing panics.

use rustd3_core::scale;
use rustd3_figure::helpers::{chart, stats, subplots};
use rustd3_figure::kurbo::Size;
use rustd3_figure::mark::Grid;
use rustd3_figure::{Figure, Held, Target};
use rustd3_testkit::hostile::{self, Rng};

use crate::figure::cache;

const CASES: usize = 60;

fn draw(fig: &Figure<'_>, rng: &mut Rng) {
    let size = if rng.chance(0.8) {
        Size::new(320.0, 240.0)
    } else {
        Size::new(rng.f64(), rng.f64())
    };
    fig.check(size);
    fig.draw_with(&mut cache(), size, Target::Vector, &Held::default());
}

fn groups(rng: &mut Rng) -> Vec<(&'static str, Vec<f64>)> {
    (0..rng.small(4))
        .map(|_| (*rng.pick(&["a", "b", "c"]), rng.values(20)))
        .collect()
}

fn bins(rng: &mut Rng) -> stats::hist::Bins {
    match rng.below(4) {
        0 => stats::hist::Bins::Nice(rng.count()),
        1 => stats::hist::Bins::Count(rng.count()),
        2 => stats::hist::Bins::Edges(rng.values(6)),
        _ => stats::hist::Bins::Auto,
    }
}

#[test]
fn presets_and_statistics_never_panic() {
    hostile::run("helpers", 0x4E_19, CASES, |rng| {
        let points: Vec<(f64, f64)> = rng.values(20).into_iter().map(|x| (x, rng.f64())).collect();
        draw(&chart::line(points.clone()).figure(), rng);
        draw(&chart::scatter(points).figure(), rng);
        let columns: Vec<Vec<f64>> = (0..rng.small(3)).map(|_| rng.values(8)).collect();
        draw(&chart::area(columns, rng.values(8)).figure(), rng);
        let slices: Vec<(String, f64)> = rng
            .values(6)
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i.to_string(), v))
            .collect();
        draw(&chart::pie(slices).figure(), rng);
        let (cols, rows) = (rng.small(5), rng.small(5));
        let values = (0..cols * rows).map(|_| rng.f64()).collect();
        if let Ok(grid) = Grid::new(cols, rows, values) {
            draw(&chart::heatmap(grid).figure(), rng);
        }

        let values = rng.values(30);
        let _ = stats::hist::table(&values, None, &bins(rng), rng.chance(0.5), rng.chance(0.5));
        let whis = if rng.chance(0.5) {
            stats::boxplot::Whis::Iqr(rng.f64())
        } else {
            stats::boxplot::Whis::Percentiles(rng.f64(), rng.f64())
        };
        stats::boxplot::summary(&values, whis);

        let mut fig = Figure::new();
        let band = fig.scale(scale::band(["a", "b", "c"]));
        let x = fig.fit(scale::linear());
        let y = fig.fit(scale::linear());
        fig.panel(0, 0)
            .mark(stats::boxplot(groups(rng), band, y).whis(whis))
            .mark(
                stats::violin(groups(rng), band, y)
                    .cut(rng.f64())
                    .points(*rng.pick(&[0, 1, 2, 7, 1 << 31, usize::MAX])),
            )
            .mark(stats::hist(rng.values(30), x, y).bins(bins(rng)))
            .mark(stats::ecdf(rng.values(30), x, y))
            .mark(stats::errorbar(
                (0..rng.small(4)).map(|_| {
                    let e = if rng.chance(0.5) {
                        stats::errorbar::Err::Sym(rng.f64())
                    } else {
                        stats::errorbar::Err::Asym(rng.f64(), rng.f64())
                    };
                    (*rng.pick(&["a", "b", "c"]), rng.f64(), e)
                }),
                band,
                y,
            ));
        draw(&fig, rng);

        let mut grid = subplots(rng.small(4), rng.small(4));
        grid.figure().title("t");
        draw(&grid.into_figure(), rng);
    });
}
