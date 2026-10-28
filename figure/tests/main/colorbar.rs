//! Colorbars: strips equal to their scales at every pixel, bars along
//! figure slots and in panel corners, and the room slots take from the
//! grid.

use rustd3_core::color::Rgb;
use rustd3_core::scale;
#[cfg(feature = "raster")]
use rustd3_core::scale::Sequential;
use rustd3_core::scheme;
use rustd3_figure::colorbar::{Ramp, colorbar};
use rustd3_figure::kurbo::{Rect, Size};
use rustd3_figure::legend::{Inset, Slot};
use rustd3_figure::list::{Command, List};
use rustd3_figure::peniko::ImageData;
use rustd3_figure::scale::{Paint, ScaleRef};
use rustd3_figure::{Figure, Held, Target, axis};

use crate::figure::{cache, line};

/// The tick-label size at notebook, px: one em.
const EM: f64 = 44.0 / 3.0;

/// The canvas padding, px.
const PAD: f64 = 0.8 * EM;

/// A one-panel figure, x and y fitted to 0 … 10 with axes, and a color
/// handle of `color`.
fn panel<S: Paint<Domain = f64> + Ramp>(color: S) -> (Figure<'static>, ScaleRef<S>) {
    let mut fig = Figure::new();
    let x = fig.fit(scale::linear());
    let y = fig.fit(scale::linear());
    let c = fig.scale(color);
    fig.panel(0, 0)
        .mark(line(x, [0.0, 10.0]))
        .mark(line(y, [0.0, 10.0]))
        .axis(axis::left(y))
        .axis(axis::bottom(x));
    (fig, c)
}

/// `fig` drawn at `size` for `target`, and its first panel's plot area.
fn drawn(fig: &Figure<'_>, size: Size, target: Target) -> (List, Rect) {
    let held = Held::default();
    let area = fig.resolve(size, &mut cache(), &held, target).plot_areas()[0];
    (fig.draw_with(&mut cache(), size, target, &held), area)
}

/// The strips drawn: each `Image` command's rect and pixels.
fn strips(list: &List) -> Vec<(Rect, &ImageData)> {
    list.commands
        .iter()
        .filter_map(|c| {
            let Command::Image {
                rect,
                image,
                smooth,
            } = c
            else {
                return None;
            };
            assert!(!smooth, "strips are nearest-sampled");
            Some((*rect, image))
        })
        .collect()
}

#[test]
fn a_right_colorbar_spans_the_plot_area_one_em_thick_outside_the_grid() {
    let size = Size::new(640.0, 480.0);
    let (fig, _) = panel(scale::sequential([0.0, 80.0], scheme::viridis));
    let (_, bare) = drawn(&fig, size, Target::Vector);
    let (mut fig, heat) = panel(scale::sequential([0.0, 80.0], scheme::viridis));
    fig.legend(Slot::Right, colorbar(heat));
    let (list, area) = drawn(&fig, size, Target::Vector);
    let [(bar, image)] = strips(&list)[..] else {
        panic!("one strip");
    };
    assert_eq!((image.width, image.height), (1, 256));
    assert!((bar.x1 - (640.0 - PAD)).abs() < 1e-9, "{bar:?}");
    assert!((bar.width() - EM).abs() < 1e-9);
    assert_eq!((bar.y0, bar.y1), (area.y0, area.y1));
    // The slot takes 0.5 em and the bar's em from the grid.
    assert!(
        (bare.x1 - area.x1 - 1.5 * EM).abs() < 1e-9,
        "{bare:?} {area:?}"
    );
}

#[test]
fn top_and_bottom_colorbars_lie_across_the_plot_areas() {
    let size = Size::new(640.0, 480.0);
    let (mut fig, heat) = panel(scale::sequential([0.0, 80.0], scheme::magma));
    fig.legend(Slot::Bottom, colorbar(heat))
        .legend(Slot::Top, colorbar(heat));
    let (list, area) = drawn(&fig, size, Target::Vector);
    let [(bottom, across), (top, up)] = strips(&list)[..] else {
        panic!("two strips");
    };
    for image in [up, across] {
        assert_eq!((image.width, image.height), (256, 1));
    }
    assert!((bottom.y1 - (480.0 - PAD)).abs() < 1e-9, "{bottom:?}");
    assert!((bottom.height() - EM).abs() < 1e-9);
    assert_eq!((top.x0, top.x1), (area.x0, area.x1));
    assert_eq!((bottom.x0, bottom.x1), (area.x0, area.x1));
    assert!(top.y1 < area.y0 && area.y1 < bottom.y0);
}

