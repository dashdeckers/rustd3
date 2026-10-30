//! Random figures with hostile data, parameters and sizes: panels in odd
//! cells and spans, every mark kind fed NaN, ±infinity, 1e±300, empty and
//! one-element data, axes with hostile counts and rotations, legends and
//! colorbars, odd themes and grids. Each is checked with `fig.check`,
//! drawn, lowered and written as SVG, rendered at scale 1 and written as
//! PDF. Nothing panics.

use rustd3_core::color::Rgb;
use rustd3_core::scale::{self, Linear};
use rustd3_core::scheme;
use rustd3_core::shape::Curve;
use rustd3_figure::colorbar::{Extend, colorbar};
use rustd3_figure::kurbo::{Point, Rect, Size};
use rustd3_figure::layout::{Cell, Grid, em, px};
use rustd3_figure::legend::{Inset, Slot, legend};
use rustd3_figure::list::{Bounds, Outlines};
use rustd3_figure::mark::{self, Bins};
use rustd3_figure::scale::{Fit, ScaleRef};
use rustd3_figure::theme::{Context, Style};
use rustd3_figure::{Figure, Held, Target, Theme, axis, lower, svg};
use rustd3_testkit::hostile::{self, Rng};

use crate::figure::cache;

const CASES: usize = 200;

fn points(rng: &mut Rng, max: usize) -> Vec<(f64, f64)> {
    let xs = rng.values(max);
    xs.into_iter()
        .map(|x| {
            let y = if rng.chance(0.3) {
                rng.f64()
            } else {
                rng.plain()
            };
            (x, y)
        })
        .collect()
}

fn size(rng: &mut Rng) -> Size {
    let side = |rng: &mut Rng| match rng.below(6) {
        0 => rng.f64(),
        1 => rng.unit() * 4.0,
        _ => 100.0 + rng.unit() * 700.0,
    };
    Size::new(side(rng), side(rng))
}

fn linear(rng: &mut Rng, fig: &mut Figure<'_>) -> ScaleRef<Linear> {
    match rng.below(4) {
        0 => fig.scale(scale::linear().domain([rng.f64(), rng.f64()])),
        1 => fig.fit_with(scale::linear(), Fit::Window(rng.f64())),
        2 => fig.fit_with(scale::linear(), Fit::Visible),
        _ => fig.fit(scale::linear()),
    }
}