#[test]
fn a_side_colorbar_spans_every_row_beside_it() {
    let mut fig = Figure::new();
    let heat = fig.scale(scale::sequential([0.0, 80.0], scheme::viridis));
    fig.panel(0, 0);
    fig.panel(1, 0);
    fig.legend(Slot::Left, colorbar(heat));
    let size = Size::new(640.0, 480.0);
    let held = Held::default();
    let areas = fig
        .resolve(size, &mut cache(), &held, Target::Vector)
        .plot_areas()
        .to_vec();
    let list = fig.draw_with(&mut cache(), size, Target::Vector, &held);
    let [(bar, _)] = strips(&list)[..] else {
        panic!("one strip");
    };
    assert_eq!((bar.y0, bar.y1), (areas[0].y0, areas[1].y1));
    assert!((bar.x0 - PAD).abs() < 1e-9);
}

#[test]
fn an_inset_colorbar_runs_along_the_longer_side_for_40_percent_of_it() {
    let (mut fig, heat) = panel(scale::sequential([0.0, 80.0], scheme::viridis));
    fig.panel(0, 0).legend(Inset::TopRight, colorbar(heat));
    let (list, area) = drawn(&fig, Size::new(640.0, 480.0), Target::Vector);
    let [(bar, image)] = strips(&list)[..] else {
        panic!("one strip");
    };
    // Wider than tall: a horizontal bar, 0.4 of the width, framed 0.4 em
    // inside and 0.5 em from the corner.
    assert_eq!((image.width, image.height), (256, 1));
    assert!((bar.width() - 0.4 * area.width()).abs() < 1e-9);
    assert!((bar.height() - EM).abs() < 1e-9);
    assert!((bar.x1 - (area.x1 - 0.9 * EM)).abs() < 1e-9);
    assert!((bar.y0 - (area.y0 + 0.9 * EM)).abs() < 1e-9);
    let (mut fig, heat) = panel(scale::sequential([0.0, 80.0], scheme::viridis));
    fig.panel(0, 0).legend(Inset::BottomLeft, colorbar(heat));
    let (list, area) = drawn(&fig, Size::new(400.0, 640.0), Target::Vector);
    let [(bar, image)] = strips(&list)[..] else {
        panic!("one strip");
    };
    assert_eq!((image.width, image.height), (1, 256));
    assert!((bar.height() - 0.4 * area.height()).abs() < 1e-9);
    assert!((bar.x0 - (area.x0 + 0.9 * EM)).abs() < 1e-9);
}

#[test]
fn binned_colorbars_draw_one_row_per_equal_bin() {
    let blues5 = ["#eff3ff", "#bdd7e7", "#6baed6", "#3182bd", "#08519c"]
        .map(|c| c.parse::<Rgb>().expect("a color"));
    let (mut fig, q) = panel(scale::quantize([0.0, 100.0], blues5));
    let red = Rgb::new(255.0, 0.0, 0.0);
    let blue = Rgb::new(0.0, 0.0, 255.0);
    let white = Rgb::new(255.0, 255.0, 255.0);
    let t = fig.scale(scale::threshold([0.0, 1.0], [red, white, blue]));
    fig.legend(Slot::Right, colorbar(q))
        .legend(Slot::Right, colorbar(t));
    let (list, _) = drawn(&fig, Size::new(640.0, 480.0), Target::Vector);
    let [(near, quantize), (far, threshold)] = strips(&list)[..] else {
        panic!("two strips");
    };
    assert_eq!((quantize.width, quantize.height), (1, 5));
    assert_eq!((threshold.width, threshold.height), (1, 3));
    // The first row is the high end: blues5's darkest, then white's row.
    assert_eq!(&quantize.data.data()[..4], [0x08, 0x51, 0x9c, 255]);
    assert_eq!(&threshold.data.data()[4..8], [255, 255, 255, 255]);
    assert!((far.x0 - near.x1 - 0.5 * EM).abs() < 1e-9);
}

/// `fig` sized so that its first plot area is `length` px long along
/// `slot` (tall beside left and right slots, wide beside top and bottom
/// ones) when drawn for a PNG at 1×: the canvas and that area.
#[cfg(feature = "raster")]
fn sized(fig: &Figure<'_>, slot: Slot, length: f64) -> (Size, Rect) {
    let target = Target::File { scale: 1.0 };
    let vertical = matches!(slot, Slot::Left | Slot::Right);
    let mut size = Size::new(length + 160.0, length + 120.0);
    for _ in 0..4 {
        let area = fig
            .resolve(size, &mut cache(), &Held::default(), target)
            .plot_areas()[0];
        let along = if vertical {
            area.height()
        } else {
            area.width()
        };
        if along == length {
            return (size, area);
        }
        if vertical {
            size.height += length - along;
        } else {
            size.width += length - along;
        }
    }
    panic!("no canvas gives a plot area {length} px long");
}

/// A colorbar of `color` drawn in `slot` beside a panel 300 px long along
/// it, at 1× by vello_cpu: each pixel's RGBA along the bar's middle from
/// its low end (bottom or left).
#[cfg(feature = "raster")]
fn rendered<S: Paint<Domain = f64> + Ramp>(color: S, slot: Slot) -> Vec<[u8; 4]> {
    let (mut fig, heat) = panel(color);
    fig.legend(slot, colorbar(heat));
    let (size, area) = sized(&fig, slot, 300.0);
    let target = Target::File { scale: 1.0 };
    let list = fig.draw_with(&mut cache(), size, target, &Held::default());
    let [(bar, _)] = strips(&list)[..] else {
        panic!("one strip");
    };
    let vertical = matches!(slot, Slot::Left | Slot::Right);
    if vertical {
        assert_eq!((bar.y0, bar.y1), (area.y0, area.y1));
    } else {
        assert_eq!((bar.x0, bar.x1), (area.x0, area.x1));
    }
    let image = crate::raster::vello::whole(&list, 1.0);
    let middle = |a: f64, b: f64| ((a + b) / 2.0).floor() as u32;
    (0..300)
        .map(|j| {
            let (x, y) = if vertical {
                (middle(bar.x0, bar.x1), bar.y1 as u32 - 1 - j)
            } else {
                (bar.x0 as u32 + j, middle(bar.y0, bar.y1))
            };
            let at = ((y * image.width + x) * 4) as usize;
            [0, 1, 2, 3].map(|k| image.rgba[at + k])
        })
        .collect()
}

/// `c`'s channels as 8 bits, as a strip stores them.
#[cfg(feature = "raster")]
fn rgba(c: Rgb) -> [u8; 4] {
    let c = c.clamp();
    [
        c.r as u8,
        c.g as u8,
        c.b as u8,
        (c.opacity * 255.0).round() as u8,
    ]
}

#[cfg(feature = "raster")]
#[test]
fn a_table_strip_equals_its_scale_at_every_pixel_of_a_300_px_bar() {
    for (name, table) in [
        ("viridis", scheme::viridis as fn(f64) -> Rgb),
        ("magma", scheme::magma),
    ] {
        let s: Sequential<Rgb> = scale::sequential([0.0, 80.0], table);
        for slot in [Slot::Right, Slot::Bottom] {
            let pixels = rendered(s.clone(), slot);
            let wrong: Vec<usize> = (0..300)
                .filter(|&j| pixels[j] != rgba(s.apply((j as f64 + 0.5) / 300.0 * 80.0)))
                .collect();
            assert_eq!(wrong, [0_usize; 0], "{name} {slot:?}");
            if name == "viridis" {
                assert_eq!(pixels[150], [0x21, 0x91, 0x8c, 255]);
                assert_eq!(pixels[0], [0x44, 0x01, 0x54, 255]);
                assert_eq!(pixels[299], [0xfd, 0xe7, 0x25, 255]);
            }
        }
    }
}

#[cfg(feature = "raster")]
#[test]
fn an_interpolator_strip_shows_the_interpolator_at_each_row_centre() {
    for interpolator in [scheme::turbo as fn(f64) -> Rgb, scheme::rd_bu] {
        for slot in [Slot::Left, Slot::Top] {
            let pixels = rendered(scale::sequential([-3.0, 5.0], interpolator), slot);
            let wrong: Vec<usize> = (0..300)
                .filter(|&j| {
                    let row = ((j as f64 + 0.5) / 300.0 * 256.0).floor();
                    pixels[j] != rgba(interpolator((row + 0.5) / 256.0))
                })
                .collect();
            assert_eq!(wrong, [0_usize; 0], "{slot:?}");
        }
    }
}