/// A random figure over the data the generator draws.
fn figure(rng: &mut Rng) -> Figure<'static> {
    let mut fig = Figure::new();
    if rng.chance(0.3) {
        fig.theme(
            Theme::seaborn(
                *rng.pick(&[Style::DarkGrid, Style::WhiteGrid, Style::Ticks]),
                *rng.pick(&[Context::Paper, Context::Poster]),
            )
            .font_scale(rng.f64()),
        );
    }
    if rng.chance(0.3) {
        let weights: Vec<f64> = rng.values(3);
        let gaps: Vec<_> = rng
            .values(3)
            .into_iter()
            .map(|v| if rng.chance(0.5) { em(v) } else { px(v) })
            .collect();
        fig.layout(
            Grid::new()
                .rows(weights.clone())
                .cols(weights)
                .row_gaps(gaps.clone())
                .col_gaps(gaps),
        );
    }
    if rng.chance(0.3) {
        fig.title(rng.string("ab $^_\\#", 12));
    }
    let x = linear(rng, &mut fig);
    let y = linear(rng, &mut fig);
    let logx = fig.fit(scale::log());
    let color = fig.scale(scale::ordinal(["a", "b", "c"], scheme::COLORBLIND));
    let heat = fig.scale(scale::sequential([rng.f64(), rng.f64()], scheme::viridis));
    let band = fig.scale(scale::band(["p", "q", "r"]).padding(rng.f64()));
    for _ in 0..1 + rng.below(3) {
        let cell = Cell::new(rng.small(3), rng.small(3)).span(rng.small(3), rng.small(3));
        let panel = fig.panel_at(cell);
        for _ in 0..rng.below(4) {
            match rng.below(13) {
                0 => {
                    let data = points(rng, 30);
                    let n = data.len();
                    let mut d = mark::dots(data, x, y)
                        .r(rng.f64())
                        .opacity(rng.f64())
                        .jitter(rng.f64());
                    if rng.chance(0.3) {
                        d = d.fill_by(heat, rng.values(n + 1));
                    }
                    if rng.chance(0.3) {
                        d = d.size_by(y, rng.values(n + 1));
                    }
                    if rng.chance(0.2) {
                        d = d.dodge();
                    }
                    panel.mark(d.fill(color, "b"));
                }
                1 => {
                    let curve = *rng.pick(&[
                        Curve::Linear,
                        Curve::MonotoneX,
                        Curve::Natural,
                        Curve::Basis,
                        Curve::CatmullRom { alpha: 0.5 },
                        Curve::Step,
                    ]);
                    panel.mark(
                        mark::line(points(rng, 40), x, y)
                            .width(rng.f64())
                            .dash(rng.values(3))
                            .curve(curve),
                    );
                }
                2 => {
                    panel.mark(
                        mark::area(points(rng, 30), x, y)
                            .base(rng.f64())
                            .opacity(rng.f64()),
                    );
                }
                3 => {
                    let rows: Vec<_> = points(rng, 20)
                        .into_iter()
                        .map(|(a, b)| (a, b, rng.f64()))
                        .collect();
                    panel.mark(mark::ribbon(rows, x, y));
                }
                4 => {
                    let rows: Vec<_> = rng
                        .values(5)
                        .into_iter()
                        .map(|v| (*rng.pick(&["p", "q", "r", "s"]), v))
                        .collect();
                    panel.mark(mark::bar_y(rows, band, y));
                }
                5 => {
                    let rows: Vec<_> = (0..rng.small(6))
                        .map(|_| ([rng.f64(), rng.f64()], [rng.f64(), rng.f64()]))
                        .collect();
                    panel.mark(mark::rect(rows, x, y));
                }
                6 => {
                    let bins = if rng.chance(0.5) {
                        Bins::Nice(rng.count())
                    } else {
                        Bins::Count(rng.count())
                    };
                    panel.mark(mark::histogram(rng.values(40), x, y).bins(bins));
                }
                7 => {
                    panel.mark(mark::density_y(rng.values(30), y, x));
                }
                8 => {
                    let cols = rng.small(5);
                    let rows = rng.small(5);
                    let values = (0..cols * rows).map(|_| rng.f64()).collect();
                    if let Ok(grid) = mark::Grid::new(cols, rows, values) {
                        let grid = grid.extent([rng.f64(), rng.f64()], [rng.f64(), rng.f64()]);
                        panel.mark(mark::image(grid, x, y, heat));
                    }
                }
                9 => {
                    panel.mark(
                        mark::note((rng.f64(), rng.f64()), rng.string("ab$ ", 6), x, y)
                            .offset(rng.f64(), rng.f64()),
                    );
                }
                10 => {
                    panel.mark(
                        mark::rule_x(rng.f64(), x)
                            .width(rng.f64())
                            .dash(rng.values(2)),
                    );
                    panel.mark(mark::rule_y(rng.f64(), y).label("r"));
                }
                11 => {
                    panel.mark(mark::dots(points(rng, 10), logx, y));
                }
                _ => {
                    panel.mark(mark::line(points(rng, 10), logx, y));
                }
            }
        }
        if rng.chance(0.7) {
            panel.axis(
                axis::bottom(x)
                    .ticks(rng.count())
                    .rotate(rng.f64())
                    .minor(rng.chance(0.5))
                    .outward(rng.f64())
                    .label("x"),
            );
        }
        if rng.chance(0.7) {
            let mut left = axis::left(y).ticks(rng.count());
            if rng.chance(0.3) {
                left = left.values(rng.values(6));
            }
            panel.axis(left.offset(rng.chance(0.5)));
        }
        if rng.chance(0.2) {
            panel.axis(axis::top(logx).ticks(rng.count()));
        }
        if rng.chance(0.2) {
            panel.axis(axis::right(band));
        }
        if rng.chance(0.2) {
            panel.legend(Inset::TopLeft, legend(color));
        }
        if rng.chance(0.2) {
            panel.title(rng.string("t ", 5));
        }
        if rng.chance(0.2) {
            panel.zoom_x(x).zoom_y(y);
        }
    }
    if rng.chance(0.2) {
        fig.legend(
            Slot::Right,
            colorbar(heat).ticks(rng.count()).extend(Extend::Both),
        );
    }
    if rng.chance(0.2) {
        fig.legend(Slot::Bottom, legend(color).title("k"));
    }
    fig
}

#[test]
fn random_figures_check_draw_and_write_without_panicking() {
    hostile::run("figure", 0xF1_6E, CASES, |rng| {
        let fig = figure(rng);
        let size = size(rng);
        fig.check(size);
        let list = fig.draw_with(&mut cache(), size, Target::Vector, &Held::default());
        if let Ok(lowered) = lower::lower(&list, svg::CAPS) {
            let _ = svg::write(&lowered.list, &svg::Options::default());
        }
        #[cfg(feature = "raster")]
        {
            let scale = if rng.chance(0.8) { 1.0 } else { rng.f64() };
            let list = fig.draw_with(&mut cache(), size, Target::File { scale }, &Held::default());
            let bounds = Bounds::of(&list, &mut Outlines::default());
            let area = Rect::from_origin_size(Point::ORIGIN, size);
            let _ =
                rustd3_figure::raster::Renderer::new(rustd3_figure::raster::Settings::default())
                    .render(&list, &bounds, area, scale);
        }
        #[cfg(feature = "pdf")]
        {
            let _ = rustd3_figure::pdf::write(&list, &rustd3_figure::pdf::Options::default());
        }
    });
}

#[test]
fn hostile_rgb_inputs_never_panic_a_figure() {
    hostile::run("figure colors", 0xC0_10, 50, |rng| {
        let mut fig = Figure::new();
        let x = fig.fit(scale::linear());
        let y = fig.fit(scale::linear());
        let c = Rgb::new(rng.f64(), rng.f64(), rng.f64()).with_opacity(rng.f64());
        fig.panel(0, 0)
            .mark(mark::line(points(rng, 5), x, y).color(c))
            .mark(mark::dots(points(rng, 5), x, y).color(c))
            .border(c);
        let list = fig.draw_with(
            &mut cache(),
            Size::new(200.0, 150.0),
            Target::Vector,
            &Held::default(),
        );
        let _ = svg::write(&list, &svg::Options::default());
    });
}
